use std::time::{SystemTime, UNIX_EPOCH};

pub fn make_req_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{:x}", now)
}