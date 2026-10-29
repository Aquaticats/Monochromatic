//! Settlement-lock checks: owner records, liveness, dead-owner retirement, unproven owners and inherited leases.

use super::*;
use crate::test_support::{fixture, remove};
use crate::worktree_copy_journal::ensure_journal_root;
use crate::wait_progress::Cadence;
use std::time::Duration;

/// The published lock directory of a common directory.
fn lock_of(common: &Path) -> PathBuf {
    return ensure_journal_root(common)
        .expect("root")
        .join(SETTLEMENT_LOCK_NAME);
}

/// Writes an owner record into a lock directory, as a published holder would.
fn publish(lock: &Path, text: &str) {
    std::fs::create_dir_all(lock).expect("lock directory");
    crate::private_storage::write_private_file(
        lock.join(OWNER_RECORD_NAME).as_path(),
        text.as_bytes(),
    )
    .expect("owner record");
}

/// A process identity that no live process has.
const DEAD_OWNER: &str = "{\"leaseToken\":\"dead-token\",\"ownerBirthIdentity\":\"no-such-birth\",\"ownerPid\":2147483646,\"schemaVersion\":1}\n";

/// The owner record encodes with the incumbent's key order and parses back.
#[test]
fn owner_record_round_trips_with_incumbent_spelling() {
    let owner: LockOwner = LockOwner {
        lease_token: String::from("token"),
        owner_pid: 5,
        owner_birth_identity: String::from("linux:9"),
        owner_pid_namespace: None,
    };
    let text: String = encode_owner(&owner);
    assert_eq!(
        text,
        "{\"leaseToken\":\"token\",\"ownerBirthIdentity\":\"linux:9\",\"ownerPid\":5,\"schemaVersion\":1}\n"
    );
    assert_eq!(parse_owner(text.as_str()), Ok(owner));
}

/// A record with missing or mistyped fields proves nothing, and the evidence says why.
#[test]
fn invalid_owner_records_are_unproven() {
    assert_eq!(
        parse_owner("{\"schemaVersion\":2}"),
        Err(String::from("missing or mistyped owner fields"))
    );
    assert_eq!(
        parse_owner(
            "{\"schemaVersion\":1,\"leaseToken\":\"\",\"ownerPid\":1,\"ownerBirthIdentity\":\"x\"}"
        ),
        Err(String::from("missing or mistyped owner fields"))
    );
    assert_eq!(
        parse_owner(
            "{\"schemaVersion\":1,\"leaseToken\":\"t\",\"ownerPid\":0,\"ownerBirthIdentity\":\"x\"}"
        ),
        Err(String::from("missing or mistyped owner fields"))
    );
    assert!(
        parse_owner("not json")
            .expect_err("not json")
            .starts_with("not JSON (")
    );
}

/// Each way the record read can fail has its own outcome:
///  a vanished or replaced lock asks for another attempt,
///  an unreadable record is unproven and names why,
///  and any other failure is an error.
#[cfg(unix)]
#[test]
fn each_failed_record_read_has_its_own_outcome() {
    use std::os::unix::fs::PermissionsExt;
    let common: PathBuf = fixture("lock-record-read-failures");
    let lock: PathBuf = lock_of(common.as_path());
    assert_eq!(read_published(lock.as_path()).expect("vanished"), Published::Retry, "the lock vanished");
    std::fs::write(&lock, b"x").expect("a file in place of the lock");
    assert_eq!(
        read_published(lock.as_path()).expect("replaced"),
        Published::Retry,
        "the lock was replaced by a file"
    );
    std::fs::remove_file(&lock).expect("remove the file");
    let record: PathBuf = lock.join(OWNER_RECORD_NAME);
    std::fs::create_dir_all(&record).expect("a directory in place of the record");
    match read_published(lock.as_path()).expect("a directory is evidence") {
        Published::Unproven(evidence) => assert!(evidence.ends_with("is unreadable (EISDIR)"), "{evidence}"),
        other => panic!("expected unproven evidence, got {other:?}"),
    }
    std::fs::remove_dir(&record).expect("remove the directory");
    std::os::unix::fs::symlink(OWNER_RECORD_NAME, &record).expect("a record that links to itself");
    assert!(read_published(lock.as_path()).is_err(), "a link loop is not evidence");
    std::fs::remove_file(&record).expect("remove the link");
    std::fs::write(&record, b"x").expect("a record");
    std::fs::set_permissions(&record, std::fs::Permissions::from_mode(0o000)).expect("close the record");
    // A superuser reads the record anyway, so only an ordinary user shows the refusal.
    if std::fs::read(&record).is_err() {
        match read_published(lock.as_path()).expect("an unreadable record is evidence") {
            Published::Unproven(evidence) => assert!(evidence.ends_with("is unreadable (EACCES)"), "{evidence}"),
            other => panic!("expected unproven evidence, got {other:?}"),
        }
    }
    std::fs::set_permissions(&record, std::fs::Permissions::from_mode(0o600)).expect("open the record");
    remove(common.as_path());
}

/// A free lock is acquired, published with a private owner record that names this process, and released by removing it.
#[cfg(unix)]
#[test]
fn free_lock_is_acquired_and_released() {
    let root: PathBuf = fixture("lock-free");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let lease: WorktreeCopyLease = try_acquire_worktree_copy_lock(common.as_path())
        .expect("attempt")
        .expect("free");
    let lock: PathBuf = lock_of(common.as_path());
    let published: String = std::fs::read_to_string(lock.join(OWNER_RECORD_NAME)).expect("record");
    let owner: LockOwner = parse_owner(published.as_str()).expect("valid owner");
    assert_eq!(owner.owner_pid, i64::from(std::process::id()));
    assert_eq!(owner.lease_token, lease.lease_token());
    lease.release().expect("released");
    assert!(!lock.exists());
    remove(root.as_path());
}

/// A lock held by a live owner is reported as held, and the holder keeps it.
#[cfg(unix)]
#[test]
fn live_holder_keeps_the_lock() {
    let root: PathBuf = fixture("lock-live");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let first: WorktreeCopyLease = try_acquire_worktree_copy_lock(common.as_path())
        .expect("attempt")
        .expect("free");
    assert!(
        try_acquire_worktree_copy_lock(common.as_path())
            .expect("attempt")
            .is_none()
    );
    first.release().expect("released");
    remove(root.as_path());
}

/// A lock whose owner is dead is retired and taken, without waiting.
#[cfg(unix)]
#[test]
fn dead_owner_lock_is_retired_and_taken() {
    let root: PathBuf = fixture("lock-dead");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    publish(lock_of(common.as_path()).as_path(), DEAD_OWNER);
    let lease: WorktreeCopyLease = try_acquire_worktree_copy_lock(common.as_path())
        .expect("attempt")
        .expect("taken from the dead owner");
    assert_ne!(lease.lease_token(), "dead-token");
    lease.release().expect("released");
    remove(root.as_path());
}

/// An owner record that proves nothing keeps the lock held for `try`, and the lock stays in place.
#[cfg(unix)]
#[test]
fn unproven_owner_keeps_the_lock_in_place() {
    let root: PathBuf = fixture("lock-unproven");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let lock: PathBuf = lock_of(common.as_path());
    publish(lock.as_path(), "garbage");
    assert!(
        try_acquire_worktree_copy_lock(common.as_path())
            .expect("attempt")
            .is_none()
    );
    assert!(lock.join(OWNER_RECORD_NAME).exists());
    remove(root.as_path());
}

/// A blocking acquisition gives up on an unproven owner after the bounded wait, names the evidence, and leaves the lock.
#[cfg(unix)]
#[test]
fn unproven_owner_fails_the_blocking_wait_and_keeps_lock() {
    let root: PathBuf = fixture("lock-unproven-wait");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let lock: PathBuf = lock_of(common.as_path());
    publish(lock.as_path(), "garbage");
    let mut stderr: Vec<u8> = Vec::new();
    let started: Instant = Instant::now();
    let failure: WorktreeCopyError =
        acquire_worktree_copy_lock(common.as_path(), &mut stderr).expect_err("timeout");
    assert!(
        started.elapsed() >= Duration::from_millis(UNPROVEN_OWNER_TIMEOUT_MS),
        "the refusal came after {:?}",
        started.elapsed()
    );
    assert!(
        failure.0.contains("stayed in place for 1000 ms"),
        "{}",
        failure.0
    );
    assert!(
        failure.0.contains("holds an invalid owner record"),
        "{}",
        failure.0
    );
    assert!(lock.join(OWNER_RECORD_NAME).exists());
    assert!(stderr.is_empty());
    remove(root.as_path());
}

/// A release after another owner replaced the lock fails with the ownership-changed wording and keeps the replacement.
#[cfg(unix)]
#[test]
fn replaced_lock_release_fails_and_keeps_replacement() {
    let root: PathBuf = fixture("lock-replaced");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let lease: WorktreeCopyLease = try_acquire_worktree_copy_lock(common.as_path())
        .expect("attempt")
        .expect("free");
    let lock: PathBuf = lock_of(common.as_path());
    std::fs::remove_dir_all(lock.as_path()).expect("drop ours");
    publish(lock.as_path(), DEAD_OWNER);
    let failure: WorktreeCopyError = lease.release().expect_err("changed");
    assert_eq!(
        failure.0,
        format!(
            "cli-git: worktree-copy lock ownership changed: {}.",
            quote(lock.to_string_lossy().as_ref())
        )
    );
    assert!(lock.join(OWNER_RECORD_NAME).exists());
    remove(root.as_path());
}

/// An inherited lease is valid only while the published owner carries that token and is alive.
#[cfg(unix)]
#[test]
fn inherited_lease_requires_the_published_live_token() {
    let root: PathBuf = fixture("lock-inherited");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let lease: WorktreeCopyLease = try_acquire_worktree_copy_lock(common.as_path())
        .expect("attempt")
        .expect("free");
    assert!(validate_inherited_lease(common.as_path(), lease.lease_token()).expect("valid"));
    assert!(!validate_inherited_lease(common.as_path(), "other").expect("other token"));
    assert!(!validate_inherited_lease(common.as_path(), "").expect("empty token"));
    lease.release().expect("released");
    remove(root.as_path());
}

/// A blocking acquisition announces a live holder once on stderr, waits until it releases, then takes the lock.
#[cfg(unix)]
#[test]
fn waiter_names_a_live_holder_and_takes_the_lock_after_release() {
    let root: PathBuf = fixture("lock-waiter");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let held: WorktreeCopyLease = try_acquire_worktree_copy_lock(common.as_path())
        .expect("attempt")
        .expect("free");
    let holder_pid: i64 = i64::from(std::process::id());
    let releaser = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(150));
        return held.release();
    });
    let mut stderr: Vec<u8> = Vec::new();
    let lease: WorktreeCopyLease =
        acquire_worktree_copy_lock(common.as_path(), &mut stderr).expect("taken after release");
    releaser.join().expect("releaser").expect("released");
    let line: String = String::from_utf8(stderr).expect("utf8");
    assert_eq!(
        line.matches("cli-git: waiting for PID ").count(),
        1,
        "{line}"
    );
    assert!(
        line.contains(&format!("waiting for PID {holder_pid},")),
        "{line}"
    );
    assert!(
        line.contains("proven by its live process identity"),
        "{line}"
    );
    lease.release().expect("released");
    remove(root.as_path());
}

/// A blocking acquisition behind a live holder names it at the start, repeats the line at every interval with the time
///  waited and the limit, and ends at the limit with a refusal that names the holder and every way forward.
#[cfg(unix)]
#[test]
fn a_long_wait_behind_a_live_holder_repeats_the_holder_line() {
    let root: PathBuf = fixture("lock-waiter-repeat");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let held: WorktreeCopyLease = try_acquire_worktree_copy_lock(common.as_path())
        .expect("attempt")
        .expect("free");
    let holder_pid: i64 = i64::from(std::process::id());
    let common_for_thread: PathBuf = common.clone();
    let (sender, receiver) = std::sync::mpsc::channel::<(Result<WorktreeCopyLease, WorktreeCopyError>, Vec<u8>)>();
    std::thread::spawn(move || {
        let mut stderr: Vec<u8> = Vec::new();
        let outcome = acquire_worktree_copy_lock_with(
            common_for_thread.as_path(),
            &mut stderr,
            Duration::from_millis(1_000),
            Cadence { grace: Duration::from_millis(100), interval: Duration::from_millis(200) },
        );
        let _sent = sender.send((outcome, stderr));
    });
    let (outcome, stderr) = match receiver.recv_timeout(Duration::from_secs(10)) {
        Ok(finished) => finished,
        Err(_) => {
            held.release().expect("released");
            panic!("the wait did not end at its limit");
        }
    };
    let text: String = String::from_utf8(stderr).expect("utf8");
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.len() >= 4, "the start and at least three repeats: {text}");
    assert!(lines[0].starts_with(&format!("cli-git: waiting for PID {holder_pid},")), "{text}");
    assert!(lines[0].contains("giving up after 1 seconds (CLI_GIT_LOCK_WAIT_SECONDS changes this)"), "{text}");
    for line in &lines[1..] {
        assert!(line.starts_with(&format!("cli-git: still waiting for PID {holder_pid},")), "{text}");
        assert!(line.contains(" seconds; giving up after 1 seconds"), "{text}");
    }
    let refusal: String = outcome.expect_err("refused").0;
    assert!(refusal.contains(&format!("for PID {holder_pid}")), "{refusal}");
    assert!(refusal.contains("let that process finish and run the command again"), "{refusal}");
    assert!(refusal.contains("check with `ps`"), "{refusal}");
    assert!(refusal.contains("raise the limit"), "{refusal}");
    held.release().expect("released");
    remove(root.as_path());
}

/// A live holder that never releases ends a blocking acquisition at its limit,
///  with a refusal naming the holder, and leaves the holder's lock in place.
#[cfg(unix)]
#[test]
fn a_live_holder_past_the_limit_ends_the_wait_and_keeps_its_lock() {
    let root: PathBuf = fixture("lock-waiter-limit");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let held: WorktreeCopyLease = try_acquire_worktree_copy_lock(common.as_path())
        .expect("attempt")
        .expect("free");
    let holder_pid: i64 = i64::from(std::process::id());
    let common_for_thread: PathBuf = common.clone();
    let (sender, receiver) = std::sync::mpsc::channel::<Result<WorktreeCopyLease, WorktreeCopyError>>();
    std::thread::spawn(move || {
        let mut stderr: Vec<u8> = Vec::new();
        let _sent = sender.send(acquire_worktree_copy_lock_for(
            common_for_thread.as_path(),
            &mut stderr,
            Duration::from_millis(200),
        ));
    });
    let outcome: Result<WorktreeCopyLease, WorktreeCopyError> = match receiver.recv_timeout(Duration::from_secs(10)) {
        Ok(finished) => finished,
        Err(_) => {
            // Releasing the lock lets a wait without a limit finish so the failure is raised with the thread done.
            held.release().expect("released");
            panic!("the wait did not end at its limit");
        }
    };
    let refusal: WorktreeCopyError = outcome.expect_err("refused");
    let text: String = refusal.0;
    assert!(text.starts_with("cli-git: gave up after waiting "), "{text}");
    assert!(
        text.contains(&format!("for PID {holder_pid}, which still holds worktree-copy settlement")),
        "{text}"
    );
    assert!(text.contains("CLI_GIT_LOCK_WAIT_SECONDS sets how long cli-git waits"), "{text}");
    assert!(text.ends_with("cli-git left the lock in place."), "{text}");
    assert!(lock_of(common.as_path()).exists(), "the holder's lock stays");
    held.release().expect("released");
    remove(root.as_path());
}

/// A lease that is dropped without a release still frees the lock.
#[cfg(unix)]
#[test]
fn a_dropped_lease_frees_the_lock() {
    let root: PathBuf = fixture("lock-dropped");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let lease: WorktreeCopyLease = try_acquire_worktree_copy_lock(common.as_path())
        .expect("attempt")
        .expect("free");
    assert!(lock_of(common.as_path()).exists());
    drop(lease);
    assert!(!lock_of(common.as_path()).exists(), "the drop removed the lock");
    remove(root.as_path());
}

/// A wait on one owner ends at once when another owner has taken the lock,
///  even though that other owner is alive.
#[cfg(unix)]
#[test]
fn a_wait_ends_when_the_lock_passes_to_another_owner() {
    let root: PathBuf = fixture("lock-passed-on");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let held: WorktreeCopyLease = try_acquire_worktree_copy_lock(common.as_path())
        .expect("attempt")
        .expect("free");
    let waited_on: LockOwner = LockOwner {
        lease_token: "the-owner-that-left".to_owned(),
        owner_pid: i64::from(std::process::id()),
        owner_birth_identity: "the-owner-that-left".to_owned(),
        owner_pid_namespace: None,
    };
    let started: Instant = Instant::now();
    wait_while_owner_lives(
        lock_of(common.as_path()).as_path(),
        &waited_on,
        Duration::from_secs(5),
        &mut Vec::<u8>::new(),
        Cadence::standard(),
    )
    .expect("the lock is no longer the waited-on owner's");
    assert!(started.elapsed() < Duration::from_secs(4), "{:?}", started.elapsed());
    held.release().expect("released");
    remove(root.as_path());
}

/// A lock whose retirement is refused for any reason other than having vanished is an error, not a lost race.
#[cfg(unix)]
#[test]
fn a_refused_retirement_is_an_error() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("lock-retire-refused");
    let parent: PathBuf = std::fs::canonicalize(&root).expect("canonical");
    let lock: PathBuf = parent.join("settlement.lock");
    publish(lock.as_path(), DEAD_OWNER);
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o500)).expect("lock");
    let outcome: Result<bool, WorktreeCopyError> = remove_lock_of(lock.as_path(), "dead-token");
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o700)).expect("unlock");
    // A superuser renames anyway, so only an ordinary user shows the refusal.
    if unsafe { libc::geteuid() } == 0 {
        remove(root.as_path());
        return;
    }
    outcome.expect_err("the rename is refused");
    assert!(lock.join(OWNER_RECORD_NAME).exists(), "the lock stays");
    remove(root.as_path());
}

/// A lock that names another owner is left alone.
#[cfg(unix)]
#[test]
fn a_lock_of_another_owner_is_not_removed() {
    let root: PathBuf = fixture("lock-other-owner");
    let lock: PathBuf = root.join("settlement.lock");
    publish(lock.as_path(), DEAD_OWNER);
    assert!(!remove_lock_of(lock.as_path(), "someone-else").expect("checked"));
    assert!(lock.join(OWNER_RECORD_NAME).exists(), "the lock stays");
    assert!(remove_lock_of(lock.as_path(), "dead-token").expect("removed"));
    assert!(!lock.exists());
    remove(root.as_path());
}

/// A rename that finds the lock gone means another process removed it first; any other refusal is a fault.
#[cfg(unix)]
#[test]
fn a_vanished_lock_is_not_ours_to_remove_and_a_refused_rename_is_a_fault() {
    let root: PathBuf = fixture("lock-rename-outcomes");
    let lock: PathBuf = root.join("settlement.lock");
    publish(lock.as_path(), DEAD_OWNER);
    let vanished = remove_lock_of_with(lock.as_path(), "dead-token", |_from, _to| {
        return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
    });
    assert!(!vanished.expect("not ours to remove"));
    let refused = remove_lock_of_with(lock.as_path(), "dead-token", |_from, _to| {
        return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
    });
    refused.expect_err("a refused rename is a fault");
    assert!(lock.join(OWNER_RECORD_NAME).exists(), "the lock stays");
    remove(root.as_path());
}

/// The jitter source gives numbers in the unit interval that differ from call to call.
#[test]
fn the_jitter_source_gives_varying_numbers_in_the_unit_interval() {
    let samples: Vec<f64> = (0..64)
        .map(|_| return unit_random().expect("random"))
        .collect();
    assert!(
        samples.iter().all(|sample| return *sample >= 0.0 && *sample < 1.0),
        "{samples:?}"
    );
    assert!(
        samples.iter().any(|sample| return *sample != samples[0]),
        "{samples:?}"
    );
}

/// A lock that another owner replaced between the read and the rename is put back, and nothing is removed.
#[cfg(unix)]
#[test]
fn a_replacement_that_raced_the_rename_is_put_back() {
    let root: PathBuf = fixture("lock-replaced-in-race");
    let lock: PathBuf = root.join("settlement.lock");
    publish(lock.as_path(), DEAD_OWNER);
    let removed = remove_lock_of_with(lock.as_path(), "dead-token", |from, to| {
        // The replacement lands after the owner was read and before the lock is moved away.
        let record: PathBuf = from.join(OWNER_RECORD_NAME);
        std::fs::remove_file(record.as_path())?;
        crate::private_storage::write_private_file(
            record.as_path(),
            DEAD_OWNER.replace("dead-token", "usurper").as_bytes(),
        )
        .map_err(|error| return std::io::Error::other(error.to_string()))?;
        return std::fs::rename(from, to);
    });
    assert!(!removed.expect("not ours to remove"));
    let restored: String = std::fs::read_to_string(lock.join(OWNER_RECORD_NAME)).expect("restored record");
    assert!(restored.contains("usurper"), "{restored}");
    remove(root.as_path());
}

/// A lock that names another owner is not even moved: the rename is never asked for.
#[cfg(unix)]
#[test]
fn a_lock_of_another_owner_is_never_moved() {
    let root: PathBuf = fixture("lock-other-owner-unmoved");
    let lock: PathBuf = root.join("settlement.lock");
    publish(lock.as_path(), DEAD_OWNER);
    let outcome = remove_lock_of_with(lock.as_path(), "someone-else", |_from, _to| {
        return Err(std::io::Error::other("the rename must not be asked for"));
    });
    assert!(!outcome.expect("checked without moving"));
    assert!(lock.join(OWNER_RECORD_NAME).exists());
    remove(root.as_path());
}

/// The namespace is an optional key between `ownerPid` and `schemaVersion`; an absent, empty or mistyped one is none.
#[test]
fn the_namespace_is_an_optional_sorted_key() {
    let owner: LockOwner = LockOwner {
        lease_token: String::from("token"),
        owner_pid: 5,
        owner_birth_identity: String::from("linux:9"),
        owner_pid_namespace: Some(String::from("pid:[4026531836]")),
    };
    let text: String = encode_owner(&owner);
    assert_eq!(
        text,
        "{\"leaseToken\":\"token\",\"ownerBirthIdentity\":\"linux:9\",\"ownerPid\":5,\"ownerPidNamespace\":\"pid:[4026531836]\",\"schemaVersion\":1}\n"
    );
    assert_eq!(parse_owner(text.as_str()), Ok(owner));
    assert_eq!(parse_owner(DEAD_OWNER).expect("old record").owner_pid_namespace, None);
    for ignored in ["\"\"", "5", "null"] {
        let with_ignored: String = DEAD_OWNER.replace(",\"schemaVersion\"", &format!(",\"ownerPidNamespace\":{ignored},\"schemaVersion\""));
        assert_eq!(parse_owner(with_ignored.as_str()).expect(ignored).owner_pid_namespace, None, "{ignored}");
    }
}
