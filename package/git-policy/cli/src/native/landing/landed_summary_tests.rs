//! Controls for the landed commit's summary line:
//! its shapes,
//! its subject,
//! and the name it carries.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};

const NAME: &str = "30b2211a4c";

/// A branch,
/// a root commit and a detached head are worded as `git commit` words them.
#[test]
fn the_line_words_the_branch_the_root_and_the_detached_head_as_git_does() {
    assert_eq!(summary_line("refs/heads/main", "abc", NAME, "subject"), format!("[main {NAME}] subject"));
    assert_eq!(summary_line("refs/heads/feature/x", "abc", NAME, "s"), format!("[feature/x {NAME}] s"));
    assert_eq!(summary_line("refs/heads/main", "", NAME, "s"), format!("[main (root-commit) {NAME}] s"));
    let zero: String = "0".repeat(40);
    assert_eq!(summary_line("refs/heads/main", zero.as_str(), NAME, "s"), format!("[main (root-commit) {NAME}] s"));
    assert_eq!(summary_line("HEAD", "abc", NAME, "s"), format!("[detached HEAD {NAME}] s"));
}

/// The subject is the first paragraph on one line,
/// as Git's `%s` makes it.
#[test]
fn the_subject_is_the_first_paragraph_on_one_line() {
    assert_eq!(subject_of("one\n"), "one");
    assert_eq!(subject_of("one\ntwo\n\nbody\n"), "one two");
    // Git's `%s` keeps the white space at the start of a line and drops the white space at its end:
    // measured, `git log -1 --format=%s` of the message `  padded  \n  twice` is `  padded   twice`.
    assert_eq!(subject_of("\n\n  padded  \n  twice\n\nbody"), "  padded   twice");
    assert_eq!(subject_of("subject\u{a0}\n"), "subject\u{a0}");
    assert_eq!(subject_of("subject\u{b}\n"), "subject\u{b}");
    assert_eq!(subject_of("\u{a0}\n"), "\u{a0}");
    assert_eq!(subject_of(""), "");
}

/// The name equals what `git rev-parse --short` prints,
/// with the key unset and with it set,
/// and the snapshot's last value wins.
#[test]
fn the_name_equals_gits_with_the_key_unset_and_set() {
    let root = fixture("landed-summary-name");
    let repo = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "one\n").expect("file");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    let commit: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    let context = git_context();
    let objects = repo.join(".git/objects");
    for setting in [None, Some("12"), Some("auto"), Some("no")] {
        if let Some(value) = setting {
            git(repo.as_path(), &["config", "core.abbrev", value]);
        }
        let listing: Vec<u8> = git_text(repo.as_path(), &["config", "--list", "-z", "--show-scope"]).into_bytes();
        let snapshot = ConfigSnapshot::parse(listing.as_slice(), true).expect("a snapshot");
        let expected: String = git_text(repo.as_path(), &["rev-parse", "--short", commit.as_str()]).trim().to_string();
        assert_eq!(short_name(&context, repo.as_path(), objects.as_path(), Some(&snapshot), commit.as_str()), expected, "core.abbrev {setting:?}");
    }
    remove(root.as_path());
}

/// Without a snapshot the in-process reader is not used and Git answers.
#[test]
fn without_a_snapshot_git_is_asked() {
    let root = fixture("landed-summary-asked");
    let repo = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "one\n").expect("file");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    git(repo.as_path(), &["config", "core.abbrev", "13"]);
    let commit: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    let named: String = short_name(&git_context(), repo.as_path(), repo.join(".git/objects").as_path(), None, commit.as_str());
    assert_eq!(named.len(), 13, "{named}");
    assert!(commit.starts_with(named.as_str()));
    remove(root.as_path());
}

/// When nothing can say,
/// the name is the first seven characters.
#[test]
fn the_last_resort_is_seven_characters() {
    let root = fixture("landed-summary-last-resort");
    let named: String = short_name(&git_context(), root.as_path(), root.join("no-objects").as_path(), None, "0123456789abcdef0123456789abcdef01234567");
    assert_eq!(named.len(), LAST_RESORT_LENGTH, "{named}");
    remove(root.as_path());
}

/// The note names every build that did not land,
/// in the order they were built,
/// and the commit that did.
#[test]
fn the_rebuilt_note_names_each_build_that_did_not_land_and_the_one_that_did() {
    let names = |words: &[&str]| -> Vec<String> { return words.iter().map(|word| { return (*word).to_string(); }).collect() };
    assert_eq!(
        rebuilt_note(&names(&["1a2b3c4"]), "5d6e7f8"),
        "monochromatic-git-policy-cli: the branch moved during the commit, so it was built again; 1a2b3c4 did not land, 5d6e7f8 did."
    );
    assert_eq!(
        rebuilt_note(&names(&["1a2b3c4", "9f8e7d6"]), "5d6e7f8"),
        "monochromatic-git-policy-cli: the branch moved during the commit, so it was built again; 1a2b3c4 and 9f8e7d6 did not land, 5d6e7f8 did."
    );
    assert_eq!(
        rebuilt_note(&names(&["1a2b3c4", "9f8e7d6", "0a0b0c0"]), "5d6e7f8"),
        "monochromatic-git-policy-cli: the branch moved during the commit, so it was built again; 1a2b3c4, 9f8e7d6 and 0a0b0c0 did not land, 5d6e7f8 did."
    );
}
