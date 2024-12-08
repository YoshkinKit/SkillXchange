use actix_web::{HttpMessage, HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CreateRequest {
    pub receiver_id: i32,
    pub skill_id: i32,
    pub cover_letter: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UpdateRequestStatus {
    pub status: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RequestResponse {
    pub request_id: i32,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub skill_id: i32,
    pub status: String,
    pub cover_letter: String,
    pub created_at: chrono::DateTime<Utc>,
}

pub async fn get_sent_requests(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let user_id = req.extensions().get::<i32>().cloned().unwrap();

    let result = sqlx::query!(
        r#"
        SELECT * FROM requests
        WHERE sender_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(requests) => {
            let requests = requests.into_iter().map(|r| RequestResponse {
                request_id: r.request_id,
                sender_id: r.sender_id,
                receiver_id: r.receiver_id,
                skill_id: r.skill_id.expect("1"),
                status: r.status,
                cover_letter: r.cover_letter.expect("No cover letter"),
                created_at: r.created_at.expect("1970-01-01T00:00:00Z"),
            }).collect::<Vec<_>>();
            HttpResponse::Ok().json(requests)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn get_received_requests(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let user_id = req.extensions().get::<i32>().cloned().unwrap();

    let result = sqlx::query!(
        r#"
        SELECT * FROM requests
        WHERE receiver_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(requests) => {
            let requests = requests.into_iter().map(|r| RequestResponse {
                request_id: r.request_id,
                sender_id: r.sender_id,
                receiver_id: r.receiver_id,
                skill_id: r.skill_id.expect("1"),
                status: r.status,
                cover_letter: r.cover_letter.expect("No cover letter"),
                created_at: r.created_at.expect("1970-01-01T00:00:00Z"),
            }).collect::<Vec<_>>();
            HttpResponse::Ok().json(requests)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn create_request(
    pool: web::Data<PgPool>,
    form: web::Json<CreateRequest>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let sender_id = req.extensions().get::<i32>().cloned().unwrap();

    let result = sqlx::query!(
        r#"
        INSERT INTO requests (sender_id, receiver_id, skill_id, status, cover_letter)
        VALUES ($1, $2, $3, 'pending', $4)
        RETURNING *
        "#,
        sender_id,
        form.receiver_id,
        form.skill_id,
        form.cover_letter,
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(request) => {
            let response = RequestResponse {
                request_id: request.request_id,
                sender_id: request.sender_id,
                receiver_id: request.receiver_id,
                skill_id: request.skill_id.expect("1"),
                status: request.status,
                cover_letter: request.cover_letter.expect("No cover letter"),
                created_at: request.created_at.expect("1970-01-01T00:00:00Z"),
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn update_request_status(
    pool: web::Data<PgPool>,
    request_id: web::Path<i32>,
    form: web::Json<UpdateRequestStatus>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let user_id = req.extensions().get::<i32>().cloned().unwrap();

    let request = sqlx::query!(
        "SELECT * FROM requests WHERE request_id = $1",
        *request_id
    )
        .fetch_one(pool.get_ref())
        .await;

    match request {
        Ok(request) => {
            if request.receiver_id != user_id {
                return HttpResponse::Forbidden().body("Только получатель может обновить статус запроса");
            }

            if !["accept", "decline"].contains(&form.status.as_str()) {
                return HttpResponse::BadRequest().body("Недопустимый статус. Используйте 'accept' или 'decline'");
            }

            let result = sqlx::query!(
                r#"
                UPDATE requests
                SET status = $1
                WHERE request_id = $2
                RETURNING *
                "#,
                form.status,
                *request_id
            )
                .fetch_one(pool.get_ref())
                .await;

            match result {
                Ok(updated) => {
                    let response = RequestResponse {
                        request_id: updated.request_id,
                        sender_id: updated.sender_id,
                        receiver_id: updated.receiver_id,
                        skill_id: updated.skill_id.expect("1"),
                        status: updated.status,
                        cover_letter: updated.cover_letter.expect("No cover letter"),
                        created_at: updated.created_at.expect("1970-01-01T00:00:00Z"),
                    };
                    HttpResponse::Ok().json(response)
                }
                Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
            }
        }
        Err(_) => HttpResponse::NotFound().body("Запрос не найден"),
    }
}

pub async fn delete_request(
    pool: web::Data<PgPool>,
    request_id: web::Path<i32>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let user_id = req.extensions().get::<i32>().cloned().unwrap();

    let request = sqlx::query!(
        "SELECT * FROM requests WHERE request_id = $1",
        *request_id
    )
        .fetch_one(pool.get_ref())
        .await;

    match request {
        Ok(request) => {
            if request.sender_id != user_id {
                return HttpResponse::Forbidden().body("Только отправитель может удалить запрос");
            }

            let result = sqlx::query!(
                "DELETE FROM requests WHERE request_id = $1",
                *request_id
            )
                .execute(pool.get_ref())
                .await;

            match result {
                Ok(_) => HttpResponse::Ok().body("Запрос удален"),
                Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
            }
        }
        Err(_) => HttpResponse::NotFound().body("Запрос не найден"),
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
    ) -> impl Service<Request, Response=ServiceResponse, Error=actix_web::Error> {
        std::env::set_var("ACCESS_TOKEN_SECRET", "TestSecret");

        let auth_middleware = HttpAuthentication::bearer(jwt_validator);

        let app = App::new()
            .app_data(web::Data::new(pool))
            .service(
                web::scope("/api/requests")
                    .wrap(auth_middleware.clone())
                    .wrap(RequireRole::new(Role::User))
                    .route("/sent", web::get().to(get_sent_requests))
                    .route("/received", web::get().to(get_received_requests))
                    .route("", web::post().to(create_request))
                    .route("/{id}/status", web::put().to(update_request_status))
                    .route("/{id}", web::delete().to(delete_request))
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

        // Отключаем проверку внешних ключей
        sqlx::query!("SET CONSTRAINTS ALL DEFERRED")
            .execute(&mut *tx)
            .await
            .unwrap();

        // Очищаем все таблицы
        sqlx::query!("TRUNCATE TABLE users, skills, categories, requests CASCADE")
            .execute(&mut *tx)
            .await
            .unwrap();

        // Сбрасываем последовательности
        sqlx::query!("ALTER SEQUENCE users_user_id_seq RESTART WITH 1")
            .execute(&mut *tx)
            .await
            .unwrap();

        sqlx::query!("ALTER SEQUENCE skills_skill_id_seq RESTART WITH 1")
            .execute(&mut *tx)
            .await
            .unwrap();

        sqlx::query!("ALTER SEQUENCE categories_category_id_seq RESTART WITH 1")
            .execute(&mut *tx)
            .await
            .unwrap();

        tx.commit().await.unwrap();
    }

    async fn setup_test_data(pool: &PgPool) -> (i32, i32, i32) {
        // Создаем пользователей без явного указания id
        let sender = sqlx::query!(
        "INSERT INTO users (username, email, password_hash, role)
         VALUES ($1, $2, $3, $4)
         RETURNING user_id",
        "sender", "sender@test.com", "hash", "user"
    )
            .fetch_one(pool)
            .await
            .unwrap();

        let receiver = sqlx::query!(
        "INSERT INTO users (username, email, password_hash, role)
         VALUES ($1, $2, $3, $4)
         RETURNING user_id",
        "receiver", "receiver@test.com", "hash", "user"
    )
            .fetch_one(pool)
            .await
            .unwrap();

        // Создаем категорию и навык
        let category = sqlx::query!(
        "INSERT INTO categories (title, description)
         VALUES ($1, $2)
         RETURNING category_id",
        "Test Category", "Test Description"
    )
            .fetch_one(pool)
            .await
            .unwrap();

        let skill = sqlx::query!(
        "INSERT INTO skills (title, description, category_id)
         VALUES ($1, $2, $3)
         RETURNING skill_id",
        "Test Skill", "Test Description", category.category_id
    )
            .fetch_one(pool)
            .await
            .unwrap();

        (sender.user_id, receiver.user_id, skill.skill_id)
    }

    #[sqlx::test]
    async fn test_get_sent_requests() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let (sender_id, receiver_id, skill_id) = setup_test_data(&pool).await;

        sqlx::query!(
            "INSERT INTO requests (sender_id, receiver_id, skill_id, status, cover_letter)
             VALUES ($1, $2, $3, $4, $5)",
            sender_id, receiver_id, skill_id, "pending", "Test request"
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool, Some(sender_id), Some(Role::User)).await;
        let token = generate_test_token(sender_id, Role::User);

        let req = test::TestRequest::get()
            .uri("/api/requests/sent")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: Vec<RequestResponse> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);
        assert_eq!(body[0].sender_id, sender_id);
    }

    #[sqlx::test]
    async fn test_get_received_requests() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let (sender_id, receiver_id, skill_id) = setup_test_data(&pool).await;

        sqlx::query!(
            "INSERT INTO requests (sender_id, receiver_id, skill_id, status, cover_letter)
             VALUES ($1, $2, $3, $4, $5)",
            sender_id, receiver_id, skill_id, "pending", "Test request"
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool, Some(receiver_id), Some(Role::User)).await;
        let token = generate_test_token(receiver_id, Role::User);

        let req = test::TestRequest::get()
            .uri("/api/requests/received")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: Vec<RequestResponse> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);
        assert_eq!(body[0].receiver_id, receiver_id);
    }

    #[sqlx::test]
    async fn test_create_request() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let (sender_id, receiver_id, skill_id) = setup_test_data(&pool).await;

        let app = create_test_app(pool.clone(), Some(sender_id), Some(Role::User)).await;
        let token = generate_test_token(sender_id, Role::User);

        let create_data = CreateRequest {
            receiver_id,
            skill_id,
            cover_letter: "New request".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/requests")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&create_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let created: RequestResponse = test::read_body_json(resp).await;
        assert_eq!(created.sender_id, sender_id);
        assert_eq!(created.status, "pending");
    }

    #[sqlx::test]
    async fn test_update_request_status() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let (sender_id, receiver_id, skill_id) = setup_test_data(&pool).await;

        let request_id = sqlx::query!(
            "INSERT INTO requests (sender_id, receiver_id, skill_id, status, cover_letter)
             VALUES ($1, $2, $3, $4, $5) RETURNING request_id",
            sender_id, receiver_id, skill_id, "pending", "Test request"
        )
            .fetch_one(&pool)
            .await
            .unwrap()
            .request_id;

        let app = create_test_app(pool.clone(), Some(receiver_id), Some(Role::User)).await;
        let token = generate_test_token(receiver_id, Role::User);

        let update_data = UpdateRequestStatus {
            status: "accept".to_string(),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/api/requests/{}/status", request_id))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&update_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let updated: RequestResponse = test::read_body_json(resp).await;
        assert_eq!(updated.status, "accept");
    }

    #[sqlx::test]
    async fn test_delete_request() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let (sender_id, receiver_id, skill_id) = setup_test_data(&pool).await;

        let request_id = sqlx::query!(
        "INSERT INTO requests (sender_id, receiver_id, skill_id, status, cover_letter)
         VALUES ($1, $2, $3, $4, $5) RETURNING request_id",
        sender_id, receiver_id, skill_id, "pending", "Test request"
    )
            .fetch_one(&pool)
            .await
            .unwrap()
            .request_id;

        let app = create_test_app(pool.clone(), Some(sender_id), Some(Role::User)).await;
        let token = generate_test_token(sender_id, Role::User);

        let req = test::TestRequest::delete()
            .uri(&format!("/api/requests/{}", request_id))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Проверяем что запрос удален
        let exists = sqlx::query!("SELECT COUNT(*) as count FROM requests WHERE request_id = $1", request_id)
            .fetch_one(&pool)
            .await
            .unwrap()
            .count
            .unwrap();

        assert_eq!(exists, 0);
    }

    // Добавляем тест на обновление статуса неверным пользователем
    #[sqlx::test]
    async fn test_update_request_status_wrong_user() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let (sender_id, receiver_id, skill_id) = setup_test_data(&pool).await;

        let request_id = sqlx::query!(
        "INSERT INTO requests (sender_id, receiver_id, skill_id, status, cover_letter)
         VALUES ($1, $2, $3, $4, $5) RETURNING request_id",
        sender_id, receiver_id, skill_id, "pending", "Test request"
    )
            .fetch_one(&pool)
            .await
            .unwrap()
            .request_id;

        // Пытаемся обновить статус от имени отправителя
        let app = create_test_app(pool.clone(), Some(sender_id), Some(Role::User)).await;
        let token = generate_test_token(sender_id, Role::User);

        let update_data = UpdateRequestStatus {
            status: "accept".to_string(),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/api/requests/{}/status", request_id))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&update_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    // Добавляем тест на обновление с неверным статусом
    #[sqlx::test]
    async fn test_update_request_status_invalid_status() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let (sender_id, receiver_id, skill_id) = setup_test_data(&pool).await;

        let request_id = sqlx::query!(
        "INSERT INTO requests (sender_id, receiver_id, skill_id, status, cover_letter)
         VALUES ($1, $2, $3, $4, $5) RETURNING request_id",
        sender_id, receiver_id, skill_id, "pending", "Test request"
    )
            .fetch_one(&pool)
            .await
            .unwrap()
            .request_id;

        let app = create_test_app(pool.clone(), Some(receiver_id), Some(Role::User)).await;
        let token = generate_test_token(receiver_id, Role::User);

        let update_data = UpdateRequestStatus {
            status: "invalid_status".to_string(),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/api/requests/{}/status", request_id))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&update_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}