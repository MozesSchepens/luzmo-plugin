use serde_json::{json, Value};

pub fn get_dataset() -> Value {
    json!({
        "id": "demo",
        "name": {"en": "Sample Dataset"},
        "description": {"en": "Demo dataset with category, date and value metrics"},
        "properties": {
            "row_limit": 100000,
            "supports_pushdown": true,
            "supports_sorting": true
        },
        "columns": [
            {
                "id": "category",
                "name": {"en": "Category"},
                "type": "string",
                "properties": {"filterable": true, "groupable": true}
            },
            {
                "id": "date",
                "name": {"en": "Date"},
                "type": "datetime",
                "subtype": "date",
                "properties": {"filterable": true, "groupable": true}
            },
            {
                "id": "value",
                "name": {"en": "Value"},
                "type": "numeric",
                "properties": {"filterable": true, "aggregable": true}
            }
        ]
    })
}