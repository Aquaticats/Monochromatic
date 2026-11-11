//! Controls for putting a person's work back across a merge:
//! the clear that hands Git its paths,
//! the restore of a merge that did not land,
//! and the carry that merges their two layers with the merge,
//! each in a disposable repository.

use super::*;
use crate::merge_isolation_carry::{capture_work, keep_capture};
use crate::merge_isolation_carry_parts::Capture;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::{Path, PathBuf};

/// The identifier a merge of a fixture names itself by,
/// which every private name a control's clear,
/// carry or restore makes carries.
const A_MERGES_ID: &str = "19191919-0000-4000-8000-000000000019";

/// Lines both sides start from.
const LINES: &str = "one\ntwo\nthree\nfour\n";

/// The line the person adds.
const PERSON_LINE: &str = "the person's line";

/// A repository whose `main` and `feature` diverge on the first and third lines of `a.txt`,
/// with a base commit before both.
fn diverged(name: &str) -> (PathBuf, PathBuf, String, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), LINES).expect("a");
    std::fs::write(repo.join("keep.txt"), b"keep\n").expect("keep");
    git(repo.as_path(), &["add", "--", "a.txt", "keep.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("a.txt"), LINES.replace("three", "three feature")).expect("a");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=feature"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("a.txt"), LINES.replace("one", "one main")).expect("a");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
    let old: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let tree: String = git_text(repo.as_path(), &["merge-tree", "--write-tree", "main", "feature"]).split('\n').next().unwrap_or_default().trim().to_owned();
    let new: String = git_text(repo.as_path(), &["commit-tree", tree.as_str(), "-p", "main", "-p", "feature", "-m", "merge feature"]).trim().to_owned();
    return (root, repo, old, new);
}

/// The merge's paths,
/// as the clear and the carry read them.
fn changes(repo: &Path, old: &str, new: &str) -> Vec<Change> {
    return changes_between(&git_context(), repo, old, new).expect("the merge's changes");
}

/// The index entry of one path as `mode oid`.
fn entry(repo: &Path, path: &str) -> String {
    let text: String = git_text(repo, &["ls-files", "--stage", "--", path]);
    let parts: Vec<&str> = text.split_whitespace().collect();
    return format!("{} {}", parts.first().copied().unwrap_or("-"), parts.get(1).copied().unwrap_or("-"));
}

/// What:
///  The clear puts a path back to the branch,
/// and the restore puts the person's own bytes and entry back.
/// Why:
///  A merge killed after the clear left the branch's content where the person's work was,
/// and the capture is the
///  only copy of it.
#[test]
fn a_clear_hands_the_path_to_git_and_a_restore_gives_it_back() {
    let (root, repo, old, new) = diverged("carry-clear-restore");
    let context = git_context();
    std::fs::write(repo.join("a.txt"), format!("{}{PERSON_LINE}\n", LINES.replace("one", "one main"))).expect("the person's edit");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    std::fs::write(repo.join("a.txt"), format!("{}{PERSON_LINE}\n", LINES.replace("one", "one main"))).expect("the person's edit again");
    let capture: Capture = keep_capture(&context, repo.as_path(), &capture_work(&context, repo.as_path(), &[b"a.txt".to_vec()]).expect("the capture")).expect("kept");
    let mut out: Vec<u8> = Vec::new();
    clear(&context, repo.as_path(), repo.join(".git").as_path(), changes(repo.as_path(), old.as_str(), new.as_str()).as_slice(), &capture, A_MERGES_ID, LockWait::BeforeTheApply, &mut out).expect("the clear");
    assert_eq!(std::fs::read_to_string(repo.join("a.txt")).expect("a.txt"), LINES.replace("one", "one main"), "the path is the branch's");
    assert_eq!(entry(repo.as_path(), "a.txt"), format!("100644 {}", git_text(repo.as_path(), &["rev-parse", "HEAD:a.txt"]).trim()), "the index is the branch's");
    restore(&context, repo.as_path(), repo.join(".git").as_path(), changes(repo.as_path(), old.as_str(), new.as_str()).as_slice(), &capture, A_MERGES_ID, LockWait::WhileFinishing, &mut out).expect("the restore");
    assert_eq!(
        std::fs::read_to_string(repo.join("a.txt")).expect("a.txt"),
        format!("{}{PERSON_LINE}\n", LINES.replace("one", "one main")),
        "the person's bytes are back"
    );
    assert_ne!(entry(repo.as_path(), "a.txt"), format!("100644 {}", git_text(repo.as_path(), &["rev-parse", "HEAD:a.txt"]).trim()), "the staged entry is back");
    remove(root.as_path());
}

/// What:
///  A merge's own change lands and the person's edit on another line is carried,
/// the index keeping the merge's entry for an unstaged edit.
/// Why:
///   That is the answer "Carry edits forward":
/// the merge lands and the edit is still there,
/// unsaved as it was.
#[test]
fn a_carry_merges_the_edit_with_the_merge() {
    let (root, repo, old, new) = diverged("carry-clean-merge");
    let context = git_context();
    let edited: String = format!("{}{PERSON_LINE}\n", LINES.replace("one", "one main").replace("three", "three feature"));
    let person: String = format!("{}{PERSON_LINE}\n", LINES.replace("one", "one main"));
    std::fs::write(repo.join("a.txt"), person.as_str()).expect("the person's edit");
    let capture: Capture = keep_capture(&context, repo.as_path(), &capture_work(&context, repo.as_path(), &[b"a.txt".to_vec()]).expect("the capture")).expect("kept");
    let mut out: Vec<u8> = Vec::new();
    let merge: Vec<Change> = changes(repo.as_path(), old.as_str(), new.as_str());
    clear(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, A_MERGES_ID, LockWait::BeforeTheApply, &mut out).expect("the clear");
    // Git's own two-way update of the branch, which is what the merge does with the paths free.
    git(repo.as_path(), &["read-tree", "-u", "-m", old.as_str(), new.as_str()]);
    let report: Report = carry(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, &[], A_MERGES_ID, LockWait::WhileFinishing, &mut out).expect("the carry");
    assert_eq!(report.conflicts.len(), 0, "nothing conflicts");
    assert_eq!(std::fs::read_to_string(repo.join("a.txt")).expect("a.txt"), edited, "the merged line and the person's line");
    assert_eq!(entry(repo.as_path(), "a.txt"), format!("100644 {}", git_text(repo.as_path(), &["rev-parse", &format!("{new}:a.txt")]).trim()), "the index holds the merge's entry");
    assert_eq!(git_text(repo.as_path(), &["diff", "--name-only"]).trim(), "a.txt", "the edit is unstaged");
    remove(root.as_path());
}

/// What:
///  A staged edit stays staged over the merged file,
/// and an edit the merge changes on the same line keeps both sides in the file.
/// Why:
///  The two promises of "Carry edits forward",
/// and of this package's conflict treatment.
#[test]
fn a_staged_edit_stays_staged_and_a_conflict_keeps_both_sides() {
    let (root, repo, old, new) = diverged("carry-staged-and-conflict");
    let context = git_context();
    let staged: String = format!("{}{PERSON_LINE}\n", LINES.replace("one", "one main"));
    std::fs::write(repo.join("a.txt"), staged.as_str()).expect("the person's edit");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    let capture: Capture = keep_capture(&context, repo.as_path(), &capture_work(&context, repo.as_path(), &[b"a.txt".to_vec()]).expect("the capture")).expect("kept");
    let merge: Vec<Change> = changes(repo.as_path(), old.as_str(), new.as_str());
    let mut out: Vec<u8> = Vec::new();
    clear(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, A_MERGES_ID, LockWait::BeforeTheApply, &mut out).expect("the clear");
    git(repo.as_path(), &["read-tree", "-u", "-m", old.as_str(), new.as_str()]);
    let report: Report = carry(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, &[], A_MERGES_ID, LockWait::WhileFinishing, &mut out).expect("the carry");
    assert_eq!(report.conflicts.len(), 0);
    let status: String = git_text(repo.as_path(), &["status", "--porcelain=v1"]);
    assert_eq!(status, "M  a.txt", "the edit is staged and nothing is unstaged over it");
    // Against the merge itself, the staged difference is the person's own line and nothing the merge brought.
    let cached: String = git_text(repo.as_path(), &["diff", "--cached", "--unified=0", new.as_str()]);
    assert!(cached.contains(&format!("+{PERSON_LINE}")), "{cached}");
    assert!(!cached.contains("one main") && !cached.contains("three feature"), "the merge's own lines are not staged back: {cached}");

    // The same path, with the person's edit on the very line the merge changes.
    std::fs::write(repo.join("a.txt"), LINES.replace("one", "one main").replace("three", "three person")).expect("the conflicting edit");
    let conflicting: Capture = keep_capture(&context, repo.as_path(), &capture_work(&context, repo.as_path(), &[b"a.txt".to_vec()]).expect("the capture")).expect("kept");
    clear(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &conflicting, A_MERGES_ID, LockWait::BeforeTheApply, &mut out).expect("the clear");
    git(repo.as_path(), &["read-tree", "-u", "-m", old.as_str(), new.as_str()]);
    let conflicted: Report = carry(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &conflicting, &[], A_MERGES_ID, LockWait::WhileFinishing, &mut out).expect("the carry");
    assert_eq!(conflicted.conflicts, vec![b"a.txt".to_vec()]);
    let text: String = std::fs::read_to_string(repo.join("a.txt")).expect("a.txt");
    assert!(text.contains("three person") && text.contains("three feature"), "both sides are in the file: {text}");
    assert!(text.contains("<<<<<<<") && text.contains(">>>>>>>"), "the file holds markers: {text}");
    remove(root.as_path());
}

/// What:
///  A path the person changed after the capture is left alone and named.
/// Why:
///   The bytes there now are newer than the ones the merge kept,
/// and writing over them would discard an edit nothing copied.
#[test]
fn a_path_changed_after_the_capture_is_left_alone() {
    let (root, repo, old, new) = diverged("carry-changed-after");
    let context = git_context();
    std::fs::write(repo.join("a.txt"), format!("{}{PERSON_LINE}\n", LINES.replace("one", "one main"))).expect("the person's edit");
    let capture: Capture = keep_capture(&context, repo.as_path(), &capture_work(&context, repo.as_path(), &[b"a.txt".to_vec()]).expect("the capture")).expect("kept");
    let merge: Vec<Change> = changes(repo.as_path(), old.as_str(), new.as_str());
    let mut out: Vec<u8> = Vec::new();
    clear(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, A_MERGES_ID, LockWait::BeforeTheApply, &mut out).expect("the clear");
    git(repo.as_path(), &["read-tree", "-u", "-m", old.as_str(), new.as_str()]);
    std::fs::write(repo.join("a.txt"), "written by another program\n").expect("a later write");
    let report: Report = carry(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, &[], A_MERGES_ID, LockWait::WhileFinishing, &mut out).expect("the carry");
    assert_eq!(report.changed, vec![b"a.txt".to_vec()]);
    assert_eq!(std::fs::read_to_string(repo.join("a.txt")).expect("a.txt"), "written by another program\n", "the later bytes stay");
    remove(root.as_path());
}

/// What:
///  A program's descriptor on `location`,
///  held by a process of its own.
/// Why:
///   The scan that proves no program can write into a file that was set aside skips this process,
///  so a control needs a holder the scan can see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holder = spawnOwned(['sh', '-c', 'exec 3<>"$0"; exec sleep 30', location]);
/// ```
fn a_holder_of(location: &Path) -> std::process::Child {
    let child: std::process::Child = crate::test_support::spawn_owned(
        std::process::Command::new("sh")
            .args(["-c", "exec 3<>\"$0\"; exec sleep 30", location.to_str().expect("a UTF-8 path")])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null()),
    )
    .expect("a holder process");
    return child;
}

/// What:
///  Wait until the holder's descriptor is on `location`'s own inode.
/// Why:
///   A shell's descriptor exists a moment after the start,
///  and the loader may reach descriptor 3 first,
///  so the control waits for the descriptor that names the file.
/// The helper itself starts nothing,
/// so a holder whose start fails is the test's own process to stop,
///  which `a_holder_of` makes one that dies with the test thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await until(() => identityOf(`/proc/${holder.pid}/fd/3`) === identityOf(location));
/// ```
fn the_holder_has_its_descriptor_on(holder: &std::process::Child, location: &Path) {
    use std::os::unix::fs::MetadataExt;
    let found: std::fs::Metadata = std::fs::metadata(location).expect("the file the holder opens");
    let wanted: (u64, u64) = (found.dev(), found.ino());
    let descriptor: PathBuf = PathBuf::from(format!("/proc/{}/fd/3", holder.id()));
    let started: std::time::Instant = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_secs(10) {
        match std::fs::metadata(descriptor.as_path()) {
            Ok(held) if held.dev() == wanted.0 && held.ino() == wanted.1 => return,
            _ => std::thread::sleep(std::time::Duration::from_millis(5)),
        }
    }
    panic!("the holder never opened {}", location.display());
}

/// Every file at the top of `repository` whose whole content is `bytes`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const filesHolding = (repository, bytes) => readdir(repository).filter((name) => read(join(repository, name)).equals(bytes));
/// ```
fn files_that_hold(repository: &Path, bytes: &[u8]) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(repository).expect("the worktree top").flatten() {
        let path: PathBuf = entry.path();
        if std::fs::read(path.as_path()).map(|content| { return content == bytes; }).unwrap_or(false) {
            found.push(path);
        }
    }
    return found;
}

/// What:
///  A save that arrives through a descriptor opened before the take is kept in the file that was set aside,
///  and named.
/// Why:
///  The take of a path the carry replaces was followed by an unlink of its private name after the last check,
///  so a program that still had that file open wrote its save into an entry no name reached and the bytes were
///  gone
///  (a cross-vendor review of the second round,
/// its first finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const buffer = open('a.txt'); const holder = spawnHolder('a.txt'); clear(); readTree(); carry(); write(buffer, save); expect(filesHolding(repo, save)).toHaveLength(1);
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_save_through_a_descriptor_opened_before_the_take_is_kept_beside_the_carried_file() {
    let (root, repo, old, new) = diverged("carry-save-through-descriptor");
    let context = git_context();
    std::fs::write(repo.join("a.txt"), format!("{}{PERSON_LINE}\n", LINES.replace("one", "one main"))).expect("the person's edit");
    let capture: Capture = keep_capture(&context, repo.as_path(), &capture_work(&context, repo.as_path(), &[b"a.txt".to_vec()]).expect("the capture")).expect("kept");
    let merge: Vec<Change> = changes(repo.as_path(), old.as_str(), new.as_str());
    let mut out: Vec<u8> = Vec::new();
    clear(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, A_MERGES_ID, LockWait::BeforeTheApply, &mut out).expect("the clear");
    git(repo.as_path(), &["read-tree", "-u", "-m", old.as_str(), new.as_str()]);
    let location: PathBuf = repo.join("a.txt");
    // A program opened the file before the carry, keeps that descriptor across the take, and has a process of its own that
    //  the scan can see holding it too; it saves once the carry has placed its own bytes.
    let mut buffer: std::fs::File = std::fs::File::options().read(true).write(true).open(location.as_path()).expect("the program's descriptor");
    let holder: std::process::Child = a_holder_of(location.as_path());
    the_holder_has_its_descriptor_on(&holder, location.as_path());
    let report: Report = carry(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, &[], A_MERGES_ID, LockWait::WhileFinishing, &mut out).expect("the carry");
    assert_eq!(report.changed, Vec::<Vec<u8>>::new(), "the carry wrote its own layer");
    let save: &[u8] = b"SAVE-THROUGH-OPEN-DESCRIPTOR\n";
    use std::io::{Seek, Write};
    buffer.set_len(0).expect("the buffer is replaced");
    buffer.rewind().expect("the buffer's start");
    buffer.write_all(save).expect("the save");
    buffer.sync_all().expect("the save is on disk");
    drop(buffer);
    let mut stopped: std::process::Child = holder;
    let _killed = stopped.kill();
    let _waited = stopped.wait();
    let kept: Vec<PathBuf> = files_that_hold(repo.as_path(), save);
    assert_eq!(kept.len(), 1, "the save is in one file a person can reach: {kept:?}");
    let said: String = String::from_utf8_lossy(out.as_slice()).into_owned();
    assert!(said.contains(kept[0].to_string_lossy().as_ref()), "the file that holds it is named: {said}");
    remove(root.as_path());
}

/// What:
///  A save that arrives through a descriptor opened before a take whose path the carry deletes is kept too,
///  and named.
/// Why:
///   The deletion branch of a carry unlinked the private name in the same step,
///  so a program that had the file open wrote its save into
///  an entry no name reached
///  (a cross-vendor review of the second round,
/// its first finding,
/// the deletion branch).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rm('a.txt'); clear(); readTree(); const buffer = open('a.txt'); const holder = spawnHolder('a.txt'); carry(); write(buffer, save); expect(filesHolding(repo, save)).toHaveLength(1);
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_save_through_a_descriptor_opened_before_the_take_of_a_deleted_file_is_kept() {
    let (root, repo, old, new) = diverged("carry-save-through-descriptor-deletion");
    let context = git_context();
    // The person deleted the file, which the merge keeps, so the carry removes the file the merge put there.
    std::fs::remove_file(repo.join("a.txt")).expect("the person's deletion");
    let capture: Capture = keep_capture(&context, repo.as_path(), &capture_work(&context, repo.as_path(), &[b"a.txt".to_vec()]).expect("the capture")).expect("kept");
    let merge: Vec<Change> = changes(repo.as_path(), old.as_str(), new.as_str());
    let mut out: Vec<u8> = Vec::new();
    clear(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, A_MERGES_ID, LockWait::BeforeTheApply, &mut out).expect("the clear");
    git(repo.as_path(), &["read-tree", "-u", "-m", old.as_str(), new.as_str()]);
    let location: PathBuf = repo.join("a.txt");
    let mut buffer: std::fs::File = std::fs::File::options().read(true).write(true).open(location.as_path()).expect("the program's descriptor");
    let holder: std::process::Child = a_holder_of(location.as_path());
    the_holder_has_its_descriptor_on(&holder, location.as_path());
    let report: Report = carry(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, &[], A_MERGES_ID, LockWait::WhileFinishing, &mut out).expect("the carry");
    assert_eq!(report.changed, Vec::<Vec<u8>>::new(), "the carry removed the file the merge put there");
    assert!(!location.exists(), "the path the person deleted is gone");
    let save: &[u8] = b"SAVE-AFTER-THE-DELETION\n";
    use std::io::{Seek, Write};
    buffer.set_len(0).expect("the buffer is replaced");
    buffer.rewind().expect("the buffer's start");
    buffer.write_all(save).expect("the save");
    buffer.sync_all().expect("the save is on disk");
    drop(buffer);
    let mut stopped: std::process::Child = holder;
    let _killed = stopped.kill();
    let _waited = stopped.wait();
    let kept: Vec<PathBuf> = files_that_hold(repo.as_path(), save);
    assert_eq!(kept.len(), 1, "the save is in one file a person can reach: {kept:?}");
    let said: String = String::from_utf8_lossy(out.as_slice()).into_owned();
    assert!(said.contains(kept[0].to_string_lossy().as_ref()), "the file that holds it is named: {said}");
    remove(root.as_path());
}

/// What:
///  A save that accepts the whole merge,
///  and arrives after the merge landed,
///  stands,
///  instead of the person's older layer being written over it.
/// Why:
///   The carry decided from a path's bytes alone that the file standing there was the merge's own,
///  so a person who saved the merge's version over an older unsaved line had their newer decision replaced with the older
///  layer,
///  which nothing had copied
///  (a cross-vendor review of the second round,
/// its second finding).
/// The identity of what Git's own update left is the one thing that tells the two apart,
///  and the carry now
///  asks it where the merge knows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clear(); readTree(); const landed = [['a.txt', identityOf('a.txt')]]; write('a.txt', mergeVersion); carry(landed); expect(read('a.txt')).toEqual(mergeVersion);
/// ```
#[test]
fn a_save_whose_bytes_accept_the_whole_merge_is_not_replaced_by_the_captured_layer() {
    let (root, repo, old, new) = diverged("carry-save-of-the-merge-version");
    let context = git_context();
    // The person's unsaved edit is a line of its own, which the merge's own lines do not touch.
    std::fs::write(repo.join("a.txt"), format!("{}{PERSON_LINE}\n", LINES.replace("one", "one main"))).expect("the person's edit");
    let capture: Capture = keep_capture(&context, repo.as_path(), &capture_work(&context, repo.as_path(), &[b"a.txt".to_vec()]).expect("the capture")).expect("kept");
    let merge: Vec<Change> = changes(repo.as_path(), old.as_str(), new.as_str());
    let mut out: Vec<u8> = Vec::new();
    clear(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, A_MERGES_ID, LockWait::BeforeTheApply, &mut out).expect("the clear");
    git(repo.as_path(), &["read-tree", "-u", "-m", old.as_str(), new.as_str()]);
    let location: PathBuf = repo.join("a.txt");
    // What Git's own update left, read where the merge lands and before any save can arrive.
    let landed: Vec<LandedFile> = crate::merge_isolation_carry_landed::landed_of(repo.as_path(), &[b"a.txt".to_vec()]);
    let saved: Vec<u8> = std::fs::read(location.as_path()).expect("the file Git wrote");
    // The editor saves its buffer, which holds the merge's version and no longer the person's older line.
    std::fs::write(location.as_path(), saved.as_slice()).expect("the save");
    let report: Report = carry(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, landed.as_slice(), A_MERGES_ID, LockWait::WhileFinishing, &mut out).expect("the carry");
    assert_eq!(report.changed, vec![b"a.txt".to_vec()], "the carry says the path was written after the capture and leaves it");
    assert_eq!(std::fs::read(location.as_path()).expect("a.txt"), saved, "the newer save stands");
    remove(root.as_path());
}

/// What:
///  A save that accepts the whole merge and lands in the instant between Git's own write and the reading of it
///  is read as Git's own,
///  so the file ends holding both sides rather than only the newer save.
/// Why:
///   No evidence this process can read tells the two apart:
/// the save leaves the merge's own bytes,
///  its identity is the one Git's write made,
///  and no clock here orders the save against that write
/// (`SPEC.md`,
/// "Merges and commits never fail",
/// the window the landed reading names).
/// The control pins what that leaves:
/// the merge's lines and the person's older line are both in the file,
///  the command names the path,
/// and nothing either side wrote is discarded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clear(); readTree(); write('a.txt', mergeVersion); const landed = readLanded('a.txt'); carry(landed); expect(read('a.txt')).toContainBoth(FEATURE_LINE, PERSON_LINE);
/// ```
#[test]
fn a_save_that_lands_before_the_reading_keeps_both_sides() {
    let (root, repo, old, new) = diverged("carry-save-before-the-reading");
    let context = git_context();
    // The person's unsaved edit is a line of its own, which the merge's own lines do not touch.
    std::fs::write(repo.join("a.txt"), format!("{}{PERSON_LINE}\n", LINES.replace("one", "one main"))).expect("the person's edit");
    let capture: Capture = keep_capture(&context, repo.as_path(), &capture_work(&context, repo.as_path(), &[b"a.txt".to_vec()]).expect("the capture")).expect("kept");
    let merge: Vec<Change> = changes(repo.as_path(), old.as_str(), new.as_str());
    let mut out: Vec<u8> = Vec::new();
    clear(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, A_MERGES_ID, LockWait::BeforeTheApply, &mut out).expect("the clear");
    git(repo.as_path(), &["read-tree", "-u", "-m", old.as_str(), new.as_str()]);
    let location: PathBuf = repo.join("a.txt");
    let merged: Vec<u8> = std::fs::read(location.as_path()).expect("the file Git wrote");
    // The editor saves its buffer, which holds the merge's version and no longer the person's older line,
    //  and the reading is taken after that save, as a reading that arrives in that instant is.
    std::fs::write(location.as_path(), merged.as_slice()).expect("the save");
    let landed: Vec<LandedFile> = crate::merge_isolation_carry_landed::landed_of(repo.as_path(), &[b"a.txt".to_vec()]);
    let report: Report = carry(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, landed.as_slice(), A_MERGES_ID, LockWait::WhileFinishing, &mut out).expect("the carry");
    let after: String = std::fs::read_to_string(location.as_path()).expect("a.txt");
    assert!(after.contains("three feature"), "the merge's line is in the file: {after:?}");
    assert!(after.contains(PERSON_LINE), "the person's older line is in the file rather than discarded: {after:?}");
    assert_eq!(report.conflicts.len() + report.merged.len() + report.changed.len(), 1, "the command names the one path it touched: {report:?}");
    remove(root.as_path());
}
