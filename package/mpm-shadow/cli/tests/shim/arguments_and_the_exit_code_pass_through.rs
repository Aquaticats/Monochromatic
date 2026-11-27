//! What:
//! The part of the shim tests that starts the binary through a link named `mpm` and watches what a person's run
//! gets back:
//! every argument arrives untouched and in order,
//! the child's exit status is the shadow's own,
//! nothing is added to a run with nothing stale,
//! a missing real `mpm` is reported as the not-found status,
//! and `prune` deletes the stale copies and says so.
//! Why:
//! These are the promises `mpm` makes to whatever typed it,
//! and they hold only in a real process:
//! a unit test of the argument vector proves nothing about what the child receives.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim/arguments_and_the_exit_code_pass_through.test.ts
//! test('arguments and exit code pass through', () => { const out = startShadow(root, ['--output-format', 'json', 'a b']); ... });
//! ```

use std::process::{
    Command,
    Output,
};

use mpm_shadow::constants::NOT_FOUND_EXIT_CODE;

use crate::{
    fixture_root,
    start_shadow,
    write_fake_mpm,
    write_stale_copies,
};

/// The arguments arrive untouched,
/// the child's exit code passes through,
/// and the stale copies are gone afterwards.
#[test]
fn passes_arguments_and_exit_code_through_then_prunes() {
    let root = fixture_root();
    write_fake_mpm(
        &root.path().join("real"),
        "#!/bin/sh\nfor a in \"$@\"; do echo \"[$a]\"; done\nexit 3\n",
    );
    write_stale_copies(root.path());

    let output: Output = start_shadow(root.path(), &["--output-format", "json", "a b"]);

    // `String::from_utf8_lossy(&bytes)` reads captured output bytes as text, replacing invalid UTF-8.
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "[--output-format]\n[json]\n[a b]\n"
    );
    assert_eq!(output.status.code(), Some(3));
    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(stderr.contains("post-run cleanup"), "{stderr}");
    assert!(
        !stderr.contains("deleted stale"),
        "the per-path list is for MPM_SHADOW_VERBOSE: {stderr}"
    );
    let cargo: std::path::PathBuf = root.path().join("cargo");
    assert!(
        !cargo
            .join("registry/cache/index.crates.io-fixture/monochromatic-lint-0.1.0.crate")
            .exists()
    );
    assert!(
        !cargo
            .join("registry/src/index.crates.io-fixture/monochromatic-lint-0.1.0")
            .exists()
    );
    assert!(
        cargo
            .join("registry/cache/index.crates.io-fixture/monochromatic-lint-0.4.0.crate")
            .exists()
    );
}

/// Standard output stays exactly mpm's when there is nothing to prune.
#[test]
fn adds_nothing_to_standard_output_or_error_when_nothing_is_stale() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\necho hello\n");

    let output: Output = start_shadow(root.path(), &[]);

    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    assert_eq!(output.status.code(), Some(0));
}

/// With no real mpm on PATH the shadow reports it and exits 127.
#[test]
fn exits_127_when_no_real_mpm_is_on_path() {
    let root = fixture_root();

    let output: Output = start_shadow(root.path(), &[]);

    assert_eq!(output.status.code(), Some(NOT_FOUND_EXIT_CODE));
    assert!(String::from_utf8_lossy(&output.stderr).contains("no real mpm found on PATH"));
}

/// Started as `mpm-shadow`,
/// the `prune` subcommand deletes the stale copies and says which.
#[test]
fn the_prune_subcommand_deletes_and_reports() {
    let root = fixture_root();
    write_stale_copies(root.path());

    let output: Output = Command::new(env!("CARGO_BIN_EXE_mpm-shadow"))
        .arg("prune")
        .env("CARGO_HOME", root.path().join("cargo"))
        .output()
        .expect("run mpm-shadow prune");

    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("deleted stale"));
    assert!(
        !root
            .path()
            .join("cargo/registry/cache/index.crates.io-fixture/monochromatic-lint-0.1.0.crate")
            .exists()
    );
}
