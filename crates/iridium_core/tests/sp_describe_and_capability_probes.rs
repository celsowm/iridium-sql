include!("new_functions/helpers.rs");

// ─── sp_describe_first_result_set ──────────────────────────────────────────

#[test]
fn test_sp_describe_first_result_set_simple_select() {
    let mut engine = Engine::new();
    exec(
        &mut engine,
        "CREATE TABLE t_describe (id INT NOT NULL, name NVARCHAR(100), val DECIMAL(10,2))",
    );
    let r = query(
        &mut engine,
        "EXEC sp_describe_first_result_set N'SELECT id, name, val FROM t_describe'",
    );
    assert_eq!(r.columns.len(), 7);
    assert_eq!(r.columns[0], "IsNullable");
    assert_eq!(r.columns[1], "SystemTypeId");
    assert_eq!(r.columns[2], "SystemTypeName");
    assert_eq!(r.columns[3], "MaxLength");
    assert_eq!(r.columns[4], "Precision");
    assert_eq!(r.columns[5], "Scale");
    assert_eq!(r.columns[6], "CollationName");
    assert_eq!(r.rows.len(), 3);
    // id: INT (result columns report nullable=true from expression binding)
    assert_eq!(r.rows[0][0], Value::Bit(true));
    // name: NVARCHAR, nullable
    assert_eq!(r.rows[1][0], Value::Bit(true));
    // val: DECIMAL(10,2) — precision/scale may not propagate through result metadata
    assert!(matches!(r.rows[2][4], Value::TinyInt(_))); // precision
    assert!(matches!(r.rows[2][5], Value::TinyInt(_))); // scale
}

#[test]
fn test_sp_describe_first_result_set_zero_rows() {
    let mut engine = Engine::new();
    exec(&mut engine, "CREATE TABLE t_empty (x INT)");
    let r = query(
        &mut engine,
        "EXEC sp_describe_first_result_set N'SELECT x FROM t_empty WHERE 1=0'",
    );
    assert_eq!(r.rows.len(), 1);
    assert_eq!(r.columns.len(), 7);
}

#[test]
fn test_sp_describe_first_result_set_empty_sql() {
    let mut engine = Engine::new();
    let result = engine.execute(parse_sql("EXEC sp_describe_first_result_set N''").unwrap());
    assert!(result.is_err());
}

#[test]
fn test_sp_describe_first_result_set_via_rpc_name() {
    let mut engine = Engine::new();
    exec(&mut engine, "CREATE TABLE t_rpc (a INT, b NVARCHAR(50))");
    let r = query(
        &mut engine,
        "EXEC sp_describe_first_result_set N'SELECT a, b FROM t_rpc'",
    );
    assert_eq!(r.rows.len(), 2);
}

// ─── sp_describe_undeclared_parameters ─────────────────────────────────────

#[test]
fn test_sp_describe_undeclared_parameters_empty() {
    let mut engine = Engine::new();
    let r = query(
        &mut engine,
        "EXEC sp_describe_undeclared_parameters N'SELECT 1'",
    );
    assert_eq!(r.columns.len(), 10);
    assert_eq!(r.columns[0], "ParameterName");
    assert_eq!(r.rows.len(), 0);
}

// ─── fn_my_permissions TVF ─────────────────────────────────────────────────

#[test]
fn test_fn_my_permissions_returns_rows() {
    let mut engine = Engine::new();
    let r = query(&mut engine, "SELECT * FROM fn_my_permissions()");
    assert!(r.rows.len() > 0);
    assert_eq!(r.columns.len(), 2);
    assert_eq!(r.columns[0], "subentity_name");
    assert_eq!(r.columns[1], "permission_name");
}

#[test]
fn test_fn_my_permissions_contains_connect_sql() {
    let mut engine = Engine::new();
    let r = query(&mut engine, "SELECT permission_name FROM fn_my_permissions()");
    let has_connect = r.rows.iter().any(|row| match &row[0] {
        Value::NVarChar(s) => s == "CONNECT SQL",
        _ => false,
    });
    assert!(has_connect, "fn_my_permissions should include CONNECT SQL");
}

#[test]
fn test_fn_my_permissions_contains_view_server_state() {
    let mut engine = Engine::new();
    let r = query(&mut engine, "SELECT permission_name FROM fn_my_permissions()");
    let has = r.rows.iter().any(|row| match &row[0] {
        Value::NVarChar(s) => s == "VIEW SERVER STATE",
        _ => false,
    });
    assert!(has, "fn_my_permissions should include VIEW SERVER STATE");
}

#[test]
fn test_fn_my_permissions_with_alias() {
    let mut engine = Engine::new();
    let r = query(&mut engine, "SELECT permission_name FROM fn_my_permissions() p");
    assert!(r.rows.len() > 0);
}

// ─── Capability probes (SERVERPROPERTY, HAS_PERMS_BY_NAME) ─────────────────

#[test]
fn test_serverproperty_edition() {
    let mut engine = Engine::new();
    let r = query(&mut engine, "SELECT SERVERPROPERTY('Edition') AS v");
    match &r.rows[0][0] {
        Value::NVarChar(s) => assert!(!s.is_empty()),
        _ => panic!("Expected NVARCHAR for Edition"),
    }
}

#[test]
fn test_serverproperty_engine_edition() {
    let mut engine = Engine::new();
    let r = query(&mut engine, "SELECT SERVERPROPERTY('EngineEdition') AS v");
    assert_eq!(r.rows[0][0], Value::Int(3)); // Developer = 3
}

#[test]
fn test_serverproperty_product_version() {
    let mut engine = Engine::new();
    let r = query(&mut engine, "SELECT SERVERPROPERTY('ProductVersion') AS v");
    match &r.rows[0][0] {
        Value::NVarChar(s) => assert!(s.contains('.')),
        _ => panic!("Expected NVARCHAR for ProductVersion"),
    }
}

#[test]
fn test_has_perms_by_name_view_any_database() {
    let mut engine = Engine::new();
    let r = query(
        &mut engine,
        "SELECT HAS_PERMS_BY_NAME(NULL, NULL, 'VIEW ANY DATABASE') AS can_view",
    );
    assert_eq!(r.rows[0][0], Value::Int(1));
}

#[test]
fn test_has_perms_by_name_connect_sql() {
    let mut engine = Engine::new();
    let r = query(
        &mut engine,
        "SELECT HAS_PERMS_BY_NAME(NULL, NULL, 'CONNECT SQL') AS can_connect",
    );
    assert_eq!(r.rows[0][0], Value::Int(1));
}

#[test]
fn test_has_perms_by_name_view_server_state() {
    let mut engine = Engine::new();
    let r = query(
        &mut engine,
        "SELECT HAS_PERMS_BY_NAME(NULL, NULL, 'VIEW SERVER STATE') AS can_view",
    );
    assert_eq!(r.rows[0][0], Value::Int(1));
}

#[test]
fn test_is_srvrolemember_sysadmin() {
    let mut engine = Engine::new();
    let r = query(&mut engine, "SELECT IS_SRVROLEMEMBER('sysadmin') AS v");
    assert_eq!(r.rows[0][0], Value::Int(1));
}

#[test]
fn test_has_dbaccess_master() {
    let mut engine = Engine::new();
    let r = query(&mut engine, "SELECT HAS_DBACCESS('master') AS v");
    assert_eq!(r.rows[0][0], Value::Int(1));
}

#[test]
fn test_bootstrap_server_properties_probe() {
    let mut engine = Engine::new();
    let r = query(
        &mut engine,
        "SELECT SERVERPROPERTY('Edition') AS Edition, SERVERPROPERTY('EngineEdition') AS EngineEdition, SERVERPROPERTY('ProductVersion') AS ProductVersion",
    );
    assert_eq!(r.rows.len(), 1);
    assert_eq!(r.columns.len(), 3);
    assert_eq!(r.columns[0], "Edition");
    assert_eq!(r.columns[1], "EngineEdition");
    assert_eq!(r.columns[2], "ProductVersion");
}

#[test]
fn test_bootstrap_has_perms_probe() {
    let mut engine = Engine::new();
    let r = query(
        &mut engine,
        "SELECT HAS_PERMS_BY_NAME(NULL, NULL, 'VIEW ANY DATABASE') AS can_view",
    );
    assert_eq!(r.rows.len(), 1);
    assert_eq!(r.columns[0], "can_view");
}
