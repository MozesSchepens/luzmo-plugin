use std::collections::HashMap;

use serde_json::Value;

use crate::engine::aggregation::{bucket_month, execute_aggregation};
use crate::engine::dataset::{col_index_map, generate_data};
use crate::engine::filters::apply_filters;
use crate::engine::plan::{build_plan, QueryPlan};
use crate::luzmo::types::{Column, QueryRequest};
use crate::utils::sanitize::{normalize_value, sanitize_json_value};

fn col_id(c: &Column) -> String {
    c.column_id
        .clone()
        .or_else(|| if c.id.is_empty() { None } else { Some(c.id.clone()) })
        .unwrap_or_default()
}

pub fn run(req: &QueryRequest) -> Result<Vec<Vec<Value>>, String> {
    execute_query(req)
}

pub fn execute_query(req: &QueryRequest) -> Result<Vec<Vec<Value>>, String> {
    let data = generate_data();
    let col_index: HashMap<String, usize> = col_index_map();
    
    eprintln!("DEBUG: Request - id: {:?}, dataset_id: {:?}, columns: {:?}", req.id, req.dataset_id, req.columns);

    // filters
    let filtered = apply_filters(&data, req.filters.clone(), &col_index)?;
    eprintln!("DEBUG: Filtered rows: {}", filtered.len());

    // plan (heeft has_agg + requested_cols)
    let plan: QueryPlan = build_plan(req, col_index.clone())?;

    // geen columns: raw
    if plan.requested_cols.is_empty() {
        let mut out = vec![];
        for mut r in filtered {
            for cell in r.iter_mut() {
                *cell = sanitize_json_value(normalize_value(cell));
            }
            out.push(r);
        }
        if let Some(l) = req.limit {
            out.truncate(l);
        }
        return Ok(out);
    }

    // raw mode
    if !plan.has_agg {
        let mut out: Vec<Vec<Value>> = Vec::with_capacity(filtered.len());

        for r in filtered {
            let mut row_out = Vec::with_capacity(plan.requested_cols.len());

            for c in &plan.requested_cols {
                let cid = col_id(c);
                let idx = *col_index
                    .get(&cid)
                    .ok_or_else(|| format!("Unknown column in request: {}", cid))?;

                let mut v = r.get(idx).cloned().unwrap_or(Value::Null);
                v = normalize_value(&v);

                if cid == "date" && c.level.as_deref() == Some("month") {
                    v = bucket_month(&v);
                }
                
                row_out.push(sanitize_json_value(v));
                
            }

            out.push(row_out);
        }

        if let Some(l) = req.limit {
            out.truncate(l);
        }
        return Ok(out);
    }

    // agg mode
    let mut out = execute_aggregation(&filtered, &plan, &col_index)?;

    for row in out.iter_mut() {
        for cell in row.iter_mut() {
            if let Value::Array(arr) = cell {
                if arr.len() == 1 {
                    *cell = arr[0].clone();
                }
            }
        }
    }

    if let Some(l) = req.limit {
        out.truncate(l);
    }
Ok(out)
}