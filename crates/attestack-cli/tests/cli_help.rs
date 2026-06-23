use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_works() {
    Command::cargo_bin("attestack")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Local-first proof layer"));
}
