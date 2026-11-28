//! What:
//!  Plan step 21's pnpm and Cargo arms:
//!  the offline frozen-lockfile install after an add,
//!  and the Cargo clean with its lock recovered across file systems,
//!  each beside an interleaved control on the checkout's own dataset.
//! Why:
//!  The plan's acceptance asks for "real dependency preparation after add"
//!  with "the same-dataset control ... using the same lockfile and home store"
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 21).
//!  The control of each arm is the same tool doing the same work on the same
//!  bytes,
//!  on the checkout's own dataset,
//!  so a ratio says what the mounted clone costs and never what the tool's own
//!  work costs.
//!  The state each arm needs is `binary_zfs_step_21_dependency_fixtures.rs`'s,
//!  the reading of the repair's line is
//!  `binary_zfs_step_21_repair_readings.rs`'s,
//!  and the launcher arm beside them is `binary_zfs_step_21_launcher_arm.rs`'s.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const control = time(() => pnpmInstall(controlFolder)); const arm = timeAdd('pnpm-0'); report(arm, control);
//! ```

#![cfg(target_os = "linux")]

use super::zfs_step_21_dependency_fixtures::{
    CARGO,
    NODE,
    PNPM,
    PRISTINE_MODULES,
    RESOLVED,
    VENDOR,
    the_cargo_workspace,
    the_node_project,
};
use super::zfs_step_21_guest_fixture::{
    AddRound,
    Fixture,
};
use super::zfs_step_21_guest_layout::{
    ROUNDS,
    device_of,
};
use super::zfs_step_21_repair_readings::{
    report,
    the_log_says,
    the_time_of,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::Instant;

/// What:
///  Run pnpm's offline frozen-lockfile install in a folder the arm prepared,
///  and time it.
/// Why:
///  This is the control of the pnpm arm:
///  the same command,
///  the same lockfile,
///  the same home store,
///  on the checkout's own dataset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const started = now(); fixture.run(PNPM, ['install', '--offline', '--frozen-lockfile'], control); return started.elapsed();
/// ```
fn pnpm_in_a_control_folder(
    fixture: &Fixture,
    round: usize,
) -> u128 {
    let control: PathBuf = fixture.control("pnpm", round);
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(control.as_path());
    std::fs::create_dir_all(control.as_path()).expect("the control folder");
    for name in ["package.json", "pnpm-lock.yaml"] {
        std::fs::copy(fixture.root().join(name), control.join(name))
            .unwrap_or_else(|error| panic!("{name} could not be copied: {error}"));
    }
    // The tarball the lockfile names lives in the project, so the control needs it as the clone does.
    fixture.run(
        Path::new("/usr/bin/cp"),
        &["--archive", VENDOR, control.to_str().expect("a text path")],
        fixture.root(),
    );
    fixture.run(
        Path::new("/usr/bin/tar"),
        &[
            "--extract",
            "--file",
            PRISTINE_MODULES,
            "--directory",
            control.to_str().expect("a text path"),
        ],
        control.as_path(),
    );
    let started: Instant = Instant::now();
    fixture.run(
        Path::new(PNPM),
        &["install", "--offline", "--frozen-lockfile"],
        control.as_path(),
    );
    let elapsed: u128 = started.elapsed().as_millis();
    std::fs::remove_dir_all(control.as_path()).expect("the control folder goes");
    println!("STEP21 pnpm control round={round} ms={elapsed}");
    return elapsed;
}

/// What:
///  Add a worktree of the node project and read the time the repair credits
///  pnpm with,
///  then resolve one dependency through the clone's own `node_modules`.
/// Why:
///  The install is the add's own,
///  so the arm times it from the repair's line instead of from a second run of
///  the command.
///  A resolution through the clone is what a relink that never ran fails at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const added = fixture.add('pnpm-0'); const millis = theTimeOf(added.said, 'pnpm relinked'); resolveThroughTheClone();
/// ```
fn pnpm_after_an_add(
    fixture: &Fixture,
    round: usize,
) -> (u128, u128) {
    let name: String = format!("pnpm-{round}");
    let added: AddRound = fixture.add(name.as_str(), None);
    assert_eq!(
        added.status,
        Some(0),
        "the add of {name} was refused: {}",
        added.said
    );
    let millis: u128 = the_time_of(added.said.as_str(), "pnpm relinked node_modules");
    let slot: PathBuf = fixture.slot(name.as_str());
    let resolved: std::process::Output = fixture.run(
        Path::new(NODE),
        &["--print", format!("require.resolve('{RESOLVED}')").as_str()],
        slot.as_path(),
    );
    let answer: String = String::from_utf8_lossy(resolved.stdout.as_slice())
        .trim()
        .to_string();
    assert!(
        answer.starts_with(slot.to_str().expect("a text path")),
        "the clone resolves {RESOLVED} outside its own tree: {answer}"
    );
    println!("STEP21 pnpm arm round={round} ms={millis} resolves={answer}");
    fixture.clear(name.as_str());
    return (millis, added.elapsed.as_millis());
}

/// What:
///  The pnpm arm:
///  one home store and one lockfile prepared once,
///  then interleaved rounds of the control and the add.
/// Why:
///  The plan asks for the offline frozen-lockfile install after an add beside a
///  same-dataset control,
///  and the rounds alternate which arm goes first so neither is warmed by the
///  other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const store = warmTheStore(); for (let n = 0; n < ROUNDS; n += 1) { n % 2 === 0 ? controlThenAdd(n) : addThenControl(n); }
/// ```
pub(super) fn the_pnpm_arm(fixture: &Fixture) {
    fixture.forget(&[], "nothing to remove before the node project");
    the_node_project(fixture);
    fixture.assert_clean();
    let mut controls: Vec<u128> = Vec::new();
    let mut measured: Vec<u128> = Vec::new();
    for round in 0..ROUNDS {
        if round % 2 == 0 {
            controls.push(pnpm_in_a_control_folder(fixture, round));
            measured.push(pnpm_after_an_add(fixture, round).0);
        } else {
            measured.push(pnpm_after_an_add(fixture, round).0);
            controls.push(pnpm_in_a_control_folder(fixture, round));
        }
    }
    report("pnpm", controls.as_slice(), measured.as_slice());
}

/// What:
///  Run the Cargo clean in a copy of the workspace on the checkout's dataset,
///  and time it.
/// Why:
///  This is the control of the Cargo arm:
///  the same clean,
///  the same lock,
///  the same built build directory,
///  on the dataset the clone was snapshotted from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copy = reflinkOf(workspace); const started = now(); fixture.run(CARGO, ['clean', '--offline', '--package', 'one'], copy); return started.elapsed();
/// ```
fn cargo_in_a_control_folder(
    fixture: &Fixture,
    round: usize,
) -> u128 {
    let control: PathBuf = fixture.control("cargo", round);
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(control.as_path());
    std::fs::create_dir_all(control.as_path()).expect("the control folder");
    for name in ["Cargo.toml", "Cargo.lock", "target", "crates"] {
        fixture.run(
            Path::new("/usr/bin/cp"),
            &[
                "--archive",
                "--reflink=auto",
                name,
                control.to_str().expect("a text path"),
            ],
            fixture.root(),
        );
    }
    let started: Instant = Instant::now();
    fixture.run(
        Path::new(CARGO),
        &["clean", "--offline", "--package", "one"],
        control.as_path(),
    );
    let elapsed: u128 = started.elapsed().as_millis();
    std::fs::remove_dir_all(control.as_path()).expect("the control folder goes");
    println!("STEP21 cargo control round={round} ms={elapsed}");
    return elapsed;
}

/// What:
///  Add a worktree of the Cargo workspace and read the time the repair credits
///  its clean with,
///  then check the lock the clean displaced.
/// Why:
///  The clone's Git directory lies on the checkout's dataset while the worktree
///  lies on its own,
///  so the exchange that would put the checked-out lock back cannot happen and
///  its bytes are written over the lock instead;
///  the arm says so from the two devices and from the copy the repair kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const added = fixture.add('cargo-0'); const millis = theTimeOf(added.said, 'Cargo kept'); keptLockExists(); devicesDiffer();
/// ```
fn cargo_after_an_add(
    fixture: &Fixture,
    round: usize,
) -> u128 {
    let name: String = format!("cargo-{round}");
    let added: AddRound = fixture.add(name.as_str(), None);
    assert_eq!(
        added.status,
        Some(0),
        "the add of {name} was refused: {}",
        added.said
    );
    let millis: u128 = the_time_of(
        added.said.as_str(),
        "Cargo kept dependency builds and cleaned the worktree's own crates",
    );
    assert!(
        the_log_says(
            added.said.as_str(),
            &["packageRepairCargo", "the clean runs here instead"]
        ),
        "the add did not take the clean-in-place branch: {}",
        added.said
    );
    let slot: PathBuf = fixture.slot(name.as_str());
    let checked_out: Vec<u8> =
        std::fs::read(fixture.root().join("Cargo.lock")).expect("the checkout's lock");
    let in_the_clone: Vec<u8> = std::fs::read(slot.join("Cargo.lock")).expect("the clone's lock");
    assert_eq!(
        in_the_clone, checked_out,
        "the clone's lock is not the checked-out one"
    );
    let displaced: PathBuf = fixture
        .common()
        .join("monochromatic-git-policy-cli-displaced-locks");
    let kept: usize = std::fs::read_dir(displaced.as_path())
        .map(|entries| return entries.count())
        .unwrap_or(0);
    let worktree_device: u64 = device_of(slot.as_path());
    let git_device: u64 = device_of(fixture.common().as_path());
    println!(
        "STEP21 cargo arm round={round} ms={millis} displaced-folders={kept} worktree-device={worktree_device} git-device={git_device} exchange-possible={}",
        worktree_device == git_device
    );
    assert!(
        kept > 0,
        "the clean displaced no lock, so nothing was kept under {}",
        displaced.display()
    );
    assert_ne!(
        worktree_device, git_device,
        "the clone and its Git directory are on one device, so the exchange that puts the lock back can happen"
    );
    fixture.clear(name.as_str());
    return millis;
}

/// What:
///  The Cargo arm:
///  the node project taken away,
///  the workspace built once,
///  its lock stripped of the sibling the manifest names,
///  a staging name planted so the add cleans where the state stands,
///  then interleaved rounds of the control and the add.
/// Why:
///  The plan asks for the Cargo prewarm and the cross-filesystem lock recovery;
///  a clean that runs in the add is the only path where the add itself reads
///  and puts back the lock,
///  and the add takes it when the copied build state cannot be moved aside.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetTheNodeProject(); const built = buildTheWorkspace(); stripTheSiblingFromTheLock(); plantTheStagingName(); rounds();
/// ```
pub(super) fn the_cargo_arm(fixture: &Fixture) {
    fixture.forget(
        &["package.json", "pnpm-lock.yaml"],
        "the node project goes, so one add runs one tool",
    );
    fixture.remove_ignored(&["node_modules"]);
    let lock: String = the_cargo_workspace(fixture);
    let staging: PathBuf = fixture.root().join("target.monochromatic-prewarm");
    // A name this wrapper did not stage makes the add leave the copied state where it is and clean it here instead.
    std::fs::create_dir_all(staging.as_path()).expect("the planted staging name");
    fixture.assert_clean();
    let probe: std::process::Output = fixture.run(
        Path::new(CARGO),
        &["metadata", "--locked", "--offline", "--no-deps"],
        fixture.root(),
    );
    println!(
        "STEP21 cargo lock-probe metadata-locked-bytes={} lock-bytes={}",
        probe.stdout.len(),
        lock.len()
    );
    let mut controls: Vec<u128> = Vec::new();
    let mut measured: Vec<u128> = Vec::new();
    for round in 0..ROUNDS {
        if round % 2 == 0 {
            controls.push(cargo_in_a_control_folder(fixture, round));
            measured.push(cargo_after_an_add(fixture, round));
        } else {
            measured.push(cargo_after_an_add(fixture, round));
            controls.push(cargo_in_a_control_folder(fixture, round));
        }
    }
    report("cargo", controls.as_slice(), measured.as_slice());
}
