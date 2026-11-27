//! What:
//! The controls of what a hidden name's boot and its process record decide:
//! a record `/proc` hides,
//! a name written by a run that could not read its own boot,
//! and a name of another boot that carries no start time.
//! Why:
//! Each decides whether a leftover is ever reclaimed,
//! and none was held:
//! the first was answered by a forced "unreadable" that returns before the code asks the kernel at all,
//! the second writes the shorter name an earlier build wrote,
//! which no later run judges,
//! and the third was read as "the system did not say" although no process survives a reboot.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-liveness-of-a-hidden-name-tests.ts
//! test('a record a proc hides is never read as an installer that ended');
//! test('a run that cannot read its boot writes a name no later run reclaims');
//! test('a name of another boot is ended whatever its start time says');
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports the owned path type this file's fixtures are built from (not `&Path`).
use std::path::PathBuf;

/// Imports the three purposes and the answer of a recovery,
/// the hidden names of a version directory,
/// and the step that reads them.
use super::super::archive_staging::{
    AnOlderVersion,
    FinishedCopy,
    Liveness,
    Owner,
    STAGING_PURPOSE,
    hidden_sibling,
    hidden_siblings,
    recover_leftovers,
    staging_dir_of,
    write_complete_marker,
};
/// Imports the reader facts a control forces,
/// and the two seams that force them.
use super::super::archive_staging::{
    ReaderView,
    read_as,
    read_as_a_proc_that_hides_records,
    start_time_of_a_process,
};
/// Imports the tool directory of a tool id.
use super::super::layout::tool_directory;
/// Imports the question that tells a finished install from an unfinished one.
use super::super::published_archive::is_finished;
/// Imports the text of a path's own last name.
use super::super::util::file_name_text;
/// Imports the fixture's tool id,
/// its program,
/// and the archive digest a marker records.
use super::{
    BIN,
    ID,
    release_digest,
};
/// Imports the rig,
/// the names a tool directory holds,
/// the writer of an executable stand-in,
/// and a program a control keeps running while it judges which installer still runs.
use super::{
    Rig,
    RunningProgram,
    names_of,
    write_executable,
};
/// Imports the file that marks a copy an installer was unpacking when it was killed,
/// and the record a lock holds for one copy.
use super::{
    WHAT_WAS_BEING_UNPACKED,
    the_locks_record,
};

/// What:
/// A boot identifier no run on this machine can have,
/// in the UUID form the kernel writes one.
/// Why:
/// A name of another boot is the one judgement the boot alone decides,
/// and no control can make this machine reboot;
/// the value stands for no program's output and matters only as a boot that is not this one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_BOOT_THIS_MACHINE_NEVER_HAD = '11111111-1111-1111-1111-111111111111';
/// ```
const A_BOOT_THIS_MACHINE_NEVER_HAD: &str = "11111111-1111-1111-1111-111111111111";

/// What:
/// A live program whose record `/proc` reports,
/// read through a `/proc` that holds no record at all.
/// Why:
/// A `hidepid` mount hides a live installer's record,
/// and reading that as an ended installer removes the directory a running install is publishing from;
/// the answer must come from the kernel's own question about the number,
/// never from the absence of the record alone.
/// The seam answers as such a mount does for every number,
/// so the process the control keeps running is the one whose record is hidden,
/// which is the state the machine cannot be put into without a mount.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('a record a proc hides is never read as an installer that ended', () => {
///   const running = RunningProgram.start();
///   const live = running.owner();
///   using hidden = readAsAProcThatHidesRecords();
///   expect(live.liveness()).toBe('unknown');
/// });
/// ```
#[test]
fn a_record_a_proc_hides_is_never_read_as_an_installer_that_ended() {
    // What: a program this test keeps running, and the owner whose name a hidden sibling would carry.
    // Why: the answer depends on the number being held by a live process, so the process has to exist.
    let running: RunningProgram = RunningProgram::start();
    let live: Owner = running.owner();
    assert_eq!(
        live.liveness(),
        Liveness::Running,
        "the program this control keeps running is reported as running while its record is there"
    );

    // What: a `/proc` that holds no record for any number, as a `hidepid` mount leaves a live process's record.
    // Why: the arm that answers "an unreadable record is not a dead process" is only reached when the read fails
    //  without the number being free.
    let _hidden = read_as_a_proc_that_hides_records();
    assert_eq!(
        live.liveness(),
        Liveness::Unknown,
        "a hidden record is not a dead process: the kernel holds the number, so nothing may be removed"
    );
}

/// What:
/// The name one run writes for its staging directory when it cannot read the boot's identifier,
/// and what a later run does with that name.
/// Why:
/// The name is the whole record of which installer made a hidden directory,
/// so a name that carries one fact fewer is a directory no later run may judge:
/// this run's own copy of a kill would then stay for ever,
/// and a person who deletes it by hand loses the finished copy it may hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('a run that cannot read its boot writes a name no later run reclaims', () => {
///   const staging = withNoBootAnswer(() => stagingDirOf(versionDir));
///   expect(basename(staging)).toBe(`.0.2.1.staging.${process.pid}.${startTimeOf(process.pid)}`);
///   const recovered = recoverLeftovers(versionDir, BIN, 'may-stay', theLocksRecord('0.2.1'));
///   expect(recovered.removed).toEqual([]);
/// });
/// ```
#[test]
fn a_run_that_cannot_read_its_boot_writes_a_name_no_later_run_reclaims() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    // The boot's identifier answers nothing, which is the state a sandbox that hides `/proc/sys` leaves.
    let staging: PathBuf = {
        let _view = read_as(ReaderView {
            boot: Some(None),
            ..ReaderView::default()
        });
        staging_dir_of(&version_dir)
    };
    // The name such a run writes: the purpose, the process id and its start time, and no namespace or boot.
    let started: u64 = start_time_of_a_process(std::process::id())
        .expect("the system reports this process's start");
    let expected: String = format!(".0.2.1.{STAGING_PURPOSE}.{}.{started}", std::process::id());
    assert_eq!(
        file_name_text(&staging),
        expected,
        "a run that cannot read its boot writes the earlier build's shorter name"
    );

    // What: an unfinished copy under that name, which is what a kill during the unpacking leaves.
    // Why: an unfinished copy is the one a later run may remove, so the name's liveness decides whether it stays.
    fs::create_dir_all(staging.join("release-folder")).expect("make the unfinished copy");
    write_executable(&staging.join("release-folder").join(BIN));
    fs::write(staging.join(WHAT_WAS_BEING_UNPACKED), "half a release").expect("write");

    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    )
    .expect("the leftovers are read");
    assert!(
        recovered.removed.is_empty(),
        "a name that carries no boot and no namespace is never judged, so nothing is removed: {recovered:?}"
    );
    assert!(
        staging.is_dir(),
        "the copy waits where it is: {:?}",
        names_of(&tool_dir)
    );
    let found = hidden_siblings(&tool_dir).expect("list the hidden names");
    let leftover = found
        .iter()
        .find(|leftover| return leftover.path == staging)
        .expect("the name is still read as one of this tool's");
    assert_eq!(
        leftover.liveness(),
        Liveness::Unknown,
        "the shorter name carries no boot, so the reader cannot place the installer in any boot"
    );

    // What: the marker an install writes last, which makes the copy under that name a finished one.
    // Why: a finished copy under a name no run may judge must not become the version either, or the README's sentence
    //  would hold for the unfinished copy alone.
    write_complete_marker(&staging, ID, "0.2.1", &release_digest("0.2.1"))
        .expect("write the completion marker");
    let again = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    )
    .expect("the leftovers are read");
    assert!(
        again.finished_copy == FinishedCopy::NothingToPublish,
        "a finished copy under a name no run may judge is never published: {again:?}"
    );
    assert!(
        !is_finished(&version_dir, BIN),
        "the version's name stays empty"
    );
    assert!(
        staging.is_dir(),
        "and the finished copy waits where it is: {:?}",
        names_of(&tool_dir)
    );
}

/// What:
/// A name of another boot that carries no start time:
/// its own liveness,
/// and what a later run does with the directory under it.
/// Why:
/// No process survives a reboot,
/// so the boot alone says that installer ended,
/// whatever the start time field holds;
/// a guard that read a start time of zero first kept such a name for ever,
/// and the README's sentence says the boot decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('a name of another boot is ended whatever its start time says', () => {
///   const elsewhere = { pid: process.pid, started: 0, boot: A_BOOT_THIS_MACHINE_NEVER_HAD, namespace: here.namespace };
///   expect(elsewhere.liveness()).toBe('gone');
///   const recovered = recoverLeftovers(versionDir, BIN, 'may-stay', theLocksRecord('0.2.1'));
///   expect(recovered.removed).toContain(hiddenSibling(versionDir, STAGING_PURPOSE, elsewhere));
/// });
/// ```
#[test]
fn a_name_of_another_boot_is_ended_whatever_its_start_time_says() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = tool_directory(&root, ID).expect("a plain name");
    let version_dir: PathBuf = tool_dir.join("0.2.1");
    let here: Owner = Owner::here();
    // A name of another boot, this namespace and this process id, with no start time:
    // a run that could not read its own record writes the zero, and the next boot reads the name.
    let elsewhere: Owner = Owner {
        pid: std::process::id(),
        started: 0,
        boot: Some(A_BOOT_THIS_MACHINE_NEVER_HAD.to_string()),
        namespace: here.namespace,
    };
    assert_eq!(
        elsewhere.liveness(),
        Liveness::Gone,
        "a name from another boot names an installer that reboot certainly ended"
    );

    // What: an unfinished copy under that name, which the reader now knows no installer holds.
    // Why: the liveness answer is only worth having if the removal follows it.
    let leftover: PathBuf = hidden_sibling(&version_dir, STAGING_PURPOSE, elsewhere);
    fs::create_dir_all(leftover.join("release-folder")).expect("make the unfinished copy");
    write_executable(&leftover.join("release-folder").join(BIN));
    fs::write(leftover.join(WHAT_WAS_BEING_UNPACKED), "half a release").expect("write");

    let recovered = recover_leftovers(
        &version_dir,
        BIN,
        AnOlderVersion::MayStay,
        Some(&the_locks_record("0.2.1")),
    )
    .expect("the leftovers are read");
    assert!(
        recovered
            .removed
            .iter()
            .any(|removed| return removed.path == leftover),
        "the directory of an installer of another boot is removed: {recovered:?}"
    );
    assert!(
        !leftover.exists(),
        "nothing of that name stays: {:?}",
        names_of(&tool_dir)
    );
}
