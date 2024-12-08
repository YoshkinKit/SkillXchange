use actix_web::{HttpMessage, HttpResponse, web};
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct CreateMessage {
    pub receiver_id: i32,
    pub content: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateMessage {
    pub content: String,
}

#[derive(serde::Serialize)]
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