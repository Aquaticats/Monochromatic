//! Controls for the ignored-path input refusal:
//! which named elements it names,
//! which it leaves to the commit,
//! what the message offers,
//! and what real Git does with the same command line.

use super::*;
use crate::commit_index_selection::IndexSelection;
use crate::test_support::{fixture, git, git_context, git_output, git_text, remove, repository};
use crate::transaction_journal::TransactionMode;
use std::path::{Path, PathBuf};

/// The most bytes a pathspec file may hold and still be read as elements;
/// a file above it is passed to Git as a file (`ITEMS_BYTES_LIMIT` in `commit_selection_staging.rs`).
const PATHS_ABOVE_THE_READER_BOUND: usize = 64 * 1024;

/// A repository whose `*.js` files are ignored,
/// with `lib/_a.js` force-staged as an addition and `lib/_c.js` left untracked,
/// as in issue 6278,
/// and its base revision.
fn ignored_repository(name: &str) -> (PathBuf, PathBuf, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".gitignore"), "*.js\n").expect("ignore file");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", ".gitignore", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::create_dir(repo.join("lib")).expect("lib");
    std::fs::write(repo.join("lib").join("_a.js"), "a1\n").expect("_a");
    git(repo.as_path(), &["add", "--force", "lib/_a.js"]);
    std::fs::write(repo.join("lib").join("_c.js"), "c1\n").expect("_c");
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    return (root, repo, base);
}

/// The selection one explicit-path commit names,
/// from the command line or from a pathspec file.
fn selection<'selection>(pathspecs: &'selection [OsString], file: Option<&'selection Path>, base: &'selection str) -> IndexSelection<'selection> {
    return IndexSelection {
        mode: TransactionMode::ExplicitPath,
        pathspecs,
        pathspec_file: file,
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: base,
    };
}

/// The named elements an ignore rule refuses,
/// as text.
fn refused(repo: &Path, base: &str, pathspecs: &[&str], file: Option<&Path>) -> Vec<String> {
    let specs: Vec<OsString> = pathspecs.iter().map(OsString::from).collect();
    let choice: IndexSelection<'_> = selection(specs.as_slice(), file, base);
    return named_ignored_files(&git_context(), repo, repo.join(".git").join("index").as_path(), &choice)
        .expect("the question is answered")
        .iter()
        .map(|path| { return String::from_utf8_lossy(path).into_owned(); })
        .collect();
}

/// A file the worktree holds,
/// that no index entry tracks and that an ignore rule matches,
/// is the refusal the commit stops with.
#[test]
fn a_named_file_an_ignore_rule_matches_is_refused() {
    let (root, repo, base) = ignored_repository("named-ignored-refused");
    assert_eq!(refused(repo.as_path(), base.as_str(), &["lib/_c.js"], None), vec!["lib/_c.js".to_owned()]);
    remove(root.as_path());
}

/// Every named file an ignore rule matches is named,
/// not only the first.
#[test]
fn every_named_file_an_ignore_rule_matches_is_named() {
    let (root, repo, base) = ignored_repository("named-ignored-every");
    std::fs::write(repo.join("other.js"), "o1\n").expect("other");
    assert_eq!(
        refused(repo.as_path(), base.as_str(), &["lib/_c.js", "a.txt", "other.js"], None),
        vec!["lib/_c.js".to_owned(), "other.js".to_owned()]
    );
    remove(root.as_path());
}

/// A file an ignore rule matches that the real index tracks is committed,
/// not refused (issue 1807):
/// the private index carries that entry before anything is staged from the worktree.
#[test]
fn a_named_file_the_index_tracks_is_not_refused() {
    let (root, repo, base) = ignored_repository("named-ignored-tracked");
    assert_eq!(refused(repo.as_path(), base.as_str(), &["lib/_a.js"], None), Vec::<String>::new());
    remove(root.as_path());
}

/// A directory and a pattern name no single file,
/// so they are staged as the tracked paths they match and never refused.
#[test]
fn a_directory_and_a_pattern_are_not_refused() {
    let (root, repo, base) = ignored_repository("named-ignored-patterns");
    assert_eq!(refused(repo.as_path(), base.as_str(), &["lib"], None), Vec::<String>::new());
    assert_eq!(refused(repo.as_path(), base.as_str(), &["*.js"], None), Vec::<String>::new());
    assert_eq!(refused(repo.as_path(), base.as_str(), &[":(top)lib/_c.js"], None), Vec::<String>::new());
    remove(root.as_path());
}

/// A file nothing ignores is a file nothing staged,
/// which the commit takes as native `git commit` does.
#[test]
fn a_named_file_no_ignore_rule_matches_is_not_refused() {
    let (root, repo, base) = ignored_repository("named-ignored-plain");
    std::fs::write(repo.join("plain.txt"), "p\n").expect("plain");
    assert_eq!(refused(repo.as_path(), base.as_str(), &["plain.txt"], None), Vec::<String>::new());
    remove(root.as_path());
}

/// A path a glob mode makes ordinary text names that file,
/// so the mode decides the refusal exactly as it decides the staging.
///  The mode reaches the wrapper as a global option and as a variable,
/// and the question is asked without either of them,
/// because Git refuses its own pathspec modes for `check-ignore`.
#[test]
fn a_literal_glob_character_names_the_file_of_that_name() {
    let (root, repo, base) = ignored_repository("named-ignored-literal");
    std::fs::write(repo.join("st*r.js"), "s\n").expect("star");
    let literal: GitContext = GitContext {
        global_prefix: vec![OsString::from("--literal-pathspecs")],
        ..git_context()
    };
    let literal_variable: GitContext = GitContext {
        overlay: [git_context().overlay.as_slice(), &[(OsString::from("GIT_LITERAL_PATHSPECS"), OsString::from("1"))]].concat(),
        ..git_context()
    };
    let specs: Vec<OsString> = ["st*r.js"].iter().map(OsString::from).collect();
    let choice: IndexSelection<'_> = selection(specs.as_slice(), None, base.as_str());
    for context in [&literal, &literal_variable] {
        assert_eq!(
            named_ignored_files(context, repo.as_path(), repo.join(".git").join("index").as_path(), &choice)
                .expect("the question is answered"),
            vec![b"st*r.js".to_vec()]
        );
    }
    remove(root.as_path());
}

/// A pathspec file that names an ignored file is refused the same way,
/// because its elements are the selection a command line carries.
#[test]
fn a_pathspec_file_naming_an_ignored_file_is_refused() {
    let (root, repo, base) = ignored_repository("named-ignored-file-list");
    let list: PathBuf = root.join("list");
    std::fs::write(&list, "lib/_c.js\n").expect("list");
    assert_eq!(
        refused(repo.as_path(), base.as_str(), &[], Some(list.as_path())),
        vec!["lib/_c.js".to_owned()]
    );
    remove(root.as_path());
}

/// A selection that names no element,
/// one read as a file,
/// and one this reader cannot take,
/// all stage nothing by a name,
/// so none starts the question.
#[test]
fn a_selection_that_names_no_readable_element_asks_nothing() {
    let (root, repo, base) = ignored_repository("named-ignored-no-elements");
    assert_eq!(refused(repo.as_path(), base.as_str(), &[], None), Vec::<String>::new());
    let above: PathBuf = root.join("above");
    let mut paths: String = String::new();
    while paths.len() <= PATHS_ABOVE_THE_READER_BOUND {
        paths.push_str("lib/_c.js\n");
    }
    std::fs::write(&above, paths.as_str()).expect("above");
    assert_eq!(refused(repo.as_path(), base.as_str(), &[], Some(above.as_path())), Vec::<String>::new());
    let absent: PathBuf = root.join("absent");
    assert_eq!(refused(repo.as_path(), base.as_str(), &[], Some(absent.as_path())), Vec::<String>::new());
    remove(root.as_path());
}

/// The question asked of Git names the real index,
/// the caller's repository and none of the caller's pathspec modes,
/// and asks about the names on standard input.
/// Why:
///  The index and the object directory are both `Option<&Path>`,
/// so passing one where the other belongs compiles and answers about the wrong thing:
///  a question asked with no index would refuse a force-added ignored file the commit can take,
/// and one asked without the caller's repository selection would answer for the directory's own repository
///  instead of the one the command line named.
#[test]
fn the_question_names_the_real_index_and_the_callers_repository() {
    let context: GitContext = GitContext {
        global_prefix: [
            OsString::from("--git-dir"),
            OsString::from("/elsewhere/config.git"),
            OsString::from("--work-tree"),
            OsString::from("/elsewhere/work"),
        ]
        .to_vec(),
        ..git_context()
    };
    let request: GitRequest = refused_paths_request(&context, Path::new("/here"), Path::new("/repo/.git/index"), b"lib/_c.js\0");
    assert_eq!(
        request.arguments,
        ["check-ignore", "-z", "--stdin"].iter().map(OsString::from).collect::<Vec<OsString>>(),
        "the names come on standard input, so no name is a path argument"
    );
    assert_eq!(request.index, Some(PathBuf::from("/repo/.git/index")), "the index is the one whose tracked paths count");
    assert_eq!(request.object_directory, None, "this command writes no object");
    assert!(request.without_prefix, "the caller's other options do not reach this start");
    assert!(request.own_pathspecs, "Git refuses a pathspec mode for this command");
    assert_eq!(
        request.repository_selection,
        [
            OsString::from("--git-dir"),
            OsString::from("/elsewhere/config.git"),
            OsString::from("--work-tree"),
            OsString::from("/elsewhere/work"),
        ]
        .to_vec(),
        "the question is asked of the repository the command line names"
    );
    assert_eq!(request.input, Some(b"lib/_c.js\0".to_vec()), "one NUL-terminated name per path");
}

/// The message names every rejected path,
/// says the commit was not made and nothing was staged by it,
/// and gives both ways forward.
#[test]
fn the_message_names_the_paths_and_both_remedies() {
    let message: String = named_ignored_message(&[b"lib/_c.js".to_vec(), b"other.js".to_vec()]);
    assert!(message.contains(r#"["lib/_c.js","other.js"]"#), "{message}");
    assert!(message.contains("this commit was not made and nothing was staged by it"), "{message}");
    assert!(message.contains("keep it untracked"), "{message}");
    assert!(message.contains("git add --force -- <path>"), "{message}");
    assert!(message.contains("is meant to become tracked"), "{message}");
}

/// Real Git 2.56.0 on the same command line,
/// and the `git add` the wrapper stages with,
/// measured with a disposable repository.
/// Why:
///  Native `git commit -- <path>` takes only paths Git already knows,
/// so an untracked path is refused as an unknown pathspec;
///  the ignore hint the wrapper used to report is `git add`'s,
/// which is the step an explicit-path commit runs to take a file nothing staged (issue 6278).
fn real_git_on_a_named_ignored_file(repo: &Path, base: &str) -> (i32, String, i32, String) {
    let committed: std::process::Output = git_output(repo, &["commit", "--quiet", "--message=native", "--", "lib/_c.js"]);
    assert_eq!(git_text(repo, &["rev-parse", "HEAD"]), base, "native Git landed nothing");
    let added: std::process::Output = git_output(repo, &["add", "--", "lib/_c.js"]);
    return (
        committed.status.code().unwrap_or(-1),
        String::from_utf8_lossy(committed.stderr.as_slice()).into_owned(),
        added.status.code().unwrap_or(-1),
        String::from_utf8_lossy(added.stderr.as_slice()).into_owned(),
    );
}

/// Real Git 2.56.0 refuses an untracked path of `git commit --` as unknown to Git,
/// and refuses the ignored path of `git add` with the force-add hint.
#[test]
fn real_git_refuses_a_commit_of_an_ignored_path() {
    let (root, repo, base) = ignored_repository("named-ignored-native");
    let (commit_status, commit_text, add_status, add_text) = real_git_on_a_named_ignored_file(repo.as_path(), base.as_str());
    assert_eq!(commit_status, 1, "native Git's status: {commit_text}");
    assert!(commit_text.contains("pathspec 'lib/_c.js' did not match any file(s) known to git"), "{commit_text}");
    assert_eq!(add_status, 1, "git add's status: {add_text}");
    assert!(add_text.contains("The following paths are ignored by one of your .gitignore files"), "{add_text}");
    assert!(add_text.contains("Use -f if you really want to add them"), "{add_text}");
    remove(root.as_path());
}
