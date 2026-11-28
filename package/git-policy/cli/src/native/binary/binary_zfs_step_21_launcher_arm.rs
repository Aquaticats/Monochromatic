//! What:
//!  Plan step 21's mise launcher arm:
//!  a worktree whose launcher record names another checkout's lock,
//!  the repair that rebuilds it beside an interleaved control.
//! Why:
//!  The plan asks to check "mise and launcher repair"
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 21),
//!  and the guest recipe carries no `mpm-shadow`,
//!  so the program the repair starts is a stand-in that records its arguments,
//!  prints the summary line a real install prints and writes the record an
//!  install writes.
//!  The state it repairs is `binary_zfs_step_21_dependency_fixtures.rs`'s and the
//!  reading of the repair's line is
//!  `binary_zfs_step_21_repair_readings.rs`'s.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! theLauncherState(fixture); const added = fixture.add('launcher-0'); theTimeOf(added.said, 'mpm-shadow rebuilt');
//! ```

#![cfg(target_os = "linux")]

use super::zfs_step_21_dependency_fixtures::{
    ELSEWHERE,
    LAUNCHER,
    MPM_START_LOG,
    the_launcher_state,
};
use super::zfs_step_21_guest_fixture::{
    AddRound,
    Fixture,
};
use super::zfs_step_21_guest_layout::ROUNDS;
use super::zfs_step_21_repair_readings::{
    report,
    the_time_of,
};
use std::path::PathBuf;
use std::time::Instant;

/// What:
///  Run the stand-in the way the repair runs it,
///  in a folder of the arm's own,
///  and time it.
/// Why:
///  This is the launcher arm's control:
///  the same program with the same arguments and the same root,
///  started by the arm instead of by the add's repair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = controlFolder(round); writeRecord(folder); const started = now(); run(standIn, ['install', '--frozen-lockfile'], folder, {MONOCHROMATIC_WORKTREE_ROOT: folder}); return started.elapsed();
/// ```
fn launcher_in_a_control_folder(
    fixture: &Fixture,
    round: usize,
) -> u128 {
    let control: PathBuf = fixture.control("launcher", round);
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(control.as_path());
    let launchers: PathBuf = control.join(".cache").join("toolchain-launchers");
    std::fs::create_dir_all(launchers.as_path()).expect("the control's launcher folder");
    std::fs::write(
        launchers.join(format!("{LAUNCHER}.launch")),
        format!(
            "{{\"toolId\":\"step21:own\",\"lockFile\":\"{ELSEWHERE}/toolchain.lock.json\",\"real\":\"/tools/own\",\"scope\":\"repo\"}}\n"
        )
        .as_bytes(),
    )
    .expect("the control's record");
    let stand_in: PathBuf = fixture
        .root()
        .join(".cache")
        .join("toolchain-bin")
        .join("mpm-shadow");
    let started: Instant = Instant::now();
    let status: std::process::ExitStatus = std::process::Command::new(stand_in)
        .args(["install", "--frozen-lockfile"])
        .current_dir(control.as_path())
        .env("MONOCHROMATIC_WORKTREE_ROOT", control.as_path())
        .status()
        .expect("the stand-in starts");
    let elapsed: u128 = started.elapsed().as_millis();
    assert!(status.success(), "the control's stand-in refused");
    std::fs::remove_dir_all(control.as_path()).expect("the control folder goes");
    println!("STEP21 launcher control round={round} ms={elapsed}");
    return elapsed;
}

/// What:
///  Add a worktree whose launcher record names another checkout's lock,
///  and read what the repair did:
///  the time it credited `mpm-shadow`,
///  the arguments and the root the stand-in was given,
///  and the record the install left.
/// Why:
///  The plan asks to check the mise launcher repair;
///  the record naming this worktree's own lock after the add is the repair's
///  result,
///  and the root the program was given is how it knew which checkout to relink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const added = fixture.add('launcher-0'); const millis = theTimeOf(added.said, 'mpm-shadow rebuilt'); recordNamesTheClone(); startLineNamesTheRoot();
/// ```
fn launcher_after_an_add(
    fixture: &Fixture,
    round: usize,
) -> u128 {
    let name: String = format!("launcher-{round}");
    let _cleared: std::io::Result<()> = std::fs::remove_file(MPM_START_LOG);
    let added: AddRound = fixture.add(name.as_str(), None);
    assert_eq!(
        added.status,
        Some(0),
        "the add of {name} was refused: {}",
        added.said
    );
    let millis: u128 = the_time_of(
        added.said.as_str(),
        "mpm-shadow rebuilt the toolchain links",
    );
    let slot: PathBuf = fixture.slot(name.as_str());
    let record: String = std::fs::read_to_string(
        slot.join(".cache")
            .join("toolchain-launchers")
            .join(format!("{LAUNCHER}.launch")),
    )
    .expect("the record the install left");
    assert!(
        record.contains(slot.to_str().expect("a text path")),
        "the record does not name the clone's own lock: {record}"
    );
    let started: String = std::fs::read_to_string(MPM_START_LOG).expect("the stand-in's own line");
    println!(
        "STEP21 launcher arm round={round} ms={millis} start={started} record-names-the-clone={}",
        record.contains(slot.to_str().expect("a text path"))
    );
    assert!(
        started.contains("install --frozen-lockfile"),
        "the stand-in was not asked for the install: {started}"
    );
    assert!(
        started.contains(slot.to_str().expect("a text path")),
        "the stand-in was given no root naming the clone: {started}"
    );
    fixture.clear(name.as_str());
    return millis;
}

/// What:
///  The launcher arm:
///  the record naming another lock prepared once,
///  then interleaved rounds of the control and the add.
/// Why:
///  The plan asks to check the mise launcher repair,
///  and a ratio to a control run in the same round is how a timing here is
///  reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theLauncherState(); for (let n = 0; n < ROUNDS; n += 1) { alternate(); }
/// ```
pub(super) fn the_launcher_arm(fixture: &Fixture) {
    fixture.forget(
        &[
            "Cargo.toml",
            "Cargo.lock",
            "crates/one/Cargo.toml",
            "crates/two/Cargo.toml",
            "crates/one/src/lib.rs",
            "crates/two/src/lib.rs",
        ],
        "the cargo workspace goes, so one add runs one tool",
    );
    fixture.remove_ignored(&[
        "target",
        "target.monochromatic-prewarm",
        "crates",
        ".step21",
    ]);
    the_launcher_state(fixture);
    fixture.assert_clean();
    let mut controls: Vec<u128> = Vec::new();
    let mut measured: Vec<u128> = Vec::new();
    for round in 0..ROUNDS {
        if round % 2 == 0 {
            controls.push(launcher_in_a_control_folder(fixture, round));
            measured.push(launcher_after_an_add(fixture, round));
        } else {
            measured.push(launcher_after_an_add(fixture, round));
            controls.push(launcher_in_a_control_folder(fixture, round));
        }
    }
    report("launcher", controls.as_slice(), measured.as_slice());
}
