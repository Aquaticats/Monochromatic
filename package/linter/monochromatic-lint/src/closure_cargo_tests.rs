//! What:
//!  Controls of the processes the closure check starts.
//! Why:
//!  The arguments and variables are the whole interface to cargo,
//! so the exact vector,
//! the offline posture,
//! the flags
//!  and the failure of each probe are checked against a stub that records what it receives.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('dylint arguments', () => { runDylint(stub, lib, manifest, setup); expect(stub.calls()).toEqual([...]); });
//! ```

use super::{
    CARGO_VARIABLE, DYLINT_FLAGS, cargo_program, dependencies_missing, dylint_probe, run_dylint,
    rustc_dev_missing, rustc_dev_missing_in,
};
use crate::{closure_probe::PackageSetup, closure_stub_support::scenario, test_fs::Fixture};
use std::path::{Path, PathBuf};

/// The default program is `cargo`,
/// and the variable's name is the documented one.
#[test]
fn the_variable_is_documented_and_the_default_is_cargo() {
    assert_eq!(CARGO_VARIABLE, "MONOCHROMATIC_LINT_CARGO");
    if std::env::var_os(CARGO_VARIABLE).is_none() {
        assert_eq!(cargo_program(), PathBuf::from("cargo"));
    }
}

/// A working `cargo dylint --version` passes and returns its line,
/// a failing one and a missing program are reasons to skip.
#[test]
fn dylint_availability_has_three_outcomes() {
    let working = scenario(&[("dylint-version.stdout", "cargo-dylint 6.3.0\n")]);
    assert_eq!(
        dylint_probe(&working.cargo),
        Ok(String::from("cargo-dylint 6.3.0\n"))
    );
    assert_eq!(
        working.calls(),
        vec![String::from("dylint-version|dylint --version|")]
    );
    let broken = scenario(&[("dylint-version.status", "101")]);
    assert!(
        dylint_probe(&broken.cargo)
            .expect_err("a reason")
            .contains("cargo dylint --version")
    );
    let missing =
        dylint_probe(Path::new("/nonexistent/monochromatic-lint/cargo")).expect_err("a reason");
    assert!(missing.contains("cargo does not run"), "{missing}");
}

/// A cold registry names the directory and the fetch command,
/// and runs offline and locked.
#[test]
fn a_cold_cache_names_the_fetch_command_and_the_directory() {
    let stub = scenario(&[
        ("metadata.status", "101"),
        (
            "metadata.stderr",
            "error: no matching package named `libc` found\nmore\n",
        ),
    ]);
    let manifest: PathBuf = stub.directory.join("pkg with space").join("Cargo.toml");
    let reason: String = dependencies_missing(&stub.cargo, &manifest).expect("cold");
    assert!(reason.contains("cargo fetch --locked"), "{reason}");
    assert!(reason.contains("pkg with space"), "{reason}");
    assert!(
        reason.contains("no matching package named `libc` found"),
        "{reason}"
    );
    assert!(
        !reason.contains("more"),
        "only the first line of cargo's message is quoted"
    );
    let calls: Vec<String> = stub.calls();
    assert_eq!(calls.len(), 1);
    assert!(
        calls[0].starts_with(
            "metadata|metadata --offline --locked --format-version 1 --manifest-path "
        ),
        "{}",
        calls[0]
    );
    assert!(
        calls[0].contains("pkg with space/Cargo.toml"),
        "{}",
        calls[0]
    );
}

/// A warm cache passes.
#[test]
fn a_warm_cache_passes() {
    let stub = scenario(&[]);
    assert_eq!(
        dependencies_missing(&stub.cargo, &stub.directory.join("Cargo.toml")),
        None
    );
}

/// A missing program is a reason,
/// not a panic.
#[test]
fn a_missing_cargo_is_a_reason_for_the_cache_probe() {
    let reason = dependencies_missing(
        Path::new("/nonexistent/monochromatic-lint/cargo"),
        Path::new("/x/Cargo.toml"),
    );
    assert!(reason.expect("a reason").contains("cargo does not run"));
}

/// The dylint run has the exact argument vector,
/// offline posture and lint flags,
/// and the setup is appended.
#[test]
fn the_dylint_run_has_the_documented_arguments_and_environment() {
    let stub = scenario(&[
        ("dylint.stdout", "{\"reason\":\"build-finished\"}\n"),
        ("dylint.stderr", "progress\n"),
    ]);
    let setup: PackageSetup = PackageSetup {
        environment: vec![(
            String::from("STUB_MARKER"),
            String::from("a value with spaces"),
        )],
        arguments: vec![
            String::from("--target"),
            String::from("aarch64-linux-android"),
        ],
    };
    let library: PathBuf = stub.directory.join("lib dir");
    let manifest: PathBuf = stub.directory.join("pkg dir").join("Cargo.toml");
    let run = run_dylint(&stub.cargo, &library, &manifest, &setup).expect("starts");
    assert!(run.success);
    assert_eq!(run.stdout, b"{\"reason\":\"build-finished\"}\n");
    assert_eq!(run.stderr, "progress\n");
    let expected: String = format!(
        "dylint|dylint --path {} --manifest-path {} -- --locked --message-format json --target aarch64-linux-android| DYLINT_RUSTFLAGS={DYLINT_FLAGS} CARGO_NET_OFFLINE=true STUB_MARKER=a value with spaces",
        library.display(),
        manifest.display()
    );
    assert_eq!(stub.calls(), vec![expected]);
}

/// The run does not use `--all-targets`,
/// so tests,
/// examples and benchmarks are not compiled.
#[test]
fn the_dylint_run_does_not_ask_for_all_targets() {
    let stub = scenario(&[]);
    run_dylint(
        &stub.cargo,
        Path::new("/lib"),
        Path::new("/p/Cargo.toml"),
        &PackageSetup::default(),
    )
    .expect("starts");
    assert!(!stub.calls()[0].contains("--all-targets"));
    assert!(!stub.calls()[0].contains("--tests"));
}

/// A failing run reports its status and still returns what it printed.
#[test]
fn a_failing_run_is_returned_with_its_status() {
    let stub = scenario(&[("dylint.status", "3"), ("dylint.stderr", "boom\n")]);
    let run = run_dylint(
        &stub.cargo,
        Path::new("/lib"),
        Path::new("/p/Cargo.toml"),
        &PackageSetup::default(),
    )
    .expect("starts");
    assert!(!run.success);
    assert!(run.status.contains('3'), "{}", run.status);
    assert_eq!(run.stderr, "boom\n");
}

/// A program that does not start is an error with a reason.
#[test]
fn a_program_that_does_not_start_is_an_error() {
    let error = run_dylint(
        Path::new("/nonexistent/monochromatic-lint/cargo"),
        Path::new("/lib"),
        Path::new("/p/Cargo.toml"),
        &PackageSetup::default(),
    )
    .expect_err("no program");
    assert!(error.contains("does not start"), "{error}");
}

/// The compiler check looks for `librustc_middle-*` in any target's library directory.
#[test]
fn rustc_dev_is_recognized_by_its_library_files() {
    let fixture: Fixture = Fixture::new();
    let sysroot: PathBuf = fixture.path.join("sysroot with space");
    assert!(
        rustc_dev_missing_in(&sysroot)
            .expect("no rustlib")
            .contains("cannot be read")
    );
    let library: PathBuf = sysroot
        .join("lib")
        .join("rustlib")
        .join("x86_64-unknown-linux-gnu")
        .join("lib");
    std::fs::create_dir_all(&library).expect("library directory");
    std::fs::create_dir_all(sysroot.join("lib").join("rustlib").join("etc"))
        .expect("a target without a lib directory");
    std::fs::write(library.join("libstd-1234.rlib"), "").expect("std only");
    assert!(
        rustc_dev_missing_in(&sysroot)
            .expect("missing")
            .contains("rustc-dev")
    );
    std::fs::write(library.join("librustc_middle-abcd.rmeta"), "").expect("rustc-dev file");
    assert_eq!(rustc_dev_missing_in(&sysroot), None);
}

/// A replacement `cargo` stands for the whole toolchain,
/// so the compiler of the machine is not inspected for it.
#[test]
fn a_replacement_cargo_is_not_asked_about_the_machine_compiler() {
    assert_eq!(rustc_dev_missing(Path::new("/some/stub/cargo")), None);
    assert_eq!(rustc_dev_missing(Path::new("cargo-nightly")), None);
}
