use actix_web::{web, HttpResponse, HttpMessage};
use sqlx::PgPool;

use crate::models::user::{User, UserResponse};
use crate::middleware::roles::Role;

pub async fn get_all_users(pool: web::Data<PgPool>) -> HttpResponse {
    let result = sqlx::query_as!(
        UserResponse,
        r#"SELECT user_id, username, email, created_at FROM users"#
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка базы данных: {}", e)),
    }
}

pub async fn get_user_by_id(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> HttpResponse {
    let result = sqlx::query_as!(
        UserResponse,
        r#"SELECT user_id, username, email, created_at FROM users WHERE user_id = $1"#,
        *user_id
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("Пользователь не найден"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

#[derive(serde::Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub bio: Option<String>,
}

pub async fn update_user(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
    data: web::Json<UpdateUser>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    // Получаем ID пользователя из токена
    let auth_user_id = req.extensions().get::<i32>().cloned();
    let role = req.extensions().get::<Role>().cloned().unwrap_or(Role::Guest);

    // Проверяем права доступа
    if auth_user_id != Some(*user_id) && role != Role::Admin {
        return HttpResponse::Forbidden().body("Доступ запрещен");
    }

    let mut user = sqlx::query_as!(
        User,
        r#"SELECT * FROM users WHERE user_id = $1"#,
        *user_id
    )
        .fetch_one(pool.get_ref())
        .await
        .map_err(|_| HttpResponse::NotFound().body("Пользователь не найден"))?;

    // Обновляем поля
    if let Some(username) = &data.username {
        user.username = username.clone();
    }
    if let Some(email) = &data.email {
        user.email = email.clone();
    }
    if let Some(bio) = &data.bio {
        user.bio = Some(bio.clone());
    }

    // Сохраняем изменения
    let result = sqlx::query!(
        r#"
        UPDATE users SET username = $1, email = $2, bio = $3 WHERE user_id = $4
        "#,
        user.username,
        user.email,
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

// Удаление пользователя
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

    let result = sqlx::query!(
        r#"DELETE FROM users WHERE user_id = $1"#,
        *user_id
    )
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Пользователь удален"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}