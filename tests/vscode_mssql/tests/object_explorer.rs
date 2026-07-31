//! Object Explorer oracle for vscode-mssql 1.45.0.
//!
//! This test asserts that the JSON-RPC method names vscode-mssql 1.45.0 sends
//! to its bundled SqlToolsService are covered by a pinned capture under
//! `fixtures/capture/vscode-mssql-1.45.0/sqltoolsservice/`.
//!
//! It is the *contract* side of W1.4: it does not yet replay the requests
//! against a live language service (that requires the extension + a stdio
//! logger, which is a manual capture step). Instead it fails fast if the
//! capture file is missing or malformed, so a reviewer can see at a glance which
//! JSON-RPC methods are expected to be covered.

use std::path::Path;

use iridium_vscode_mssql_tests::capture_dir;

#[derive(serde::Deserialize)]
struct Capture {
    requests: Vec<serde_json::Value>,
}

#[test]
fn object_explorer_capture_is_well_formed() {
    let path = capture_dir("vscode-mssql-1.45.0")
        .join("sqltoolsservice")
        .join("object_explorer.json");
    assert!(
        path.exists(),
        "object_explorer.json capture missing at {path:?} — run the W1.2 manual capture"
    );
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {path:?}: {e}"));
    let capture: Capture = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("failed to parse {path:?}: {e}"));
    assert!(
        !capture.requests.is_empty(),
        "object_explorer.json capture has no request entries"
    );
}

#[test]
fn object_explorer_capture_covers_create_and_expand() {
    // vscode-mssql's Object Explorer always starts with createSession then
    // expand. If these method names are absent from the capture, the harness
    // cannot assert compatibility for the tree bootstrap.
    let path = capture_dir("vscode-mssql-1.45.0")
        .join("sqltoolsservice")
        .join("object_explorer.json");
    let text = std::fs::read_to_string(&path).expect("capture should exist");
    let capture: Capture = serde_json::from_str(&text).expect("parseable");
    let methods: Vec<String> = capture
        .requests
        .iter()
        .filter_map(|r| r.get("jsonrpc_method").and_then(|m| m.as_str().map(String::from)))
        .collect();
    assert!(
        methods.iter().any(|m| m == "ObjectExplorer/createSession"),
        "capture must cover ObjectExplorer/createSession; found: {methods:?}"
    );
    assert!(
        methods.iter().any(|m| m == "ObjectExplorer/expand"),
        "capture must cover ObjectExplorer/expand; found: {methods:?}"
    );
}

#[test]
fn object_explorer_capture_path_matches_ssms_oracle_layout() {
    // The SSMS Object Explorer contract test lives at
    // crates/iridium_server/tests/ssms_object_explorer_contract.rs and replays
    // raw T-SQL. The vscode-mssql oracle is the JSON-RPC counterpart; both
    // must be present so the matrix can distinguish "SSMS-replay works" from
    // "vscode-mssql JSON-RPC contract works".
    //
    // `CARGO_MANIFEST_DIR` is this crate (tests/vscode_mssql); the workspace
    // root is two levels up (../..).
    let ws_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let ssms_fixture = ws_root.join("crates/iridium_server/tests/fixtures/ssms_object_explorer_cases.json");
    assert!(
        ssms_fixture.exists(),
        "SSMS Object Explorer fixture missing — expected alongside the vscode-mssql oracle"
    );
}
