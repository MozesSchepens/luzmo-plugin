use chrono::{Duration, NaiveDate};
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn col_index_map() -> HashMap<String, usize> {
    let mut map = HashMap::new();
    map.insert("category".to_string(), 0);
    map.insert("date".to_string(), 1);
    map.insert("value".to_string(), 2);
    map
}

pub fn generate_data() -> Vec<Vec<Value>> {
    let mut rows = Vec::new();
    let categories = vec!["A", "B", "C", "D", "E", "F"];
    let mut seed: u64 = 123;

    let mut rng = || {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (seed as f64) / (u64::MAX as f64)
    };
    let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();

    for d in 0..180 {
        let date = start + Duration::days(d);
        let date_str = format!("{}T00:00:00.000Z", date.format("%Y-%m-%d"));
        for category in &categories {
            let n = 1 + (rng() * 5.0) as usize;
            for _ in 0..n {
                let base = 5.0 + rng() * 50.0;
                let seasonal = 10.0 * ((2.0 * std::f64::consts::PI * d as f64) / 30.0).sin();
                let noise = rng() * 5.0;
                let value = (base + seasonal + noise).max(0.0);
                let rounded = (value * 100.0).round() / 100.0;

                rows.push(vec![
                json!(category.to_string()),
                json!(date_str.clone()),
                json!(rounded),
            ]);
            }
        }
    }

    rows
}