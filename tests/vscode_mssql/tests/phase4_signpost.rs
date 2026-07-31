use iridium_vscode_mssql_tests::capture_dir;

#[derive(serde::Deserialize)]
struct Capture {
    requests: Vec<serde_json::Value>,
}

fn load_capture(subdir: &str) -> Capture {
    let path = capture_dir("vscode-mssql-1.45.0")
        .join("sqltoolsservice")
        .join(subdir);
    assert!(path.exists(), "fixture missing at {path:?}");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {path:?}: {e}"));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("failed to parse {path:?}: {e}"))
}

// ── W4.1/W4.2: Schema Compare ──

#[test]
fn schema_compare_capture_is_well_formed() {
    let c = load_capture("schema_compare.json");
    assert!(!c.requests.is_empty());
}

#[test]
fn schema_compare_capture_marked_unsupported() {
    let path = capture_dir("vscode-mssql-1.45.0")
        .join("sqltoolsservice")
        .join("schema_compare.json");
    let text = std::fs::read_to_string(&path).expect("exists");
    let doc: serde_json::Value = serde_json::from_str(&text).expect("parseable");
    let status = doc.get("status").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(
        status, "unsupported",
        "schema_compare must be marked unsupported"
    );
}

// ── W4.3: Backup/Restore (disaster_recovery) ──

#[test]
fn disaster_recovery_capture_is_well_formed() {
    let c = load_capture("disaster_recovery.json");
    assert!(!c.requests.is_empty());
}

#[test]
fn disaster_recovery_capture_marked_unsupported() {
    let path = capture_dir("vscode-mssql-1.45.0")
        .join("sqltoolsservice")
        .join("disaster_recovery.json");
    let text = std::fs::read_to_string(&path).expect("exists");
    let doc: serde_json::Value = serde_json::from_str(&text).expect("parseable");
    let status = doc.get("status").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(
        status, "unsupported",
        "disaster_recovery must be marked unsupported"
    );
}

// ── W4.4: Extended Events Profiler ──

#[test]
fn profiler_capture_is_well_formed() {
    let c = load_capture("profiler.json");
    assert!(!c.requests.is_empty());
}

#[test]
fn profiler_capture_marked_unsupported() {
    let path = capture_dir("vscode-mssql-1.45.0")
        .join("sqltoolsservice")
        .join("profiler.json");
    let text = std::fs::read_to_string(&path).expect("exists");
    let doc: serde_json::Value = serde_json::from_str(&text).expect("parseable");
    let status = doc.get("status").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(
        status, "unsupported",
        "profiler must be marked unsupported"
    );
}
