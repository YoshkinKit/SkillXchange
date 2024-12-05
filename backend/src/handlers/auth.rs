use actix_web::{HttpResponse, web};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, DecodingKey, encode, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::models::user::UserResponse;

#[derive(Deserialize)]
pub struct RegisterInfo {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub async fn register_user(
    pool: web::Data<PgPool>,
    form: web::Json<RegisterInfo>,
) -> HttpResponse {
    let password_hash = match hash(&form.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Ошибка хеширования: {}", e)),
    };

    let result = sqlx::query_as!(
        UserResponse,
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING user_id, username, email, created_at
        "#,
        form.username,
        form.email,
        password_hash
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка базы данных: {}", e)),
    }
}

// Объявляем структуры Claims
#[derive(Serialize, Deserialize)]
struct Claims {
    sub: i32,
    exp: usize,
}

#[derive(Deserialize)]
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
        SELECT user_id, password_hash FROM users WHERE email = $1
        "#,
        form.email
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(record) => {
            if verify(&form.password, &record.password_hash).unwrap() {
                // Генерация access-токена (живет 3 минуты)
                let access_exp = Utc::now()
                    .checked_add_signed(Duration::minutes(3))
                    .expect("Ошибка при установке времени")
                    .timestamp();
                let access_claims = Claims {
                    sub: record.user_id,
                    exp: access_exp as usize,
                };
                let access_token = encode(
                    &Header::default(),
                    &access_claims,
                    &EncodingKey::from_secret(std::env::var("ACCESS_TOKEN_SECRET").unwrap().as_ref()),
                ).unwrap();

                // Генерация refresh-токена (живёт 30 дней)
                let refresh_exp = Utc::now()
                    .checked_add_signed(Duration::days(30))
                    .expect("Ошибка при установке времени")
                    .timestamp();
                let refresh_claims = Claims {
                    sub: record.user_id,
                    exp: refresh_exp as usize,
                };
                let refresh_token = encode(
                    &Header::default(),
                    &refresh_claims,
                    &EncodingKey::from_secret(std::env::var("REFRESH_TOKEN_SECRET").unwrap().as_ref()),
                ).unwrap();

                // Сохраняем refresh-токен в базе данных
                sqlx::query!(
                    "INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES ($1, $2, to_timestamp($3))",
                    record.user_id,
                    refresh_token,
                    refresh_exp as f64
                )
                    .execute(pool.get_ref())
                    .await
                    .unwrap();

                // Возвращаем токены клиенту
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

    // Проверяем валидность refresh-токена
    let token_data = decode::<Claims>(
        refresh_token,
        &DecodingKey::from_secret(std::env::var("REFRESH_TOKEN_SECRET").unwrap().as_ref()),
        &Validation::default(),
    );

    match token_data {
        Ok(data) => {
            let user_id = data.claims.sub;

            // Проверяем, что токен есть в базе и не истек
            let result = sqlx::query!(
                "SELECT * FROM refresh_tokens WHERE user_id = $1 AND token = $2 AND expires_at > NOW()",
                user_id,
                refresh_token
            )
                .fetch_one(pool.get_ref())
                .await;

            match result {
                Ok(_) => {
                    // Генерируем новый access-токен
                    let access_exp = Utc::now()
                        .checked_add_signed(Duration::minutes(3))
                        .expect("Ошибка при установке времени")
                        .timestamp();
                    let access_claims = Claims {
                        sub: user_id,
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