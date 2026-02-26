use serde::{Deserialize, Serialize};
use serde_json::Value;
// Common types for the Luzmo plugin
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Column {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub column_id: Option<String>,
    #[serde(default)]
    pub aggregation: Option<String>,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilterExpr {
    #[serde(default)]
    pub column_id: Option<String>,
    #[serde(default)]
    pub id: Option<String>,

    #[serde(default, alias = "operator")]
    pub expression: Option<String>,

    #[serde(default, alias = "values")]
    pub value: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SortExpr {
    #[serde(default)]
    pub column_id: Option<String>,
    #[serde(default)]
    pub column_index: Option<usize>,
    #[serde(default)]
    pub index: Option<usize>,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default)]
    pub order: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct QueryOptions {
    #[serde(default)]
    pub pushdown: bool,
    #[serde(default)]
    pub include_metadata: bool,
    #[serde(default)]
    pub sort: Option<Vec<SortExpr>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct QueryRequest {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub dataset_id: Option<String>,
    #[serde(default)]
    pub columns: Option<Vec<Column>>,
    #[serde(default)]
    pub filters: Option<Vec<FilterExpr>>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub options: Option<QueryOptions>,
}