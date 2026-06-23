mod common;

use common::{attestack, run_readme_demo_flow};
use predicates::prelude::*;
use tempfile::tempdir;

/// End-to-end test matching the README / docs quick-start demo flow.
#[test]
fn readme_demo_flow() {
    let dir = tempdir().unwrap();
    let bundle_path = run_readme_demo_flow(dir.path());

    let events = std::fs::read_to_string(
        std::fs::read_dir(dir.path().join(".attestack/sessions"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path()
            .join("events.jsonl"),
    )
    .unwrap();
    assert!(events.contains("command.started"));
    assert!(events.contains("git.snapshot"));
    assert!(events.contains("bundle.created"));

    attestack()
        .current_dir(dir.path())
        .args(["report"])
        .assert()
        .success()
        .stdout(predicate::str::contains("fix billing webhook"));
    assert!(bundle_path.is_file());
}
