use actix_files::Files;
use actix_web::{web, App, HttpServer};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Получаем порт из переменной окружения или устанавливаем по умолчанию 8000
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let port: u16 = port.parse().expect("PORT должен быть числом");

    HttpServer::new(|| {
        App::new()
            // Обслуживаем статические файлы из директории frontend/dist
            .service(Files::new("/", "./frontend/dist").index_file("index.html"))
        // Добавьте ваши маршруты API здесь, например:
        // .route("/api", web::get().to(api_handler))
    })
        .bind(("0.0.0.0", port))?
        .run()
        .await
}