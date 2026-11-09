//! Controls for the review findings on the conversion:
//! the journal's files,
//! the identity of the stage,
//! what a late writer
//!  loses,
//! the pinned removal,
//! nested registrations in any layout,
//! refusals by kind,
//! and a
//!  failed index refresh.

use super::*;
use crate::fs_beneath::{identity_at, open_root};
use crate::worktree_convert_journal::is_recorded_object;
use crate::worktree_convert_remove::{Removal, remove_recorded_old_tree};
use crate::subvolume_test_support::{PlainPlatform, backdate, checkout_with_worktree};
use crate::test_support::{fixture, git, remove};
use crate::worktree_convert_journal::Phase as JournalPhase;
use crate::worktree_convert_remove::remove_recorded_old_tree_seamed;
use std::os::unix::fs::MetadataExt;

/// The options of a real run.
const RUN: ConvertOptions = ConvertOptions { dry_run: false, abandon: false, command_line_attributes: false };

/// A checkout with one settled detached linked worktree `wt`:
/// old files,
/// a refreshed index,
/// so it is quiet.
fn quiet_worktree(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let (parent, root) = checkout_with_worktree(name);
    let worktree: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical worktree");
    backdate(worktree.as_path());
    git(worktree.as_path(), &["update-index", "--refresh"]);
    return (parent, root, worktree);
}

/// Run one conversion and collect its lines.
fn run(platform: &dyn Platform, worktree: &Path, options: ConvertOptions, interrupt: &Interrupt) -> (Result<Verdict, MigrateError>, Vec<String>) {
    let mut lines: Vec<String> = Vec::new();
    let verdict = convert_worktree(platform, worktree, options, interrupt, &mut |text: &str| { lines.push(String::from(text)); });
    return (verdict, lines);
}

/// The administrative directory of a worktree.
fn admin_of(worktree: &Path) -> PathBuf {
    return inspect(worktree).expect("registered").admin;
}

/// The words of a failed run.
fn failure_of(verdict: Result<Verdict, MigrateError>) -> String {
    return verdict.expect_err("the run fails").to_string();
}

/// The reasons of a skipped run,
/// joined.
fn skipped(verdict: Result<Verdict, MigrateError>) -> String {
    return match verdict.expect("no failure") {
        Verdict::Skipped(reasons) => reasons.join("\n"),
        other => panic!("expected a skip, got {other:?}"),
    };
}

/// The journal's temporary file is never a name that exists:
/// a link at the old fixed name is not followed or truncated.
#[test]
fn the_journal_never_writes_through_a_file_that_is_in_the_way() {
    let (parent, _root, worktree) = quiet_worktree("safety-journal-tmp");
    let victim: PathBuf = parent.join("victim");
    std::fs::write(&victim, b"keep me").expect("victim");
    std::os::unix::fs::symlink(&victim, admin_of(worktree.as_path()).join("cli-git-convert.tmp")).expect("link at the old name");
    let platform: PlainPlatform = PlainPlatform::new();
    let (verdict, _lines) = run(&platform, worktree.as_path(), RUN, &Interrupt::never());
    assert_eq!(verdict.expect("converted"), Verdict::Converted);
    assert_eq!(std::fs::read(&victim).expect("victim"), b"keep me");
    remove(parent.as_path());
}

/// A journal is made only where there is none,
/// and replacing one is a separate act.
#[test]
fn a_fresh_journal_never_replaces_one_that_exists() {
    let (parent, _root, worktree) = quiet_worktree("safety-journal-new");
    let admin: PathBuf = admin_of(worktree.as_path());
    let made = Journal { phase: JournalPhase::Started, token: String::from("a"), path: b"/x".to_vec(), old_identity: ObjectId { device: 1, inode: 2, birth: Some((3, 4)) }, git_file: b"g".to_vec(), new_identity: None, synced_ms: None };
    made.create_new(admin.as_path()).expect("first journal");
    let other = Journal { token: String::from("b"), ..made.clone() };
    assert!(other.create_new(admin.as_path()).is_err(), "the second one must not replace the first");
    assert_eq!(Journal::load(admin.as_path()).expect("load"), Some(made));
    let names: Vec<String> = std::fs::read_dir(&admin).expect("listing").flatten().map(|entry: std::fs::DirEntry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    assert!(names.iter().all(|name: &String| { return !name.contains(".tmp."); }), "no temporary file is left: {names:?}");
    remove(parent.as_path());
}

/// A directory at the converting name that the journal has no identity for is foreign:
/// it is not synced into,
/// not exchanged and
///  not destroyed by the abandon,
/// whether someone made it or the run was killed between creating and recording it.
#[test]
fn a_foreign_stage_is_never_synced_into_exchanged_or_destroyed() {
    for point in ["journaled", "stage-unrecorded"] {
        let (parent, _root, worktree) = quiet_worktree(format!("safety-foreign-{point}").as_str());
        let platform: PlainPlatform = PlainPlatform::new();
        let converting: PathBuf = converting_path(worktree.as_path());
        let (killed, _lines) = run(&platform, worktree.as_path(), RUN, &Interrupt::at(point, 0));
        assert!(killed.is_err(), "{point}: the run was killed");
        if point == "journaled" {
            platform.create_subvolume(converting.as_path()).expect("a foreign subvolume");
        }
        std::fs::write(converting.join("theirs"), b"theirs").expect("their file");
        let reasons: String = skipped(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0);
        assert!(reasons.contains("exists") && reasons.contains("owns it"), "{point}: {reasons}");
        assert_eq!(std::fs::read(converting.join("theirs")).expect("theirs"), b"theirs", "{point}: nothing was synced over it");
        assert!(!platform.is_subvolume_root(worktree.as_path()).expect("examined"), "{point}: nothing was exchanged");
        let (abandoned, lines) = run(&platform, worktree.as_path(), ConvertOptions { dry_run: false, abandon: true, command_line_attributes: false }, &Interrupt::never());
        assert_eq!(abandoned.expect("abandoned"), Verdict::Abandoned, "{point}");
        assert!(converting.join("theirs").exists(), "{point}: the abandon left the foreign directory: {lines:?}");
        assert!(!admin_of(worktree.as_path()).join(JOURNAL_NAME).exists(), "{point}: the journal is gone");
        remove(parent.as_path());
    }
}

/// After the swap,
/// a different subvolume with the same `.git` bytes at the path is not the installed copy,
/// and the old tree stays.
#[test]
fn a_replaced_installed_subvolume_is_not_accepted() {
    let (parent, _root, worktree) = quiet_worktree("safety-replaced");
    let platform: PlainPlatform = PlainPlatform::new();
    let converting: PathBuf = converting_path(worktree.as_path());
    let git_file: Vec<u8> = std::fs::read(worktree.join(".git")).expect(".git");
    assert!(run(&platform, worktree.as_path(), RUN, &Interrupt::at("exchanged", 0)).0.is_err());
    std::fs::rename(&worktree, parent.join("installed-aside")).expect("move the installed copy away");
    platform.create_subvolume(worktree.as_path()).expect("another subvolume");
    std::fs::write(worktree.join(".git"), git_file).expect("same .git bytes");
    let message: String = failure_of(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0);
    assert!(message.contains("not the subvolume this conversion created"), "{message}");
    assert!(converting.join("README.md").exists(), "the old tree was not removed");
    remove(parent.as_path());
}

/// A file written after the copy was proved and before a kill is in the converted worktree,
/// because the copy is made again.
#[test]
fn a_file_written_after_a_synced_kill_is_not_lost() {
    let (parent, _root, worktree) = quiet_worktree("safety-synced");
    let platform: PlainPlatform = PlainPlatform::new();
    assert!(run(&platform, worktree.as_path(), RUN, &Interrupt::at("synced", 0)).0.is_err());
    std::fs::write(worktree.join("late.txt"), b"written after the proof\n").expect("late file");
    let (finished, lines) = run(&platform, worktree.as_path(), RUN, &Interrupt::never());
    assert_eq!(finished.expect("finished"), Verdict::Resumed, "{lines:#?}");
    assert_eq!(std::fs::read(worktree.join("late.txt")).expect("late file"), b"written after the proof\n");
    assert!(platform.is_subvolume_root(worktree.as_path()).expect("examined"));
    remove(parent.as_path());
}

/// A permission change made in the last window of a conversion,
/// after the proving pass and before the removal,
/// keeps the old tree and is named;
/// nothing is deleted and the installed copy is not altered.
#[test]
fn a_permission_change_after_the_proof_keeps_the_old_tree() {
    let (parent, _root, worktree) = quiet_worktree("safety-mode");
    let platform: PlainPlatform = PlainPlatform::new();
    let converting: PathBuf = converting_path(worktree.as_path());
    assert!(run(&platform, worktree.as_path(), RUN, &Interrupt::at("exchanged", 0)).0.is_err());
    std::fs::set_permissions(converting.join("script.sh"), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("a narrowed copy in the old tree");
    let message: String = failure_of(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0);
    assert!(message.contains("script.sh") && message.contains("permission bits") && message.contains("kept"), "{message}");
    assert!(converting.join("script.sh").exists(), "the old tree is kept whole");
    assert_eq!(std::fs::metadata(worktree.join("script.sh")).expect("installed").mode() & 0o777, 0o755, "the installed copy is unchanged");
    remove(parent.as_path());
}

/// What:
///  A same-size edit made in the last window of a conversion,
/// with its modification time put back,
/// keeps the old tree and is named.
/// Why:
///  The comparison could only see the type,
/// size,
/// time,
/// mode,
/// owner and attributes;
/// an edit that keeps the length and puts the time back leaves all of them
///  equal,
/// so the removal deleted the bytes the person wrote.
/// The file's change time cannot be put back,
/// and
///  the contents are compared for a file that changed after the proof.
#[test]
fn a_same_size_edit_with_the_time_put_back_after_the_proof_keeps_the_old_tree() {
    let (parent, _root, worktree) = quiet_worktree("safety-same-size");
    let platform: PlainPlatform = PlainPlatform::new();
    let converting: PathBuf = converting_path(worktree.as_path());
    assert!(run(&platform, worktree.as_path(), RUN, &Interrupt::at("exchanged", 0)).0.is_err());
    let edited: PathBuf = converting.join("script.sh");
    let before: std::fs::Metadata = std::fs::metadata(&edited).expect("the old copy");
    let mut bytes: Vec<u8> = std::fs::read(&edited).expect("the old copy's bytes");
    bytes[0] = b'A';
    std::fs::write(&edited, &bytes).expect("an edit of the same length");
    std::fs::OpenOptions::new().write(true).open(&edited).expect("open the edited file").set_times(std::fs::FileTimes::new().set_modified(before.modified().expect("modified"))).expect("put the time back");
    assert_eq!(std::fs::metadata(&edited).expect("the edited copy").len(), before.len(), "the length is unchanged");
    let message: String = failure_of(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0);
    assert!(message.contains("script.sh") && message.contains("contents") && message.contains("kept"), "{message}");
    assert!(edited.exists(), "the old tree is kept whole");
    remove(parent.as_path());
}

/// What:
///  A same-size edit made during the proving pass,
/// after that file was examined,
/// with its modification time put
///  back,
/// keeps the old tree and is named.
/// Why:
///  The instant the removal compares change times against was taken when the pass ended,
/// so an edit
///  made while the pass ran had a change time before it and its contents were never compared,
/// and the
///  removal deleted the only copy of the edited bytes.
#[test]
fn a_same_size_edit_during_the_proving_pass_keeps_the_old_tree() {
    let (parent, _root, worktree) = quiet_worktree("safety-during-proof");
    let platform: PlainPlatform = PlainPlatform::new();
    let target: PathBuf = worktree.join("script.sh");
    let mut visits: u32 = 0;
    crate::subvolume_sync::watch_entries(Some(Box::new(move |path: &Path| {
        if path != target {
            return;
        }
        visits += 1;
        // The first pass copies the file; the proving pass examines it again, and this runs before that examination.
        if visits != 2 {
            return;
        }
        let before: std::fs::Metadata = std::fs::metadata(&target).expect("the file");
        let mut bytes: Vec<u8> = std::fs::read(&target).expect("its bytes");
        bytes[0] = b'A';
        std::fs::write(&target, &bytes).expect("an edit of the same length");
        std::fs::OpenOptions::new().write(true).open(&target).expect("open the edited file").set_times(std::fs::FileTimes::new().set_modified(before.modified().expect("modified"))).expect("put the time back");
        // The pass goes on for longer than the clock reading it records is worth, so the edit is unmistakably before the
        // instant the removal compares against; a real pass over a whole tree takes far longer than this.
        std::thread::sleep(std::time::Duration::from_millis(5));
    })));
    let outcome = run(&platform, worktree.as_path(), RUN, &Interrupt::never());
    crate::subvolume_sync::watch_entries(None);
    let message: String = failure_of(outcome.0);
    assert!(message.contains("script.sh") && message.contains("contents") && message.contains("kept"), "{message}");
    remove(parent.as_path());
}

/// What:
///  A same-size edit made to the *installed* copy after the proof,
/// with its modification time put back,
/// keeps the old tree and is named.
/// Why:
///  Only the old file's change time decided whether the contents were read,
/// so an edit to the installed file
///  left both sides equal by every compared field,
/// and the old tree,
/// the only copy of the earlier bytes,
///  was deleted.
#[test]
fn a_same_size_edit_to_the_installed_copy_keeps_the_old_tree() {
    let (parent, _root, worktree) = quiet_worktree("safety-installed-edit");
    let platform: PlainPlatform = PlainPlatform::new();
    assert!(run(&platform, worktree.as_path(), RUN, &Interrupt::at("exchanged", 0)).0.is_err());
    let installed: PathBuf = worktree.join("script.sh");
    let before: std::fs::Metadata = std::fs::metadata(&installed).expect("the installed copy");
    let mut bytes: Vec<u8> = std::fs::read(&installed).expect("its bytes");
    bytes[0] = b'B';
    std::fs::write(&installed, &bytes).expect("an edit of the same length");
    std::fs::OpenOptions::new().write(true).open(&installed).expect("open the edited file").set_times(std::fs::FileTimes::new().set_modified(before.modified().expect("modified"))).expect("put the time back");
    let message: String = failure_of(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0);
    assert!(message.contains("script.sh") && message.contains("contents") && message.contains("kept"), "{message}");
    remove(parent.as_path());
}

/// A difference between the old tree and the installed one keeps the old tree and is reported;
/// nothing is ever copied from the
///  old tree,
/// so an edit made to the installed worktree is never overwritten.
#[test]
fn a_difference_keeps_the_old_tree_and_nothing_is_copied() {
    let (parent, _root, worktree) = quiet_worktree("safety-late");
    let platform: PlainPlatform = PlainPlatform::new();
    let converting: PathBuf = converting_path(worktree.as_path());
    assert!(run(&platform, worktree.as_path(), RUN, &Interrupt::at("exchanged", 0)).0.is_err());
    std::fs::write(worktree.join("README.md"), b"an edit made to the installed worktree\n").expect("edit of the installed copy");
    std::fs::write(converting.join("README.md"), b"a later write to the old tree\n").expect("write to the old tree");
    std::fs::write(converting.join("late.txt"), b"only in the old tree\n").expect("late file in the old tree");
    let message: String = failure_of(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0);
    assert!(message.contains("differs") && message.contains("kept"), "{message}");
    assert_eq!(std::fs::read(worktree.join("README.md")).expect("installed"), b"an edit made to the installed worktree\n", "the installed edit is not overwritten");
    assert!(!worktree.join("late.txt").exists(), "nothing was copied from the old tree");
    assert!(converting.join("late.txt").exists(), "the old tree is kept whole");
    // The person settles it by hand; the next run then removes the old tree.
    std::fs::remove_file(converting.join("late.txt")).expect("settled");
    std::fs::copy(worktree.join("README.md"), converting.join("README.md")).expect("settled");
    let mtime = std::fs::metadata(worktree.join("README.md")).expect("metadata").modified().expect("mtime");
    std::fs::OpenOptions::new().write(true).open(converting.join("README.md")).expect("open").set_modified(mtime).expect("same time");
    let (finished, lines) = run(&platform, worktree.as_path(), RUN, &Interrupt::never());
    assert_eq!(finished.expect("finished"), Verdict::Resumed, "{lines:#?}");
    assert!(std::fs::symlink_metadata(&converting).is_err(), "now the old tree is gone");
    remove(parent.as_path());
}

/// A process that holds the old tree open keeps it,
/// and the run says which.
#[test]
fn a_process_holding_the_old_tree_keeps_it() {
    let (parent, _root, worktree) = quiet_worktree("safety-holder");
    let platform: PlainPlatform = PlainPlatform::new();
    let converting: PathBuf = converting_path(worktree.as_path());
    assert!(run(&platform, worktree.as_path(), RUN, &Interrupt::at("refreshed", 0)).0.is_err());
    let mut child: std::process::Child = crate::test_support::spawn_owned(std::process::Command::new("sleep").arg("30").current_dir(&converting)).expect("sleep");
    let message: String = failure_of(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0);
    assert!(message.contains("kept because process"), "{message}");
    assert!(converting.join("README.md").exists());
    child.kill().expect("kill");
    child.wait().expect("wait");
    assert_eq!(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0.expect("finished"), Verdict::Resumed);
    remove(parent.as_path());
}

/// The old tree is removed beneath the directory that was pinned:
/// a directory put at the name after the check survives.
#[test]
fn the_removal_follows_the_pinned_directory_not_the_name() {
    let (parent, _root, worktree) = quiet_worktree("safety-pinned");
    let platform: PlainPlatform = PlainPlatform::new();
    let old: PathBuf = parent.join("old");
    std::fs::create_dir(&old).expect("old");
    let installed: PathBuf = parent.join("installed");
    std::fs::create_dir(&installed).expect("installed");
    let recorded: ObjectId = identity_at(&open_root(parent.as_path()).expect("held parent"), std::ffi::OsStr::new("old")).expect("identity");
    let held = open_root(parent.as_path()).expect("held parent");
    let moved: PathBuf = parent.join("moved");
    let outcome = remove_recorded_old_tree_seamed(&platform, &held, std::ffi::OsStr::new("old"), recorded, installed.as_path(), None, &mut || {
        std::fs::rename(&old, &moved).expect("move the old tree away");
        std::fs::create_dir(&old).expect("another directory at the name");
        std::fs::write(old.join("theirs"), b"theirs").expect("theirs");
    })
    .expect("no failure");
    assert!(matches!(outcome, Removal::Kept(_)), "{outcome:?}");
    assert_eq!(std::fs::read(old.join("theirs")).expect("theirs"), b"theirs", "the directory that took the name is untouched");
    drop(worktree);
    remove(parent.as_path());
}

/// A worktree registered inside another is found whatever the common directory is called:
/// here a bare repository.
#[test]
fn a_nested_registration_is_found_in_a_bare_layout() {
    let (parent, root) = checkout_with_worktree("safety-bare");
    let bare: PathBuf = parent.join("r.git");
    git(parent.as_path(), &["clone", "--quiet", "--bare", root.to_str().expect("UTF-8"), bare.to_str().expect("UTF-8")]);
    git(bare.as_path(), &["worktree", "add", "--quiet", "--detach", "../outer"]);
    let outer: PathBuf = std::fs::canonicalize(parent.join("outer")).expect("canonical outer");
    backdate(outer.as_path());
    git(outer.as_path(), &["update-index", "--refresh"]);
    git(bare.as_path(), &["worktree", "add", "--quiet", "--detach", "../outer/inner"]);
    let platform: PlainPlatform = PlainPlatform::new();
    let reasons: String = skipped(run(&platform, outer.as_path(), ConvertOptions { dry_run: true, abandon: false, command_line_attributes: false }, &Interrupt::never()).0);
    assert!(reasons.contains("registered inside it") && reasons.contains("inner"), "{reasons}");
    remove(parent.as_path());
}

/// A refusal is exempt by what it is and who owns it,
/// not by its words:
/// a nested worktree whose name contains the converting
///  suffix still stops a resumed conversion.
#[test]
fn a_nested_worktree_named_like_the_stage_still_stops_a_resume() {
    let (parent, root, worktree) = quiet_worktree("safety-suffix");
    let platform: PlainPlatform = PlainPlatform::new();
    assert!(run(&platform, worktree.as_path(), RUN, &Interrupt::at("journaled", 0)).0.is_err());
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", worktree.join(format!("inner{CONVERTING_SUFFIX}-backup")).to_str().expect("UTF-8")]);
    let reasons: String = skipped(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0);
    assert!(reasons.contains("registered inside it") && reasons.contains(CONVERTING_SUFFIX), "{reasons}");
    remove(parent.as_path());
}

/// An index that cannot be refreshed stops the conversion with the old tree kept and the journal where it was;
/// the next run
///  finishes it.
#[test]
fn a_failed_index_refresh_keeps_the_old_tree() {
    let (parent, _root, worktree) = quiet_worktree("safety-index");
    let platform: PlainPlatform = PlainPlatform::new();
    let converting: PathBuf = converting_path(worktree.as_path());
    let index: PathBuf = admin_of(worktree.as_path()).join("index");
    assert!(run(&platform, worktree.as_path(), RUN, &Interrupt::at("exchanged", 0)).0.is_err());
    std::fs::set_permissions(&index, std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("make the index unreadable");
    let message: String = failure_of(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0);
    assert!(message.contains("not refreshed") && message.contains("kept"), "{message}");
    assert!(converting.join("README.md").exists(), "the old tree is kept");
    assert!(admin_of(worktree.as_path()).join(JOURNAL_NAME).exists(), "the journal is kept");
    std::fs::set_permissions(&index, std::os::unix::fs::PermissionsExt::from_mode(0o644)).expect("restore");
    assert_eq!(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0.expect("finished"), Verdict::Resumed);
    assert!(std::fs::symlink_metadata(&converting).is_err());
    remove(parent.as_path());
}

/// What:
///  Run `work` on a thread and give its answer,
///  failing when the run it waits in shows no sign of work instead.
/// Why:
///   The event is the answer itself:
/// a run that blocked on a named pipe's open never gives one,
/// and a run that
///  is merely slow on this disk gives it late.
/// A stated patience confused the two:
/// measured on 2026-11-09 with the host at a load average of 70,
/// the control below failed alone in 24.7 s at
///  "a run with a pipe for a journal did not finish within 20s" while the run was working,
/// and passed alone on quieter minutes.
/// The window is the watchdog's own (`test_support::STILL_WORKING_WINDOW`), so a blocked open ends the wait and a slow
///  disk does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await answerWhileTheRunWorks(what, work); // rejects when the run shows nothing for the window
/// ```
fn finishes<T: Send + 'static>(what: &str, work: impl FnOnce() -> T + Send + 'static) -> T {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _sent = sender.send(work());
    });
    let mut judge: crate::run_progress::Judge = crate::run_progress::Judge::new(crate::test_support::STILL_WORKING_WINDOW);
    let began: std::time::Instant = std::time::Instant::now();
    // This thread's own looks are not the run's work, and neither are another reader's.
    let _reading: Option<crate::run_progress::Reading> = crate::run_progress::Reading::begin();
    loop {
        match receiver.recv_timeout(crate::test_support::WAIT_LOOK_EVERY) {
            Ok(answer) => return answer,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if let Some(ending) = judge.observe(began.elapsed(), crate::run_progress::look_now()) {
                    panic!("{what} did not finish: the run {ending}");
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => panic!("{what}: the thread that runs it ended without an answer"),
        }
    }
}

/// The mode of a file only its owner may use.
const OWNER_ONLY: libc::mode_t = 0o600;

/// A directory that can be searched but not listed.
const SEARCH_WITHOUT_LISTING: u32 = 0o311;

/// An ordinary directory mode.
const LISTABLE: u32 = 0o755;

/// A named pipe at `path`,
/// which blocks whoever opens it for reading until a writer comes.
fn make_pipe(path: &Path) {
    let name = std::ffi::CString::new(std::os::unix::ffi::OsStrExt::as_bytes(path.as_os_str())).expect("path text");
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let result: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), OWNER_ONLY) };
    assert_eq!(result, 0, "mkfifo {}", path.display());
}

/// A registered-worktree listing that cannot be read is a refusal,
/// not the absence of nested worktrees.
#[test]
fn an_unreadable_list_of_registrations_is_a_refusal() {
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    let (parent, _root, worktree) = quiet_worktree("safety-registrations");
    let platform: PlainPlatform = PlainPlatform::new();
    let registrations: PathBuf = inspect(worktree.as_path()).expect("registered").common.join("worktrees");
    std::fs::set_permissions(&registrations, std::os::unix::fs::PermissionsExt::from_mode(SEARCH_WITHOUT_LISTING)).expect("search without listing");
    let reasons: String = skipped(run(&platform, worktree.as_path(), ConvertOptions { dry_run: true, abandon: false, command_line_attributes: false }, &Interrupt::never()).0);
    std::fs::set_permissions(&registrations, std::os::unix::fs::PermissionsExt::from_mode(LISTABLE)).expect("restore");
    assert!(reasons.contains("cannot be listed"), "{reasons}");
    remove(parent.as_path());
}

/// A worktree moved while a conversion was unfinished is refused loudly:
/// the journal names the path it began at.
#[test]
fn a_worktree_moved_during_an_unfinished_conversion_is_refused() {
    let (parent, root, worktree) = quiet_worktree("safety-moved");
    let platform: PlainPlatform = PlainPlatform::new();
    assert!(run(&platform, worktree.as_path(), RUN, &Interrupt::at("synced", 0)).0.is_err());
    let moved: PathBuf = worktree.with_file_name("wt-moved");
    git(root.as_path(), &["worktree", "move", worktree.to_str().expect("UTF-8"), moved.to_str().expect("UTF-8")]);
    for dry_run in [false, true] {
        let message: String = failure_of(run(&platform, moved.as_path(), ConvertOptions { dry_run, abandon: false, command_line_attributes: false }, &Interrupt::never()).0);
        assert!(message.contains("moved") && message.contains(worktree.to_str().expect("UTF-8")), "dry run {dry_run}: {message}");
    }
    assert!(!platform.is_subvolume_root(moved.as_path()).expect("examined"), "nothing was exchanged");
    remove(parent.as_path());
}

/// A journal or a `.git` that is a named pipe ends the run with a reason instead of blocking while the lock is held.
#[test]
fn a_named_pipe_in_the_place_of_a_file_does_not_block() {
    let (parent, _root, worktree) = quiet_worktree("safety-pipe-journal");
    let admin: PathBuf = admin_of(worktree.as_path());
    make_pipe(admin.join(JOURNAL_NAME).as_path());
    let at: PathBuf = worktree.clone();
    let outcome = finishes("a run with a pipe for a journal", move || { return run(&PlainPlatform::new(), at.as_path(), RUN, &Interrupt::never()).0; });
    assert!(outcome.is_err(), "{outcome:?}");
    std::fs::remove_file(admin.join(JOURNAL_NAME)).expect("remove the pipe");
    let (other, _other_root, second) = quiet_worktree("safety-pipe-git");
    std::fs::remove_file(second.join(".git")).expect("remove the pointer");
    make_pipe(second.join(".git").as_path());
    let there: PathBuf = second.clone();
    let skipped_run = finishes("a run with a pipe for .git", move || { return run(&PlainPlatform::new(), there.as_path(), RUN, &Interrupt::never()).0; });
    assert!(matches!(skipped_run, Ok(Verdict::Skipped(_))), "{skipped_run:?}");
    remove(parent.as_path());
    remove(other.as_path());
}

/// A dry run after a kill between the exchange and its journal line sees the exchange,
/// like a real run does.
#[test]
fn a_dry_run_sees_an_exchange_the_journal_has_not_recorded() {
    let (parent, _root, worktree) = quiet_worktree("safety-dry-exchanged");
    let platform: PlainPlatform = PlainPlatform::new();
    assert!(run(&platform, worktree.as_path(), RUN, &Interrupt::at("exchanged", 0)).0.is_err());
    let (verdict, lines) = run(&platform, worktree.as_path(), ConvertOptions { dry_run: true, abandon: false, command_line_attributes: false }, &Interrupt::never());
    assert_eq!(verdict.expect("no failure"), Verdict::Resumed, "{lines:#?}");
    let journal: Journal = Journal::load(admin_of(worktree.as_path()).as_path()).expect("load").expect("journal");
    assert_eq!(journal.phase, JournalPhase::Synced, "a dry run writes nothing");
    remove(parent.as_path());
}

/// The old root is recorded with its birth time:
/// the journal keeps it,
/// one written before it was recorded still reads,
/// and a
///  directory with the same inode but another birth time is not the old root.
#[test]
fn the_old_root_is_recorded_with_its_birth_time_and_compared_with_it() {
    let (parent, _root, worktree) = quiet_worktree("safety-birth");
    let admin: PathBuf = admin_of(worktree.as_path());
    let identity = ObjectId { device: 1, inode: 2, birth: Some((3, 4)) };
    let made = Journal { phase: JournalPhase::Started, token: String::from("a"), path: b"/x".to_vec(), old_identity: identity, git_file: b"g".to_vec(), new_identity: None, synced_ms: None };
    made.save(admin.as_path()).expect("saved");
    assert_eq!(Journal::load(admin.as_path()).expect("load"), Some(made));
    let legacy: String = format!("cli-git-convert v1\nphase=started\ntoken=a\npath=2f78\nold_device=5\nold_inode=6\ngit_file=67\nnew_identity=-\n");
    std::fs::write(admin.join(JOURNAL_NAME), legacy).expect("legacy journal");
    let read = Journal::load(admin.as_path()).expect("load").expect("journal");
    assert_eq!(read.old_identity, ObjectId { device: 5, inode: 6, birth: None });
    let found = ObjectId { device: 5, inode: 6, birth: Some((7, 8)) };
    assert!(is_recorded_object(&read.old_identity, &found), "a journal without a birth time compares device and inode");
    assert!(is_recorded_object(&found, &found));
    assert!(!is_recorded_object(&found, &ObjectId { birth: Some((9, 8)), ..found }), "another birth time is another directory");
    assert!(!is_recorded_object(&found, &ObjectId { birth: None, ..found }), "a missing birth time does not match a recorded one");
    remove(parent.as_path());
}

/// The removal checks the birth time on the descriptor it pins:
/// the same inode with another birth time is left alone.
#[test]
fn a_directory_with_another_birth_time_is_not_removed() {
    let (parent, _root, worktree) = quiet_worktree("safety-birth-removal");
    let platform: PlainPlatform = PlainPlatform::new();
    let old: PathBuf = parent.join("old");
    std::fs::create_dir(&old).expect("old");
    let held = open_root(parent.as_path()).expect("held parent");
    let real: ObjectId = identity_at(&held, std::ffi::OsStr::new("old")).expect("identity");
    let other_birth: ObjectId = ObjectId { birth: Some((0, 0)), ..real };
    assert_ne!(Some((0, 0)), real.birth, "the control needs a real birth time to differ from");
    let outcome = remove_recorded_old_tree(&platform, &held, std::ffi::OsStr::new("old"), other_birth, worktree.as_path(), None).expect("no failure");
    assert!(matches!(outcome, Removal::Kept(_)), "{outcome:?}");
    assert!(old.exists(), "nothing was removed");
    remove(parent.as_path());
}

/// The first process number no process can have:
/// the largest Linux hands out is bounded by `PID_MAX_LIMIT`,
/// 4 * 1024 * 1024,
/// so nothing can be running under a number above it.
const A_PID_NO_PROCESS_HAS: u32 = 4 * 1024 * 1024 + 1;

/// An administrative directory with the given files,
/// each holding a byte.
fn administrative_directory(name: &str, files: &[&str]) -> PathBuf {
    let root: PathBuf = fixture(name);
    let admin: PathBuf = root.join("admin");
    std::fs::create_dir_all(admin.as_path()).expect("administrative directory");
    for file in files {
        std::fs::write(admin.join(file), b"x").expect("file");
    }
    return admin;
}

/// The sweep removes a temporary file only when the process that made it is gone:
/// the journal,
/// the claim of the conversion,
/// the temporary file of a running process and every name that is no
/// temporary file all stay.
#[test]
fn the_sweep_keeps_every_file_whose_maker_may_still_run() {
    let live: String = format!("cli-git-convert.tmp.{}.{}", std::process::id(), 1);
    let kept: [String; 6] = [
        String::from("cli-git-convert"),
        format!("{}.1", crate::worktree_convert_journal::CLAIM_PREFIX),
        live.clone(),
        String::from("cli-git-convert.tmp.no-number.7"),
        format!("cli-git-convert.tmp.{A_PID_NO_PROCESS_HAS}.no-clock"),
        String::from("cli-git-convert.tmp."),
    ];
    let names: Vec<&str> = kept.iter().map(String::as_str).collect();
    let admin: PathBuf = administrative_directory("convert-journal-sweep-kept", names.as_slice());
    assert_eq!(crate::worktree_convert_journal::sweep_temporary_files(admin.as_path()), 0, "a live maker keeps its file");
    for name in &kept {
        assert!(admin.join(name).is_file(), "{name} was removed");
    }
    let root: PathBuf = admin.parent().expect("root").to_path_buf();
    remove(root.as_path());
}

/// A temporary file whose process number names no process is the leftover of a conversion that was killed,
/// so the sweep removes it and nothing else.
#[test]
fn the_sweep_removes_a_temporary_file_of_a_process_that_is_gone() {
    let dead: String = format!("cli-git-convert.tmp.{A_PID_NO_PROCESS_HAS}.99");
    let kept: String = format!("cli-git-convert.tmp.{}.99", std::process::id());
    let admin: PathBuf = administrative_directory("convert-journal-sweep-gone", &[dead.as_str(), kept.as_str(), "cli-git-convert"]);
    assert_eq!(crate::worktree_convert_journal::sweep_temporary_files(admin.as_path()), 1);
    assert!(!admin.join(dead.as_str()).exists(), "the leftover of a killed conversion is gone");
    assert!(admin.join(kept.as_str()).is_file(), "the file of a running process stays");
    assert!(admin.join("cli-git-convert").is_file(), "the journal stays");
    let root: PathBuf = admin.parent().expect("root").to_path_buf();
    remove(root.as_path());
}

/// Every journal write sweeps the left-overs of its own administrative directory first,
/// so the first write of a run after a killed one clears what that one left,
/// while a file another running process holds is never taken.
#[test]
fn a_journal_write_sweeps_the_temporary_file_of_a_killed_conversion() {
    let dead: String = format!("cli-git-convert.tmp.{A_PID_NO_PROCESS_HAS}.12345");
    let live: String = format!("cli-git-convert.tmp.{}.12345", std::process::id());
    let admin: PathBuf = administrative_directory("convert-journal-sweep-write", &[dead.as_str(), live.as_str()]);
    let journal = Journal {
        phase: JournalPhase::Started,
        token: String::from("token"),
        path: b"/worktree".to_vec(),
        old_identity: ObjectId { device: 1, inode: 2, birth: None },
        git_file: b"gitdir: /common/worktrees/wt\n".to_vec(),
        new_identity: None,
        synced_ms: None,
    };
    journal.save(admin.as_path()).expect("the journal is written");
    assert!(admin.join("cli-git-convert").is_file(), "the write published the journal");
    assert!(!admin.join(dead.as_str()).exists(), "the write swept what the killed conversion left");
    assert!(admin.join(live.as_str()).is_file(), "a running process keeps its temporary file");
    let root: PathBuf = admin.parent().expect("root").to_path_buf();
    remove(root.as_path());
}
