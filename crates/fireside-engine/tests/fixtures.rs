//! Runs the shared conformance fixture corpus at `protocol/fixtures/`
//! against the Rust validator and checks it against the same
//! `protocol/fixtures.expected.json` expectations `protocol/validate.mjs`
//! is checked against — proving Rust/Node rule-id parity is a tested fact,
//! not just an assertion resting on matching rule-name strings. See
//! `specs/004-spec-patch-0-1-1/contracts/fixture-corpus.md`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use fireside_core::Graph;
use fireside_engine::{has_errors, validate};

fn protocol_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("protocol")
}

fn fixture_paths(dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("reading fixture dir {}: {e}", dir.display()))
        .map(|entry| entry.expect("readable dir entry").path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths
}

#[test]
fn fixture_corpus_matches_documented_expectations() {
    let protocol = protocol_dir();
    let expected_raw = fs::read_to_string(protocol.join("fixtures.expected.json"))
        .expect("protocol/fixtures.expected.json is readable");
    let expected: serde_json::Value =
        serde_json::from_str(&expected_raw).expect("fixtures.expected.json is valid JSON");
    let expected = expected.as_object().expect("expectations is a JSON object");

    let fixtures_dir = protocol.join("fixtures");
    let mut seen_keys: BTreeSet<String> = BTreeSet::new();
    let mut checked = 0usize;

    for (subdir, expect_errors) in [("valid", false), ("invalid", true)] {
        for path in fixture_paths(&fixtures_dir.join(subdir)) {
            let rel_key = format!("{subdir}/{}", path.file_name().unwrap().to_string_lossy());
            seen_keys.insert(rel_key.clone());

            let expected_rules: BTreeSet<String> = expected
                .get(&rel_key)
                .unwrap_or_else(|| panic!("no expectation entry for fixture {rel_key}"))
                .as_array()
                .expect("expectation value is an array")
                .iter()
                .map(|v| v.as_str().expect("rule id is a string").to_owned())
                .collect();

            let contents = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("reading fixture {}: {e}", path.display()));
            let graph = Graph::from_json(&contents)
                .unwrap_or_else(|e| panic!("fixture {} is a valid Graph: {e}", path.display()));
            let diags = validate(&graph);

            let actual_rules: BTreeSet<String> = diags.iter().map(|d| d.rule.to_owned()).collect();

            assert_eq!(
                actual_rules, expected_rules,
                "fixture {rel_key}: fired rule-ids don't match fixtures.expected.json"
            );

            assert_eq!(
                has_errors(&diags),
                expect_errors,
                "fixture {rel_key}: expected has_errors={expect_errors} (directory: {subdir}/)"
            );

            checked += 1;
        }
    }

    let documented_keys: BTreeSet<String> = expected.keys().cloned().collect();
    assert_eq!(
        seen_keys, documented_keys,
        "fixtures on disk and fixtures.expected.json entries must match exactly"
    );
    assert!(
        checked >= 10,
        "expected at least 10 fixtures, checked {checked}"
    );
}
