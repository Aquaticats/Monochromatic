//! What:
//! The part of the shim tests that watches a start that cannot happen:
//! a real `mpm` whose interpreter is missing gets the full explanation and the failure status,
//! the program the shadow starts learns how deep the nesting is,
//! and at the bound the shadow starts nothing and says why.
//! Why:
//! A person reading the output must learn which program failed,
//! at which stage,
//! and what to do,
//! and a nesting that would run the shadow for ever has to stop before it starts anything:
//! the marker file is what proves nothing started.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim/a_start_that_fails_and_the_nesting_bound.test.ts
//! test('a real mpm that cannot start is explained', () => { ... });
//! ```

use std::process::Output;

use mpm_shadow::constants::EXEC_FAILED_EXIT_CODE;
use mpm_shadow::nesting::MAX_NESTING as NESTING_LIMIT;

use crate::{
    fixture_root,
    start_shadow,
    start_shadow_with,
    write_fake_mpm,
    write_stale_copies,
};

/// A real mpm that cannot be started (its interpreter is missing) gets the full explanation,
/// the exit status 126,
/// and no cleanup.
#[test]
fn a_real_mpm_that_cannot_start_is_explained_with_command_program_stage_cause_and_status() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/nonexistent/interpreter\n");
    write_stale_copies(root.path());
    let program: String = root.path().join("real").join("mpm").display().to_string();

    let output: Output = start_shadow(root.path(), &["installed"]);

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(
        output.status.code(),
        Some(EXEC_FAILED_EXIT_CODE),
        "{stderr}"
    );
    for expected in [
        "mpm installed",
        &program,
        "starting the real `mpm`",
        "126",
        "could not be started",
        "post-run cleanup did not run",
    ] {
        assert!(stderr.contains(expected), "{expected}: {stderr}");
    }
    assert!(!stderr.contains("deleted stale"), "{stderr}");
    assert!(
        root.path()
            .join("cargo/registry/cache/index.crates.io-fixture/monochromatic-lint-0.1.0.crate")
            .exists(),
        "no cleanup ran"
    );
}

/// The program the shadow starts learns how deep it is.
#[test]
fn the_program_it_starts_gets_a_nesting_depth_one_deeper_than_its_own() {
    let root = fixture_root();
    write_fake_mpm(
        &root.path().join("real"),
        "#!/bin/sh\nprintf '%s' \"$MPM_SHADOW_NESTING\"\n",
    );

    let top: Output = start_shadow(root.path(), &[]);
    assert_eq!(String::from_utf8_lossy(&top.stdout), "1");

    let deeper: Output = start_shadow_with(root.path(), &[], &[("MPM_SHADOW_NESTING", "2")]);
    assert_eq!(String::from_utf8_lossy(&deeper.stdout), "3");
}

/// At the bound the shadow starts nothing and says why.
#[test]
fn a_shadow_nested_to_the_limit_refuses_to_start_another_program() {
    let root = fixture_root();
    let marker: std::path::PathBuf = root.path().join("ran");
    write_fake_mpm(
        &root.path().join("real"),
        &format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
    );

    let output: Output = start_shadow_with(
        root.path(),
        &["installed"],
        &[("MPM_SHADOW_NESTING", &NESTING_LIMIT.to_string())],
    );

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(
        output.status.code(),
        Some(EXEC_FAILED_EXIT_CODE),
        "{stderr}"
    );
    assert!(!marker.exists(), "the program must not have started");
    for expected in [
        "mpm installed",
        "started by another mpm-shadow",
        "itself",
        "126",
    ] {
        assert!(stderr.contains(expected), "{expected}: {stderr}");
    }
}
