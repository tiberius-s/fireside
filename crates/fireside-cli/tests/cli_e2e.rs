//! End-to-end tests for the three CLI verbs.

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn fireside() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("fireside"))
}

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

#[test]
fn bare_invocation_teaches_the_three_verbs() {
    fireside()
        .assert()
        .success()
        .stdout(predicate::str::contains("fireside <file>"))
        .stdout(predicate::str::contains("validate"))
        .stdout(predicate::str::contains("new"));
}

#[test]
fn validate_hello_exits_zero() {
    fireside()
        .arg("validate")
        .arg(repo_root().join("docs/examples/hello.json"))
        .assert()
        .success()
        .stdout(predicate::str::contains("0 errors"));
}

#[test]
fn validate_missing_file_suggests_creating_it() {
    fireside()
        .arg("validate")
        .arg("nonexistent.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No deck named nonexistent.json — \"fireside new nonexistent\" creates one.",
        ));
}

#[test]
fn present_missing_file_suggests_creating_it() {
    fireside()
        .arg("nope.fireside.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No deck named nope.fireside.json — \"fireside new nope\" creates one.",
        ));
}

#[test]
fn validate_markdown_file_suggests_import_first() {
    let temp = tempfile::tempdir().expect("temp dir");
    let deck = temp.path().join("talk.md");
    std::fs::write(&deck, "# My Conference Talk\n").expect("write fixture");

    fireside()
        .arg("validate")
        .arg(&deck)
        .assert()
        .failure()
        .stderr(predicate::str::contains("This is a Markdown file"));
}

#[test]
fn present_markdown_file_suggests_import_first() {
    let temp = tempfile::tempdir().expect("temp dir");
    let deck = temp.path().join("talk.md");
    std::fs::write(&deck, "# My Conference Talk\n").expect("write fixture");

    fireside().arg(&deck).assert().failure().stderr(
        predicate::str::contains("This is a Markdown file")
            .and(predicate::str::contains("fireside import"))
            .and(predicate::str::contains("talk.fireside.json")),
    );
}

#[test]
fn present_without_a_tty_gives_a_plain_message() {
    fireside()
        .arg("demo")
        .write_stdin("")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "fireside needs an interactive terminal to present",
        ))
        .stderr(predicate::str::contains("panicked").not());
}

#[test]
fn validate_reports_dangling_targets_in_plain_language() {
    let temp = tempfile::tempdir().expect("temp dir");
    let deck = temp.path().join("broken.json");
    std::fs::write(
        &deck,
        r#"{"nodes":[{"id":"a","traversal":"ghost","content":[]}]}"#,
    )
    .expect("write fixture");

    fireside()
        .arg("validate")
        .arg(&deck)
        .assert()
        .failure()
        .stdout(predicate::str::contains("no node has that id"));
}

#[test]
fn present_refuses_a_broken_deck_before_taking_the_screen() {
    let temp = tempfile::tempdir().expect("temp dir");
    let deck = temp.path().join("broken.json");
    std::fs::write(
        &deck,
        r#"{"nodes":[{"id":"a","traversal":"ghost","content":[]}]}"#,
    )
    .expect("write fixture");

    fireside()
        .arg(&deck)
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be presented yet"));
}

#[test]
fn new_scaffolds_a_deck_that_validates_clean() {
    let temp = tempfile::tempdir().expect("temp dir");

    fireside()
        .current_dir(temp.path())
        .arg("new")
        .arg("Test Talk")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created test-talk.fireside.json"));

    let file = temp.path().join("test-talk.fireside.json");
    let contents = std::fs::read_to_string(&file).expect("scaffold is readable");
    let parsed: serde_json::Value = serde_json::from_str(&contents).expect("scaffold is JSON");
    assert!(parsed.get("nodes").is_some(), "expected nodes field");

    fireside()
        .arg("validate")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("0 errors"));
}

/// Kills and reaps the wrapped child even if an assertion panics, so a
/// failing test never leaves a `--watch` process running on the machine.
struct KillOnDrop(std::process::Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn validate_watch_prints_the_first_result_immediately() {
    let temp = tempfile::tempdir().expect("temp dir");
    let deck = temp.path().join("deck.json");
    std::fs::write(&deck, r#"{"nodes":[{"id":"a","content":[]}]}"#).expect("write fixture");

    let child = std::process::Command::new(assert_cmd::cargo::cargo_bin!("fireside"))
        .arg("validate")
        .arg("--watch")
        .arg(&deck)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn fireside validate --watch");
    let mut guard = KillOnDrop(child);

    let mut stdout = std::io::BufReader::new(guard.0.stdout.take().expect("piped stdout"));
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        use std::io::BufRead;
        let mut first_line = String::new();
        let _ = stdout.read_line(&mut first_line);
        let _ = tx.send(first_line);
    });

    let first_line = rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("--watch prints its first result within 5s");
    assert!(
        first_line.contains("no problems found"),
        "expected the immediate first-check result: {first_line:?}"
    );
}

#[test]
fn new_accepts_a_template_and_author_flag_non_interactively() {
    let temp = tempfile::tempdir().expect("temp dir");

    fireside()
        .current_dir(temp.path())
        .arg("new")
        .arg("Onboarding Workshop")
        .arg("--template")
        .arg("workshop")
        .arg("--author")
        .arg("Ada Lovelace")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Created onboarding-workshop.fireside.json",
        ));

    let file = temp.path().join("onboarding-workshop.fireside.json");
    let contents = std::fs::read_to_string(&file).expect("scaffold is readable");
    assert!(contents.contains("\"author\": \"Ada Lovelace\""));
    assert!(contents.contains("\"agenda\""));

    fireside()
        .arg("validate")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("no problems found"));
}

#[test]
fn new_non_interactive_never_prompts_to_present() {
    let temp = tempfile::tempdir().expect("temp dir");

    fireside()
        .current_dir(temp.path())
        .arg("new")
        .arg("Solo Deck")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created solo-deck.fireside.json"))
        .stdout(predicate::str::contains("Present it now").not());
}

#[test]
fn new_without_a_name_prompts_interactively() {
    let temp = tempfile::tempdir().expect("temp dir");

    fireside()
        .current_dir(temp.path())
        .arg("new")
        .write_stdin("My Workshop\n3\nGrace Hopper\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Created my-workshop.fireside.json",
        ));

    let file = temp.path().join("my-workshop.fireside.json");
    let contents = std::fs::read_to_string(&file).expect("scaffold is readable");
    assert!(contents.contains("\"author\": \"Grace Hopper\""));
    assert!(contents.contains("\"agenda\""));
}

#[test]
fn new_with_banner_flag_embeds_ascii_art_and_validates() {
    let temp = tempfile::tempdir().expect("temp dir");

    fireside()
        .current_dir(temp.path())
        .arg("new")
        .arg("Test Talk")
        .arg("--banner")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created test-talk.fireside.json"));

    let file = temp.path().join("test-talk.fireside.json");
    let contents = std::fs::read_to_string(&file).expect("scaffold is readable");
    assert!(contents.contains("\"kind\": \"ascii-art\""));

    fireside()
        .arg("validate")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("0 errors"));
}

#[test]
fn new_refuses_to_overwrite() {
    let temp = tempfile::tempdir().expect("temp dir");
    fireside()
        .current_dir(temp.path())
        .arg("new")
        .arg("twice")
        .assert()
        .success();
    fireside()
        .current_dir(temp.path())
        .arg("new")
        .arg("twice")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn import_compiles_markdown_to_a_default_path_and_the_result_validates() {
    let temp = tempfile::tempdir().expect("temp dir");
    let input = temp.path().join("talk.md");
    std::fs::write(
        &input,
        "## Welcome\n\nThanks for coming.\n\n## Thanks\n\nQuestions?\n",
    )
    .expect("write fixture");

    fireside()
        .current_dir(temp.path())
        .arg("import")
        .arg("talk.md")
        .assert()
        .success()
        .stdout(predicate::str::contains("Imported talk.fireside.json"));

    let output = temp.path().join("talk.fireside.json");
    assert!(output.exists(), "default output path was created");

    fireside()
        .arg("validate")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("no problems found"));
}

#[test]
fn import_refuses_to_overwrite_an_existing_output() {
    let temp = tempfile::tempdir().expect("temp dir");
    let input = temp.path().join("talk.md");
    std::fs::write(&input, "## Only\n\nHi.\n").expect("write fixture");

    fireside()
        .current_dir(temp.path())
        .arg("import")
        .arg("talk.md")
        .assert()
        .success();

    fireside()
        .current_dir(temp.path())
        .arg("import")
        .arg("talk.md")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn import_ascii_art_fence_becomes_a_real_block() {
    let temp = tempfile::tempdir().expect("temp dir");
    let input = temp.path().join("talk.md");
    std::fs::write(
        &input,
        "## Welcome\n\n```ascii-art\n _ __\n| '__|\n| |\n|_|\n```\n",
    )
    .expect("write fixture");

    fireside()
        .current_dir(temp.path())
        .arg("import")
        .arg("talk.md")
        .assert()
        .success();

    let output = temp.path().join("talk.fireside.json");
    let contents = std::fs::read_to_string(&output).expect("scaffold is readable");
    assert!(contents.contains("\"kind\": \"ascii-art\""));
    assert!(!contents.contains("\"kind\": \"code\""));
}

#[test]
fn art_text_prints_a_multiline_banner() {
    fireside()
        .arg("art")
        .arg("text")
        .arg("Fireside")
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| s.lines().count() > 1));
}

#[test]
fn art_text_warns_on_stderr_when_too_wide() {
    fireside()
        .arg("art")
        .arg("text")
        .arg("A Title So Long It Cannot Possibly Fit The Card")
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| s.lines().count() > 1))
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn art_text_silent_on_stderr_when_it_fits() {
    fireside()
        .arg("art")
        .arg("text")
        .arg("Hi")
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn art_text_partial_recognition_still_produces_output() {
    fireside()
        .arg("art")
        .arg("text")
        .arg("Hi 🔥")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn art_text_with_no_recognized_characters_errors_clearly() {
    fireside()
        .arg("art")
        .arg("text")
        .arg("🔥🔥🔥")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no recognized characters"));
}

#[test]
fn art_image_converts_a_readable_file() {
    fireside()
        .arg("art")
        .arg("image")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.png"))
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| s.lines().count() > 1));
}

#[test]
fn art_image_reports_a_clear_error_for_a_missing_file() {
    fireside()
        .arg("art")
        .arg("image")
        .arg("nonexistent.png")
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not read"));
}

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Number of distinct characters appearing in `s` — a proxy for how much
/// of the charset's tonal range an ASCII-art conversion actually used.
fn distinct_chars(s: &str) -> usize {
    s.chars().collect::<std::collections::HashSet<_>>().len()
}

#[test]
fn art_image_stretches_low_contrast_image_by_default() {
    let stretched = fireside()
        .arg("art")
        .arg("image")
        .arg(fixture("low-contrast.png"))
        .output()
        .expect("fireside runs");
    let unstretched = fireside()
        .arg("art")
        .arg("image")
        .arg(fixture("low-contrast.png"))
        .arg("--no-normalize")
        .output()
        .expect("fireside runs");

    assert!(stretched.status.success());
    assert!(unstretched.status.success());
    let stretched_out = String::from_utf8(stretched.stdout).unwrap();
    let unstretched_out = String::from_utf8(unstretched.stdout).unwrap();
    assert!(
        distinct_chars(&stretched_out) > distinct_chars(&unstretched_out),
        "expected the default (stretched) conversion to use a wider variety of \
         characters than --no-normalize"
    );
}

#[test]
fn art_image_no_normalize_reproduces_prior_behavior() {
    fireside()
        .arg("art")
        .arg("image")
        .arg(fixture("tiny.png"))
        .arg("--no-normalize")
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| s.lines().count() > 1));
}

#[test]
fn art_image_charset_flag_changes_output_characters() {
    let block = fireside()
        .arg("art")
        .arg("image")
        .arg(fixture("tiny.png"))
        .arg("--charset")
        .arg("block")
        .output()
        .expect("fireside runs");
    let default = fireside()
        .arg("art")
        .arg("image")
        .arg(fixture("tiny.png"))
        .output()
        .expect("fireside runs");

    assert!(block.status.success());
    let block_out = String::from_utf8(block.stdout).unwrap();
    let default_out = String::from_utf8(default.stdout).unwrap();
    assert_ne!(block_out, default_out);
    let block_chars: std::collections::HashSet<char> =
        block_out.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        block_chars.is_subset(&" ░▒▓█".chars().collect()),
        "expected --charset block output to use only block-shading characters, got {block_chars:?}"
    );
}

#[test]
fn art_image_invert_flag_flips_shading() {
    let inverted = fireside()
        .arg("art")
        .arg("image")
        .arg(fixture("tiny.png"))
        .arg("--invert")
        .output()
        .expect("fireside runs");
    let normal = fireside()
        .arg("art")
        .arg("image")
        .arg(fixture("tiny.png"))
        .output()
        .expect("fireside runs");

    assert!(inverted.status.success());
    assert_ne!(inverted.stdout, normal.stdout);
}

#[test]
fn art_image_default_charset_matches_unflagged_output() {
    let explicit = fireside()
        .arg("art")
        .arg("image")
        .arg(fixture("tiny.png"))
        .arg("--charset")
        .arg("default")
        .output()
        .expect("fireside runs");
    let unflagged = fireside()
        .arg("art")
        .arg("image")
        .arg(fixture("tiny.png"))
        .output()
        .expect("fireside runs");

    assert_eq!(explicit.stdout, unflagged.stdout);
}

#[test]
fn art_image_warns_on_stderr_for_low_contrast_source() {
    fireside()
        .arg("art")
        .arg("image")
        .arg(fixture("flat.png"))
        .assert()
        .success()
        .stderr(predicate::str::contains("brightness range"))
        .stdout(predicate::function(|s: &str| s.lines().count() > 1));
}

#[test]
fn art_image_silent_on_stderr_for_normal_contrast_source() {
    fireside()
        .arg("art")
        .arg("image")
        .arg(fixture("tiny.png"))
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn art_image_warning_fires_even_with_no_normalize() {
    fireside()
        .arg("art")
        .arg("image")
        .arg(fixture("flat.png"))
        .arg("--no-normalize")
        .assert()
        .success()
        .stderr(predicate::str::contains("brightness range"));
}
