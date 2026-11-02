//! Controls for the cleanliness check against real Git 2.56.0 repositories: a clean worktree is proven clean, and each
//!  kind of difference (content, mode, type, deletion, staging, untracked file, operation in progress, submodule,
//!  racy entry) is refused with its reason, while an ignored file changes nothing.

use super::*;
use crate::test_support::{age, fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// A repository with two tracked files, a tracked symbolic link and an ignored directory, all older than the index.
fn aged_repository(root: &Path) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join(".gitignore"), b"ignored/\n").expect("gitignore");
    std::fs::write(repo.join("a.txt"), b"alpha\n").expect("a");
    std::fs::create_dir(repo.join("dir")).expect("dir");
    std::fs::write(repo.join("dir/b.txt"), b"bravo\n").expect("b");
    std::os::unix::fs::symlink("a.txt", repo.join("link")).expect("link");
    for name in [".gitignore", "a.txt", "dir/b.txt", "link"] {
        age(repo.join(name).as_path());
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    return repo;
}

/// The check for a main checkout, whose Git directory is its `.git`.
fn check(repo: &Path) -> Result<CleanSource, String> {
    let git_dir: PathBuf = repo.join(".git");
    return check_clean(&git_context(), repo, git_dir.as_path(), git_dir.as_path());
}

/// A clean, aged worktree is clean and reports the commit and tree of `HEAD`.
#[test]
fn a_clean_worktree_is_clean_and_names_its_head() {
    let root: PathBuf = fixture("clean-ok");
    let repo: PathBuf = aged_repository(root.as_path());
    std::fs::create_dir(repo.join("ignored")).expect("ignored directory");
    std::fs::write(repo.join("ignored/cache"), b"cache\n").expect("ignored file");
    let clean: CleanSource = check(repo.as_path()).expect("clean");
    assert_eq!(clean.head_commit, git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim());
    assert_eq!(to_hex_for_test(clean.head_tree.as_slice()), git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]).trim());
    assert_eq!(clean.tracked, 4);
    remove(root.as_path());
}

/// The raw object name as hexadecimal.
fn to_hex_for_test(name: &[u8]) -> String {
    return name.iter().map(|byte: &u8| { return format!("{byte:02x}"); }).collect();
}

/// Each difference is refused, and the reason names the path or the state.
#[test]
fn each_difference_is_refused_with_its_reason() {
    let root: PathBuf = fixture("clean-dirty");
    type Make = fn(&Path);
    let cases: [(&str, &str, Make); 10] = [
        ("rewritten content", "a.txt differs from its index entry", |tree| { std::fs::write(tree.join("a.txt"), b"ALPHA\n").expect("write"); }),
        ("deleted", "dir/b.txt is missing", |tree| { std::fs::remove_file(tree.join("dir/b.txt")).expect("remove"); }),
        ("executable bit", "a.txt changed its executable bit", |tree| {
            std::fs::set_permissions(tree.join("a.txt"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("chmod");
        }),
        ("type change", "link changed its type", |tree| {
            std::fs::remove_file(tree.join("link")).expect("remove");
            std::fs::write(tree.join("link"), b"a.txt").expect("write");
        }),
        ("untracked", "new.txt is untracked and not ignored", |tree| { std::fs::write(tree.join("new.txt"), b"n").expect("write"); }),
        ("untracked in a directory", "dir/", |tree| { std::fs::write(tree.join("dir/new.txt"), b"n").expect("write"); }),
        ("staged change", "a.txt is staged with a change", |tree| {
            std::fs::write(tree.join("a.txt"), b"staged\n").expect("write");
            git(tree, &["add", "a.txt"]);
        }),
        ("staged deletion", "the index and the tree of HEAD hold different numbers", |tree| { git(tree, &["rm", "--quiet", "--cached", "a.txt"]); }),
        ("merge in progress", "MERGE_HEAD exists", |tree| {
            let head: String = git_text(tree, &["rev-parse", "HEAD"]);
            std::fs::write(tree.join(".git/MERGE_HEAD"), head.as_bytes()).expect("write");
        }),
        ("held index lock", "index.lock exists", |tree| { std::fs::write(tree.join(".git/index.lock"), b"").expect("write"); }),
    ];
    for (label, expected, make) in cases {
        let one: PathBuf = repository(root.as_path(), format!("case-{}", label.replace(' ', "-")).as_str());
        std::fs::write(one.join(".gitignore"), b"ignored/\n").expect("gitignore");
        std::fs::write(one.join("a.txt"), b"alpha\n").expect("a");
        std::fs::create_dir(one.join("dir")).expect("dir");
        std::fs::write(one.join("dir/b.txt"), b"bravo\n").expect("b");
        std::os::unix::fs::symlink("a.txt", one.join("link")).expect("link");
        for name in [".gitignore", "a.txt", "dir/b.txt", "link"] {
            age(one.join(name).as_path());
        }
        git(one.as_path(), &["add", "--all"]);
        git(one.as_path(), &["commit", "--quiet", "--message=files"]);
        assert!(check(one.as_path()).is_ok(), "{label}: clean before the change");
        make(one.as_path());
        let reason: String = check(one.as_path()).expect_err(label);
        assert!(reason.contains(expected), "{label}: {reason}");
    }
    remove(root.as_path());
}

/// An entry written in the same second as the index proves nothing by its stat fields, so it is refused.
#[test]
fn a_racy_entry_is_refused() {
    let root: PathBuf = fixture("clean-racy");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("fresh.txt"), b"fresh\n").expect("fresh");
    git(repo.as_path(), &["add", "fresh.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=fresh"]);
    let reason: String = check(repo.as_path()).expect_err("racy");
    assert!(reason.contains("as new as the index"), "{reason}");
    remove(root.as_path());
}

/// A submodule entry is refused: its working directory would point at the source's module directory.
#[test]
fn a_submodule_entry_is_refused() {
    let root: PathBuf = fixture("clean-submodule");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["update-index", "--add", "--cacheinfo", format!("160000,{},sub", head.trim()).as_str()]);
    git(repo.as_path(), &["commit", "--quiet", "--message=submodule"]);
    let reason: String = check(repo.as_path()).expect_err("submodule");
    assert!(reason.contains("sub is a submodule"), "{reason}");
    remove(root.as_path());
}

/// A linked worktree is checked against its own Git directory and index.
#[test]
fn a_linked_worktree_is_checked_against_its_own_git_directory() {
    let root: PathBuf = fixture("clean-linked");
    let repo: PathBuf = aged_repository(root.as_path());
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "linked", "../linked"]);
    let linked: PathBuf = std::fs::canonicalize(root.join("linked")).expect("canonical");
    for name in ["a.txt", "dir/b.txt", ".gitignore", "link"] {
        age(linked.join(name).as_path());
    }
    git(linked.as_path(), &["update-index", "--refresh"]);
    let admin: PathBuf = repo.join(".git/worktrees/linked");
    // Git wrote the linked index within the second the files were aged, so refresh made it newer than the entries.
    let outcome: Result<CleanSource, String> = check_clean(&git_context(), linked.as_path(), admin.as_path(), repo.join(".git").as_path());
    assert!(outcome.is_ok(), "{outcome:?}");
    std::fs::write(linked.join("untracked.txt"), b"u").expect("write");
    assert!(check_clean(&git_context(), linked.as_path(), admin.as_path(), repo.join(".git").as_path()).is_err());
    remove(root.as_path());
}

/// The comparison below a held directory finds a tracked directory that was replaced by a link, which a walk by path would
///  follow to a copy; with the real directories it agrees with the path comparison.
#[test]
fn the_comparison_below_a_handle_refuses_a_link_in_place_of_a_directory() {
    let root: PathBuf = fixture("clean-beneath");
    let repo: PathBuf = aged_repository(root.as_path());
    let clean: CleanSource = check(repo.as_path()).expect("clean");
    let entries: Vec<StatEntry> = stored_stats(clean.index_bytes.as_slice(), clean.hash_length).expect("entries");
    let held: std::fs::File = crate::fs_beneath::open_directory(repo.as_path()).expect("handle");
    assert_eq!(tree_differs_from_index_beneath(&held, entries.as_slice(), clean.index_seconds), None, "positive control");
    std::fs::create_dir(root.join("copy")).expect("copy");
    std::fs::copy(repo.join("dir/b.txt"), root.join("copy/b.txt")).expect("copy of the file");
    std::fs::rename(repo.join("dir"), root.join("dir.orig")).expect("moved away");
    std::os::unix::fs::symlink(root.join("copy"), repo.join("dir")).expect("link in its place");
    let reason: String = tree_differs_from_index_beneath(&held, entries.as_slice(), clean.index_seconds).expect("refused");
    assert!(reason.contains("dir/b.txt") && reason.contains("link"), "{reason}");
    remove(root.as_path());
}

/// A metadata that equals the entry is clean and one that does not names its path.
#[test]
fn metadata_of_a_file_in_hand_is_judged_like_its_path() {
    let root: PathBuf = fixture("clean-metadata");
    let repo: PathBuf = aged_repository(root.as_path());
    let clean: CleanSource = check(repo.as_path()).expect("clean");
    let entries: Vec<StatEntry> = stored_stats(clean.index_bytes.as_slice(), clean.hash_length).expect("entries");
    let entry: &StatEntry = entries.iter().find(|entry: &&StatEntry| { return entry.path == b"a.txt"; }).expect("a.txt");
    let held = std::fs::File::open(repo.join("a.txt")).expect("open").metadata().expect("metadata");
    assert_eq!(metadata_differs_from_entry(entry, &held, clean.index_seconds), None);
    std::fs::write(repo.join("a.txt"), b"ALPHA\n").expect("rewritten");
    let changed = std::fs::File::open(repo.join("a.txt")).expect("open").metadata().expect("metadata");
    assert!(metadata_differs_from_entry(entry, &changed, clean.index_seconds).expect("changed").contains("a.txt"));
    remove(root.as_path());
}
