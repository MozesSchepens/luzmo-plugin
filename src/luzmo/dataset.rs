use serde_json::{json, Value};

pub fn get_dataset() -> Value {
    json!({
        [
  {
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
  }
]
    })
}