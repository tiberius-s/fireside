//! Deck diagnostics rendered as plain-language reports: parse-error carets,
//! validation summaries, and the `validate`/`validate --watch` verb.

use std::path::Path;

use anyhow::Result;
use fireside_core::{CoreError, Graph};
use fireside_engine::{Diagnostic, Severity, validate};

use crate::load;
use crate::watch::watch_loop;

/// A parse failure the author can act on: the line before, the offending
/// line, and a caret under the exact column.
pub(crate) fn parse_report(path: &Path, text: &str, err: &serde_json::Error) -> String {
    let line = err.line();
    let column = err.column().max(1);
    let mut out = format!("✗ {} is not a valid deck\n", path.display());

    let lines: Vec<&str> = text.lines().collect();
    if line >= 1 && line <= lines.len() {
        let gutter = line.to_string().len();
        out.push('\n');
        if line >= 2 {
            out.push_str(&format!("  {:>gutter$} │ {}\n", line - 1, lines[line - 2]));
        }
        out.push_str(&format!("  {:>gutter$} │ {}\n", line, lines[line - 1]));
        out.push_str(&format!(
            "  {:>gutter$} │ {}^ {}\n",
            "",
            " ".repeat(column - 1),
            strip_position(err),
        ));
    } else {
        out.push_str(&format!("\n  {err}\n"));
    }
    out.push_str("\nFix the file and try again.");
    out
}

/// serde_json appends " at line L column C" to every message; the report
/// and the reload flash show the position themselves, so drop it.
pub(crate) fn strip_position(err: &serde_json::Error) -> String {
    let full = err.to_string();
    full.split(" at line ").next().unwrap_or(&full).to_owned()
}

/// Render a validation result exactly as `validate` has always printed it:
/// a success line, or the full diagnostic list plus a summary. Shared by
/// the one-shot path and the watch loop so their output never drifts apart.
fn diagnostics_report(path: &Path, diags: &[Diagnostic]) -> String {
    if diags.is_empty() {
        return format!("✓ {} — no problems found", path.display());
    }

    let mut errors = 0usize;
    let mut warnings = 0usize;
    let mut lines: Vec<String> = diags
        .iter()
        .map(|d| {
            let icon = match d.severity {
                Severity::Error => {
                    errors += 1;
                    "✗"
                }
                Severity::Warning => {
                    warnings += 1;
                    "⚠"
                }
                Severity::Info => "ℹ",
            };
            format!("  {icon} {}", d.message)
        })
        .collect();
    lines.push(format!(
        "\n{}: {errors} error(s), {warnings} warning(s), {} note(s)",
        path.display(),
        diags.len() - errors - warnings
    ));
    lines.join("\n")
}

pub(crate) fn validate_file(path: &Path, watch: bool) -> Result<()> {
    if watch {
        return watch_loop(path);
    }

    let graph = load(path)?;
    let diags = validate(&graph);
    let has_errors = diags.iter().any(|d| d.severity == Severity::Error);
    println!("{}", diagnostics_report(path, &diags));
    if has_errors {
        std::process::exit(1);
    }
    Ok(())
}

/// Check the file once and render the result — a success line, the
/// diagnostic list, a caret-pointed parse report, or a one-line message if
/// the file can't currently be read. Never exits the process, so it is
/// safe to call on every tick of the watch loop.
pub(crate) fn watch_report(path: &Path) -> String {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => return format!("✗ could not read {}: {err}", path.display()),
    };
    match Graph::from_json(&text) {
        Err(CoreError::Parse(err)) => parse_report(path, &text, &err),
        Ok(graph) => diagnostics_report(path, &validate(&graph)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A single terminal node with no traversal and no content — the
    /// smallest deck that produces zero diagnostics of any severity, so
    /// `diagnostics_report` takes its empty-diagnostics branch.
    const SPOTLESS_DECK: &str = r#"{"nodes":[{"id":"a","content":[]}]}"#;

    #[test]
    fn parse_report_points_at_the_line_with_a_caret() {
        let text = "{\n  \"fireside-version\": \"0.1.0\",\n  \"nodes\": [}\n}";
        let err = Graph::from_json(text).expect_err("invalid JSON");
        let CoreError::Parse(err) = err;
        let report = parse_report(Path::new("broken.json"), text, &err);
        assert!(
            report.contains("broken.json is not a valid deck"),
            "{report}"
        );
        assert!(
            report.contains("3 │   \"nodes\": [}"),
            "offending line shown: {report}"
        );
        assert!(report.contains('^'), "caret shown: {report}");
        assert!(
            !report.contains("at line"),
            "no duplicated position: {report}"
        );
    }

    #[test]
    fn watch_report_confirms_a_valid_deck() {
        let temp = tempfile::tempdir().expect("temp dir");
        let deck = temp.path().join("deck.json");
        std::fs::write(&deck, SPOTLESS_DECK).expect("write fixture");

        let report = watch_report(&deck);
        assert!(report.contains("no problems found"), "{report}");
    }

    #[test]
    fn watch_report_shows_diagnostics_for_a_dangling_target() {
        let temp = tempfile::tempdir().expect("temp dir");
        let deck = temp.path().join("broken.json");
        std::fs::write(
            &deck,
            r#"{"nodes":[{"id":"a","traversal":"ghost","content":[]}]}"#,
        )
        .expect("write fixture");

        let report = watch_report(&deck);
        assert!(
            report.contains("no node has that id"),
            "expected the dangling-target diagnostic: {report}"
        );
        assert!(
            report.contains("error(s)"),
            "expected the summary line: {report}"
        );
    }

    #[test]
    fn watch_report_shows_a_caret_report_for_malformed_json() {
        let temp = tempfile::tempdir().expect("temp dir");
        let deck = temp.path().join("broken.json");
        std::fs::write(&deck, "{\n  \"nodes\": [}\n}").expect("write fixture");

        let report = watch_report(&deck);
        assert!(report.contains("is not a valid deck"), "{report}");
        assert!(report.contains('^'), "expected a caret: {report}");
    }

    #[test]
    fn watch_report_names_a_missing_file_without_panicking() {
        let temp = tempfile::tempdir().expect("temp dir");
        let missing = temp.path().join("nope.json");

        let report = watch_report(&missing);
        assert!(
            report.contains("could not read"),
            "expected a missing-file message: {report}"
        );
    }

    #[test]
    fn watch_report_recovers_after_a_file_is_deleted_and_recreated() {
        let temp = tempfile::tempdir().expect("temp dir");
        let deck = temp.path().join("deck.json");
        std::fs::write(&deck, SPOTLESS_DECK).expect("write fixture");
        assert!(watch_report(&deck).contains("no problems found"));

        std::fs::remove_file(&deck).expect("delete fixture");
        let missing_report = watch_report(&deck);
        assert!(
            missing_report.contains("could not read"),
            "{missing_report}"
        );

        std::fs::write(&deck, SPOTLESS_DECK).expect("recreate fixture");
        let recovered_report = watch_report(&deck);
        assert!(
            recovered_report.contains("no problems found"),
            "{recovered_report}"
        );
    }
}
