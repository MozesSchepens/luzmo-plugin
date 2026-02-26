use serde_json::Value;
use std::collections::HashMap;

use crate::luzmo::types::FilterExpr;
use crate::utils::sanitize::normalize_value;
// Filter application logic
fn resolve_column_id(f: &FilterExpr) -> Option<String> {
    f.column_id.clone().or_else(|| f.id.clone())
}

fn cmp_gt(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(an), Value::Number(bn)) => an.as_f64().unwrap_or(0.0) > bn.as_f64().unwrap_or(0.0),
        (Value::String(as_), Value::String(bs)) => as_ > bs,
        _ => false,
    }
}
fn cmp_ge(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(an), Value::Number(bn)) => an.as_f64().unwrap_or(0.0) >= bn.as_f64().unwrap_or(0.0),
        (Value::String(as_), Value::String(bs)) => as_ >= bs,
        _ => false,
    }
}
fn cmp_lt(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(an), Value::Number(bn)) => an.as_f64().unwrap_or(0.0) < bn.as_f64().unwrap_or(0.0),
        (Value::String(as_), Value::String(bs)) => as_ < bs,
        _ => false,
    }
}
fn cmp_le(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(an), Value::Number(bn)) => an.as_f64().unwrap_or(0.0) <= bn.as_f64().unwrap_or(0.0),
        (Value::String(as_), Value::String(bs)) => as_ <= bs,
        _ => false,
    }
}
fn normalize_op(op: Option<&str>) -> Option<&str> {
    match op?.trim(){
        ">"|"greater_than" => Some(">"),
        ">="|"greater_than_or_equal" => Some(">="),
        "<"|"less_than" => Some("<"),
        "<="|"less_than_or_equal" => Some("<="),
        "="|"=="|"equal" => Some("=="),
        "!="|"!=="|"not_equal" => Some("!="),
        "in" => Some("in"),
        "contains"| "like" => Some("contains"),
        "is missing"|"is null" =>Some("is null"),
        "is not missing"|"is not null" => Some("is not null"),
        other => Some(other),
    }
}

pub fn apply_filters(
    rows: &[Vec<Value>],
    filters: Option<Vec<FilterExpr>>,
    col_index: &HashMap<String, usize>,
) -> Result<Vec<Vec<Value>>, String> {
    let Some(filters) = filters else {
        return Ok(rows.to_vec());
    };
    if filters.is_empty() {
        return Ok(rows.to_vec());
    }

    let mut result: Vec<Vec<Value>> = rows.to_vec();

    for f in filters {
        let col = resolve_column_id(&f).unwrap_or_default();
        let expr = normalize_op(f.expression.as_deref());
        let idx = match col_index.get(&col) {
            Some(&i) => i,
            None => continue,
        };

        let raw_val = f.value.as_ref().map(normalize_value);

        result = match expr {
            Some("is not null") => result
                .into_iter()
                .filter(|row| row.get(idx).map(|v| !v.is_null()).unwrap_or(false))
                .collect(),

            Some("in") => {
                let vals = match f.value {
                    Some(Value::Array(v)) => v,
                    Some(v) => vec![v],
                    None => continue,
                };
                let vals_norm: Vec<Value> = vals.into_iter().map(|x| normalize_value(&x)).collect();

                result
                    .into_iter()
                    .filter(|row| {
                        row.get(idx)
                            .map(|v| {
                                let v = normalize_value(v);
                                vals_norm.contains(&v)
                            })
                            .unwrap_or(false)
                    })
                    .collect()
            }

            Some("=") | Some("==") => {
                let cmp_val = raw_val.clone().unwrap_or(Value::Null);
                result
                    .into_iter()
                    .filter(|row| row.get(idx).map(|v| normalize_value(v) == cmp_val).unwrap_or(false))
                    .collect()
            }

            Some("!=") | Some("!==") => {
                let cmp_val = raw_val.clone().unwrap_or(Value::Null);
                result
                    .into_iter()
                    .filter(|row| row.get(idx).map(|v| normalize_value(v) != cmp_val).unwrap_or(false))
                    .collect()
            }

            Some(">=") => {
                let cmp_val = raw_val.clone().unwrap_or(Value::Null);
                result.into_iter().filter(|row| row.get(idx).map_or(false, |v| cmp_ge(v, &cmp_val))).collect()
            }

            Some(">") => {
                let cmp_val = raw_val.clone().unwrap_or(Value::Null);
                result.into_iter().filter(|row| row.get(idx).map_or(false, |v| cmp_gt(v, &cmp_val))).collect()
            }

            Some("<=") => {
                let cmp_val = raw_val.clone().unwrap_or(Value::Null);
                result.into_iter().filter(|row| row.get(idx).map_or(false, |v| cmp_le(v, &cmp_val))).collect()
            }

            Some("<") => {
                let cmp_val = raw_val.clone().unwrap_or(Value::Null);
                result.into_iter().filter(|row| row.get(idx).map_or(false, |v| cmp_lt(v, &cmp_val))).collect()
            }

            Some("contains") | Some("like") => {
                let needle = raw_val
                    .as_ref()
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_lowercase();

                result
                    .into_iter()
                    .filter(|row| {
                        row.get(idx)
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&needle)
                    })
                    .collect()
            }

            _ => result,
        };
    }

    Ok(result)
}