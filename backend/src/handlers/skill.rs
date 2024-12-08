use actix_http::HttpMessage;
use actix_web::{HttpResponse, web};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::middleware::roles::Role;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CreateSkill {
    pub title: String,
    pub description: String,
    pub category_id: i32,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UpdateSkill {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<i32>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SkillResponse {
    pub skill_id: i32,
    pub title: String,
    pub description: String,
    pub category_id: i32,
    pub created_at: DateTime<Utc>,
}

pub async fn get_all_skills(pool: web::Data<PgPool>) -> HttpResponse {
    let result = sqlx::query!(
        "SELECT * FROM skills WHERE is_deleted = FALSE ORDER BY created_at DESC"
    )
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(skills) => {
            let skills = skills.into_iter().map(|skill| SkillResponse {
                skill_id: skill.skill_id,
                title: skill.title,
                description: skill.description.expect("No description"),
                category_id: skill.category_id.expect("No category_id"),
                created_at: skill.created_at.expect("1970-01-01T00:00:00Z"),
            }).collect::<Vec<_>>();
            HttpResponse::Ok().json(skills)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn get_skill_by_id(
    pool: web::Data<PgPool>,
    skill_id: web::Path<i32>,
) -> HttpResponse {
    let result = sqlx::query!(
        "SELECT * FROM skills WHERE skill_id = $1 AND is_deleted = FALSE",
        *skill_id
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(skill) => {
            let skill = SkillResponse {
                skill_id: skill.skill_id,
                title: skill.title,
                description: skill.description.expect("No description"),
                category_id: skill.category_id.expect("No category_id"),
                created_at: skill.created_at.expect("1970-01-01T00:00:00Z"),
            };
            HttpResponse::Ok().json(skill)
        }
        Err(_) => HttpResponse::NotFound().body("Навык не найден"),
    }
}

pub async fn create_skill(
    pool: web::Data<PgPool>,
    form: web::Json<CreateSkill>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let role = req.extensions().get::<Role>().cloned().unwrap_or(Role::Guest);

    if role != Role::Admin {
        return HttpResponse::Forbidden().body("Только администратор может добавлять навыки");
    }

    let result = sqlx::query!(
        r#"
        INSERT INTO skills (title, description, category_id)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        form.title,
        form.description,
        form.category_id,
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(skill) => {
            let skill = SkillResponse {
                skill_id: skill.skill_id,
                title: skill.title,
                description: skill.description.expect("No description"),
                category_id: skill.category_id.expect("No category_id"),
                created_at: skill.created_at.expect("1970-01-01T00:00:00Z"),
            };
            HttpResponse::Ok().json(skill)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn update_skill(
    pool: web::Data<PgPool>,
    skill_id: web::Path<i32>,
    form: web::Json<UpdateSkill>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let role = req.extensions().get::<Role>().cloned().unwrap_or(Role::Guest);

    if role != Role::Admin {
        return HttpResponse::Forbidden().body("Только администратор может обновлять навыки");
    }

    let result = sqlx::query!(
        r#"
        UPDATE skills
        SET
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            category_id = COALESCE($3, category_id)
        WHERE skill_id = $4 AND is_deleted = FALSE
        RETURNING *
        "#,
        form.title,
        form.description,
        form.category_id,
        *skill_id
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(skill) => {
            let skill = SkillResponse {
                skill_id: skill.skill_id,
                title: skill.title,
                description: skill.description.expect("No description"),
                category_id: skill.category_id.expect("No category_id"),
                created_at: skill.created_at.expect("1970-01-01T00:00:00Z"),
            };
            HttpResponse::Ok().json(skill)
        }
        Err(_) => HttpResponse::NotFound().body("Навык не найден"),
    }
}

pub async fn delete_skill(
    pool: web::Data<PgPool>,
    skill_id: web::Path<i32>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let role = req.extensions().get::<Role>().cloned().unwrap_or(Role::Guest);

    if role != Role::Admin {
        return HttpResponse::Forbidden().body("Только администратор может удалять навыки");
    }

    let result = sqlx::query!(
        "UPDATE skills SET is_deleted = TRUE WHERE skill_id = $1",
        *skill_id
    )
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Навык помечен как удалённый"),
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
    use chrono::Utc;
    use sqlx::{Pool, Postgres};

    use crate::middleware::auth::jwt_validator;
    use crate::middleware::roles::{RequireRole, Role};

    use super::*;

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
                web::scope("/api/skills")
                    .route("", web::get().to(get_all_skills))
                    .route("/{id}", web::get().to(get_skill_by_id))
            )
            .service(
                web::scope("/api/admin/skills")
                    .wrap(auth_middleware.clone())
                    .wrap(RequireRole::new(Role::Admin))
                    .route("", web::post().to(create_skill))
                    .route("/{id}", web::put().to(update_skill))
                    .route("/{id}", web::delete().to(delete_skill))
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
        sqlx::query!("TRUNCATE TABLE skills, categories CASCADE")
            .execute(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }

    async fn setup_category(pool: &PgPool) -> i32 {
        let result = sqlx::query!(
            "INSERT INTO categories (title, description) VALUES ($1, $2) RETURNING category_id",
            "Test Category",
            "Test Description"
        )
            .fetch_one(pool)
            .await
            .unwrap();

        result.category_id
    }

    #[sqlx::test]
    async fn test_get_all_skills() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let category_id = setup_category(&pool).await;

        sqlx::query!(
            "INSERT INTO skills (title, description, category_id) VALUES ($1, $2, $3)",
            "Test Skill",
            "Test Description",
            category_id
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool, None, None).await;

        let req = test::TestRequest::get()
            .uri("/api/skills")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: Vec<SkillResponse> = test::read_body_json(resp).await;
        assert!(!body.is_empty());
        assert_eq!(body[0].title, "Test Skill");
    }

    #[sqlx::test]
    async fn test_get_skill_by_id() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let category_id = setup_category(&pool).await;

        sqlx::query!(
            "INSERT INTO skills (skill_id, title, description, category_id) VALUES ($1, $2, $3, $4)",
            1, "Test Skill", "Test Description", category_id
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool, None, None).await;

        let req = test::TestRequest::get()
            .uri("/api/skills/1")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let skill: SkillResponse = test::read_body_json(resp).await;
        assert_eq!(skill.title, "Test Skill");
    }

    #[sqlx::test]
    async fn test_create_skill_as_admin() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let category_id = setup_category(&pool).await;

        let app = create_test_app(pool.clone(), Some(1), Some(Role::Admin)).await;
        let token = generate_test_token(1, Role::Admin);

        let create_data = CreateSkill {
            title: "New Skill".to_string(),
            description: "New Description".to_string(),
            category_id,
        };

        let req = test::TestRequest::post()
            .uri("/api/admin/skills")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&create_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let created: SkillResponse = test::read_body_json(resp).await;
        assert_eq!(created.title, "New Skill");
    }

    #[sqlx::test]
    async fn test_create_skill_as_user() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let category_id = setup_category(&pool).await;

        let app = create_test_app(pool, Some(1), Some(Role::User)).await;
        let token = generate_test_token(1, Role::User);

        let create_data = CreateSkill {
            title: "New Skill".to_string(),
            description: "New Description".to_string(),
            category_id,
        };

        let req = test::TestRequest::post()
            .uri("/api/admin/skills")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&create_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test]
    async fn test_update_skill() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let category_id = setup_category(&pool).await;

        sqlx::query!(
            "INSERT INTO skills (skill_id, title, description, category_id) VALUES ($1, $2, $3, $4)",
            1, "Old Skill", "Old Description", category_id
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool.clone(), Some(1), Some(Role::Admin)).await;
        let token = generate_test_token(1, Role::Admin);

        let update_data = UpdateSkill {
            title: Some("Updated Skill".to_string()),
            description: Some("Updated Description".to_string()),
            category_id: None,
        };

        let req = test::TestRequest::put()
            .uri("/api/admin/skills/1")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&update_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let updated: SkillResponse = test::read_body_json(resp).await;
        assert_eq!(updated.title, "Updated Skill");
    }

    #[sqlx::test]
    async fn test_delete_skill() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let category_id = setup_category(&pool).await;

        sqlx::query!(
            "INSERT INTO skills (skill_id, title, description, category_id) VALUES ($1, $2, $3, $4)",
            1, "To Delete", "Description", category_id
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool.clone(), Some(1), Some(Role::Admin)).await;
        let token = generate_test_token(1, Role::Admin);

        let req = test::TestRequest::delete()
            .uri("/api/admin/skills/1")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Проверяем что навык помечен как удаленный
        let skill = sqlx::query!("SELECT is_deleted FROM skills WHERE skill_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert!(skill.is_deleted);
    }
}