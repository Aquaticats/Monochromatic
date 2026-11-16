//! Controls of the rule that a person's bytes are deleted only when they are proved to stand in the tree that is kept.
//! The generated control walks every way a name reaches the new tree against every way the old file changes, for both
//! commands that delete something; the rest hold one finding each of the review of `6fbbed8e84`: a name too long to
//! park, a listed file turned into a pipe, a record a rollback wrote, a person's file whose name looks like one this
//! program parked, a parked copy a kill left and the words that name it, and a file another command puts at a name
//! while the pass puts its parked copy back.

use crate::subvolume_arguments::{MigrateArguments, MigrateMode};
use crate::subvolume_error::Interrupt;
use crate::subvolume_preflight::layout_of;
use crate::subvolume_sync::watch_entries;
use crate::subvolume_sync_pass_state_and_checks::{watch_before_the_judgement, watch_before_the_put_back, watch_the_freed_name};
use crate::subvolume_test_support::{PlainPlatform, checkout_with_worktree, migrate_arguments, run_command};
use crate::test_support::remove;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

/// The exit status a finished command gives.
const DONE: i32 = 0;

/// The exit status of a refusal.
const REFUSED: i32 = 1;

/// The exit status of a failure after the journal began.
const FAILED: i32 = 2;

/// How long a command may take in a control that is watching for a wait.
const A_BOUND_THIS_CONTROL_OWNS: Duration = Duration::from_secs(20);

/// Set both times of a path to whole seconds after the epoch.
fn set_seconds(path: &Path, seconds: i64) {
    use std::os::unix::ffi::OsStrExt;
    let times: [libc::timespec; 2] = [libc::timespec { tv_sec: seconds, tv_nsec: 0 }, libc::timespec { tv_sec: seconds, tv_nsec: 0 }];
    let name = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("path text");
    // SAFETY: `name` is NUL-terminated and `times` holds two values; both outlive the call.
    let result: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) };
    assert_eq!(result, 0, "set the time of {}", path.display());
}

/// The arguments of the deletion of the old tree.
fn remove_old_arguments() -> MigrateArguments {
    return MigrateArguments { mode: MigrateMode::RemoveOld, ..migrate_arguments() };
}

/// The arguments of the deletion of the old tree that also discards what the carry left out.
fn remove_old_discarding_left_out() -> MigrateArguments {
    return MigrateArguments { discard_left_out: true, ..remove_old_arguments() };
}

/// The arguments of the rollback.
fn rollback_arguments() -> MigrateArguments {
    return MigrateArguments { mode: MigrateMode::Rollback, ..migrate_arguments() };
}

/// The arguments of the rollback that also discards what the carry back left out.
fn rollback_discarding_left_out() -> MigrateArguments {
    return MigrateArguments { discard_left_out: true, ..rollback_arguments() };
}

/// Run one command to completion and answer its status and everything it said.
fn run(platform: &PlainPlatform, arguments: &MigrateArguments, root: &Path) -> (i32, String) {
    let (code, lines) = run_command(platform, arguments, root, &Interrupt::never());
    return (code, lines.join("\n"));
}

/// Whether the file at `path` holds exactly `bytes`.
fn holds(path: &Path, bytes: &[u8]) -> bool {
    return std::fs::read(path).map(|found| return found == bytes).unwrap_or(false);
}

/// What:
///  The ways a name reaches the new tree, which the rule holds to one proof.
/// Why:
///   The review named them: a copy of a regular file, a name linked to an earlier copy, a file that equals its copy at
///  the listing, and an entry put back after its judgement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Way = 'copied' | 'linked' | 'equal at the listing' | 'put back';
/// ```
const WAYS_A_NAME_REACHES_THE_NEW_TREE: [&str; 4] = ["copied", "linked", "equal at the listing", "put back"];

/// The ways the old file changes, which decide what the pass does with it.
const WAYS_THE_OLD_FILE_CHANGES: [&str; 2] = ["older time", "newer time"];

/// The commands that delete a tree.
#[derive(Clone, Copy, PartialEq)]
enum Deletion {
    /// `--remove-old` deletes the old tree.
    TheOldTree,
    /// `--rollback` deletes the new subvolume.
    TheNewSubvolume,
}

impl Deletion {
    /// The arguments of this command.
    fn arguments(self) -> MigrateArguments {
        return match self {
            Deletion::TheOldTree => remove_old_arguments(),
            Deletion::TheNewSubvolume => rollback_arguments(),
        };
    }

    /// The tree this command deletes,
    /// which is the tree whose bytes have to be proved to stand in the other one.
    fn deleted(self, old_path: &Path, new_path: &Path) -> PathBuf {
        return match self {
            Deletion::TheOldTree => old_path.to_path_buf(),
            Deletion::TheNewSubvolume => new_path.to_path_buf(),
        };
    }

    /// The name of this command, for a control's case name.
    fn name(self) -> &'static str {
        return match self {
            Deletion::TheOldTree => "--remove-old",
            Deletion::TheNewSubvolume => "--rollback",
        };
    }
}

/// What:
///  One cell of the generated control: a way a name reaches the new tree, a way the old file changes, and the command.
/// Why:
///   The rule is one for every cell, so each cell asserts the same thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cell = { way: string; change: string; deletion: Deletion };
/// ```
struct Cell<'a> {
    way: &'a str,
    change: &'a str,
    deletion: Deletion,
}

/// What:
///  Every way a name reaches the new tree against every way the old file changes, for both commands that delete a tree.
///  After the command the newest bytes of the changed file are in a tree the person keeps,
///  or the command refused and named the path,
///  and a status of 0 with the newest bytes gone never happens.
/// Why:
///   This is the rule of `SPEC.md`, "What never discarding a person's data means", as one assertion per cell:
///  the old tree, or any file of it, is deleted only when every byte it holds is proved to be in the new one,
///  by the same proof for every way a name got there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const way of ways) for (const change of changes) for (const deletion of deletions) assertKeptOrTold(cell);
/// ```
#[test]
fn review_generated_no_way_of_placing_a_name_loses_the_newest_bytes_unless_it_is_told() {
    let mut failed: Vec<String> = Vec::new();
    for way in WAYS_A_NAME_REACHES_THE_NEW_TREE {
        for change in WAYS_THE_OLD_FILE_CHANGES {
            for deletion in [Deletion::TheOldTree, Deletion::TheNewSubvolume] {
                let cell: Cell<'_> = Cell { way, change, deletion };
                if let Err(reason) = one_cell(&cell) {
                    failed.push(format!("{} / {} / {}: {reason}", cell.way, cell.change, cell.deletion.name()));
                }
            }
        }
    }
    assert!(failed.is_empty(), "{} of {} cells lost a person's bytes without telling:\n{}", failed.len(), 16, failed.join("\n"));
}

/// What:
///  One cell of the generated control:
///  a migrated fixture,
///  the old tree's file prepared for the way,
///  the change made at that way's seam,
///  and the assertion on what the command left.
/// Why:
///   See `review_generated_no_way_of_placing_a_name_loses_the_newest_bytes_unless_it_is_told`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cell = await arrangeAndRun(way, change, deletion); expectKeptOrTold(cell);
/// ```
fn one_cell(cell: &Cell<'_>) -> Result<(), String> {
    let name: String = format!("generated-{}-{}-{}", cell.way.replace(' ', "-"), cell.change.replace(' ', "-"), cell.deletion.name().trim_start_matches('-'));
    let (parent, root) = checkout_with_worktree(name.as_str());
    let platform: PlainPlatform = PlainPlatform::new();
    if cell.way == "linked" {
        // Two names of one file, in the checkout before the migration, so both trees hold them.
        std::fs::write(root.join("h1"), b"one file, two names\n").expect("h1");
        std::fs::hard_link(root.join("h1"), root.join("h2")).expect("h2");
    }
    let (code, said) = run(&platform, &migrate_arguments(), root.as_path());
    if code != DONE {
        remove(parent.as_path());
        return Err(format!("the migration failed: {said}"));
    }
    let layout = layout_of(root.as_path()).expect("layout");
    // The old file's newest bytes, which the pass has to prove or tell.
    let newest: Vec<u8> = format!("NEWEST-{}-{}\n", cell.way, cell.change).into_bytes();
    let older_than_the_copy: i64 = 1_000_000_000;
    let newer_than_the_copy: i64 = 3_000_000_000;
    let at_that_time: i64 = match cell.change {
        "older time" => older_than_the_copy,
        _ => newer_than_the_copy,
    };
    let target: &str = if cell.way == "linked" { "h2" } else { "README.md" };
    let seam: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    match cell.way {
        "copied" | "linked" => {
            // The old tree's file is changed before the command, and the pass finds a newer or an older file.
            std::fs::write(layout.old_path.join(target), newest.as_slice()).map_err(|error| return error.to_string())?;
            set_seconds(layout.old_path.join(target).as_path(), at_that_time);
        }
        "equal at the listing" => {
            // The file equals its copy when the pass lists its folder; the change lands while the pass is there,
            //  for an entry it handles before this one.
            let old_tree: PathBuf = layout.old_path.clone();
            let changed_path: PathBuf = old_tree.join(target);
            let only_once: Arc<AtomicBool> = seam.clone();
            let newest_written: Vec<u8> = newest.clone();
            watch_entries(Some(Box::new(move |examined: &Path| {
                // The pass hands the walk each entry of a folder after the whole folder was listed, so a write here lands
                //  after the listing and before this entry is handled.
                let is_a_later_entry: bool = examined.file_name().map(|part| return part == "src").unwrap_or(false);
                if !is_a_later_entry || only_once.swap(true, Ordering::SeqCst) {
                    return;
                }
                std::fs::write(changed_path.as_path(), newest_written.as_slice()).expect("a write while the pass is in the folder");
                set_seconds(changed_path.as_path(), at_that_time);
            })));
        }
        "put back" => {
            // The destination holds an entry the pass replaces, and the entry it parks is written while it is parked,
            //  so the judgement puts it back instead of placing the copy.
            std::fs::write(layout.old_path.join(target), newest.as_slice()).map_err(|error| return error.to_string())?;
            set_seconds(layout.old_path.join(target).as_path(), at_that_time);
            let only_once: Arc<AtomicBool> = seam.clone();
            watch_the_freed_name(Some(Box::new(move |_freed: &Path| {
                if only_once.swap(true, Ordering::SeqCst) {
                    return;
                }
                // Nothing here: the parked entry is written by the seam below, which needs its name.
            })));
            let once_for_the_judgement: Arc<AtomicBool> = seam.clone();
            let wanted: std::ffi::OsString = crate::subvolume_sync_pass_state_and_checks::parked_name(Path::new(target));
            watch_before_the_judgement(Some(Box::new(move |parked: &Path| {
                if parked.file_name() != Some(wanted.as_os_str()) || once_for_the_judgement.swap(true, Ordering::SeqCst) {
                    return;
                }
                std::fs::write(parked, b"THE-OTHER-PARTYS-WRITE\n").expect("a write into the parked entry");
            })));
        }
        other => return Err(format!("no way named {other}")),
    }
    let (status, run_said) = run(&platform, &cell.deletion.arguments(), root.as_path());
    watch_entries(None);
    watch_the_freed_name(None);
    watch_before_the_judgement(None);
    let deleted: PathBuf = cell.deletion.deleted(layout.old_path.as_path(), layout.new_path.as_path());
    // The tree the person keeps, and the tree this command deletes while it stands.
    let in_the_checkout: bool = holds(root.join(target).as_path(), newest.as_slice());
    let in_the_deleted_tree: bool = holds(deleted.join(target).as_path(), newest.as_slice());
    let deleted_still_stands: bool = deleted.exists();
    if status == DONE && deleted_still_stands && in_the_deleted_tree {
        // A command that finished deletes the tree it names; if it stands, something else keeps it.
        remove(parent.as_path());
        return Err(format!("status 0 but {} still stands: {run_said}", deleted.display()));
    }
    if status == DONE && !in_the_checkout {
        remove(parent.as_path());
        return Err(format!("status 0 and the newest bytes are nowhere a person keeps: {run_said}"));
    }
    let told: bool = status != DONE && run_said.contains(target);
    if !in_the_checkout && !told {
        remove(parent.as_path());
        return Err(format!("status {status} and nothing named {target}: {run_said}"));
    }
    remove(parent.as_path());
    return Ok(());
}

//region One control per finding of the review of 6fbbed8e84

/// The review, finding 1:
/// a file whose name is long enough to make the parked name too long migrates and leaves no lock.
#[test]
fn review_a_name_too_long_to_park_migrates_and_leaves_no_index_lock() {
    let (parent, root) = checkout_with_worktree("long-name");
    let platform: PlainPlatform = PlainPlatform::new();
    let name: String = format!("{}.txt", "n".repeat(240));
    std::fs::write(root.join(name.as_str()), b"one\n").expect("a long-named file");
    let (added, added_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(added, DONE, "{added_said}");
    let layout = layout_of(root.as_path()).expect("layout");
    // A file of the old tree that changes after the catch-up is carried by the late-write carry, which replaces the
    //  checkout's copy, and that is the step that has to park a name this long.
    std::fs::write(layout.old_path.join(name.as_str()), b"two, written later\n").expect("a later write");
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    println!("removal code {code}; output: {said}");
    assert_ne!(code, FAILED, "the command failed instead of carrying or refusing: {said}");
    assert!(!root.join(".git/index.lock").exists(), "no index lock is left in the checkout");
    assert!(!layout.old_path.join(".git/index.lock").exists(), "and none in the old tree");
    assert!(
        holds(root.join(name.as_str()).as_path(), b"two, written later\n") || (code == REFUSED && said.contains(name.as_str())),
        "the newest bytes are in the checkout or named: {said}"
    );
    remove(parent.as_path());
}

/// The review, finding 3:
/// a listed file turned into a named pipe before it is opened is not waited on.
#[test]
fn review_a_listed_file_turned_into_a_pipe_is_not_waited_on() {
    let (parent, root) = checkout_with_worktree("pipe-in-the-walk");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migrated, migrated_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migrated, DONE, "{migrated_said}");
    let layout = layout_of(root.as_path()).expect("layout");
    // The old tree's b.txt is newer than its copy, so the carry opens it; the seam turns it into a pipe first.
    let pipe: PathBuf = layout.old_path.join("README.md");
    std::fs::write(pipe.as_path(), b"a write before the pipe\n").expect("the file to come");
    let once: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let (sender, receiver): (Sender<(i32, String)>, Receiver<(i32, String)>) = channel();
    let root_for_the_thread: PathBuf = root.clone();
    let pipe_after_the_pass: PathBuf = pipe.clone();
    let run_in_a_thread = std::thread::spawn(move || {
        // The seam is installed in the thread that runs the pass, because the walk's callback belongs to its thread.
        let the_pipe: PathBuf = pipe;
        let only_once: Arc<AtomicBool> = once;
        watch_entries(Some(Box::new(move |examined: &Path| {
            let is_a_later_entry: bool = examined.file_name().map(|part| return part == "src").unwrap_or(false);
            if !is_a_later_entry || only_once.swap(true, Ordering::SeqCst) {
                return;
            }
            std::fs::remove_file(the_pipe.as_path()).expect("remove the file");
            let mut made: std::process::Command = std::process::Command::new("/usr/bin/mkfifo");
            made.arg(the_pipe.as_path()).status().expect("mkfifo");
        })));
        let in_the_thread: PlainPlatform = PlainPlatform::new();
        let answered: (i32, String) = run(&in_the_thread, &remove_old_arguments(), root_for_the_thread.as_path());
        watch_entries(None);
        let _sent = sender.send(answered);
    });
    let answered: Option<(i32, String)> = receiver.recv_timeout(A_BOUND_THIS_CONTROL_OWNS).ok();
    watch_entries(None);
    if answered.is_none() {
        // Release the pass, which is blocked opening the pipe, and fail the control.
        let writer: std::io::Result<std::fs::File> = std::fs::OpenOptions::new().write(true).open(pipe_after_the_pass.as_path());
        drop(writer);
        panic!("the pass waited on the pipe: nothing was answered within {:?}", A_BOUND_THIS_CONTROL_OWNS);
    }
    let (code, said): (i32, String) = answered.expect("answered");
    let _joined = run_in_a_thread.join();
    println!("removal code {code}; output: {said}");
    assert_ne!(code, FAILED, "the pass failed rather than telling: {said}");
    remove(parent.as_path());
}

/// The review, finding 7:
/// the record a rollback wrote does not outlive what it describes.
#[test]
fn review_a_record_a_rollback_wrote_does_not_outlive_it() {
    let (parent, root) = checkout_with_worktree("rollback-record");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migrated, migrated_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migrated, DONE, "{migrated_said}");
    let layout = layout_of(root.as_path()).expect("layout");
    // In the checkout a directory becomes a file, so the carry back keeps the original tree's directory and refuses.
    std::fs::remove_dir_all(root.join("ignored/node_modules").as_path()).expect("remove the directory");
    std::fs::write(root.join("ignored/node_modules"), b"a file now\n").expect("a file instead");
    let (refused, refused_said) = run(&platform, &rollback_arguments(), root.as_path());
    assert_eq!(refused, REFUSED, "{refused_said}");
    let (done, done_said) = run(&platform, &rollback_discarding_left_out(), root.as_path());
    assert_eq!(done, DONE, "{done_said}");
    assert!(!layout.left_out_path.exists(), "the record the rollback wrote is removed with what it describes: {:?}", std::fs::read_to_string(layout.left_out_path.as_path()));
    // A clean migration then says nothing about what the rollback carried back, and its old tree can go.
    let (again, again_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(again, DONE, "{again_said}");
    assert!(!again_said.contains("stayed in the old tree"), "a clean migration reports no stale entry: {again_said}");
    let (removed, removed_said) = run(&platform, &remove_old_arguments(), root.as_path());
    assert_eq!(removed, DONE, "the deletion of a clean migration finishes: {removed_said}");
    remove(parent.as_path());
}

/// The review, finding 9:
/// an ordinary file whose name ends in the parked suffix is a person's file and migrates.
#[test]
fn review_a_persons_file_that_ends_in_the_parked_suffix_migrates() {
    let (parent, root) = checkout_with_worktree("parked-suffix");
    let platform: PlainPlatform = PlainPlatform::new();
    let name: &str = "b.txt.cli-git-migrate-displaced";
    std::fs::write(root.join(name), b"a person's own file\n").expect("a file with that ending");
    let (code, said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(code, DONE, "{said}");
    assert!(!said.contains(".cli-git-migrate-parked-") && !said.contains("this command parked"), "no pass takes a person's file for one this program parked: {said}");
    assert!(holds(root.join(name).as_path(), b"a person's own file\n"));
    let (removed, removed_said) = run(&platform, &remove_old_arguments(), root.as_path());
    assert_eq!(removed, DONE, "the deletion finishes, since the copy equals the original: {removed_said}");
    remove(parent.as_path());
}

/// The review, finding 8:
/// a parked copy a kill left is named where it is and what it is, and nothing is refused for ever by it.
#[test]
fn review_a_parked_copy_a_kill_left_is_named_where_it_is_and_what_it_is() {
    let (parent, root) = checkout_with_worktree("parked-words");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migrated, migrated_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migrated, DONE, "{migrated_said}");
    let layout = layout_of(root.as_path()).expect("layout");
    // The old tree's b.txt is newer, so the carry replaces the checkout's copy; the pass is stopped right after it
    //  parked the entry, which is the instant a kill leaves a parked copy behind.
    std::fs::write(layout.old_path.join("README.md"), b"a later write\n").expect("the later write");
    let (stopped, stopped_said) = run_command(&platform, &remove_old_arguments(), root.as_path(), &Interrupt::at("parked", 0));
    assert_eq!(stopped, FAILED, "the pass stops right after parking: {}", stopped_said.join(" | "));
    let parked: Vec<PathBuf> = parked_copies_in(root.as_path());
    assert_eq!(parked.len(), 1, "one parked copy stands in the checkout: {parked:?}");
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    println!("removal code {code}; output: {said}");
    let words: String = format!("{said}{}", std::fs::read_to_string(layout.left_out_path.as_path()).unwrap_or_default());
    assert!(words.contains("parked"), "the words say the copy was parked: {words}");
    assert!(words.contains("README.md"), "and which file it is a copy of: {words}");
    assert!(words.contains("delete it when nothing in it is yours"), "and what the person can do: {words}");
    assert!(parked[0].is_file(), "nothing deletes the copy a kill parked");
    // The deletion with the flag that accepts what was left out still does not remove the parked copy, and the words
    //  of the run that finishes name it.
    let (finished, finished_said) = run(&platform, &remove_old_discarding_left_out(), root.as_path());
    assert!(finished_said.contains("parked") || finished == REFUSED, "the run that deletes the old tree names the parked copy: {finished_said}");
    let _ = finished;
    // A person who removes the copy is not refused for ever by it.
    std::fs::remove_file(parked[0].as_path()).expect("the person's removal");
    let (after, after_said) = run(&platform, &remove_old_arguments(), root.as_path());
    assert!(!after_said.contains(".cli-git-migrate-parked-"), "the copy a person removed is not named again: {after_said}");
    let _ = after;
    remove(parent.as_path());
}

/// The review, finding 6:
/// a file another command puts at a name while the pass puts its parked copy back is not replaced.
#[test]
fn review_a_file_put_at_the_name_during_the_put_back_is_kept() {
    let (parent, root) = checkout_with_worktree("put-back");
    let platform: PlainPlatform = PlainPlatform::new();
    let (migrated, migrated_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migrated, DONE, "{migrated_said}");
    let layout = layout_of(root.as_path()).expect("layout");
    let third_party: &[u8] = b"ANOTHER-COMMANDS-FILE\n";
    std::fs::write(layout.old_path.join("README.md"), b"a later write\n").expect("the later write");
    // The two seams speak for README.md alone, because the pass parks and puts back other entries too.
    let the_parked_name: std::ffi::OsString = crate::subvolume_sync_pass_state_and_checks::parked_name(Path::new("README.md"));
    let into_the_parked_entry: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    {
        let only_once: Arc<AtomicBool> = into_the_parked_entry.clone();
        let wanted: std::ffi::OsString = the_parked_name.clone();
        watch_before_the_judgement(Some(Box::new(move |parked: &Path| {
            if parked.file_name() != Some(wanted.as_os_str()) || only_once.swap(true, Ordering::SeqCst) {
                return;
            }
            std::fs::write(parked, b"THE-OTHER-PARTYS-WRITE\n").expect("a write into the parked entry");
        })));
    }
    let at_the_freed_name: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    {
        let only_once: Arc<AtomicBool> = at_the_freed_name.clone();
        watch_before_the_put_back(Some(Box::new(move |freed: &Path| {
            if freed.file_name() != Some(std::ffi::OsStr::new("README.md")) || only_once.swap(true, Ordering::SeqCst) {
                return;
            }
            std::fs::write(freed, third_party).expect("another command's file at the freed name");
        })));
    }
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    watch_before_the_judgement(None);
    watch_before_the_put_back(None);
    println!("removal code {code}; output: {said}");
    let anywhere: bool = parked_copies_in(root.as_path()).iter().any(|path| return holds(path.as_path(), third_party));
    assert!(
        holds(root.join("README.md").as_path(), third_party) || anywhere,
        "another command's file at that name is kept, or the pass kept it as a parked copy it names: {said}"
    );
    remove(parent.as_path());
}

/// The parked copies a directory and its subdirectories hold,
/// by the names the pass parks them under.
fn parked_copies_in(root: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(listing) = std::fs::read_dir(directory.as_path()) else { continue };
        for item in listing.flatten() {
            let path: PathBuf = item.path();
            let Ok(kind) = item.file_type() else { continue };
            if kind.is_dir() {
                if item.file_name() != ".git" {
                    pending.push(path);
                }
                continue;
            }
            if item.file_name().as_encoded_bytes().starts_with(crate::subvolume_sync_pass_state_and_checks::PARKED_PREFIX.as_bytes()) {
                found.push(path);
            }
        }
    }
    return found;
}

//endregion
