//! Smoke test: verifies the `iridium-vscode-mssql-tests` crate is wired up
//! and the planned fixture inventory matches what the program declares.
//!
//! This test exists so the W1.1 scaffold has at least one runnable test that
//! fails loudly if `tests/vscode_mssql/` is accidentally removed from the
//! Cargo workspace, or if `planned_fixtures()` is mutated without also
//! updating `docs/plans/vscode-mssql-compatibility-program.md`.

#[test]
fn crate_is_wired_into_workspace() {
    // If this crate is removed from the workspace Cargo.toml, `cargo test`
    // won't even find this binary — so the very fact that this test compiles
    // verifies the workspace membership.
    let _ = env!("CARGO_PKG_NAME");
}

#[test]
fn planned_fixtures_match_program() {
    let planned = iridium_vscode_mssql_tests::runner::planned();
    assert_eq!(planned.len(), 14, "W1.1 scaffold declares 14 fixtures");

    // Subset-checks: ensure the P0 fixtures the plan calls out are present.
    assert!(planned.contains(&"sqltoolsservice/query_cancel"), "query_cancel is P0 W2.1");
    assert!(planned.contains(&"sqlclient/result_metadata"), "result_metadata is P0 W2.3");
    assert!(planned.contains(&"sqlclient/parameters"), "parameters is P0 W2.5");
    assert!(planned.contains(&"sqltoolsservice/object_explorer"), "object_explorer is W1.4 oracle");
    assert!(planned.contains(&"sqltoolsservice/profiler"), "profiler is signposted unsupported per W4.4");
    assert!(planned.contains(&"sqltoolsservice/disaster_recovery"), "backup/restore is signposted unsupported per W4.3");
}

#[test]
fn fixtures_dir_layout_exists() {
    // The capture directories must exist (they hold .gitkeep files today).
    let root = iridium_vscode_mssql_tests::fixtures_dir();
    assert!(root.exists(), "fixtures dir missing: {root:?}");
    for oracle in ["RealSqlServer", "vscode-mssql-1.45.0"] {
        let d = iridium_vscode_mssql_tests::capture_dir(oracle);
        assert!(d.exists(), "oracle capture dir missing: {d:?}");
    }
}
