//! What:
//!  Controls of the records an interrupted copy leaves that cannot be read:
//!  each is moved aside,
//!  never deleted,
//!  never over another record,
//!  and the writer is judged before anything moves.
//! Why:
//!  The harm the pass prevents is one record that stops every command of the repository,
//!  so the controls pin what becomes of the record and of what it named,
//!  and that a record whose writer is gone is the only one acted on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const notices = await settleUnreadableRecords({ commonDir, verdicts }); expect(moved).toBe(present);
//! ```

use crate::worktree_copy_journal::{journal_root, ensure_journal_root};
use crate::worktree_copy_journal_encoding_and_validation::{Journal, PendingJournal, Phase};
use crate::worktree_copy_owner::{CopyOwner, copy_owner_of_process};
use crate::worktree_copy_stage_claim::{claim_stage_for_add_as, claim_names};
use crate::worktree_copy_stage_claim_reaping::StageVerdicts;
use crate::worktree_copy_unreadable_records::{UNREADABLE_FOLDER, move_aside, moved_record_path, settle_unreadable_records, unreadable_record_names, unreadable_records_stand};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// A directory under the test's own scratch space,
/// private to this account like every item of a copy.
fn scratch(name: &str) -> PathBuf {
    let base: PathBuf = crate::test_support::fixture_with_prefix("unreadable", name);
    std::fs::remove_dir_all(base.join("common").as_path()).ok();
    std::fs::create_dir_all(base.join("common").as_path()).expect("common directory");
    std::fs::create_dir_all(base.join("worktrees").as_path()).expect("worktree parents");
    std::fs::set_permissions(base.join("common").as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private common directory");
    std::fs::set_permissions(base.join("worktrees").as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private worktree parent");
    return std::fs::canonicalize(base).expect("canonical scratch");
}

/// The owner of a process that really started,
/// had its birth identity read while it ran,
/// and was then killed and reaped:
/// what the records of an interrupted copy carry.
fn dead_owner() -> CopyOwner {
    let mut child: std::process::Child = std::process::Command::new("sleep").arg("60").spawn().expect("a child that runs");
    let pid: i64 = i64::from(child.id());
    let birth: String = crate::process_identity::process_birth_identity(pid).expect("identity readable").expect("the child runs");
    child.kill().expect("the child is killed");
    child.wait().expect("the child is reaped");
    return CopyOwner { pid, birth_identity: birth, pid_namespace: crate::pid_namespace::current_pid_namespace() };
}

/// One journal record of a copy into `destination`,
/// written by `owner`.
fn write_journal(common: &Path, destination: &Path, owner: &CopyOwner, body: &[u8]) {
    let root: PathBuf = ensure_journal_root(common).expect("journal root");
    let container: PathBuf = destination.parent().expect("a destination parent").join(format!(".cli-git-worktree-copy-{}", owner.token()));
    std::fs::create_dir_all(container.join("payload").as_path()).expect("stage container");
    std::fs::set_permissions(container.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private stage");
    let journal: Journal = Journal {
        created_entries: Vec::new(),
        intended_entries: Vec::new(),
        destination_root: destination.to_path_buf(),
        phase: Phase::Staged,
        selected_roots: Vec::new(),
        source_root: destination.parent().expect("a destination parent").join("source"),
        stage_container: container.clone(),
        stage_root: container.join("payload"),
    };
    let _pending: PendingJournal = PendingJournal { path: root.join(format!("{}.{}.json", "8f14e45f", owner.token())), journal };
    std::fs::write(_pending.path.as_path(), body).expect("the journal's bytes");
    std::fs::set_permissions(_pending.path.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o600)).expect("private journal");
}

/// Every journal file standing under the root,
/// so a control can say what became of one.
fn journals(common: &Path) -> Vec<String> {
    let root: PathBuf = journal_root(common);
    let mut names: Vec<String> = Vec::new();
    for listed in std::fs::read_dir(root.as_path()).expect("journal root") {
        let name: String = listed.expect("an entry").file_name().to_string_lossy().into_owned();
        if name.ends_with(".json") {
            names.push(name);
        }
    }
    names.sort();
    return names;
}

/// An empty journal of a writer that ended is moved aside rather than deleted,
/// and the folder beside the records keeps it under its own name.
#[test]
fn an_empty_journal_of_a_dead_writer_is_moved_aside_and_kept() {
    let base: PathBuf = scratch("journal-empty");
    let common: PathBuf = base.join("common");
    let destination: PathBuf = base.join("worktrees/two");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    write_journal(common.as_path(), destination.as_path(), &dead_owner(), b"");
    let mut verdicts: StageVerdicts = StageVerdicts::default();

    let notices: Vec<String> = settle_unreadable_records(common.as_path(), &mut verdicts).expect("settled");

    assert_eq!(journals(common.as_path()), Vec::<String>::new(), "no unreadable journal stands where every command reads");
    assert!(unreadable_records_stand(common.as_path()), "the record is kept");
    let kept: Vec<String> = unreadable_record_names(common.as_path()).expect("names");
    assert_eq!(kept.len(), 1, "{kept:?}");
    assert!(kept[0].ends_with(".json"), "{kept:?}");
    assert_eq!(notices.len(), 1, "{notices:?}");
    assert!(notices[0].contains("could not be read"), "{}", notices[0]);
    assert!(notices[0].contains(UNREADABLE_FOLDER), "the line says where it went: {}", notices[0]);
    assert!(
        notices[0].contains("stage folder"),
        "the line says what may be left on disk: {}",
        notices[0]
    );
    assert_eq!(std::fs::read(moved_record_path(common.as_path(), OsStr::new(kept[0].as_str()))).expect("the kept record"), b"");
}

/// A record whose writer still runs is that writer's:
/// it is left whole,
/// and the command that found it changes nothing.
#[test]
fn a_record_of_a_live_writer_is_left_alone() {
    let base: PathBuf = scratch("journal-live");
    let common: PathBuf = base.join("common");
    let destination: PathBuf = base.join("worktrees/two");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    write_journal(common.as_path(), destination.as_path(), &copy_owner_of_process(std::process::id()).expect("this process's owner"), b"");
    let mut verdicts: StageVerdicts = StageVerdicts::default();

    let notices: Vec<String> = settle_unreadable_records(common.as_path(), &mut verdicts).expect("settled");

    assert_eq!(notices, Vec::<String>::new(), "a running copy's record is not settled");
    assert_eq!(journals(common.as_path()).len(), 1, "the record stands where its writer left it");
    assert!(!unreadable_records_stand(common.as_path()), "nothing was moved");
}

/// A moved record never replaces one an earlier command kept:
/// the second one gets a name of its own and both keep their bytes.
#[test]
fn a_second_record_of_one_name_never_replaces_the_first() {
    let base: PathBuf = scratch("two-records");
    let common: PathBuf = base.join("common");
    let first: PathBuf = base.join("first.json");
    let second: PathBuf = base.join("second.json");
    std::fs::write(first.as_path(), b"first\n").expect("first");
    std::fs::write(second.as_path(), b"second\n").expect("second");

    let moved_first: PathBuf = move_aside(common.as_path(), first.as_path(), OsStr::new("one.json")).expect("moved").expect("named");
    let moved_second: PathBuf = move_aside(common.as_path(), second.as_path(), OsStr::new("one.json")).expect("moved").expect("named");

    assert_ne!(moved_first, moved_second, "the second record is kept under a name of its own");
    assert_eq!(std::fs::read(moved_first).expect("the first record"), b"first\n");
    assert_eq!(std::fs::read(moved_second).expect("the second record"), b"second\n");
    assert!(!first.exists() && !second.exists(), "no record is left where the readers look");
}

/// A record another command settled first is nobody's failure.
#[test]
fn a_record_that_vanished_is_not_an_error() {
    let base: PathBuf = scratch("vanished");
    let common: PathBuf = base.join("common");

    let moved: Option<PathBuf> = move_aside(common.as_path(), base.join("gone.json").as_path(), OsStr::new("gone.json")).expect("settled");

    assert_eq!(moved, None, "nothing was there to move");
}

/// An unreadable claim whose stage folder the record beside it names is written again,
/// so the sweep that makes the copy an add still owes has a claim to read.
#[test]
fn a_claim_whose_folder_is_named_beside_it_is_written_again() {
    let base: PathBuf = scratch("claim-rebuilt");
    let common: PathBuf = base.join("common");
    let destination: PathBuf = base.join("worktrees/two");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    let claim: crate::worktree_copy_stage_claim::StageClaim =
        claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_owner()).expect("an add's claim");
    let claim_path: PathBuf = journal_root(common.as_path()).join(format!("{}.stage", claim.identifier()));
    std::fs::write(claim_path.as_path(), b"").expect("the claim loses what it said");
    std::fs::set_permissions(claim_path.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o600)).expect("private claim");
    let mut verdicts: StageVerdicts = StageVerdicts::default();

    let notices: Vec<String> = settle_unreadable_records(common.as_path(), &mut verdicts).expect("settled");

    let names: Vec<String> = claim_names(journal_root(common.as_path()).as_path()).expect("claims");
    assert_eq!(names.len(), 1, "a claim stands again under its own name: {names:?}");
    let recovered: Option<(PathBuf, Option<String>)> = crate::worktree_copy_stage_claim_reaping::read_claim(claim_path.as_path()).expect("read");
    let (container, owner) = recovered.expect("the claim reads again");
    assert_eq!(container, claim.container().to_path_buf(), "it names the folder the add claimed");
    assert_eq!(owner, None, "a claim written again names no owner, so nothing waits for one");
    assert!(verdicts.names_dead_claim(names[0].as_str()), "this pass settles it as a dead owner's claim");
    assert!(notices[0].contains("written again"), "{}", notices[0]);
}

/// The claim file one claim was written to.
/// Why a control needs it:
///  the field is the claim's own,
///  and a control damages and removes the file at that name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claimPathOf = (claim) => join(journalRoot(common), `${claim.identifier()}.stage`);
/// ```
fn claim_path_of(common: &Path, claim: &crate::worktree_copy_stage_claim::StageClaim) -> PathBuf {
    return journal_root(common).join(format!("{}.stage", claim.identifier()));
}

/// What:
///  Every event the kernel reported for one file while `run` took its course.
/// Why:
///   The claim's own name must never be absent while a command settles it,
///  and the end state is the same whether the name was taken away and written again or never moved,
///  so only the events of the call tell the two apart:
///  a rename of the watched file reports `IN_MOVE_SELF`,
///  a link to it and a rename over it report its link count changing (`IN_ATTRIB`).
///  The watch is read once the call is done,
///  because the kernel queues the events whether or not anybody reads them in between.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const masks = watchEvents(claimPath, () => settleUnreadableRecords(common, verdicts));
/// ```
#[cfg(target_os = "linux")]
fn watch_events<F: FnOnce()>(file: &Path, run: F) -> Vec<u32> {
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    use std::os::unix::ffi::OsStrExt;
    // SAFETY: the call takes no arguments and answers a new descriptor or a negative number.
    let raw: libc::c_int = unsafe { libc::inotify_init1(libc::IN_NONBLOCK | libc::IN_CLOEXEC) };
    assert!(raw >= 0, "a watch is opened: {}", std::io::Error::last_os_error());
    // SAFETY: `raw` is a descriptor this call alone owns, and it is closed when `fd` drops.
    let fd: OwnedFd = unsafe { OwnedFd::from_raw_fd(raw) };
    let name: std::ffi::CString = std::ffi::CString::new(file.as_os_str().as_bytes()).expect("no NUL in a fixture path");
    // SAFETY: `name` is a NUL-terminated path that outlives the call, and the descriptor is open.
    let watch: libc::c_int = unsafe { libc::inotify_add_watch(fd.as_raw_fd(), name.as_ptr(), libc::IN_MOVE_SELF | libc::IN_DELETE_SELF | libc::IN_ATTRIB) };
    assert!(watch >= 0, "the file is watched: {}", std::io::Error::last_os_error());
    run();
    let mut masks: Vec<u32> = Vec::new();
    let mut buffer: [u8; 4096] = [0; 4096];
    loop {
        // SAFETY: the descriptor is open, and the buffer is the length handed to the call.
        let read: libc::ssize_t = unsafe { libc::read(fd.as_raw_fd(), buffer.as_mut_ptr().cast(), buffer.len()) };
        if read <= 0 {
            break;
        }
        let mut at: usize = 0;
        while at + 16 <= read as usize {
            let mask: [u8; 4] = buffer[at + 4..at + 8].try_into().expect("four bytes of a mask");
            let name_length: [u8; 4] = buffer[at + 12..at + 16].try_into().expect("four bytes of a name length");
            masks.push(u32::from_ne_bytes(mask));
            at += 16 + u32::from_ne_bytes(name_length) as usize;
        }
    }
    return masks;
}

/// Whether the watched file was taken away from its own name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const movedAway = (masks) => masks.some((mask) => (mask & (IN_MOVE_SELF | IN_DELETE_SELF)) !== 0);
/// ```
#[cfg(target_os = "linux")]
fn was_taken_away(masks: &[u32]) -> bool {
    return masks.iter().any(|mask: &u32| { return mask & (libc::IN_MOVE_SELF | libc::IN_DELETE_SELF) != 0; });
}

/// An unreadable claim is settled without its own name ever being taken away,
/// so a command that reaps in the instant between the two steps never reads the stage as one no add stands behind.
/// Why the control reads this way:
///  the end state is the same either way,
///  and only the events of the call tell the two apart,
///  so the positive control below watches a claim whose folder nothing names being moved,
/// which the same watch sees.
#[cfg(target_os = "linux")]
#[test]
fn settling_an_unreadable_claim_never_takes_its_own_name_away() {
    let base: PathBuf = scratch("claim-name-kept");
    let common: PathBuf = base.join("common");
    let destination: PathBuf = base.join("worktrees/two");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    let claim: crate::worktree_copy_stage_claim::StageClaim =
        claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_owner()).expect("an add's claim");
    let claim_path: PathBuf = claim_path_of(common.as_path(), &claim);
    // The worst damage this pass handles: a crash that published the rename and none of the bytes.
    std::fs::write(claim_path.as_path(), b"").expect("the claim loses what it said");
    std::fs::set_permissions(claim_path.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o600)).expect("private claim");
    let mut verdicts: StageVerdicts = StageVerdicts::default();
    let masks: Vec<u32> = watch_events(claim_path.as_path(), || {
        settle_unreadable_records(common.as_path(), &mut verdicts).expect("settled");
    });
    assert!(!was_taken_away(masks.as_slice()), "the claim's name was never taken away: {masks:?}");
    // The positive control: a claim whose folder nothing beside it names is moved, and this watch sees that.
    let other: PathBuf = scratch("claim-name-lost");
    let other_common: PathBuf = other.join("common");
    let other_destination: PathBuf = other.join("worktrees/three");
    std::fs::create_dir_all(other_destination.as_path()).expect("destination");
    let lost: crate::worktree_copy_stage_claim::StageClaim = crate::worktree_copy_stage_claim::claim_stage_for(other_common.as_path(), other_destination.as_path(), &dead_owner()).expect("a copy's claim");
    let lost_path: PathBuf = claim_path_of(other_common.as_path(), &lost);
    std::fs::write(lost_path.as_path(), b"").expect("the claim loses what it said");
    std::fs::set_permissions(lost_path.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o600)).expect("private claim");
    let mut other_verdicts: StageVerdicts = StageVerdicts::default();
    let seen: Vec<u32> = watch_events(lost_path.as_path(), || {
        settle_unreadable_records(other_common.as_path(), &mut other_verdicts).expect("settled");
    });
    assert!(was_taken_away(seen.as_slice()), "the watch sees a claim that is moved aside, so the control above could have seen one: {seen:?}");
    std::fs::remove_dir_all(base.as_path()).expect("the fixture");
    std::fs::remove_dir_all(other.as_path()).expect("the fixture");
}

/// The variable that marks the child process that reaps one journal root.
const CHILD_VARIABLE: &str = "CLI_GIT_REAP_CHILD";

/// The variable that names the common directory whose journal root the child reaps.
const CHILD_COMMON_VARIABLE: &str = "CLI_GIT_REAP_CHILD_COMMON";

/// The body the parent starts in a child process:
/// it reaps the journal root of the common directory its environment names,
/// and prints every line it settled.
/// Run without the child's variable it returns at once and checks nothing.
#[test]
fn child_reaps_the_journal_root_its_environment_names() {
    if std::env::var_os(CHILD_VARIABLE).is_none() {
        return;
    }
    let common: PathBuf = PathBuf::from(std::env::var_os(CHILD_COMMON_VARIABLE).expect("the child is given a common directory"));
    let notices: Vec<String> = crate::worktree_copy_stage_claim_reaping::reap_abandoned_stages(common.as_path()).expect("the reap");
    for notice in notices {
        println!("NOTICE {notice}");
    }
}

/// The notices a second process's reap of `common` printed.
/// Why it says so when no test ran:
///  a name that matches no test ends with success and runs nothing,
/// and a control would pass on that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = reapInAChild(common); // throws unless the child ran its test
/// ```
fn reap_in_a_child(common: &Path) -> String {
    let module: &str = module_path!();
    let path: String = format!(
        "{}::child_reaps_the_journal_root_its_environment_names",
        module.split_once("::").map_or(module, |(_, rest)| { return rest; })
    );
    let output: std::process::Output = std::process::Command::new(crate::test_support::this_test_binary().expect("the test binary"))
        .args(["--exact", path.as_str(), "--nocapture", "--test-threads=1"])
        .env(CHILD_VARIABLE, "1")
        .env(CHILD_COMMON_VARIABLE, common)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the child process runs the reap");
    let said: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    let heard: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert!(output.status.success(), "the child failed:\n{said}\n{heard}");
    assert!(said.contains("1 passed"), "the child ran no reap:\n{said}\n{heard}");
    return said;
}

/// A record of a stage's object whose claim is gone is kept while its stage still stands,
/// so a command that reaps in the instant between a claim being taken away and written again never removes the only
/// proof of a folder a person's file may be in.
/// Why the control reads this way:
///  the sweeps that run in every command take an absent claim to mean no add stands behind the records beside it,
///  and the record they would delete is the only thing that says the folder is the wrapper's.
#[test]
fn a_reap_in_another_process_keeps_the_record_of_a_stage_that_still_stands() {
    let base: PathBuf = scratch("record-kept-with-its-stage");
    let common: PathBuf = base.join("common");
    let destination: PathBuf = base.join("worktrees/four");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    let claim: crate::worktree_copy_stage_claim::StageClaim =
        claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_owner()).expect("an add's claim");
    let container: PathBuf = claim.container().to_path_buf();
    std::fs::create_dir_all(container.join("payload").as_path()).expect("the stage");
    std::fs::set_permissions(container.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private stage");
    std::fs::write(container.join("precious.txt"), b"a person's only copy").expect("a person's file");
    crate::worktree_copy_stage_made_object::record_creating_at(common.as_path(), container.as_path(), std::time::SystemTime::now(), None)
        .expect("the record of the creation");
    let claim_path: PathBuf = claim_path_of(common.as_path(), &claim);
    let made: PathBuf = crate::worktree_copy_stage_made_object::made_record_beside_claim(claim_path.as_path());
    // The claim is gone: the instant between another command's two steps of settling a claim it could not read.
    std::fs::remove_file(claim_path.as_path()).expect("the claim is taken away");
    let said: String = reap_in_a_child(common.as_path());
    assert!(made.exists(), "the record of the object the stage's creation made stands while its stage stands: {said}");
    assert!(container.join("precious.txt").exists(), "nothing of the person's was removed: {said}");
    // Once the folder is gone the record proves nothing, and the next sweep removes it.
    std::fs::remove_dir_all(container.as_path()).expect("the control's stage");
    let again: String = reap_in_a_child(common.as_path());
    assert!(!made.exists(), "a record whose folder is gone is swept: {again}");
    std::fs::remove_dir_all(base.as_path()).expect("the fixture");
}

/// A stage the wrapper made is judged by the record of its object even when the destination record beside it was
/// settled away,
/// so a stage a person put a file in is kept and named instead of being removed as a copy's own.
/// Why the control reads this way:
///  the destination record is what said the stage belongs to an add,
///  and a command that settles that record while another settles the claim leaves the stage's own record as the only
///  thing that keeps it from being taken for a copy's own folder,
/// which is removed without asking what it holds.
#[cfg(target_os = "linux")]
#[test]
fn a_reap_in_another_process_keeps_the_stage_of_an_add_whose_destination_record_is_gone() {
    let Some(root) = crate::fs_btrfs_test_support::btrfs_fixture("claim-add-without-its-destination-record") else { return };
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(common.as_path()).expect("common directory");
    std::fs::create_dir_all(root.join("worktrees").as_path()).expect("worktree parents");
    std::fs::set_permissions(common.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private common directory");
    std::fs::set_permissions(root.join("worktrees").as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private worktree parent");
    let destination: PathBuf = root.join("worktrees/five");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    let source: PathBuf = root.join("source");
    crate::fs_btrfs_ioctl::subvolume_create_at(source.as_path()).expect("the source subvolume");
    std::fs::write(source.join("tracked.txt"), b"tracked").expect("a file");
    let claim: crate::worktree_copy_stage_claim::StageClaim =
        claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_owner()).expect("an add's claim");
    let container: PathBuf = claim.container().to_path_buf();
    // The record first, then the creation, then the person's file: the order a kill in the window leaves.
    let origin = crate::worktree_copy_stage_snapshot_origin::origin_of_the_source(source.as_path());
    crate::worktree_copy_stage_made_object::record_creating_at(common.as_path(), container.as_path(), std::time::SystemTime::now(), origin.as_ref())
        .expect("the record of the creation");
    crate::fs_btrfs_ioctl::snapshot_create_at(source.as_path(), container.as_path()).expect("the snapshot the killed add made");
    std::fs::set_permissions(container.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private stage");
    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(container.join("precious.txt"), b"a person's only copy").expect("a person's file");
    // The destination record was settled away by another command, and the claim it belonged to cannot be read.
    let record: PathBuf = claim_path_of(common.as_path(), &claim).with_extension("add");
    std::fs::remove_file(record.as_path()).expect("the destination record is settled away");
    let said: String = reap_in_a_child(common.as_path());
    assert!(container.join("precious.txt").exists(), "the person's file and the only copy of it survive: {said}");
    assert!(said.contains("precious.txt") && said.contains("was not changed"), "the line names the file that kept the stage: {said}");
    assert!(container.exists(), "the stage stands, so the person removes it once nothing needs it: {said}");
    crate::subvolume_remove::remove_tree(container.as_path()).expect("the control's snapshot");
    std::fs::remove_dir_all(root.as_path()).expect("the fixture");
}

/// The variable that marks the child which keeps a claim aside and is then killed.
const KEEP_CHILD_VARIABLE: &str = "CLI_GIT_KEEP_ASIDE_CHILD";

/// The variable that names the common directory the child works in.
const KEEP_CHILD_COMMON_VARIABLE: &str = "CLI_GIT_KEEP_ASIDE_COMMON";

/// The variable that names the claim the child keeps aside.
const KEEP_CHILD_CLAIM_VARIABLE: &str = "CLI_GIT_KEEP_ASIDE_CLAIM";

/// The body the parent starts in a child process for the kill between keeping a claim aside and writing it again:
/// it keeps the claim the environment names,
/// and the signal comes at the instruction after that,
/// which no handler can catch.
/// Run without the child's variable it returns at once and checks nothing.
#[test]
fn child_keeps_a_claim_aside_and_is_killed() {
    if std::env::var_os(KEEP_CHILD_VARIABLE).is_none() {
        return;
    }
    let common: PathBuf = PathBuf::from(std::env::var_os(KEEP_CHILD_COMMON_VARIABLE).expect("the child is given a common directory"));
    let claim: PathBuf = PathBuf::from(std::env::var_os(KEEP_CHILD_CLAIM_VARIABLE).expect("the child is given a claim"));
    let name: std::ffi::OsString = claim.file_name().expect("a claim has a name").to_os_string();
    crate::worktree_copy_unreadable_records::keep_aside(common.as_path(), claim.as_path(), name.as_os_str()).expect("kept");
    // SAFETY: `raise` has no preconditions, and `SIGKILL` ends this process before it can be caught or ignored.
    unsafe { libc::raise(libc::SIGKILL) };
    unreachable!("SIGKILL ends the child");
}

/// What:
///  Run the child that keeps the claim aside and is killed at the instruction after that,
/// and answer what the parent saw of it.
/// Why:
///   A kill between the two steps is what the state is,
/// so the child must be a process the kernel ends rather than one that returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const killed = keepAsideThenKill(common, claim); expect(killed.signal).toBe('SIGKILL');
/// ```
fn keep_aside_then_kill(common: &Path, claim: &Path) -> std::process::Output {
    use std::os::unix::process::ExitStatusExt;
    let module: &str = module_path!();
    let path: String = format!("{}::child_keeps_a_claim_aside_and_is_killed", module.split_once("::").map_or(module, |(_, rest)| { return rest; }));
    let output: std::process::Output = std::process::Command::new(crate::test_support::this_test_binary().expect("the test binary"))
        .args(["--exact", path.as_str(), "--nocapture", "--test-threads=1"])
        .env(KEEP_CHILD_VARIABLE, "1")
        .env(KEEP_CHILD_COMMON_VARIABLE, common)
        .env(KEEP_CHILD_CLAIM_VARIABLE, claim)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the child process runs");
    assert_eq!(output.status.signal(), Some(libc::SIGKILL), "the child was killed where the control says: {:?}", output.status);
    return output;
}

/// Keeping a claim that cannot be read leaves it one name,
/// because every recovery refuses a journal file with two,
/// and every command in a linked worktree reads every claim before it runs.
#[test]
fn keeping_a_claim_that_cannot_be_read_leaves_it_one_name() {
    use std::os::unix::fs::MetadataExt;
    let base: PathBuf = scratch("claim-kept-one-name");
    let common: PathBuf = base.join("common");
    let destination: PathBuf = base.join("worktrees/six");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    let claim: crate::worktree_copy_stage_claim::StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_owner()).expect("an add's claim");
    let claim_path: PathBuf = claim_path_of(common.as_path(), &claim);
    std::fs::write(claim_path.as_path(), b"").expect("the claim loses what it said");
    std::fs::set_permissions(claim_path.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o600)).expect("private claim");
    let kept: Option<PathBuf> = crate::worktree_copy_unreadable_records::keep_aside(common.as_path(), claim_path.as_path(), std::ffi::OsStr::new(claim_path.file_name().expect("a name"))).expect("kept");
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(claim_path.as_path()).expect("the claim stands at its own name");
    assert_eq!(metadata.nlink(), 1, "a claim has one name, and every recovery refuses one with two");
    assert_eq!(std::fs::read(kept.expect("the bytes are kept")).expect("the copy reads"), b"", "the bytes that could not be read are kept beside the records");
    crate::worktree_copy_stage_claim_reaping::judge_stage_owners(common.as_path()).expect("every recovery reads the claim and goes on");
    std::fs::remove_dir_all(base.as_path()).expect("the fixture");
}

/// A kill between keeping a claim aside and writing it again leaves a claim every recovery reads and goes on with,
/// and the next pass finishes what the kill interrupted.
/// Why the control reads this way:
///  the claim's bytes were kept by a second name for them,
///  and a journal file with two names stops every command of the repository until a person deletes it.
#[test]
fn a_kill_between_keeping_a_claim_aside_and_writing_it_again_leaves_a_command_that_goes_on() {
    let base: PathBuf = scratch("claim-kept-then-killed");
    let common: PathBuf = base.join("common");
    let destination: PathBuf = base.join("worktrees/seven");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    let claim: crate::worktree_copy_stage_claim::StageClaim = claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_owner()).expect("an add's claim");
    let container: PathBuf = claim.container().to_path_buf();
    let claim_path: PathBuf = claim_path_of(common.as_path(), &claim);
    std::fs::write(claim_path.as_path(), b"").expect("the claim loses what it said");
    std::fs::set_permissions(claim_path.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o600)).expect("private claim");
    keep_aside_then_kill(common.as_path(), claim_path.as_path());
    let mut verdicts: StageVerdicts = crate::worktree_copy_stage_claim_reaping::judge_stage_owners(common.as_path()).expect("every recovery reads the claim and goes on");
    let notices: Vec<String> = settle_unreadable_records(common.as_path(), &mut verdicts).expect("the next pass finishes the keep");
    assert!(notices.iter().any(|notice: &String| { return notice.contains("could not be read"); }), "the person is told what could not be read: {notices:?}");
    let recovered = crate::worktree_copy_stage_claim_reaping::read_claim(claim_path.as_path()).expect("read").expect("the claim reads again");
    assert_eq!(recovered.0, container, "the claim names the stage folder its records name");
    assert!(unreadable_records_stand(common.as_path()), "the bytes that could not be read stand in the folder beside the records");
    std::fs::remove_dir_all(base.as_path()).expect("the fixture");
}
