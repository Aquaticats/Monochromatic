//! Controls for the service journal: an intent survives a write and a read, the restart decision follows the branch,
//!  and a damaged or temporary file is never an intent.

use super::*;
use crate::test_support::{fixture, remove};

fn intent(seq: u64) -> Intent {
    return Intent {
        seq,
        repository: PathBuf::from("/work/repo"),
        common_dir: PathBuf::from("/work/repo/.git"),
        reference: String::from("refs/heads/main"),
        old: String::from("a".repeat(40)),
        new: String::from("b".repeat(40)),
        ids: vec![String::from("id-1"), String::from("id-2")],
        commits: vec![String::from("c".repeat(40)), String::from("d".repeat(40))],
        paths: vec![String::from("src/a b.txt"), String::from("c.txt")],
        derived: false,
        index: PathBuf::from("/work/repo/.git/index"),
        git_dir: PathBuf::from("/work/repo/.git"),
        cleanup: None,
    };
}

/// An intent's text parses back to the same intent, with spaces in paths.
#[test]
fn an_intent_round_trips_through_its_text() {
    let sent: Intent = intent(7);
    assert_eq!(parse_intent(intent_text(&sent).as_str()), Some(sent));
}

/// The index rule and the conclusion state of a commit Git built survive a restart, and an intent written before they
///  existed reads as one that names neither.
#[test]
fn a_derived_index_rule_and_a_cleanup_round_trip_and_older_intents_name_neither() {
    let mut sent: Intent = intent(8);
    sent.derived = true;
    sent.cleanup = Some(Cleanup { git_dir: PathBuf::from("/work/repo/.git"), scratch: PathBuf::from("/work/repo/.git/cli-git-service-x"), reftable: true });
    assert_eq!(parse_intent(intent_text(&sent).as_str()), Some(sent.clone()));
    let older: String = intent_text(&intent(9)).lines().filter(|line| { return !line.starts_with("derived=") && !line.starts_with("cleanup="); }).collect::<Vec<&str>>().join("\n");
    let read: Intent = parse_intent(older.as_str()).expect("an older intent still reads");
    assert!(!read.derived && read.cleanup.is_none());
}

/// A text with a field missing, or a sequence that is not a number, is no intent.
#[test]
fn damaged_text_is_no_intent() {
    let text: String = intent_text(&intent(1));
    assert_eq!(parse_intent(""), None);
    assert_eq!(parse_intent(text.replace("old=", "older=").as_str()), None);
    assert_eq!(parse_intent(text.replace("seq=1", "seq=one").as_str()), None);
    assert_eq!(parse_intent(&text[..text.len() / 3]), None);
}

/// The restart decision follows the branch: unmoved drops, at the new value or past it finishes, elsewhere drops.
#[test]
fn recovery_follows_the_branch() {
    let batch: Intent = intent(1);
    assert_eq!(recovery_for(&batch, batch.old.as_str(), false), Recovery::Drop);
    assert_eq!(recovery_for(&batch, batch.new.as_str(), false), Recovery::Finish);
    assert_eq!(recovery_for(&batch, "c".repeat(40).as_str(), true), Recovery::Finish);
    assert_eq!(recovery_for(&batch, "c".repeat(40).as_str(), false), Recovery::Drop);
    assert_eq!(recovery_for(&batch, batch.old.as_str(), true), Recovery::Drop, "an old value is never a landing");
}

/// A written intent is listed as open, a finished one is listed as done with its identities remembered, and the
///  listing is in batch order.
#[test]
fn written_intents_are_listed_and_finished_ones_are_remembered() {
    let root: PathBuf = fixture("service-journal-list");
    for seq in [3, 1, 2] {
        write_intent(root.as_path(), &intent(seq)).expect("write");
    }
    let open: Vec<u64> = list_intents(root.as_path(), OPEN_EXTENSION)
        .iter()
        .map(|(_, parsed)| { return parsed.as_ref().expect("readable").seq; })
        .collect();
    assert_eq!(open, vec![1, 2, 3]);
    assert!(landed_commits(root.as_path()).is_empty());
    mark_done(root.as_path(), 2).expect("mark");
    assert_eq!(landed_commits(root.as_path()).get("id-2"), Some(&"d".repeat(40)));
    assert_eq!(list_intents(root.as_path(), OPEN_EXTENSION).len(), 2);
    remove(root.as_path());
}

/// A temporary file left by a kill during the write is removed and never read, and a damaged intent is listed as
///  unreadable so recovery can drop it.
#[test]
fn a_torn_write_leaves_nothing_readable() {
    let root: PathBuf = fixture("service-journal-torn");
    std::fs::write(batch_path(root.as_path(), 5, "tmp"), "seq=5\nrep").expect("torn temporary");
    std::fs::write(batch_path(root.as_path(), 6, OPEN_EXTENSION), "seq=6\nrep").expect("torn intent");
    let listed = list_intents(root.as_path(), OPEN_EXTENSION);
    assert_eq!(listed.len(), 1);
    assert!(listed[0].1.is_none(), "the torn intent does not parse");
    assert!(!batch_path(root.as_path(), 5, "tmp").exists(), "the temporary file is removed");
    remove(root.as_path());
}
