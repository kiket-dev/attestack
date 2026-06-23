mod common;

use std::fs::File;
use std::io::{Read, Write};

use common::{attestack, init_and_start, init_git_repo, latest_bundle_path, stop_session};
use predicates::prelude::*;
use tempfile::tempdir;
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[test]
fn phase1_flow_in_git_repo() {
    let dir = tempdir().unwrap();
    init_git_repo(dir.path());
    init_and_start(dir.path(), "test session");
    attestack().current_dir(dir.path()).args(["run", "--", "echo", "hello"]).assert().success();
    attestack().current_dir(dir.path()).args(["note", "manual review"]).assert().success();
    attestack()
        .current_dir(dir.path())
        .args(["snapshot"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Git snapshot recorded"));
    stop_session(dir.path());

    let events_path = std::fs::read_dir(dir.path().join(".attestack/sessions"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
        .join("events.jsonl");
    let events = std::fs::read_to_string(events_path).unwrap();
    assert!(events.contains("command.started"));
    assert!(events.contains("git.snapshot"));
}

#[test]
fn run_false_exits_nonzero() {
    let dir = tempdir().unwrap();
    init_and_start(dir.path(), "demo");
    attestack()
        .current_dir(dir.path())
        .args(["run", "--", "false"])
        .assert()
        .code(predicate::eq(1));
}

#[test]
fn bundle_create_verify_and_detect_tampering() {
    let dir = tempdir().unwrap();
    init_git_repo(dir.path());
    init_and_start(dir.path(), "bundle demo");
    attestack().current_dir(dir.path()).args(["run", "--", "echo", "bundle"]).assert().success();
    stop_session(dir.path());
    attestack()
        .current_dir(dir.path())
        .args(["bundle", "create"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Bundle created"));

    let bundle_path = latest_bundle_path(dir.path());
    attestack()
        .current_dir(dir.path())
        .arg("verify")
        .arg(&bundle_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Verification passed"));

    let tampered = dir.path().join("tampered.attestack.zip");
    tamper_events_in_bundle(&bundle_path, &tampered);
    attestack()
        .current_dir(dir.path())
        .arg("verify")
        .arg(&tampered)
        .assert()
        .code(predicate::eq(1))
        .stdout(predicate::str::contains("Verification failed"));
}

#[test]
fn report_and_doctor_commands() {
    let dir = tempdir().unwrap();
    init_git_repo(dir.path());
    init_and_start(dir.path(), "cli report");
    attestack()
        .current_dir(dir.path())
        .args(["doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[pass] store"));
    attestack().current_dir(dir.path()).args(["note", "cli note"]).assert().success();
    stop_session(dir.path());
    attestack()
        .current_dir(dir.path())
        .args(["report"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cli report"))
        .stdout(predicate::str::contains("cli note"));
}

#[test]
fn cli_help_lists_subcommands() {
    attestack()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("report"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("bundle"))
        .stdout(predicate::str::contains("agent"))
        .stdout(predicate::str::contains("ci"));
}

#[test]
fn bundle_redact_paths_flag() {
    let dir = tempdir().unwrap();
    init_git_repo(dir.path());
    init_and_start(dir.path(), "redact");
    stop_session(dir.path());
    attestack()
        .current_dir(dir.path())
        .args(["bundle", "create", "--redact-paths"])
        .assert()
        .success();

    let bundle_path = latest_bundle_path(dir.path());
    let repo_path = dir.path().display().to_string();
    let file = File::open(&bundle_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        if !entry.name().ends_with("session.json") {
            continue;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(!text.contains(&repo_path), "session.json should redact absolute repo path");
    }
}

fn tamper_events_in_bundle(source: &std::path::Path, dest: &std::path::Path) {
    let source_file = File::open(source).unwrap();
    let mut archive = ZipArchive::new(source_file).unwrap();
    let dest_file = File::create(dest).unwrap();
    let mut writer = ZipWriter::new(dest_file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let name = entry.name().to_string();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if name.ends_with("events.jsonl") {
            bytes = b"{\"tampered\":true}\n".to_vec();
        }
        writer.start_file(name, options).unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap();
}
