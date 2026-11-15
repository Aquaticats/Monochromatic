//! What:
//!  Every option the guard reads to decide what a command writes,
//!  with what Git does when it is given twice and what the guard does,
//!  each row checked against real Git.
//! Why:
//!  An option written twice is the last one, negation included,
//!  and a reading that stopped at the first spelling disagreed with Git:
//!  `git checkout --no-guess --guess --force topic` created and checked out the guessed branch while
//!  the guard read no tree at all, and `git rm -f --no-force` was read as a forced removal Git refuses.
//!  The table is the authority: every option the written-path reading consults is a row,
//!  and a row whose guard reading and real Git disagree fails.
//!  Options the guard does not read are named in the same table with why they cannot disagree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for each repeated option: guardState(positiveThenNegative) === gitState(positiveThenNegative)
//! ```

use crate::command_clean::parse_clean_region;
use crate::command_discard::{Discard, force_discard};
use crate::command_test_support::{git, git_status, repository_with_tracked_file};
use crate::command_written_paths::{PathspecTarget, WrittenPaths, written_paths};
use crate::rule_linked_worktree::{GuardedCommand, dirty_question};
use crate::rule_worktree_dirty::DirtyScope;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The words as arguments, each its own.
fn words(values: &[&str]) -> Vec<OsString> {
    return values.iter().map(OsString::from).collect();
}

/// What:
///  A repository with a tracked file edited in the worktree, and a linked worktree on the same file.
///  `&str` names the fixture; `&[(&str, &[u8])]` is what the base commit holds.
/// Why:
///  A forced form's state is only visible on a worktree that differs from the index,
///  so every probe for `--force` needs that difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { repo, linked } = editedFixture('repeated-rm-force', [['m.txt', 'base\n']]);
/// ```
fn edited_fixture(name: &str, files: &[(&str, &[u8])]) -> (PathBuf, PathBuf, PathBuf) {
    let (root, repository): (PathBuf, PathBuf) = repository_with_tracked_file(name);
    for (file, bytes) in files {
        std::fs::write(repository.join(file), bytes).expect("a tracked file");
        git(repository.as_path(), &["add", "--", file]);
    }
    if !files.is_empty() {
        git(repository.as_path(), &["commit", "--quiet", "--message=files"]);
    }
    let linked: PathBuf = root.join("linked");
    git(repository.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    return (root, repository, linked);
}

/// The branch a checkout created, or nothing when Git refused the name.
fn branch_created(repository: &Path, name: &str) -> bool {
    return git_status(repository, &["rev-parse", "--verify", "--quiet", &format!("refs/heads/{name}")]).status.success();
}

/// What:
///  `--no-guess --guess` guesses and `--guess --no-guess` does not,
///  in real Git and in the guard's reading of the tree a forced checkout writes.
/// Why:
///  Git reads the guessed branch and checks out its tree, so a guard that read no tree inspected
///  nothing while Git wrote over the person's untracked file.
#[test]
fn a_repeated_guess_is_the_last_spelling_in_git_and_in_the_guard() {
    let (root, repository, _linked): (PathBuf, PathBuf, PathBuf) = edited_fixture("repeated-guess", &[("base.txt", b"base\n")]);
    std::fs::write(repository.join("victim.txt"), b"remote\n").expect("the guessed tree's file");
    git(repository.as_path(), &["add", "--", "victim.txt"]);
    git(repository.as_path(), &["commit", "--quiet", "--message=remote"]);
    let remote: String = String::from_utf8_lossy(&git(repository.as_path(), &["rev-parse", "HEAD"]).stdout).trim().to_owned();
    git(repository.as_path(), &["reset", "--hard", "HEAD~1"]);
    git(repository.as_path(), &["remote", "add", "origin", repository.to_str().expect("utf-8 path")]);
    git(repository.as_path(), &["update-ref", "refs/remotes/origin/topic", remote.as_str()]);
    // The positive first, then the negative: Git obeys the negative, so no branch is made.
    let refused = git_status(repository.as_path(), &["checkout", "--guess", "--no-guess", "topic"]);
    assert!(!refused.status.success(), "Git refuses the name when the last spelling turns guessing off");
    assert!(!branch_created(repository.as_path(), "topic"), "no branch was created");
    // The negative first, then the positive: Git obeys the positive and creates the branch.
    let guessed = git_status(repository.as_path(), &["checkout", "--no-guess", "--guess", "topic"]);
    assert!(guessed.status.success(), "Git guesses when the last spelling turns guessing on: {guessed:?}");
    assert!(branch_created(repository.as_path(), "topic"), "the guessed branch exists");
    // The guard's reading of the tree a forced checkout writes follows the same last spelling.
    let guess_of = |values: &[&str]| -> bool {
        return match written_paths(GuardedCommand::CheckoutPaths, words(values).as_slice()) {
            Some(WrittenPaths::Tree { guesses_a_remote_branch, .. }) => guesses_a_remote_branch,
            other => panic!("a forced checkout of a name writes a tree: {other:?}"),
        };
    };
    assert!(!guess_of(&["-f", "--guess", "--no-guess", "topic"]), "the last spelling turns the guess off");
    assert!(guess_of(&["-f", "--no-guess", "--guess", "topic"]), "the last spelling turns the guess on");
    crate::command_test_support::remove(root.as_path());
}

/// What:
///  `--no-force` after `-f` is an unforced removal,
///  which Git refuses for a modified file and the guard must judge unforced.
/// Why:
///  The guard's forced-removal policy exists for a removal that overwrites;
///  a scan that stopped at the first `-f` read `git rm -f --no-force` as one of those.
#[test]
fn a_repeated_remove_force_is_the_last_spelling_in_git_and_in_the_guard() {
    let (root, _repository, linked): (PathBuf, PathBuf, PathBuf) = edited_fixture("repeated-rm-force", &[("m.txt", b"base\n")]);
    let file: PathBuf = linked.join("m.txt");
    std::fs::write(file.as_path(), b"modified\n").expect("the edit");
    let unforced = git_status(linked.as_path(), &["rm", "-f", "--no-force", "m.txt"]);
    assert!(!unforced.status.success(), "Git refuses a modified file when the last spelling is --no-force");
    assert!(file.exists(), "the file is where it was");
    let forced = git_status(linked.as_path(), &["rm", "--no-force", "-f", "m.txt"]);
    assert!(forced.status.success(), "Git removes it when the last spelling is -f: {forced:?}");
    assert!(!file.exists(), "the file is gone");
    assert!(matches!(force_discard(b"rm", words(&["-f", "--no-force", "m.txt"]).as_slice()), Discard::Harmless));
    assert!(matches!(force_discard(b"rm", words(&["--no-force", "-f", "m.txt"]).as_slice()), Discard::Overwrites));
    crate::command_test_support::remove(root.as_path());
}

/// What:
///  `-f --no-force` and `--no-force -f` for `checkout-index`,
///  in real Git and in the guard's forced-form reading.
/// Why:
///  `checkout-index --force` writes over the worktree copy and without it writes nothing over;
///  the guard's policy follows the same last spelling.
#[test]
fn a_repeated_checkout_index_force_is_the_last_spelling_in_git_and_in_the_guard() {
    let (root, _repository, linked): (PathBuf, PathBuf, PathBuf) = edited_fixture("repeated-checkout-index-force", &[("a.txt", b"index\n")]);
    let file: PathBuf = linked.join("a.txt");
    std::fs::write(file.as_path(), b"edited\n").expect("the edit");
    let unforced = git_status(linked.as_path(), &["checkout-index", "-f", "--no-force", "--", "a.txt"]);
    assert!(!unforced.status.success(), "without force Git refuses to write over the file: {unforced:?}");
    assert_eq!(std::fs::read(file.as_path()).expect("the file"), b"edited\n", "the last spelling is --no-force");
    git(linked.as_path(), &["checkout-index", "--no-force", "-f", "--", "a.txt"]);
    assert_eq!(std::fs::read(file.as_path()).expect("the file"), b"index\n", "the last spelling is -f");
    assert!(matches!(
        force_discard(b"checkout-index", words(&["-f", "--no-force", "--", "a.txt"]).as_slice()),
        Discard::Harmless
    ));
    assert!(matches!(
        force_discard(b"checkout-index", words(&["--no-force", "-f", "--", "a.txt"]).as_slice()),
        Discard::Overwrites
    ));
    crate::command_test_support::remove(root.as_path());
}

/// What:
///  `-a --no-all` and `--no-all -a` for `checkout-index`,
///  in real Git and in the paths the guard reads.
/// Why:
///  `--all` makes the form write the whole index and `--no-all` takes that back,
///  so the guard's written set is the whole index only when the last spelling is `-a`.
#[test]
fn a_repeated_all_is_the_last_spelling_in_git_and_in_the_guard() {
    let (root, _repository, linked): (PathBuf, PathBuf, PathBuf) =
        edited_fixture("repeated-checkout-index-all", &[("a.txt", b"index a\n"), ("b.txt", b"index b\n")]);
    for file in ["a.txt", "b.txt"] {
        std::fs::write(linked.join(file), b"edited\n").expect("the edit");
    }
    // Git refuses to mix `--all` with filenames, so the probe names none.
    let taken_back = git_status(linked.as_path(), &["checkout-index", "-f", "-a", "--no-all"]);
    assert!(taken_back.status.success(), "with --all taken back and no path, Git checks out nothing and exits 0: {taken_back:?}");
    assert_eq!(std::fs::read(linked.join("a.txt")).expect("a"), b"edited\n", "the last spelling is --no-all");
    assert_eq!(std::fs::read(linked.join("b.txt")).expect("b"), b"edited\n", "nothing else was written");
    for file in ["a.txt", "b.txt"] {
        std::fs::write(linked.join(file), b"edited\n").expect("the edit");
    }
    git(linked.as_path(), &["checkout-index", "-f", "--no-all", "-a"]);
    assert_eq!(std::fs::read(linked.join("a.txt")).expect("a"), b"index a\n", "the last spelling is -a");
    assert_eq!(std::fs::read(linked.join("b.txt")).expect("b"), b"index b\n", "all of the index was written");
    let paths_of = |values: &[&str]| -> Option<Vec<OsString>> {
        return match written_paths(GuardedCommand::CheckoutIndex, words(values).as_slice()) {
            Some(WrittenPaths::Index { pathspecs }) => Some(pathspecs),
            None => None,
            other => panic!("a checkout-index writes the index: {other:?}"),
        };
    };
    assert_eq!(
        paths_of(&["-f", "-a", "--no-all"]),
        None,
        "with --all taken back and no path, Git has nothing to check out and the guard reads no write"
    );
    assert_eq!(
        paths_of(&["-f", "--no-all", "-a"]),
        Some(Vec::<OsString>::new()),
        "with -a last, the guard writes the whole index"
    );
    crate::command_test_support::remove(root.as_path());
}

/// What:
///  `--pathspec-file-nul` and `--no-pathspec-file-nul` on a checkout,
///  in real Git and in the separator the guard reads the file with.
/// Why:
///  The file is NUL-separated only when the last spelling says so;
///  a reading that kept the first `--pathspec-file-nul` read a line-separated list as NUL-separated,
///  which is a different set of paths.
#[test]
fn a_repeated_pathspec_file_nul_is_the_last_spelling_in_git_and_in_the_guard() {
    let (root, _repository, linked): (PathBuf, PathBuf, PathBuf) =
        edited_fixture("repeated-pathspec-file-nul", &[("a.txt", b"index a\n"), ("b.txt", b"index b\n")]);
    let list: PathBuf = linked.join("nul-list");
    // Two paths, each followed by the NUL separator.
    std::fs::write(list.as_path(), b"a.txt\x00b.txt\x00").expect("the list");
    for file in ["a.txt", "b.txt"] {
        std::fs::write(linked.join(file), b"edited\n").expect("the edit");
    }
    // The NUL separator first and then taken back: the file is read by lines, so the run of both names and their
    //  NULs is one entry and reaches no second path.
    let lines = git_status(linked.as_path(), &["checkout", "-f", "--pathspec-from-file=nul-list", "--pathspec-file-nul", "--no-pathspec-file-nul"]);
    assert!(lines.status.success(), "Git reads the file by lines: {lines:?}");
    assert_eq!(
        std::fs::read(linked.join("b.txt")).expect("b"),
        b"edited\n",
        "read by lines, the second path of the NUL-separated file is not reached"
    );
    // Taken back first and then set: the file is read by NUL and both paths are written.
    git(linked.as_path(), &["checkout", "-f", "--pathspec-from-file=nul-list", "--no-pathspec-file-nul", "--pathspec-file-nul"]);
    assert_eq!(std::fs::read(linked.join("a.txt")).expect("a"), b"index a\n", "the last spelling is --pathspec-file-nul");
    assert_eq!(std::fs::read(linked.join("b.txt")).expect("b"), b"index b\n", "both paths of the list are written");
    let nul_of = |values: &[&str]| -> bool {
        return match written_paths(GuardedCommand::CheckoutPaths, words(values).as_slice()) {
            Some(WrittenPaths::PathspecFile { nul_separated, .. }) => nul_separated,
            other => panic!("a pathspec file is read as a file of pathspecs: {other:?}"),
        };
    };
    assert!(!nul_of(&["-f", "--pathspec-from-file=nul-list", "--pathspec-file-nul", "--no-pathspec-file-nul"]));
    assert!(nul_of(&["-f", "--pathspec-from-file=nul-list", "--no-pathspec-file-nul", "--pathspec-file-nul"]));
    // The target of the form is the revision it names, whatever the separator.
    assert!(matches!(
        written_paths(GuardedCommand::CheckoutPaths, words(&["-f", "topic", "--pathspec-from-file=nul-list"]).as_slice()),
        Some(WrittenPaths::PathspecFile { target: PathspecTarget::Tree(Some(_)), .. })
    ));
    crate::command_test_support::remove(root.as_path());
}

/// What:
///  A clean's `--no-force` resets the count Git reads,
///  and the guard's reading of the same words counts the same forces.
/// Why:
///  `OPTION_COUNTUP` resets on its negation,
///  so `clean -f -f --no-force -f` is one force,
///  and the guard decides what a clean deletes from that count.
#[test]
fn a_repeated_clean_force_is_the_last_spelling_in_git_and_in_the_guard() {
    let (root, repository, _linked): (PathBuf, PathBuf, PathBuf) = edited_fixture("repeated-clean-force", &[]);
    std::fs::write(repository.join(".gitignore"), b"ignored/\n").expect("ignore rules");
    std::fs::create_dir_all(repository.join("ignored")).expect("the ignored directory");
    let forces_of = |values: &[&str]| -> usize {
        return parse_clean_region(words(values).as_slice(), &[]).expect("a clean Git accepts").forces;
    };
    // Taken back to none: Git refuses the clean while it requires a force, and the guard reads zero forces.
    let unforced = git_status(repository.as_path(), &["clean", "-f", "-f", "--no-force", "-f", "--no-force", "-d", "-X"]);
    assert!(!unforced.status.success(), "zero forces is a clean Git refuses here: {unforced:?}");
    assert_eq!(forces_of(&["-f", "-f", "--no-force", "-f", "--no-force", "-d", "-X"]), 0);
    // One force last: Git runs the clean, and the guard reads one force.
    let forced = git_status(repository.as_path(), &["clean", "--no-force", "-f", "-d", "-X"]);
    assert!(forced.status.success(), "one force runs: {forced:?}");
    assert_eq!(forces_of(&["--no-force", "-f", "-d", "-X"]), 1);
    crate::command_test_support::remove(root.as_path());
}

/// What:
///  An unforced `rm --sparse --no-sparse` keeps the skip-worktree edit and `--no-sparse --sparse`
///  reaches it,
///  and the guard asks about hidden edits only in the second case.
/// Why:
///  The scope of a removal's question is whether it reaches a skipped edit,
///  and that is the last spelling too.
#[test]
fn a_repeated_remove_sparse_is_the_last_spelling_in_git_and_in_the_guard() {
    let (root, _repository, linked): (PathBuf, PathBuf, PathBuf) = edited_fixture("repeated-rm-sparse", &[("s.txt", b"committed\n")]);
    let file: PathBuf = linked.join("s.txt");
    git(linked.as_path(), &["update-index", "--skip-worktree", "s.txt"]);
    std::fs::write(file.as_path(), b"the hidden edit\n").expect("the edit");
    let kept = git_status(linked.as_path(), &["rm", "--sparse", "--no-sparse", "s.txt"]);
    assert!(!kept.status.success(), "without --sparse Git keeps the skipped file: {kept:?}");
    assert!(file.exists(), "the edit is still there");
    let reached = git_status(linked.as_path(), &["rm", "--no-sparse", "--sparse", "s.txt"]);
    assert!(reached.status.success(), "with --sparse last Git deletes it: {reached:?}");
    assert!(!file.exists(), "the edit is gone");
    let scope_of = |values: &[&str]| -> Option<DirtyScope> {
        return match dirty_question(GuardedCommand::Remove, words(values).as_slice()) {
            Some(crate::rule_linked_worktree::DirtyQuestion::Worktree(scope)) => Some(scope),
            _ => None,
        };
    };
    assert_eq!(scope_of(&["--sparse", "--no-sparse", "s.txt"]), Some(DirtyScope::Tracked));
    assert_eq!(scope_of(&["--no-sparse", "--sparse", "s.txt"]), Some(DirtyScope::TrackedAndSkipWorktreeEdits));
    crate::command_test_support::remove(root.as_path());
}
