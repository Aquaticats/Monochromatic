//! Controls for the index flag reader,
//! each compared with what real Git reports for the same index:
//!  `git ls-files -v` tags an assume-unchanged entry with a lower-case letter and a skip-worktree entry with `S`.

use super::*;
use crate::path_bytes::os_from_path_bytes;
use crate::test_support::{fixture, git, git_text, remove, repository};
use std::path::{Path, PathBuf};

/// Names of every length from one character up,
/// so every padding of an entry is read.
fn names_of_every_length() -> Vec<String> {
    return (1..=18).map(|length| { return "n".repeat(length); }).collect();
}

/// A repository with the files committed,
/// plus a path longer than one directory name can be.
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

/// What:
///  What real Git says the marks hide:
///  `git ls-files -v` tags a skip-worktree entry with `S` and an assume-unchanged one with a lower-case
///  letter,
/// and prints the mark before the path.
/// Why:
///   The reader's own answer is the two path lists,
/// so Git's listing is the oracle for which paths they name and not only whether there are any.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hidden = gitHidden(repo); // { skipWorktree: string[]; assumeUnchanged: string[] }
/// ```
fn git_says_hidden(repo: &Path) -> MarkedPaths {
    let listing: String = git_text(repo, &["ls-files", "-v"]);
    let mut hidden: MarkedPaths = MarkedPaths { skip_worktree: Vec::new(), assume_unchanged: Vec::new() };
    for line in listing.lines() {
        let Some(tag) = line.chars().next() else { continue };
        let Some(path) = line.strip_prefix(tag).and_then(|rest| { return rest.strip_prefix(' '); }) else { continue };
        if tag == 'S' {
            hidden.skip_worktree.push(path.as_bytes().to_vec());
        } else if tag.is_ascii_lowercase() {
            hidden.assume_unchanged.push(path.as_bytes().to_vec());
        }
    }
    return hidden;
}

/// What:
///  The reader and Git agree on the expected answer for the repository's index,
///  path for path.
/// Why:
///   The scenario sets what it says it sets,
/// and a reader that reported the wrong path would send the staging to the wrong entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assertAgrees(repo, expectedMarked);
/// ```
fn assert_agrees(repo: &Path, expected_marked: bool) {
    let hidden: MarkedPaths = git_says_hidden(repo);
    let git_marked: bool = !hidden.skip_worktree.is_empty() || !hidden.assume_unchanged.is_empty();
    assert_eq!(git_marked, expected_marked, "the scenario sets what it says it sets");
    match read(repo) {
        IndexFlags::Marked(marks) => {
            assert!(expected_marked, "an unmarked index must be plain");
            assert_eq!(marks.skip_worktree, hidden.skip_worktree, "the skip-worktree paths Git lists");
            assert_eq!(marks.assume_unchanged, hidden.assume_unchanged, "the assume-unchanged paths Git lists");
        }
        other => assert_eq!(other, IndexFlags::Plain, "the reader's answer for an unmarked index"),
    }
}

/// An index with no mark is plain,
/// whatever the version,
/// entry padding and name length.
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

/// An assume-unchanged entry marks the index,
/// in the old and the prefix-compressed layout,
/// for the first,
/// a middle and
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

/// Whether real Git reports an intent-to-add entry:
/// `git status --porcelain` prints it as ` A` (index unchanged,
/// work tree added).
fn git_says_intent_to_add(repo: &Path) -> bool {
    return git_text(repo, &["status", "--porcelain"]).lines().any(|line| { return line.starts_with(" A "); });
}

/// The paths three intent-to-add entries name:
/// one plain,
/// one with a byte that is not UTF-8,
/// and one with spaces,
/// a quote and a backslash in it,
/// in the order Git sorts them.
fn odd_names(repo: &Path) -> Vec<Vec<u8>> {
    use std::os::unix::ffi::OsStringExt as _;
    let latin: std::ffi::OsString = std::ffi::OsString::from_vec(b"latin-\xe9.txt".to_vec());
    let odd: &str = "spaces, \"quotes\" and \\ a backslash.txt";
    std::fs::write(repo.join("added.txt"), "new\n").expect("new file");
    std::fs::write(repo.join(&latin), "new\n").expect("a name that is not UTF-8");
    std::fs::write(repo.join(odd), "new\n").expect("a name with the characters Git quotes");
    for name in [std::ffi::OsString::from("added.txt"), latin.clone(), std::ffi::OsString::from(odd)] {
        git(repo, &[std::ffi::OsString::from("add"), std::ffi::OsString::from("--intent-to-add"), std::ffi::OsString::from("--"), name]);
    }
    return vec![b"added.txt".to_vec(), latin.into_vec(), odd.as_bytes().to_vec()];
}

/// An intent-to-add entry names its path,
/// in both entry layouts that carry extended flags,
/// and clearing it makes the index plain again.
#[test]
fn an_intent_to_add_entry_marks_the_index_with_its_paths() {
    let root: PathBuf = fixture("index-flags-intent");
    let repo: PathBuf = populated_repository(root.as_path());
    let paths: Vec<Vec<u8>> = odd_names(repo.as_path());
    assert!(git_says_intent_to_add(repo.as_path()), "the scenario sets what it says it sets");
    for version in ["3", "4"] {
        git(repo.as_path(), &["update-index", "--index-version", version]);
        assert_eq!(read(repo.as_path()), IndexFlags::IntentToAdd(paths.clone()), "version {version}");
    }
    for path in &paths {
        git(repo.as_path(), &[std::ffi::OsString::from("add"), std::ffi::OsString::from("--"), os_from_path_bytes(path.as_slice())]);
    }
    assert!(!git_says_intent_to_add(repo.as_path()));
    assert_eq!(read(repo.as_path()), IndexFlags::Plain);
    remove(root.as_path());
}

/// An assume-unchanged or skip-worktree entry sends the whole index to the slower build,
/// so it answers that the index is marked even where an intent-to-add entry is there too.
#[test]
fn a_hidden_entry_wins_over_an_intent_to_add_one() {
    let root: PathBuf = fixture("index-flags-intent-hidden");
    let repo: PathBuf = populated_repository(root.as_path());
    odd_names(repo.as_path());
    git(repo.as_path(), &["update-index", "--skip-worktree", "--", "n"]);
    assert_agrees(repo.as_path(), true);
    git(repo.as_path(), &["update-index", "--no-skip-worktree", "--", "n"]);
    git(repo.as_path(), &["update-index", "--assume-unchanged", "--", "n"]);
    assert_agrees(repo.as_path(), true);
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

/// A SHA-256 repository has longer object names;
/// the reader finds the marks there too.
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

/// A split index keeps entries in another file,
/// so nothing is known about its marks.
#[test]
fn a_split_index_is_unreadable() {
    let root: PathBuf = fixture("index-flags-split");
    let repo: PathBuf = populated_repository(root.as_path());
    git(repo.as_path(), &["update-index", "--split-index"]);
    assert_eq!(read(repo.as_path()), IndexFlags::Unreadable);
    remove(root.as_path());
}

/// An index with no entries is plain,
/// and anything that is not a whole index is unreadable,
/// never plain.
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
