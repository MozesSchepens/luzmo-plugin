use std::collections::HashMap;

use crate::luzmo::types::{Column, FilterExpr, QueryRequest};

#[derive(Debug, Clone)]
pub struct GroupCol {
    pub id: String,
    pub level: Option<String>,
    pub col_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Measure {
    pub id: String,
    pub agg: String,
    pub col_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct QueryPlan {
    pub dataset_id: String,
    pub group_cols: Vec<GroupCol>,
    pub measures: Vec<Measure>,
    pub filters: Option<Vec<FilterExpr>>,
    pub limit: Option<usize>,
    pub has_agg: bool,
    pub requested_cols: Vec<Column>,
}

fn col_id(c: &Column) -> String {
    c.column_id
        .clone()
        .or_else(|| if c.id.is_empty() { None } else { Some(c.id.clone()) })
        .unwrap_or_default()
}

pub fn build_plan(req: &QueryRequest, col_index: HashMap<String, usize>) -> Result<QueryPlan, String> {
    let dataset_id = req
        .dataset_id
        .clone()
        .or_else(|| req.id.clone())
        .unwrap_or_default();

    let cols = req.columns.clone().unwrap_or_default();

    // validate ids (behalve "*" bij count)
    for c in &cols {
        let cid = col_id(c);
        if cid == "*" {
            continue;
        }
        if !cid.is_empty() && !col_index.contains_key(&cid) {
            return Err(format!("Unknown column in request: {}", cid));
        }
    }

    let has_agg = cols.iter().any(|c| c.aggregation.as_deref().unwrap_or("").len() > 0);

    let mut group_cols = vec![];
    let mut measures = vec![];

    for c in &cols {
        let cid = col_id(c);
        let agg = c.aggregation.clone().unwrap_or_default();

        if agg.is_empty() {
            group_cols.push(GroupCol {
                id: cid,
                level: c.level.clone(),
                col_type: c.r#type.clone(),
            });
        } else {
            measures.push(Measure {
                id: cid,
                agg,
                col_type: c.r#type.clone(),
            });
        }
    }

    Ok(QueryPlan {
        dataset_id,
        group_cols,
        measures,
        filters: req.filters.clone(),
        limit: req.limit,
        has_agg,
        requested_cols: cols,
    })
}