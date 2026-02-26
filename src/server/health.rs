use actix_web::{HttpResponse, Responder};
use chrono::Utc;
use serde_json::json;

pub async fn root() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "ok": true,
        "timestamp": Utc::now().to_rfc3339()
    }))
}