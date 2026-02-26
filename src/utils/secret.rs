use actix_web::{HttpRequest, HttpResponse};
use serde_json::json;
use std::env;

pub fn expected_secret() -> String {
    // In production: moet gezet zijn
    // In development: fallback ok
    env::var("LUZMO_PLUGIN_SECRET").unwrap_or_else(|_| {
        let mode = env::var("NODE_ENV").unwrap_or_else(|_| "development".to_string());
        if mode == "development" {
            "dev_secret".to_string()
        } else {
            // geen panic: liever duidelijke 401/500 flow
            "NOT_SET".to_string()
        }
    })
}

pub fn check_secret(req: &HttpRequest) -> Result<(), HttpResponse> {
    let got = req
        .headers()
        .get("X-Secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let expected = expected_secret();

    if expected == "NOT_SET" {
        return Err(HttpResponse::InternalServerError().json(json!({
            "type": { "code": 500, "description": "Internal Server Error" },
            "message": "LUZMO_PLUGIN_SECRET is not set"
        })));
    }

    if got != expected {
        return Err(HttpResponse::Unauthorized().json(json!({
            "type": { "code": 401, "description": "Unauthorized" },
            "message": "Missing or invalid X-Secret"
        })));
    }

    Ok(())
}