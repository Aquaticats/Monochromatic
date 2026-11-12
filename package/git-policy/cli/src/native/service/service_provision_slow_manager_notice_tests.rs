//! Controls for the line a commit prints when the service manager has not answered its provisioning within a moment:
//! that it is said once when the calls outlast the moment,
//! that a command whose manager answered at once prints nothing,
//! and that the sentence names the unit whose service is being set up and the seconds it waited.

use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// What:
///  The sentence a wait for the manager is explained with,
/// with the unit and the seconds it waited in it.
/// Why:
///   The person can ask the manager about that very unit,
/// and the seconds are what tells a wait that has only begun from one that is
///  stuck.
#[test]
fn the_sentence_names_the_unit_the_seconds_and_that_the_commit_goes_on() {
    let said: String = slow_manager_notice("monochromatic-git-policy-cli@0123456789abcdef0123456789abcdef.socket", SLOW_MANAGER_NOTICE_AFTER);
    assert!(said.contains("monochromatic-git-policy-cli@0123456789abcdef0123456789abcdef.socket"), "{said}");
    assert!(said.contains(&format!("{} seconds", SLOW_MANAGER_NOTICE_AFTER.as_secs())), "{said}");
    assert!(said.contains("this commit goes on"), "the commit is not lost by the wait: {said}");
    assert!(said.starts_with("monochromatic-git-policy-cli: "), "the line names the program it is about: {said}");
}

/// What:
///  A manager that answers at once says nothing,
/// and one that keeps the command waiting says the line once.
/// Why:
///   The guard ends the thread as soon as the calls are over,
/// so a healthy commit prints nothing however many times it provisions,
/// and a slow one
///  says it once and not at every second of the wait.
#[test]
fn the_line_is_said_once_and_only_when_the_calls_outlast_the_moment() {
    let quiet: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let counted: Arc<AtomicUsize> = Arc::clone(&quiet);
    drop(SlowManagerNotice::arm_saying(move || {
        let _counted = counted.fetch_add(1, Ordering::SeqCst);
    }));
    assert_eq!(quiet.load(Ordering::SeqCst), 0, "a manager that answered at once says nothing");
    let slow: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let counted: Arc<AtomicUsize> = Arc::clone(&slow);
    let notice: SlowManagerNotice = SlowManagerNotice::arm_saying(move || {
        let _counted = counted.fetch_add(1, Ordering::SeqCst);
    });
    std::thread::sleep(SLOW_MANAGER_NOTICE_AFTER + std::time::Duration::from_millis(250));
    assert_eq!(slow.load(Ordering::SeqCst), 1, "a manager that keeps the calls waiting is said once");
    drop(notice);
    std::thread::sleep(std::time::Duration::from_millis(50));
    assert_eq!(slow.load(Ordering::SeqCst), 1, "and the thread it was said from is ended, so nothing says it again");
}
