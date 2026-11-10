//! The repository's own exclude line,
//! and that real Git honours it in a linked worktree.

use super::{TRASH_EXCLUDE_LINE, ensure_trash_is_excluded};
use crate::test_support::{fixture, git, git_text, remove, repository};
use crate::worktree_trash::TRASH_PARENT;
use crate::worktree_trash_ignore_rule::trash_location_is_ignored;
use std::path::{Path, PathBuf};

/// Write a file and its directories.
fn put(root: &Path, path: &str, bytes: &[u8]) {
    let full: PathBuf = root.join(path);
    std::fs::create_dir_all(full.parent().expect("a parent")).expect("directories");
    std::fs::write(full, bytes).expect("the file");
}

/// The bytes of a fixture file.
fn read(root: &Path, path: &str) -> String {
    return String::from_utf8(std::fs::read(root.join(path)).expect("the file")).expect("text");
}

/// What:
///  The line is added once,
/// the person's own patterns are kept as they were,
/// and a file that did not end in a newline
///  still gets the line as a line of its own.
/// Why:
///   `info/exclude` is the person's file;
/// the wrapper adds one line to it and must never rewrite what is there.
#[test]
fn the_line_joins_the_persons_patterns_without_rewriting_them() {
    let root: PathBuf = fixture("trash-exclude-line");
    put(root.as_path(), ".git/info/exclude", b"node_modules/\n*.log");
    let added: bool = ensure_trash_is_excluded(root.join(".git").as_path()).expect("the line is written");
    assert!(added, "the line was not there before");
    assert_eq!(read(root.as_path(), ".git/info/exclude"), format!("node_modules/\n*.log\n{TRASH_EXCLUDE_LINE}\n"), "the person's bytes are kept and the line joins them");
    let again: bool = ensure_trash_is_excluded(root.join(".git").as_path()).expect("read again");
    assert!(!again, "a second add of the same worktree writes nothing");
    assert_eq!(read(root.as_path(), ".git/info/exclude"), format!("node_modules/\n*.log\n{TRASH_EXCLUDE_LINE}\n"), "and the file is exactly as it was");
    remove(root.as_path());
}

/// A repository with no `info/exclude` at all gets one holding the line.
#[test]
fn a_repository_with_no_exclude_file_gets_one() {
    let root: PathBuf = fixture("trash-exclude-absent");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::remove_file(repo.join(".git/info/exclude")).expect("Git writes one at init");
    let added: bool = ensure_trash_is_excluded(repo.join(".git").as_path()).expect("the line is written");
    assert!(added);
    assert_eq!(read(repo.as_path(), ".git/info/exclude"), format!("{TRASH_EXCLUDE_LINE}\n"));
    remove(root.as_path());
}

/// What:
///  Real Git honours the repository's line in a linked worktree:
/// before the line the trash place is not ignored
///  and shows as untracked,
/// after it `git check-ignore` names the repository's own file and `git add --all` stages
///  nothing of a trash that stands there.
/// Why:
///   A linked worktree reads `$GIT_COMMON_DIR/info/exclude`,
/// never its own worktree's,
/// and that is measured rather than
///  assumed:
/// a line written for the worktree alone leaves the trash stageable.
/// The trash a target commit could stage is another
///  command's state,
/// so the proof is `git add --all` itself and not the ignore query alone.
#[test]
fn real_git_keeps_a_standing_trash_out_of_the_index_of_a_linked_worktree() {
    let root: PathBuf = fixture("trash-exclude-git");
    let repo: PathBuf = repository(root.as_path(), "repo");
    put(repo.as_path(), "tracked.txt", b"tracked\n");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--detach", linked.to_str().expect("text")]);
    let trash: String = format!("{TRASH_PARENT}/{}/d0", "a".repeat(40));
    put(linked.as_path(), format!("{trash}/payload/credentials").as_str(), b"another worktree's token\n");
    assert!(!trash_location_is_ignored(&crate::test_support::git_context(), linked.as_path()), "without the line the trash place is not ignored");
    assert_eq!(git_text(linked.as_path(), &["status", "--porcelain"]), "?? .claude/", "and Git shows the trash as untracked");
    let added: bool = ensure_trash_is_excluded(repo.join(".git").as_path()).expect("the line is written");
    assert!(added);
    assert!(trash_location_is_ignored(&crate::test_support::git_context(), linked.as_path()), "with the line the trash place is ignored");
    assert_eq!(git_text(linked.as_path(), &["status", "--porcelain"]), "", "Git lists nothing untracked any more");
    git(linked.as_path(), &["add", "--all"]);
    assert_eq!(git_text(linked.as_path(), &["diff", "--cached", "--name-only"]), "", "and `git add --all` stages nothing of it");
    remove(root.as_path());
}

/// What:
///  A target commit that stops ignoring the trash place does not open it again,
/// and neither does one that tries to
///  re-include it.
/// Why:
///   The commit is what changes the ignore rules,
/// so it must not be able to decide whether another command's state
///  is stageable.
/// `info/exclude` outranks nothing a commit can write for a path below a directory it excludes,
/// and the
///  measurement is `git add --all` after the commit,
/// with the trash standing.
#[test]
fn a_target_commit_that_stops_ignoring_the_trash_place_cannot_stage_it() {
    let root: PathBuf = fixture("trash-exclude-target");
    let repo: PathBuf = repository(root.as_path(), "repo");
    put(repo.as_path(), "tracked.txt", b"tracked\n");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--detach", linked.to_str().expect("text")]);
    ensure_trash_is_excluded(repo.join(".git").as_path()).expect("the line is written");
    let trash: String = format!("{TRASH_PARENT}/{}/d0", "b".repeat(40));
    put(linked.as_path(), format!("{trash}/payload/credentials").as_str(), b"another worktree's token\n");
    for rules in [".gitignore ignores nothing under .claude:\nnode_modules/\n", ".gitignore tries to re-include the trash parent:\n.claude/\n!.claude/cli-git-trash/\n"] {
        put(linked.as_path(), ".gitignore", rules.as_bytes());
        git(linked.as_path(), &["add", "--all"]);
        let staged: String = git_text(linked.as_path(), &["diff", "--cached", "--name-only"]);
        assert_eq!(staged, ".gitignore", "{rules}: only the commit's own file is staged and never the trash");
        git(linked.as_path(), &["reset", "--quiet"]);
    }
    remove(root.as_path());
}
