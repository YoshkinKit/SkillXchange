use actix_web::{HttpMessage, HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;

use crate::middleware::roles::Role;

#[derive(serde::Deserialize)]
pub struct CreateRequest {
    pub receiver_id: i32,
    pub skill_id: i32,
    pub cover_letter: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateRequestStatus {
    pub status: String,
}

#[derive(serde::Serialize)]
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