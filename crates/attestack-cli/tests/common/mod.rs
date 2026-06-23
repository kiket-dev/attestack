use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use assert_cmd::Command as AssertCommand;

pub fn init_git_repo(path: &Path) {
    Command::new("git").args(["init", "-b", "main"]).current_dir(path).output().unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()
        .unwrap();
    fs::write(path.join("README.md"), "hello\n").unwrap();
    Command::new("git").args(["add", "README.md"]).current_dir(path).output().unwrap();
    Command::new("git").args(["commit", "-m", "init"]).current_dir(path).output().unwrap();
}

pub fn attestack() -> AssertCommand {
    AssertCommand::cargo_bin("attestack").unwrap()
}

pub fn init_and_start(repo: &Path, title: &str) {
    attestack().current_dir(repo).args(["init"]).assert().success();
    attestack().current_dir(repo).args(["start", title]).assert().success();
}

pub fn stop_session(repo: &Path) {
    attestack().current_dir(repo).args(["stop"]).assert().success();
}

pub fn latest_bundle_path(repo: &Path) -> PathBuf {
    fs::read_dir(repo.join(".attestack/bundles")).unwrap().next().unwrap().unwrap().path()
}

#[allow(dead_code)]
pub fn run_readme_demo_flow(repo: &Path) -> PathBuf {
    init_git_repo(repo);
    init_and_start(repo, "fix billing webhook");
    attestack().current_dir(repo).args(["run", "--", "echo", "hello"]).assert().success();
    attestack()
        .current_dir(repo)
        .args(["note", "Reviewed AI-generated auth changes manually."])
        .assert()
        .success();
    attestack().current_dir(repo).args(["snapshot"]).assert().success();
    stop_session(repo);
    attestack()
        .current_dir(repo)
        .args(["bundle", "create"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Bundle created"));
    let bundle_path = latest_bundle_path(repo);
    attestack()
        .current_dir(repo)
        .arg("verify")
        .arg(&bundle_path)
        .assert()
        .success()
        .stdout(predicates::str::contains("Verification passed"));
    bundle_path
}
