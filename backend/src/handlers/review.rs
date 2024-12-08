use actix_web::{HttpMessage, HttpResponse, web};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::middleware::roles::Role;

#[derive(serde::Deserialize)]
pub struct CreateReview {
    pub receiver_id: i32,
    pub skill_id: i32,
    pub rating: i32,
    pub comment: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateReview {
    pub rating: i32,
    pub comment: String,
}

#[derive(serde::Serialize)]
pub struct ReviewResponse {
    pub review_id: i32,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub skill_id: i32,
    pub rating: i32,
    pub comment: String,
    pub created_at: DateTime<Utc>,
}

pub async fn get_user_reviews(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> HttpResponse {
    let result = sqlx::query!(
        r#"
        SELECT * FROM reviews
        WHERE receiver_id = $1
        ORDER BY created_at DESC
        "#,
        *user_id
    )
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(reviews) => {
            let reviews = reviews.into_iter().map(|r| ReviewResponse {
                review_id: r.review_id,
                sender_id: r.sender_id,
                receiver_id: r.receiver_id,
                skill_id: r.skill_id.expect("1"),
                rating: r.rating.expect("3"),
                comment: r.comment.expect("No comment"),
                created_at: r.created_at.expect("1970-01-01T00:00:00Z"),
            }).collect::<Vec<_>>();
            HttpResponse::Ok().json(reviews)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn get_user_reviews_by_skill(
    pool: web::Data<PgPool>,
    params: web::Path<(i32, i32)>, // (user_id, skill_id)
) -> HttpResponse {
    let (user_id, skill_id) = params.into_inner();
    let result = sqlx::query!(
        r#"
        SELECT * FROM reviews
        WHERE receiver_id = $1 AND skill_id = $2
        ORDER BY created_at DESC
        "#,
        user_id,
        skill_id
    )
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(reviews) => {
            let reviews = reviews.into_iter().map(|r| ReviewResponse {
                review_id: r.review_id,
                sender_id: r.sender_id,
                receiver_id: r.receiver_id,
                skill_id: r.skill_id.expect("1"),
                rating: r.rating.expect("3"),
                comment: r.comment.expect("No comment"),
                created_at: r.created_at.expect("1970-01-01T00:00:00Z"),
            }).collect::<Vec<_>>();
            HttpResponse::Ok().json(reviews)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn get_sent_reviews(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> HttpResponse {
    let result = sqlx::query!(
        r#"
        SELECT * FROM reviews
        WHERE sender_id = $1
        ORDER BY created_at DESC
        "#,
        *user_id
    )
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(reviews) => {
            let reviews = reviews.into_iter().map(|r| ReviewResponse {
                review_id: r.review_id,
                sender_id: r.sender_id,
                receiver_id: r.receiver_id,
                skill_id: r.skill_id.expect("1"),
                rating: r.rating.expect("3"),
                comment: r.comment.expect("No comment"),
                created_at: r.created_at.expect("1970-01-01T00:00:00Z"),
            }).collect::<Vec<_>>();
            HttpResponse::Ok().json(reviews)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn create_review(
    pool: web::Data<PgPool>,
    form: web::Json<CreateReview>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let sender_id = req.extensions().get::<i32>().cloned().unwrap();

    let result = sqlx::query!(
        r#"
        INSERT INTO reviews (sender_id, receiver_id, skill_id, rating, comment)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
        sender_id,
        form.receiver_id,
        form.skill_id,
        form.rating,
        form.comment,
    )
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(review) => {
            let response = ReviewResponse {
                review_id: review.review_id,
                sender_id: review.sender_id,
                receiver_id: review.receiver_id,
                skill_id: review.skill_id.expect("1"),
                rating: review.rating.expect("3"),
                comment: review.comment.expect("No comment"),
                created_at: review.created_at.expect("1970-01-01T00:00:00Z"),
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
    }
}

pub async fn update_review(
    pool: web::Data<PgPool>,
    review_id: web::Path<i32>,
    form: web::Json<UpdateReview>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let user_id = req.extensions().get::<i32>().cloned().unwrap();
    let role = req.extensions().get::<Role>().cloned().unwrap_or(Role::Guest);

    let review = sqlx::query!(
        "SELECT * FROM reviews WHERE review_id = $1",
        *review_id
    )
        .fetch_one(pool.get_ref())
        .await;

    match review {
        Ok(review) => {
            if review.sender_id != user_id && role != Role::Admin {
                return HttpResponse::Forbidden().body("Только автор отзыва или администратор могут его обновить");
            }

            let result = sqlx::query!(
                r#"
                UPDATE reviews
                SET rating = $1, comment = $2
                WHERE review_id = $3
                RETURNING *
                "#,
                form.rating,
                form.comment,
                *review_id
            )
                .fetch_one(pool.get_ref())
                .await;

            match result {
                Ok(updated) => {
                    let response = ReviewResponse {
                        review_id: updated.review_id,
                        sender_id: updated.sender_id,
                        receiver_id: updated.receiver_id,
                        skill_id: updated.skill_id.expect("1"),
                        rating: updated.rating.expect("3"),
                        comment: updated.comment.expect("No comment"),
                        created_at: updated.created_at.expect("1970-01-01T00:00:00Z"),
                    };
                    HttpResponse::Ok().json(response)
                }
                Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
            }
        }
        Err(_) => HttpResponse::NotFound().body("Отзыв не найден"),
    }
}

pub async fn delete_review(
    pool: web::Data<PgPool>,
    review_id: web::Path<i32>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let user_id = req.extensions().get::<i32>().cloned().unwrap();
    let role = req.extensions().get::<Role>().cloned().unwrap_or(Role::Guest);

    let review = sqlx::query!(
        "SELECT * FROM reviews WHERE review_id = $1",
        *review_id
    )
        .fetch_one(pool.get_ref())
        .await;

    match review {
        Ok(review) => {
            if review.sender_id != user_id && role != Role::Admin {
                return HttpResponse::Forbidden().body("Только автор отзыва или администратор могут его удалить");
            }

            let result = sqlx::query!(
                "DELETE FROM reviews WHERE review_id = $1",
                *review_id
            )
                .execute(pool.get_ref())
                .await;

            match result {
                Ok(_) => HttpResponse::Ok().body("Отзыв удален"),
                Err(e) => HttpResponse::InternalServerError().body(format!("Ошибка: {}", e)),
            }
        }
        Err(_) => HttpResponse::NotFound().body("Отзыв не найден"),
    }
}