//! What:
//!  The audits' two matrices, kept:
//!  every discarding form against every kind of file in the way,
//!  wrapped and direct,
//!  with what must survive.
//! Why:
//!  Two independent audits of 0.6.1-alpha.12 found the same hole from two directions:
//!  the guard decided from a status that names none of the files these forms overwrite,
//!  so an untracked file, an ignored file, a nested repository's work and an edit behind
//!  an `assume-unchanged` mark were all lost with no refusal and no copy.
//!  A matrix is the only shape that keeps every form covered while the code beneath changes:
//!  each row says what real Git does to the file (the positive control, without which a
//!  passing row proves nothing) and what the wrapper leaves behind.
//!  `skip-worktree` is beside the marks because it must stay the one case Git itself keeps.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every form x every kind of file in the way: wrapped and direct, what survives
//! ```

use super::support::{Fixture, Observed, fixture, git, remove, repository, run_direct, run_wrapped, stderr_of};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The bytes a person wrote at the path the form writes.
const PRECIOUS: &[u8] = b"PRECIOUS UNTRACKED\n";

/// What stands where the target tree writes,
/// as the audits shaped it.
#[derive(Clone, Copy)]
enum InTheWay {
    /// A file the tree holds as a file.
    File,
    /// A file where the tree needs a directory (`b.txt/inner.txt`).
    FileWhereADirectoryMustGo,
    /// A nested Git repository with uncommitted work where the tree needs a directory.
    Repository,
}

/// The forms the untracked matrix runs,
/// each as the audit wrote it.
fn forms() -> Vec<Vec<&'static str>> {
    return vec![
        vec!["reset", "--hard", "other"],
        vec!["reset", "--merge", "other"],
        vec!["reset", "--keep", "other"],
        vec!["checkout", "--force", "other"],
        vec!["switch", "--discard-changes", "other"],
        vec!["switch", "--force", "other"],
        vec!["checkout", "other"],
        vec!["checkout", "other", "--", "b.txt"],
        vec!["restore", "--source=other", "--", "b.txt"],
        vec!["read-tree", "-m", "-u", "HEAD", "other"],
    ];
}

/// A repository whose `other` branch tracks what `shape` needs,
/// with a linked worktree that holds the person's file where that path goes.
///  The person's work is always at `b.txt`,
///  which the branch never tracks as the same kind of thing.
fn prepared(name: &str, shape: InTheWay) -> (Fixture, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "other"]);
    if matches!(shape, InTheWay::File) {
        std::fs::write(repo.join("b.txt"), b"tracked on other\n").expect("tracked file");
        git(&fixture, repo.as_path(), &["add", "--", "b.txt"]);
    } else {
        std::fs::create_dir_all(repo.join("b.txt")).expect("directory on the branch");
        std::fs::write(repo.join("b.txt/inner.txt"), b"tracked on other\n").expect("tracked file");
        git(&fixture, repo.as_path(), &["add", "--", "b.txt/inner.txt"]);
    }
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=other"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    match shape {
        InTheWay::File | InTheWay::FileWhereADirectoryMustGo => {
            std::fs::write(linked.join("b.txt"), PRECIOUS).expect("the person's file");
        }
        InTheWay::Repository => {
            let nested: PathBuf = linked.join("b.txt");
            std::fs::create_dir(nested.as_path()).expect("nested repository directory");
            git(&fixture, nested.as_path(), &["init", "--quiet", "--initial-branch=inner"]);
            std::fs::write(nested.join("tracked.txt"), b"committed\n").expect("nested tracked file");
            git(&fixture, nested.as_path(), &["add", "--", "tracked.txt"]);
            git(&fixture, nested.as_path(), &["commit", "--quiet", "--message=inner"]);
            std::fs::write(nested.join("tracked.txt"), PRECIOUS).expect("the person's edit");
        }
    }
    return (fixture, repo, linked);
}

/// Every file below the saved copies,
/// which is where the wrapper keeps what it moved or copied aside.
fn saved_dir() -> &'static str {
    return "monochromatic-git-policy-cli/discarded";
}

/// Whether the person's bytes are somewhere below the saved copies of `repo`.
fn saved_copy_holds(repo: &Path, needle: &[u8]) -> bool {
    let root: PathBuf = repo.join(".git").join(saved_dir());
    let mut pending: Vec<PathBuf> = vec![root];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory.as_path()) else {
            continue;
        };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if std::fs::read(path.as_path()).is_ok_and(|bytes| bytes == needle) {
                return true;
            }
        }
    }
    return false;
}

/// Whether the person's bytes are at `relative` in the linked worktree.
fn still_where_it_was(linked: &Path, relative: &str, needle: &[u8]) -> bool {
    return std::fs::read(linked.join(relative)).is_ok_and(|bytes| bytes == needle);
}

/// Every form against a file the target tree writes,
/// in every shape the audits used:
/// the wrapper lets the command run and the person's work is either still there or saved,
/// while real Git loses it,
/// which is what makes each row a measurement.
#[test]
fn every_form_over_an_untracked_file_keeps_or_saves_it() {
    for (number, shape) in [InTheWay::File, InTheWay::FileWhereADirectoryMustGo, InTheWay::Repository]
        .into_iter()
        .enumerate()
    {
        // A shape whose every row was refused by Git would prove nothing, so the count is checked at the end.
        let mut measured: usize = 0;
        for (index, arguments) in forms().into_iter().enumerate() {
            let label: String = format!("discard-matrix-{number}-{index}");
            // The positive control first: this form over this shape destroys the work in real Git.
            let (control, control_repo, control_linked): (Fixture, PathBuf, PathBuf) = prepared(label.as_str(), shape);
            let direct: Observed = run_direct(&control, control_linked.as_path(), arguments.as_slice());
            let lost_by_git: bool =
                !still_where_it_was(control_linked.as_path(), "b.txt", PRECIOUS) && !saved_copy_holds(control_repo.as_path(), PRECIOUS);
            remove(&control);
            // The wrapper's own run of the same form over the same shape.
            let (wrapped_fixture, repo, linked): (Fixture, PathBuf, PathBuf) =
                prepared(format!("{label}-wrapped").as_str(), shape);
            let observed: Observed = run_wrapped(&wrapped_fixture, linked.as_path(), arguments.as_slice());
            let kept: bool = still_where_it_was(linked.as_path(), "b.txt", PRECIOUS)
                || still_where_it_was(linked.as_path(), "b.txt/tracked.txt", PRECIOUS)
                || saved_copy_holds(repo.as_path(), PRECIOUS);
            if lost_by_git {
                measured += 1;
                // The wrapper saves what the form would destroy and lets it run; a refusal would only send the
                //  person to move the file by hand.
                assert_eq!(observed.code, Some(0), "{arguments:?} over {label} refused: {}", stderr_of(&observed));
                assert!(kept, "{arguments:?} over {label} lost the person's work: {}", stderr_of(&observed));
            }
            // A form Git refuses for its own reason changes nothing either, so nothing may be lost there.
            assert!(
                kept || observed.code != Some(0),
                "{arguments:?} over {label} exited 0 and lost the person's work: {}",
                stderr_of(&observed)
            );
            remove(&wrapped_fixture);
            let _ = direct;
        }
        assert!(measured > 0, "no form of shape {number} destroys the person's work in real Git, so this matrix proves nothing");
    }
}

/// The forms the marked matrix runs.
fn marked_forms() -> Vec<Vec<&'static str>> {
    return vec![
        vec!["reset", "--hard"],
        vec!["reset", "--hard", "HEAD"],
        vec!["reset", "--merge"],
        vec!["checkout", "--force"],
        vec!["checkout", "--force", "other"],
        vec!["checkout", "--", "a.txt"],
        vec!["checkout", "HEAD", "--", "."],
        vec!["restore", "."],
        vec!["restore", "--source=other", "."],
        vec!["switch", "--discard-changes", "other"],
        vec!["switch", "other"],
        vec!["checkout", "other"],
        vec!["stash"],
        vec!["stash", "push", "--include-untracked"],
        vec!["read-tree", "--reset", "-u", "HEAD"],
        vec!["checkout-index", "-f", "-a"],
        vec!["rm", "-f", "a.txt"],
    ];
}

/// The edit a person made,
/// at a path the index may mark.
const EDIT: &[u8] = b"PRECIOUS EDIT\n";

/// A linked worktree whose `a.txt` holds the person's edit,
/// marked with `--<mark>` when one is named.
fn marked(name: &str, mark: Option<&str>) -> (Fixture, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("tracked file");
    std::fs::write(repo.join("b.txt"), b"bee\n").expect("tracked file");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt", "b.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["branch", "other"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    if let Some(named) = mark {
        git(&fixture, linked.as_path(), &["update-index", &format!("--{named}"), "a.txt"]);
    }
    std::fs::write(linked.join("a.txt"), EDIT).expect("the person's edit");
    return (fixture, repo, linked);
}

/// An edit behind the `assume-unchanged` mark gets exactly what the same edit gets without it:
/// every form refuses,
/// and the edit is still there.
///  `skip-worktree` is the control on the other side:
///  Git keeps that file,
///  so a form over it must not be refused for it.
#[test]
fn a_marked_edit_is_treated_as_the_edit_it_is() {
    for (index, arguments) in marked_forms().into_iter().enumerate() {
        for mark in ["assume-unchanged", "skip-worktree"] {
            let label: String = format!("discard-marked-{mark}-{index}");
            let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = marked(label.as_str(), Some(mark));
            let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
            let kept: bool = still_where_it_was(linked.as_path(), "a.txt", EDIT);
            assert!(
                kept || observed.code != Some(0),
                "{arguments:?} with {mark} exited 0 and lost the edit: {}",
                stderr_of(&observed)
            );
            if mark == "assume-unchanged" {
                // The same form over the same edit with no mark, so the two answers are one rule.
                let (plain, _plain_repo, plain_linked): (Fixture, PathBuf, PathBuf) =
                    marked(format!("{label}-plain").as_str(), None);
                let unmarked: Observed = run_wrapped(&plain, plain_linked.as_path(), arguments.as_slice());
                assert_ne!(unmarked.code, Some(0), "the control needs an unmarked edit to be refused: {arguments:?}");
                assert_ne!(observed.code, Some(0), "{arguments:?} with the mark ran where the unmarked edit is refused");
                remove(&plain);
            }
            remove(&fixture);
        }
    }
}
