//! What:
//! The tests of the runner that starts the real cargo for the install and update steps.
//! Why:
//! Every start a shadow makes carries the arrival marker of `shadow_identity.rs`,
//! so a program that turns out to be a shadow stops at its first instruction.
//! The candidate check and the loop guard recognize a shadow by its file or by its record;
//! what they cannot see
//! (a script that runs a shadow,
//! a launcher with no record)
//! is stopped by the marker,
//! and the cargo of an install step was the one start without one
//! (`doc/planning/one-release-each-batch-2026-11.md`,
//! section "Not done here").

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::cargo_run::{CargoRunner, RealCargo};
use crate::shadow_identity::{ARRIVAL_VARIABLE, Arrival, StartMode, own_identity, parse_arrival, process_birth};
use crate::test_scratch::scratch;

/// The longest one cargo call may take here:
/// the script below ends at once,
/// and the bound only keeps a broken start from waiting.
const SCRIPT_BOUND_SECONDS: u64 = 60;

/// Writes an executable shell script named `name` into `directory`.
fn executable_script(directory: &Path, name: &str, body: &str) -> PathBuf {
    let path: PathBuf = directory.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("write the script");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("make it executable");
    return path;
}

/// The cargo of an install step is started with the arrival marker of this shadow,
/// which names this process as the one that started it.
/// The test process may itself have been started through a shadow (`mise` runs the task's `cargo`,
/// and that `cargo` is the shadow),
/// so the marker it inherited is read first and the child's marker has to be a different one.
#[test]
fn the_cargo_of_an_install_step_arrives_with_the_marker_of_the_shadow_that_started_it() {
    let directory = scratch();
    let body: String = format!("printf '%s' \"${ARRIVAL_VARIABLE}\"");
    let program: PathBuf = executable_script(directory.path(), "cargo", &body);
    let inherited: Option<String> = std::env::var(ARRIVAL_VARIABLE).ok();
    let cargo = RealCargo { program, toolchain: String::new(), bound: Duration::from_secs(SCRIPT_BOUND_SECONDS) };

    let outcome = cargo.run(directory.path(), &["metadata".to_string()]).expect("the script starts");

    assert!(outcome.ok, "{outcome:?}");
    let marker: &str = outcome.stdout.trim();
    assert!(inherited.as_deref() != Some(marker), "the install step gave its cargo no marker of its own; it printed the inherited {marker:?}");
    let arrival: Arrival = parse_arrival(marker).expect("the marker parses");
    assert_eq!(arrival.mode, StartMode::Spawn, "{marker}");
    assert_eq!(arrival.pid, std::process::id(), "{marker}");
    assert_eq!(arrival.birth, process_birth(std::process::id()), "{marker}");
    assert_eq!(arrival.starter, own_identity(), "{marker}");
}
