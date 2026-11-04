//! Differential controls for `diff-index --cached --quiet`: the exit status equals real Git's in every state, and the states
//! whose answer depends on `.gitmodules` are declined.

use crate::repo_read_query::tests::{answered, declined, mine};
use crate::test_support::{fixture, git, git_output, remove};
use std::path::{Path, PathBuf};

/// The command lines the wrapper issues: with `HEAD` and a trailing `--`, with a commit name and no `--`, and with a path.
fn lines(commit: &str) -> Vec<Vec<String>> {
    return vec![
        ["diff-index", "--quiet", "--cached", "HEAD", "--"].map(String::from).to_vec(),
        ["diff-index", "--cached", "--quiet", commit].map(String::from).to_vec(),
        ["diff-index", "--cached", "--quiet", "HEAD", "--", "a"].map(String::from).to_vec(),
        ["diff-index", "--cached", "--quiet", "HEAD", "--", "dir"].map(String::from).to_vec(),
        ["diff-index", "--cached", "--quiet", "HEAD", "--", "untouched.txt"].map(String::from).to_vec(),
    ];
}

/// A repository with one commit of a file, a directory, a link and an executable.
fn committed(root: &Path, name: &str) -> PathBuf {
    let repo: PathBuf = root.join(name);
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::create_dir_all(repo.join("dir")).expect("dirs");
    for (path, text) in [("a", "one\n"), ("dir/b", "bee\n"), ("untouched.txt", "same\n")] {
        std::fs::write(repo.join(path), text).expect("write");
    }
    std::os::unix::fs::symlink("a", repo.join("link")).expect("link");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    return repo;
}

/// Every command line answers as Git does; returns Git's exit code for the first line.
fn all_answer(root: &Path, repo: &Path) -> i32 {
    let commit: String = String::from_utf8(git_output(repo, &["rev-parse", "HEAD"]).stdout).expect("UTF-8").trim_end().to_string();
    for line in lines(commit.as_str()) {
        let words: Vec<&str> = line.iter().map(String::as_str).collect();
        answered(root, repo, words.as_slice());
    }
    return git_output(repo, &["diff-index", "--quiet", "--cached", "HEAD", "--"]).status.code().expect("exit code");
}

/// A clean index is 0, and each plain kind of staged change is 1.
#[test]
fn plain_states_equal_git() {
    let root: PathBuf = fixture("quiet-plain");
    let repo: PathBuf = committed(root.as_path(), "repo");
    assert_eq!(all_answer(root.as_path(), repo.as_path()), 0, "clean");
    std::fs::write(repo.join("a"), "two\n").expect("edit");
    git(repo.as_path(), &["add", "a"]);
    assert_eq!(all_answer(root.as_path(), repo.as_path()), 1, "changed content");
    git(repo.as_path(), &["reset", "--quiet", "--hard", "HEAD"]);
    std::fs::write(repo.join("added"), "new\n").expect("add");
    git(repo.as_path(), &["add", "added"]);
    assert_eq!(all_answer(root.as_path(), repo.as_path()), 1, "added");
    git(repo.as_path(), &["rm", "--quiet", "--cached", "added"]);
    git(repo.as_path(), &["rm", "--quiet", "--cached", "dir/b"]);
    assert_eq!(all_answer(root.as_path(), repo.as_path()), 1, "removed");
    git(repo.as_path(), &["reset", "--quiet", "--hard", "HEAD"]);
    git(repo.as_path(), &["update-index", "--chmod=+x", "a"]);
    assert_eq!(all_answer(root.as_path(), repo.as_path()), 1, "mode");
    git(repo.as_path(), &["reset", "--quiet", "--hard", "HEAD"]);
    std::fs::remove_file(repo.join("link")).expect("rm");
    std::fs::write(repo.join("link"), "a file now\n").expect("file");
    git(repo.as_path(), &["add", "link"]);
    assert_eq!(all_answer(root.as_path(), repo.as_path()), 1, "type change");
    remove(root.as_path());
}

/// A path outside the pathspec does not count, and an unstaged edit is not the index's difference.
#[test]
fn a_pathspec_limits_the_comparison() {
    let root: PathBuf = fixture("quiet-pathspec");
    let repo: PathBuf = committed(root.as_path(), "repo");
    std::fs::write(repo.join("dir/b"), "changed\n").expect("edit");
    git(repo.as_path(), &["add", "dir/b"]);
    std::fs::write(repo.join("a"), "unstaged\n").expect("unstaged");
    assert_eq!(all_answer(root.as_path(), repo.as_path()), 1);
    let only_a = mine(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--quiet", "HEAD", "--", "a"]).expect("answered");
    assert_eq!(only_a.code, 0);
    remove(root.as_path());
}

/// An unmerged path is a difference, also when the tree has the path and also under a pathspec that names it, and it does not
/// count under a pathspec that does not.
#[test]
fn unmerged_paths_equal_git() {
    let root: PathBuf = fixture("quiet-unmerged");
    let repo: PathBuf = committed(root.as_path(), "repo");
    let blob = |text: &str| -> String {
        let mut child = std::process::Command::new(crate::test_support::REAL_GIT).current_dir(&repo).args(["hash-object", "-w", "--stdin"]).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().expect("start");
        std::io::Write::write_all(&mut child.stdin.take().expect("stdin"), text.as_bytes()).expect("write");
        return String::from_utf8(child.wait_with_output().expect("output").stdout).expect("UTF-8").trim_end().to_string();
    };
    let lines_in: String = format!("100644 {} 1\tuntouched.txt\n100644 {} 2\tuntouched.txt\n100644 {} 3\tuntouched.txt\n", blob("1"), blob("2"), blob("3"));
    let mut child = std::process::Command::new(crate::test_support::REAL_GIT).current_dir(&repo).args(["update-index", "--index-info"]).stdin(std::process::Stdio::piped()).spawn().expect("start");
    std::io::Write::write_all(&mut child.stdin.take().expect("stdin"), lines_in.as_bytes()).expect("write");
    assert!(child.wait().expect("wait").success());
    assert_eq!(all_answer_for(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--quiet", "HEAD", "--"]), 1);
    assert_eq!(all_answer_for(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--quiet", "HEAD", "--", "untouched.txt"]), 1);
    assert_eq!(all_answer_for(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--quiet", "HEAD", "--", "a"]), 0);
    remove(root.as_path());
}

/// The answer to one command line equals Git's; returns the exit code.
fn all_answer_for(root: &Path, repo: &Path, line: &[&str]) -> i32 {
    answered(root, repo, line);
    return git_output(repo, line).status.code().expect("exit code");
}

/// An intent-to-add entry is an ordinary entry for `diff-index`.
#[test]
fn an_intent_to_add_entry_equals_git() {
    let root: PathBuf = fixture("quiet-ita");
    let repo: PathBuf = committed(root.as_path(), "repo");
    std::fs::write(repo.join("later"), "later\n").expect("write");
    git(repo.as_path(), &["add", "--intent-to-add", "later"]);
    assert_eq!(all_answer(root.as_path(), repo.as_path()), 1, "Git reports the entry as added");
    remove(root.as_path());
}

/// A submodule entry: the same commit is no difference; another commit is one when no `.gitmodules` exists anywhere, and the
/// readers leave it to Git when one does (it could say `ignore = all`).
#[test]
fn submodule_entries_equal_git_and_gitmodules_is_left_to_git() {
    let root: PathBuf = fixture("quiet-gitlink");
    let repo: PathBuf = committed(root.as_path(), "repo");
    let one: String = "1".repeat(40);
    let two: String = "2".repeat(40);
    git(repo.as_path(), &["update-index", "--add", "--cacheinfo", format!("160000,{one},sub").as_str()]);
    git(repo.as_path(), &["commit", "--quiet", "--message=with a submodule"]);
    assert_eq!(all_answer(root.as_path(), repo.as_path()), 0, "same submodule commit");
    git(repo.as_path(), &["update-index", "--cacheinfo", format!("160000,{two},sub").as_str()]);
    assert_eq!(all_answer(root.as_path(), repo.as_path()), 1, "moved submodule, no .gitmodules");
    git(repo.as_path(), &["update-index", "--force-remove", "sub"]);
    assert_eq!(all_answer(root.as_path(), repo.as_path()), 1, "removed submodule");
    std::fs::write(repo.join(".gitmodules"), "[submodule \"sub\"]\n\tpath = sub\n\turl = ./elsewhere\n\tignore = all\n").expect("gitmodules");
    for line in lines("HEAD") {
        let words: Vec<&str> = line.iter().map(String::as_str).collect();
        if words.len() == 5 && words[4] == "--" {
            declined(root.as_path(), repo.as_path(), words.as_slice());
        }
    }
    assert_eq!(git_output(repo.as_path(), &["diff-index", "--quiet", "--cached", "HEAD", "--"]).status.code(), Some(0), "Git hides it, which is why the readers decline");
    remove(root.as_path());
}

/// An unborn branch, a detached head and a head named by a tag-less hexadecimal name are what Git says or the readers decline.
#[test]
fn unborn_and_detached_heads_equal_git_or_are_declined() {
    let root: PathBuf = fixture("quiet-heads");
    let repo: PathBuf = root.join("unborn");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    declined(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--quiet", "HEAD", "--"]);
    let committed_repo: PathBuf = committed(root.as_path(), "repo");
    git(committed_repo.as_path(), &["checkout", "--quiet", "--detach"]);
    assert_eq!(all_answer(root.as_path(), committed_repo.as_path()), 0);
    remove(root.as_path());
}
