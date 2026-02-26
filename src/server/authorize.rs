use actix_web::{HttpRequest, HttpResponse};
use serde_json::json;

use crate::utils::secret::check_secret;

pub async fn authorize(req: HttpRequest) -> HttpResponse {
    if let Err(resp) = check_secret(&req) {
        return resp; 
    }

    HttpResponse::Ok().json(json!({ "ok": true }))
}