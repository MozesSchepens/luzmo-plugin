use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;

use crate::engine::execute::run;
use crate::luzmo::types::QueryRequest;
use crate::utils::ids::make_req_id;
use crate::utils::secret::check_secret;

pub async fn handle_query(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    let rid = make_req_id();
    
    // Log request details
    println!("[{}] ===== NEW QUERY REQUEST =====", rid);
    println!("[{}] Headers: {:?}", rid, req.headers());
    
    if let Err(resp) = check_secret(&req) {
        println!("[{}] ✗ Secret check failed", rid);
        return resp;
    }

    let raw = String::from_utf8_lossy(&body);
    println!("[{}] /query raw body:\n{}", rid, raw);

    let parsed: Result<QueryRequest, _> = serde_json::from_slice(body.as_ref());
    let q = match parsed {
        Ok(v) => v,
        Err(e) => {
            println!("[{}] Json deserialize error: {}", rid, e);
            return HttpResponse::BadRequest().json(json!({
                "type": { "code": 400, "description": "Bad Request" },
                "message": format!("Json deserialize error: {}", e)
            }));
        }
    };

    let dataset_id = q.dataset_id.clone().or_else(|| q.id.clone()).unwrap_or_default();
    if dataset_id != "demo" {
        return HttpResponse::NotFound().json(json!({
            "type": { "code": 404, "description": "Not Found" },
            "message": format!("Unknown dataset id: {}", dataset_id)
        }));
    }

    match run(&q) {
        Ok(rows) => {
            println!("[{}] ✓ rows_out={}", rid, rows.len());
            HttpResponse::Ok().json(rows)
        }
        Err(e) => {
            println!("[{}] ❌ Query error: {}", rid, e);
            HttpResponse::InternalServerError().json(json!({
                "type": { "code": 500, "description": "Internal Server Error" },
                "message": format!("Query error: {}", e)
            }))
        }
    }
}