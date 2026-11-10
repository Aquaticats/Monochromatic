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
    clear(&context, repo.as_path(), repo.join(".git").as_path(), changes(repo.as_path(), old.as_str(), new.as_str()).as_slice(), &capture, LockWait::BeforeTheApply, &mut out).expect("the clear");
    assert_eq!(std::fs::read_to_string(repo.join("a.txt")).expect("a.txt"), LINES.replace("one", "one main"), "the path is the branch's");
    assert_eq!(entry(repo.as_path(), "a.txt"), format!("100644 {}", git_text(repo.as_path(), &["rev-parse", "HEAD:a.txt"]).trim()), "the index is the branch's");
    restore(&context, repo.as_path(), repo.join(".git").as_path(), changes(repo.as_path(), old.as_str(), new.as_str()).as_slice(), &capture, LockWait::WhileFinishing, &mut out).expect("the restore");
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
    clear(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, LockWait::BeforeTheApply, &mut out).expect("the clear");
    // Git's own two-way update of the branch, which is what the merge does with the paths free.
    git(repo.as_path(), &["read-tree", "-u", "-m", old.as_str(), new.as_str()]);
    let report: Report = carry(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, LockWait::WhileFinishing, &mut out).expect("the carry");
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
    clear(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, LockWait::BeforeTheApply, &mut out).expect("the clear");
    git(repo.as_path(), &["read-tree", "-u", "-m", old.as_str(), new.as_str()]);
    let report: Report = carry(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, LockWait::WhileFinishing, &mut out).expect("the carry");
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
    clear(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &conflicting, LockWait::BeforeTheApply, &mut out).expect("the clear");
    git(repo.as_path(), &["read-tree", "-u", "-m", old.as_str(), new.as_str()]);
    let conflicted: Report = carry(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &conflicting, LockWait::WhileFinishing, &mut out).expect("the carry");
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
    clear(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, LockWait::BeforeTheApply, &mut out).expect("the clear");
    git(repo.as_path(), &["read-tree", "-u", "-m", old.as_str(), new.as_str()]);
    std::fs::write(repo.join("a.txt"), "written by another program\n").expect("a later write");
    let report: Report = carry(&context, repo.as_path(), repo.join(".git").as_path(), merge.as_slice(), &capture, LockWait::WhileFinishing, &mut out).expect("the carry");
    assert_eq!(report.changed, vec![b"a.txt".to_vec()]);
    assert_eq!(std::fs::read_to_string(repo.join("a.txt")).expect("a.txt"), "written by another program\n", "the later bytes stay");
    remove(root.as_path());
}
