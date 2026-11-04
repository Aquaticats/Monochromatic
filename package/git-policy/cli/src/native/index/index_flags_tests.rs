//! Controls for the index flag reader, each compared with what real Git reports for the same index:
//!  `git ls-files -v` tags an assume-unchanged entry with a lower-case letter and a skip-worktree entry with `S`.

use super::*;
use crate::test_support::{fixture, git, git_text, remove, repository};
use std::path::{Path, PathBuf};

/// Names of every length from one character up, so every padding of an entry is read.
fn names_of_every_length() -> Vec<String> {
    return (1..=18).map(|length| { return "n".repeat(length); }).collect();
}

/// A repository with the files committed, plus a path longer than one directory name can be.
fn populated_repository(root: &Path) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    let deep: PathBuf = repo.join("d".repeat(200)).join("e".repeat(200));
    std::fs::create_dir_all(&deep).expect("deep directories");
    std::fs::write(deep.join("deep.txt"), "deep\n").expect("deep file");
    for name in names_of_every_length() {
        std::fs::write(repo.join(&name), format!("{name}\n")).expect("file");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    return repo;
}

/// What the reader says about the repository's own index file.
fn read(repo: &Path) -> IndexFlags {
    let bytes: Vec<u8> = std::fs::read(repo.join(".git").join("index")).expect("index file");
    return read_index_flags(bytes.as_slice());
}

/// What real Git says: whether any entry is tagged assume-unchanged (lower case) or skip-worktree (`S`).
fn git_says_marked(repo: &Path) -> bool {
    let listing: String = git_text(repo, &["ls-files", "-v"]);
    return listing.lines().any(|line| {
        return line.chars().next().is_some_and(|tag| { return tag.is_ascii_lowercase() || tag == 'S'; });
    });
}

/// The reader and Git agree on the expected answer for the repository's index.
fn assert_agrees(repo: &Path, expected_marked: bool) {
    assert_eq!(git_says_marked(repo), expected_marked, "the scenario sets what it says it sets");
    let expected: IndexFlags = if expected_marked { IndexFlags::Marked } else { IndexFlags::Plain };
    assert_eq!(read(repo), expected);
}

/// An index with no mark is plain, whatever the version, entry padding and name length.
#[test]
fn an_unmarked_index_is_plain_in_every_version() {
    let root: PathBuf = fixture("index-flags-plain");
    let repo: PathBuf = populated_repository(root.as_path());
    assert_agrees(repo.as_path(), false);
    for version in ["3", "4", "2"] {
        git(repo.as_path(), &["update-index", "--index-version", version]);
        assert_agrees(repo.as_path(), false);
    }
    remove(root.as_path());
}

/// An assume-unchanged entry marks the index, in the old and the prefix-compressed layout, for the first, a middle and
///  the last entry.
#[test]
fn an_assume_unchanged_entry_marks_the_index() {
    let root: PathBuf = fixture("index-flags-assume");
    let repo: PathBuf = populated_repository(root.as_path());
    let names: Vec<String> = names_of_every_length();
    let candidates: [&str; 3] = [names[0].as_str(), names[names.len() / 2].as_str(), names[names.len() - 1].as_str()];
    for version in ["2", "4"] {
        git(repo.as_path(), &["update-index", "--index-version", version]);
        for name in candidates {
            git(repo.as_path(), &["update-index", "--assume-unchanged", "--", name]);
            assert_agrees(repo.as_path(), true);
            git(repo.as_path(), &["update-index", "--no-assume-unchanged", "--", name]);
            assert_agrees(repo.as_path(), false);
        }
    }
    remove(root.as_path());
}

/// Whether real Git reports an intent-to-add entry: `git status --porcelain` prints it as ` A` (index unchanged, work tree added).
fn git_says_intent_to_add(repo: &Path) -> bool {
    return git_text(repo, &["status", "--porcelain"]).lines().any(|line| { return line.starts_with(" A "); });
}

/// An intent-to-add entry marks the index, in both entry layouts that carry extended flags, and clearing it unmarks it.
#[test]
fn an_intent_to_add_entry_marks_the_index() {
    let root: PathBuf = fixture("index-flags-intent");
    let repo: PathBuf = populated_repository(root.as_path());
    std::fs::write(repo.join("added.txt"), "new\n").expect("new file");
    git(repo.as_path(), &["add", "--intent-to-add", "--", "added.txt"]);
    assert!(git_says_intent_to_add(repo.as_path()), "the scenario sets what it says it sets");
    for version in ["3", "4"] {
        git(repo.as_path(), &["update-index", "--index-version", version]);
        assert_eq!(read(repo.as_path()), IndexFlags::Marked, "version {version}");
    }
    git(repo.as_path(), &["add", "--", "added.txt"]);
    assert!(!git_says_intent_to_add(repo.as_path()));
    assert_eq!(read(repo.as_path()), IndexFlags::Plain);
    remove(root.as_path());
}

/// A skip-worktree entry marks the index and clearing it unmarks it.
#[test]
fn a_skip_worktree_entry_marks_the_index() {
    let root: PathBuf = fixture("index-flags-skip");
    let repo: PathBuf = populated_repository(root.as_path());
    for version in ["3", "4"] {
        git(repo.as_path(), &["update-index", "--index-version", version]);
        git(repo.as_path(), &["update-index", "--skip-worktree", "--", "n"]);
        assert_agrees(repo.as_path(), true);
        git(repo.as_path(), &["update-index", "--no-skip-worktree", "--", "n"]);
        assert_agrees(repo.as_path(), false);
    }
    remove(root.as_path());
}

/// A SHA-256 repository has longer object names; the reader finds the marks there too.
#[test]
fn a_sha256_index_is_read_with_its_longer_object_names() {
    let root: PathBuf = fixture("index-flags-sha256");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repo directory");
    git(repo.as_path(), &["init", "--quiet", "--object-format=sha256"]);
    for name in names_of_every_length() {
        std::fs::write(repo.join(&name), format!("{name}\n")).expect("file");
    }
    git(repo.as_path(), &["add", "--all"]);
    assert_agrees(repo.as_path(), false);
    git(repo.as_path(), &["update-index", "--assume-unchanged", "--", "nn"]);
    assert_agrees(repo.as_path(), true);
    remove(root.as_path());
}

/// A split index keeps entries in another file, so nothing is known about its marks.
#[test]
fn a_split_index_is_unreadable() {
    let root: PathBuf = fixture("index-flags-split");
    let repo: PathBuf = populated_repository(root.as_path());
    git(repo.as_path(), &["update-index", "--split-index"]);
    assert_eq!(read(repo.as_path()), IndexFlags::Unreadable);
    remove(root.as_path());
}

/// An index with no entries is plain, and anything that is not a whole index is unreadable, never plain.
#[test]
fn an_empty_index_is_plain_and_damaged_bytes_are_unreadable() {
    let root: PathBuf = fixture("index-flags-damaged");
    let repo: PathBuf = populated_repository(root.as_path());
    let whole: Vec<u8> = std::fs::read(repo.join(".git").join("index")).expect("index file");
    git(repo.as_path(), &["read-tree", "--empty"]);
    assert_eq!(read(repo.as_path()), IndexFlags::Plain);
    assert_eq!(read_index_flags(&[]), IndexFlags::Unreadable);
    assert_eq!(read_index_flags(b"not an index at all, just text"), IndexFlags::Unreadable);
    for cut in [whole.len() - 1, whole.len() / 2, HEADER_LENGTH] {
        assert_eq!(read_index_flags(&whole[..cut]), IndexFlags::Unreadable, "cut at {cut}");
    }
    let mut wrong_version: Vec<u8> = whole.clone();
    wrong_version[7] = 9;
    assert_eq!(read_index_flags(wrong_version.as_slice()), IndexFlags::Unreadable);
    remove(root.as_path());
}
