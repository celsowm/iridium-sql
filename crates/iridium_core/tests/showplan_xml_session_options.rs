use iridium_core::{parse_batch, types::Value, Engine, QueryResult};
use iridium_core::ast::Statement;

fn exec_then_select(set_sql: &str) -> QueryResult {
    let engine = Engine::new();
    let set_stmts = parse_batch(set_sql).expect("parse SET failed");
    engine.execute_batch(set_stmts).expect("SET execute failed");

    let select_stmts = parse_batch("SELECT 1 AS x").expect("parse SELECT failed");
    engine
        .execute_batch(select_stmts)
        .expect("SELECT execute failed")
        .expect("expected a result set")
}

#[test]
fn test_set_showplan_xml_on_intercepts_select() {
    let r = exec_then_select("SET SHOWPLAN_XML ON");
    assert_eq!(r.rows.len(), 1);
    match &r.rows[0][0] {
        Value::NVarChar(xml) => {
            assert!(xml.contains("<ShowPlanXML"), "got: {xml}");
            assert!(
                xml.contains("schemas.microsoft.com/sqlserver/2004/07/showplan"),
                "missing namespace"
            );
        }
        other => panic!("expected NVarChar XML, got {other:?}"),
    }
}

#[test]
fn test_set_statistics_xml_on_intercepts_select() {
    let r = exec_then_select("SET STATISTICS XML ON");
    assert_eq!(r.rows.len(), 1);
    match &r.rows[0][0] {
        Value::NVarChar(xml) => {
            assert!(xml.contains("<ShowPlanXML"));
            assert!(xml.contains("RunTimeInformation"));
        }
        other => panic!("expected NVarChar XML, got {other:?}"),
    }
}

#[test]
fn test_set_showplan_xml_off_does_not_intercept() {
    let r = exec_then_select("SET SHOWPLAN_XML OFF");
    assert_eq!(r.rows.len(), 1);
    match &r.rows[0][0] {
        Value::Int(v) => assert_eq!(*v, 1),
        other => panic!("expected Int 1, got {other:?}"),
    }
}

#[test]
fn test_set_statistics_xml_off_does_not_intercept() {
    let r = exec_then_select("SET STATISTICS XML OFF");
    assert_eq!(r.rows.len(), 1);
    match &r.rows[0][0] {
        Value::Int(v) => assert_eq!(*v, 1),
        other => panic!("expected Int 1, got {other:?}"),
    }
}

#[test]
fn test_showplan_xml_underscore_syntax_accepted() {
    let r = exec_then_select("SET SHOWPLAN_XML ON");
    assert!(matches!(&r.rows[0][0], Value::NVarChar(_)));
}

#[test]
fn test_statistics_xml_underscore_syntax_accepted() {
    let r = exec_then_select("SET STATISTICS_XML ON");
    assert!(matches!(&r.rows[0][0], Value::NVarChar(_)));
}

#[test]
fn test_showplan_xml_space_syntax_accepted() {
    let r = exec_then_select("SET SHOWPLAN XML ON");
    assert!(matches!(&r.rows[0][0], Value::NVarChar(_)));
}

#[test]
fn test_statistics_xml_space_syntax_accepted() {
    let r = exec_then_select("SET STATISTICS XML ON");
    assert!(matches!(&r.rows[0][0], Value::NVarChar(_)));
}

#[test]
fn test_combined_batch_showplan_xml_returns_xml_and_no_row_data() {
    let stmts: Vec<Statement> = parse_batch("SET SHOWPLAN_XML ON; SELECT 1 AS x")
        .expect("parse failed");
    let engine = Engine::new();
    let r = engine
        .execute_batch(stmts)
        .expect("execute failed")
        .expect("expected result");
    assert_eq!(r.rows.len(), 1);
    match &r.rows[0][0] {
        Value::NVarChar(_) => {}
        other => panic!("expected NVarChar XML, got {other:?}"),
    }
}
