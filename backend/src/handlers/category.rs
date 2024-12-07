use actix_web::{web, HttpResponse, HttpMessage};
use sqlx::PgPool;
use crate::middleware::roles::Role;
use chrono::{DateTime, Utc};

#[derive(serde::Deserialize)]
pub struct CreateCategory {
    pub title: String,
    pub description: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateCategory {
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(serde::Serialize)]
pub struct CategoryResponse {
    pub category_id: i32,
    pub title: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

pub async fn get_all_categories(pool: web::Data<PgPool>) -> HttpResponse {
    let result = sqlx::query!(
        "SELECT * FROM categories WHERE is_deleted = FALSE ORDER BY created_at DESC"
    )
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(categories) => {
            let categories = categories.into_iter().map(|category| CategoryResponse {
                category_id: category.category_id,
                title: category.title,
                description: category.description.expect("No description"),
                created_at: category.created_at.expect("1970-01-01T00:00:00Z"),
            }).collect::<Vec<_>>();
            HttpResponse::Ok().json(categories)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn get_category_by_id(
    pool: web::Data<PgPool>,
    category_id: web::Path<i32>,
) -> HttpResponse {
    let result = sqlx::query!(
        "SELECT * FROM categories WHERE category_id = $1 AND is_deleted = FALSE",
        *category_id
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(category) => {
            let category = CategoryResponse {
                category_id: category.category_id,
                title: category.title,
                description: category.description.expect("No description"),
                created_at: category.created_at.expect("1970-01-01T00:00:00Z"),
            };
            HttpResponse::Ok().json(category)
        }
        Err(_) => HttpResponse::NotFound().body("Категория не найдена"),
    }
}

pub async fn create_category(
    pool: web::Data<PgPool>,
    form: web::Json<CreateCategory>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let role = req.extensions().get::<Role>().cloned().unwrap_or(Role::Guest);

    if role != Role::Admin {
        return HttpResponse::Forbidden().body("Только администратор может создавать категории");
    }

    let result = sqlx::query!(
        r#"
        INSERT INTO categories (title, description)
        VALUES ($1, $2)
        RETURNING *
        "#,
        form.title,
        form.description,
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(category) => {
            let category = CategoryResponse {
                category_id: category.category_id,
                title: category.title,
                description: category.description.expect("No description"),
                created_at: category.created_at.expect("1970-01-01T00:00:00Z"),
            };
            HttpResponse::Ok().json(category)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn update_category(
    pool: web::Data<PgPool>,
    category_id: web::Path<i32>,
    form: web::Json<UpdateCategory>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let role = req.extensions().get::<Role>().cloned().unwrap_or(Role::Guest);

    if role != Role::Admin {
        return HttpResponse::Forbidden().body("Только администратор может обновлять категории");
    }

    let result = sqlx::query!(
        r#"
        UPDATE categories
        SET
            title = COALESCE($1, title),
            description = COALESCE($2, description)
        WHERE category_id = $3
        RETURNING *
        "#,
        form.title,
        form.description,
        *category_id
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(category) => {
            let category = CategoryResponse {
                category_id: category.category_id,
                title: category.title,
                description: category.description.expect("No description"),
                created_at: category.created_at.expect("1970-01-01T00:00:00Z"),
            };
            HttpResponse::Ok().json(category)
        }
        Err(_) => HttpResponse::NotFound().body("Категория не найдена"),
    }
}

pub async fn delete_category(
    pool: web::Data<PgPool>,
    category_id: web::Path<i32>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let role = req.extensions().get::<Role>().cloned().unwrap_or(Role::Guest);

    if role != Role::Admin {
        return HttpResponse::Forbidden().body("Только администратор может удалять категории");
    }

    let result = sqlx::query!(
        "UPDATE categories SET is_deleted = TRUE WHERE category_id = $1",
        *category_id
    )
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Категория помечена как удалённая"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}