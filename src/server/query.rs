use actix_web::{web, HttpRequest, HttpResponse};

use crate::engine::execute::run;
use crate::errors::PluginError;
use crate::luzmo::types::QueryRequest;
use crate::utils::ids::make_req_id;
use crate::utils::secret::check_secret;

// Main query handler
pub async fn handle_query(
    req: HttpRequest,
    body: web::Bytes,
) -> Result<HttpResponse, PluginError> {
    let rid = make_req_id();

    println!("[{}] ===== NEW QUERY REQUEST =====", rid);
    println!("[{}] Headers: {:?}", rid, req.headers());

    if let Err(resp) = check_secret(&req) {
        println!("[{}] ✗ Secret check failed", rid);
        return Ok(resp);
    }

    let raw = String::from_utf8_lossy(&body);
    println!("[{}] /query raw body:\n{}", rid, raw);

    let q: QueryRequest = serde_json::from_slice(body.as_ref()).map_err(|e| {
        println!("[{}] Json deserialize error: {}", rid, e);
        PluginError::InvalidRequest {
            message: format!("Json deserialize error: {}", e),
        }
    })?;

    let dataset_id = q
        .dataset_id
        .clone()
        .or_else(|| q.id.clone())
        .unwrap_or_default();

    if dataset_id != "demo" {
        return Err(PluginError::DatasetNotFound {
            message: format!("Unknown dataset id: {}", dataset_id),
        });
    }

    let rows = run(&q)?;

    println!("[{}] ✓ rows_out={}", rid, rows.len());
    Ok(HttpResponse::Ok().json(rows))
}