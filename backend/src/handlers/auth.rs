use actix_web::{HttpResponse, web};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, DecodingKey, encode, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use regex::Regex;
use serde_json::json;

fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

#[derive(Deserialize, Serialize)]
pub struct RegisterInfo {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(Deserialize, Serialize)]
pub struct RegisterResponse {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub role: String,
    pub created_at: chrono::DateTime<Utc>,
}

pub async fn register_user(
    pool: web::Data<PgPool>,
    form: web::Json<RegisterInfo>,
) -> HttpResponse {
    if !is_valid_email(&form.email) {
        return HttpResponse::BadRequest().json(json!({
            "message": "Неверный формат email"
        }));
    }

    let existing_user = sqlx::query!(
        "SELECT username, email FROM users WHERE username = $1 OR email = $2",
        form.username,
        form.email
    )
    .fetch_optional(pool.get_ref())
    .await;

    match existing_user {
        Ok(Some(user)) => {
            if user.username == form.username {
                return HttpResponse::BadRequest().json(json!({
                    "message": "Пользователь с таким именем уже существует"
                }));
            } else {
                return HttpResponse::BadRequest().json(json!({
                    "message": "Пользователь с таким email уже существует"
                }));
            }
        }
        Ok(None) => {
            let password_hash = match hash(&form.password, DEFAULT_COST) {
                Ok(hash) => hash,
                Err(_) => return HttpResponse::InternalServerError().json(json!({
                    "message": "Ошибка при создании пользователя"
                })),
            };

            let result = sqlx::query!(
                r#"
                INSERT INTO users (username, email, password_hash, role)
                VALUES ($1, $2, $3, $4)
                RETURNING user_id, username, email, role, created_at
                "#,
                form.username,
                form.email,
                password_hash,
                form.role
            )
            .fetch_one(pool.get_ref())
            .await;

            match result {
                Ok(user) => HttpResponse::Ok().json(RegisterResponse {
                    user_id: user.user_id,
                    username: user.username,
                    email: user.email,
                    role: user.role,
                    created_at: user.created_at.expect("REASON"),
                }),
                Err(_) => HttpResponse::InternalServerError().json(json!({
                    "message": "Ошибка при создании пользователя"
                })),
            }
        }
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "message": "Ошибка сервера при проверке пользователя"
        })),
    }
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: i32,
    role: String,
    exp: usize,
}

#[derive(Deserialize, Serialize)]
pub struct LoginInfo {
    pub email: String,
    pub password: String,
}

pub async fn login_user(
    pool: web::Data<PgPool>,
    form: web::Json<LoginInfo>,
) -> HttpResponse {
    let result = sqlx::query!(
        r#"
        SELECT user_id, password_hash, role FROM users WHERE email = $1
        "#,
        form.email
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(record) => {
            if verify(&form.password, &record.password_hash).unwrap() {
                let access_exp = Utc::now()
                    .checked_add_signed(Duration::minutes(3))
                    .expect("Ошибка при установке времени")
                    .timestamp();
                let access_claims = Claims {
                    sub: record.user_id,
                    role: record.role.clone(),
                    exp: access_exp as usize,
                };
                let access_token = encode(
                    &Header::default(),
                    &access_claims,
                    &EncodingKey::from_secret(std::env::var("ACCESS_TOKEN_SECRET").unwrap().as_ref()),
                ).unwrap();

                let refresh_exp = Utc::now()
                    .checked_add_signed(Duration::days(30))
                    .expect("Ошибка при установке времени")
                    .timestamp();
                let refresh_claims = Claims {
                    sub: record.user_id,
                    role: record.role.clone(),
                    exp: refresh_exp as usize,
                };
                let refresh_token = encode(
                    &Header::default(),
                    &refresh_claims,
                    &EncodingKey::from_secret(std::env::var("REFRESH_TOKEN_SECRET").unwrap().as_ref()),
                ).unwrap();

                sqlx::query!(
                    "INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES ($1, $2, to_timestamp($3))",
                    record.user_id,
                    refresh_token,
                    refresh_exp as f64
                )
                    .execute(pool.get_ref())
                    .await
                    .unwrap();

                HttpResponse::Ok().json(serde_json::json!({
                    "access_token": access_token,
                    "refresh_token": refresh_token
                }))
            } else {
                HttpResponse::Unauthorized().body("Неверный пароль")
            }
        }
        Err(_) => HttpResponse::Unauthorized().body("Пользователь не найден"),
    }
}

pub async fn refresh_token(
    pool: web::Data<PgPool>,
    form: web::Json<serde_json::Value>,
) -> HttpResponse {
    let refresh_token = form["refresh_token"].as_str().unwrap_or("");

    let token_data = decode::<Claims>(
        refresh_token,
        &DecodingKey::from_secret(std::env::var("REFRESH_TOKEN_SECRET").unwrap().as_ref()),
        &Validation::default(),
    );

    match token_data {
        Ok(data) => {
            let user_id = data.claims.sub;

            let result = sqlx::query!(
                "SELECT * FROM refresh_tokens WHERE user_id = $1 AND token = $2 AND expires_at > NOW()",
                user_id,
                refresh_token
            )
                .fetch_one(pool.get_ref())
                .await;

            match result {
                Ok(_) => {
                    let access_exp = Utc::now()
                        .checked_add_signed(Duration::minutes(3))
                        .expect("Ошибка при установке времени")
                        .timestamp();
                    let access_claims = Claims {
                        sub: user_id,
                        role: data.claims.role,
                        exp: access_exp as usize,
                    };
                    let access_token = encode(
                        &Header::default(),
                        &access_claims,
                        &EncodingKey::from_secret(std::env::var("ACCESS_TOKEN_SECRET").unwrap().as_ref()),
                    ).unwrap();

                    HttpResponse::Ok().json(serde_json::json!({
                        "access_token": access_token
                    }))
                }
                Err(_) => HttpResponse::Unauthorized().body("Недействительный refresh-токен"),
            }
        }
        Err(_) => HttpResponse::Unauthorized().body("Ошибка верификации токена"),
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::*;
    use actix_web::{test, App, web};
    use sqlx::PgPool;

    async fn clean_db(pool: &PgPool) {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query!("TRUNCATE TABLE users, refresh_tokens CASCADE")
            .execute(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }

    #[test]
    async fn test_email_validation() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name@domain.co.uk"));
        assert!(!is_valid_email("invalid.email"));
        assert!(!is_valid_email("@domain.com"));
        assert!(!is_valid_email("user@.com"));
    }

    #[sqlx::test]
    async fn test_register_user_success() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str())
            .await
            .unwrap();
        clean_db(&pool).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(web::resource("/api/auth/register").route(web::post().to(register_user)))
        ).await;

        let register_data = RegisterInfo {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            role: "user".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/auth/register")
            .set_json(&register_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let user: RegisterResponse = test::read_body_json(resp).await;
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }

    #[sqlx::test]
    async fn test_register_user_invalid_email() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str())
            .await
            .unwrap();
        clean_db(&pool).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(web::resource("/api/auth/register").route(web::post().to(register_user)))
        ).await;

        let register_data = RegisterInfo {
            username: "testuser".to_string(),
            email: "invalid.email".to_string(),
            password: "password123".to_string(),
            role: "user".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/auth/register")
            .set_json(&register_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[sqlx::test]
    async fn test_login_user_success() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str())
            .await
            .unwrap();
        clean_db(&pool).await;

        // Создаем тестового пользователя
        let password = "password123";
        let hashed_password = hash(password, DEFAULT_COST).unwrap();

        sqlx::query!(
            "INSERT INTO users (username, email, password_hash, role) VALUES ($1, $2, $3, $4)",
            "testuser",
            "test@example.com",
            hashed_password,
            "user"
        )
            .execute(&pool)
            .await
            .unwrap();

        std::env::set_var("ACCESS_TOKEN_SECRET", "test_secret");
        std::env::set_var("REFRESH_TOKEN_SECRET", "test_refresh_secret");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(web::resource("/api/auth/login").route(web::post().to(login_user)))
        ).await;

        let login_data = LoginInfo {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/auth/login")
            .set_json(&login_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("access_token").is_some());
        assert!(body.get("refresh_token").is_some());
    }

    #[sqlx::test]
    async fn test_login_user_wrong_password() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str())
            .await
            .unwrap();
        clean_db(&pool).await;

        let password = "password123";
        let hashed_password = hash(password, DEFAULT_COST).unwrap();

        sqlx::query!(
            "INSERT INTO users (username, email, password_hash, role) VALUES ($1, $2, $3, $4)",
            "testuser",
            "test@example.com",
            hashed_password,
            "user"
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(web::resource("/api/auth/login").route(web::post().to(login_user)))
        ).await;

        let login_data = LoginInfo {
            email: "test@example.com".to_string(),
            password: "wrongpassword".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/auth/login")
            .set_json(&login_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test]
    async fn test_refresh_token() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str())
            .await
            .unwrap();
        clean_db(&pool).await;

        std::env::set_var("ACCESS_TOKEN_SECRET", "test_secret");
        std::env::set_var("REFRESH_TOKEN_SECRET", "test_refresh_secret");

        // Создаем тестового пользователя
        sqlx::query!(
            "INSERT INTO users (user_id, username, email, password_hash, role) VALUES ($1, $2, $3, $4, $5)",
            1, "testuser", "test@example.com", "hash", "user"
        )
            .execute(&pool)
            .await
            .unwrap();

        // Создаем тестовый refresh token
        let claims = Claims {
            sub: 1,
            role: "user".to_string(),
            exp: (Utc::now() + Duration::days(30)).timestamp() as usize,
        };

        let rt = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("test_refresh_secret".as_ref()),
        ).unwrap();

        sqlx::query!(
            "INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES ($1, $2, NOW() + INTERVAL '30 days')",
            1,
            rt
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(web::resource("/api/auth/refresh").route(web::post().to(refresh_token)))
        ).await;

        let refresh_data = serde_json::json!({
            "refresh_token": rt
        });

        let req = test::TestRequest::post()
            .uri("/api/auth/refresh")
            .set_json(&refresh_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("access_token").is_some());
    }
}