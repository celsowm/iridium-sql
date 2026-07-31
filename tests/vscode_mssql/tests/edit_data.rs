//! Edit Data oracle for vscode-mssql 1.45.0.
//!
//! This test asserts that the pinned edit_data.json capture under
//! `fixtures/capture/vscode-mssql-1.45.0/sqltoolsservice/` is well-formed
//! and covers the core Edit Data JSON-RPC methods. The differential harness
//! (tests/vscode_mssql/tests/edit_data.rs) verifies the file exists; the
//! engine-side contract is exercised separately by
//! `crates/iridium_core/tests/edit_data_contract.rs`.

use iridium_vscode_mssql_tests::capture_dir;

#[derive(serde::Deserialize)]
struct Capture {
    requests: Vec<serde_json::Value>,
}

#[test]
fn edit_data_capture_is_well_formed() {
    let path = capture_dir("vscode-mssql-1.45.0")
        .join("sqltoolsservice")
        .join("edit_data.json");
    assert!(
        path.exists(),
        "edit_data.json capture missing at {path:?} — see W1.2 manual capture / W3.1 stub"
    );
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {path:?}: {e}"));
    let capture: Capture = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("failed to parse {path:?}: {e}"));
    assert!(
        !capture.requests.is_empty(),
        "edit_data.json capture has no request entries"
    );
}

#[test]
fn edit_data_capture_covers_core_methods() {
    let path = capture_dir("vscode-mssql-1.45.0")
        .join("sqltoolsservice")
        .join("edit_data.json");
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
        "EditData/initialize",
        "EditData/fetchRows",
        "EditData/updateCell",
        "EditData/insertRow",
        "EditData/deleteRow",
    ] {
        assert!(
            methods.iter().any(|m| m == required),
            "capture must cover {required}; found: {methods:?}"
        );
    }
}

#[test]
fn edit_data_capture_includes_editability_probe() {
    let path = capture_dir("vscode-mssql-1.45.0")
        .join("sqltoolsservice")
        .join("edit_data.json");
    let text = std::fs::read_to_string(&path).expect("capture should exist");
    let capture: Capture = serde_json::from_str(&text).expect("parseable");
    let has_editability: bool = capture
        .requests
        .iter()
        .any(|r| {
            r.get("jsonrpc_method")
                .and_then(|m| m.as_str())
                .map(|m| m == "EditData/initialize/editability")
                .unwrap_or(false)
        });
    assert!(
        has_editability,
        "capture must include EditData/initialize/editability probe (the unsupported-table-family gate)"
    );
}
