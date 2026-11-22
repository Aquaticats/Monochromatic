//! Controls for the object questions a declined in-process reader leaves to Git.
//! Why:
//!  The reader declines a reftable store and a files store holding a replace ref,
//! and each question below then starts Git.
//!  A question that silently answered nothing would take a wrong answer back to the landing,
//! so each is asked of both kinds of declined repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const repo of [reftable, filesWithReplace]) { expect(commitBytes(repo, older)).toBe(gitCatFile(repo, older)); }
//! ```

use super::*;
use crate::recording_git::{recorded_starts, recording_context};
use crate::test_support::{FIXTURE_GIT_BOUND, REAL_GIT, fixture, git, git_output, git_text, real_git_within, remove};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// A repository whose object reader declines,
/// with two commits:
/// a reftable one,
/// or a files one holding a replace ref.
fn a_declined_repository(root: &Path, kind: &str) -> PathBuf {
    let repo: PathBuf = a_plain_repository(root, kind, kind == "reftable");
    // The reftable backend keeps the replace ref in its tables and the files backend in `refs/replace`,
    // so one command makes both kinds a store the reader declines.
    let older: String = git_text(repo.as_path(), &["rev-parse", "HEAD~1"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["replace", older.as_str(), head.as_str()]);
    return repo;
}

/// A repository the reader opens,
/// with two commits,
/// unless its ref store is the reftable one.
fn a_plain_repository(root: &Path, name: &str, reftable: bool) -> PathBuf {
    let repo: PathBuf = root.join(name);
    std::fs::create_dir(&repo).expect("dir");
    if reftable {
        git(repo.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
    } else {
        git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    }
    std::fs::write(repo.join("a.txt"), "one\n").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    std::fs::write(repo.join("a.txt"), "two\n").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=second"]);
    return repo;
}

/// The two kinds of declined repository,
/// built under one fixture.
fn both_kinds(root: &Path) -> Vec<(&'static str, PathBuf)> {
    return vec![
        ("reftable", a_declined_repository(root, "reftable")),
        ("files with a replace ref", a_declined_repository(root, "files")),
    ];
}

/// The commit names of a repository's last two commits,
/// the older first.
fn last_two(repo: &Path) -> (String, String) {
    return (git_text(repo, &["rev-parse", "HEAD~1"]), git_text(repo, &["rev-parse", "HEAD"]));
}

/// One real Git run with input,
/// in the hermetic fixture environment.
fn git_with_input(repo: &Path, arguments: &[&str], input: &str) -> Vec<u8> {
    let environment: Vec<(OsString, OsString)> = fixture_environment(repo);
    let words: Vec<OsString> = arguments.iter().map(|argument| { return OsString::from(argument); }).collect();
    let answer = real_git_within(Path::new(REAL_GIT), Some(repo), environment.as_slice(), words.as_slice(), Some(input.as_bytes()), FIXTURE_GIT_BOUND);
    assert!(answer.output.status.success(), "git {arguments:?} failed: {}", String::from_utf8_lossy(answer.output.stderr.as_slice()));
    return answer.output.stdout;
}

/// The environment a fixture Git runs with.
fn fixture_environment(repo: &Path) -> Vec<(OsString, OsString)> {
    return vec![
        (OsString::from("PATH"), OsString::from("/usr/bin:/bin")),
        (OsString::from("GIT_CONFIG_NOSYSTEM"), OsString::from("1")),
        (OsString::from("GIT_CONFIG_GLOBAL"), OsString::from("/nonexistent-global-config")),
        (OsString::from("HOME"), repo.join("no-home").into_os_string()),
    ];
}

/// Drop a loose object the fixture Git wrote.
fn remove_loose_object(repo: &Path, name: &str) {
    let path: PathBuf = repo.join(".git").join("objects").join(&name[..2]).join(&name[2..]);
    std::fs::remove_file(path.as_path()).unwrap_or_else(|error| panic!("remove {}: {error}", path.display()));
}

/// A tree of the fixture's object store with one blob below a subtree.
fn a_tree_with_one_blob(repo: &Path) -> (String, String) {
    let blob: String = String::from_utf8(git_with_input(repo, &["hash-object", "-w", "--stdin"], "gone\n")).expect("UTF-8");
    let blob: String = String::from(blob.trim());
    let subtree_line: String = format!("100644 blob {blob}\tsub.txt\n");
    let subtree: String = String::from_utf8(git_with_input(repo, &["mktree"], subtree_line.as_str())).expect("UTF-8");
    let subtree: String = String::from(subtree.trim());
    let root_line: String = format!("040000 tree {subtree}\tsub\n");
    let root: String = String::from_utf8(git_with_input(repo, &["mktree"], root_line.as_str())).expect("UTF-8");
    return (String::from(root.trim()), blob);
}

/// `commit_bytes` prints the commit Git reads in a declined repository,
/// the replacement a replace ref names included,
/// and reads nothing for a name Git has no commit of.
#[test]
fn commit_bytes_answers_with_the_commit_git_reads() {
    let root: PathBuf = fixture("object-questions-commit-bytes");
    for (kind, repo) in both_kinds(root.as_path()) {
        let (older, _head) = last_two(repo.as_path());
        let expected: Vec<u8> = git_output(repo.as_path(), &["cat-file", "commit", older.as_str()]).stdout;
        assert_eq!(commit_bytes(&crate::test_support::git_context(), repo.as_path(), older.as_str()), Some(expected), "{kind}: the bytes Git prints");
        // Positive control: the fixture's replacement makes Git's answer differ from the object the files hold.
        let unreplaced: Vec<u8> = git_output(repo.as_path(), &["--no-replace-objects", "cat-file", "commit", older.as_str()]).stdout;
        assert_ne!(git_output(repo.as_path(), &["cat-file", "commit", older.as_str()]).stdout, unreplaced, "{kind}: the fixture replaces the older commit");
        let absent: String = "1".repeat(older.len());
        assert_eq!(commit_bytes(&crate::test_support::git_context(), repo.as_path(), absent.as_str()), None, "{kind}: a name Git has no commit of");
    }
    remove(root.as_path());
}

/// `objects_missing` sees a blob a pruned subtree names in a declined repository:
/// the subtree is present,
/// the blob below it is gone,
/// so the whole tree is not complete.
#[test]
fn objects_missing_sees_a_blob_a_subtree_names() {
    let root: PathBuf = fixture("object-questions-objects-missing");
    for (kind, repo) in both_kinds(root.as_path()) {
        let common_dir: PathBuf = repo.join(".git");
        let (tree, blob) = a_tree_with_one_blob(repo.as_path());
        let context = crate::test_support::git_context();
        assert!(!objects_missing(&context, repo.as_path(), common_dir.as_path(), &[tree.as_str()]), "{kind}: the tree is complete");
        remove_loose_object(repo.as_path(), blob.as_str());
        // Positive control: Git itself no longer finds the blob the subtree names.
        assert_eq!(git_output(repo.as_path(), &["cat-file", "-e", blob.as_str()]).status.code(), Some(1), "{kind}: the blob is gone");
        assert!(objects_missing(&context, repo.as_path(), common_dir.as_path(), &[tree.as_str()]), "{kind}: the subtree names a gone blob");
    }
    remove(root.as_path());
}

/// A request larger than one command line still reaches Git,
/// because the names travel on standard input.
/// Why:
///  One name on the command line costs its own bytes plus the terminator,
/// so the count is derived from the host's own limit rather than a guessed number.
#[test]
fn a_request_too_large_for_one_command_line_reaches_git() {
    let root: PathBuf = fixture("object-questions-large-request");
    let repo: PathBuf = a_declined_repository(root.as_path(), "files");
    let common_dir: PathBuf = repo.join(".git");
    let context = crate::test_support::git_context();
    // SAFETY: `sysconf` only reads the process's argument limit and touches no memory of ours.
    let limit: i64 = unsafe { libc::sysconf(libc::_SC_ARG_MAX) };
    let limit: usize = usize::try_from(limit).expect("the host answers a positive argument limit");
    let name: usize = 40;
    let count: usize = limit / (name + 1) + 1;
    let names: Vec<String> = (1..=count).map(|index| { return format!("{index:0name$}", name = name); }).collect();
    let bytes: usize = count * (name + 1);
    assert!(bytes > limit, "the request's names would not fit one command line: {bytes} bytes against {limit}");
    let borrowed: Vec<&str> = names.iter().map(String::as_str).collect();
    assert!(objects_missing(&context, repo.as_path(), common_dir.as_path(), borrowed.as_slice()), "a request this large still reaches Git");
    remove(root.as_path());
}

/// `object_missing` sees a commit Git built once its object is gone,
/// and a present one is not missing.
#[test]
fn object_missing_sees_a_gone_commit_git_built() {
    let root: PathBuf = fixture("object-questions-object-missing");
    for (kind, repo) in both_kinds(root.as_path()) {
        let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
        let built: String = git_text(repo.as_path(), &["commit-tree", tree.as_str(), "-m", "built"]);
        let context = crate::test_support::git_context();
        assert!(!object_missing(&context, repo.as_path(), built.as_str()), "{kind}: the commit is present");
        remove_loose_object(repo.as_path(), built.as_str());
        assert_eq!(git_output(repo.as_path(), &["cat-file", "-e", built.as_str()]).status.code(), Some(1), "{kind}: the commit is gone");
        assert!(object_missing(&context, repo.as_path(), built.as_str()), "{kind}: the gone commit is missing");
    }
    remove(root.as_path());
}

/// `built_commits_missing` asks each commit of a request:
/// one gone commit among present ones is missing,
/// present ones are not,
/// and no commit is not.
#[test]
fn built_commits_missing_asks_every_commit() {
    let root: PathBuf = fixture("object-questions-built-commits");
    for (kind, repo) in both_kinds(root.as_path()) {
        let (_older, head) = last_two(repo.as_path());
        let absent: String = "2".repeat(head.len());
        let context = crate::test_support::git_context();
        assert!(!built_commits_missing(&context, repo.as_path(), &[head.as_str()]), "{kind}: the commit is present");
        assert!(built_commits_missing(&context, repo.as_path(), &[head.as_str(), absent.as_str()]), "{kind}: the second commit is gone");
        assert!(!built_commits_missing(&context, repo.as_path(), &[]), "{kind}: no commit is asked");
    }
    remove(root.as_path());
}

/// A landing asks Git once for a commit it reads twice,
/// and the answer is still the bytes Git prints.
#[test]
fn a_landing_reads_a_commit_once() {
    let root: PathBuf = fixture("object-questions-read-once");
    let repo: PathBuf = a_declined_repository(root.as_path(), "reftable");
    let (_older, head) = last_two(repo.as_path());
    let (context, log) = recording_context(root.as_path());
    let answer: Vec<u8> = crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let first: Vec<u8> = commit_bytes(&context, repo.as_path(), head.as_str()).expect("read");
        let second: Vec<u8> = commit_bytes(&context, repo.as_path(), head.as_str()).expect("read");
        assert_eq!(first, second, "the second read answers the first");
        return first;
    });
    assert_eq!(answer, git_output(repo.as_path(), &["cat-file", "commit", head.as_str()]).stdout, "the bytes Git prints");
    let starts: Vec<String> = recorded_starts(log.as_path());
    assert_eq!(starts.len(), 1, "one Git start for one commit read twice: {starts:?}");
    assert!(starts[0].contains("cat-file commit"), "the start is the commit question: {starts:?}");
    remove(root.as_path());
}
