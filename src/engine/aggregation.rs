use std::collections::HashMap;

use serde_json::{json, Value};

use crate::engine::plan::{Measure, QueryPlan};
use crate::utils::sanitize::{normalize_value, sanitize_json_value};

#[derive(Default, Clone)]
struct AggState {
    count: f64,
    sum: f64,
    min: Option<f64>,
    max: Option<f64>,
}

fn round2f(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

pub fn bucket_month(v: &Value) -> Value {
    if let Value::String(s) = v {
        if s.len() >= 7 {
            return Value::String(format!("{}-01T00:00:00.000Z", &s[0..7]));
        }
    }
    v.clone()
}

fn measure_update(st: &mut AggState, m: &Measure, raw: Value) -> Result<(), String> {
    match m.agg.as_str() {
        "count" => {
            if m.id == "*" {
                st.count += 1.0;
                return Ok(());
            }
            let v = normalize_value(&raw);
            if !v.is_null() {
                st.count += 1.0;
            }
            Ok(())
        }
        "sum" | "avg" | "min" | "max" => {
            let v = normalize_value(&raw);
            if let Some(n) = v.as_f64() {
                st.count += 1.0;
                st.sum += n;
                st.min = Some(st.min.map(|m| m.min(n)).unwrap_or(n));
                st.max = Some(st.max.map(|m| m.max(n)).unwrap_or(n));
            }
            Ok(())
        }
        _ => Err(format!("Unsupported aggregation: {}", m.agg)),
    }
}

fn measure_finalize(st: &AggState, m: &Measure) -> Value {
    match m.agg.as_str() {
        "count" => json!(st.count as i64),
        "sum" => json!(round2f(st.sum)),
        "avg" => json!(if st.count > 0.0 { round2f(st.sum / st.count) } else { 0.0 }),
        "min" => json!(round2f(st.min.unwrap_or(0.0))),
        "max" => json!(round2f(st.max.unwrap_or(0.0))),
        _ => Value::Null,
    }
}

pub fn execute_aggregation(
    rows: &[Vec<Value>],
    plan: &QueryPlan,
    col_index: &HashMap<String, usize>,
) -> Result<Vec<Vec<Value>>, String> {
    let mut groups: HashMap<String, (Vec<Value>, Vec<AggState>)> = HashMap::new();

    for r in rows {
        let mut gvals: Vec<Value> = Vec::with_capacity(plan.group_cols.len());

        for g in &plan.group_cols {
            let idx = *col_index
                .get(&g.id)
                .ok_or_else(|| format!("Unknown group column: {}", g.id))?;

            let mut v = r.get(idx).cloned().unwrap_or(Value::Null);
            v = normalize_value(&v);

            if g.id == "date" && g.level.as_deref() == Some("month") {
                v = bucket_month(&v);
            }
            // hierarchy group keys moeten array-pad zijn
            if g.col_type.as_deref() == Some("hierarchy") {
                if let Value::String(_) = v {
                    v = Value::Array(vec![v]);
                }
            }

            gvals.push(sanitize_json_value(v));
        }

        let key = serde_json::to_string(&gvals).unwrap_or_default();

        let entry = groups
            .entry(key)
            .or_insert_with(|| (gvals.clone(), vec![AggState::default(); plan.measures.len()]));

        for (mi, m) in plan.measures.iter().enumerate() {
            let st = &mut entry.1[mi];

            if m.agg == "count" && m.id == "*" {
                measure_update(st, m, Value::Null)?;
                continue;
            }

            let idx = *col_index
                .get(&m.id)
                .ok_or_else(|| format!("Unknown measure column: {}", m.id))?;

            let raw = r.get(idx).cloned().unwrap_or(Value::Null);
            measure_update(st, m, raw)?;
        }
    }

    // deterministisch
    let mut items: Vec<(String, (Vec<Value>, Vec<AggState>))> = groups.into_iter().collect();
    items.sort_by(|a, b| a.0.cmp(&b.0));

    let mut out: Vec<Vec<Value>> = Vec::with_capacity(items.len());
    for (_k, (gvals, mstates)) in items {
        let mut row = vec![];
        row.extend(gvals);

        for (mi, m) in plan.measures.iter().enumerate() {
            row.push(sanitize_json_value(measure_finalize(&mstates[mi], m)));
        }

        out.push(row);
    }

    Ok(out)
}