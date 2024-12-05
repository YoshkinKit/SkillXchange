use std::env;

use actix_files::Files;
use actix_web::{middleware::Logger, App, HttpServer, web};
use actix_web_httpauth::middleware::HttpAuthentication;

use middleware::auth::jwt_validator;

mod db;
mod handlers;
mod middleware;
mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Получаем порт из переменной окружения или устанавливаем по умолчанию 8000
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let port: u16 = port.parse().expect("PORT должен быть числом");

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Создаем пул соединений к базе данных
    let pool = db::get_db_pool().await;

    HttpServer::new(move || {
        let auth_middleware = HttpAuthentication::bearer(jwt_validator);

        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            // Обслуживаем статические файлы из директории frontend/dist
            //
            // Маршруты для авторизации
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(handlers::auth::register_user))
                    .route("/login", web::post().to(handlers::auth::login_user))
                    .route("/refresh", web::post().to(handlers::auth::refresh_token)),
            )
            // Защищенные маршруты API
            .service(
                web::scope("/api")
                    .wrap(auth_middleware)
            )
            .service(Files::new("/static", "./frontend/dist").index_file("index.html"))
    })
        .bind(("127.0.0.1", port))?
        .run()
        .await
}