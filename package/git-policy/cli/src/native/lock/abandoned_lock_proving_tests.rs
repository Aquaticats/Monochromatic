//! What:
//!  Controls for the proof that a lock's PID file writer is gone:
//!  every way a number,
//! a record and a lock file can disagree,
//!  and the removal's own last step,
//!  where a Git takes the freed name and must keep its own PID file.
//! Why:
//!  The proof is about the writer of the PID file,
//! and the ways it can be wrong are finite:
//!  a number used again,
//!  two records for one number,
//!  a record written after the lock,
//!  a process that became another program,
//!  a holder in another PID namespace or on another machine,
//!  a holder the kernel will not let this account look at,
//!  and a lock no record explains.
//!  Each has a control here or the sentence in `abandoned_lock_proving.rs` that says the lock stays.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! plantLock({ pid, identity }); expect(clearOne(...)).toBe(undefined | clearance);
//! ```

use crate::abandoned_lock_proving::{Clearance, Evidence, UNLINKED_THE_LOCK, clear_one};
use crate::lock_holder_records::{folder_of, identity_of_birth, identity_of_process, record_text};
use crate::stale_ref_lock_walk::pid_file_beside;
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// A process of this control that holds nothing,
/// ended when it is dropped.
struct Helper(Child);

impl Drop for Helper {
    fn drop(&mut self) {
        let _killed = self.0.kill();
        let _waited = self.0.wait();
    }
}

/// What:
///  A directory shaped like a Git directory,
///  with the record folder the wrapper writes.
/// Why:
///   `clear_one` takes the lock's path and the common Git directory,
///  and the record folder is the only thing below it that matters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commonDir = fixture('abandoned-lock'); const lock = join(commonDir, 'index.lock');
/// ```
fn a_git_directory(name: &str) -> PathBuf {
    let root: PathBuf = fixture(name);
    std::fs::create_dir_all(folder_of(root.as_path())).expect("the record folder");
    return root;
}

/// What:
///  Plant one lock,
/// the PID file Git writes beside it,
///  and the record that names `pid` with `identity`.
/// Why:
///   The three files are the whole state the proof reads,
///  and a control makes each of them as a kill would leave it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plant({ commonDir, name: 'index.lock', pid, identities: [id] });
/// ```
fn plant(common_dir: &Path, name: &str, pid: i64, identities: &[String]) {
    let lock: PathBuf = common_dir.join(name);
    std::fs::create_dir_all(lock.parent().expect("the lock's folder")).expect("the lock's folder");
    std::fs::write(lock.as_path(), b"a half-written copy the dead holder never renamed\n").expect("the lock");
    // The clearing controls need the order the gate asks for: the PID file born strictly after the lock, which a file
    //  system of one tick cannot show for two files written at once. The helper makes it again until it can, and a file
    //  system that records no birth times fails the control naming what it said instead of testing nothing.
    crate::lock_birth_times::tests::write_after_the_lock(lock.as_path(), format!("pid {pid}\n").as_str());
    for (index, identity) in identities.iter().enumerate() {
        let file: PathBuf = folder_of(common_dir).join(format!("record-{name}-{index}"));
        std::fs::write(file, record_text(&[("child", pid, identity.clone())])).expect("the record");
    }
}

/// What:
///  A process that ran and ended,
///  with the identity its record would carry.
/// Why:
///   A record of a dead process is what a killed command leaves,
///  and its birth is one this boot had and no process has now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [pid, identity] = deadHolder();
/// ```
fn dead_holder() -> (i64, String) {
    let mut helper: Command = Command::new("sleep");
    helper.arg("600");
    return end_a_helper(helper);
}

/// What:
///  Start `program`,
/// read the identity of the process that runs it while that process lives,
///  then end it and collect it.
/// Why:
///   A helper that is already gone when its identity is read makes the control fail for the machine's
///  speed and not for the logic it tests:
///  a review of `d189162e3e` saw two of ten controls fail that way on `git --version`,
///  which exits before a reader can look.
///  A program that waits for its own end keeps the identity readable until this control ends it,
///  and the bounds below say how long the wait for that identity may take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const helper = spawn('sleep 600'); const identity = identityOf(helper.pid); helper.kill(); helper.wait();
/// ```
fn end_a_helper(mut program: Command) -> (i64, String) {
    let mut child: Child = program.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("a helper starts");
    let pid: i64 = i64::from(child.id());
    let identity: String = identity_of_process(pid).expect("the helper's identity is readable while it waits");
    let _killed = child.kill();
    child.wait().expect("the helper ends");
    return (pid, identity);
}

/// What:
///  A process that runs now,
///  with the identity its record would carry.
/// Why:
///   A number a live process has is the case no record can prove anything about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const live = liveHolder(); // killed when dropped
/// ```
fn live_holder() -> (i64, String, Helper) {
    let child: Child = Command::new("sleep").arg("600").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("a live helper");
    let pid: i64 = i64::from(child.id());
    let identity: String = identity_of_process(pid).expect("the live helper's identity");
    return (pid, identity, Helper(child));
}

/// What:
///  The identity text with one of its parts replaced,
///  so a record of another boot,
///  another PID namespace or another machine can be planted.
/// Why:
///  `birth#boot#scope#machine` is the whole identity,
///  and each part is what a judgement compares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withPart(identity, 1, 'a-different-boot')
/// ```
fn with_part(identity: &str, index: usize, value: &str) -> String {
    let mut parts: Vec<String> = identity.split('#').map(String::from).collect();
    while parts.len() < 4 {
        parts.push(String::new());
    }
    parts[index] = String::from(value);
    return parts.join("#");
}

/// What the control asks of one call.
#[derive(Debug)]
enum Ended {
    /// A lock was removed.
    Cleared(Clearance),
    /// The lock was left.
    Kept,
}

/// Run one call and say what it did,
/// checking that a kept lock is still there and a cleared one is gone.
fn looked(common_dir: &Path, name: &str) -> Ended {
    let lock: PathBuf = common_dir.join(name);
    return match clear_one(lock.as_path(), common_dir) {
        Some(clearance) => Ended::Cleared(clearance),
        None => {
            assert!(lock.exists(), "{name}: a lock the proof does not cover must be left where it is");
            Ended::Kept
        }
    };
}

/// What:
///  A lock whose PID file names a number no live process has,
///  and whose record is a process of this boot that ended,
///  is cleared.
/// Why:
///  This is the state a killed wrapper command leaves,
///  the case the whole change exists for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(looked(commonDir, 'index.lock')).toEqual(clearedWith(deadPid));
/// ```
#[test]
fn a_lock_of_a_dead_recorded_holder_with_a_free_number_is_cleared() {
    let common_dir: PathBuf = a_git_directory("cleared");
    let (pid, identity) = dead_holder();
    plant(common_dir.as_path(), "index.lock", pid, &[identity]);
    let ended: Ended = looked(common_dir.as_path(), "index.lock");
    let Ended::Cleared(clearance) = ended else { panic!("a lock of a dead recorded holder must be cleared: {ended:?}"); };
    assert_eq!(clearance.pid, pid);
    assert_eq!(clearance.evidence, Evidence::PidFile);
    assert!(!pid_file_beside(common_dir.join("index.lock").as_path()).exists(), "the PID file of a cleared lock goes with it");
    remove(common_dir.as_path());
}

/// What:
///  A lock and a PID file born in one tick of a coarse file system,
///  or on one that records no birth time,
///  leave the lock where it is:
///  nothing shows whether the PID file was written for this lock.
/// Why:
///  This is the state the review of `d189162e3e` reached by planting a PID file beside a live Git's
///  editor-phase lock:
///  the two births carried one tick of this volume's clock,
///  the pass removed the live lock,
///  and the commit failed with `unable to write new index file`.
///  A file system of a second's granularity gives that state to files written moments apart,
///  and one without birth times gives it to every pair.
///  The recorded-times run beside them is the positive control:
/// the same fixture is cleared when the
///  order is proven,
///  so the two keepings are the gate's answer and not a pass that stopped working.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withBirthTimes({ kind: 'coarse', tickMs: 2000 }, () => expect(kept)).toBe(true);
/// withBirthTimes({ kind: 'hidden' }, () => expect(kept)).toBe(true);
/// ```
#[test]
fn equal_or_missing_birth_times_keep_the_lock() {
    use crate::lock_birth_times::{BirthTimes, with_birth_times};
    let coarse: PathBuf = a_git_directory("coarse-birth-times");
    let (coarse_pid, coarse_identity) = dead_holder();
    plant(coarse.as_path(), "index.lock", coarse_pid, &[coarse_identity]);
    let kept: bool = with_birth_times(BirthTimes::Coarse(std::time::Duration::from_secs(3600)), || {
        return matches!(looked(coarse.as_path(), "index.lock"), Ended::Kept);
    });
    assert!(kept, "a lock and its PID file in one tick prove no order, so the lock stays");
    remove(coarse.as_path());
    let hidden: PathBuf = a_git_directory("no-birth-times");
    let (hidden_pid, hidden_identity) = dead_holder();
    plant(hidden.as_path(), "index.lock", hidden_pid, &[hidden_identity]);
    let kept_without_times: bool = with_birth_times(BirthTimes::Hidden, || { return matches!(looked(hidden.as_path(), "index.lock"), Ended::Kept); });
    assert!(kept_without_times, "a file system that records no birth time proves no order, so the lock stays");
    remove(hidden.as_path());
    let recorded: PathBuf = a_git_directory("recorded-birth-times");
    let (recorded_pid, recorded_identity) = dead_holder();
    plant(recorded.as_path(), "index.lock", recorded_pid, &[recorded_identity]);
    assert!(matches!(looked(recorded.as_path(), "index.lock"), Ended::Cleared(_)), "the same fixture is cleared when the order is proven");
    remove(recorded.as_path());
}

/// What:
///  A lock whose PID file names a number a live process has now is kept,
///  whatever the records say about that number's earlier owner.
/// Why:
///   The number was used before,
///  and a record of that earlier process proves nothing about the program that has the number now:
///  it may be the Git that wrote this very PID file.
///  Removing its lock made a real `git commit --all` fail with the index recovery message
///  (the review of `d4a0803231`,
/// finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const live = liveHolder(); const dead = deadHolder(); plant({ pid: live.pid, identities: [dead.identity] });
/// expect(looked(commonDir, 'index.lock')).toEqual('kept');
/// ```
#[test]
fn a_lock_whose_number_a_live_process_has_is_kept_even_when_a_dead_record_names_it() {
    let common_dir: PathBuf = a_git_directory("reused-number");
    let (live_pid, _live_identity, _holder) = live_holder();
    let (_dead_pid, dead_identity) = dead_holder();
    plant(common_dir.as_path(), "index.lock", live_pid, &[dead_identity]);
    assert!(matches!(looked(common_dir.as_path(), "index.lock"), Ended::Kept), "a live number is never proved gone");
    remove(common_dir.as_path());
}

/// What:
///  A lock whose PID file names a process that recorded itself and still runs is kept.
/// Why:
///   That is the wrapper's own forwarded Git (`exec` keeps the number and the birth),
///  and its lock is its own until it ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const live = liveHolder(); plant({ pid: live.pid, identities: [live.identity] }); expect(kept).toBe(true);
/// ```
#[test]
fn a_lock_whose_record_matches_the_live_process_is_kept() {
    let common_dir: PathBuf = a_git_directory("live-recorded");
    let (pid, identity, _holder) = live_holder();
    plant(common_dir.as_path(), "index.lock", pid, &[identity]);
    assert!(matches!(looked(common_dir.as_path(), "index.lock"), Ended::Kept), "a live recorded holder keeps its lock");
    remove(common_dir.as_path());
}

/// What:
///  Two records for one number must both prove a gone process,
///  and one record of another PID namespace keeps the lock.
/// Why:
///   A record of another namespace may be the writer,
///  and the first record read is not the only fact about the number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plant({ pid, identities: [deadOfThisBoot, deadOfAnotherNamespace] }); expect(kept).toBe(true);
/// ```
#[test]
fn two_records_for_one_number_clear_only_when_every_one_is_gone() {
    let common_dir: PathBuf = a_git_directory("two-records");
    let (pid, identity) = dead_holder();
    let (other_pid, other_identity) = dead_holder();
    let elsewhere: String = with_part(other_identity.as_str(), 2, "999999999");
    plant(common_dir.as_path(), "index.lock", pid, &[identity.clone(), elsewhere]);
    assert!(matches!(looked(common_dir.as_path(), "index.lock"), Ended::Kept), "one record of another namespace keeps the lock");
    // With the namespace record gone from the folder, the same lock is cleared.
    std::fs::remove_file(folder_of(common_dir.as_path()).join("record-index.lock-1")).expect("the other namespace's record");
    assert!(matches!(looked(common_dir.as_path(), "index.lock"), Ended::Cleared(_)), "both records of this boot that ended prove the writer gone");
    let _ = other_pid;
    remove(common_dir.as_path());
}

/// What:
///  A record of an earlier boot is gone,
/// and a record of another machine or another namespace is unknown.
/// Why:
///   A boot that ended ends every process in it,
///  so a record of it cannot name a live writer;
///  another machine's or namespace's record can,
///  and keeps the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(cleared(withPart(identity, 1, otherBoot))).toBe(true); expect(kept(withPart(identity, 3, otherMachine))).toBe(true);
/// ```
#[test]
fn a_record_of_another_boot_is_gone_and_one_of_another_machine_is_unknown() {
    let (pid, identity) = dead_holder();
    let earlier_boot: PathBuf = a_git_directory("another-boot");
    plant(earlier_boot.as_path(), "index.lock", pid, &[with_part(identity.as_str(), 1, "0123456789abcdef0123456789abcdef")]);
    assert!(matches!(looked(earlier_boot.as_path(), "index.lock"), Ended::Cleared(_)), "a boot that ended ends every process in it");
    remove(earlier_boot.as_path());
    let other_machine: PathBuf = a_git_directory("another-machine");
    plant(other_machine.as_path(), "index.lock", pid, &[with_part(identity.as_str(), 3, "fedcba9876543210fedcba9876543210")]);
    assert!(matches!(looked(other_machine.as_path(), "index.lock"), Ended::Kept), "another machine's record proves nothing here");
    remove(other_machine.as_path());
}

/// What:
///  A lock with no record,
/// and a lock with no PID file,
/// are kept.
/// Why:
///   A lock the wrapper did not take carries no record,
///  and a number nothing names is no proof at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(kept(noRecord)).toBe(true); expect(kept(noPidFile)).toBe(true);
/// ```
#[test]
fn a_lock_with_no_record_or_no_pid_file_is_kept() {
    let common_dir: PathBuf = a_git_directory("no-proof");
    let (pid, _identity) = dead_holder();
    plant(common_dir.as_path(), "index.lock", pid, &[]);
    assert!(matches!(looked(common_dir.as_path(), "index.lock"), Ended::Kept), "a lock no record explains is kept");
    std::fs::remove_file(pid_file_beside(common_dir.join("index.lock").as_path())).expect("the PID file");
    plant(common_dir.as_path(), "HEAD.lock", pid, &[identity_of_birth("linux:1")]);
    std::fs::remove_file(pid_file_beside(common_dir.join("HEAD.lock").as_path())).expect("the PID file");
    assert!(matches!(looked(common_dir.as_path(), "HEAD.lock"), Ended::Kept), "a lock with no PID file names nobody");
    remove(common_dir.as_path());
}

/// The variable that makes this test binary hold a lock open,
/// non-dumpably,
/// for the control below.
const HOLD_NON_DUMPABLY: &str = "CLI_GIT_TEST_NON_DUMPABLE_LOCK";

/// What:
///  The child body:
/// hold the lock open for writing,
/// become non-dumpable,
/// and wait to be killed.
/// Why:
///   `PR_SET_DUMPABLE` is reset by `execve`,
///  so a shell that is made non-dumpable before its own start is dumpable again after it:
///  the holder must be a process this binary ran,
///  with no exec after the call.
///  Inert unless the control set the variable,
///  so the suite's own run of this test does nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!process.env.HOLD) return; const fd = open(lock, 'r+'); prctl(0); writeFileSync(ready); park();
/// ```
#[test]
fn a_non_dumpable_writer_holds_the_lock_for_the_control() {
    let Ok(lock) = std::env::var(HOLD_NON_DUMPABLY) else {
        return;
    };
    let file: std::fs::File = std::fs::OpenOptions::new().read(true).write(true).open(lock.as_str()).expect("open the lock for writing");
    // SAFETY: `prctl` takes integers and touches no memory of this process.
    let refused: libc::c_int = unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0) };
    assert_eq!(refused, 0, "the holder must become non-dumpable");
    std::fs::write(format!("{lock}.ready"), b"ready\n").expect("the ready file");
    // The descriptor stays open for as long as this process lives, and parking costs the host nothing.
    let _held = file;
    std::thread::park();
}

/// What:
///  A lock a live writer of this account holds open is kept,
///  even when this account may not read that process's descriptors.
/// Why:
///   The kernel closes `/proc/<pid>/fd` of a process of this account that is not dumpable,
///  and such a process may write into the file after its name goes.
///  The kernel's read lease is refused while any program has the lock open for writing,
///  whatever account owns it and whether or not it is dumpable,
///  so the removal is refused without reading any descriptor
///  (`index_lock_holders_accounts.rs`,
/// the Kimi K3.1 Pro audit finding;
///  `merge_isolation_take_asides_proof.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holder = nonDumpableWriter(lock); expect(kept).toBe(true);
/// ```
#[test]
fn a_lock_a_non_dumpable_writer_of_this_account_holds_open_is_kept() {
    let common_dir: PathBuf = a_git_directory("non-dumpable-writer");
    let (pid, identity) = dead_holder();
    plant(common_dir.as_path(), "index.lock", pid, &[identity]);
    let lock: PathBuf = common_dir.join("index.lock");
    let mut holder: Child = start_non_dumpable_holder(lock.as_path());
    assert!(unreadable_holder(lock.as_path(), i64::from(holder.id())), "the holder must be non-dumpable and hold the lock open");
    assert!(matches!(looked(common_dir.as_path(), "index.lock"), Ended::Kept), "a writer nobody can look into keeps the lock");
    let _killed = holder.kill();
    let _waited = holder.wait();
    remove(common_dir.as_path());
}

/// What:
///  This test binary running the child body on `lock`,
///  which holds the file open for writing and is not dumpable.
/// Why:
///  Only a process that does both in one life can be the writer this control is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawn(process.execPath, ['--exact', name], { env: { [HOLD]: lock } })
/// ```
fn start_non_dumpable_holder(lock: &Path) -> Child {
    let binary: PathBuf = crate::test_support::this_test_binary().expect("this test binary");
    return Command::new(binary)
        .args(["--exact", "lock::abandoned_lock_proving_tests::a_non_dumpable_writer_holds_the_lock_for_the_control", "--nocapture", "--test-threads=1"])
        .env(HOLD_NON_DUMPABLY, lock)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("the non-dumpable holder starts");
}

/// What:
///  Whether the process holding `lock` is there and this account cannot read its descriptor directory,
///  which is what makes it the case the lease exists for.
/// Why:
///   A control whose holder turned out to be dumpable would pass for the wrong reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(fs.readdirSync(`/proc/${pid}/fd`)).toThrow(/EACCES/);
/// ```
fn unreadable_holder(lock: &Path, pid: i64) -> bool {
    let ready: PathBuf = PathBuf::from(format!("{}.ready", lock.display()));
    let began: std::time::Instant = std::time::Instant::now();
    while began.elapsed() < std::time::Duration::from_secs(10) {
        if ready.exists() {
            return std::fs::read_dir(format!("/proc/{pid}/fd")).is_err();
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    return false;
}

/// What:
///  A lock only a reader holds open is cleared,
///  because a reader cannot write into it.
/// Why:
///   The proof is that nothing can write into the file whose name goes,
///  and a program holding it open for reading cannot:
///  the kernel grants the read lease beside it,
///  and the new holder's lock is a different file.
///  This is the other half of what the kernel's answer tells apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reader = spawn('exec 9<lock; sleep 60'); expect(cleared).toBe(true);
/// ```
#[test]
fn a_lock_only_a_reader_holds_open_is_cleared_because_a_reader_cannot_write_into_it() {
    let common_dir: PathBuf = a_git_directory("reader-holder");
    let (pid, identity) = dead_holder();
    plant(common_dir.as_path(), "index.lock", pid, &[identity]);
    let lock: PathBuf = common_dir.join("index.lock");
    let mut reader: Child = Command::new("sh")
        .args(["-c", "exec 9<\"$1\"; exec sleep 120", "sh"])
        .arg(lock.as_path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("a reader starts");
    assert!(wait_until_open(lock.as_path(), i64::from(reader.id())), "the reader must hold the lock open before the proof is asked");
    assert!(matches!(looked(common_dir.as_path(), "index.lock"), Ended::Cleared(_)), "a reader cannot write into the file, so its name may go");
    let _killed = reader.kill();
    let _waited = reader.wait();
    remove(common_dir.as_path());
}

/// What:
///  A Git that takes the freed name between the lock's unlink and its PID file's
///  keeps its own PID file.
/// Why:
///   The removal unlinks the PID file only when it is still the very file that was judged;
///  a new holder's file has a different inode and change time,
///  and a live command must not lose the file that explains its lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hooks.set(UNLINKED_THE_LOCK, () => takeTheName()); expect(newPidFileExists()).toBe(true);
/// ```
#[test]
fn the_pid_file_of_a_git_that_takes_the_freed_name_is_left_alone() {
    let common_dir: PathBuf = a_git_directory("taken-name");
    let (pid, identity) = dead_holder();
    plant(common_dir.as_path(), "index.lock", pid, &[identity]);
    let lock: PathBuf = common_dir.join("index.lock");
    let pid_file: PathBuf = pid_file_beside(lock.as_path());
    let (new_holder_pid, _new_holder_identity, _holder) = live_holder();
    let taken: PathBuf = common_dir.clone();
    crate::service_fault::hooks::set(
        UNLINKED_THE_LOCK,
        std::rc::Rc::new(move || {
            // The Git that won the freed name: its lock first, then its PID file, as Git writes them.
            std::fs::write(taken.join("index.lock").as_path(), b"the new holder's index\n").expect("the new lock");
            std::fs::write(pid_file_beside(taken.join("index.lock").as_path()).as_path(), format!("pid {new_holder_pid}\n").as_bytes()).expect("the new PID file");
        }),
    );
    let ended: Ended = looked(common_dir.as_path(), "index.lock");
    crate::service_fault::hooks::clear();
    assert!(matches!(ended, Ended::Cleared(_)), "the abandoned lock itself is cleared");
    assert!(lock.exists(), "the new holder's lock stands");
    assert!(pid_file.exists(), "the new holder keeps its own PID file");
    assert_eq!(crate::stopped_git_locks::pid_named_by(pid_file.as_path()), i32::try_from(new_holder_pid).ok(), "the PID file is the new holder's");
    remove(common_dir.as_path());
}

/// Wait for the child to hold `lock` open,
/// bounded,
/// because the child must have it before the proof is asked.
fn wait_until_open(lock: &Path, pid: i64) -> bool {
    let opened: PathBuf = PathBuf::from(format!("/proc/{pid}/fd"));
    let began: std::time::Instant = std::time::Instant::now();
    while began.elapsed() < std::time::Duration::from_secs(10) {
        if let Ok(entries) = std::fs::read_dir(opened.as_path()) {
            for entry in entries.flatten() {
                if std::fs::read_link(entry.path()).is_ok_and(|target| { return target == lock; }) {
                    return true;
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    return false;
}

/// What:
///  Plant one lock and the PID file Git writes beside it,
///  as a kill leaves them,
///  and the record a command that watched its Git take the lock writes.
/// Why:
///   This is the state the change exists for:
///  the two files are written microseconds apart and a file system of coarse times gives them one
///  stamp,
///  and only the record ties the PID file to this lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantBound({ commonDir, name: 'index.lock', pid, identity });
/// ```
fn plant_bound(common_dir: &Path, name: &str, pid: i64, identity: &str) {
    let lock: PathBuf = common_dir.join(name);
    std::fs::write(lock.as_path(), b"a half-written copy the dead holder never renamed\n").expect("the lock");
    let pid_file: PathBuf = pid_file_beside(lock.as_path());
    std::fs::write(pid_file.as_path(), format!("pid {pid}\n").as_bytes()).expect("the PID file");
    let file: PathBuf = folder_of(common_dir).join(format!("record-{name}"));
    std::fs::write(file, record_text(&[("child", pid, String::from(identity))])).expect("the record");
    assert!(
        crate::lock_bindings::write(common_dir, lock.as_path(), &(pid, String::from(identity))).is_some(),
        "the lock that stands can be pinned"
    );
}

/// What:
///  A lock whose record names a dead Git is cleared
///  though the lock and its PID file carry one birth stamp.
/// Why:
///  This is the state no other proof on this volume can act on:
///  the birth order of the two files is not proven,
///  and the record a command wrote while its Git ran is the tie the PID file cannot give.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withBirthTimes(coarse(3600), () => expect(looked()).toEqual(clearedWith(deadPid)))
/// ```
#[test]
fn a_bound_lock_of_a_dead_git_is_cleared_though_the_two_births_are_equal() {
    use crate::lock_birth_times::{BirthTimes, with_birth_times};
    let common_dir: PathBuf = a_git_directory("bound-cleared");
    let (pid, identity) = dead_holder();
    plant_bound(common_dir.as_path(), "index.lock", pid, identity.as_str());
    let ended: Ended = with_birth_times(BirthTimes::Coarse(std::time::Duration::from_secs(3600)), || {
        return looked(common_dir.as_path(), "index.lock");
    });
    let Ended::Cleared(clearance) = ended else { panic!("a lock bound to a dead Git's own record is cleared: {ended:?}"); };
    assert_eq!(clearance.pid, pid);
    assert_eq!(clearance.evidence, Evidence::Binding, "the line says which proof acted");
    assert!(!pid_file_beside(common_dir.join("index.lock").as_path()).exists(), "the PID file of a cleared lock goes with it");
    assert_eq!(crate::lock_bindings::of(common_dir.as_path(), common_dir.join("index.lock").as_path()), None, "the record of a cleared lock goes with it");
    remove(common_dir.as_path());
}

/// What:
///  A lock made again at the path the record pinned is kept,
///  and so is a lock whose PID file names another process than the one the record does.
/// Why:
///  The first is the sequence a record of an inode alone would clear wrongly:
///  a later Git's lock at the same path.
///  The second is a lock whose PID file another Git wrote,
///  which is not the file the record's holder was named in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantBound(…); rm(lock); writeFileSync(lock, ''); expect(looked()).toEqual(kept)
/// ```
#[test]
fn a_lock_the_record_does_not_describe_is_kept() {
    let common_dir: PathBuf = a_git_directory("bound-replaced-lock");
    let (pid, identity) = dead_holder();
    plant_bound(common_dir.as_path(), "index.lock", pid, identity.as_str());
    let lock: PathBuf = common_dir.join("index.lock");
    std::fs::remove_file(lock.as_path()).expect("the pinned lock goes");
    std::fs::write(lock.as_path(), b"a later Git's own lock\n").expect("a later lock at the same path");
    assert!(matches!(looked(common_dir.as_path(), "index.lock"), Ended::Kept), "a record describes the file it pinned and no later one");
    remove(common_dir.as_path());
    let renamed: PathBuf = a_git_directory("bound-pid-file-of-another");
    let (second_pid, second_identity) = dead_holder();
    plant_bound(renamed.as_path(), "index.lock", second_pid, second_identity.as_str());
    let pid_file: PathBuf = pid_file_beside(renamed.join("index.lock").as_path());
    std::fs::write(pid_file.as_path(), b"pid 1\n").expect("a PID file naming another process");
    assert!(matches!(looked(renamed.as_path(), "index.lock"), Ended::Kept), "the PID file must still name the number the record does");
    remove(renamed.as_path());
}

/// What:
///  A lock bound to a Git that runs now is kept,
///  and so is one bound to a process no record of this repository explains.
/// Why:
///  A number a live process has proves nothing about the lock,
///  and a lock of a Git the wrapper did not start is the one a person must clear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantBound(…, live.pid, live.identity); expect(looked()).toEqual(kept)
/// ```
#[test]
fn a_bound_lock_of_a_live_or_unrecorded_git_is_kept() {
    let common_dir: PathBuf = a_git_directory("bound-live");
    let (pid, identity, _holder) = live_holder();
    plant_bound(common_dir.as_path(), "index.lock", pid, identity.as_str());
    assert!(matches!(looked(common_dir.as_path(), "index.lock"), Ended::Kept), "a lock bound to a live Git is that Git's own");
    remove(common_dir.as_path());
    let unrecorded: PathBuf = a_git_directory("bound-unrecorded");
    let (dead_pid, dead_identity) = dead_holder();
    let lock: PathBuf = unrecorded.join("index.lock");
    std::fs::write(lock.as_path(), b"a lock of a Git no command of this repository started\n").expect("the lock");
    let pid_file: PathBuf = pid_file_beside(lock.as_path());
    std::fs::write(pid_file.as_path(), format!("pid {dead_pid}\n").as_bytes()).expect("the PID file");
    assert!(crate::lock_bindings::write(unrecorded.as_path(), lock.as_path(), &(dead_pid, dead_identity)).is_some());
    assert!(matches!(looked(unrecorded.as_path(), "index.lock"), Ended::Kept), "a record naming a process no record of this repository explains proves nothing");
    remove(unrecorded.as_path());
}
