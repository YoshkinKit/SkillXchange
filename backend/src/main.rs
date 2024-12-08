use std::env;

use actix_files::Files;
use actix_web::{App, HttpServer, middleware::Logger, web};
use actix_web_httpauth::middleware::HttpAuthentication;

use middleware::auth::jwt_validator;

use crate::middleware::roles::{RequireRole, Role};

mod db;
mod handlers;
mod middleware;
mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let port: u16 = port.parse().expect("PORT должен быть числом");

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let pool = db::get_db_pool().await;

    HttpServer::new(move || {
        let auth_middleware = HttpAuthentication::bearer(jwt_validator);

        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            // Маршруты для аутентификации и авторизации
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(handlers::auth::register_user))
                    .route("/login", web::post().to(handlers::auth::login_user))
                    .route("/refresh", web::post().to(handlers::auth::refresh_token)),
            )
            // Маршруты для работы с пользователями
            .service(
                web::scope("/api/users")
                    .wrap(auth_middleware.clone())
                    .wrap(RequireRole::new(Role::User))
                    .route("", web::get().to(handlers::user::get_all_users))
                    .route("/{id}", web::get().to(handlers::user::get_user_by_id))
                    .route("/{id}", web::put().to(handlers::user::update_user))
                    .route("/{id}", web::delete().to(handlers::user::delete_user))
                    .route("/skill/{id}", web::get().to(handlers::user::get_users_by_skill))
                    .route("/{id}/skills", web::post().to(handlers::user::add_user_skill))
                    .route("/{id}/skills/{skill_id}", web::delete().to(handlers::user::delete_user_skill)),
            )
            // Маршруты для работы с категориями
            .service(
                web::scope("/api/categories")
                    .route("", web::get().to(handlers::category::get_all_categories))
                    .route("/{id}", web::get().to(handlers::category::get_category_by_id)),
            )
            // Маршруты для работы с навыками
            .service(
                web::scope("/api/skills")
                    .route("", web::get().to(handlers::skill::get_all_skills))
                    .route("/{id}", web::get().to(handlers::skill::get_skill_by_id)),
            )
            // Маршруты для работы с запросами
            .service(
                web::scope("/api/requests")
                    .wrap(auth_middleware.clone())
                    .wrap(RequireRole::new(Role::User))
                    .route("/sent", web::get().to(handlers::request::get_sent_requests))
                    .route("/received", web::get().to(handlers::request::get_received_requests))
                    .route("", web::post().to(handlers::request::create_request))
                    .route("/{id}/status", web::put().to(handlers::request::update_request_status))
                    .route("/{id}", web::delete().to(handlers::request::delete_request)),
            )
            // Маршруты для работы с отзывами
            .service(
                web::scope("/api/reviews")
                    .wrap(auth_middleware.clone())
                    .wrap(RequireRole::new(Role::User))
                    .route("/user/{id}", web::get().to(handlers::review::get_user_reviews))
                    .route("/user/{id}/skill/{skill_id}", web::get().to(handlers::review::get_user_reviews_by_skill))
                    .route("/sent/{id}", web::get().to(handlers::review::get_sent_reviews))
                    .route("", web::post().to(handlers::review::create_review))
                    .route("/{id}", web::put().to(handlers::review::update_review))
                    .route("/{id}", web::delete().to(handlers::review::delete_review)),
            )
            // Маршруты для работы с сообщениями
            .service(
                web::scope("/api/messages")
                    .wrap(auth_middleware.clone())
                    .wrap(RequireRole::new(Role::User))
                    .route("/chat/{id}", web::get().to(handlers::message::get_chat_messages))
                    .route("", web::post().to(handlers::message::create_message))
                    .route("/{id}", web::put().to(handlers::message::update_message))
                    .route("/{id}", web::delete().to(handlers::message::delete_message)),
            )
            .service(
                // Маршруты для администратора
                web::scope("api/admin")
                    .service(
                        web::scope("/categories")
                            .wrap(auth_middleware.clone())
                            .wrap(RequireRole::new(Role::Admin))
                            .route("", web::post().to(handlers::category::create_category))
                            .route("/{id}", web::put().to(handlers::category::update_category))
                            .route("/{id}", web::delete().to(handlers::category::delete_category)),
                    )
                    .service(
                        web::scope("/skills")
                            .wrap(auth_middleware.clone())
                            .wrap(RequireRole::new(Role::Admin))
                            .route("", web::post().to(handlers::skill::create_skill))
                            .route("/{id}", web::put().to(handlers::skill::update_skill))
                            .route("/{id}", web::delete().to(handlers::skill::delete_skill)),
                    ),
            )
            .service(Files::new("/static", "../frontend/dist").index_file("index.html"))
    })
        .bind(("127.0.0.1", port))?
        .run()
        .await
}