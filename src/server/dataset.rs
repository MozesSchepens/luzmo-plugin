use actix_web::{HttpRequest, HttpResponse};
use serde_json::json;

use crate::utils::secret::check_secret;
use crate::utils::sanitize::get_dataset;
pub async fn handle_datasets(req: HttpRequest) -> HttpResponse {
    if let Err(resp) = check_secret(&req) {
        return resp;
    }

    HttpResponse::Ok().json(json!([get_dataset()]))
}