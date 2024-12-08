use actix_web::{HttpMessage, HttpResponse, web};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::middleware::roles::Role;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CreateReview {
    pub receiver_id: i32,
    pub skill_id: i32,
    pub rating: i32,
    pub comment: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UpdateReview {
    pub rating: i32,
    pub comment: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
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

#[cfg(test)]
mod tests {
    use actix_http::Request;
    use actix_web::{
        App, dev::{Service, ServiceResponse},
        http::StatusCode,
        test,
        web,
    };
    use actix_web_httpauth::middleware::HttpAuthentication;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use sqlx::{Pool, Postgres};

    use crate::middleware::auth::jwt_validator;
    use crate::middleware::roles::{RequireRole, Role};

    use super::*;

    #[derive(serde::Deserialize, serde::Serialize)]
    struct TestClaims {
        sub: i32,
        role: String,
        exp: usize
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

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("TestSecret".as_ref()),
        ).unwrap()
    }

    async fn clean_db(pool: &PgPool) {
        let mut tx = pool.begin().await.unwrap();

        sqlx::query!("TRUNCATE TABLE users, skills, categories, reviews CASCADE")
            .execute(&mut *tx)
            .await
            .unwrap();

        sqlx::query!("ALTER SEQUENCE users_user_id_seq RESTART WITH 1")
            .execute(&mut *tx)
            .await
            .unwrap();

        tx.commit().await.unwrap();
    }

    async fn setup_test_data(pool: &PgPool) -> (i32, i32, i32) {
        // Создаем двух пользователей и навык
        let sender = sqlx::query!(
            "INSERT INTO users (username, email, password_hash, role)
             VALUES ($1, $2, $3, $4) RETURNING user_id",
            "reviewer", "reviewer@test.com", "hash", "user"
        )
            .fetch_one(pool)
            .await
            .unwrap();

        let receiver = sqlx::query!(
            "INSERT INTO users (username, email, password_hash, role)
             VALUES ($1, $2, $3, $4) RETURNING user_id",
            "reviewed", "reviewed@test.com", "hash", "user"
        )
            .fetch_one(pool)
            .await
            .unwrap();

        let category = sqlx::query!(
            "INSERT INTO categories (title, description)
             VALUES ($1, $2) RETURNING category_id",
            "Test Category", "Description"
        )
            .fetch_one(pool)
            .await
            .unwrap();

        let skill = sqlx::query!(
            "INSERT INTO skills (title, description, category_id)
             VALUES ($1, $2, $3) RETURNING skill_id",
            "Test Skill", "Description", category.category_id
        )
            .fetch_one(pool)
            .await
            .unwrap();

        (sender.user_id, receiver.user_id, skill.skill_id)
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
                web::scope("/api/reviews")
                    .wrap(auth_middleware.clone())
                    .wrap(RequireRole::new(Role::User))
                    .route("/user/{id}", web::get().to(get_user_reviews))
                    .route("/user/{id}/skill/{skill_id}", web::get().to(get_user_reviews_by_skill))
                    .route("/sent/{id}", web::get().to(get_sent_reviews))
                    .route("", web::post().to(create_review))
                    .route("/{id}", web::put().to(update_review))
                    .route("/{id}", web::delete().to(delete_review))
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

    #[sqlx::test]
    async fn test_get_user_reviews() {
        let pool = PgPool::connect("postgres://postgres:111@localhost:5432/test_db").await.unwrap();
        clean_db(&pool).await;

        let (sender_id, receiver_id, skill_id) = setup_test_data(&pool).await;

        // Создаем отзыв
        sqlx::query!(
            "INSERT INTO reviews (sender_id, receiver_id, skill_id, rating, comment)
             VALUES ($1, $2, $3, $4, $5)",
            sender_id, receiver_id, skill_id, 5, "Great!"
        )
            .execute(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool, Some(sender_id), Some(Role::User)).await;
        let token = generate_test_token(sender_id, Role::User);

        let req = test::TestRequest::get()
            .uri(&format!("/api/reviews/user/{}", receiver_id))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let reviews: Vec<ReviewResponse> = test::read_body_json(resp).await;
        assert_eq!(reviews.len(), 1);
        assert_eq!(reviews[0].rating, 5);
    }

    #[sqlx::test]
    async fn test_create_review() {
        let pool = PgPool::connect("postgres://postgres:111@localhost:5432/test_db").await.unwrap();
        clean_db(&pool).await;

        let (sender_id, receiver_id, skill_id) = setup_test_data(&pool).await;

        let app = create_test_app(pool, Some(sender_id), Some(Role::User)).await;
        let token = generate_test_token(sender_id, Role::User);

        let create_data = CreateReview {
            receiver_id,
            skill_id,
            rating: 5,
            comment: "Excellent!".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/reviews")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&create_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let review: ReviewResponse = test::read_body_json(resp).await;
        assert_eq!(review.rating, 5);
        assert_eq!(review.comment, "Excellent!");
    }

    #[sqlx::test]
    async fn test_update_review() {
        let pool = PgPool::connect("postgres://postgres:111@localhost:5432/test_db").await.unwrap();
        clean_db(&pool).await;

        let (sender_id, receiver_id, skill_id) = setup_test_data(&pool).await;

        let review = sqlx::query!(
            "INSERT INTO reviews (sender_id, receiver_id, skill_id, rating, comment)
             VALUES ($1, $2, $3, $4, $5) RETURNING review_id",
            sender_id, receiver_id, skill_id, 4, "Good"
        )
            .fetch_one(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool, Some(sender_id), Some(Role::User)).await;
        let token = generate_test_token(sender_id, Role::User);

        let update_data = UpdateReview {
            rating: 5,
            comment: "Updated: Excellent!".to_string(),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/api/reviews/{}", review.review_id))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&update_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let updated: ReviewResponse = test::read_body_json(resp).await;
        assert_eq!(updated.rating, 5);
        assert_eq!(updated.comment, "Updated: Excellent!");
    }

    #[sqlx::test]
    async fn test_delete_review() {
        let pool = PgPool::connect("postgres://postgres:111@localhost:5432/test_db").await.unwrap();
        clean_db(&pool).await;

        let (sender_id, receiver_id, skill_id) = setup_test_data(&pool).await;

        let review = sqlx::query!(
            "INSERT INTO reviews (sender_id, receiver_id, skill_id, rating, comment)
             VALUES ($1, $2, $3, $4, $5) RETURNING review_id",
            sender_id, receiver_id, skill_id, 3, "Average"
        )
            .fetch_one(&pool)
            .await
            .unwrap();

        let app = create_test_app(pool.clone(), Some(sender_id), Some(Role::User)).await;
        let token = generate_test_token(sender_id, Role::User);

        let req = test::TestRequest::delete()
            .uri(&format!("/api/reviews/{}", review.review_id))
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Проверяем что отзыв удален
        let exists = sqlx::query!(
            "SELECT COUNT(*) as count FROM reviews WHERE review_id = $1",
            review.review_id
        )
            .fetch_one(&pool)
            .await
            .unwrap()
            .count
            .unwrap();

        assert_eq!(exists, 0);
    }
}