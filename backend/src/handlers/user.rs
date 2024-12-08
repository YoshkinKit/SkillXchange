use actix_web::{HttpMessage, HttpResponse, web};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use crate::middleware::roles::Role;
use crate::models::user::User;

#[derive(serde::Serialize)]
struct UserResponse {
    user_id: i32,
    username: String,
    email: String,
    role: String,
    profile_picture: Option<String>,
    bio: Option<String>,
    created_at: DateTime<Utc>,
}

pub async fn get_all_users(pool: web::Data<PgPool>) -> HttpResponse {
    let result = sqlx::query!(
        r#"SELECT user_id, username, email, role, profile_picture, bio, created_at FROM users"#
    )
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(users) => {
            let users = users.into_iter().map(|user| UserResponse {
                user_id: user.user_id,
                username: user.username,
                email: user.email,
                role: user.role,
                profile_picture: user.profile_picture,
                bio: user.bio,
                created_at: user.created_at.expect("1970-01-01T00:00:00Z"),
            }).collect::<Vec<_>>();
            HttpResponse::Ok().json(users)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка базы данных: {}", e)),
    }
}

pub async fn get_user_by_id(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> HttpResponse {
    let result = sqlx::query!(
        r#"SELECT user_id, username, email, role, profile_picture, bio, created_at FROM users WHERE user_id = $1"#,
        *user_id
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(user) => {
            let user = UserResponse {
                user_id: user.user_id,
                username: user.username,
                email: user.email,
                role: user.role,
                profile_picture: user.profile_picture,
                bio: user.bio,
                created_at: user.created_at.expect("1970-01-01T00:00:00Z"),
            };
            HttpResponse::Ok().json(user)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("Пользователь не найден"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

#[derive(serde::Deserialize, Serialize)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub profile_picture: Option<String>,
    pub bio: Option<String>,
}

pub async fn update_user(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
    data: web::Json<UpdateUser>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let auth_user_id = req.extensions().get::<i32>().cloned();
    let role = req.extensions().get::<Role>().cloned().unwrap_or(Role::Guest);

    if auth_user_id != Some(*user_id) && role != Role::Admin {
        return HttpResponse::Forbidden().body("Доступ запрещен");
    }

    let user_result = sqlx::query!(
        r#"
        SELECT
            user_id,
            username,
            email,
            password_hash,
            role,
            profile_picture,
            bio,
            created_at as "created_at!"
        FROM users
        WHERE user_id = $1
        "#,
        *user_id
    )
        .fetch_one(pool.get_ref())
        .await;

    let mut user = match user_result {
        Ok(record) => User {
            user_id: record.user_id,
            username: record.username,
            email: record.email,
            password_hash: record.password_hash,
            role: record.role,
            profile_picture: record.profile_picture,
            bio: record.bio,
            created_at: record.created_at,
        },
        Err(_) => return HttpResponse::NotFound().body("Пользователь не найден"),
    };

    if let Some(username) = &data.username {
        user.username = username.clone();
    }
    if let Some(email) = &data.email {
        user.email = email.clone();
    }
    if let Some(profile_picture) = &data.profile_picture {
        user.profile_picture = Some(profile_picture.clone());
    }
    if let Some(bio) = &data.bio {
        user.bio = Some(bio.clone());
    }

    let result = sqlx::query!(
        r#"
        UPDATE users SET
            username = $1,
            email = $2,
            profile_picture = $3,
            bio = $4
        WHERE user_id = $5
        "#,
        user.username,
        user.email,
        user.profile_picture,
        user.bio,
        user.user_id
    )
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Пользователь обновлен"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn delete_user(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let auth_user_id = req.extensions().get::<i32>().cloned();
    let role = req.extensions().get::<Role>().cloned().unwrap_or(Role::Guest);

    if auth_user_id != Some(*user_id) && role != Role::Admin {
        return HttpResponse::Forbidden().body("Доступ запрещен");
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Ошибка начала транзакции: {}", e)),
    };

    if let Err(e) = sqlx::query!(
        r#"DELETE FROM refresh_tokens WHERE user_id = $1"#,
        *user_id
    )
        .execute(&mut *tx)
        .await
    {
        return HttpResponse::InternalServerError().body(format!("Ошибка удаления токенов: {}", e));
    }

    if let Err(e) = sqlx::query!(
        r#"DELETE FROM users WHERE user_id = $1"#,
        *user_id
    )
        .execute(&mut *tx)
        .await
    {
        return HttpResponse::InternalServerError().body(format!("Ошибка удаления пользователя: {}", e));
    }

    match tx.commit().await {
        Ok(_) => HttpResponse::Ok().body("Пользователь удален"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка фиксации транзакции: {}", e)),
    }
}

#[derive(serde::Deserialize, Serialize)]
pub struct AddUserSkill {
    pub skill_id: i32,
}

pub async fn get_users_by_skill(
    pool: web::Data<PgPool>,
    skill_id: web::Path<i32>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let role = req.extensions().get::<Role>().cloned().unwrap_or(Role::Guest);

    if role != Role::Admin && role != Role::User {
        return HttpResponse::Forbidden().body("Доступ запрещен");
    }

    let result = sqlx::query!(
        r#"
        SELECT u.user_id, u.username, u.email, u.role, u.profile_picture, u.bio, u.created_at
        FROM users u
        JOIN user_skills us ON u.user_id = us.user_id
        WHERE us.skill_id = $1
        "#,
        *skill_id
    )
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(users) => {
            let users = users.into_iter().map(|user| UserResponse {
                user_id: user.user_id,
                username: user.username,
                email: user.email,
                role: user.role,
                profile_picture: user.profile_picture,
                bio: user.bio,
                created_at: user.created_at.expect("1970-01-01T00:00:00Z"),
            }).collect::<Vec<_>>();
            HttpResponse::Ok().json(users)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn add_user_skill(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
    skill: web::Json<AddUserSkill>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let auth_user_id = req.extensions().get::<i32>().cloned();
    let role = req.extensions().get::<Role>().cloned().unwrap_or(Role::Guest);

    if auth_user_id != Some(*user_id) && role != Role::Admin {
        return HttpResponse::Forbidden().body("Доступ запрещен");
    }

    let result = sqlx::query!(
        r#"
        INSERT INTO user_skills (user_id, skill_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id, skill_id) DO NOTHING
        "#,
        *user_id,
        skill.skill_id
    )
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Навык добавлен"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn delete_user_skill(
    pool: web::Data<PgPool>,
    params: web::Path<(i32, i32)>, // (user_id, skill_id)
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let (user_id, skill_id) = params.into_inner();
    let auth_user_id = req.extensions().get::<i32>().cloned();
    let role = req.extensions().get::<Role>().cloned().unwrap_or(Role::Guest);

    if auth_user_id != Some(user_id) && role != Role::Admin {
        return HttpResponse::Forbidden().body("Доступ запрещен");
    }

    let result = sqlx::query!(
        r#"DELETE FROM user_skills WHERE user_id = $1 AND skill_id = $2"#,
        user_id,
        skill_id
    )
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Навык удален"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use actix_http::Request;
    use actix_web::{
        App, dev::{Service, ServiceResponse},
        http::StatusCode,
        test,
        web,
    };
    use actix_web_httpauth::middleware::HttpAuthentication;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use sqlx::{Pool, Postgres};

    use crate::middleware::auth::jwt_validator;
    use crate::middleware::roles::{RequireRole, Role};

    use super::*;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct UserResponse {
        user_id: i32,
        username: String,
        email: String,
        role: String,
        profile_picture: Option<String>,
        bio: Option<String>,
        created_at: DateTime<Utc>,
    }

    #[derive(serde::Serialize)]
    struct TestClaims {
        sub: i32,
        role: String,
        exp: usize,
    }

    // Хелпер для создания тестового JWT
    fn generate_test_token(user_id: i32, role: Role) -> String {
        let exp = (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize;

        let role = match role {
            Role::Admin => "admin",
            Role::User => "user",
            Role::Guest => "guest",
        };

        let claims = TestClaims {
            sub: user_id,
            role: role.to_string(),
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("TestSecret".as_ref()),
        ).unwrap()
    }

    async fn create_test_app(
        pool: Pool<Postgres>,
        user_id: Option<i32>,
        role: Option<Role>,
    ) -> impl Service<
        Request,
        Response=ServiceResponse,
        Error=actix_web::Error,
    > {
        std::env::set_var("ACCESS_TOKEN_SECRET", "TestSecret");

        let auth_middleware = HttpAuthentication::bearer(jwt_validator);

        let app = App::new()
            .app_data(web::Data::new(pool))
            .service(
                web::scope("/api/users")
                    .wrap(auth_middleware.clone())
                    .wrap(RequireRole::new(Role::User))
                    .route("", web::get().to(get_all_users))
                    .route("/{id}", web::get().to(get_user_by_id))
                    .route("/{id}", web::put().to(update_user))
                    .route("/{id}", web::delete().to(delete_user))
                    .route("/skill/{id}", web::get().to(get_users_by_skill))
                    .route("/{id}/skills", web::post().to(add_user_skill))
                    .route("/{id}/skills/{skill_id}", web::delete().to(delete_user_skill)),
            )
            .wrap_fn(move |req, srv| {
                if let Some(id) = user_id {
                    req.extensions_mut().insert(id);
                }
                if let Some(r) = role.clone() {
                    req.extensions_mut().insert(r);
                }
                srv.call(req)
            });

        test::init_service(app).await
    }

    async fn clean_db(pool: &PgPool) {
        let mut tx = pool.begin().await.unwrap();

        // Отключаем проверку внешних ключей на время очистки
        sqlx::query!("SET CONSTRAINTS ALL DEFERRED")
            .execute(&mut *tx)
            .await
            .unwrap();

        // Очищаем все таблицы
        sqlx::query!("TRUNCATE TABLE users, user_skills, reviews, requests, categories, messages, refresh_tokens CASCADE")
            .execute(&mut *tx)
            .await
            .unwrap();

        tx.commit().await.unwrap();
    }

    #[sqlx::test]
    async fn test_get_all_users() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str())
            .await
            .unwrap();

        clean_db(&pool).await;

        // Подготовка тестовых данных
        sqlx::query!(
            "INSERT INTO users (username, email, password_hash, role)
             VALUES ($1, $2, $3, $4)",
            "test_user",
            "test@test.com",
            "hash",
            "user"
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool, Some(1), Some(Role::User)).await;
        let token = generate_test_token(1, Role::User);

        let req = test::TestRequest::get()
            .uri("/api/users")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: Vec<UserResponse> = test::read_body_json(resp).await;
        assert!(!body.is_empty());
        assert_eq!(body[0].username, "test_user");
    }

    #[sqlx::test]
    async fn test_get_all_users_unauthorized() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str())
            .await
            .unwrap();

        clean_db(&pool).await;

        let app = create_test_app(pool, None, None).await;

        // Запрос без токена
        let req = test::TestRequest::get()
            .uri("/api/users")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test]
    async fn test_get_user_by_id() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str())
            .await
            .unwrap();
        clean_db(&pool).await;

        // Создаем тестового пользователя
        sqlx::query!(
            "INSERT INTO users (user_id, username, email, password_hash, role)
             VALUES ($1, $2, $3, $4, $5)",
            1, "test_user", "test@test.com", "hash", "user"
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool, Some(1), Some(Role::User)).await;
        let token = generate_test_token(1, Role::User);

        let req = test::TestRequest::get()
            .uri("/api/users/1")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let user: UserResponse = test::read_body_json(resp).await;
        assert_eq!(user.username, "test_user");
    }

    #[sqlx::test]
    async fn test_update_user_success() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str())
            .await
            .unwrap();
        clean_db(&pool).await;

        // Создаем пользователя
        sqlx::query!(
            "INSERT INTO users (user_id, username, email, password_hash, role)
             VALUES ($1, $2, $3, $4, $5)",
            1, "old_name", "old@test.com", "hash", "user"
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool.clone(), Some(1), Some(Role::User)).await;
        let token = generate_test_token(1, Role::User);

        let update_data = UpdateUser {
            username: Some("new_name".to_string()),
            email: Some("new@test.com".to_string()),
            profile_picture: None,
            bio: Some("New bio".to_string()),
        };

        let req = test::TestRequest::put()
            .uri("/api/users/1")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&update_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Проверяем, что данные обновились
        let updated_user = sqlx::query!("SELECT * FROM users WHERE user_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(updated_user.username, "new_name");
        assert_eq!(updated_user.email, "new@test.com");
        assert_eq!(updated_user.bio.unwrap(), "New bio");
    }

    #[sqlx::test]
    async fn test_update_user_forbidden() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str())
            .await
            .unwrap();

        clean_db(&pool).await;

        sqlx::query!(
            "INSERT INTO users (user_id, username, email, password_hash, role)
             VALUES ($1, $2, $3, $4, $5)",
            1, "user1", "user1@test.com", "hash", "user"
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool, Some(2), Some(Role::User)).await;
        let token = generate_test_token(2, Role::User);

        let update_data = UpdateUser {
            username: Some("hacker".to_string()),
            email: None,
            profile_picture: None,
            bio: None,
        };

        let req = test::TestRequest::put()
            .uri("/api/users/1")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&update_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test]
    async fn test_delete_user_success() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str())
            .await
            .unwrap();
        clean_db(&pool).await;

        sqlx::query!(
            "INSERT INTO users (user_id, username, email, password_hash, role)
             VALUES ($1, $2, $3, $4, $5)",
            1, "to_delete", "delete@test.com", "hash", "user"
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool.clone(), Some(1), Some(Role::User)).await;
        let token = generate_test_token(1, Role::User);

        let req = test::TestRequest::delete()
            .uri("/api/users/1")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Проверяем что пользователь удален
        let user_exists = sqlx::query!("SELECT COUNT(*) as count FROM users WHERE user_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap()
            .count
            .unwrap();

        assert_eq!(user_exists, 0);
    }

    #[sqlx::test]
    async fn test_add_user_skill() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str())
            .await
            .unwrap();
        clean_db(&pool).await;

        // Создаем категорию
        sqlx::query!(
            "INSERT INTO categories (category_id, title, description)
             VALUES ($1, $2, $3)",
            1, "Programming", "Programming skills"
        )
            .execute(&pool)
            .await
            .unwrap();

        // Создаем пользователя и навык
        sqlx::query!(
            "INSERT INTO users (user_id, username, email, password_hash, role)
             VALUES ($1, $2, $3, $4, $5)",
            1, "user1", "user1@test.com", "hash", "user"
        )
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query!(
            "INSERT INTO skills (skill_id, title, description, category_id)
             VALUES ($1, $2, $3, $4)",
            1, "Rust", "Programming", 1
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool.clone(), Some(1), Some(Role::User)).await;
        let token = generate_test_token(1, Role::User);

        let add_skill = AddUserSkill { skill_id: 1 };

        let req = test::TestRequest::post()
            .uri("/api/users/1/skills")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&add_skill)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Проверяем что навык добавлен
        let skill_exists = sqlx::query!(
            "SELECT COUNT(*) as count FROM user_skills WHERE user_id = 1 AND skill_id = 1"
        )
            .fetch_one(&pool)
            .await
            .unwrap()
            .count
            .unwrap();

        assert_eq!(skill_exists, 1);
    }

    #[sqlx::test]
    async fn test_delete_user_skill() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str())
            .await
            .unwrap();

        clean_db(&pool).await;

        sqlx::query!(
            "INSERT INTO categories (category_id, title, description)
             VALUES ($1, $2, $3)",
            1, "Programming", "Programming skills"
        )
            .execute(&pool)
            .await
            .unwrap();

        // Подготовка данных
        sqlx::query!(
            "INSERT INTO users (user_id, username, email, password_hash, role)
             VALUES ($1, $2, $3, $4, $5)",
            1, "user1", "user1@test.com", "hash", "user"
        )
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query!(
            "INSERT INTO skills (skill_id, title, description, category_id)
             VALUES ($1, $2, $3, $4)",
            1, "Rust", "Programming", 1
        )
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query!(
            "INSERT INTO user_skills (user_id, skill_id) VALUES ($1, $2)",
            1, 1
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool.clone(), Some(1), Some(Role::User)).await;
        let token = generate_test_token(1, Role::User);

        let req = test::TestRequest::delete()
            .uri("/api/users/1/skills/1")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Проверяем что навык удален
        let skill_exists = sqlx::query!(
            "SELECT COUNT(*) as count FROM user_skills WHERE user_id = 1 AND skill_id = 1"
        )
            .fetch_one(&pool)
            .await
            .unwrap()
            .count
            .unwrap();

        assert_eq!(skill_exists, 0);
    }
}