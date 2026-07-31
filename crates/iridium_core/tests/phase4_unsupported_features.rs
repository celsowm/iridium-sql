use iridium_core::{parse_batch, types::Value, Engine, QueryResult};

fn try_exec(engine: &mut Engine, sql: &str) -> bool {
    match parse_batch(sql) {
        Ok(stmts) => {
            for stmt in stmts {
                if engine.execute(stmt).is_err() {
                    return true;
                }
            }
            false
        }
        Err(_) => true,
    }
}

fn query(engine: &mut Engine, sql: &str) -> QueryResult {
    let stmts = parse_batch(sql).expect("parse failed");
    engine
        .execute_batch(stmts)
        .expect("execute failed")
        .expect("expected result set")
}

// ── W4.3: Backup/Restore ──

#[test]
fn test_backup_database_fails() {
    let mut engine = Engine::new();
    assert!(
        try_exec(&mut engine, "BACKUP DATABASE test TO DISK = 'test.bak'"),
        "BACKUP should fail"
    );
}

#[test]
fn test_restore_database_fails() {
    let mut engine = Engine::new();
    assert!(
        try_exec(&mut engine, "RESTORE DATABASE test FROM DISK = 'test.bak'"),
        "RESTORE should fail"
    );
}

// ── W4.5: Principals / roles / grants catalog shims ──

#[test]
fn test_sys_server_principals_queryable() {
    let mut engine = Engine::new();
    let r = query(&mut engine, "SELECT name, type_desc FROM sys.server_principals");
    assert!(!r.rows.is_empty(), "sys.server_principals must have rows");
}

#[test]
fn test_sys_database_principals_returns_rows() {
    let mut engine = Engine::new();
    let r = query(
        &mut engine,
        "SELECT name, type_desc FROM sys.database_principals",
    );
    assert!(
        !r.rows.is_empty(),
        "sys.database_principals must have rows"
    );
    let names: Vec<String> = r
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::NVarChar(s) | Value::VarChar(s) => s.clone(),
            _ => panic!("expected string name"),
        })
        .collect();
    assert!(
        names.iter().any(|n| n == "dbo"),
        "dbo principal must exist; got: {names:?}"
    );
}

#[test]
fn test_sys_database_permissions_queryable() {
    let mut engine = Engine::new();
    let r = query(&mut engine, "SELECT * FROM sys.database_permissions");
    // May be empty for a fresh engine — the query must succeed
    assert!(
        r.rows.is_empty() || !r.rows.is_empty(),
        "sys.database_permissions queryable"
    );
}

#[test]
fn test_sys_database_role_members_queryable() {
    let mut engine = Engine::new();
    let r = query(
        &mut engine,
        "SELECT role_principal_id, member_principal_id FROM sys.database_role_members",
    );
    assert!(
        r.rows.is_empty() || !r.rows.is_empty(),
        "sys.database_role_members queryable"
    );
}

#[test]
fn test_is_srvrolemember_sysadmin() {
    let mut engine = Engine::new();
    let r = query(&mut engine, "SELECT IS_SRVROLEMEMBER('sysadmin')");
    assert_eq!(r.rows.len(), 1);
    match &r.rows[0][0] {
        Value::Int(v) => assert_eq!(*v, 1, "sysadmin returns 1"),
        Value::TinyInt(v) => assert_eq!(*v, 1, "sysadmin returns 1"),
        other => panic!("expected int 1, got {other:?}"),
    }
}

#[test]
fn test_is_srvrolemember_public() {
    let mut engine = Engine::new();
    let r = query(&mut engine, "SELECT IS_SRVROLEMEMBER('public')");
    assert_eq!(r.rows.len(), 1);
    match &r.rows[0][0] {
        Value::Int(v) => assert!(*v == 0 || *v == 1, "valid result: {v}"),
        Value::TinyInt(v) => assert!(*v == 0 || *v == 1, "valid result: {v}"),
        other => panic!("expected int, got {other:?}"),
    }
}

#[test]
fn test_is_member_db_owner() {
    let mut engine = Engine::new();
    let r = query(&mut engine, "SELECT IS_MEMBER('db_owner')");
    assert_eq!(r.rows.len(), 1);
    match &r.rows[0][0] {
        Value::Int(v) => assert_eq!(*v, 1, "db_owner returns 1"),
        Value::TinyInt(v) => assert_eq!(*v, 1, "db_owner returns 1"),
        other => panic!("expected int 1, got {other:?}"),
    }
}

#[test]
fn test_has_perms_by_name_connect_sql() {
    let mut engine = Engine::new();
    let r = query(
        &mut engine,
        "SELECT HAS_PERMS_BY_NAME(NULL, NULL, 'CONNECT SQL')",
    );
    assert_eq!(r.rows.len(), 1);
    match &r.rows[0][0] {
        Value::Int(v) => assert_eq!(*v, 1, "CONNECT SQL grants 1"),
        Value::TinyInt(v) => assert_eq!(*v, 1, "CONNECT SQL grants 1"),
        other => panic!("expected int 1, got {other:?}"),
    }
}

#[test]
fn test_has_perms_by_name_view_server_state() {
    let mut engine = Engine::new();
    let r = query(
        &mut engine,
        "SELECT HAS_PERMS_BY_NAME(NULL, NULL, 'VIEW SERVER STATE')",
    );
    assert_eq!(r.rows.len(), 1);
    match &r.rows[0][0] {
        Value::Int(v) => assert_eq!(*v, 1, "VIEW SERVER STATE grants 1"),
        Value::TinyInt(v) => assert_eq!(*v, 1, "VIEW SERVER STATE grants 1"),
        other => panic!("expected int 1, got {other:?}"),
    }
}

#[test]
fn test_has_dbaccess() {
    let mut engine = Engine::new();
    let r = query(&mut engine, "SELECT HAS_DBACCESS('master')");
    assert_eq!(r.rows.len(), 1);
    match &r.rows[0][0] {
        Value::Int(v) => assert_eq!(*v, 1, "HAS_DBACCESS master returns 1"),
        Value::TinyInt(v) => assert_eq!(*v, 1, "HAS_DBACCESS master returns 1"),
        other => panic!("expected int 1, got {other:?}"),
    }
}

// ── W4.6: Advanced Table Designer families / unsupported DDL ──

#[test]
fn test_create_database_unsupported() {
    let mut engine = Engine::new();
    assert!(
        try_exec(&mut engine, "CREATE DATABASE test_db"),
        "CREATE DATABASE should fail (single-db mode)"
    );
}

#[test]
fn test_alter_database_set_unsupported() {
    let mut engine = Engine::new();
    assert!(
        try_exec(&mut engine, "ALTER DATABASE master SET READ_ONLY"),
        "ALTER DATABASE SET should fail"
    );
}

#[test]
fn test_create_assembly_unsupported() {
    let mut engine = Engine::new();
    assert!(
        try_exec(&mut engine, "CREATE ASSEMBLY test FROM 'test.dll'"),
        "CREATE ASSEMBLY should fail"
    );
}

#[test]
fn test_sp_add_job_unsupported() {
    let mut engine = Engine::new();
    assert!(
        try_exec(&mut engine, "EXEC sp_add_job @job_name = 'test'"),
        "sp_add_job should fail (SQL Agent)"
    );
}

#[test]
fn test_create_partition_function_unsupported() {
    let mut engine = Engine::new();
    assert!(
        try_exec(
            &mut engine,
            "CREATE PARTITION FUNCTION pf(int) AS RANGE LEFT FOR VALUES (1, 100, 1000)"
        ),
        "CREATE PARTITION FUNCTION should fail"
    );
}

#[test]
fn test_create_message_type_unsupported() {
    let mut engine = Engine::new();
    assert!(
        try_exec(&mut engine, "CREATE MESSAGE TYPE TestMessage"),
        "CREATE MESSAGE TYPE should fail (Service Broker)"
    );
}

#[test]
fn test_create_certificate_unsupported() {
    let mut engine = Engine::new();
    assert!(
        try_exec(&mut engine, "CREATE CERTIFICATE my_cert FROM FILE = 'cert.cer'"),
        "CREATE CERTIFICATE should fail"
    );
}

// ── W4.1/W4.2: Schema Compare / DACPAC (no engine-side test needed,
//    only fixture stubs; but we verify the unsupported signpost is coherent) ──

#[test]
fn test_alter_authorization_unsupported() {
    let mut engine = Engine::new();
    assert!(
        try_exec(
            &mut engine,
            "ALTER AUTHORIZATION ON DATABASE::master TO sa"
        ),
        "ALTER AUTHORIZATION should fail"
    );
}
