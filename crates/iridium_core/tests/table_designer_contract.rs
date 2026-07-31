use iridium_core::{parse_batch, types::Value, Engine, QueryResult};

fn str_value(v: &Value) -> String {
    match v {
        Value::VarChar(s) | Value::NVarChar(s) => s.clone(),
        _ => panic!("expected string value, got {v:?}"),
    }
}

fn exec(engine: &mut Engine, sql: &str) {
    let stmts = parse_batch(sql).expect("parse failed");
    engine.execute_batch(stmts).expect("execute failed");
}

fn query(engine: &mut Engine, sql: &str) -> QueryResult {
    let stmts = parse_batch(sql).expect("parse failed");
    engine
        .execute_batch(stmts)
        .expect("execute failed")
        .expect("expected result set")
}

const CREATE_TABLE: &str = "
CREATE TABLE dbo.MyTable (
    id INT NOT NULL PRIMARY KEY,
    col1 VARCHAR(50) NULL,
    col2 INT NOT NULL DEFAULT 0
);
";

#[test]
fn table_designer_init_returns_table_definition() {
    let mut engine = Engine::new();
    exec(&mut engine, CREATE_TABLE);
    let r = query(
        &mut engine,
        "SELECT t.name, SCHEMA_NAME(t.schema_id) FROM sys.tables t WHERE t.object_id = OBJECT_ID('dbo.MyTable')",
    );
    assert_eq!(r.rows.len(), 1);
    match &r.rows[0][0] {
        Value::VarChar(s) | Value::NVarChar(s) => assert_eq!(s, "MyTable"),
        other => panic!("expected 'MyTable', got {other:?}"),
    }
}

#[test]
fn table_designer_init_columns_listing() {
    let mut engine = Engine::new();
    exec(&mut engine, CREATE_TABLE);
    let r = query(
        &mut engine,
        "SELECT c.name, c.is_nullable, c.is_identity FROM sys.columns c WHERE c.object_id = OBJECT_ID('dbo.MyTable') ORDER BY c.column_id",
    );
    assert_eq!(r.rows.len(), 3);
    // Verify ordering: id, col1, col2
    assert_eq!(str_value(&r.rows[0][0]), "id");
    assert_eq!(str_value(&r.rows[1][0]), "col1");
    assert_eq!(str_value(&r.rows[2][0]), "col2");
    // id is NOT NULL
    assert!(matches!(&r.rows[0][1], Value::Bit(false)));
    // col1 IS NULL
    assert!(matches!(&r.rows[1][1], Value::Bit(true)));
}

#[test]
fn table_designer_init_primary_key_columns() {
    let mut engine = Engine::new();
    exec(&mut engine, CREATE_TABLE);
    let r = query(
        &mut engine,
        "SELECT i.name, i.is_primary_key FROM sys.indexes i WHERE i.object_id = OBJECT_ID('dbo.MyTable') AND i.is_primary_key = 1",
    );
    assert_eq!(r.rows.len(), 1);
    // Confirm this index is marked as primary key
    assert!(matches!(&r.rows[0][1], Value::Bit(true)));
}

#[test]
fn table_designer_publish_add_column_via_alter_table() {
    let mut engine = Engine::new();
    exec(&mut engine, CREATE_TABLE);
    exec(&mut engine, "ALTER TABLE dbo.MyTable ADD newcol INT NULL");
    let r = query(
        &mut engine,
        "SELECT c.name FROM sys.columns c WHERE c.object_id = OBJECT_ID('dbo.MyTable') ORDER BY c.column_id",
    );
    let col_names: Vec<String> = r
        .rows
        .into_iter()
        .map(|r| str_value(&r[0]))
        .collect();
    assert!(col_names.contains(&"newcol".to_string()), "newcol present: {col_names:?}");
}

#[test]
fn table_designer_publish_drop_column_via_alter_table() {
    let mut engine = Engine::new();
    exec(&mut engine, CREATE_TABLE);
    exec(&mut engine, "ALTER TABLE dbo.MyTable DROP COLUMN col1");
    let r = query(
        &mut engine,
        "SELECT c.name FROM sys.columns c WHERE c.object_id = OBJECT_ID('dbo.MyTable') ORDER BY c.column_id",
    );
    let col_names: Vec<String> = r
        .rows
        .into_iter()
        .map(|r| str_value(&r[0]))
        .collect();
    assert!(!col_names.contains(&"col1".to_string()), "col1 gone: {col_names:?}");
}

#[test]
fn table_designer_publish_alter_column_nullability() {
    let mut engine = Engine::new();
    exec(&mut engine, CREATE_TABLE);
    exec(&mut engine, "ALTER TABLE dbo.MyTable ALTER COLUMN col1 VARCHAR(100) NOT NULL");
    let r = query(
        &mut engine,
        "SELECT is_nullable FROM sys.columns WHERE object_id = OBJECT_ID('dbo.MyTable') AND name = 'col1'",
    );
    assert_eq!(r.rows.len(), 1);
    assert!(matches!(&r.rows[0][0], Value::Bit(false)), "col1 is now NOT NULL");
}

#[test]
fn table_designer_unsupported_table_family_probe() {
    // Iridium does not model columnstore/temporal/graph/memory-optimized tables,
    // so the editability probe returns all-zero flags for any user table. The
    // deployment side is responsible for emitting UnsupportedFeatureError when
    // any of these are non-zero (which never happens for Iridium tables).
    let mut engine = Engine::new();
    exec(&mut engine, CREATE_TABLE);
    let r = query(
        &mut engine,
        "SELECT t.is_memory_optimized, t.temporal_type, t.is_edge, t.is_node FROM sys.tables t WHERE t.object_id = OBJECT_ID('dbo.MyTable')",
    );
    assert_eq!(r.rows.len(), 1);
    let row = &r.rows[0];
    assert!(matches!(row[0], Value::Bit(false)), "is_memory_optimized=0");
    assert!(matches!(row[1], Value::TinyInt(0)), "temporal_type=0");
    assert!(matches!(row[2], Value::Bit(false)), "is_edge=0");
    assert!(matches!(row[3], Value::Bit(false)), "is_node=0");
}
