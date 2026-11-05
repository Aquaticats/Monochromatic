//! Differential controls for the in-process answers:
//!  for each command line the wrapper issues, standard output and exit code equal what real Git prints in the same
//!  repository, in loose and packed object states, index versions 2 to 4, SHA-1 and SHA-256;
//!  and every state Git words differently (unmerged paths, a file that became a directory, a gitlink,
//!  intent-to-add, a possible rename, a decisive configuration key) is declined.

use super::*;
use crate::test_support::{fixture, git, git_output, remove};
use std::os::unix::ffi::OsStringExt;
use std::path::Path;

/// The scope facts that match the hermetic environment the fixtures give Git.
pub(crate) fn scope(root: &Path) -> ScopeEnvironment {
    return ScopeEnvironment { home: Some(root.join("no-home")), no_system: true, global: Some(PathBuf::from("/nonexistent-global-config")), ..ScopeEnvironment::default() };
}

/// What the readers answer for `arguments` run in `repo`.
pub(crate) fn mine(root: &Path, repo: &Path, arguments: &[&str]) -> Option<Answer> {
    let given: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    return answer_with(&Query { arguments: given.as_slice(), directory: Some(repo), variables: &[], input: None }, Vec::new(), &scope(root));
}

/// Require equal output and exit code with Git; returns whether the readers answered.
fn compare(root: &Path, repo: &Path, arguments: &[&str]) -> bool {
    let expected = git_output(repo, arguments);
    let Some(found) = mine(root, repo, arguments) else {
        return false;
    };
    assert_eq!(found.stdout, expected.stdout, "{arguments:?}");
    assert_eq!(found.stderr, expected.stderr, "{arguments:?}");
    assert_eq!(Some(found.code), expected.status.code(), "{arguments:?}");
    return true;
}

/// Require that the readers answer and equal Git.
pub(crate) fn answered(root: &Path, repo: &Path, arguments: &[&str]) {
    assert!(compare(root, repo, arguments), "the readers declined {arguments:?}");
}

/// Require that the readers decline.
pub(crate) fn declined(root: &Path, repo: &Path, arguments: &[&str]) {
    assert!(mine(root, repo, arguments).is_none(), "the readers answered {arguments:?} but must leave it to Git");
}

/// The text of `git <arguments>` without its final newline.
pub(crate) fn text(repo: &Path, arguments: &[&str]) -> String {
    return String::from_utf8(git_output(repo, arguments).stdout).expect("UTF-8").trim_end().to_string();
}

/// A repository with a first commit of varied entries and a staged second state: modified, mode-changed, type-changed,
/// deleted and added paths, including a non-UTF-8 name and a name with a space.
fn staged_repository(root: &Path, name: &str, format: &str) -> PathBuf {
    let repo = root.join(name);
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main", &format!("--object-format={format}")]);
    std::fs::create_dir_all(repo.join("dir/sub")).expect("dirs");
    std::fs::write(repo.join("a.txt"), "one\n").expect("a");
    std::fs::write(repo.join("exec.sh"), "#!/bin/sh\n").expect("exec");
    std::os::unix::fs::symlink("a.txt", repo.join("link")).expect("link");
    std::fs::write(repo.join("dir/b.txt"), "bee\n").expect("b");
    std::fs::write(repo.join("dir/sub/c.txt"), "sea\n").expect("c");
    std::fs::write(repo.join("sp ace.txt"), "space\n").expect("space");
    std::fs::write(repo.join(std::ffi::OsString::from_vec(vec![b'r', 0xff, b'.', b't'])), "raw\n").expect("raw");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    std::fs::write(repo.join("a.txt"), "one\ntwo\n").expect("edit");
    std::fs::set_permissions(repo.join("exec.sh"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("chmod");
    std::fs::remove_file(repo.join("link")).expect("rm link");
    std::fs::write(repo.join("link"), "now a file\n").expect("file");
    std::fs::remove_file(repo.join("dir/b.txt")).expect("rm b");
    std::fs::write(repo.join("dir/sub/new.txt"), "new\n").expect("new");
    std::fs::write(repo.join("z-added.txt"), "z\n").expect("z");
    git(repo.as_path(), &["add", "--all"]);
    return repo;
}

/// Every command line of the shapes the wrapper issues, against the staged repository.
fn all_shapes(root: &Path, repo: &Path) {
    let head = text(repo, &["rev-parse", "HEAD"]);
    let tree = text(repo, &["rev-parse", "HEAD^{tree}"]);
    answered(root, repo, &["ls-files", "--stage", "-z", "--full-name"]);
    answered(root, repo, &["ls-files", "--stage", "-z"]);
    answered(root, repo, &["ls-files", "--unmerged", "-z"]);
    answered(root, repo, &["ls-files", "--stage", "-z", "--full-name", "--", ":(top,literal)a.txt"]);
    answered(root, repo, &["ls-files", "--stage", "-z", "--full-name", "--", ":(top,literal)dir"]);
    answered(root, repo, &["ls-files", "--stage", "-z", "--full-name", "--", "dir/sub", "sp ace.txt"]);
    answered(root, repo, &["--no-literal-pathspecs", "ls-files", "--stage", "-z", "--full-name", "--", ":/"]);
    for object in [head.as_str(), tree.as_str()] {
        answered(root, repo, &["diff-index", "--cached", "--name-only", "-z", "--no-renames", object]);
        answered(root, repo, &["diff-index", "--cached", "--raw", "-z", "--no-renames", "--no-abbrev", object]);
        for filter in ["A", "D", "M", "T", "AD", "MT"] {
            let option = format!("--diff-filter={filter}");
            answered(root, repo, &["diff-index", "--cached", "--name-only", &option, "-z", object]);
            answered(root, repo, &["diff-index", "--cached", "--raw", "--no-abbrev", "--no-renames", "-z", &option, object, "--", "a.txt"]);
        }
        answered(root, repo, &["diff-index", "--cached", "--raw", "-z", "--no-renames", "--no-abbrev", object, "--", ":(top,literal)dir/sub/new.txt"]);
        answered(root, repo, &["diff-index", "--cached", "--raw", "-z", "--no-renames", "--no-abbrev", object, "--", "dir"]);
    }
    answered(root, repo, &["diff-tree", "--root", "--no-commit-id", "-r", "-z", "-m", "--no-renames", "--no-abbrev", &head]);
    answered(root, repo, &["show", "--no-patch", "--format=%s", &head]);
}

/// All shapes agree with Git on a loose, a packed and a version 4 index state, in SHA-1 and SHA-256.
#[test]
fn the_shapes_equal_git_in_every_state() {
    let root = fixture("repo-read-query-shapes");
    for format in ["sha1", "sha256"] {
        let repo = staged_repository(root.as_path(), format, format);
        all_shapes(root.as_path(), repo.as_path());
        git(repo.as_path(), &["repack", "-a", "-d", "-f", "--quiet"]);
        git(repo.as_path(), &["prune-packed"]);
        all_shapes(root.as_path(), repo.as_path());
        git(repo.as_path(), &["update-index", "--index-version", "4"]);
        all_shapes(root.as_path(), repo.as_path());
        git(repo.as_path(), &["update-index", "--index-version", "3"]);
        all_shapes(root.as_path(), repo.as_path());
    }
    remove(root.as_path());
}

/// Subjects of one-line, multi-line, blank-led, tab-led and body-carrying messages, and a root commit's raw diff.
#[test]
fn subjects_and_root_commits_equal_git() {
    let root = fixture("repo-read-query-subjects");
    let repo = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    for (index, message) in ["plain", "two\nline subject\n\nbody text\n", "\n\nleading blanks\n", "  indented  \ttail \t\n", "trailing\n\n\n", "é ünïcode\n"].iter().enumerate() {
        std::fs::write(repo.join("f.txt"), format!("{index}\n")).expect("write");
        git(repo.as_path(), &["add", "--all"]);
        git(repo.as_path(), &["commit", "--quiet", "--allow-empty-message", "--cleanup=verbatim", "--message", message]);
        let head = text(repo.as_path(), &["rev-parse", "HEAD"]);
        answered(root.as_path(), repo.as_path(), &["show", "--no-patch", "--format=%s", &head]);
        answered(root.as_path(), repo.as_path(), &["diff-tree", "--root", "--no-commit-id", "-r", "-z", "-m", "--no-renames", "--no-abbrev", &head]);
    }
    remove(root.as_path());
}

/// Every state whose output Git decides by more than the lists is declined.
#[test]
fn states_that_need_git_are_declined() {
    let root = fixture("repo-read-query-declined");
    // A file that became a directory.
    let repo = staged_repository(root.as_path(), "dir-file", "sha1");
    let head = text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=second"]);
    std::fs::remove_file(repo.join("a.txt")).expect("rm");
    std::fs::create_dir(repo.join("a.txt")).expect("dir");
    std::fs::write(repo.join("a.txt/inside"), "x\n").expect("inside");
    git(repo.as_path(), &["add", "--all"]);
    let second = text(repo.as_path(), &["rev-parse", "HEAD"]);
    declined(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--name-only", "-z", &second]);
    assert!(compare(root.as_path(), repo.as_path(), &["ls-files", "--stage", "-z", "--full-name"]), "the listing is unaffected");
    // A possible rename: the porcelain declines, the plumbing answers.
    let moved = staged_repository(root.as_path(), "rename", "sha1");
    git(moved.as_path(), &["commit", "--quiet", "--message=second"]);
    git(moved.as_path(), &["mv", "sp ace.txt", "spaced.txt"]);
    let tip = text(moved.as_path(), &["rev-parse", "HEAD"]);
    declined(root.as_path(), moved.as_path(), &["diff", "--cached", "--name-only", "-z", &tip]);
    answered(root.as_path(), moved.as_path(), &["diff-index", "--cached", "--name-only", "-z", &tip]);
    // Intent-to-add.
    std::fs::write(moved.join("later.txt"), "later\n").expect("later");
    git(moved.as_path(), &["add", "--intent-to-add", "later.txt"]);
    declined(root.as_path(), moved.as_path(), &["diff-index", "--cached", "--name-only", "-z", &tip]);
    // Unmerged paths.
    let conflict = root.join("conflict");
    std::fs::create_dir(&conflict).expect("dir");
    git(conflict.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(conflict.join("f"), "base\n").expect("f");
    git(conflict.as_path(), &["add", "f"]);
    git(conflict.as_path(), &["commit", "--quiet", "--message=base"]);
    git(conflict.as_path(), &["switch", "--quiet", "--create", "side"]);
    std::fs::write(conflict.join("f"), "side\n").expect("f");
    git(conflict.as_path(), &["commit", "--quiet", "--all", "--message=side"]);
    git(conflict.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(conflict.join("f"), "main\n").expect("f");
    git(conflict.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
    assert!(!git_output(conflict.as_path(), &["merge", "side"]).status.success());
    let conflict_tip = text(conflict.as_path(), &["rev-parse", "HEAD"]);
    answered(root.as_path(), conflict.as_path(), &["ls-files", "--unmerged", "-z"]);
    answered(root.as_path(), conflict.as_path(), &["ls-files", "--stage", "-z", "--full-name"]);
    declined(root.as_path(), conflict.as_path(), &["diff-index", "--cached", "--name-only", "-z", &conflict_tip]);
    // A merge commit and a gitlink.
    git(conflict.as_path(), &["checkout", "--quiet", "--theirs", "f"]);
    git(conflict.as_path(), &["add", "f"]);
    git(conflict.as_path(), &["commit", "--quiet", "--message=merged"]);
    let merge = text(conflict.as_path(), &["rev-parse", "HEAD"]);
    declined(root.as_path(), conflict.as_path(), &["diff-tree", "--root", "--no-commit-id", "-r", "-z", "-m", "--no-renames", "--no-abbrev", &merge]);
    let linked = staged_repository(root.as_path(), "gitlink", "sha1");
    git(linked.as_path(), &["commit", "--quiet", "--message=second"]);
    let inner = text(linked.as_path(), &["rev-parse", "HEAD"]);
    git(linked.as_path(), &["update-index", "--add", "--cacheinfo", &format!("160000,{inner},module")]);
    let linked_tip = text(linked.as_path(), &["rev-parse", "HEAD"]);
    declined(root.as_path(), linked.as_path(), &["diff-index", "--cached", "--name-only", "-z", &linked_tip]);
    // A decisive configuration key.
    git(repo.as_path(), &["config", "diff.orderFile", "order.txt"]);
    declined(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--name-only", "-z", &head]);
    // An unknown command line, a subdirectory start for a path-relative listing, and an object name that is not hexadecimal.
    let clean = staged_repository(root.as_path(), "unknown-shapes", "sha1");
    declined(root.as_path(), clean.as_path(), &["ls-files", "--stage", "-z", "--others"]);
    declined(root.as_path(), clean.as_path(), &["diff-index", "--cached", "--name-only", "-z", "HEAD"]);
    declined(root.as_path(), clean.as_path(), &["diff-index", "--cached", "--name-only", "--diff-filter=d", "-z", &head]);
    declined(root.as_path(), clean.as_path(), &["ls-files", "--stage", "-z", "--", "*.txt"]);
    // A later exclusion after `:/`, a repeated `--diff-filter`, and a first `-C` that fails before a valid second one.
    declined(root.as_path(), clean.as_path(), &["ls-files", "--stage", "-z", "--", ":/", ":(top,exclude)a.txt"]);
    let clean_tip = text(clean.as_path(), &["rev-parse", "HEAD"]);
    declined(root.as_path(), clean.as_path(), &["diff-index", "--cached", "--name-only", "-z", "--diff-filter=A", "--diff-filter=D", &clean_tip]);
    let nowhere = root.join("no-such-directory");
    let real = clean.to_str().expect("UTF-8");
    declined(root.as_path(), root.as_path(), &["-C", nowhere.to_str().expect("UTF-8"), "-C", real, "ls-files", "--stage", "-z"]);
    answered(root.as_path(), root.as_path(), &["-C", real, "ls-files", "--stage", "-z"]);
    declined(root.as_path(), clean.join("dir").as_path(), &["ls-files", "--stage", "-z", "--full-name"]);
    remove(root.as_path());
}

/// A repository whose index really is a sparse index: a cone-mode checkout of one directory with `--sparse-index`.
fn sparse_index_repository(root: &Path) -> PathBuf {
    let repo = root.join("sparse");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    for directory in ["kept", "left-out/deep", "also-out"] {
        std::fs::create_dir_all(repo.join(directory)).expect("dirs");
        std::fs::write(repo.join(directory).join("f.txt"), format!("{directory}\n")).expect("file");
    }
    std::fs::write(repo.join("top.txt"), "top\n").expect("top");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    git(repo.as_path(), &["sparse-checkout", "init", "--cone", "--sparse-index"]);
    git(repo.as_path(), &["sparse-checkout", "set", "kept"]);
    return repo;
}

/// A real sparse index is left to Git by every query, whether the configuration keys that announce it are present or not,
/// and where the readers would answer, they equal Git.
#[test]
fn a_real_sparse_index_is_declined() {
    let root = fixture("repo-read-query-sparse");
    let repo = sparse_index_repository(root.as_path());
    // The fixture really is sparse: the index file carries the sparse-directory extension, which `ls-files --sparse` lists.
    let index = std::fs::read(repo.join(".git/index")).expect("index");
    assert!(index.windows(4).any(|window| { return window == b"sdir"; }), "the index has no sparse-directory extension");
    let listed = text(repo.as_path(), &["ls-files", "--sparse", "--stage"]);
    let full = text(repo.as_path(), &["ls-files", "--stage"]);
    assert_ne!(listed, full, "the sparse index lists a sparse directory that a full listing does not");
    let head = text(repo.as_path(), &["rev-parse", "HEAD"]);
    let shapes: [Vec<&str>; 3] = [
        vec!["ls-files", "--stage", "-z", "--full-name"],
        vec!["diff-index", "--cached", "--name-only", "-z", &head],
        vec!["diff-index", "--cached", "--raw", "-z", "--no-renames", "--no-abbrev", &head],
    ];
    for shape in &shapes {
        declined(root.as_path(), repo.as_path(), shape);
    }
    // Without the keys that announce a sparse checkout, the index extension alone must still make the readers decline.
    let kept_lines: Vec<String> = std::fs::read_to_string(repo.join(".git/config"))
        .expect("config")
        .lines()
        .filter(|line| {
            let key = line.trim().to_ascii_lowercase();
            return !(key.starts_with("sparsecheckout") || key.starts_with("sparse ") || key.starts_with("worktreeconfig"));
        })
        .map(str::to_string)
        .collect();
    std::fs::write(repo.join(".git/config"), kept_lines.join("\n") + "\n").expect("config");
    if repo.join(".git/config.worktree").exists() {
        std::fs::remove_file(repo.join(".git/config.worktree")).expect("worktree config");
    }
    for shape in &shapes {
        assert!(mine(root.as_path(), repo.as_path(), shape).is_none() || compare(root.as_path(), repo.as_path(), shape), "{shape:?}");
    }
    assert!(mine(root.as_path(), repo.as_path(), &shapes[0]).is_none(), "the index extension must be declined without the keys");
    remove(root.as_path());
}

/// With `extensions.worktreeConfig` on, the readers answer and equal Git while `config.worktree` holds nothing that
///  changes the output, and decline whenever that file sets a key that could (`core.sparseCheckout` is where
///  `git sparse-checkout` itself writes it, `core.ignoreCase` and `diff.relative` change what is printed),
///  also through an include.
#[test]
fn a_worktree_config_that_sets_a_content_key_is_declined() {
    let root = fixture("repo-read-query-worktree-config");
    let repo = staged_repository(root.as_path(), "wt", "sha1");
    git(repo.as_path(), &["config", "--local", "extensions.worktreeConfig", "true"]);
    let head = text(repo.as_path(), &["rev-parse", "HEAD"]);
    let shapes: [Vec<&str>; 2] = [
        vec!["ls-files", "--stage", "-z", "--full-name"],
        vec!["diff-index", "--cached", "--name-only", "-z", "--no-renames", head.as_str()],
    ];
    let worktree_file = repo.join(".git/config.worktree");
    std::fs::write(&worktree_file, "[user]\n\tname = Only Here\n").expect("config.worktree");
    for shape in &shapes {
        answered(root.as_path(), repo.as_path(), shape);
    }
    for (label, text) in [
        ("core.sparseCheckout", "[core]\n\tsparseCheckout = true\n"),
        ("core.ignoreCase", "[core]\n\tignoreCase = true\n"),
        ("diff.relative", "[diff]\n\trelative = true\n"),
        ("through an include", "[include]\n\tpath = extra\n"),
    ] {
        std::fs::write(repo.join(".git/extra"), "[core]\n\tsparseCheckoutCone = true\n[diff]\n\trelative = true\n").expect("extra");
        std::fs::write(&worktree_file, text).expect("config.worktree");
        for shape in &shapes {
            assert!(mine(root.as_path(), repo.as_path(), shape).is_none(), "{label}: the readers answered {shape:?} but must leave it to Git");
        }
    }
    remove(root.as_path());
}

/// What a commit changed is found by reading the trees along the changed path and no others: the two trees of a commit share
/// nearly every subtree, and the listing of a commit's changes used to read every file of both trees (measured: 20,740 files,
/// about 1,400 trees on each side, in the gate that follows every commit). A repository of many directories with one changed
/// file is the shape where the two ways differ most; the answer must still equal Git's.
#[test]
fn the_changes_of_a_commit_are_found_without_reading_the_unchanged_directories() {
    let root = fixture("repo-read-query-changes-only");
    let repo = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    let directories: usize = 40;
    for directory in 0..directories {
        std::fs::create_dir_all(repo.join(format!("d{directory}/sub"))).expect("dirs");
        std::fs::write(repo.join(format!("d{directory}/sub/f.txt")), format!("{directory}\n")).expect("file");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    std::fs::write(repo.join("d3/sub/f.txt"), "changed\n").expect("edit");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=second"]);
    git(repo.as_path(), &["repack", "-a", "-d", "-f", "--quiet"]);
    git(repo.as_path(), &["prune-packed"]);
    let head = text(repo.as_path(), &["rev-parse", "HEAD"]);
    let shape = ["diff-tree", "--root", "--no-commit-id", "-r", "-z", "-m", "--no-renames", "--no-abbrev", head.as_str()];
    let before = crate::repo_read_pack::DECODED_TOTAL.load(std::sync::atomic::Ordering::Relaxed);
    answered(root.as_path(), repo.as_path(), &shape);
    let decoded = crate::repo_read_pack::DECODED_TOTAL.load(std::sync::atomic::Ordering::Relaxed) - before;
    // Positive control: the repository has more directories than the changed path is deep, so reading all of both trees would
    // decode more entries than there are directories.
    assert!(decoded > 0, "the answer reads the pack");
    assert!(decoded < directories as u64, "only the trees along the changed path are read, not every directory: {decoded} entries decoded");
    remove(root.as_path());
}
