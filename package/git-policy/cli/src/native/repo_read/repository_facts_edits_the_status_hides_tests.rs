//! What:
//!  The index's own record of a marked path read back from a `git ls-files -v --stage --debug -z`
//!  answer.
//! Why:
//!  This reading decides whether a marked edit is visible,
//!  and a header read from the wrong bytes compares the wrong file:
//!  the format puts the five stat lines after the entry's own NUL,
//!  so the next header follows them rather than the NUL.
//!  A chunk that is not a header must name nothing,
//!  because a path nobody checked must never be reported as unedited.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parseMarkedEntries(Buffer.from('h 100644 <sha> 0\ta.txt\0  ctime: …\n  mtime: 1:2\n  …'))
//! //   === [{ mode: '100644', object: '<sha>', path: 'a.txt', size: 2, modified: [1, 2] }]
//! ```

use super::{after_stat_lines, edits_the_status_hides, entry_of, is_marked, parse_marked_entries};
use crate::command_test_support::{git, repository_with_tracked_file};
use crate::repo_read::repository_facts::{GitFacts, git_facts};
use crate::test_support::{REAL_GIT, isolated_overlay, remove};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// A provider for real Git after `-C <repository>`,
///  which is the same Git the wrapper runs a command against.
fn facts_in(repository: &Path) -> GitFacts {
    let mut prefix: Vec<OsString> = Vec::<OsString>::new();
    prefix.push(OsString::from("-C"));
    prefix.push(repository.as_os_str().to_os_string());
    return git_facts(Path::new(REAL_GIT), prefix.as_slice(), isolated_overlay().as_slice());
}

/// The five stat lines Git writes after one entry,
/// as this control spells them.
const STAT_BLOCK: &str = "  ctime: 1:2\n  mtime: 1700000000:123456789\n  dev: 1\tino: 2\n  uid: 3\tgid: 4\n  size: 2\tflags: 8000\n";

/// One entry's own bytes,
/// with the stat block that follows them.
fn answer_of(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut answer: Vec<u8> = Vec::<u8>::new();
    for (header, path) in entries {
        answer.extend_from_slice(header.as_bytes());
        answer.push(b'\t');
        answer.extend_from_slice(path);
        answer.push(0);
        answer.extend_from_slice(STAT_BLOCK.as_bytes());
    }
    return answer;
}

/// A marked entry is read with its mode,
/// its object,
/// its path,
/// the size the index recorded and the modification time it kept.
#[test]
fn a_marked_entry_is_read_with_its_own_stat() {
    let answer: Vec<u8> = answer_of(&[("h 100644 1111111111111111111111111111111111111111 0", b"a.txt")]);
    let entries = parse_marked_entries(answer.as_slice());
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].mode, "100644");
    assert_eq!(entries[0].object, "1111111111111111111111111111111111111111");
    assert_eq!(entries[0].path, b"a.txt".to_vec());
    assert_eq!(entries[0].size, 2);
    assert_eq!(entries[0].modified, (1700000000, 123456789));
}

/// Two entries are both read,
/// and the second one's header is found after the first one's stat lines.
#[test]
fn the_second_entry_is_read_after_the_first_stat_block() {
    let answer: Vec<u8> = answer_of(&[
        ("h 100644 1111111111111111111111111111111111111111 0", b"a.txt"),
        ("h 100755 2222222222222222222222222222222222222222 0", b"dir/b.txt"),
    ]);
    let entries = parse_marked_entries(answer.as_slice());
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[1].path, b"dir/b.txt".to_vec());
    assert_eq!(entries[1].mode, "100755");
}

/// A path that holds a newline is read whole:
/// the boundary between two entries is the fifth stat line, never the path's own bytes.
#[test]
fn a_path_that_holds_a_newline_is_read_whole() {
    let answer: Vec<u8> = answer_of(&[
        ("h 100644 1111111111111111111111111111111111111111 0", b"a\nb.txt"),
        ("h 100644 2222222222222222222222222222222222222222 0", b"c.txt"),
    ]);
    let entries = parse_marked_entries(answer.as_slice());
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].path, b"a\nb.txt".to_vec());
    assert_eq!(entries[1].path, b"c.txt".to_vec());
}

/// A chunk that is not a header names nothing,
/// and an answer that holds fewer stat lines than Git writes stops the reading.
#[test]
fn a_chunk_that_is_no_header_names_nothing() {
    assert!(entry_of(b"not a header at all", STAT_BLOCK.as_bytes()).is_none());
    assert!(entry_of(b"h 100644 1111111111111111111111111111111111111111 1\ta.txt", STAT_BLOCK.as_bytes()).is_none());
    assert!(entry_of(b"h 100644 1111111111111111111111111111111111111111 0\ta.txt", b"no stat here\n").is_none());
    assert!(after_stat_lines(b"  ctime: 1:2\n").is_none());
}

/// Only a single lower-case tag is the mark;
/// an upper-case tag is an entry Git looks at,
/// and a `skip-worktree` entry keeps its file.
#[test]
fn only_the_assume_unchanged_tag_is_a_mark() {
    assert!(is_marked(b"h"));
    assert!(!is_marked(b"H"));
    assert!(!is_marked(b"S"));
    assert!(!is_marked(b"hh"));
    assert!(!is_marked(b""));
}

/// A tracked edit whose path the index marks assume-unchanged is named,
/// and the same edit read without the mark is named by the status instead.
#[test]
fn a_marked_edit_is_named_and_an_unmarked_one_is_not() {
    let (root, repository): (PathBuf, PathBuf) = repository_with_tracked_file("facts-edits-the-status-hides");
    std::fs::write(repository.join("tracked.txt"), b"the person's edit\n").expect("the edit");
    git(repository.as_path(), &["update-index", "--assume-unchanged", "tracked.txt"]);
    // The status the guard asks is empty, so the edit would be lost without this reading.
    let status = crate::command_test_support::git_status(repository.as_path(), &["status", "--porcelain=v1", "--untracked-files=no"]);
    assert!(status.stdout.is_empty(), "the mark must hide the edit from the status: {status:?}");
    let mut facts: GitFacts = facts_in(repository.as_path());
    assert_eq!(
        edits_the_status_hides(&mut facts, repository.as_path()),
        Ok(vec![String::from("tracked.txt")])
    );
    // A marked path that still holds the index's bytes is no edit, so nothing is named.
    std::fs::write(repository.join("tracked.txt"), b"one\n").expect("the committed bytes");
    let mut facts: GitFacts = facts_in(repository.as_path());
    assert_eq!(edits_the_status_hides(&mut facts, repository.as_path()), Ok(Vec::<String>::new()));
    // A touched path whose bytes are unchanged is no edit either: the bytes decide, not the clock.
    std::fs::write(repository.join("tracked.txt"), b"one\n").expect("the committed bytes again");
    let mut facts: GitFacts = facts_in(repository.as_path());
    assert_eq!(edits_the_status_hides(&mut facts, repository.as_path()), Ok(Vec::<String>::new()));
    remove(root.as_path());
}
