use actix_web::{HttpMessage, HttpResponse, web};
use chrono::{DateTime, Utc};
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

#[derive(serde::Deserialize)]
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

#[derive(serde::Deserialize)]
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