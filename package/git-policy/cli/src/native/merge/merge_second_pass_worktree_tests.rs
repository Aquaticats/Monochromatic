//! Controls for the pieces of the worktree pass that need no running Git:
//! the stages read from index entries,
//! the records
//!  that put a conflict back,
//! and the closing words.
//! The pass as a whole runs through the built executable in
//!  `binary_word_merge_tests.rs`.

use super::*;
use std::path::Path;

/// An index entry of a path at a stage.
fn entry(path: &str, stage: u8, mode: u32, byte: u8) -> IndexEntry {
    return IndexEntry { path: path.as_bytes().to_vec(), mode, id: vec![byte; 20], stage, extended: 0 };
}

/// Only the named paths are read,
/// every stage lands where it belongs,
/// and a resolved entry is ignored.
#[test]
fn the_stages_of_the_named_paths_are_read_from_the_index() {
    let entries: Vec<IndexEntry> = vec![
        entry("a.md", 0, 0o100_644, 9),
        entry("b.md", 1, 0o100_644, 1),
        entry("b.md", 2, 0o100_755, 2),
        entry("b.md", 3, 0o100_644, 3),
        entry("c.md", 2, 0o100_644, 4),
        entry("c.md", 3, 0o100_644, 5),
    ];
    let only: BTreeSet<Vec<u8>> = [b"b.md".to_vec(), b"c.md".to_vec()].into_iter().collect();
    let conflicts: Vec<Conflict> = conflicts_of_index(entries.as_slice(), &only);
    assert_eq!(conflicts.len(), 2);
    assert_eq!(conflicts[0].path, b"b.md".to_vec());
    assert_eq!(conflicts[0].base, Some(Stage { mode: 0o100_644, oid: "01".repeat(20) }));
    assert_eq!(conflicts[0].ours.as_ref().map(|stage| { return stage.mode; }), Some(0o100_755));
    assert_eq!(conflicts[0].theirs, Some(Stage { mode: 0o100_644, oid: "03".repeat(20) }));
    assert_eq!((conflicts[1].path.as_slice(), conflicts[1].base.is_none()), (b"c.md".as_slice(), true));
    let none: BTreeSet<Vec<u8>> = BTreeSet::new();
    assert!(conflicts_of_index(entries.as_slice(), &none).is_empty());
}

/// Putting a conflict back removes the resolved entry first and then lists each stage the path had,
/// in order.
#[test]
fn the_restore_records_remove_the_resolved_entry_and_list_the_stages() {
    let conflict = Conflict {
        path: b"d/x.md".to_vec(),
        base: Some(Stage { mode: 0o100_644, oid: String::from("aa") }),
        ours: Some(Stage { mode: 0o100_644, oid: String::from("bb") }),
        theirs: Some(Stage { mode: 0o100_755, oid: String::from("cc") }),
    };
    let expected: Vec<u8> = [b"0 00\td/x.md\0".as_slice(), b"100644 aa 1\td/x.md\0", b"100644 bb 2\td/x.md\0", b"100755 cc 3\td/x.md\0"].concat();
    assert_eq!(restore_records(&conflict), expected);
}

/// The closing words differ by whether a conflicted file remains,
/// and name the operation.
#[test]
fn the_closing_words_say_whether_a_conflict_remains() {
    assert!(closing_words("rebase", 0).contains("no conflicted file is left") && closing_words("rebase", 0).contains("finish the rebase"));
    assert!(closing_words("merge", 2).contains("still lists as conflicted") && closing_words("merge", 2).contains("git diff --cached"));
}

/// A runner that records every start and fails the command it is told to.
struct Scripted {
    /// The command word and the standard input of every start,
    /// in order.
    calls: std::cell::RefCell<Vec<(String, Vec<u8>)>>,
    /// The command word that exits with status 1.
    failing: &'static str,
}

/// Recording the starts instead of running Git.
impl GitRunner for Scripted {
    fn run(&self, arguments: &[&str], input: Option<&[u8]>, _index_file: Option<&Path>) -> Result<GitReply, String> {
        self.calls.borrow_mut().push((arguments[0].to_owned(), input.map(<[u8]>::to_vec).unwrap_or_default()));
        let code: Option<i32> = if arguments[0] == self.failing { Some(1) } else { Some(0) };
        return Ok(GitReply { code, stdout: Vec::new(), stderr: b"boom".to_vec() });
    }
}

/// One merged file with the stages it came from.
fn merged_file() -> Merged {
    let stage = |mode: u32, oid: &str| { return Some(Stage { mode, oid: oid.to_owned() }); };
    let stages = Conflict { path: b"a.md".to_vec(), base: stage(0o100_644, "aa"), ours: stage(0o100_644, "bb"), theirs: stage(0o100_644, "cc") };
    return Merged { path: b"a.md".to_vec(), mode: 0o100_644, bytes: b"merged\n".to_vec(), parts: Vec::new(), stages };
}

/// A file is staged and then written to the worktree;
/// nothing else runs when both work.
#[test]
fn a_merged_file_is_staged_and_then_checked_out() {
    let runner = Scripted { calls: std::cell::RefCell::new(Vec::new()), failing: "none" };
    install(&runner, &[merged_file()], &[String::from("dd")]).expect("both steps work");
    let calls = runner.calls.borrow();
    assert_eq!(calls.iter().map(|(word, _)| { return word.as_str(); }).collect::<Vec<&str>>(), vec!["update-index", "checkout-index"]);
    assert_eq!(calls[0].1, resolved_record(0o100_644, "dd", b"a.md"));
    assert_eq!(calls[1].1, b"a.md\0".to_vec());
}

/// A worktree write that fails puts the conflict stages back,
/// so the index never says resolved over a file with markers.
#[test]
fn a_failed_checkout_puts_the_conflict_stages_back() {
    let runner = Scripted { calls: std::cell::RefCell::new(Vec::new()), failing: "checkout-index" };
    let error: String = install(&runner, &[merged_file()], &[String::from("dd")]).expect_err("the checkout failed");
    assert!(error.contains("boom"), "{error}");
    let calls = runner.calls.borrow();
    assert_eq!(calls.iter().map(|(word, _)| { return word.as_str(); }).collect::<Vec<&str>>(), vec!["update-index", "checkout-index", "update-index"]);
    assert_eq!(calls[2].1, restore_records(&merged_file().stages));
}

/// A failed staging stops before the worktree is written.
#[test]
fn a_failed_staging_writes_nothing_to_the_worktree() {
    let runner = Scripted { calls: std::cell::RefCell::new(Vec::new()), failing: "update-index" };
    assert!(install(&runner, &[merged_file()], &[String::from("dd")]).is_err());
    assert_eq!(runner.calls.borrow().len(), 1);
}
