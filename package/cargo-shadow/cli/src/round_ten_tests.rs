//! What:
//! Round 10 tests: the unsettled scope (9a D1), the strict boot identity (9a D2), the discovery's shared deadline (9a D3), the
//! authority of the boot id (9a section 4), and the false greens of part 9b.
//! Why:
//! The review of round 9 (`cargo-shadow-sol-9a.txt`, `-9b.txt`) named schedules inside existing rows of the acceptance list;
//! each test here is one of them, with a handshake at the boundary and not an elapsed sleep.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('round ten', () => { it('an unsettled empty scope is not a run that ended', ...) })
//! ```

use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::rc::Rc;

use crate::cgroup_test_support::{delegated_route, start_in_scope};
use crate::control_group::{ControlGroup, Identity, Route, current_identity, read_current_identity, use_identity_for_test, view_of};
use crate::crates::LockSnapshot;
use crate::journal::recover;
use crate::journal_recover::survivor_note;
use crate::journal_support::plant;
use crate::journal_unready::after_query_for_test;
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/n/one";

/// The live text of the lockfile a killed run left.
const LEFT_BY_THE_KILLED_RUN: &str = "unchecked output of the killed run";

/// A snapshot of the lockfile of the case.
fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: source.to_string() };
}

/// A disposable repository whose crate root holds the output of a killed run.
fn repository() -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), LEFT_BY_THE_KILLED_RUN).expect("lock");
    return root;
}

/// A judge that refuses the live lockfile, so that a recovery which goes ahead restores the snapshot.
fn refusing(_before: &[LockSnapshot], _live: &[LockSnapshot]) -> Vec<String> {
    return vec!["dep@1.0.1 refused".to_string()];
}

/// Plants a journal whose `groups` file holds `records`.
fn journal_with_records(root: &Path, records: &str) -> PathBuf {
    let journal: PathBuf = plant(root, &[snapshot("original")], &[]);
    fs::write(journal.join("groups"), records).expect("write the records");
    return journal;
}

/// The text of the live lockfile.
fn live_lock(root: &Path) -> String {
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).expect("lock");
}

/// A route whose scopes are made inside a delegated holder scope, which the user may write to,
/// so a test can create and remove a control group by hand exactly where a manager would.
fn route_inside(holder: &Path, route: &Route) -> Route {
    return Route { slice_dir: holder.to_path_buf(), slice_view: view_of(holder).expect("the view of the holder"), ..route.clone() };
}

/// Acceptance rows A1, V2, V3/V4 (9a D1, creation before placement):
/// the manager has made the scope's control group and not yet attached the launcher, so the scope exists with `populated 0`.
/// The run did not record its end, so a recovery must not read that empty scope as a run that ended.
#[test]
fn an_unsettled_scope_that_exists_empty_is_not_read_as_a_run_that_ended() {
    let Some(route) = delegated_route() else {
        return;
    };
    let Some(holder) = start_in_scope(Command::new("sleep").arg("600")) else {
        return;
    };
    let inside: Route = route_inside(holder.group.directory(), &route);
    let planned: ControlGroup = ControlGroup::plan(&inside).expect("a scope name");
    fs::create_dir(planned.directory()).expect("make the scope's control group the way the manager does, before it attaches a process");
    assert_eq!(fs::read_to_string(planned.directory().join("cgroup.events")).expect("events").lines().find(|line| { return line.starts_with("populated"); }), Some("populated 0"));
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), &format!("{}\n", planned.record()));

    let note: Option<String> = survivor_note(&journal);
    let messages: Vec<String> = recover(root.path(), &refusing);
    let lock: String = live_lock(root.path());
    let _ = fs::remove_dir(planned.directory());

    let blocked: String = note.expect("an unsettled, empty scope blocks");
    assert!(blocked.contains("did not record its end") && blocked.contains("cargo-shadow acknowledge"), "{blocked}");
    assert_eq!(lock, LEFT_BY_THE_KILLED_RUN, "a recovery restored files while a program could still be attached: {messages:?}");
}

/// Acceptance rows A1, V2, V3/V4 (9a D1, the race between two observations):
/// the check finds the scope absent, and the launcher's request then makes it before the check ends.
/// Nothing may decide from a second look that the run is complete.
#[test]
fn a_scope_that_appears_between_the_query_and_the_judgement_is_not_read_as_a_run_that_ended() {
    let Some(route) = delegated_route() else {
        return;
    };
    let Some(holder) = start_in_scope(Command::new("sleep").arg("600")) else {
        return;
    };
    let inside: Route = route_inside(holder.group.directory(), &route);
    let planned: ControlGroup = ControlGroup::plan(&inside).expect("a scope name");
    assert!(!planned.directory().exists());
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), &format!("{}\n", planned.record()));
    let directory: PathBuf = planned.directory().to_path_buf();
    after_query_for_test(Some(Rc::new(move || { fs::create_dir(&directory).expect("the launcher's request makes the scope"); })));

    let note: Option<String> = survivor_note(&journal);
    after_query_for_test(None);
    let created: bool = planned.directory().exists();
    let _ = fs::remove_dir(planned.directory());

    assert!(created, "the hook did not run between the query and the judgement");
    let blocked: String = note.expect("a scope that appeared after the query blocks");
    assert!(blocked.contains("did not record its end"), "{blocked}");
}

/// A process in a process group of its own, which is alive until the test kills it.
fn live_group() -> Child {
    return Command::new("sleep").arg("600").process_group(0).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("a live group");
}

/// This machine's identity with its boot id changed in one hexadecimal digit, as a real reboot would change it.
fn after_a_reboot() -> Identity {
    let now: Identity = current_identity().expect("this machine has an identity");
    let mut boot: Vec<u8> = now.boot.into_bytes();
    boot[0] = if boot[0] == b'0' { b'1' } else { b'0' };
    return Identity { boot: String::from_utf8(boot).expect("ascii"), machine: now.machine };
}

/// Rewrites the identity lines of the journal's view record and keeps its first line.
fn set_identity_lines(journal: &Path, lines: &str) {
    let recorded: String = fs::read_to_string(journal.join("view")).expect("the view record");
    let first: &str = recorded.lines().next().expect("a view line");
    fs::write(journal.join("view"), format!("{first}\n{lines}")).expect("rewrite the view");
}

/// Acceptance rows V2 and the weaker path (9a D2, recorded side):
/// a recorded boot or machine field that is not in the format of an id is not a different boot,
/// a repeated field or an unknown line makes the record unusable, and each of them keeps a live process group blocking.
/// The control is a well-formed record of another boot, which does clear it (so the probe can move).
#[test]
fn a_malformed_recorded_identity_is_not_a_reboot() {
    let root = repository();
    let mut group = live_group();
    let journal: PathBuf = journal_with_records(root.path(), &format!("pgroup {}\n", group.id()));
    let now: Identity = current_identity().expect("an identity");
    let other: Identity = after_a_reboot();
    let boot_line = |boot: &str| -> String { return format!("boot {boot}\nmachine {}\n", now.machine); };
    let cases: Vec<(&str, String)> = vec![
        ("a boot field that is not an id", boot_line("not-a-boot-id")),
        ("an upper-case boot id", boot_line(&other.boot.to_uppercase())),
        ("a boot id with a trailing space", boot_line(&format!("{} ", other.boot))),
        ("an all-zero boot id", boot_line("00000000-0000-0000-0000-000000000000")),
        ("a machine field that is not an id", format!("boot {}\nmachine not-a-machine-id\n", other.boot)),
        ("a repeated boot field", format!("boot {}\nboot {}\nmachine {}\n", other.boot, now.boot, now.machine)),
        ("a repeated machine field", format!("boot {}\nmachine {}\nmachine {}\n", other.boot, now.machine, now.machine)),
        ("an unknown line", format!("boot {}\nmachine {}\nthree {}\n", other.boot, now.machine, now.machine)),
    ];
    for (label, lines) in &cases {
        set_identity_lines(&journal, lines);
        let note: Option<String> = survivor_note(&journal);
        assert!(note.is_some_and(|text| { return text.contains("still running"); }), "{label}: a malformed identity cleared a live process group");
    }
    // The record says another boot than the one running now: the machine is the same, so a reboot ended the group.
    set_identity_lines(&journal, &boot_line(&other.boot));
    let cleared: Option<String> = survivor_note(&journal);
    let _ = group.kill();
    let _ = group.wait();
    assert_eq!(cleared, None, "the control: a well-formed record of another boot did not clear the group, so the probe cannot move");
}

/// Acceptance rows V2 and the weaker path (9a D2, current side):
/// a current identity that is not well formed disables the shortcut instead of showing a reboot.
#[test]
fn a_malformed_current_identity_is_not_a_reboot() {
    let root = repository();
    let mut group = live_group();
    let journal: PathBuf = journal_with_records(root.path(), &format!("pgroup {}\n", group.id()));
    let now: Identity = current_identity().expect("an identity");
    let cases: Vec<(&str, Identity)> = vec![
        ("a current boot field that is not an id", Identity { boot: "not-a-boot-id".to_string(), machine: now.machine.clone() }),
        ("a current machine field that is not an id", Identity { boot: after_a_reboot().boot, machine: "not-a-machine-id".to_string() }),
    ];
    for (label, identity) in cases {
        use_identity_for_test(Some(Ok(identity)));
        let note: Option<String> = survivor_note(&journal);
        use_identity_for_test(None);
        assert!(note.is_some_and(|text| { return text.contains("still running"); }), "{label}: cleared a live process group");
    }
    let _ = group.kill();
    let _ = group.wait();
}

/// The files the current identity is read from must have the format of an id (9a D2): each malformed file is an error.
#[test]
fn the_current_identity_is_read_strictly_from_its_files() {
    let directory = scratch();
    let machine: PathBuf = directory.path().join("machine-id");
    let real_boot: &Path = Path::new(crate::control_group::BOOT_ID_PATH);
    for (label, text) in [("empty", ""), ("too short", "abcdef\n"), ("upper case", "ABCDEF0123456789ABCDEF0123456789\n"), ("a UUID with hyphens", "0123456789abcdef-0123456789abcdef\n"), ("all zero", "00000000000000000000000000000000\n")] {
        fs::write(&machine, text).expect("write");
        assert!(read_current_identity(real_boot, &machine).is_err(), "{label}: accepted as a machine id");
    }
    fs::write(&machine, "0123456789abcdef0123456789abcdef\n").expect("write");
    let read: Identity = read_current_identity(real_boot, &machine).expect("a well-formed machine id and this host's boot id");
    assert_eq!(read.machine, "0123456789abcdef0123456789abcdef");
}
