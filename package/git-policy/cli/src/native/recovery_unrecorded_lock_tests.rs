//! Controls for the release of a lock that a dead owner's PID file announces without a lock record.

use super::*;
use crate::process_use::ProcessUser;
use crate::test_support::{fixture, remove};
use crate::transaction_owner::{current_transaction_owner, encode_transaction_owner};
use std::time::Duration;

/// The transaction the fixtures use.
const ID: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";

/// The process number of the dead owner.
const DEAD: i64 = 4242;

/// A process probe that finds no process.
fn missing(_pid: i64) -> ProcessStart {
    return ProcessStart::Missing;
}

/// A process probe that finds a running process, as a reused number would.
fn running(_pid: i64) -> ProcessStart {
    return ProcessStart::Running { started_at_ms: 1.0, resolution_ms: 1 };
}

/// A scan that finds nobody.
fn nobody(_target: &UseTarget<'_>) -> UseScan {
    return UseScan::default();
}

/// A scan that finds a process holding the lock open.
fn a_holder(_target: &UseTarget<'_>) -> UseScan {
    return UseScan { users: vec![ProcessUser { pid: 77, command: Some(String::from("git")) }], unreadable: Vec::new() };
}

/// A scan that could not read a process of this account.
fn a_blind_scan(_target: &UseTarget<'_>) -> UseScan {
    return UseScan { users: Vec::new(), unreadable: vec![String::from("PID 9 fd 3: Permission denied")] };
}

/// Date a path.
fn date(path: &Path, seconds: u64) {
    std::fs::File::open(path)
        .expect("open")
        .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds))
        .expect("date");
}

/// One worktree directory with a published transaction, an empty lock and the PID file, dated in the order a landing
/// writes them: the owner record, then the lock, then the PID file.
struct Scene {
    /// The fixture root.
    root: PathBuf,
    /// The transaction directory.
    directory: PathBuf,
    /// The real index, which does not exist; only its lock and PID file do.
    index: PathBuf,
}

/// Build the scene with the dead owner `DEAD`.
fn scene(name: &str) -> Scene {
    let root: PathBuf = fixture(name);
    let directory: PathBuf = root.join("cli-git-transactions").join(ID);
    std::fs::create_dir_all(&directory).expect("transaction");
    let mut owner = current_transaction_owner(ID, "t").expect("owner");
    owner.owner_pid = DEAD;
    std::fs::write(directory.join(OWNER_FILENAME), encode_transaction_owner(&owner)).expect("owner record");
    let index: PathBuf = root.join("index");
    std::fs::write(real_lock_path(index.as_path()), b"").expect("lock");
    std::fs::write(lock_pid_path(index.as_path()), format!("pid {DEAD}\n")).expect("PID file");
    date(directory.join(OWNER_FILENAME).as_path(), 100);
    date(real_lock_path(index.as_path()).as_path(), 200);
    date(lock_pid_path(index.as_path()).as_path(), 300);
    return Scene { root, directory, index };
}

/// Run the release with a probe and a scan.
fn release(
    scene: &Scene,
    pid: i64,
    probe: fn(i64) -> ProcessStart,
    scan: fn(&UseTarget<'_>) -> UseScan,
) -> UnrecordedLockRelease {
    return release_announced_lock_with(scene.directory.as_path(), scene.index.as_path(), pid, probe, scan).expect("judged");
}

/// Every proof holds: the lock is empty, dated between the owner record and the PID file, the PID file names the dead
/// owner, no process has that number and nobody holds the lock.
#[test]
fn a_lock_announced_by_a_dead_owner_with_every_proof_is_released() {
    let scene: Scene = scene("unrecorded-released");
    assert_eq!(release(&scene, DEAD, missing, nobody), UnrecordedLockRelease::Released);
    assert!(!real_lock_path(scene.index.as_path()).exists());
    remove(scene.root.as_path());
}

/// A process of this account that the scan cannot read (`systemd --user` is one) does not keep the lock; only a holder
/// the scan can see does. This pins the named limit of the open-file proof.
#[test]
fn an_unreadable_process_does_not_keep_the_lock() {
    let scene: Scene = scene("unrecorded-blind");
    assert_eq!(release(&scene, DEAD, missing, a_blind_scan), UnrecordedLockRelease::Released);
    remove(scene.root.as_path());
}

/// No lock is nothing to release.
#[test]
fn an_absent_lock_is_absent() {
    let scene: Scene = scene("unrecorded-absent");
    std::fs::remove_file(real_lock_path(scene.index.as_path())).expect("remove");
    assert_eq!(release(&scene, DEAD, missing, nobody), UnrecordedLockRelease::Absent);
    remove(scene.root.as_path());
}

/// Each missing proof keeps the lock and names itself.
#[test]
fn a_missing_proof_keeps_the_lock_and_names_the_proof() {
    let lock = |scene: &Scene| { return real_lock_path(scene.index.as_path()); };
    let cases: [(&str, &str, fn(&Scene), fn(i64) -> ProcessStart, fn(&UseTarget<'_>) -> UseScan); 7] = [
        ("a reused number", "may have been reused", |_| {}, running, nobody),
        ("a holder", "process 77 holds it open", |_| {}, missing, a_holder),
        ("a PID file of another owner", "no PID file names the dead owner", |scene| {
            std::fs::write(lock_pid_path(scene.index.as_path()), "pid 99\n").expect("other owner");
            date(lock_pid_path(scene.index.as_path()).as_path(), 300);
        }, missing, nobody),
        ("no PID file", "no PID file names the dead owner", |scene| {
            std::fs::remove_file(lock_pid_path(scene.index.as_path())).expect("remove");
        }, missing, nobody),
        ("a lock newer than the PID file", "another process took it", |scene| {
            date(real_lock_path(scene.index.as_path()).as_path(), 400);
        }, missing, nobody),
        ("a lock older than the transaction", "before the transaction was published", |scene| {
            date(real_lock_path(scene.index.as_path()).as_path(), 50);
        }, missing, nobody),
        ("a lock with bytes", "something wrote to it", |scene| {
            std::fs::write(real_lock_path(scene.index.as_path()), b"x").expect("bytes");
            date(real_lock_path(scene.index.as_path()).as_path(), 200);
        }, missing, nobody),
    ];
    for (name, expected, damage, probe, scan) in cases {
        let scene: Scene = scene(format!("unrecorded-kept-{}", name.replace(' ', "-")).as_str());
        damage(&scene);
        let outcome: UnrecordedLockRelease = release(&scene, DEAD, probe, scan);
        assert!(
            matches!(&outcome, UnrecordedLockRelease::Kept(reason) if reason.contains(expected)),
            "{name}: {outcome:?}"
        );
        assert!(lock(&scene).exists(), "{name}: the lock stays");
        remove(scene.root.as_path());
    }
}

/// An owner of another PID namespace has a number that names nothing here, and an abandoned directory has no number.
#[test]
fn an_owner_without_a_trustworthy_number_keeps_the_lock() {
    let scene: Scene = scene("unrecorded-foreign");
    let mut owner = current_transaction_owner(ID, "t").expect("owner");
    owner.owner_pid = DEAD;
    owner.owner_pid_namespace = Some(String::from("pid:[1]"));
    std::fs::write(scene.directory.join(OWNER_FILENAME), encode_transaction_owner(&owner)).expect("foreign owner");
    date(scene.directory.join(OWNER_FILENAME).as_path(), 100);
    let outcome: UnrecordedLockRelease = release(&scene, DEAD, missing, nobody);
    assert!(matches!(&outcome, UnrecordedLockRelease::Kept(reason) if reason.contains("another PID namespace")), "{outcome:?}");
    std::fs::remove_file(scene.directory.join(OWNER_FILENAME)).expect("lose the owner record");
    let lost: UnrecordedLockRelease = release(&scene, 0, missing, nobody);
    assert!(matches!(&lost, UnrecordedLockRelease::Kept(reason) if reason.contains("no PID file names")), "{lost:?}");
    assert!(real_lock_path(scene.index.as_path()).exists());
    remove(scene.root.as_path());
}

/// The PID file announces an owner only by its exact text.
#[test]
fn a_pid_file_announces_its_owner_by_exact_text() {
    let scene: Scene = scene("unrecorded-announces");
    assert!(announces_owner(scene.index.as_path(), DEAD));
    assert!(!announces_owner(scene.index.as_path(), DEAD + 1));
    assert!(!announces_owner(scene.index.as_path(), 0));
    std::fs::write(lock_pid_path(scene.index.as_path()), format!("pid {DEAD}")).expect("no newline");
    assert!(!announces_owner(scene.index.as_path(), DEAD));
    std::fs::remove_file(lock_pid_path(scene.index.as_path())).expect("remove");
    assert!(!announces_owner(scene.index.as_path(), DEAD));
    remove(scene.root.as_path());
}

/// Without `preparing.json` the real index is the worktree's own, found from the registry's place.
#[test]
fn the_real_index_defaults_to_the_worktrees_own() {
    let scene: Scene = scene("unrecorded-default-index");
    assert_eq!(real_index_of_transaction(scene.directory.as_path()), scene.index);
    std::fs::write(scene.directory.join(PREPARING_FILENAME), b"not a record").expect("garbage");
    assert_eq!(real_index_of_transaction(scene.directory.as_path()), scene.index);
    remove(scene.root.as_path());
}

/// A dead owner with no lock record and no announcing PID file progressed nowhere, and nothing is touched.
#[test]
fn a_transaction_that_announced_nothing_is_not_progress() {
    let scene: Scene = scene("unrecorded-no-progress");
    std::fs::remove_file(lock_pid_path(scene.index.as_path())).expect("remove");
    let progressed: bool =
        release_without_preparing(scene.root.join("objects").as_path(), scene.directory.as_path(), &[], ID, DEAD).expect("judged");
    assert!(!progressed);
    assert!(real_lock_path(scene.index.as_path()).exists());
    remove(scene.root.as_path());
}
