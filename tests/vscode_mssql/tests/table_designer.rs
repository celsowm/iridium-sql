//! Table Designer oracle for vscode-mssql 1.45.0.
//!
//! Verifies the pinned table_designer.json fixture is well-formed and that
//! the JSON-RPC method names the Table Designer contract relies on (init,
//! publish, publish variations) are covered. The TDS contract itself is
//! exercised separately by
//! `crates/iridium_core/tests/table_designer_contract.rs`.

use iridium_vscode_mssql_tests::capture_dir;

#[derive(serde::Deserialize)]
struct Capture {
    requests: Vec<serde_json::Value>,
}

#[test]
fn table_designer_capture_is_well_formed() {
    let path = capture_dir("vscode-mssql-1.45.0")
        .join("sqltoolsservice")
        .join("table_designer.json");
    assert!(
        path.exists(),
        "table_designer.json capture missing at {path:?} — see W3.2 stub"
    );
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {path:?}: {e}"));
    let capture: Capture = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("failed to parse {path:?}: {e}"));
    assert!(
        !capture.requests.is_empty(),
        "table_designer.json capture has no request entries"
    );
}

#[test]
fn table_designer_capture_covers_core_methods() {
    let path = capture_dir("vscode-mssql-1.45.0")
        .join("sqltoolsservice")
        .join("table_designer.json");
    let text = std::fs::read_to_string(&path).expect("capture should exist");
    let capture: Capture = serde_json::from_str(&text).expect("parseable");
    let methods: Vec<String> = capture
        .requests
        .iter()
        .filter_map(|r| {
            r.get("jsonrpc_method")
                .and_then(|m| m.as_str().map(String::from))
        })
        .collect();
    for required in [
        "TableDesigner/init",
        "TableDesigner/publish",
    ] {
        assert!(
            methods.iter().any(|m| m == required),
            "capture must cover {required}; found: {methods:?}"
        );
    }
}

#[test]
fn table_designer_capture_includes_unsupported_family_probe() {
    let path = capture_dir("vscode-mssql-1.45.0")
        .join("sqltoolsservice")
        .join("table_designer.json");
    let text = std::fs::read_to_string(&path).expect("capture should exist");
    let capture: Capture = serde_json::from_str(&text).expect("parseable");
    let has_unsupported: bool = capture
        .requests
        .iter()
        .any(|r| {
            r.get("jsonrpc_method")
                .and_then(|m| m.as_str())
                .map(|m| m == "TableDesigner/publish/unsupported_family")
                .unwrap_or(false)
        });
    assert!(
        has_unsupported,
        "capture must include TableDesigner/publish/unsupported_family (the gate for columnstore/temporal/graph/memory-optimized)"
    );
}
