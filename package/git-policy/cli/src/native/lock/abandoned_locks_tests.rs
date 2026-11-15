//! What:
//!  Controls of the answer that decides whether a standing lock is named to the command that met it:
//!  the worktree the command runs in and the shared common directory,
//!  and never another linked worktree's own administrative directory.
//! Why:
//!  The gate on `0.6.1-alpha.34` failed an add that printed a standing-lock line about another add's `index.lock`,
//!  which the pass had found by walking the common directory into every linked worktree.
//!  The proof that clears or keeps a lock is unchanged everywhere;
//!  only the words stay where the person is,
//!  so these controls hold the line between the two.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(standsInTheWay('/r/.git/worktrees/held/index.lock', '/r/.git', '/r/.git')).toBe(false);
//! ```

use super::stands_in_the_way;
use std::path::Path;

/// The Git directory of a repository whose worktrees are below it,
/// as every path of a control is written.
const COMMON: &str = "/r/.git";

/// What:
///  A command of the main checkout meets the lock of the shared index and of a shared reference,
///  and not the lock of a linked worktree's own administrative directory.
/// Why:
///  The shared index and the references are the files this command's own Git writes;
///  the worktree below `worktrees/` is another worktree's state,
///  which its own commands meet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(standsInTheWay('/r/.git/index.lock', '/r/.git', '/r/.git')).toBe(true);
/// ```
#[test]
fn a_main_checkouts_command_meets_the_shared_locks_and_no_other_worktrees() {
    let common: &Path = Path::new(COMMON);
    assert!(stands_in_the_way(Path::new("/r/.git/index.lock"), common, common), "the shared index is this command's");
    assert!(stands_in_the_way(Path::new("/r/.git/refs/heads/main.lock"), common, common), "a shared reference is this command's");
    assert!(!stands_in_the_way(Path::new("/r/.git/worktrees/held/index.lock"), common, common), "another worktree's index lock is not");
}

/// What:
///  A command of a linked worktree meets its own administrative directory's lock,
///  the shared locks,
///  and not a sibling worktree's lock.
/// Why:
///  The index of a linked worktree lives in its own administrative directory,
///  so that lock is the one this command's Git writes;
///  a sibling's is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(standsInTheWay('/r/.git/worktrees/held/index.lock', '/r/.git/worktrees/held', '/r/.git')).toBe(true);
/// ```
#[test]
fn a_linked_worktrees_command_meets_its_own_administrative_directory_and_the_shared_locks() {
    let common: &Path = Path::new(COMMON);
    let own: &Path = Path::new("/r/.git/worktrees/held");
    assert!(stands_in_the_way(Path::new("/r/.git/worktrees/held/index.lock"), own, common), "its own index lock is this command's");
    assert!(stands_in_the_way(Path::new("/r/.git/index.lock"), own, common), "the shared index is this command's");
    assert!(!stands_in_the_way(Path::new("/r/.git/worktrees/other/index.lock"), own, common), "a sibling's index lock is not");
}

/// What:
///  A lock the pass cannot place below the common directory is named.
/// Why:
///  A path this cannot place is the state every lock got before this answer existed,
///  so a lock is named unless the pass knows it is another worktree's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(standsInTheWay('/elsewhere/index.lock', '/r/.git', '/r/.git')).toBe(true);
/// ```
#[test]
fn a_lock_no_worktree_explains_is_named() {
    assert!(stands_in_the_way(Path::new("/elsewhere/index.lock"), Path::new(COMMON), Path::new(COMMON)), "a lock outside the Git directories is named rather than hidden");
}

/// What:
///  A lock of a reference whose namespace is named `objects` or `logs` is found by the pass,
///  as one in any other namespace is.
/// Why:
///  Git accepts `refs/heads/objects/topic` and `refs/heads/logs/topic`,
///  and a lock taken for one stands in the folder that name gives.
///  A walk that skipped those names at every depth never found such a lock,
///  so a record of it was a licence nothing acted on,
///  and the command that met the lock could not clear it
///  (a review of `6a506fc03f`,
/// finding 5).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const ns of ['ordinary', 'objects', 'logs']) write(refs/heads/ns/topic.lock)
/// expect(walked([root]).locks).toEqual(all three)
/// ```
#[test]
fn a_lock_in_a_reference_namespace_named_objects_is_found() {
    let root: std::path::PathBuf = crate::test_support::fixture("abandoned-locks-objects-namespace");
    let mut expected: Vec<std::path::PathBuf> = Vec::new();
    for namespace in ["ordinary", "objects", "logs"] {
        let folder: std::path::PathBuf = root.join("refs/heads").join(namespace);
        std::fs::create_dir_all(folder.as_path()).expect("the reference namespace");
        let lock: std::path::PathBuf = folder.join("topic.lock");
        std::fs::write(lock.as_path(), b"a reference update\n").expect("the reference lock");
        expected.push(lock);
    }
    let found: Vec<std::path::PathBuf> = super::walked(std::slice::from_ref(&root)).locks;
    crate::test_support::remove(root.as_path());
    for lock in expected {
        assert!(found.contains(&lock), "the pass finds the lock of {}: {found:?}", lock.display());
    }
}

/// The names this module and the watch treat specially,
/// each of which Git also accepts as a reference and as a namespace of one.
/// Why:
///  A name that changes what the code does must be proved with a name a person can choose:
///  `objects` and `logs` are skipped below a Git directory,
///  `worktrees` and `modules` are Git's own layout folders there,
///  and `HEAD` and `packed-refs` are names Git uses for files of its own.
///  All six are legal reference names, and one of them in a reference folder was the trap
///  (a review of `ca936e0b93`,
/// finding 2).
const SPECIAL_NAMES: [&str; 6] = ["objects", "logs", "worktrees", "modules", "HEAD", "packed-refs"];

/// What:
///  Write the record of a child this repository started,
///  and the record that pins the lock a killed update left,
///  for a lock at `lock`.
/// Why:
///  The pass clears a lock on two records of this repository:
///  the one that pins the lock's own file and names the Git that took it,
///  and the one that says that number and birth are a process this boot had and no longer has.
///  A control that means to see a clearance writes both, as a run of the wrapper does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantChildRecord(pid, identity); plantBinding(common, lock, pid, identity)
/// ```
fn plant_the_two_records_of_a_killed_update(common: &Path, lock: &Path, pid: i64, identity: &str) {
    // A killed update leaves the lock, the PID file Git wrote beside it, and this repository's two records.
    std::fs::write(crate::stale_ref_lock_walk::pid_file_beside(lock), format!("pid {pid}\n")).expect("the update's PID file");
    let holders: std::path::PathBuf = crate::lock_holder_records::folder_of(common);
    std::fs::create_dir_all(holders.as_path()).expect("the holder record folder");
    std::fs::write(holders.join(format!("a-child-record-{pid}")), crate::lock_holder_records::record_text(&[("child", pid, String::from(identity))]))
        .expect("the child record");
    let folder = crate::index_lock_evidence::read_lock_metadata(lock.parent().expect("the lock's folder")).ok().flatten().expect("the folder's identity");
    let named: (u64, u64) = (folder.device, folder.inode);
    assert!(
        crate::lock_bindings::write(common, lock, &(pid, String::from(identity)), named).is_some(),
        "the lock that stands can be pinned: {}",
        lock.display()
    );
}

/// What:
///  Every name the code treats specially is a reference Git accepts and a namespace it accepts,
///  and a lock a killed update left at either is found and cleared.
/// Why:
///  The walk decided what a Git directory is by a file a folder holds,
///  and a reference named `HEAD` in `refs/heads/marker` made that folder look like one,
///  so its `objects` child was skipped and a lock there was never found or cleared
///  (a review of `ca936e0b93`,
/// finding 2).
///  A folder's kind is now a lookup in the Git directories the wrapper was told,
///  and this control states it with the names themselves:
///  each name as a ref (`refs/heads/<name>.lock`),
///  and each name as a namespace under a folder that holds a `HEAD` reference
///  (`refs/heads/marker/<name>/topic.lock`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// layoutTheTraps(); expect(walked(root).locks).toEqual(everyLock); expect(clearAbandonedLocks(root).cleared.length).toBe(everyLock.length)
/// ```
#[test]
fn every_name_the_code_treats_specially_is_found_and_cleared_under_refs() {
    let root: std::path::PathBuf = crate::test_support::fixture("abandoned-locks-special-names");
    let heads: std::path::PathBuf = root.join("refs/heads");
    std::fs::create_dir_all(heads.as_path()).expect("the reference folder");
    // The trap of the review: a reference named HEAD in the folder whose children are namespaces.
    std::fs::create_dir_all(heads.join("marker")).expect("the marker folder");
    std::fs::write(heads.join("marker").join("HEAD"), b"").expect("a reference named HEAD");
    let (pid, identity) = a_dead_holder();
    let mut every_lock: Vec<std::path::PathBuf> = Vec::new();
    for name in SPECIAL_NAMES {
        // The name as a reference: its lock stands beside the references of `refs/heads`.
        let as_a_reference: std::path::PathBuf = heads.join(format!("{name}.lock"));
        std::fs::write(as_a_reference.as_path(), b"a killed update's lock\n").expect("the reference lock");
        plant_the_two_records_of_a_killed_update(root.as_path(), as_a_reference.as_path(), pid, identity.as_str());
        every_lock.push(as_a_reference);
        if name == "HEAD" {
            // The trap itself: the reference named HEAD whose lock stands beside it.
            let beside_the_trap: std::path::PathBuf = heads.join("marker").join("HEAD.lock");
            std::fs::write(beside_the_trap.as_path(), b"a killed update's lock\n").expect("the reference lock");
            plant_the_two_records_of_a_killed_update(root.as_path(), beside_the_trap.as_path(), pid, identity.as_str());
            every_lock.push(beside_the_trap);
            continue;
        }
        // The name as a namespace, below the folder that holds the reference named HEAD.
        let namespace: std::path::PathBuf = heads.join("marker").join(name);
        std::fs::create_dir_all(namespace.as_path()).expect("the reference namespace");
        std::fs::write(namespace.join("topic"), b"").expect("a reference in the namespace");
        let in_the_namespace: std::path::PathBuf = namespace.join("topic.lock");
        std::fs::write(in_the_namespace.as_path(), b"a killed update's lock\n").expect("the namespace lock");
        plant_the_two_records_of_a_killed_update(root.as_path(), in_the_namespace.as_path(), pid, identity.as_str());
        every_lock.push(in_the_namespace);
    }
    let found: Vec<std::path::PathBuf> = super::walked(std::slice::from_ref(&root)).locks;
    let missing: Vec<&std::path::PathBuf> = every_lock.iter().filter(|lock| { return !found.contains(lock); }).collect();
    assert!(missing.is_empty(), "the walk finds every lock of a name Git accepts: {missing:?}");
    let outcome = super::clear_abandoned_locks(std::slice::from_ref(&root));
    let standing: Vec<std::path::PathBuf> = every_lock.iter().filter(|lock| { return lock.exists(); }).cloned().collect();
    let cleared: usize = outcome.cleared.len();
    crate::test_support::remove(root.as_path());
    assert!(standing.is_empty(), "the pass clears every lock a killed update left, and these stand: {standing:?}");
    assert_eq!(cleared, every_lock.len(), "every lock was reported cleared");
}

/// What:
///  The number and the birth identity of a process this boot had and no longer has.
/// Why:
///  A record is only a proof when its holder is a process of this boot, this PID namespace and this machine
///  that has ended;
///  a made-up identity judges unknown and proves nothing,
///  so a control that means to see a clearance names a real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holder = spawn('sleep', ['30']); const identity = identityOf(holder.pid); kill(holder); await ended;
/// ```
fn a_dead_holder() -> (i64, String) {
    let mut child: std::process::Child = std::process::Command::new("sleep")
        .arg("30")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("a helper that waits for its own end");
    let pid: i64 = i64::from(child.id());
    let identity: String = crate::lock_holder_records::identity_of_process(pid).expect("the helper's identity while it waits");
    let _killed = child.kill();
    let _reaped = child.wait();
    return (pid, identity);
}
