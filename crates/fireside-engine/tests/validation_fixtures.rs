use std::path::Path;

use fireside_engine::load_graph;
use fireside_engine::validation::{Severity, validate_graph};

fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn valid_linear_fixture_has_no_diagnostics() {
    let graph = load_graph(&fixture_path("valid_linear.json")).expect("fixture should load");
    let diagnostics = validate_graph(&graph);
    assert!(
        diagnostics.is_empty(),
        "expected no diagnostics, got {diagnostics:?}"
    );
}

#[test]
fn valid_branching_fixture_has_no_diagnostics() {
    let graph = load_graph(&fixture_path("valid_branching.json")).expect("fixture should load");
    let diagnostics = validate_graph(&graph);
    assert!(
        diagnostics.is_empty(),
        "expected no diagnostics, got {diagnostics:?}"
    );
}

#[test]
fn invalid_dangling_ref_fixture_reports_error() {
    let graph =
        load_graph(&fixture_path("invalid_dangling_ref.json")).expect("fixture should load");
    let diagnostics = validate_graph(&graph);

    assert!(!diagnostics.is_empty(), "expected dangling diagnostics");
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == Severity::Error && diagnostic.message.contains("unknown node")
        }),
        "expected error diagnostic mentioning unknown node, got {diagnostics:?}"
    );
}

#[test]
fn invalid_empty_fixture_fails_to_load() {
    let error = load_graph(&fixture_path("invalid_empty.json")).expect_err("fixture should fail");
    let message = format!("{error:#}");
    assert!(
        message.contains("graph contains no nodes") || message.contains("empty graph"),
        "expected empty graph message, got: {message}"
    );
}

#[test]
fn invalid_duplicate_id_fixture_fails_to_load() {
    let error =
        load_graph(&fixture_path("invalid_duplicate_id.json")).expect_err("fixture should fail");
    let message = format!("{error:#}");
    assert!(
        message.contains("duplicate") && message.contains("node"),
        "expected duplicate-id message, got: {message}"
    );
}
