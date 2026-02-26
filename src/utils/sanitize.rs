use serde_json::{json, Map, Value};

pub fn normalize_value(v: &Value) -> Value {
    match v {
        Value::Array(arr) if arr.len() == 1 => arr[0].clone(),
        _ => v.clone(),
    }
}

// Extra veilig: recurse door arrays/objects.
// (serde_json kan geen NaN/Infinity opslaan als Number,
// maar dit is handig als je ooit floats verwerkt via Value::from_f64.)
pub fn sanitize_json_value(v: Value) -> Value {
    match v {
        Value::Array(arr) => Value::Array(arr.into_iter().map(sanitize_json_value).collect()),
        Value::Object(obj) => {
            let mut out = Map::new();
            for (k, val) in obj {
                out.insert(k, sanitize_json_value(val));
            }
            Value::Object(out)
        }
        other => other,
    }
}

pub fn get_dataset() -> serde_json::Value {
    json!({
      "id": "demo",
      "name": { "en": "Sample Dataset" },
      "description": { "en": "Demo dataset with category, date and value metrics" },
      "properties": {
        "row_limit": 100000,
        "supports_pushdown": true,
        "supports_sorting": false
      },
      "columns": [
        {
          "id": "category",
          "name": { "en": "Category" },
          "type": "hierarchy",
          "properties": {
            "display_name": { "en": "Category" },
            "filterable": true,
            "groupable": true
          }
        },
        {
          "id": "date",
          "name": { "en": "Date" },
          "type": "datetime",
          "subtype": "date",
          "properties": {
            "display_name": { "en": "Date" },
            "filterable": true,
            "groupable": true,
            "format": "YYYY-MM-DD"
          }
        },
        {
          "id": "value",
          "name": { "en": "Value" },
          "type": "numeric",
          "properties": {
            "display_name": { "en": "Value" },
            "filterable": true,
            "aggregable": true,
            "format": "0.00"
          }
        }
      ]
    })
}