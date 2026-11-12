//! Controls for the mark of an install running in one worktree.

use super::*;
use crate::test_support::{fixture, remove};

/// An owner record for a process that runs,
/// which is this test.
fn live(transaction_id: &str) -> TransactionOwner {
    return crate::transaction_owner::current_transaction_owner(transaction_id, "2026-11-12T00:00:00.000Z").expect("an owner record");
}

/// What:
///  An owner record naming a process that ended,
///  so the mark it left is one the next command may clear.
/// Why:
///   The liveness rule reads the process table,
/// so a control needs a number no process runs under.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = await spawn('true'); await gone.exited; return { ownerPid: gone.pid, ownerIdentity: '0', ... };
/// ```
fn gone(transaction_id: &str) -> TransactionOwner {
    let mut ended = std::process::Command::new("/bin/sh").arg("-c").arg("exit 0").spawn().expect("a process that ends");
    let pid: i64 = i64::from(ended.id());
    ended.wait().expect("it ends");
    return TransactionOwner {
        transaction_id: String::from(transaction_id),
        owner_pid: pid,
        owner_identity: String::from("0"),
        created_at: String::from("2026-11-12T00:00:00.000Z"),
        owner_pid_namespace: None,
    };
}

/// The Git directory of a fixture:
/// the mark lives in a directory the wrapper makes beside the records.
fn git_dir(where_: &std::path::Path) -> std::path::PathBuf {
    std::fs::create_dir_all(where_.join(".git")).expect("a Git directory");
    return where_.join(".git");
}

/// What:
///  A command takes the mark,
///  and a second take names the process that holds it.
/// Why:
///   Two merges that arrive together must not both install:
/// the mark is one file,
/// taken with `O_EXCL`,
///  which one `open` decides.
#[test]
fn a_take_names_this_command_and_a_second_take_names_the_holder() {
    let root: std::path::PathBuf = fixture("merge-market-take");
    let git_dir: std::path::PathBuf = git_dir(root.as_path());
    let mine: TransactionOwner = live("11111111-0000-4000-8000-000000000001");
    let held: InstallGuard = match take_install_guard(git_dir.as_path(), &mine).expect("a take") {
        TakeOutcome::Took(found) => found,
        TakeOutcome::HeldBy(pid) => panic!("nothing held the worktree, and {pid} was named"),
    };
    assert!(held.path().exists());
    match take_install_guard(git_dir.as_path(), &live("22222222-0000-4000-8000-000000000002")).expect("a second take") {
        TakeOutcome::Took(_) => panic!("a second command took a mark another live merge holds"),
        TakeOutcome::HeldBy(pid) => assert_eq!(pid, i64::from(std::process::id())),
    }
    drop(held);
    assert!(!install_guard_path(git_dir.as_path()).exists(), "the mark goes when the command that took it ends");
    assert!(matches!(take_install_guard(git_dir.as_path(), &mine).expect("a take after the first went"), TakeOutcome::Took(_)));
    remove(root.as_path());
}

/// What:
///  A mark a killed merge left is taken,
///  because nothing runs behind it.
/// Why:
///   The mark's owner is the same owner record a merge's intent record carries,
/// so recovery's own liveness rule decides it;
///  a person is never asked to remove it.
#[test]
fn a_mark_a_dead_merge_left_is_taken_and_holds_nothing() {
    let root: std::path::PathBuf = fixture("merge-market-dead");
    let git_dir: std::path::PathBuf = git_dir(root.as_path());
    let path: std::path::PathBuf = install_guard_path(git_dir.as_path());
    std::fs::create_dir_all(path.parent().expect("a directory")).expect("the records directory");
    std::fs::write(path.as_path(), encode_transaction_owner(&gone("33333333-0000-4000-8000-000000000003"))).expect("a dead merge's mark");
    assert!(a_dead_install_guard(git_dir.as_path()).expect("the question"), "the mark of a dead merge is one recovery must look at");
    match take_install_guard(git_dir.as_path(), &live("44444444-0000-4000-8000-000000000004")).expect("a take") {
        TakeOutcome::Took(held) => assert_eq!(held.path(), path.as_path()),
        TakeOutcome::HeldBy(pid) => panic!("a dead merge's mark was read as live ({pid})"),
    }
    remove(root.as_path());
}

/// What:
///  A mark whose bytes are not an owner record refuses,
///  and stays where it is.
/// Why:
///   Guessing at it could let this command install beside a merge that still runs;
/// the words name the file,
///  and a person can remove it.
#[test]
fn a_malformed_mark_refuses_and_is_left_in_place() {
    let root: std::path::PathBuf = fixture("merge-market-malformed");
    let git_dir: std::path::PathBuf = git_dir(root.as_path());
    let path: std::path::PathBuf = install_guard_path(git_dir.as_path());
    std::fs::create_dir_all(path.parent().expect("a directory")).expect("the records directory");
    std::fs::write(path.as_path(), b"not a record\n").expect("a torn mark");
    let refused: RecoveryError = match take_install_guard(git_dir.as_path(), &live("55555555-0000-4000-8000-000000000005")) {
        Err(error) => error,
        Ok(_) => panic!("a torn mark was taken"),
    };
    assert!(refused.0.contains("malformed"), "{}", refused.0);
    assert!(refused.0.contains(path.to_string_lossy().as_ref()), "{}", refused.0);
    assert!(path.exists(), "the torn mark is kept for a person to look at");
    assert!(a_dead_install_guard(git_dir.as_path()).is_err(), "the reader that decides whether recovery is needed refuses as well");
    remove(root.as_path());
}

/// What:
///  A wait with no merge installing is one look and no notice.
/// Why:
///   Every command that is not contending pays this,
/// so it must cost no line and no pause.
#[test]
fn the_wait_returns_at_once_when_no_merge_installs() {
    let root: std::path::PathBuf = fixture("merge-market-free");
    let git_dir: std::path::PathBuf = git_dir(root.as_path());
    let mut said: Vec<u8> = Vec::new();
    let began: std::time::Instant = std::time::Instant::now();
    let waited: GuardWait = wait_within(git_dir.as_path(), std::time::Duration::from_secs(30), Cadence::standard(), &mut said).expect("the wait");
    assert_eq!(waited, GuardWait::Free);
    assert!(said.is_empty(), "nothing is said when there is nothing to wait for");
    assert!(began.elapsed() < std::time::Duration::from_secs(5), "a free worktree is one look");
    remove(root.as_path());
}

/// What:
///  A wait over a live merge says what it waits for and who holds it,
///  and ends at its ceiling.
/// Why:
///   The owner's rule for every wait is one line naming the blocker,
/// and the command must not hang on a merge that stopped for good.
#[test]
fn the_wait_names_the_holder_and_ends_at_its_ceiling() {
    let root: std::path::PathBuf = fixture("merge-market-ceiling");
    let git_dir: std::path::PathBuf = git_dir(root.as_path());
    let held: InstallGuard = match take_install_guard(git_dir.as_path(), &live("66666666-0000-4000-8000-000000000006")).expect("a take") {
        TakeOutcome::Took(found) => found,
        TakeOutcome::HeldBy(pid) => panic!("nothing held the worktree, and {pid} was named"),
    };
    let mut said: Vec<u8> = Vec::new();
    let waited: GuardWait = wait_within(
        git_dir.as_path(),
        std::time::Duration::from_millis(400),
        Cadence::from_interval(std::time::Duration::from_millis(50)),
        &mut said,
    )
    .expect("the wait");
    assert!(matches!(waited, GuardWait::StillHeld(_)), "{waited:?}");
    let text: String = String::from_utf8_lossy(said.as_slice()).into_owned();
    assert!(text.contains("still waiting for the merge that"), "{text}");
    assert!(text.contains(format!("process {}", std::process::id()).as_str()), "the line names the holder: {text}");
    assert!(text.contains("one merge installs in a worktree at a time"), "the line says what it waits for: {text}");
    assert!(text.contains("saves itself on a ref of its own"), "the line says what becomes of this merge: {text}");
    assert!(held.path().exists(), "the holder's mark is not a waiter's to remove");
    drop(held);
    remove(root.as_path());
}

/// What:
///  A wait ends as soon as the mark goes,
///  which is what a merge that installed leaves.
/// Why:
///   The wait is what a second merge makes instead of installing,
/// so it must end on the release and not at a pause or the ceiling.
#[test]
fn the_wait_returns_as_soon_as_the_mark_goes() {
    let root: std::path::PathBuf = fixture("merge-market-released");
    let git_dir: std::path::PathBuf = git_dir(root.as_path());
    let path: std::path::PathBuf = install_guard_path(git_dir.as_path());
    let held: InstallGuard = match take_install_guard(git_dir.as_path(), &live("77777777-0000-4000-8000-000000000007")).expect("a take") {
        TakeOutcome::Took(found) => found,
        TakeOutcome::HeldBy(pid) => panic!("nothing held the worktree, and {pid} was named"),
    };
    let releaser: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(100));
        std::fs::remove_file(path.as_path()).expect("the holder's release");
    });
    let mut said: Vec<u8> = Vec::new();
    let waited: GuardWait = wait_within(git_dir.as_path(), std::time::Duration::from_secs(30), Cadence::standard(), &mut said).expect("the wait");
    releaser.join().expect("the release");
    assert!(matches!(waited, GuardWait::Released(_)), "the wait ends on the release, not at the ceiling: {waited:?}");
    drop(held);
    remove(root.as_path());
}
