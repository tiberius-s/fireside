use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

#[test]
fn validate_hello_exits_zero() {
    let hello = repo_root().join("docs/examples/hello.json");

    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("fireside"));
    command.arg("validate").arg(hello);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("is valid"));
}

#[test]
fn validate_missing_file_exits_nonzero() {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("fireside"));
    command.arg("validate").arg("nonexistent.json");

    command.assert().failure();
}

#[test]
fn new_scaffolds_file() {
    let temp = tempfile::tempdir().expect("temp dir should be created");

    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("fireside"));
    command
        .arg("new")
        .arg("test-talk")
        .arg("--dir")
        .arg(temp.path());

    command.assert().success();

    let file = temp.path().join("test-talk.json");
    assert!(
        file.exists(),
        "expected scaffolded file at {}",
        file.display()
    );

    let contents = std::fs::read_to_string(&file).expect("scaffolded file should be readable");
    let parsed: serde_json::Value =
        serde_json::from_str(&contents).expect("scaffolded file should be valid json");
    assert!(parsed.get("nodes").is_some(), "expected nodes field");
}

#[test]
fn new_project_scaffolds_directory() {
    let temp = tempfile::tempdir().expect("temp dir should be created");

    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("fireside"));
    command
        .arg("new")
        .arg("test-course")
        .arg("--project")
        .arg("--dir")
        .arg(temp.path());

    command.assert().success();

    let project = temp.path().join("test-course");
    assert!(
        project.exists(),
        "expected project dir at {}",
        project.display()
    );
    assert!(
        project.join("fireside.json").exists(),
        "expected fireside.json"
    );
    assert!(
        project.join("nodes/main.json").exists(),
        "expected nodes/main.json"
    );
    assert!(project.join("themes").exists(), "expected themes directory");
}
