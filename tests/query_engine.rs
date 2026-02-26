use luzmo_plugin::engine::execute::run;
use luzmo_plugin::luzmo::types::{Column, QueryRequest};

#[test]
fn raw_query_returns_rows() {
    // columns = None => "raw mode" (alle kolommen)
    let req = QueryRequest {
        dataset_id: Some("demo".to_string()),
        limit: Some(20),
        ..Default::default()
    };

    let rows = run(&req).unwrap();
    assert!(rows.len() > 0);
    assert!(rows[0].len() >= 3); // category, date, value
}

#[test]
fn non_agg_selects_requested_columns() {
    let req = QueryRequest {
        dataset_id: Some("demo".to_string()),
        columns: Some(vec![
            Column { id: "category".into(), ..Default::default() },
            Column { id: "date".into(), ..Default::default() },
        ]),
        limit: Some(10),
        ..Default::default()
    };

    let rows = run(&req).unwrap();
    assert!(rows.len() > 0);
    assert_eq!(rows[0].len(), 2);
    assert!(rows[0][0].is_string());
    assert!(rows[0][1].is_string());
}

#[test]
fn agg_query_groups_and_sums() {
    let req = QueryRequest {
        dataset_id: Some("demo".to_string()),
        columns: Some(vec![
            Column { id: "category".into(), ..Default::default() },
            Column { id: "value".into(), aggregation: Some("sum".into()), ..Default::default() },
        ]),
        limit: Some(10),
        ..Default::default()
    };

    let rows = run(&req).unwrap();
    assert!(rows.len() > 0);
    assert_eq!(rows[0].len(), 2); // category + sum(value)
}