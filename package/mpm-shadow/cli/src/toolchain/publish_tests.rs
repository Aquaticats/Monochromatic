//! Unit tests of the scope lock and the publish that waits on it.

/// Imports `fs` from `std`.
use std::fs;
/// Imports `Duration` from `std::time`.
use std::time::Duration;

/// Imports `Rig` from `super::fakes`.
use super::fakes::Rig;
/// Imports what these tests use from `super::publish`.
use super::publish::{GUARD_WAIT, publish_lock, read_revisions};

/// What:
/// A publish whose scope lock another command holds answers within the stated bound,
/// says which file was held and how long it waited,
/// and writes nothing.
/// Why:
/// The review of 2026-11-14 found that `install`'s lock preparation publishes through `publish_lock`,
/// which took the scope lock with a blocking `File::lock`,
/// so a holder that was stopped or hung stopped every installing command in that checkout for ever.
#[test]
fn a_publish_waits_for_the_scope_lock_no_longer_than_it_says_and_writes_nothing() {
    let rig = Rig::new();
    let expected = read_revisions(&rig.scope).expect("read the revisions");
    let before = expected.clone();
    // Another command of this checkout holds the scope's lock, and it does not let go.
    fs::create_dir_all(rig.scope.guard_file.parent().expect("the guard file has a directory")).expect("make the guard file's directory");
    let held: fs::File = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&rig.scope.guard_file)
        .expect("open the guard file");
    held.lock().expect("hold the scope lock");
    let guard_file = rig.scope.guard_file.clone();
    let scope = rig.scope.clone();
    let (sender, receiver) = std::sync::mpsc::channel::<Result<(), String>>();
    std::thread::spawn(move || {
        let outcome: Result<(), String> = publish_lock(&scope, &before, "{\"schemaVersion\": 1, \"tools\": {}}\n")
            .map_err(|error| return error.message);
        let _sent = sender.send(outcome);
    });

    // The wait has a bound, so the publish answers instead of waiting for the holder for ever.
    let waited: Duration = GUARD_WAIT + Duration::from_secs(15);
    let Ok(outcome) = receiver.recv_timeout(waited) else {
        panic!(
            "the publish was still waiting on {} {waited:?} after it started, so the wait on the scope lock has no bound",
            guard_file.display()
        );
    };
    let Err(message) = outcome else {
        panic!("the publish took a scope lock another command held");
    };

    assert!(message.contains(&guard_file.display().to_string()), "the sentence names the file it waited on: {message}");
    assert!(
        message.contains(&format!("within {} s", GUARD_WAIT.as_secs())),
        "the sentence names the bound it waited: {message}"
    );
    assert!(message.contains("nothing was written and nothing was changed"), "the sentence says what it did not do: {message}");
    drop(held);
    assert_eq!(read_revisions(&rig.scope).expect("read the revisions again"), expected, "the lock was not written");
}
