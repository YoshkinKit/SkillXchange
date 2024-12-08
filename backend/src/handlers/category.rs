use actix_web::{HttpMessage, HttpResponse, web};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::middleware::roles::Role;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CreateCategory {
    pub title: String,
    pub description: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UpdateCategory {
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
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

#[cfg(test)]
mod tests {
    use std::env;
    use actix_http::Request;
    use actix_web::{
        App, dev::{Service, ServiceResponse},
        http::StatusCode,
        test, web,
    };
    use actix_web_httpauth::middleware::HttpAuthentication;
    use chrono::Utc;
    use sqlx::{Pool, Postgres};

    use crate::middleware::auth::jwt_validator;
    use crate::middleware::roles::{RequireRole, Role};

    use super::*;

    // Копируем вспомогательные функции из user.rs
    #[derive(serde::Serialize)]
    struct TestClaims {
        sub: i32,
        role: String,
        exp: usize,
    }

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

        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret("TestSecret".as_ref()),
        ).unwrap()
    }

    async fn create_test_app(
        pool: Pool<Postgres>,
        user_id: Option<i32>,
        role: Option<Role>,
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        std::env::set_var("ACCESS_TOKEN_SECRET", "TestSecret");

        let auth_middleware = HttpAuthentication::bearer(jwt_validator);

        let app = App::new()
            .app_data(web::Data::new(pool))
            .service(
                web::scope("/api/categories")
                    .route("", web::get().to(get_all_categories))
                    .route("/{id}", web::get().to(get_category_by_id))
            )
            .service(
                web::scope("/api/admin/categories")
                    .wrap(auth_middleware.clone())
                    .wrap(RequireRole::new(Role::Admin))
                    .route("", web::post().to(create_category))
                    .route("/{id}", web::put().to(update_category))
                    .route("/{id}", web::delete().to(delete_category))
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
        sqlx::query!("TRUNCATE TABLE categories CASCADE")
            .execute(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }

    #[sqlx::test]
    async fn test_get_all_categories() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        sqlx::query!(
            "INSERT INTO categories (category_id, title, description) VALUES ($1, $2, $3)",
            1, "Test Category", "Test Description"
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool, None, None).await;

        let req = test::TestRequest::get()
            .uri("/api/categories")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: Vec<CategoryResponse> = test::read_body_json(resp).await;
        assert!(!body.is_empty());
        assert_eq!(body[0].title, "Test Category");
    }

    #[sqlx::test]
    async fn test_get_category_by_id() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        sqlx::query!(
            "INSERT INTO categories (category_id, title, description) VALUES ($1, $2, $3)",
            1, "Test Category", "Test Description"
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool, None, None).await;

        let req = test::TestRequest::get()
            .uri("/api/categories/1")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let category: CategoryResponse = test::read_body_json(resp).await;
        assert_eq!(category.title, "Test Category");
    }

    #[sqlx::test]
    async fn test_create_category_as_admin() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let app = create_test_app(pool.clone(), Some(1), Some(Role::Admin)).await;
        let token = generate_test_token(1, Role::Admin);

        let create_data = CreateCategory {
            title: "New Category".to_string(),
            description: "New Description".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/admin/categories")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&create_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let created: CategoryResponse = test::read_body_json(resp).await;
        assert_eq!(created.title, "New Category");
    }

    #[sqlx::test]
    async fn test_create_category_as_user() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let app = create_test_app(pool, Some(1), Some(Role::User)).await;
        let token = generate_test_token(1, Role::User);

        let create_data = CreateCategory {
            title: "New Category".to_string(),
            description: "New Description".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/admin/categories")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&create_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test]
    async fn test_update_category() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        sqlx::query!(
            "INSERT INTO categories (category_id, title, description) VALUES ($1, $2, $3)",
            1, "Old Title", "Old Description"
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool.clone(), Some(1), Some(Role::Admin)).await;
        let token = generate_test_token(1, Role::Admin);

        let update_data = UpdateCategory {
            title: Some("Updated Title".to_string()),
            description: Some("Updated Description".to_string()),
        };

        let req = test::TestRequest::put()
            .uri("/api/admin/categories/1")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&update_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let updated: CategoryResponse = test::read_body_json(resp).await;
        assert_eq!(updated.title, "Updated Title");
    }

    #[sqlx::test]
    async fn test_delete_category() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        sqlx::query!(
            "INSERT INTO categories (category_id, title, description) VALUES ($1, $2, $3)",
            1, "To Delete", "Description"
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool.clone(), Some(1), Some(Role::Admin)).await;
        let token = generate_test_token(1, Role::Admin);

        let req = test::TestRequest::delete()
            .uri("/api/admin/categories/1")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Проверяем что категория помечена как удаленная
        let category = sqlx::query!("SELECT is_deleted FROM categories WHERE category_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert!(category.is_deleted);
    }
}