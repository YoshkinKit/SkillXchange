use std::env;

use actix_files::Files;
use actix_web::{middleware::Logger, App, HttpServer, web};
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
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(handlers::auth::register_user))
                    .route("/login", web::post().to(handlers::auth::login_user))
                    .route("/refresh", web::post().to(handlers::auth::refresh_token)),
            )
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
            .service(
                web::scope("/api/categories")
                    .route("", web::get().to(handlers::category::get_all_categories))
                    .route("/{id}", web::get().to(handlers::category::get_category_by_id)),
            )
            .service(
                web::scope("api/admin")
                    .wrap(auth_middleware.clone())
                    .wrap(RequireRole::new(Role::Admin))
                    .service(
                        web::scope("/categories")
                            .route("", web::post().to(handlers::category::create_category))
                            .route("/{id}", web::put().to(handlers::category::update_category))
                            .route("/{id}", web::delete().to(handlers::category::delete_category)),
                    )
            )
            .service(Files::new("/static", "../frontend/dist").index_file("index.html"))
    })
        .bind(("127.0.0.1", port))?
        .run()
        .await
}