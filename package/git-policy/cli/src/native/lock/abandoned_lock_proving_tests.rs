//! What:
//!  Controls for the proof that a lock's PID file writer is gone:
//!  every way a number,
//! a record and a lock file can disagree,
//!  and the removal's own last step,
//!  where the PID file is unlinked only if its device,
//! inode,
//! change time and contents are the ones judged,
//!  where a file another program wrote over it is kept by its contents,
//!  and where a Git that takes the freed name writes none.
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

use crate::abandoned_lock_proving::{
    Clearance,
    Evidence,
    UNLINKED_THE_LOCK,
    clear_one,
};
use crate::lock_holder_records::{
    folder_of,
    identity_of_birth,
    identity_of_process,
    record_text,
};
use crate::stale_ref_lock_names::pid_file_beside;
use crate::test_support::{
    fixture,
    remove,
};
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Child,
    Command,
    Stdio,
};

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
fn plant(
    common_dir: &Path,
    name: &str,
    pid: i64,
    identities: &[String],
) {
    let lock: PathBuf = common_dir.join(name);
    std::fs::create_dir_all(lock.parent().expect("the lock's folder")).expect("the lock's folder");
    std::fs::write(
        lock.as_path(),
        b"a half-written copy the dead holder never renamed\n",
    )
    .expect("the lock");
    // The clearing controls need the order the gate asks for: the PID file born strictly after the lock, which a file
    //  system of one tick cannot show for two files written at once. The helper makes it again until it can, and a file
    //  system that records no birth times fails the control naming what it said instead of testing nothing.
    crate::lock_birth_times::tests::write_after_the_lock(
        lock.as_path(),
        format!("pid {pid}\n").as_str(),
    );
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
///  The helper leads a process group of its own, which the test thread's fixtures kill when the thread ends,
///  so an assertion that unwinds past the kill below leaves no helper behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const helper = spawnOwnedQuiet('sleep 600'); const identity = identityOf(helper.pid); helper.kill(); helper.wait();
/// ```
fn end_a_helper(mut program: Command) -> (i64, String) {
    // Quiet, because nothing reads this helper's output: it holds no stream of the run open.
    let mut child: Child =
        crate::test_support::spawn_owned_quiet(&mut program).expect("a helper starts");
    let pid: i64 = i64::from(child.id());
    let identity: String =
        identity_of_process(pid).expect("the helper's identity is readable while it waits");
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
    let mut command: Command = Command::new("sleep");
    command.arg("600");
    // The control ends this helper through `Helper`, and the fixtures end the group it leads should an assertion unwind first.
    let child: Child = crate::test_support::spawn_owned_quiet(&mut command).expect("a live helper");
    let pid: i64 = i64::from(child.id());
    let identity: String = identity_of_process(pid).expect("the live helper's identity");
    return (pid, identity, Helper(child));
}

/// The position of each part of an identity text:
/// the birth,
/// the boot,
/// the PID namespace,
/// and the machine.
const BIRTH_PART: usize = 0;
const BOOT_PART: usize = 1;
const PID_SCOPE_PART: usize = 2;
const MACHINE_PART: usize = 3;

/// A boot ID that is no boot of this host's.
const ANOTHER_BOOT: &str = "0123456789abcdef0123456789abcdef";

/// A machine ID that is no host's here.
const ANOTHER_MACHINE: &str = "fedcba9876543210fedcba9876543210";

/// What:
///  The identity text with the part at `index` replaced by `value`.
/// Why:
///   A control that plants a record of another boot,
///  another PID namespace or another machine replaces one part of the text a writer of this host produced,
///  and every other part must stay that writer's.
///  A part the writer did not write makes a text no command of this repository ever leaves,
///  so the proof would be asked about a record that cannot exist:
///  the gate's container has no machine ID,
///  so its identity there ends at the PID namespace,
///  and the padding this function used to add made a fourth part with nothing in it,
///  which the reader answers unknown for
///  (the control below passed in a container for a reason it does not test).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withPart(identity, 1, 'a-different-boot') // fails when the text has no part 1
/// ```
fn with_part(
    identity: &str,
    index: usize,
    value: &str,
) -> String {
    let mut parts: Vec<String> = identity.split('#').map(String::from).collect();
    // What:     `parts.get_mut(index)` is the one part to replace, or nothing when the text is shorter than the index;
    //           the `else` ends the control naming the text instead of planting a record nothing wrote.
    // Why:      A replacement may only stand where the writer wrote a part.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (index >= parts.length) throw new Error(`the identity ${identity} has no part ${index}`);
    // ```
    let Some(part) = parts.get_mut(index) else {
        panic!("the identity {identity} has no part {index}, so this control cannot name it");
    };
    *part = String::from(value);
    return parts.join("#");
}

/// What:
///  The identity text of a writer that could read a machine ID:
///  the birth,
///  the boot,
///  the PID namespace and `machine`.
/// Why:
///   The machine is the one part a writer leaves out on a host that has none,
///  which is every container whose image carries no `/etc/machine-id`,
///  so a control that plants another machine's record must write that part itself
///  rather than rely on this host having one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withMachine(identity, 'fedcba98…') // birth#boot#scope#machine
/// ```
fn with_machine(
    identity: &str,
    machine: &str,
) -> String {
    let mut parts: Vec<String> = identity.split('#').map(String::from).collect();
    // The machine stands after the birth, the boot and the PID namespace, which every Linux writer names, so those three
    //  parts are there whatever this host could read.
    assert!(
        parts.len() >= MACHINE_PART,
        "the identity {identity} names no boot and PID namespace to put a machine after"
    );
    parts.truncate(MACHINE_PART);
    parts.push(String::from(machine));
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
fn looked(
    common_dir: &Path,
    name: &str,
) -> Ended {
    let lock: PathBuf = common_dir.join(name);
    return match clear_one(lock.as_path(), common_dir) {
        Some(clearance) => Ended::Cleared(clearance),
        None => {
            assert!(
                lock.exists(),
                "{name}: a lock the proof does not cover must be left where it is"
            );
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
    let Ended::Cleared(clearance) = ended else {
        panic!("a lock of a dead recorded holder must be cleared: {ended:?}");
    };
    assert_eq!(clearance.pid, pid);
    assert_eq!(clearance.evidence, Evidence::PidFile);
    assert!(
        !pid_file_beside(common_dir.join("index.lock").as_path()).exists(),
        "the PID file of a cleared lock goes with it"
    );
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
    use crate::lock_birth_times::{
        BirthTimes,
        with_birth_times,
    };
    let coarse: PathBuf = a_git_directory("coarse-birth-times");
    let (coarse_pid, coarse_identity) = dead_holder();
    plant(
        coarse.as_path(),
        "index.lock",
        coarse_pid,
        &[coarse_identity],
    );
    let kept: bool = with_birth_times(
        BirthTimes::Coarse(std::time::Duration::from_secs(3600)),
        || {
            return matches!(looked(coarse.as_path(), "index.lock"), Ended::Kept);
        },
    );
    assert!(
        kept,
        "a lock and its PID file in one tick prove no order, so the lock stays"
    );
    remove(coarse.as_path());
    let hidden: PathBuf = a_git_directory("no-birth-times");
    let (hidden_pid, hidden_identity) = dead_holder();
    plant(
        hidden.as_path(),
        "index.lock",
        hidden_pid,
        &[hidden_identity],
    );
    let kept_without_times: bool = with_birth_times(BirthTimes::Hidden, || {
        return matches!(looked(hidden.as_path(), "index.lock"), Ended::Kept);
    });
    assert!(
        kept_without_times,
        "a file system that records no birth time proves no order, so the lock stays"
    );
    remove(hidden.as_path());
    let recorded: PathBuf = a_git_directory("recorded-birth-times");
    let (recorded_pid, recorded_identity) = dead_holder();
    plant(
        recorded.as_path(),
        "index.lock",
        recorded_pid,
        &[recorded_identity],
    );
    assert!(
        matches!(looked(recorded.as_path(), "index.lock"), Ended::Cleared(_)),
        "the same fixture is cleared when the order is proven"
    );
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
    plant(
        common_dir.as_path(),
        "index.lock",
        live_pid,
        &[dead_identity],
    );
    assert!(
        matches!(looked(common_dir.as_path(), "index.lock"), Ended::Kept),
        "a live number is never proved gone"
    );
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
    assert!(
        matches!(looked(common_dir.as_path(), "index.lock"), Ended::Kept),
        "a live recorded holder keeps its lock"
    );
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
    plant(
        common_dir.as_path(),
        "index.lock",
        pid,
        &[identity.clone(), elsewhere],
    );
    assert!(
        matches!(looked(common_dir.as_path(), "index.lock"), Ended::Kept),
        "one record of another namespace keeps the lock"
    );
    // With the namespace record gone from the folder, the same lock is cleared.
    std::fs::remove_file(folder_of(common_dir.as_path()).join("record-index.lock-1"))
        .expect("the other namespace's record");
    assert!(
        matches!(
            looked(common_dir.as_path(), "index.lock"),
            Ended::Cleared(_)
        ),
        "both records of this boot that ended prove the writer gone"
    );
    let _ = other_pid;
    remove(common_dir.as_path());
}

/// What:
///  A record of an earlier boot is gone,
/// and a record of another machine or another namespace is unknown,
/// in a container as on a host that names a machine.
/// Why:
///   A boot that ended ends every process in it,
///  so a record of it cannot name a live writer;
///  another machine's or namespace's record can,
///  and keeps the lock.
///  Each record is planted by replacing one part of the identity this host's writer would leave,
///  never by adding a part it did not write:
///  a container's image carries no machine ID,
///  so the identity there has three parts,
///  and a fourth one with nothing in it names no machine and keeps every lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(cleared(withPart(identity, 1, otherBoot))).toBe(true); expect(kept(withMachine(identity, otherMachine))).toBe(true);
/// ```
#[test]
fn a_record_of_another_boot_is_gone_and_one_of_another_machine_is_unknown() {
    let (pid, identity) = dead_holder();
    let earlier_boot: PathBuf = a_git_directory("another-boot");
    plant(
        earlier_boot.as_path(),
        "index.lock",
        pid,
        &[with_part(identity.as_str(), BOOT_PART, ANOTHER_BOOT)],
    );
    assert!(
        matches!(
            looked(earlier_boot.as_path(), "index.lock"),
            Ended::Cleared(_)
        ),
        "a boot that ended ends every process in it"
    );
    remove(earlier_boot.as_path());
    let other_machine: PathBuf = a_git_directory("another-machine");
    plant(
        other_machine.as_path(),
        "index.lock",
        pid,
        &[with_machine(identity.as_str(), ANOTHER_MACHINE)],
    );
    assert!(
        matches!(looked(other_machine.as_path(), "index.lock"), Ended::Kept),
        "another machine's record proves nothing here"
    );
    remove(other_machine.as_path());
}

/// What:
///  A record whose text is not the shape a writer of this release writes,
///  because its boot part is no boot,
///  keeps the lock.
/// Why:
///   Every part after the birth stands in its own place,
///  and a writer that cannot read the boot ID of its host leaves that part out,
///  so the number of its PID namespace stands where the boot belongs.
///  Read as a boot it is another boot,
///  which proves every process of it ended,
///  and the proof would remove the lock of a holder that runs:
///  the one verdict a lock's reader must never give wrongly.
///  The control is asked in the gate's container as on a host:
///  a host that names a machine answers unknown for a text with no machine part already,
///  so only a container shows the text being read as a boot that ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plant({ pid, identities: [`linux:1234#${scope}`] }); expect(looked()).toEqual('kept')
/// ```
#[test]
fn a_record_whose_boot_part_is_no_boot_keeps_the_lock() {
    let common_dir: PathBuf = a_git_directory("no-boot-part");
    let (pid, identity) = dead_holder();
    let parts: Vec<&str> = identity.split('#').collect();
    // The birth and the PID namespace are the two parts a writer names when it cannot read the boot ID of its host.
    assert!(
        parts.len() > PID_SCOPE_PART,
        "the identity {identity} names no PID namespace, so this control cannot drop its boot"
    );
    let without_boot: String = format!("{}#{}", parts[BIRTH_PART], parts[PID_SCOPE_PART]);
    plant(common_dir.as_path(), "index.lock", pid, &[without_boot]);
    assert!(
        matches!(looked(common_dir.as_path(), "index.lock"), Ended::Kept),
        "a text whose boot part is no boot proves nothing about its writer"
    );
    remove(common_dir.as_path());
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
    assert!(
        matches!(looked(common_dir.as_path(), "index.lock"), Ended::Kept),
        "a lock no record explains is kept"
    );
    std::fs::remove_file(pid_file_beside(common_dir.join("index.lock").as_path()))
        .expect("the PID file");
    plant(
        common_dir.as_path(),
        "HEAD.lock",
        pid,
        &[identity_of_birth("linux:1")],
    );
    std::fs::remove_file(pid_file_beside(common_dir.join("HEAD.lock").as_path()))
        .expect("the PID file");
    assert!(
        matches!(looked(common_dir.as_path(), "HEAD.lock"), Ended::Kept),
        "a lock with no PID file names nobody"
    );
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
    let file: std::fs::File = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(lock.as_str())
        .expect("open the lock for writing");
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
    assert!(
        unreadable_holder(lock.as_path(), i64::from(holder.id())),
        "the holder must be non-dumpable and hold the lock open"
    );
    assert!(
        matches!(looked(common_dir.as_path(), "index.lock"), Ended::Kept),
        "a writer nobody can look into keeps the lock"
    );
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
    // The holder answers nothing the parent reads: it records itself through the lock file and its ready marker.
    let environment: [(&str, &std::ffi::OsStr); 1] = [(HOLD_NON_DUMPABLY, lock.as_os_str())];
    return crate::child_test_process::started(crate::child_test_process::ChildTest {
        exact_test: "lock::abandoned_lock_proving_tests::a_non_dumpable_writer_holds_the_lock_for_the_control",
        environment: environment.as_slice(),
        standard_output: Stdio::null(),
        standard_error: Stdio::null(),
        ..crate::child_test_process::ChildTest::default()
    });
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
fn unreadable_holder(
    lock: &Path,
    pid: i64,
) -> bool {
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
    let mut command: Command = Command::new("sh");
    command
        .args(["-c", "exec 9<\"$1\"; exec sleep 120", "sh"])
        .arg(lock.as_path());
    // Owned, so the `sleep` behind this shell goes with the test whichever way the control ends.
    let mut reader: Child =
        crate::test_support::spawn_owned_quiet(&mut command).expect("a reader starts");
    assert!(
        wait_until_open(lock.as_path(), i64::from(reader.id())),
        "the reader must hold the lock open before the proof is asked"
    );
    assert!(
        matches!(
            looked(common_dir.as_path(), "index.lock"),
            Ended::Cleared(_)
        ),
        "a reader cannot write into the file, so its name may go"
    );
    let _killed = reader.kill();
    let _waited = reader.wait();
    remove(common_dir.as_path());
}

/// What the holder that takes the freed name does to the PID file beside the new lock.
/// Why:
///  The removal's last look compares four facts,
///  and a control makes one of them differ by what this holder writes
///  or by what the arm answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheNewHolder = 'no-pid-file' | 'its-number-over-the-judged-file' | 'the-judged-number-over-the-judged-file' | 'the-judged-number-in-a-new-file' | 'other-bytes-over-the-judged-file';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TheNewHolder {
    /// Writes no PID file,
    /// because Git's exclusive create is refused while the judged file stands.
    WritesNone,
    /// Truncates the judged PID file and writes its own number over it,
    /// through the same inode.
    ItsNumberInPlace,
    /// Truncates the judged PID file and writes the number already there,
    /// so its bytes do not change.
    TheJudgedNumberInPlace,
    /// Removes the judged PID file and writes the judged number again,
    /// which is a new file with the same bytes.
    TheJudgedNumberInANewOne,
    /// Truncates the judged PID file and writes the judged number in other bytes,
    /// so the number the file names is the same and its contents are not.
    OtherBytesInPlace(OtherBytes),
}

/// What:
///  Which other bytes a holder writes over the judged PID file,
///  leaving the number the same.
/// Why:
///  Every one of these parses to the judged number,
///  so a comparison of the number cannot tell it from the judged file
///  and a comparison of the contents can.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OtherBytes = 'leading-zero' | 'trailing-padding' | 'carriage-return' | 'no-newline';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OtherBytes {
    /// `pid 0<number>`.
    LeadingZero,
    /// `pid <number>` and spaces before the newline.
    TrailingPadding,
    /// `pid <number>` and a carriage return before the newline.
    CarriageReturn,
    /// `pid <number>` and no newline at all.
    NoNewline,
}

/// A change time every read of one control reports,
/// so that two writes of that control compare equal whatever the clock says.
const A_CHANGE_TIME_EVERY_READ_REPORTS: i64 = 1_000;

/// The number a file system that hands every file the freed inode number back would report.
const ONE_INODE_FOR_EVERY_FILE: u64 = 1;

/// What:
///  One control's fixture:
///  which holder takes the freed name,
/// what it writes to the PID file,
/// and what the arm answers for the two reads.
/// Why:
///  Every control of the removal's last look builds the same fixture and differs in one of those,
///  so one struct states the difference and the controls themselves read as their names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FreedName = { name: string; holder: TheNewHolder; everyReadReportsChangeTime?: number; everyInodeIs?: number; later?: Changed };
/// ```
struct FreedName {
    /// The fixture directory's name.
    name: &'static str,
    /// What the holder that wins the freed name does to the PID file.
    holder: TheNewHolder,
    /// The change time every read of the PID file reports,
    /// if the control arms one,
    /// so no clock decides what that control holds.
    every_read_reports_change_time: Option<i64>,
    /// The inode number every read of the PID file reports,
    /// if the control arms one.
    every_inode_is: Option<u64>,
    /// The term a read of the PID file taken after the lock's name went reports differently,
    /// if the control arms one.
    later: Option<crate::lock_metadata_arm::Changed>,
}

/// What one such fixture left behind.
struct TakenName {
    /// The directory the control removes when it ends.
    common_dir: PathBuf,
    /// The new holder's lock.
    lock: PathBuf,
    /// The PID file beside it.
    pid_file: PathBuf,
    /// The number the abandoned lock's PID file named.
    judged_pid: i64,
    /// The number the new holder has.
    new_pid: i64,
    /// What the removal did.
    ended: Ended,
}

/// What:
///  A fixture with an abandoned lock,
///  a holder that takes the freed name at the instant the lock's name goes,
///  and the arm over the two reads of the PID file.
/// Why:
///  The state,
/// the write and the answer are the whole difference between the controls of this section,
///  and the hook runs on this thread inside `clear_one`,
///  so the arm of this thread answers its reads,
///  and it moves that arm to the phase the removal's last look is in,
///  so the read the removal judged is the one it took before the lock's name went.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = aFreedNameTaken({ name, holder, everyReadReportsChangeTime, everyInodeIs, later }); expect(taken.ended).toMatchObject({ cleared: true });
/// ```
fn a_freed_name_taken(fixture: FreedName) -> TakenName {
    let common_dir: PathBuf = a_git_directory(fixture.name);
    let (judged_pid, identity) = dead_holder();
    plant(common_dir.as_path(), "index.lock", judged_pid, &[identity]);
    let lock: PathBuf = common_dir.join("index.lock");
    let pid_file: PathBuf = pid_file_beside(lock.as_path());
    let (new_pid, _new_identity, _holder) = live_holder();
    let taken: PathBuf = common_dir.clone();
    crate::service_fault::hooks::set(
        UNLINKED_THE_LOCK,
        std::rc::Rc::new(move || {
            // The removal's last look starts here, so the arm answers every read of the PID file from this point as one
            //  taken after the lock's name went, and a judged read taken here cannot pass for the judged one.
            crate::lock_metadata_arm::the_locks_name_is_gone();
            let new_lock: PathBuf = taken.join("index.lock");
            let new_pid_file: PathBuf = pid_file_beside(new_lock.as_path());
            // The holder that won the freed name writes its lock first, as every writer does.
            std::fs::write(new_lock.as_path(), b"the new holder's index\n").expect("the new lock");
            match fixture.holder {
                TheNewHolder::WritesNone => {
                    let refused: std::io::Result<std::fs::File> = std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(new_pid_file.as_path());
                    assert!(
                        refused.is_err(),
                        "an exclusive create of the PID file is refused while the judged file stands"
                    );
                }
                TheNewHolder::ItsNumberInPlace => {
                    std::fs::write(
                        new_pid_file.as_path(),
                        format!("pid {new_pid}\n").as_bytes(),
                    )
                    .expect("its number over the judged file");
                }
                TheNewHolder::TheJudgedNumberInPlace => {
                    std::fs::write(
                        new_pid_file.as_path(),
                        format!("pid {judged_pid}\n").as_bytes(),
                    )
                    .expect("the judged number over the judged file");
                }
                TheNewHolder::TheJudgedNumberInANewOne => {
                    std::fs::remove_file(new_pid_file.as_path()).expect("the judged PID file goes");
                    std::fs::write(
                        new_pid_file.as_path(),
                        format!("pid {judged_pid}\n").as_bytes(),
                    )
                    .expect("the judged number in a new file");
                }
                TheNewHolder::OtherBytesInPlace(other) => {
                    let bytes: String = match other {
                        OtherBytes::LeadingZero => format!("pid 0{judged_pid}\n"),
                        OtherBytes::TrailingPadding => format!("pid {judged_pid}   \n"),
                        OtherBytes::CarriageReturn => format!("pid {judged_pid}\r\n"),
                        OtherBytes::NoNewline => format!("pid {judged_pid}"),
                    };
                    std::fs::write(new_pid_file.as_path(), bytes.as_bytes())
                        .expect("the judged number in other bytes over the judged file");
                }
            }
        }),
    );
    let arm: crate::lock_metadata_arm::MetadataArm = crate::lock_metadata_arm::MetadataArm {
        path: pid_file.clone(),
        every_read_reports_change_time: fixture.every_read_reports_change_time,
        every_inode_is: fixture.every_inode_is,
        later: fixture.later,
    };
    let ended: Ended = crate::lock_metadata_arm::with_arm(arm, || {
        return looked(common_dir.as_path(), "index.lock");
    });
    crate::service_fault::hooks::clear();
    return TakenName {
        common_dir,
        lock,
        pid_file,
        judged_pid,
        new_pid,
        ended,
    };
}

/// What:
///  A PID file another program wrote over the judged one,
/// whose bytes name another number,
/// is left alone.
/// Why:
///   This is a real state and not a fixture's invention:
///  `gitbutlerapp/grit` writes Git's sidecar with an open that creates and truncates
///  (`grit-lib/src/lockfile.rs`,
///  `write_lock_pid_file`,
///  631 to 640,
///  at `d1637255d9`),
///  after it has taken the lock with an exclusive create
///  (`grit-lib/src/index/core.rs`,
///  `IndexLockFile::acquire`,
///  4215 to 4231),
///  so a grit that takes the freed name truncates the judged file through the same inode.
///  Its device and inode are then the judged ones,
///  and the control also arms a file system that hands the freed inode number back and one change time both reads report,
///  so neither the inode nor the clock can decide:
///  the contents do,
/// and they are not the judged ones.
///  The real state is this one,
/// and the pin of the contents term is not it:
///  the four controls that write the judged number in other bytes are that pin.
///  It compares the file's contents byte for byte with the contents it judged,
///  and the one case left open is a program that writes the very same bytes back,
///  and the instant between the last read and the unlink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = aFreedNameTaken({ name: 'written-over', holder: 'its-number-over-the-judged-file', everyReadReportsChangeTime: 1000, everyInodeIs: 1 });
/// expect(exists(taken.pidFile)).toBe(true); expect(readPid(taken.pidFile)).toBe(taken.newPid);
/// ```
#[test]
fn a_pid_file_another_program_wrote_over_the_judged_one_is_left_alone() {
    let taken: TakenName = a_freed_name_taken(FreedName {
        name: "written-over",
        holder: TheNewHolder::ItsNumberInPlace,
        every_read_reports_change_time: Some(A_CHANGE_TIME_EVERY_READ_REPORTS),
        every_inode_is: Some(ONE_INODE_FOR_EVERY_FILE),
        later: None,
    });
    assert!(
        matches!(taken.ended, Ended::Cleared(_)),
        "the abandoned lock itself is cleared: {:?}",
        taken.ended
    );
    assert!(taken.lock.exists(), "the new holder's lock stands");
    assert!(
        taken.pid_file.exists(),
        "a PID file another program wrote over the judged one is left alone"
    );
    assert_eq!(
        crate::stale_ref_lock_names::pid_named_by(taken.pid_file.as_path()),
        i32::try_from(taken.new_pid).ok(),
        "the file names the writer that stands"
    );
    remove(taken.common_dir.as_path());
}

/// What:
///  A PID file whose number gained a leading zero is left alone.
/// Why:
///   `pid 0<number>` names the judged number and is not the bytes the removal judged.
///  It compares the file's contents byte for byte with the contents it judged,
///  and the one case left open is a program that writes the very same bytes back,
///  and the instant between the last read and the unlink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = aFreedNameTaken({ name: 'leading-zero', holder: { otherBytesInPlace: 'leading-zero' }, everyReadReportsChangeTime: 1000 });
/// expect(exists(taken.pidFile)).toBe(true);
/// ```
#[test]
fn a_pid_file_whose_number_gained_a_leading_zero_is_left_alone() {
    let taken: TakenName = a_freed_name_taken(FreedName {
        name: "leading-zero",
        holder: TheNewHolder::OtherBytesInPlace(OtherBytes::LeadingZero),
        every_read_reports_change_time: Some(A_CHANGE_TIME_EVERY_READ_REPORTS),
        every_inode_is: None,
        later: None,
    });
    assert!(
        matches!(taken.ended, Ended::Cleared(_)),
        "the abandoned lock itself is cleared: {:?}",
        taken.ended
    );
    assert!(
        taken.pid_file.exists(),
        "a PID file whose number gained a leading zero is left alone"
    );
    remove(taken.common_dir.as_path());
}

/// What:
///  A PID file with spaces before its newline is left alone.
/// Why:
///   `pid <number>   ` followed by a newline names the judged number and is not the bytes the removal judged.
///  It compares the file's contents byte for byte with the contents it judged,
///  and the one case left open is a program that writes the very same bytes back,
///  and the instant between the last read and the unlink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = aFreedNameTaken({ name: 'trailing-padding', holder: { otherBytesInPlace: 'trailing-padding' }, everyReadReportsChangeTime: 1000 });
/// expect(exists(taken.pidFile)).toBe(true);
/// ```
#[test]
fn a_pid_file_with_trailing_padding_is_left_alone() {
    let taken: TakenName = a_freed_name_taken(FreedName {
        name: "trailing-padding",
        holder: TheNewHolder::OtherBytesInPlace(OtherBytes::TrailingPadding),
        every_read_reports_change_time: Some(A_CHANGE_TIME_EVERY_READ_REPORTS),
        every_inode_is: None,
        later: None,
    });
    assert!(
        matches!(taken.ended, Ended::Cleared(_)),
        "the abandoned lock itself is cleared: {:?}",
        taken.ended
    );
    assert!(
        taken.pid_file.exists(),
        "a PID file with spaces before its newline is left alone"
    );
    remove(taken.common_dir.as_path());
}

/// What:
///  A PID file with a carriage return before its newline is left alone.
/// Why:
///   `pid <number>` and a carriage return names the judged number and is not the bytes the removal judged.
///  It compares the file's contents byte for byte with the contents it judged,
///  and the one case left open is a program that writes the very same bytes back,
///  and the instant between the last read and the unlink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = aFreedNameTaken({ name: 'carriage-return', holder: { otherBytesInPlace: 'carriage-return' }, everyReadReportsChangeTime: 1000 });
/// expect(exists(taken.pidFile)).toBe(true);
/// ```
#[test]
fn a_pid_file_with_a_carriage_return_before_the_newline_is_left_alone() {
    let taken: TakenName = a_freed_name_taken(FreedName {
        name: "carriage-return",
        holder: TheNewHolder::OtherBytesInPlace(OtherBytes::CarriageReturn),
        every_read_reports_change_time: Some(A_CHANGE_TIME_EVERY_READ_REPORTS),
        every_inode_is: None,
        later: None,
    });
    assert!(
        matches!(taken.ended, Ended::Cleared(_)),
        "the abandoned lock itself is cleared: {:?}",
        taken.ended
    );
    assert!(
        taken.pid_file.exists(),
        "a PID file with a carriage return before its newline is left alone"
    );
    remove(taken.common_dir.as_path());
}

/// What:
///  A PID file with no newline is left alone.
/// Why:
///   `pid <number>` and no newline names the judged number and is not the bytes the removal judged.
///  It compares the file's contents byte for byte with the contents it judged,
///  and the one case left open is a program that writes the very same bytes back,
///  and the instant between the last read and the unlink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = aFreedNameTaken({ name: 'no-newline', holder: { otherBytesInPlace: 'no-newline' }, everyReadReportsChangeTime: 1000 });
/// expect(exists(taken.pidFile)).toBe(true);
/// ```
#[test]
fn a_pid_file_with_no_newline_is_left_alone() {
    let taken: TakenName = a_freed_name_taken(FreedName {
        name: "no-newline",
        holder: TheNewHolder::OtherBytesInPlace(OtherBytes::NoNewline),
        every_read_reports_change_time: Some(A_CHANGE_TIME_EVERY_READ_REPORTS),
        every_inode_is: None,
        later: None,
    });
    assert!(
        matches!(taken.ended, Ended::Cleared(_)),
        "the abandoned lock itself is cleared: {:?}",
        taken.ended
    );
    assert!(
        taken.pid_file.exists(),
        "a PID file with no newline is left alone"
    );
    remove(taken.common_dir.as_path());
}

/// What:
///  A PID file of another inode and the judged number is left alone,
///  whatever number the file system gives the holder's new file.
/// Why:
///   The holder removes the judged file and writes the judged number again,
///  so the two files hold the same number and the same device;
///  the arm answers one inode for every read and one change time both reads report,
///  so neither the file system's own number for the new file nor the clock can decide,
///  and the arm's answer for a read taken after the lock's name went is the only difference.
///  With that answer gone the removal compares one inode number with itself and takes the file,
///  which is the term this control exists for.
///  The state is a real replacement of the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = aFreedNameTaken({ name: 'another-inode', holder: 'the-judged-number-in-a-new-file', everyReadReportsChangeTime: 1000, everyInodeIs: 1, later: 'inode' });
/// expect(exists(taken.pidFile)).toBe(true);
/// ```
#[test]
fn a_pid_file_of_another_inode_is_left_alone() {
    let taken: TakenName = a_freed_name_taken(FreedName {
        name: "another-inode",
        holder: TheNewHolder::TheJudgedNumberInANewOne,
        every_read_reports_change_time: Some(A_CHANGE_TIME_EVERY_READ_REPORTS),
        every_inode_is: Some(ONE_INODE_FOR_EVERY_FILE),
        later: Some(crate::lock_metadata_arm::Changed::Inode),
    });
    assert!(
        matches!(taken.ended, Ended::Cleared(_)),
        "the abandoned lock itself is cleared: {:?}",
        taken.ended
    );
    assert!(taken.lock.exists(), "the new holder's lock stands");
    assert!(
        taken.pid_file.exists(),
        "a PID file of another inode is left alone"
    );
    remove(taken.common_dir.as_path());
}

/// What:
///  A PID file whose change time reads another millisecond is left alone,
///  and nothing else about it differs.
/// Why:
///   No fixture can make a file's change time differ by a whole millisecond on request:
///  this host's ZFS stamps two writes in place with the same number in 957 of 1000 pairs
///  (the integration pass of 2026-11-22,
///  `doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
///  cause L),
///  so a control that waited for it would be the lottery issue 6342 was filed about.
///  The arm answers the later read with the change time one millisecond back,
///  and the file itself is the judged one,
///  so the term is the only difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = aFreedNameTaken({ name: 'another-change-time', holder: 'no-pid-file', later: 'change-time' });
/// expect(exists(taken.pidFile)).toBe(true);
/// ```
#[test]
fn a_pid_file_of_another_change_time_is_left_alone() {
    let taken: TakenName = a_freed_name_taken(FreedName {
        name: "another-change-time",
        holder: TheNewHolder::WritesNone,
        every_read_reports_change_time: None,
        every_inode_is: None,
        later: Some(crate::lock_metadata_arm::Changed::ChangeTime),
    });
    assert!(
        matches!(taken.ended, Ended::Cleared(_)),
        "the abandoned lock itself is cleared: {:?}",
        taken.ended
    );
    assert!(
        taken.pid_file.exists(),
        "a PID file whose change time reads another millisecond is left alone"
    );
    remove(taken.common_dir.as_path());
}

/// What:
///  A PID file whose device reads another is left alone,
///  and nothing else about it differs.
/// Why:
///   A fixture has one mount,
///  so no file it makes can be on another device,
///  and the arm answers the later read with the next device number.
///  The file is the judged one and its inode,
///  change time and bytes are the ones judged,
///  so the device is the only difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = aFreedNameTaken({ name: 'another-device', holder: 'no-pid-file', later: 'device' });
/// expect(exists(taken.pidFile)).toBe(true);
/// ```
#[test]
fn a_pid_file_of_another_device_is_left_alone() {
    let taken: TakenName = a_freed_name_taken(FreedName {
        name: "another-device",
        holder: TheNewHolder::WritesNone,
        every_read_reports_change_time: None,
        every_inode_is: None,
        later: Some(crate::lock_metadata_arm::Changed::Device),
    });
    assert!(
        matches!(taken.ended, Ended::Cleared(_)),
        "the abandoned lock itself is cleared: {:?}",
        taken.ended
    );
    assert!(
        taken.pid_file.exists(),
        "a PID file of another device is left alone"
    );
    remove(taken.common_dir.as_path());
}

/// What:
///  A PID file a holder rewrote in place with the very bytes it was judged with
///  goes with the lock,
///  and what that costs.
/// Why:
///   Four facts cannot tell this file from the judged one:
///  the same device,
///  the same inode (it is the judged file itself,
/// rewritten in place),
///  the same change time (the arm reports one value for both reads,
/// so no clock decides),
///  and the same contents,
///  so the removal takes it.
///  This is the one state the comparison of the contents leaves open,
///  and it is why the round was told to state it.
///  What that costs is the PID file of a program that wrote the bytes the record proves gone,
///  which means the program that named itself with the dead holder's number;
///  its lock stands,
///  and the file it lost named a process this boot has not.
///  It compares the file's contents byte for byte with the contents it judged,
///  and the one case left open is a program that writes the very same bytes back,
///  and the instant between the last read and the unlink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = aFreedNameTaken({ name: 'the-same-bytes', holder: 'the-judged-number-over-the-judged-file', everyReadReportsChangeTime: 1000 });
/// expect(exists(taken.pidFile)).toBe(false);
/// ```
#[test]
fn a_pid_file_with_the_very_bytes_it_was_judged_with_goes_with_the_lock() {
    let taken: TakenName = a_freed_name_taken(FreedName {
        name: "the-same-bytes",
        holder: TheNewHolder::TheJudgedNumberInPlace,
        every_read_reports_change_time: Some(A_CHANGE_TIME_EVERY_READ_REPORTS),
        every_inode_is: None,
        later: None,
    });
    let Ended::Cleared(clearance) = taken.ended else {
        panic!("the abandoned lock itself is cleared: {:?}", taken.ended);
    };
    assert_eq!(clearance.pid, taken.judged_pid);
    assert!(
        !taken.pid_file.exists(),
        "a PID file with the very bytes it was judged with goes with the lock"
    );
    remove(taken.common_dir.as_path());
}

/// What:
///  The arm a control sets is put back when that control's own body panics.
/// Why:
///   A control that left an arm set would change every later control of the same thread,
///  and a panic inside the body is the way out that has no return statement.
///  `with_arm` holds a `PutBackWhenDropped`,
///  so the arm goes back on the way out of an unwind as it does on a return.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(() => withArm(arm, () => { throw new Error('a control failed'); })).toThrow(); expect(aReadIsUnarmed()).toBe(true);
/// ```
#[test]
fn an_arm_is_put_back_when_the_control_panics() {
    let root: PathBuf = a_git_directory("arm-put-back");
    let file: PathBuf = root.join("index~pid.lock");
    std::fs::write(file.as_path(), b"pid 1\n").expect("the file");
    let recorded: crate::index_lock_evidence::LockMetadata =
        crate::index_lock_evidence::read_lock_metadata(file.as_path())
            .ok()
            .flatten()
            .expect("the file's metadata");
    let judged_change_time: i64 = recorded.ctime_ms;
    let arm: crate::lock_metadata_arm::MetadataArm = crate::lock_metadata_arm::MetadataArm {
        path: file.clone(),
        every_read_reports_change_time: Some(A_CHANGE_TIME_EVERY_READ_REPORTS),
        every_inode_is: Some(ONE_INODE_FOR_EVERY_FILE),
        later: None,
    };
    let panicked: Result<(), Box<dyn std::any::Any + Send>> =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::lock_metadata_arm::with_arm(arm, || {
                panic!("a control's own failure");
            });
        }));
    assert!(
        panicked.is_err(),
        "the body must panic for this control to hold anything"
    );
    let afterwards: Option<crate::index_lock_evidence::LockMetadata> =
        crate::lock_metadata_arm::armed(file.as_path(), Some(recorded));
    assert_eq!(
        afterwards.map(|found| {
            return found.ctime_ms;
        }),
        Some(judged_change_time),
        "the arm is not set any more, or the next control of this thread would read armed metadata"
    );
    remove(root.as_path());
}

/// What:
///  A Git that takes the freed name while the abandoned holder's PID file still stands
///  writes no PID file of its own,
///  and the removal takes the abandoned one with its lock and leaves the new lock alone.
/// Why:
///   This is the state the two unlinks meet in a repository:
///  `create_lock_pid_file` creates `<name>~pid.lock` with `O_EXCL` and writes nothing when the name is taken
///  (`lockfile.c`,
///  Git 2.56.1),
///  so the file that stands at that name is the very one the removal judged,
///  the live holder has no file there that a removal could take,
///  and what it owns is its lock,
///  which stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hooks.set(UNLINKED_THE_LOCK, () => { writeTheNewLock(); try { writeFileSync(pidFile, text, { flag: 'wx' }); } catch {} });
/// expect(exists(lock)).toBe(true); expect(exists(pidFile)).toBe(false);
/// ```
#[test]
fn a_git_that_takes_the_freed_name_writes_no_pid_file_and_keeps_its_lock() {
    let taken: TakenName = a_freed_name_taken(FreedName {
        name: "taken-name-exclusive",
        holder: TheNewHolder::WritesNone,
        every_read_reports_change_time: None,
        every_inode_is: None,
        later: None,
    });
    let Ended::Cleared(clearance) = taken.ended else {
        panic!("the abandoned lock itself is cleared: {:?}", taken.ended);
    };
    assert_eq!(clearance.pid, taken.judged_pid);
    assert!(taken.lock.exists(), "the new holder's lock stands");
    assert!(
        !taken.pid_file.exists(),
        "the abandoned holder's PID file goes with its lock, and the new holder has none to lose"
    );
    remove(taken.common_dir.as_path());
}

/// Wait for the child to hold `lock` open,
/// bounded,
/// because the child must have it before the proof is asked.
fn wait_until_open(
    lock: &Path,
    pid: i64,
) -> bool {
    let opened: PathBuf = PathBuf::from(format!("/proc/{pid}/fd"));
    let began: std::time::Instant = std::time::Instant::now();
    while began.elapsed() < std::time::Duration::from_secs(10) {
        if let Ok(entries) = std::fs::read_dir(opened.as_path()) {
            for entry in entries.flatten() {
                if std::fs::read_link(entry.path()).is_ok_and(|target| {
                    return target == lock;
                }) {
                    return true;
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    return false;
}
