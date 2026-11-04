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
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use crate::cgroup_test_support::{delegated_route, start_in_scope};
use crate::control_group::{ControlGroup, Route, View, view_of};
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

/// The view of a path is not used here; this keeps the import of `View` honest for the tests added next.
#[allow(dead_code)]
fn unused_view(_view: View) {}
