use iridium_core::{parse_batch, types::Value, Engine, QueryResult};

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
    col1 VARCHAR(50) NULL DEFAULT 'hello',
    col2 INT NOT NULL DEFAULT 0,
    col3 DATETIME NULL DEFAULT GETDATE()
);
";

#[test]
fn edit_data_create_and_fetch_top_n_rows() {
    let mut engine = Engine::new();
    exec(&mut engine, CREATE_TABLE);
    exec(&mut engine, "INSERT INTO dbo.MyTable (id, col1) VALUES (1, 'a')");
    exec(&mut engine, "INSERT INTO dbo.MyTable (id, col1) VALUES (2, 'b')");
    let r = query(&mut engine, "SELECT TOP (200) * FROM dbo.MyTable ORDER BY id");
    assert_eq!(r.rows.len(), 2);
    assert!(matches!(&r.rows[0][0], Value::Int(1)));
    assert!(matches!(&r.rows[1][0], Value::Int(2)));
}

#[test]
fn edit_data_update_cell_by_primary_key() {
    let mut engine = Engine::new();
    exec(&mut engine, CREATE_TABLE);
    exec(&mut engine, "INSERT INTO dbo.MyTable (id, col1) VALUES (1, 'a')");
    exec(&mut engine, "UPDATE dbo.MyTable SET col1 = 'z' WHERE id = 1");
    let r = query(&mut engine, "SELECT col1 FROM dbo.MyTable WHERE id = 1");
    assert_eq!(r.rows.len(), 1);
    match &r.rows[0][0] {
        Value::VarChar(s) => assert_eq!(s, "z"),
        other => panic!("expected VarChar 'z', got {other:?}"),
    }
}

#[test]
fn edit_data_delete_row_by_primary_key() {
    let mut engine = Engine::new();
    exec(&mut engine, CREATE_TABLE);
    exec(&mut engine, "INSERT INTO dbo.MyTable (id, col1) VALUES (1, 'a')");
    exec(&mut engine, "INSERT INTO dbo.MyTable (id, col1) VALUES (2, 'b')");
    exec(&mut engine, "DELETE FROM dbo.MyTable WHERE id = 1");
    let r = query(&mut engine, "SELECT id FROM dbo.MyTable ORDER BY id");
    assert_eq!(r.rows.len(), 1);
    assert!(matches!(&r.rows[0][0], Value::Int(2)));
}

#[test]
fn edit_data_insert_row_with_default_applied() {
    let mut engine = Engine::new();
    exec(&mut engine, CREATE_TABLE);
    exec(&mut engine, "INSERT INTO dbo.MyTable (id) VALUES (5)");
    let r = query(&mut engine, "SELECT col2 FROM dbo.MyTable WHERE id = 5");
    assert_eq!(r.rows.len(), 1);
    assert!(matches!(&r.rows[0][0], Value::Int(0)), "default 0 applied");
}

#[test]
fn edit_data_revert_rereads_after_rollback() {
    let mut engine = Engine::new();
    exec(&mut engine, CREATE_TABLE);
    exec(&mut engine, "INSERT INTO dbo.MyTable (id, col1) VALUES (1, 'orig')");
    exec(&mut engine, "BEGIN TRANSACTION");
    exec(&mut engine, "UPDATE dbo.MyTable SET col1 = 'temp' WHERE id = 1");
    exec(&mut engine, "ROLLBACK");
    let r = query(&mut engine, "SELECT col1 FROM dbo.MyTable WHERE id = 1");
    match &r.rows[0][0] {
        Value::VarChar(s) => assert_eq!(s, "orig"),
        other => panic!("expected VarChar 'orig', got {other:?}"),
    }
}

#[test]
fn edit_data_sys_tables_editability_probe_all_tables_editable() {
    let mut engine = Engine::new();
    exec(&mut engine, CREATE_TABLE);
    let r = query(
        &mut engine,
        "SELECT t.is_memory_optimized, t.temporal_type, t.is_edge, t.is_node FROM sys.tables t WHERE t.name = 'MyTable'",
    );
    assert_eq!(r.rows.len(), 1);
    let row = &r.rows[0];
    assert!(matches!(row[0], Value::Bit(false)), "not memory_optimized");
    assert!(matches!(row[1], Value::TinyInt(0)), "temporal_type=0");
    assert!(matches!(row[2], Value::Bit(false)), "not edge");
    assert!(matches!(row[3], Value::Bit(false)), "not node");
}

#[test]
fn edit_data_sys_tables_columns_get_metadata() {
    let mut engine = Engine::new();
    exec(&mut engine, CREATE_TABLE);
    let r = query(
        &mut engine,
        "SELECT c.name, c.is_identity, c.is_computed, c.is_nullable FROM sys.columns c WHERE OBJECT_NAME(c.object_id) = 'MyTable' ORDER BY c.column_id",
    );
    assert_eq!(r.rows.len(), 4);
    // id INT NOT NULL PRIMARY KEY
    match &r.rows[0][0] {
        Value::VarChar(s) => assert_eq!(s, "id"),
        other => panic!("expected 'id', got {other:?}"),
    }
    assert!(matches!(&r.rows[0][3], Value::Bit(false)), "id NOT NULL");
    // col1 VARCHAR(50) NULL
    match &r.rows[1][0] {
        Value::VarChar(s) => assert_eq!(s, "col1"),
        other => panic!("expected 'col1', got {other:?}"),
    }
    assert!(matches!(&r.rows[1][3], Value::Bit(true)), "col1 NULL");
}
