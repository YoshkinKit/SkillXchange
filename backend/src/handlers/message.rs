use actix_web::{HttpMessage, HttpResponse, web};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CreateMessage {
    pub receiver_id: i32,
    pub content: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UpdateMessage {
    pub content: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MessageResponse {
    pub message_id: i32,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub is_deleted: bool,
}

pub async fn get_chat_messages(
    pool: web::Data<PgPool>,
    other_user_id: web::Path<i32>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let user_id = req.extensions().get::<i32>().cloned().unwrap();

    let result = sqlx::query!(
        r#"
        SELECT * FROM messages
        WHERE (sender_id = $1 AND receiver_id = $2)
           OR (sender_id = $2 AND receiver_id = $1)
        ORDER BY created_at ASC
        "#,
        user_id,
        *other_user_id
    )
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(messages) => {
            let messages = messages.into_iter().map(|m| MessageResponse {
                message_id: m.message_id,
                sender_id: m.sender_id,
                receiver_id: m.receiver_id,
                content: m.content,
                created_at: m.created_at.expect("1970-01-01T00:00:00Z"),
                is_deleted: m.is_deleted,
            }).collect::<Vec<_>>();
            HttpResponse::Ok().json(messages)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn create_message(
    pool: web::Data<PgPool>,
    form: web::Json<CreateMessage>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let sender_id = req.extensions().get::<i32>().cloned().unwrap();

    let result = sqlx::query!(
        r#"
        INSERT INTO messages (sender_id, receiver_id, content, is_deleted)
        VALUES ($1, $2, $3, false)
        RETURNING *
        "#,
        sender_id,
        form.receiver_id,
        form.content,
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(message) => {
            let response = MessageResponse {
                message_id: message.message_id,
                sender_id: message.sender_id,
                receiver_id: message.receiver_id,
                content: message.content,
                created_at: message.created_at.expect("1970-01-01T00:00:00Z"),
                is_deleted: message.is_deleted,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn update_message(
    pool: web::Data<PgPool>,
    message_id: web::Path<i32>,
    form: web::Json<UpdateMessage>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let user_id = req.extensions().get::<i32>().cloned().unwrap();

    let message = sqlx::query!(
        "SELECT * FROM messages WHERE message_id = $1",
        *message_id
    )
        .fetch_one(pool.get_ref())
        .await;

    match message {
        Ok(message) => {
            if message.sender_id != user_id {
                return HttpResponse::Forbidden().body("Только отправитель может обновить сообщение");
            }

            let hours_passed = Utc::now()
                .signed_duration_since(message.created_at.expect("1970-01-01T00:00:00Z"))
                .num_hours();

            if hours_passed >= 12 {
                return HttpResponse::BadRequest().body("Нельзя обновить сообщение старше 12 часов");
            }

            let result = sqlx::query!(
                r#"
                UPDATE messages
                SET content = $1
                WHERE message_id = $2
                RETURNING *
                "#,
                form.content,
                *message_id
            )
                .fetch_one(pool.get_ref())
                .await;

            match result {
                Ok(updated) => {
                    let response = MessageResponse {
                        message_id: updated.message_id,
                        sender_id: updated.sender_id,
                        receiver_id: updated.receiver_id,
                        content: updated.content,
                        created_at: updated.created_at.expect("1970-01-01T00:00:00Z"),
                        is_deleted: updated.is_deleted,
                    };
                    HttpResponse::Ok().json(response)
                }
                Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
            }
        }
        Err(_) => HttpResponse::NotFound().body("Сообщение не найдено"),
    }
}

pub async fn delete_message(
    pool: web::Data<PgPool>,
    message_id: web::Path<i32>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let user_id = req.extensions().get::<i32>().cloned().unwrap();

    let result = sqlx::query!(
        r#"
        UPDATE messages
        SET is_deleted = true
        WHERE message_id = $1 AND sender_id = $2
        RETURNING *
        "#,
        *message_id,
        user_id
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Сообщение удалено"),
        Err(_) => HttpResponse::NotFound().body("Сообщение не найдено или у вас нет прав на его удаление"),
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
    use chrono::{Duration, Utc};
    use sqlx::{Pool, Postgres};

    use crate::middleware::auth::jwt_validator;
    use crate::middleware::roles::{RequireRole, Role};

    use super::*;

    async fn clean_db(pool: &PgPool) {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query!("TRUNCATE TABLE users, messages CASCADE")
            .execute(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }

    async fn setup_test_users(pool: &PgPool) -> (i32, i32) {
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

        (sender.user_id, receiver.user_id)
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
                web::scope("/api/messages")
                    .wrap(auth_middleware.clone())
                    .wrap(RequireRole::new(Role::User))
                    .route("/chat/{id}", web::get().to(get_chat_messages))
                    .route("", web::post().to(create_message))
                    .route("/{id}", web::put().to(update_message))
                    .route("/{id}", web::delete().to(delete_message))
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

    #[derive(serde::Deserialize, serde::Serialize)]
    struct TestClaims {
        sub: i32,
        role: String,
        exp: usize,
    }

    fn generate_test_token(user_id: i32, role: Role) -> String {
        let exp = (Utc::now() + Duration::hours(1)).timestamp() as usize;

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

    #[sqlx::test]
    async fn test_get_chat_messages() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let (sender_id, receiver_id) = setup_test_users(&pool).await;

        sqlx::query!(
            "INSERT INTO messages (sender_id, receiver_id, content, is_deleted)
             VALUES ($1, $2, $3, false)",
            sender_id, receiver_id, "Test message"
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool, Some(sender_id), Some(Role::User)).await;
        let token = generate_test_token(sender_id, Role::User);

        let req = test::TestRequest::get()
            .uri(&format!("/api/messages/chat/{}", receiver_id))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let messages: Vec<MessageResponse> = test::read_body_json(resp).await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Test message");
    }

    #[sqlx::test]
    async fn test_create_message() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let (sender_id, receiver_id) = setup_test_users(&pool).await;

        let app = create_test_app(pool.clone(), Some(sender_id), Some(Role::User)).await;
        let token = generate_test_token(sender_id, Role::User);

        let create_data = CreateMessage {
            receiver_id,
            content: "New message".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/messages")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&create_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let message: MessageResponse = test::read_body_json(resp).await;
        assert_eq!(message.content, "New message");
        assert!(!message.is_deleted);
    }

    #[sqlx::test]
    async fn test_update_message() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let (sender_id, receiver_id) = setup_test_users(&pool).await;

        let message = sqlx::query!(
            "INSERT INTO messages (sender_id, receiver_id, content, is_deleted)
             VALUES ($1, $2, $3, false)
             RETURNING message_id",
            sender_id, receiver_id, "Original message"
        )
            .fetch_one(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool, Some(sender_id), Some(Role::User)).await;
        let token = generate_test_token(sender_id, Role::User);

        let update_data = UpdateMessage {
            content: "Updated message".to_string(),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/api/messages/{}", message.message_id))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&update_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let updated: MessageResponse = test::read_body_json(resp).await;
        assert_eq!(updated.content, "Updated message");
    }

    #[sqlx::test]
    async fn test_delete_message() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL не установлена");
        let pool = PgPool::connect(database_url.as_str()).await.unwrap();
        clean_db(&pool).await;

        let (sender_id, receiver_id) = setup_test_users(&pool).await;

        let message = sqlx::query!(
            "INSERT INTO messages (sender_id, receiver_id, content, is_deleted)
             VALUES ($1, $2, $3, false)
             RETURNING message_id",
            sender_id, receiver_id, "Message to delete"
        )
            .fetch_one(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool.clone(), Some(sender_id), Some(Role::User)).await;
        let token = generate_test_token(sender_id, Role::User);

        let req = test::TestRequest::delete()
            .uri(&format!("/api/messages/{}", message.message_id))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Проверяем что сообщение помечено как удаленное
        let deleted = sqlx::query!(
            "SELECT is_deleted FROM messages WHERE message_id = $1",
            message.message_id
        )
            .fetch_one(&pool)
            .await
            .unwrap();

        assert!(deleted.is_deleted);
    }
}