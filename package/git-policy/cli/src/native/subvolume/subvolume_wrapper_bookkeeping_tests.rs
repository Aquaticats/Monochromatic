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
use crate::subvolume_anchor::Anchor;
use crate::subvolume_sync::{SyncOptions, SyncStats, sync_tree, watch_entries};
use crate::subvolume_sync_pass_state_and_checks::{Directory, PARKED_SUFFIX, PARTIAL_NAME, Pass, Published, watch_before_the_judgement, watch_the_freed_name};
use crate::subvolume_wrapper_bookkeeping::{OwnBookkeeping, notice_kinds, own_bookkeeping_of};
use crate::subvolume_test_support::{PlainPlatform, checkout_with_worktree, migrate_arguments, run_command};
use crate::test_support::{fixture, remove};
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
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
    return sync_tree(&PlainPlatform::new(), source, destination, options, &Interrupt::never(), None).expect("a pass");
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
    let records: PathBuf = folder.to_path_buf();
    let written: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    watch_entries(Some(Box::new(move |examined: &Path| {
        if examined.parent() != Some(records.as_path()) || written.swap(true, Ordering::SeqCst) {
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
    let in_destination: PathBuf = destination.join(".git").join(folder_below_the_common_git_directory());
    assert!(!in_destination.join(record_name(A_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE)).exists(), "the record is not carried");
    assert!(!in_destination.join("notes-from-me.txt").exists(), "the person's file is not carried either");
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
    let (removal_code, removal_said) = run(&platform, &remove_old_arguments(), root.as_path());
    assert_eq!(removal_code, DONE, "{removal_said}");
    assert!(!layout.old_path.exists(), "the old tree is deleted");
    assert!(removal_said.contains("record(s) of this program's own lock holders stayed in the source"), "the pass says what it did with the record: {removal_said}");
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
        let written_git_directory: PathBuf = git_directory.clone();
        let written_note: PathBuf = note.clone();
        let written_beside_a_record: PathBuf = beside_a_record.clone();
        watch_entries(Some(Box::new(move |examined: &Path| {
            if examined.parent() != Some(written_git_directory.as_path()) || written.swap(true, Ordering::SeqCst) {
                return;
            }
            std::fs::write(written_note.as_path(), b"a note I put there while the copy ran\n").expect("a person's note");
            set_mtime(written_note.as_path(), BEFORE_EVERY_CUTOFF);
            std::fs::write(written_beside_a_record.as_path(), b"my own file at a name the wrapper uses\n").expect("a person's file");
            set_mtime(written_beside_a_record.as_path(), BEFORE_EVERY_CUTOFF);
        })));
    }
    let (code, said) = run(&platform, &migrate_arguments(), root.as_path());
    watch_entries(None);
    assert_eq!(code, DONE, "{said}");
    assert!(note.exists() && beside_a_record.exists(), "the catch-up carried both into the checkout");
    // The person deleted them from the checkout on purpose, which is the entry the cut-off exists for.
    std::fs::remove_file(note.as_path()).expect("the person's deletion");
    std::fs::remove_file(beside_a_record.as_path()).expect("the person's deletion");
    let (removal_code, removal_said) = run(&platform, &remove_old_arguments(), root.as_path());
    assert_eq!(removal_code, REFUSED, "{removal_said}");
    let layout = layout_of(root.as_path()).expect("layout");
    assert!(layout.old_path.exists(), "nothing was deleted");
    assert!(removal_said.contains("2 entr(ies) absent") && removal_said.contains("person-notes.txt"), "{removal_said}");
    assert!(removal_said.contains(record_name(A_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE).as_str()), "a person's file at a name this program keeps is named too: {removal_said}");
    remove(parent.as_path());
}

//region The three controls of the review of 311a65f852, reproduced here as they were written
// They were added first and run against that tip, and each failed there; the fixes below are what answers them.

/// What the command says when it runs against a stand-in that acts at the copy's clone calls.
fn run_hooked(platform: &crate::subvolume_test_support::Hooked, arguments: &MigrateArguments, root: &Path) -> (i32, String) {
    let (code, lines) = run_command(platform, arguments, root, &Interrupt::never());
    return (code, lines.join("\n"));
}

/// The review, finding 3:
/// a notice in the spelling the push reader explicitly accepts from an earlier release
/// must be carried instead of holding back deletion of the old tree.
#[test]
fn review_legacy_push_notice_does_not_hold_back_old_tree_removal() {
    let (parent, root) = checkout_with_worktree("review-legacy-push-notice");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migration_code, migration_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migration_code, DONE, "{migration_said}");
    let layout = layout_of(root.as_path()).expect("layout");
    let directory: PathBuf = layout.old_path.join(".git/cli-git/notices");
    std::fs::create_dir_all(directory.as_path()).expect("legacy notice directory");
    let notice: PathBuf = directory.join("push+refs+heads+main+1111111111111111111111111111111111111111.txt");
    let text: &str = "cli-git push notice 1\nbranch: refs/heads/main\ncommit: 1111111111111111111111111111111111111111\n\nThe old remote refused it\n";
    std::fs::write(notice.as_path(), text.as_bytes()).expect("legacy notice");
    set_mtime(notice.as_path(), BEFORE_EVERY_CUTOFF);
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    assert_eq!(code, DONE, "the accepted legacy notice spelling was not carried: {said}");
    assert_eq!(std::fs::read_to_string(root.join(".git/cli-git/notices").join(notice.file_name().expect("name"))).expect("carried notice"), text);
    remove(parent.as_path());
}

/// The review, finding 1:
/// a regular record can become a directory after the directory listing and before bookkeeping classification.
/// Its contents must not vanish without ever entering the pass.
#[test]
fn review_record_replaced_by_a_directory_does_not_lose_the_directory_contents() {
    let (parent, root) = checkout_with_worktree("review-record-becomes-directory");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migration_code, migration_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migration_code, DONE, "{migration_said}");
    let layout = layout_of(root.as_path()).expect("layout");
    let record: PathBuf = write_record(records_folder(layout.old_path.as_path()).as_path(), A_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE);
    set_mtime(record.as_path(), BEFORE_EVERY_CUTOFF);
    let relative: PathBuf = record.strip_prefix(layout.old_path.as_path()).expect("below old tree").to_path_buf();
    let replaced: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let target: PathBuf = record.clone();
    watch_entries(Some(Box::new(move |examined: &Path| {
        if examined != target.as_path() || replaced.swap(true, Ordering::SeqCst) {
            return;
        }
        std::fs::remove_file(target.as_path()).expect("replace the listed record");
        std::fs::create_dir(target.as_path()).expect("directory at the record name");
        std::fs::write(target.join("person.txt"), b"words written beside the migration\n").expect("person's file in the replacement directory");
    })));
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    watch_entries(None);
    let in_old_tree: PathBuf = record.join("person.txt");
    let in_checkout: PathBuf = root.join(relative).join("person.txt");
    assert!(in_old_tree.is_file() || in_checkout.is_file(), "removal code {code}; the replacement directory's person's file exists in neither tree: {said}");
    remove(parent.as_path());
}

/// The review, finding 2:
/// a destination notice written after the pass's initial absence check is concurrent work.
/// Publishing the old source notice must not rename over it.
#[test]
fn review_notice_copy_does_not_replace_a_concurrently_created_destination_file() {
    let (parent, root) = checkout_with_worktree("review-destination-created-during-copy");
    let mut platform: crate::subvolume_test_support::Hooked = crate::subvolume_test_support::Hooked::new(Box::new(|_calls| {}));
    let (migration_code, migration_said) = run_hooked(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migration_code, DONE, "{migration_said}");
    let layout = layout_of(root.as_path()).expect("layout");
    record_push_failure(layout.old_path.join(".git").as_path(), "refs/heads/main", A_COMMIT, PushOutcome::Refused, "the source notice").expect("source notice");
    let old_notice: PathBuf = only_entry(notice_directory(layout.old_path.join(".git").as_path()).as_path());
    set_mtime(old_notice.as_path(), BEFORE_EVERY_CUTOFF);
    let relative: PathBuf = old_notice.strip_prefix(layout.old_path.as_path()).expect("below old tree").to_path_buf();
    let destination_notice: PathBuf = root.join(relative);
    let concurrent_notice: PathBuf = destination_notice.clone();
    let concurrent_text: &str = "monochromatic-git-policy-cli push notice 1\nbranch: refs/heads/main\ncommit: 2222222222222222222222222222222222222222\n\nconcurrent words\n";
    platform.on_clone = Box::new(move |_calls| {
        let Some(folder) = concurrent_notice.parent() else { return };
        std::fs::write(concurrent_notice.as_path(), concurrent_text.as_bytes()).expect("concurrent destination notice");
        assert!(folder.is_dir());
    });
    let (code, said) = run_hooked(&platform, &remove_old_arguments(), root.as_path());
    let after: String = std::fs::read_to_string(destination_notice.as_path()).unwrap_or_else(|error| format!("destination notice cannot be read: {error}"));
    assert_eq!(after, concurrent_text, "removal code {code}; the copy replaced the concurrent destination notice: {said}");
    remove(parent.as_path());
}
//endregion

//region What the three fixes of the review's findings are held by
// The review's controls hold the three states it found; these hold the roads to them, and the list of spellings the
// writers and readers own.

/// The options a control that drives one step at a time runs with:
/// copy only,
/// which is what the first pass of a migration does.
const COPY: SyncOptions<'static> = SyncOptions { delete: false, update_only: false, excluded: &[], created_since: None };

/// A pass over `source` and `destination` that a control drives one step at a time.
fn pass_over<'a>(platform: &'a PlainPlatform, source: &'a Path, destination: &'a Path, interrupt: &'a Interrupt) -> Pass<'a> {
    return Pass {
        note_uncarried: None,
        platform,
        source,
        destination,
        options: COPY,
        interrupt,
        stats: SyncStats::default(),
        links: HashMap::new(),
        stage_holders: HashSet::new(),
    };
}

/// The destination directory of a relative path,
/// held open with the temporary name a pass uses.
fn directory_of(destination: &Path) -> Directory {
    let anchor: Anchor = Anchor::open_below(destination, Path::new("")).expect("open the destination");
    return Directory { anchor, temporary: OsString::from(PARTIAL_NAME) };
}

/// What:
///  A file line and a name no writer of this program uses,
///  for the controls that need a person's file inside a folder this program keeps.
/// Why:
///   The mark decides, not the folder,
///  so the same folder has to hold a file this program did not write.
const A_PERSONS_FIRST_LINE: &str = "my own text, not a record of any program\n";

/// Every kind this module knows is told through every first line its own reader accepts,
/// in every folder its writer spells,
/// and a file whose first line its reader refuses is a person's.
#[test]
fn every_kind_is_told_through_every_spelling_its_reader_accepts() {
    let parent: PathBuf = fixture("bookkeeping-spellings");
    let tree: PathBuf = parent.join("tree");
    std::fs::create_dir_all(tree.join(".git")).expect("Git directory");
    let records: PathBuf = records_below_the_git_directory();
    let mut told: Vec<PathBuf> = Vec::new();
    // The records of the processes this program started: one folder, one header.
    let record: PathBuf = write_record(tree.join(records.as_path()).as_path(), A_PROCESS_NUMBER_THIS_MACHINE_CANNOT_HAVE);
    assert_eq!(own_bookkeeping_of(tree.as_path(), record.strip_prefix(tree.as_path()).expect("below the tree"), &std::fs::symlink_metadata(record.as_path()).expect("record")), Some(OwnBookkeeping::LeftOut));
    told.push(record);
    for kind in notice_kinds() {
        assert!(!kind.first_lines.is_empty(), "a kind with no spelling to walk");
        for folder in &kind.folders {
            for line in &kind.first_lines {
                let path: PathBuf = tree.join(".git").join(folder).join(format!("notice-{}", told.len()));
                std::fs::create_dir_all(path.parent().expect("folder")).expect("notice folder");
                std::fs::write(path.as_path(), format!("{line}\nbranch: refs/heads/main\n\nwords\n").as_bytes()).expect("notice");
                let relative: &Path = path.strip_prefix(tree.as_path()).expect("below the tree");
                let listed: std::fs::Metadata = std::fs::symlink_metadata(path.as_path()).expect("notice");
                assert_eq!(own_bookkeeping_of(tree.as_path(), relative, &listed), Some(OwnBookkeeping::CarriedOver), "{} in {}", line, folder.display());
                told.push(path);
            }
            let personal: PathBuf = tree.join(".git").join(folder).join(format!("person-{}", told.len()));
            std::fs::write(personal.as_path(), A_PERSONS_FIRST_LINE.as_bytes()).expect("a person's file");
            let relative: &Path = personal.strip_prefix(tree.as_path()).expect("below the tree");
            let listed: std::fs::Metadata = std::fs::symlink_metadata(personal.as_path()).expect("a person's file");
            assert_eq!(own_bookkeeping_of(tree.as_path(), relative, &listed), None, "a person's file in {}", folder.display());
            told.push(personal);
        }
    }
    assert!(told.len() > 1, "the walk told nothing");
    // An entry of a kind that has no mark at all, under a folder this program keeps, is nobody's bookkeeping either.
    let pipe: PathBuf = tree.join(".git").join(&notice_kinds()[0].folders[0]).join("a-named-pipe");
    crate::subvolume_sync_attrs::make_fifo(pipe.as_path(), 0o600).expect("a named pipe");
    let pipe_listed: std::fs::Metadata = std::fs::symlink_metadata(pipe.as_path()).expect("a named pipe");
    assert_eq!(own_bookkeeping_of(tree.as_path(), pipe.strip_prefix(tree.as_path()).expect("below the tree"), &pipe_listed), None);
    // A folder is carried, so the walk reaches whatever it holds.
    let held: PathBuf = tree.join(".git").join(&notice_kinds()[0].folders[0]);
    let folder_listed: std::fs::Metadata = std::fs::symlink_metadata(held.as_path()).expect("the notice folder");
    assert_eq!(own_bookkeeping_of(tree.as_path(), held.strip_prefix(tree.as_path()).expect("below the tree"), &folder_listed), Some(OwnBookkeeping::CarriedOver));
    // A path outside the folders this program keeps is nobody's bookkeeping, whatever its first line says.
    let outside: PathBuf = tree.join(".git").join("person-notes.txt");
    std::fs::write(outside.as_path(), format!("{}\n", notice_kinds()[0].first_lines[0]).as_bytes()).expect("note outside");
    let listed: std::fs::Metadata = std::fs::symlink_metadata(outside.as_path()).expect("note");
    assert_eq!(own_bookkeeping_of(tree.as_path(), outside.strip_prefix(tree.as_path()).expect("below the tree"), &listed), None);
    remove(parent.as_path());
}

/// What:
///  The walk the controls install to replace one entry after the pass listed its folder.
/// Why:
///   The look and the act are two steps,
///  so a control of what happens between them has to stand in that moment,
///  which is where the walk hands every listed entry on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watchEntries((examined) => { if (examined === target) replaceAt(target); });
/// ```
fn replace_after_the_listing(target: PathBuf, replace: impl Fn(&Path) + 'static) {
    let replaced: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    watch_entries(Some(Box::new(move |examined: &Path| {
        if examined != target.as_path() || replaced.swap(true, Ordering::SeqCst) {
            return;
        }
        replace(target.as_path());
    })));
}

/// A source file the pass listed as a file and that is a directory with a person's file in it when the copy reaches
///  it is reported as what it is,
/// not counted as a vanished file,
/// and the deletion of the old tree is held back.
#[test]
fn a_source_file_that_became_a_directory_holds_the_deletion_back() {
    let (parent, root) = checkout_with_worktree("changed-kind-removal");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migration_code, migration_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migration_code, DONE, "{migration_said}");
    let layout = layout_of(root.as_path()).expect("layout");
    // A file newer than the catch-up, absent from the checkout: the late-write carry tries to copy it.
    let note: PathBuf = layout.old_path.join("person-notes.txt");
    std::fs::write(note.as_path(), b"a note I wrote after the migration\n").expect("a person's note");
    replace_after_the_listing(note.clone(), |path: &Path| {
        std::fs::remove_file(path).expect("replace the listed file");
        std::fs::create_dir(path).expect("a directory at the note's name");
        std::fs::write(path.join("inside.txt"), b"words inside the replacement\n").expect("a person's file inside");
    });
    let (removal_code, removal_said) = run(&platform, &remove_old_arguments(), root.as_path());
    watch_entries(None);
    assert_eq!(removal_code, REFUSED, "{removal_said}");
    assert!(removal_said.contains("did not carry person-notes.txt: it changed while the copy ran"), "{removal_said}");
    assert!(note.join("inside.txt").is_file(), "the person's file inside the replacement directory is still in the old tree");
    assert!(layout.old_path.exists(), "nothing was deleted");
    remove(parent.as_path());
}

/// The same state at the pass: a listed file that is a directory when the copy reaches it is counted as the other kind
///  of entry,
/// and never as a vanished file.
#[test]
fn a_listed_file_that_became_a_directory_is_counted_as_another_kind() {
    let (parent, source, destination) = trees("changed-kind-pass");
    let note: PathBuf = source.join("person-notes.txt");
    std::fs::write(note.as_path(), b"a note\n").expect("a person's note");
    // Newer than the cut-off, so the pass tries to copy it rather than leaving it out.
    set_mtime(note.as_path(), A_CUTOFF_AFTER_EVERY_ENTRY + 1);
    replace_after_the_listing(note.clone(), |path: &Path| {
        std::fs::remove_file(path).expect("replace the listed file");
        std::fs::create_dir(path).expect("a directory at the note's name");
        std::fs::write(path.join("inside.txt"), b"words inside\n").expect("a person's file inside");
    });
    let stats: SyncStats = carry(source.as_path(), destination.as_path());
    watch_entries(None);
    assert_eq!(stats.changed_while_passing, vec![PathBuf::from("person-notes.txt")], "{stats:#?}");
    assert_eq!(stats.vanished, 0, "a changed kind is not a vanished entry: {stats:#?}");
    assert!(note.join("inside.txt").is_file(), "the person's file is still in the source");
    remove(parent.as_path());
}

/// A file the pass found absent at the destination and that another command created while the copy ran is kept,
///  and the source entry is reported:
/// the same publish serves an ordinary file as serves a notice.
#[test]
fn an_ordinary_file_the_destination_gained_during_the_copy_is_kept() {
    let (parent, source, destination) = trees("kept-ordinary-file");
    let note: PathBuf = source.join("person-notes.txt");
    std::fs::write(note.as_path(), b"a note in the source\n").expect("a person's note");
    let concurrent_text: &[u8] = b"a note another command wrote into the destination\n";
    let written_into: PathBuf = destination.join("person-notes.txt");
    let mut platform: crate::subvolume_test_support::Hooked = crate::subvolume_test_support::Hooked::new(Box::new(move |_calls| {
        std::fs::write(written_into.as_path(), concurrent_text).expect("the concurrent file");
    }));
    let options: SyncOptions<'_> = SyncOptions { delete: false, update_only: false, excluded: &[], created_since: None };
    let stats: SyncStats = sync_tree(&platform, source.as_path(), destination.as_path(), options, &Interrupt::never(), None).expect("a pass");
    platform.on_clone = Box::new(|_calls| {});
    assert_eq!(stats.kept_at_the_destination, vec![PathBuf::from("person-notes.txt")], "{stats:#?}");
    assert_eq!(std::fs::read(destination.join("person-notes.txt")).expect("the destination file"), concurrent_text, "the concurrent file is not replaced");
    assert_eq!(std::fs::read(note.as_path()).expect("the source file"), b"a note in the source\n", "the source file is untouched");
    remove(parent.as_path());
}

/// The publish's own roads:
/// a name the pass found free is created and never replaced,
/// and a name that held an entry is freed by parking what stands there,
///  which is judged and put back unless it is the very object the pass examined.
#[test]
fn a_name_the_pass_found_free_is_created_and_one_it_saw_is_replaced_only_as_the_same_object() {
    let (parent, source, destination) = trees("publish-roads");
    let directory: Directory = directory_of(destination.as_path());
    let file: PathBuf = source.join("file.txt");
    std::fs::write(file.as_path(), b"the source file\n").expect("the source file");
    let listing: std::fs::Metadata = std::fs::symlink_metadata(file.as_path()).expect("listing");
    let (platform, interrupt): (PlainPlatform, Interrupt) = (PlainPlatform::new(), Interrupt::never());
    let mut pass: Pass<'_> = pass_over(&platform, source.as_path(), destination.as_path(), &interrupt);
    // The free name, created by somebody else between the look and the write.
    std::fs::write(directory.partial().as_path(), b"the copy").expect("the finished copy");
    std::fs::write(destination.join("file.txt"), b"somebody else's file").expect("the concurrent entry");
    assert_eq!(pass.publish(&directory, Path::new("file.txt"), None).expect("a publish"), Published::KeptTheDestinationEntry);
    assert_eq!(std::fs::read(destination.join("file.txt")).expect("kept"), b"somebody else's file");
    assert!(!directory.partial().exists(), "the copy of the entry that was kept is dropped");
    // An entry the pass did not examine: it is parked, judged, and put back over the copy this pass made there.
    std::fs::write(directory.partial().as_path(), b"the copy").expect("the finished copy");
    assert_eq!(pass.publish(&directory, Path::new("file.txt"), Some(&listing)).expect("a publish"), Published::KeptTheDestinationEntry);
    assert_eq!(std::fs::read(destination.join("file.txt")).expect("kept"), b"somebody else's file", "the entry the pass did not examine is put back");
    assert!(!destination.join(format!("file.txt{PARKED_SUFFIX}")).exists(), "and nothing stays parked after the judgement");
    // The very object the pass examined: it is superseded, and the copy takes the name.
    let seen: std::fs::Metadata = std::fs::symlink_metadata(destination.join("file.txt")).expect("the entry the pass saw");
    std::fs::write(directory.partial().as_path(), b"the copy").expect("the finished copy");
    assert_eq!(pass.publish(&directory, Path::new("file.txt"), Some(&seen)).expect("a publish"), Published::Placed);
    assert_eq!(std::fs::read(destination.join("file.txt")).expect("replaced"), b"the copy", "the object the pass examined is replaced");
    assert!(!destination.join(format!("file.txt{PARKED_SUFFIX}")).exists(), "the parked entry the pass examined is removed");
    remove(parent.as_path());
}
//endregion

//region The three tests of the review of cd7206407c, reproduced as they were written
// They were applied to a disposable clone of that tip and run first; each failed there.

/// The review, finding 1:
/// an in-place write to a destination file survives a concurrent copy.
#[test]
fn review_repro_destination_written_in_place_during_copy_is_kept() {
    let (parent, source, destination) = trees("review-repro-in-place-destination");
    let source_note: PathBuf = source.join("person-notes.txt");
    let destination_note: PathBuf = destination.join("person-notes.txt");
    std::fs::write(source_note.as_path(), b"source words\n").expect("source note");
    std::fs::write(destination_note.as_path(), b"original destination\n").expect("destination note");
    set_mtime(destination_note.as_path(), BEFORE_EVERY_CUTOFF);
    set_mtime(source_note.as_path(), BEFORE_EVERY_CUTOFF + 1);
    let concurrent_text: &[u8] = b"words written in place during the copy\n";
    let written_into: PathBuf = destination_note.clone();
    let platform: crate::subvolume_test_support::Hooked = crate::subvolume_test_support::Hooked::new(Box::new(move |_calls| {
        std::fs::write(written_into.as_path(), concurrent_text).expect("in-place destination write");
    }));
    let options: SyncOptions<'_> = SyncOptions { delete: false, update_only: true, excluded: &[], created_since: None };
    let stats: SyncStats = sync_tree(&platform, source.as_path(), destination.as_path(), options, &Interrupt::never(), None).expect("a pass");
    println!("destination after publication: {:?}; kept report: {:?}", std::fs::read(destination_note.as_path()), stats.kept_at_the_destination);
    assert_eq!(std::fs::read(destination_note.as_path()).expect("destination after publication"), concurrent_text);
    remove(parent.as_path());
}

/// The review, finding 2:
/// replacing the source path while its old inode is copied cannot lose the replacement.
#[test]
fn review_repro_source_replaced_during_copy_is_not_deleted() {
    let (parent, root) = checkout_with_worktree("review-repro-source-replacement");
    let mut platform: crate::subvolume_test_support::Hooked = crate::subvolume_test_support::Hooked::new(Box::new(|_calls| {}));
    let (migration_code, migration_said) = run_hooked(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migration_code, DONE, "{migration_said}");
    let layout = layout_of(root.as_path()).expect("layout");
    let note: PathBuf = layout.old_path.join("person-notes.txt");
    std::fs::write(note.as_path(), b"the inode the pass opened\n").expect("source note");
    let replacement: PathBuf = layout.old_path.join("replacement.txt");
    let replaced_note: PathBuf = note.clone();
    let replacement_text: &[u8] = b"the replacement written while the old inode was copied\n";
    platform.on_clone = Box::new(move |_calls| {
        std::fs::write(replacement.as_path(), replacement_text).expect("replacement note");
        std::fs::rename(replacement.as_path(), replaced_note.as_path()).expect("replace the source path");
    });
    let (code, said) = run_hooked(&platform, &remove_old_arguments(), root.as_path());
    platform.on_clone = Box::new(|_calls| {});
    let wanted_text: String = String::from_utf8(replacement_text.to_vec()).expect("replacement text");
    let checkout_text: Option<String> = std::fs::read_to_string(root.join("person-notes.txt")).ok();
    let old_tree_text: Option<String> = std::fs::read_to_string(layout.old_path.join("person-notes.txt")).ok();
    println!("removal code {code}; old tree exists {}; checkout holds {checkout_text:?}; old tree holds {old_tree_text:?}; output: {said}", layout.old_path.exists());
    // A hook that replaces the source path on every copy never lets it stand still, so the pass copies it a bounded
    //  number of times and then counts it as an entry that changed while passing. The property is that the replacement
    //  is in one of the two trees and that nothing deleted it.
    assert!(
        checkout_text.as_ref() == Some(&wanted_text) || (code == REFUSED && old_tree_text.as_ref() == Some(&wanted_text)),
        "the replacement is in one of the two trees and nothing deleted it: {said}"
    );
    assert!(code != DONE || checkout_text.as_ref() == Some(&wanted_text), "a finished deletion means the checkout holds the replacement: {said}");
    remove(parent.as_path());
}

/// The review, finding 3:
/// drop a run after its left-out list is durable, then rerun the real removal.
#[test]
fn review_repro_rerun_reads_the_durable_left_out_list() {
    let (parent, root) = checkout_with_worktree("review-repro-left-out-resume");
    let mut platform: crate::subvolume_test_support::Hooked = crate::subvolume_test_support::Hooked::new(Box::new(|_calls| {}));
    let (migration_code, migration_said) = run_hooked(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migration_code, DONE, "{migration_said}");
    let layout = layout_of(root.as_path()).expect("layout");
    let note: PathBuf = layout.old_path.join("person-notes.txt");
    std::fs::write(note.as_path(), b"source words that still exist only in the old tree\n").expect("source note");
    let destination_note: PathBuf = root.join("person-notes.txt");
    let concurrent_text: &[u8] = b"a newer destination file created during the copy\n";
    let written_into: PathBuf = destination_note.clone();
    platform.on_clone = Box::new(move |_calls| {
        std::fs::write(written_into.as_path(), concurrent_text).expect("concurrent destination file");
    });
    let interrupt: Interrupt = Interrupt::never();
    let mut lines: Vec<String> = Vec::new();
    let mut say = |text: &str| lines.push(text.to_string());
    let mut first_run: crate::subvolume_run::Run<'_> = crate::subvolume_run::Run::open(&platform, &layout, &interrupt, &mut say).expect("open the first run");
    let first_stats: SyncStats = first_run.pass_late("late writes", layout.old_path.as_path(), layout.root.as_path(), i64::MIN).expect("the late pass");
    assert_eq!(first_stats.kept_at_the_destination, vec![PathBuf::from("person-notes.txt")], "{first_stats:#?}");
    drop(first_run);
    drop(say);
    assert!(std::fs::read_to_string(layout.left_out_path.as_path()).expect("durable left-out list").contains("person-notes.txt"));
    platform.on_clone = Box::new(|_calls| {});
    let (code, said) = run_hooked(&platform, &remove_old_arguments(), root.as_path());
    println!("first pass kept {:?}; second removal code {code}; old tree exists {}; output: {said}", first_stats.kept_at_the_destination, layout.old_path.exists());
    assert_eq!(code, REFUSED, "the persisted uncopied entry must hold back deletion: {said}");
    assert!(layout.old_path.exists());
    remove(parent.as_path());
}
//endregion

/// What:
///  A kill at the instruction after the record of an uncarried entry was written:
///  the next command learns nothing new there and still refuses,
///  because the deletion asks the record.
/// Why:
///  The deletion of the old tree is the one act that cannot be taken back,
///  so every instruction between the pass learning an entry and that act has to leave the fact behind.
///  The record is written where the entry is learned, before the pass reaches a checkpoint,
///  so this is the first instruction a kill can take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await passLate({interruptAt: 'copy-kept'}); // killed; then removeOld() must refuse
/// ```
#[test]
fn a_kill_after_the_record_of_an_uncarried_entry_still_refuses() {
    let (parent, root) = checkout_with_worktree("kill-after-the-record");
    let mut platform: crate::subvolume_test_support::Hooked = crate::subvolume_test_support::Hooked::new(Box::new(|_calls| {}));
    let (migration_code, migration_said) = run_hooked(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migration_code, DONE, "{migration_said}");
    let layout = layout_of(root.as_path()).expect("layout");
    let note: PathBuf = layout.old_path.join("person-notes.txt");
    std::fs::write(note.as_path(), b"only in the old tree, and newer\n").expect("a note in the old tree");
    // A newer destination file takes the name while the copy runs, so the pass keeps the destination's and records it.
    let destination: PathBuf = root.join("person-notes.txt");
    let concurrent_text: &[u8] = b"a newer file that takes the name during the copy\n";
    let written_into: PathBuf = destination.clone();
    platform.on_clone = Box::new(move |_calls| {
        std::fs::write(written_into.as_path(), concurrent_text).expect("the concurrent destination file");
    });
    let interrupt: Interrupt = Interrupt::at("copy-kept", 0);
    let mut lines: Vec<String> = Vec::new();
    let mut say = |text: &str| lines.push(text.to_string());
    let mut first_run: crate::subvolume_run::Run<'_> = crate::subvolume_run::Run::open(&platform, &layout, &interrupt, &mut say).expect("open the first run");
    let killed: Result<SyncStats, crate::subvolume_error::MigrateError> = first_run.pass_late("late writes", layout.old_path.as_path(), layout.root.as_path(), i64::MIN);
    assert!(killed.is_err(), "the pass stopped at the instruction after the record was written: {lines:#?}");
    drop(first_run);
    drop(say);
    let record: String = std::fs::read_to_string(layout.left_out_path.as_path()).expect("the record beside the checkout");
    assert!(record.contains("person-notes.txt"), "the record is durable at that instruction: {record:?}");
    platform.on_clone = Box::new(|_calls| {});
    let (code, said) = run_hooked(&platform, &remove_old_arguments(), root.as_path());
    assert_eq!(code, REFUSED, "{said}");
    assert!(layout.old_path.join("person-notes.txt").is_file() && destination.is_file(), "the old tree and the checkout keep their files");
    remove(parent.as_path());
}

//region What the four findings are held by, beside the review's own tests

/// The review, finding 4:
/// a file put at the name while the pass held it freed stays,
/// and the entry the pass had examined is not lost from the tree.
#[test]
fn review_repro_a_file_put_at_the_freed_name_is_kept() {
    let (parent, source, destination) = trees("review-repro-freed-name");
    let source_note: PathBuf = source.join("person-notes.txt");
    let destination_note: PathBuf = destination.join("person-notes.txt");
    std::fs::write(source_note.as_path(), b"source words\n").expect("source note");
    std::fs::write(destination_note.as_path(), b"original destination\n").expect("destination note");
    set_mtime(destination_note.as_path(), BEFORE_EVERY_CUTOFF);
    set_mtime(source_note.as_path(), BEFORE_EVERY_CUTOFF + 1);
    let concurrent_text: &[u8] = b"a file put there while the name stood free\n";
    let placed_into: PathBuf = destination_note.clone();
    let once: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    {
        let only_once = once.clone();
        watch_the_freed_name(Some(Box::new(move |freed: &Path| {
            if only_once.swap(true, Ordering::SeqCst) {
                return;
            }
            std::fs::write(freed, concurrent_text).expect("a file at the freed name");
        })));
    }
    let options: SyncOptions<'_> = SyncOptions { delete: false, update_only: true, excluded: &[], created_since: None };
    let stats: SyncStats = sync_tree(&PlainPlatform::new(), source.as_path(), destination.as_path(), options, &Interrupt::never(), None).expect("a pass");
    watch_the_freed_name(None);
    assert_eq!(std::fs::read(destination_note.as_path()).expect("the destination after the pass"), concurrent_text, "the file put at the freed name is not replaced: {stats:#?}");
    assert!(!stats.kept_at_the_destination.is_empty(), "the pass says it did not place the source's copy: {stats:#?}");
    assert_eq!(std::fs::read(placed_into.with_file_name(format!("person-notes.txt{PARKED_SUFFIX}"))).expect("the entry the pass examined"), b"original destination\n", "the entry the pass had examined is not lost");
    remove(parent.as_path());
}

/// A parked entry a killed pass left behind is named by the next command and never deleted:
/// the deletion of the old tree waits for it.
#[test]
fn a_parked_entry_left_by_a_kill_holds_the_deletion_back() {
    let (parent, root) = checkout_with_worktree("parked-entry-left");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migration_code, migration_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migration_code, DONE, "{migration_said}");
    let parked: PathBuf = root.join(format!("person-notes.txt{PARKED_SUFFIX}"));
    std::fs::write(parked.as_path(), b"what a killed replacement had parked\n").expect("a parked entry");
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    assert_eq!(code, REFUSED, "{said}");
    assert!(said.contains("person-notes.txt") && said.contains(PARKED_SUFFIX), "the refusal names the parked entry: {said}");
    assert!(parked.is_file(), "nothing deletes a parked entry");
    remove(parent.as_path());
}

/// A record an earlier run wrote is read by a run that has passed nothing:
/// the deletion asks the durable record, not only its own memory.
#[test]
fn the_deletion_reads_the_durable_record_of_an_earlier_run() {
    let (parent, root) = checkout_with_worktree("durable-record-fresh-run");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migration_code, migration_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migration_code, DONE, "{migration_said}");
    let layout = layout_of(root.as_path()).expect("layout");
    std::fs::write(layout.old_path.join("person-notes.txt"), b"only in the old tree\n").expect("a note in the old tree");
    std::fs::write(layout.left_out_path.as_path(), b"person-notes.txt\n").expect("the record an earlier run wrote");
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    assert_eq!(code, REFUSED, "a run that passed nothing must still read the record: {said}");
    assert!(said.contains("person-notes.txt"), "{said}");
    assert!(layout.old_path.join("person-notes.txt").is_file(), "nothing was deleted");
    remove(parent.as_path());
}

/// A source path replaced while its inode is copied is reported by the pass,
/// so the entry that changed while passing keeps the old tree.
#[test]
fn a_source_path_replaced_during_the_copy_is_counted_as_another_kind() {
    let (parent, source, destination) = trees("source-replaced-pass");
    let note: PathBuf = source.join("person-notes.txt");
    std::fs::write(note.as_path(), b"the inode the pass opens\n").expect("source note");
    let replacement: PathBuf = source.join("replacement.txt");
    let replaced: PathBuf = note.clone();
    let mut platform: crate::subvolume_test_support::Hooked = crate::subvolume_test_support::Hooked::new(Box::new(move |_calls| {
        std::fs::write(replacement.as_path(), b"the replacement written during the copy\n").expect("replacement");
        std::fs::rename(replacement.as_path(), replaced.as_path()).expect("replace the source path");
    }));
    let options: SyncOptions<'_> = SyncOptions { delete: false, update_only: false, excluded: &[], created_since: None };
    let stats: SyncStats = sync_tree(&platform, source.as_path(), destination.as_path(), options, &Interrupt::never(), None).expect("a pass");
    platform.on_clone = Box::new(|_calls| {});
    assert_eq!(stats.changed_while_passing, vec![PathBuf::from("person-notes.txt")], "{stats:#?}");
    assert!(!destination.join("person-notes.txt").exists(), "the copy of the inode that no longer stands there is not placed: {stats:#?}");
    assert_eq!(std::fs::read(note.as_path()).expect("the replacement at the source path"), b"the replacement written during the copy\n");
    remove(parent.as_path());
}
//endregion

//region The three reproductions of the review of 28254ad188, as they were written
// They were run first against that tip in a disposable clone and failed there, as the review reports.

/// The review, finding 1:
/// a source write after the last source check is not in the published copy.
#[test]
fn review_repro_source_write_after_the_last_source_check_is_not_in_the_published_copy() {
    let (parent, source, destination) = trees("review-source-write-after-check");
    let source_file: PathBuf = source.join("file.txt");
    let destination_file: PathBuf = destination.join("file.txt");
    std::fs::write(source_file.as_path(), b"old source\n").expect("source");
    std::fs::write(destination_file.as_path(), b"old dest\n").expect("destination");
    set_mtime(source_file.as_path(), 2_000_000_000);
    set_mtime(destination_file.as_path(), 1_000_000_000);
    let during_freed_name: PathBuf = source_file.clone();
    watch_the_freed_name(Some(Box::new(move |_freed: &Path| {
        std::fs::write(during_freed_name.as_path(), b"new source written after the source check\n").expect("concurrent source write");
    })));
    let options: SyncOptions<'_> = SyncOptions { delete: false, update_only: true, excluded: &[], created_since: None };
    let stats: SyncStats = sync_tree(&PlainPlatform::new(), source.as_path(), destination.as_path(), options, &Interrupt::never(), None).expect("a pass");
    watch_the_freed_name(None);
    println!("source={:?} destination={:?} stats={stats:?}", std::fs::read(source_file.as_path()), std::fs::read(destination_file.as_path()));
    assert_eq!(std::fs::read(destination_file.as_path()).expect("destination"), b"new source written after the source check\n");
    remove(parent.as_path());
}

/// The review, finding 2:
/// a destination write of the same size and timestamp is not deleted as superseded.
#[test]
fn review_repro_same_size_and_timestamp_destination_write_is_not_deleted() {
    let (parent, source, destination) = trees("review-destination-same-metadata");
    let source_file: PathBuf = source.join("file.txt");
    let destination_file: PathBuf = destination.join("file.txt");
    std::fs::write(source_file.as_path(), b"source\n").expect("source");
    std::fs::write(destination_file.as_path(), b"old words\n").expect("destination");
    set_mtime(source_file.as_path(), 2_000_000_000);
    set_mtime(destination_file.as_path(), 1_000_000_000);
    let parked: PathBuf = destination_file.with_file_name(format!("file.txt{PARKED_SUFFIX}"));
    let during_freed_name: PathBuf = parked.clone();
    watch_the_freed_name(Some(Box::new(move |_freed: &Path| {
        std::fs::write(during_freed_name.as_path(), b"new words\n").expect("same-size concurrent write");
        set_mtime(during_freed_name.as_path(), 1_000_000_000);
    })));
    let options: SyncOptions<'_> = SyncOptions { delete: false, update_only: true, excluded: &[], created_since: None };
    let stats: SyncStats = sync_tree(&PlainPlatform::new(), source.as_path(), destination.as_path(), options, &Interrupt::never(), None).expect("a pass");
    watch_the_freed_name(None);
    println!("destination={:?} parked_exists={} stats={stats:?}", std::fs::read(destination_file.as_path()), parked.exists());
    assert_eq!(std::fs::read(destination_file.as_path()).expect("destination"), b"new words\n");
    remove(parent.as_path());
}

/// The review, finding 3:
/// an interrupted second pass keeps the first pass's left-out record.
#[test]
fn review_repro_an_interrupted_second_pass_keeps_the_first_pass_left_out_record() {
    let (parent, root, old_tree) = trees("review-record-overwrite");
    let other_source: PathBuf = parent.join("other-source");
    std::fs::create_dir(other_source.as_path()).expect("other source");
    let first: PathBuf = old_tree.join("first-left-out.txt");
    std::fs::write(first.as_path(), b"only in the first old tree\n").expect("first entry");
    set_mtime(first.as_path(), 1_000_000_000);
    let interrupt: Interrupt = Interrupt::at("copy-partial", 0);
    let platform: PlainPlatform = PlainPlatform::new();
    let layout = layout_of(root.as_path()).expect("layout");
    let mut output: String = String::new();
    let mut say = |line: &str| output.push_str(line);
    let mut run: crate::subvolume_run::Run<'_> = crate::subvolume_run::Run::open(&platform, &layout, &interrupt, &mut say).expect("run");
    run.pass_late("first", old_tree.as_path(), root.as_path(), 2_000_000_000).expect("first pass");
    assert_eq!(std::fs::read_to_string(layout.left_out_path.as_path()).expect("first record"), "first-left-out.txt\n");
    let second_left_out: PathBuf = other_source.join("second-left-out.txt");
    let second_copied: PathBuf = other_source.join("second-copied.txt");
    std::fs::write(second_left_out.as_path(), b"left out in the interrupted pass\n").expect("second left-out entry");
    std::fs::write(second_copied.as_path(), b"copy stops here\n").expect("second copied entry");
    let first_in_listing: PathBuf = std::fs::read_dir(other_source.as_path()).expect("read order").next().expect("first directory entry").expect("first entry").path();
    let (old_entry, new_entry) = if first_in_listing == second_left_out { (second_left_out.clone(), second_copied.clone()) } else { (second_copied.clone(), second_left_out.clone()) };
    set_mtime(old_entry.as_path(), 1_000_000_000);
    set_mtime(new_entry.as_path(), 3_000_000_000);
    let error: crate::subvolume_error::MigrateError = run.pass_late("second", other_source.as_path(), root.as_path(), 2_000_000_000).expect_err("interrupt");
    println!("interrupt={error:?} record={:?}", std::fs::read_to_string(layout.left_out_path.as_path()));
    assert_eq!(std::fs::read_to_string(layout.left_out_path.as_path()).expect("record after interrupt"), "first-left-out.txt\nsecond-left-out.txt\n");
    remove(parent.as_path());
}
//endregion

//region The walk of what the other party writes, at every instruction a control can stand in

/// What:
///  The seams a control can stand in while one file is copied and placed,
///  named as the instruction they stand at.
/// Why:
///   A look and an act are two steps,
///  so what a control can prove is that no write of the other party is lost without a record,
///  and that has to hold at every instruction a control can reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seams = ['after-the-listing', 'while-the-copy-runs', 'while-the-name-is-freed', 'before-the-judgement'];
/// ```
const SEAMS_WHILE_ONE_ENTRY_IS_PLACED: [&str; 3] = ["after-the-listing", "while-the-copy-runs", "while-the-name-is-freed"];

/// Install a writer at one named seam that changes `path` to `text`, once.
fn write_at(seam: &str, path: PathBuf, text: &'static [u8], platform: &mut crate::subvolume_test_support::Hooked) {
    let once: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    match seam {
        "after-the-listing" => {
            let only_once = once.clone();
            let target: PathBuf = path.clone();
            watch_entries(Some(Box::new(move |examined: &Path| {
                if examined != target.as_path() || only_once.swap(true, Ordering::SeqCst) {
                    return;
                }
                std::fs::write(target.as_path(), text).expect("a write after the listing");
            })));
        }
        "while-the-copy-runs" => {
            let only_once = once.clone();
            let target: PathBuf = path.clone();
            platform.on_clone = Box::new(move |_calls: u32| {
                if only_once.swap(true, Ordering::SeqCst) {
                    return;
                }
                std::fs::write(target.as_path(), text).expect("a write while the copy runs");
            });
        }
        "while-the-name-is-freed" => {
            let only_once = once.clone();
            let target: PathBuf = path.clone();
            watch_the_freed_name(Some(Box::new(move |_freed: &Path| {
                if only_once.swap(true, Ordering::SeqCst) {
                    return;
                }
                std::fs::write(target.as_path(), text).expect("a write while the name is freed");
            })));
        }
        other => panic!("no seam named {other}"),
    }
}

/// Clear every seam a walk may have installed.
fn clear_the_seams() {
    watch_entries(None);
    watch_the_freed_name(None);
    watch_before_the_judgement(None);
}

/// The review, finding 1, walked:
/// a source write at each instruction a control can stand in is either in the published copy or recorded,
/// so no byte the other party wrote is lost without a record.
#[test]
fn review_walk_a_source_write_at_every_instruction_is_carried_or_recorded() {
    let written: &[u8] = b"the other party's newer bytes\n";
    for seam in SEAMS_WHILE_ONE_ENTRY_IS_PLACED {
        let (parent, source, destination) = trees(format!("walk-source-{seam}").as_str());
        let source_file: PathBuf = source.join("file.txt");
        let destination_file: PathBuf = destination.join("file.txt");
        std::fs::write(source_file.as_path(), b"old source\n").expect("source");
        std::fs::write(destination_file.as_path(), b"old dest\n").expect("destination");
        set_mtime(source_file.as_path(), 2_000_000_000);
        set_mtime(destination_file.as_path(), 1_000_000_000);
        let mut platform: crate::subvolume_test_support::Hooked = crate::subvolume_test_support::Hooked::new(Box::new(|_calls| {}));
        write_at(seam, source_file.clone(), written, &mut platform);
        let options: SyncOptions<'_> = SyncOptions { delete: false, update_only: true, excluded: &[], created_since: None };
        let stats: SyncStats = sync_tree(&platform, source.as_path(), destination.as_path(), options, &Interrupt::never(), None).expect("a pass");
        clear_the_seams();
        let at_the_destination: Vec<u8> = std::fs::read(destination_file.as_path()).unwrap_or_default();
        let recorded: bool = stats.changed_while_passing.iter().any(|path| return path == Path::new("file.txt"))
            || stats.left_out.iter().any(|path| return path == Path::new("file.txt"))
            || stats.kept_at_the_destination.iter().any(|path| return path == Path::new("file.txt"));
        println!("seam {seam}: destination={:?} recorded={recorded} stats={stats:?}", at_the_destination);
        assert!(at_the_destination == written || recorded, "at the seam {seam} the source's newer bytes are neither at the destination nor recorded: {stats:#?}");
        remove(parent.as_path());
    }
}

/// The review, finding 2, walked:
/// a write to the entry the pass parked, at each instruction a control can stand in, is kept or recorded,
/// so no byte the other party wrote is deleted unseen.
#[test]
fn review_walk_a_write_to_the_parked_entry_at_every_instruction_is_kept_or_recorded() {
    let written: &[u8] = b"new words\n";
    for seam in ["while-the-name-is-freed", "before-the-judgement"] {
        let (parent, source, destination) = trees(format!("walk-parked-{seam}").as_str());
        let source_file: PathBuf = source.join("file.txt");
        let destination_file: PathBuf = destination.join("file.txt");
        std::fs::write(source_file.as_path(), b"source\n").expect("source");
        std::fs::write(destination_file.as_path(), b"old words\n").expect("destination");
        set_mtime(source_file.as_path(), 2_000_000_000);
        set_mtime(destination_file.as_path(), 1_000_000_000);
        let parked: PathBuf = destination_file.with_file_name(format!("file.txt{PARKED_SUFFIX}"));
        let mut platform: crate::subvolume_test_support::Hooked = crate::subvolume_test_support::Hooked::new(Box::new(|_calls| {}));
        let once: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        if seam == "while-the-name-is-freed" {
            write_at(seam, parked.clone(), written, &mut platform);
            let settled: PathBuf = parked.clone();
            watch_the_freed_name(Some(Box::new(move |_freed: &Path| {
                // Keep the size and the time the examination saw, which is what makes the write hard to tell.
                std::fs::write(settled.as_path(), written).expect("the same-size write");
                set_mtime(settled.as_path(), 1_000_000_000);
            })));
        } else {
            let only_once = once.clone();
            let target: PathBuf = parked.clone();
            watch_before_the_judgement(Some(Box::new(move |_parked: &Path| {
                if only_once.swap(true, Ordering::SeqCst) {
                    return;
                }
                std::fs::write(target.as_path(), written).expect("the same-size write");
                set_mtime(target.as_path(), 1_000_000_000);
            })));
        }
        let options: SyncOptions<'_> = SyncOptions { delete: false, update_only: true, excluded: &[], created_since: None };
        let stats: SyncStats = sync_tree(&platform, source.as_path(), destination.as_path(), options, &Interrupt::never(), None).expect("a pass");
        clear_the_seams();
        let at_the_destination: Vec<u8> = std::fs::read(destination_file.as_path()).unwrap_or_default();
        let at_the_parked_name: Vec<u8> = std::fs::read(parked.as_path()).unwrap_or_default();
        let recorded: bool = stats.kept_at_the_destination.iter().any(|path| return path == Path::new("file.txt") || path == &PathBuf::from(format!("file.txt{PARKED_SUFFIX}")))
            || stats.changed_while_passing.iter().any(|path| return path == Path::new("file.txt"));
        println!("seam {seam}: destination={:?} parked={:?} recorded={recorded} stats={stats:?}", at_the_destination, at_the_parked_name);
        assert!(
            at_the_destination == written || (at_the_parked_name == written && recorded),
            "at the seam {seam} the other party's bytes are neither at the destination nor parked and recorded: {stats:#?}"
        );
        remove(parent.as_path());
    }
}
//endregion
