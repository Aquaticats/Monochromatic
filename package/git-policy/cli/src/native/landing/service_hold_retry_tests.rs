//! Controls for running a forwarded index writer again when the commit service took the index before its Git did.

use super::*;
use crate::service_index_lock::{LOCK_MARKER, lock_path_of};
use crate::test_support::{fixture, remove, scratch_environment};
use std::path::PathBuf;
use std::time::Duration;

fn words(texts: &[&str]) -> Vec<OsString> {
    return texts.iter().map(OsString::from).collect();
}

/// Only the few commands that take the index lock first and read nothing from the person run again,
/// and only with options
///  that are known to leave that so.
#[test]
fn only_known_commands_with_known_options_may_run_again() {
    for (word, after) in [
        ("add", vec!["--", "a.txt"]),
        ("add", vec!["-A"]),
        ("add", vec!["--all", "--force", "--", "-odd-name"]),
        ("rm", vec!["--cached", "-r", "dir"]),
        ("mv", vec!["-k", "a", "b"]),
    ] {
        assert!(may_run_again(word.as_bytes(), &words(&after)), "{word} {after:?}");
    }
    for (word, after) in [
        ("add", vec!["-p"]),
        ("add", vec!["-i"]),
        ("add", vec!["--interactive"]),
        ("add", vec!["--edit"]),
        ("add", vec!["--pathspec-from-file=-"]),
        ("add", vec!["--pathspec-from-file", "-"]),
        ("add", vec!["--chmod=+x", "a.txt"]),
        ("add", vec!["-Av"]),
        ("rm", vec!["--force-something"]),
        ("update-index", vec!["--stdin"]),
        ("checkout", vec!["-b", "x"]),
        ("rebase", vec![]),
        ("stash", vec![]),
        ("commit", vec![]),
    ] {
        assert!(!may_run_again(word.as_bytes(), &words(&after)), "{word} {after:?}");
    }
}

/// A fake Git in `root`:
/// it counts its starts in `count`,
/// and from start number `succeeds_from` on it succeeds,
/// before that
///  it dies with `refusal` on standard error and status 128.
fn fake_git(root: &Path, succeeds_from: usize, refusal: &str) -> (PathBuf, PathBuf) {
    let count: PathBuf = root.join("starts");
    let program: PathBuf = root.join("fake-git");
    std::fs::write(
        &program,
        format!(
            "#!/bin/sh\nn=$(cat '{0}' 2>/dev/null || echo 0)\nn=$((n+1))\necho $n > '{0}'\nif [ $n -ge {1} ]; then echo 'done' >&2; exit 0; fi\necho '{2}' >&2\nexit 128\n",
            count.display(),
            succeeds_from,
            refusal
        ),
    )
    .expect("fake git");
    std::fs::set_permissions(&program, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("executable");
    return (program, count);
}

fn starts(count: &Path) -> usize {
    return std::fs::read_to_string(count).map_or(0, |text| { return text.trim().parse().expect("a number"); });
}

/// The refusal names the lock the way Git does.
const REFUSAL: &str = "fatal: Unable to create '/repo/.git/index.lock': File exists.";

/// Lost to the service once:
/// the command waits for the hold to end,
/// runs again,
/// succeeds,
/// and the first refusal is never shown.
#[test]
fn a_command_that_lost_the_index_to_the_service_runs_again_and_hides_the_lost_attempt() {
    let root: PathBuf = fixture("hold-retry-lost");
    let index: PathBuf = root.join("index");
    let lock: PathBuf = lock_path_of(index.as_path());
    std::fs::write(&lock, format!("{LOCK_MARKER}1\n")).expect("the service's hold");
    let (program, count) = fake_git(root.as_path(), 2, REFUSAL);
    let released: PathBuf = lock.clone();
    let releaser = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(150));
        std::fs::remove_file(released).expect("release");
    });
    let mut shown: Vec<u8> = Vec::new();
    let ran: ForwardedRun = run_again_while_service_holds(program.as_path(), &words(&["add", "a.txt"]), &[], scratch_environment().as_slice(), index.as_path(), "add", &mut shown);
    releaser.join().expect("releaser");
    assert_eq!(ran, ForwardedRun::Ran(0));
    assert_eq!(starts(count.as_path()), 2);
    let text: String = String::from_utf8_lossy(&shown).into_owned();
    assert!(text.contains("done") && !text.contains("index.lock"), "{text}");
    remove(root.as_path());
}

/// A hold that never ends,
/// with a command that is refused every time:
/// the series of reruns ends at the ceiling with Git's words
/// shown and its status,
/// where the count of fifty starts would have kept it for as long as every wait lasts.
#[test]
fn a_hold_that_never_ends_stops_the_reruns_at_the_ceiling() {
    let root: PathBuf = fixture("hold-retry-ceiling");
    let index: PathBuf = root.join("index");
    let lock: PathBuf = lock_path_of(index.as_path());
    let identity: String = crate::service_index_marker::own_identity().expect("identity");
    std::fs::write(&lock, format!("{LOCK_MARKER}1 {} {identity}\n", std::process::id())).expect("a hold of a service that runs");
    let (program, count) = fake_git(root.as_path(), usize::MAX, REFUSAL);
    let (finished, ended) = std::sync::mpsc::channel();
    let environment = scratch_environment();
    let index_path: PathBuf = index.clone();
    std::thread::spawn(move || {
        let mut shown: Vec<u8> = Vec::new();
        let ran: ForwardedRun = run_again_within(program.as_path(), &words(&["add"]), &[], environment.as_slice(), index_path.as_path(), "add", Duration::from_millis(400), &mut shown);
        let _sent = finished.send((ran, String::from_utf8_lossy(&shown).into_owned()));
    });
    let (ran, text) = ended.recv_timeout(Duration::from_secs(20)).expect("the reruns ended at their ceiling");
    assert_eq!(ran, ForwardedRun::Ran(128));
    assert!(text.contains("index.lock"), "Git's own words are shown at the end: {text}");
    assert!(starts(count.as_path()) < ATTEMPTS, "the ceiling, not the count, ended it");
    remove(root.as_path());
}

/// A lock that is a foreign program's is native Git's business:
/// one start,
/// and Git's words shown.
#[test]
fn a_foreign_hold_is_not_waited_for_and_the_refusal_is_shown() {
    let root: PathBuf = fixture("hold-retry-foreign");
    let index: PathBuf = root.join("index");
    std::fs::write(lock_path_of(index.as_path()), b"someone else\n").expect("a foreign hold");
    let (program, count) = fake_git(root.as_path(), usize::MAX, REFUSAL);
    let mut shown: Vec<u8> = Vec::new();
    let ran: ForwardedRun = run_again_while_service_holds(program.as_path(), &words(&["add"]), &[], scratch_environment().as_slice(), index.as_path(), "add", &mut shown);
    assert_eq!(ran, ForwardedRun::Ran(128));
    assert_eq!(starts(count.as_path()), 1);
    assert!(String::from_utf8_lossy(&shown).contains("index.lock"), "{}", String::from_utf8_lossy(&shown));
    remove(root.as_path());
}

/// A status 128 that is not the lock's refusal is Git's own answer:
/// one start,
/// and the words shown.
#[test]
fn another_failure_with_the_same_status_is_not_run_again() {
    let root: PathBuf = fixture("hold-retry-other");
    let index: PathBuf = root.join("index");
    let (program, count) = fake_git(root.as_path(), usize::MAX, "fatal: pathspec 'x' did not match any files");
    let mut shown: Vec<u8> = Vec::new();
    let ran: ForwardedRun = run_again_while_service_holds(program.as_path(), &words(&["add", "x"]), &[], scratch_environment().as_slice(), index.as_path(), "add", &mut shown);
    assert_eq!(ran, ForwardedRun::Ran(128));
    assert_eq!(starts(count.as_path()), 1);
    assert!(String::from_utf8_lossy(&shown).contains("did not match"), "{}", String::from_utf8_lossy(&shown));
    remove(root.as_path());
}

/// A service that keeps taking the index first ends the retries at the bound,
/// and the last refusal is shown once.
#[test]
fn the_retries_end_at_the_bound_and_show_the_last_refusal_once() {
    let root: PathBuf = fixture("hold-retry-bound");
    let index: PathBuf = root.join("index");
    let (program, count) = fake_git(root.as_path(), usize::MAX, REFUSAL);
    let mut shown: Vec<u8> = Vec::new();
    let ran: ForwardedRun = run_again_while_service_holds(program.as_path(), &words(&["add"]), &[], scratch_environment().as_slice(), index.as_path(), "add", &mut shown);
    assert_eq!(ran, ForwardedRun::Ran(128));
    assert_eq!(starts(count.as_path()), ATTEMPTS);
    assert_eq!(String::from_utf8_lossy(&shown).matches("index.lock").count(), 1, "{}", String::from_utf8_lossy(&shown));
    remove(root.as_path());
}
