//! Controls for the judgement of a published transaction directory without an owner record.

use super::*;
use crate::process_use::ProcessUser;
use crate::test_support::{Sleeper, fixture, remove};
use std::time::Duration;

/// A scan that finds nobody.
fn nobody(_directory: &Path) -> UseScan {
    return UseScan::default();
}

/// A scan that finds one process.
fn one_user(_directory: &Path) -> UseScan {
    return UseScan { users: vec![ProcessUser { pid: 4242, command: Some(String::from("hook")) }], unreadable: Vec::new() };
}

/// Date a path.
fn date(path: &Path, when: SystemTime) {
    std::fs::File::open(path).expect("open").set_modified(when).expect("date");
}

/// A transaction directory whose every entry is dated `when`: a record, a nested record and the directories.
fn dated_directory(name: &str, when: SystemTime) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let directory: PathBuf = root.join("0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10");
    std::fs::create_dir_all(directory.join("hooks")).expect("hooks");
    std::fs::write(directory.join("preparing.json"), b"{}").expect("record");
    std::fs::write(directory.join("hooks/plan.json"), b"{}").expect("nested record");
    for path in [directory.join("hooks/plan.json"), directory.join("preparing.json"), directory.join("hooks"), directory.clone()] {
        date(path.as_path(), when);
    }
    return (root, directory);
}

/// The instant the fixtures date their entries.
fn then() -> SystemTime {
    return SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
}

/// A directory that changed within the patience is not abandoned, and the reason says how long ago.
#[test]
fn a_directory_that_changed_within_the_patience_is_unproven() {
    let (root, directory) = dated_directory("ownerless-fresh", then());
    let now: SystemTime = then() + Duration::from_secs(600);
    let standing: OwnerlessStanding = ownerless_standing_with(directory.as_path(), now, nobody).expect("judged");
    assert_eq!(standing, OwnerlessStanding::Unproven(String::from("Its newest entry changed 600 seconds ago")));
    remove(root.as_path());
}

/// Exactly the patience still counts as in use, and one second more is abandoned: the boundary of the shared rule.
#[test]
fn the_patience_boundary_is_the_shared_one() {
    let (root, directory) = dated_directory("ownerless-boundary", then());
    let edge: SystemTime = then() + FOREIGN_OWNER_PATIENCE;
    assert!(matches!(
        ownerless_standing_with(directory.as_path(), edge, nobody).expect("at the edge"),
        OwnerlessStanding::Unproven(_)
    ));
    let past: SystemTime = edge + Duration::from_secs(1);
    assert_eq!(ownerless_standing_with(directory.as_path(), past, nobody).expect("past"), OwnerlessStanding::Abandoned);
    remove(root.as_path());
}

/// A nested record written later than everything else keeps the directory in use: the age is the newest entry's.
#[test]
fn the_newest_nested_entry_decides_the_age() {
    let (root, directory) = dated_directory("ownerless-nested", then());
    let later: SystemTime = then() + Duration::from_secs(2_000);
    date(directory.join("hooks/plan.json").as_path(), later);
    let now: SystemTime = then() + FOREIGN_OWNER_PATIENCE + Duration::from_secs(60);
    let standing: OwnerlessStanding = ownerless_standing_with(directory.as_path(), now, nobody).expect("judged");
    assert!(matches!(&standing, OwnerlessStanding::Unproven(reason) if reason.contains("seconds ago")), "{standing:?}");
    remove(root.as_path());
}

/// An old directory that a process still uses is not abandoned, and the reason names the process.
#[test]
fn a_directory_in_use_is_unproven_even_when_old() {
    let (root, directory) = dated_directory("ownerless-used", then());
    let now: SystemTime = then() + FOREIGN_OWNER_PATIENCE + Duration::from_secs(60);
    let standing: OwnerlessStanding = ownerless_standing_with(directory.as_path(), now, one_user).expect("judged");
    assert_eq!(standing, OwnerlessStanding::Unproven(String::from("PID 4242 (hook) has a file or working directory in it")));
    remove(root.as_path());
}

/// An entry dated in the future cannot be aged, so the directory is not abandoned.
#[test]
fn a_future_dated_entry_is_unproven() {
    let (root, directory) = dated_directory("ownerless-future", then() + Duration::from_secs(10_000));
    let standing: OwnerlessStanding = ownerless_standing_with(directory.as_path(), then(), nobody).expect("judged");
    assert_eq!(standing, OwnerlessStanding::Unproven(String::from("Its newest entry is dated in the future")));
    remove(root.as_path());
}

/// The production judgement sees a real process whose working directory is inside the directory.
#[cfg(target_os = "linux")]
#[test]
fn a_live_process_inside_an_old_directory_keeps_it() {
    let (root, directory) = dated_directory("ownerless-live", SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000));
    let holder: Sleeper = Sleeper::start(directory.join("hooks").as_path(), None);
    let standing: OwnerlessStanding = ownerless_standing(directory.as_path()).expect("judged");
    assert!(
        matches!(&standing, OwnerlessStanding::Unproven(reason) if reason.contains(format!("PID {} (sleep)", holder.pid()).as_str())),
        "{standing:?}"
    );
    drop(holder);
    assert_eq!(ownerless_standing(directory.as_path()).expect("judged again"), OwnerlessStanding::Abandoned);
    remove(root.as_path());
}

/// The refusal keeps its opening words, names the directory twice, and says when cli-git acts, what to confirm and the
/// one case in which removal by hand loses work.
#[test]
fn the_refusal_names_the_directory_the_automatic_condition_and_the_safe_removal() {
    let directory: &Path = Path::new("/repo/.git/cli-git-transactions/0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10");
    let message: String = ownerless_refusal(directory, "Its newest entry changed 12 seconds ago").0;
    assert!(message.starts_with(format!("Transaction owner record is missing: {}.", directory.display()).as_str()), "{message}");
    assert!(message.contains(format!("rm -r {}", directory.display()).as_str()), "{message}");
    assert!(message.contains("Its newest entry changed 12 seconds ago."), "{message}");
    assert!(message.contains("60 minutes"), "{message}");
    assert!(message.contains("no cli-git or git command is running"), "{message}");
    assert!(message.contains("landing-<n>.json"), "{message}");
    assert!(!message.contains('\u{2014}') && !message.contains('\u{2013}'), "{message}");
}
