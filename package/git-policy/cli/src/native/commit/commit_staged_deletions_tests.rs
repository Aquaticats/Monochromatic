//! Controls for the staged-deletion finding:
//! which selected paths it names and what the message offers.

use super::*;
use crate::commit_selection_staging::ITEMS_BYTES_LIMIT;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_git::GitContext;
use std::path::PathBuf;

/// A repository with `a.txt`,
/// `b.txt` and `dir/c.txt` committed,
/// and the real index path.
fn committed_repository(name: &str) -> (PathBuf, PathBuf, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::create_dir(repo.join("dir")).expect("dir");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    std::fs::write(repo.join("b.txt"), "b\n").expect("b");
    std::fs::write(repo.join("dir").join("c.txt"), "c\n").expect("c");
    git(repo.as_path(), &["add", "a.txt", "b.txt", "dir/c.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    return (root, repo, base);
}

/// The selected paths with a staged deletion and a worktree entry,
/// for the given pathspecs.
fn present(repo: &Path, base: &str, pathspecs: &[&str]) -> Vec<String> {
    let specs: Vec<OsString> = pathspecs.iter().map(OsString::from).collect();
    return present_with(&git_context(), repo, base, specs.as_slice());
}

/// The same answer for one pathspec context,
/// so a control can set the modes the caller wrote.
fn present_with(context: &GitContext, repo: &Path, base: &str, specs: &[OsString]) -> Vec<String> {
    return staged_deletions_present_on_disk(
        context,
        repo,
        repo,
        repo.join(".git").join("index").as_path(),
        base,
        specs,
    )
    .expect("the read succeeds")
    .iter()
    .map(|path| { return String::from_utf8_lossy(path).into_owned(); })
    .collect();
}

/// The context of a command that wrote `--literal-pathspecs` as a global option.
fn literal_context() -> GitContext {
    let mut context: GitContext = git_context();
    context.global_prefix.push(OsString::from("--literal-pathspecs"));
    return context;
}

/// A deletion staged with `git rm --cached` while the file stays on disk is named.
#[test]
fn a_staged_deletion_whose_path_stays_on_disk_is_named() {
    let (root, repo, base) = committed_repository("staged-deletion-present");
    git(repo.as_path(), &["rm", "--quiet", "--cached", "--", "a.txt"]);
    assert_eq!(present(repo.as_path(), base.as_str(), &["a.txt"]), vec!["a.txt".to_owned()]);
    remove(root.as_path());
}

/// A deletion whose file is gone from the disk is an ordinary explicit-path commit,
/// so nothing is named.
#[test]
fn a_staged_deletion_whose_path_is_gone_is_not_named() {
    let (root, repo, base) = committed_repository("staged-deletion-gone");
    git(repo.as_path(), &["rm", "--quiet", "--", "a.txt"]);
    assert_eq!(present(repo.as_path(), base.as_str(), &["a.txt"]), Vec::<String>::new());
    remove(root.as_path());
}

/// Only the selected paths are examined:
/// another path's staged deletion is not this commit's concern.
#[test]
fn a_staged_deletion_outside_the_pathspecs_is_not_named() {
    let (root, repo, base) = committed_repository("staged-deletion-unselected");
    git(repo.as_path(), &["rm", "--quiet", "--cached", "--", "a.txt"]);
    assert_eq!(present(repo.as_path(), base.as_str(), &["b.txt"]), Vec::<String>::new());
    assert_eq!(present(repo.as_path(), base.as_str(), &["dir"]), Vec::<String>::new());
    remove(root.as_path());
}

/// A directory pathspec selects the deletions below it,
/// and a deletion that was ignored since is still named.
#[test]
fn a_directory_pathspec_names_the_deletions_below_it_even_when_ignored() {
    let (root, repo, base) = committed_repository("staged-deletion-ignored-parent");
    std::fs::write(repo.join(".gitignore"), "/dir/\n").expect("ignore");
    git(repo.as_path(), &["rm", "--quiet", "--cached", "--", "dir/c.txt"]);
    assert_eq!(present(repo.as_path(), base.as_str(), &["dir/c.txt"]), vec!["dir/c.txt".to_owned()]);
    assert_eq!(present(repo.as_path(), base.as_str(), &["dir"]), vec!["dir/c.txt".to_owned()]);
    remove(root.as_path());
}

/// No pathspec means no explicit-path selection,
/// so no read is made and nothing is named.
#[test]
fn no_pathspecs_name_nothing() {
    let (root, repo, base) = committed_repository("staged-deletion-no-pathspec");
    git(repo.as_path(), &["rm", "--quiet", "--cached", "--", "a.txt"]);
    assert_eq!(present(repo.as_path(), base.as_str(), &[]), Vec::<String>::new());
    remove(root.as_path());
}

/// A modification,
/// which is not a deletion,
/// is never named.
#[test]
fn a_staged_modification_is_not_named() {
    let (root, repo, base) = committed_repository("staged-deletion-modification");
    std::fs::write(repo.join("a.txt"), "changed\n").expect("edit");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    assert_eq!(present(repo.as_path(), base.as_str(), &["a.txt"]), Vec::<String>::new());
    remove(root.as_path());
}

/// A literal pathspec names one file and never a glob,
/// so the staged deletion of another file is not the caller's selection:
/// `--literal-pathspecs` makes `literal*.txt` the file of that name.
#[test]
fn a_literal_pathspec_does_not_name_the_deletion_of_a_glob_match() {
    let root: PathBuf = fixture("staged-deletion-literal");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("literal*.txt"), "star\n").expect("star");
    std::fs::write(repo.join("literal-other.txt"), "other\n").expect("other");
    git(repo.as_path(), &["add", "literal*.txt", "literal-other.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["rm", "--quiet", "--cached", "--", "literal-other.txt"]);
    // The glob reading names the other file, which is the answer this guard gave before the mode reached it.
    assert_eq!(present(repo.as_path(), base.as_str(), &["literal*.txt"]), vec!["literal-other.txt".to_owned()]);
    assert_eq!(present_with(&literal_context(), repo.as_path(), base.as_str(), &[OsString::from("literal*.txt")]), Vec::<String>::new());
    remove(root.as_path());
}

/// A selection past one command line is asked in bounded starts,
/// and the path named only in a later start is still named.
#[test]
fn a_selection_past_one_command_line_names_the_deletion_of_its_last_element() {
    let (root, repo, base) = committed_repository("staged-deletion-chunks");
    git(repo.as_path(), &["rm", "--quiet", "--cached", "--", "a.txt"]);
    let mut specs: Vec<OsString> = Vec::new();
    let copies: usize = usize::try_from(ITEMS_BYTES_LIMIT).expect("the bound fits a machine word") / "b.txt".len();
    for _ in 0..copies {
        specs.push(OsString::from("b.txt"));
    }
    specs.push(OsString::from("a.txt"));
    assert_eq!(present_with(&git_context(), repo.as_path(), base.as_str(), specs.as_slice()), vec!["a.txt".to_owned()]);
    remove(root.as_path());
}

/// Every start of a selection past one command line carries the elements that subtract paths,
/// so an exclusion narrows the elements of a later start too.
#[test]
fn an_exclusion_narrows_every_start_of_a_selection_past_one_command_line() {
    let (root, repo, base) = committed_repository("staged-deletion-chunks-excluded");
    git(repo.as_path(), &["rm", "--quiet", "--cached", "--", "a.txt"]);
    let mut specs: Vec<OsString> = Vec::new();
    let copies: usize = usize::try_from(ITEMS_BYTES_LIMIT).expect("the bound fits a machine word") / "b.txt".len();
    for _ in 0..copies {
        specs.push(OsString::from("b.txt"));
    }
    specs.push(OsString::from("a.txt"));
    specs.push(OsString::from(":(exclude)a.txt"));
    assert_eq!(present_with(&git_context(), repo.as_path(), base.as_str(), specs.as_slice()), Vec::<String>::new());
    remove(root.as_path());
}

/// The message lists the paths and offers both intents:
/// commit the removal,
/// or stage the file again.
#[test]
fn the_message_names_the_paths_and_both_ways_forward() {
    let message: String = staged_deletion_message(&[b"a.txt".to_vec(), b"dir/c.txt".to_vec()]);
    assert!(message.contains("[\"a.txt\",\"dir/c.txt\"]"), "{message}");
    assert!(message.contains("--no-only"), "{message}");
    assert!(message.contains("every staged change"), "{message}");
    assert!(message.contains("git add --force"), "{message}");
    assert!(!message.contains("--no-enforce-only"), "{message}");
}
