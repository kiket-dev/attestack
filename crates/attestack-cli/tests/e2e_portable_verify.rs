mod common;

use common::{attestack, run_readme_demo_flow};
use predicates::prelude::*;
use tempfile::tempdir;

/// A bundle copied outside the session tree still verifies when the store identity is available.
#[test]
fn verify_copied_bundle_by_absolute_path() {
    let repo = tempdir().unwrap();
    let bundle_path = run_readme_demo_flow(repo.path());

    let outside = tempdir().unwrap();
    let copied = outside.path().join("portable.attestack.zip");
    std::fs::copy(&bundle_path, &copied).unwrap();

    attestack()
        .current_dir(repo.path())
        .arg("verify")
        .arg(copied.canonicalize().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Verification passed"));
}
