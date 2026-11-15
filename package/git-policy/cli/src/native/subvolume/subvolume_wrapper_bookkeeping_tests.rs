//! Controls for telling this program's own bookkeeping from a person's file in a tree pass,
//! at the pass itself and through the command that deletes the old tree.
//! The sighting they were written for,
//! of 2026-11-15:
//! the migration of this repository refused its own checkout,
//! because a wrapper process wrote its own lock-holder record while the copy ran and the
//!  next command's lock pass swept that record from the new checkout,
//! so the deletion of the old tree met a record the checkout lacked.
//! They run on the stand-in platform,
//! so they run on any filesystem,
//! including the container gate's overlay.

use crate::lock_holder_records::{SELF_KIND, folder_below_the_common_git_directory, identity_of_birth, record_text};
use crate::push_failure_notice::{PushOutcome, notice_directory, record_push_failure};
use crate::service_instance::digest_of_bytes;
use crate::subvolume_arguments::{MigrateArguments, MigrateMode};
use crate::subvolume_error::Interrupt;
use crate::subvolume_preflight::layout_of;
use crate::subvolume_sync::{SyncOptions, SyncStats, sync_tree, watch_entries};
use crate::subvolume_test_support::{PlainPlatform, checkout_with_worktree, migrate_arguments, run_command};
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The exit status a finished command gives.
const DONE: i32 = 0;

/// The exit status of a refusal.
const REFUSED: i32 = 1;

/// A process number no process of this machine can have,
/// so a record naming it is one whose holder this host proves gone.
const A_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE: i64 = 4_294_967_295;

/// A process number beside the first,
/// for the record a second wrapper process writes.
const ANOTHER_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE: i64 = A_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE - 1;

/// A time before every cut-off these controls set.
const BEFORE_EVERY_CUTOFF: i64 = 1_000_000_000;

/// A cut-off later than every entry these controls date.
const A_CUTOFF_AFTER_EVERY_ENTRY: i64 = 2_000_000_000;

/// A commit object name a push notice can name.
const A_COMMIT: &str = "1111111111111111111111111111111111111111";

/// Set both times of a path to whole seconds after the epoch,
/// as a writer of that moment would have left them.
fn set_mtime(path: &Path, seconds: i64) {
    let times: [libc::timespec; 2] = [libc::timespec { tv_sec: seconds, tv_nsec: 0 }, libc::timespec { tv_sec: seconds, tv_nsec: 0 }];
    let name = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("path text");
    // SAFETY: `name` is NUL-terminated and `times` holds two values; both outlive the call.
    let result: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) };
    assert_eq!(result, 0, "set the time of {}", path.display());
}

/// A source tree and an empty destination beside it,
/// each with a Git directory.
fn trees(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let source: PathBuf = parent.join("source");
    let destination: PathBuf = parent.join("destination");
    std::fs::create_dir_all(source.join(".git")).expect("source Git directory");
    std::fs::create_dir_all(destination.join(".git")).expect("destination Git directory");
    return (parent, source, destination);
}

/// The folder a repository's records live in,
/// below a tree's Git directory.
fn records_folder(tree: &Path) -> PathBuf {
    return tree.join(".git").join(folder_below_the_common_git_directory());
}

/// The file name the writer gives its record of `pid`,
/// which is the number and the digest of the identity it wrote.
fn record_name(pid: i64) -> String {
    let identity: String = identity_of_birth(format!("linux:{pid}").as_str());
    return format!("{pid}.{}", digest_of_bytes(identity.as_bytes()));
}

/// The text of the record `record_name` names.
fn record_text_of(pid: i64) -> String {
    let identity: String = identity_of_birth(format!("linux:{pid}").as_str());
    return record_text(&[(SELF_KIND, pid, identity)]);
}

/// Write one of this program's own records into `folder`.
fn write_record(folder: &Path, pid: i64) -> PathBuf {
    std::fs::create_dir_all(folder).expect("records folder");
    let path: PathBuf = folder.join(record_name(pid));
    std::fs::write(path.as_path(), record_text_of(pid)).expect("record");
    return path;
}

/// What the record folder is called in the paths a pass reports,
/// relative to a tree's root.
fn records_below_the_git_directory() -> PathBuf {
    return PathBuf::from(".git").join(folder_below_the_common_git_directory());
}

/// One update-only pass with a cut-off,
/// as the deletion of the old tree runs it.
fn carry(source: &Path, destination: &Path) -> SyncStats {
    let options: SyncOptions<'_> = SyncOptions { delete: false, update_only: true, excluded: &[], created_since: Some(A_CUTOFF_AFTER_EVERY_ENTRY) };
    return sync_tree(&PlainPlatform::new(), source, destination, options, &Interrupt::never()).expect("a pass");
}

/// The exit status of one command and everything it said,
/// joined for one assertion message.
fn run(platform: &PlainPlatform, arguments: &MigrateArguments, root: &Path) -> (i32, String) {
    let (code, lines) = run_command(platform, arguments, root, &Interrupt::never());
    return (code, lines.join("\n"));
}

/// The arguments of the deletion of the old tree.
fn remove_old_arguments() -> MigrateArguments {
    return MigrateArguments { mode: MigrateMode::RemoveOld, ..migrate_arguments() };
}

/// What:
///  Install the callback a copy pass runs for every entry it examines,
///  which writes `path` with `text` the first time the pass hands it an entry of the folder.
/// Why:
///   The record has to appear after the pass listed that folder,
///  so the copy does not carry it,
///  and the moment the walk hands the pass one of the folder's entries is exactly then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watchEntries((examined) => { if (dir(examined) === folder && !written) write(path, text); });
/// ```
fn write_a_second_wrappers_record_when_the_copy_lists(folder: &Path, path: PathBuf, text: String) {
    let folder: PathBuf = folder.to_path_buf();
    let written: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    watch_entries(Some(Box::new(move |examined: &Path| {
        if examined.parent() != Some(folder.as_path()) || written.swap(true, Ordering::SeqCst) {
            return;
        }
        std::fs::write(path.as_path(), text.as_bytes()).expect("the second wrapper's record");
        set_mtime(path.as_path(), BEFORE_EVERY_CUTOFF);
    })));
}

/// The only entry of a folder a writer filled with one.
fn only_entry(folder: &Path) -> PathBuf {
    let mut found: Vec<PathBuf> = Vec::new();
    for item in std::fs::read_dir(folder).expect("listing").flatten() {
        found.push(item.path());
    }
    assert_eq!(found.len(), 1, "one entry in {}: {found:?}", folder.display());
    return found.remove(0);
}

/// A record of this program's own that the destination lacks is left out,
/// while a person's file beside it is left out as a person's entry,
/// which is what the deletion of the old tree refuses over.
#[test]
fn a_lock_holders_record_the_destination_lacks_is_left_out_and_a_persons_file_beside_it_is_not() {
    let (parent, source, destination) = trees("bookkeeping-record");
    let folder: PathBuf = records_folder(source.as_path());
    let record: PathBuf = write_record(folder.as_path(), A_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE);
    let personal: PathBuf = folder.join("notes-from-me.txt");
    std::fs::write(personal.as_path(), b"a note I keep beside those records\n").expect("a person's file");
    for path in [&record, &personal] {
        set_mtime(path.as_path(), BEFORE_EVERY_CUTOFF);
    }
    std::fs::create_dir_all(records_folder(destination.as_path())).expect("the destination's folder");
    let stats: SyncStats = carry(source.as_path(), destination.as_path());
    assert_eq!(stats.holder_records_left_out, vec![records_below_the_git_directory().join(record_name(A_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE))], "{stats:#?}");
    assert_eq!(stats.left_out, vec![records_below_the_git_directory().join("notes-from-me.txt")], "a person's file is a person's entry: {stats:#?}");
    assert!(stats.notices_carried.is_empty(), "{stats:#?}");
    let destination: PathBuf = destination.join(".git");
    assert!(!destination.join(folder_below_the_common_git_directory()).join(record_name(A_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE)).exists(), "the record is not carried");
    assert!(!destination.join(folder_below_the_common_git_directory()).join("notes-from-me.txt").exists(), "the person's file is not carried either");
    remove(parent.as_path());
}

/// A notice this program wrote for the person is carried into the destination although it is older than the
///  pass's cut-off,
/// so what it says does not go with the old tree.
#[test]
fn a_notice_this_program_wrote_is_carried_over_although_it_is_older_than_the_cut_off() {
    let (parent, source, destination) = trees("bookkeeping-notice");
    let git_directory: PathBuf = source.join(".git");
    record_push_failure(git_directory.as_path(), "refs/heads/main", A_COMMIT, PushOutcome::Refused, "the remote refused it").expect("notice");
    let notice: PathBuf = only_entry(notice_directory(git_directory.as_path()).as_path());
    set_mtime(notice.as_path(), BEFORE_EVERY_CUTOFF);
    let below: PathBuf = notice.strip_prefix(source.as_path()).expect("below the source").to_path_buf();
    let stats: SyncStats = carry(source.as_path(), destination.as_path());
    assert_eq!(stats.notices_carried, vec![below.clone()], "{stats:#?}");
    assert!(stats.left_out.is_empty(), "{stats:#?}");
    assert!(destination.join(&below).exists(), "the notice reaches the destination");
    remove(parent.as_path());
}

/// A person's file under a name this program keeps is still a person's file:
/// without the mark its writer puts on it,
/// it is left out like any other entry.
#[test]
fn a_persons_file_under_a_name_this_program_keeps_still_counts() {
    let (parent, source, destination) = trees("bookkeeping-personal");
    let records: PathBuf = records_folder(source.as_path());
    std::fs::create_dir_all(records.as_path()).expect("records folder");
    let beside_a_record: PathBuf = records.join(record_name(A_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE));
    std::fs::write(beside_a_record.as_path(), b"my own file at a name the wrapper uses\n").expect("a person's file");
    let git_directory: PathBuf = source.join(".git");
    record_push_failure(git_directory.as_path(), "refs/heads/main", A_COMMIT, PushOutcome::Refused, "the remote refused it").expect("notice");
    let beside_a_notice: PathBuf = notice_directory(git_directory.as_path()).join("mine.txt");
    std::fs::write(beside_a_notice.as_path(), b"my own text, not a notice\n").expect("a person's file");
    for path in [&beside_a_record, &beside_a_notice] {
        set_mtime(path.as_path(), BEFORE_EVERY_CUTOFF);
    }
    let stats: SyncStats = carry(source.as_path(), destination.as_path());
    let mut observed: Vec<PathBuf> = stats.left_out.clone();
    observed.sort();
    let mut wanted: Vec<PathBuf> = vec![
        beside_a_record.strip_prefix(source.as_path()).expect("below the source").to_path_buf(),
        beside_a_notice.strip_prefix(source.as_path()).expect("below the source").to_path_buf(),
    ];
    wanted.sort();
    assert_eq!(observed, wanted, "each of them counts as a person's entry: {stats:#?}");
    assert!(stats.holder_records_left_out.is_empty(), "{stats:#?}");
    assert_eq!(stats.notices_carried.len(), 1, "the notice beside the person's file is still carried: {stats:#?}");
    remove(parent.as_path());
}

/// A folder under one of those names is carried so the pass walks into it,
/// and a person's file below it still counts while the notice beside it is carried.
#[test]
fn a_folder_under_one_of_those_names_is_walked_so_a_persons_file_below_it_still_counts() {
    let (parent, source, destination) = trees("bookkeeping-folder");
    let git_directory: PathBuf = source.join(".git");
    record_push_failure(git_directory.as_path(), "refs/heads/main", A_COMMIT, PushOutcome::Refused, "the remote refused it").expect("notice");
    let notice: PathBuf = only_entry(notice_directory(git_directory.as_path()).as_path());
    let folder: PathBuf = notice.parent().expect("the notice's folder").to_path_buf();
    let personal: PathBuf = folder.join("mine.txt");
    std::fs::write(personal.as_path(), b"my own text, not a notice\n").expect("a person's file");
    for path in [&notice, &personal, &folder] {
        set_mtime(path.as_path(), BEFORE_EVERY_CUTOFF);
    }
    let stats: SyncStats = carry(source.as_path(), destination.as_path());
    assert_eq!(stats.notices_carried.len(), 1, "{stats:#?}");
    assert_eq!(stats.left_out.len(), 1, "the person's file below the folder still counts: {stats:#?}");
    assert_eq!(stats.left_out[0].file_name().expect("a name"), std::ffi::OsStr::new("mine.txt"), "{stats:#?}");
    assert!(destination.join(notice.strip_prefix(source.as_path()).expect("below the source")).exists(), "the notice reaches the destination");
    assert!(!destination.join(personal.strip_prefix(source.as_path()).expect("below the source")).exists(), "the person's file does not");
    remove(parent.as_path());
}

/// What the sighting of 2026-11-15 came to,
/// through the code's own seams:
/// the copy pass hands the walk each entry of a folder only after it listed that folder,
/// so a second wrapper process writing its own record there is one the copy does not carry;
/// the catch-up and the first delta copy it into the new checkout;
/// the next command's lock pass sweeps it from the checkout;
/// and the deletion of the old tree then meets a record the checkout lacks.
/// The deletion must not stop for it:
/// the record names processes that ran in the old tree and holds nothing a person wrote.
#[test]
fn the_old_trees_record_of_this_programs_own_process_does_not_stop_the_deletion() {
    let (parent, root) = checkout_with_worktree("bookkeeping-sighting");
    let platform: PlainPlatform = PlainPlatform::new();
    let records: PathBuf = records_folder(root.as_path());
    let carried: PathBuf = write_record(records.as_path(), A_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE);
    set_mtime(carried.as_path(), BEFORE_EVERY_CUTOFF);
    let later: PathBuf = records.join(record_name(ANOTHER_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE));
    let later_text: String = record_text_of(ANOTHER_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE);
    write_a_second_wrappers_record_when_the_copy_lists(records.as_path(), later.clone(), later_text);
    let (code, said) = run(&platform, &migrate_arguments(), root.as_path());
    watch_entries(None);
    assert_eq!(code, DONE, "{said}");
    assert!(later.exists() && carried.exists(), "both records are in the old tree: {}", later.display());
    let layout = layout_of(root.as_path()).expect("layout");
    assert!(layout.old_path.join(".git").join(folder_below_the_common_git_directory()).join(record_name(ANOTHER_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE)).exists(), "the old tree holds the record the second wrapper wrote");
    let checkout_records: PathBuf = records_folder(root.as_path());
    assert!(checkout_records.join(record_name(A_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE)).exists(), "the copy carried the first record into the new checkout");
    // The next command's lock pass removes a record whose holders are all gone and whose numbers stand beside no lock:
    //  a record of a process of this program's own is litter once that process has ended.
    crate::lock_holder_record_reads::sweep(root.join(".git").as_path(), &[]);
    assert!(!checkout_records.join(record_name(A_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE)).exists(), "the sweep removed the record the checkout held");
    assert!(!checkout_records.join(record_name(ANOTHER_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE)).exists(), "and the one the catch-up carried there");
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    assert_eq!(code, DONE, "{said}");
    assert!(!layout.old_path.exists(), "the old tree is deleted");
    assert!(said.contains("record(s) of this program's own lock holders stayed in the source"), "the pass says what it did with the record: {said}");
    remove(parent.as_path());
}

/// A person's file placed while the copy ran is still a person's,
/// and the deletion of the old tree still refuses when the checkout lost it:
/// one at an ordinary name,
/// and one at a name this program keeps whose text is not that program's.
#[test]
fn a_persons_file_the_checkout_lost_still_stops_the_deletion() {
    let (parent, root) = checkout_with_worktree("bookkeeping-personal-removal");
    let platform: PlainPlatform = PlainPlatform::new();
    let git_directory: PathBuf = root.join(".git");
    let records: PathBuf = records_folder(root.as_path());
    std::fs::create_dir_all(records.as_path()).expect("records folder");
    let note: PathBuf = git_directory.join("person-notes.txt");
    let beside_a_record: PathBuf = records.join(record_name(A_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE));
    let written: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    {
        let git_directory: PathBuf = git_directory.clone();
        let note: PathBuf = note.clone();
        let beside_a_record: PathBuf = beside_a_record.clone();
        watch_entries(Some(Box::new(move |examined: &Path| {
            if examined.parent() != Some(git_directory.as_path()) || written.swap(true, Ordering::SeqCst) {
                return;
            }
            std::fs::write(note.as_path(), b"a note I put there while the copy ran\n").expect("a person's note");
            set_mtime(note.as_path(), BEFORE_EVERY_CUTOFF);
            std::fs::write(beside_a_record.as_path(), b"my own file at a name the wrapper uses\n").expect("a person's file");
            set_mtime(beside_a_record.as_path(), BEFORE_EVERY_CUTOFF);
        })));
    }
    let (code, said) = run(&platform, &migrate_arguments(), root.as_path());
    watch_entries(None);
    assert_eq!(code, DONE, "{said}");
    assert!(note.exists() && beside_a_record.exists(), "the catch-up carried both into the checkout");
    // The person deleted them from the checkout on purpose, which is the entry the cut-off exists for.
    std::fs::remove_file(note.as_path()).expect("the person's deletion");
    std::fs::remove_file(beside_a_record.as_path()).expect("the person's deletion");
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    assert_eq!(code, REFUSED, "{said}");
    let layout = layout_of(root.as_path()).expect("layout");
    assert!(layout.old_path.exists(), "nothing was deleted");
    assert!(said.contains("2 entr(ies) absent") && said.contains("person-notes.txt"), "{said}");
    assert!(said.contains(record_name(A_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE).as_str()), "a person's file at a name this program keeps is named too: {said}");
    remove(parent.as_path());
}
