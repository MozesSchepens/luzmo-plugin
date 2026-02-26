use actix_web::{web, App, HttpServer};

mod engine;
mod errors;
mod luzmo;
mod server;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    println!("SERVER: Rust HTTP plugin listening on http://{}:{}", host, port);

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(server::health::root))
            .route("/health", web::get().to(server::health::health))
            .route("/datasets", web::get().to(server::dataset::handle_datasets))
            .route("/datasets", web::post().to(server::dataset::handle_datasets))
            .route("/query", web::post().to(server::query::handle_query))
            .route("/authorize", web::post().to(server::authorize::authorize))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}