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

/// The text of an intent without its Git directory, as an earlier release wrote it, for a repository and an index elsewhere.
fn older_text(repository: &Path, index: &Path) -> String {
    let mut sent: Intent = intent(10);
    sent.repository = repository.to_path_buf();
    sent.index = index.to_path_buf();
    return intent_text(&sent).lines().filter(|line| { return !line.starts_with("gitdir="); }).collect::<Vec<&str>>().join("\n") + "\n";
}

/// An intent written before it recorded its Git directory names the Git directory of its repository, not the directory its
///  index sits in: an alternate index (`GIT_INDEX_FILE`) can sit anywhere.
#[test]
fn an_older_intent_names_the_git_directory_of_its_repository_and_not_of_its_index() {
    let root: PathBuf = fixture("service-journal-older-main");
    let repository: PathBuf = root.join("work/repo");
    std::fs::create_dir_all(repository.join(".git")).expect("a plain repository");
    let alternate: PathBuf = root.join("elsewhere/alternate.index");
    let read: Intent = parse_intent(older_text(repository.as_path(), alternate.as_path()).as_str()).expect("an older intent still reads");
    assert_eq!(read.git_dir, repository.join(".git"));
    assert_ne!(Some(read.git_dir.as_path()), alternate.parent(), "not the directory of the index");
    remove(root.as_path());
}

/// A linked worktree's `.git` file names its Git directory, and a repository that cannot be read falls back to `.git`
///  beneath it.
#[test]
fn an_older_intent_of_a_linked_worktree_reads_the_pointer_in_its_dot_git_file() {
    let root: PathBuf = fixture("service-journal-older-linked");
    let linked: PathBuf = root.join("work/linked");
    let admin: PathBuf = root.join("work/main/.git/worktrees/linked");
    std::fs::create_dir_all(&linked).expect("linked worktree");
    std::fs::create_dir_all(&admin).expect("administrative directory");
    std::fs::write(linked.join(".git"), format!("gitdir: {}\n", admin.display())).expect("pointer");
    let alternate: PathBuf = root.join("elsewhere/alternate.index");
    let read: Intent = parse_intent(older_text(linked.as_path(), alternate.as_path()).as_str()).expect("reads");
    assert_eq!(read.git_dir, admin);
    let missing: PathBuf = root.join("work/gone");
    let fallback: Intent = parse_intent(older_text(missing.as_path(), alternate.as_path()).as_str()).expect("reads");
    assert_eq!(fallback.git_dir, missing.join(".git"));
    remove(root.as_path());
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

/// Pruning keeps the newest finished batches and nothing else is touched: open batches, files that are not named as a batch
///  number, and directories stay.
#[test]
fn pruning_keeps_the_newest_finished_batches_and_touches_nothing_else() {
    let root: PathBuf = fixture("service-journal-prune");
    let retained: usize = 3;
    let total: u64 = 7;
    for seq in 1..=total {
        write_intent(root.as_path(), &intent(seq)).expect("write");
        mark_done(root.as_path(), seq).expect("mark");
    }
    let open: u64 = total + 1;
    write_intent(root.as_path(), &intent(open)).expect("an open batch");
    std::fs::write(root.join("notes.done"), "kept").expect("a file that is no batch");
    std::fs::create_dir(batch_path(root.as_path(), 0, DONE_EXTENSION)).expect("a directory named like a batch");
    let removed: usize = prune_done(root.as_path(), retained);
    assert_eq!(removed, usize::try_from(total).expect("a count") - retained);
    let oldest_kept: u64 = total - u64::try_from(retained).expect("a count") + 1;
    for seq in 1..oldest_kept {
        assert!(!batch_path(root.as_path(), seq, DONE_EXTENSION).exists(), "batch {seq} is among the oldest");
    }
    for seq in oldest_kept..=total {
        assert!(batch_path(root.as_path(), seq, DONE_EXTENSION).exists(), "batch {seq} is among the newest");
    }
    assert!(batch_path(root.as_path(), open, OPEN_EXTENSION).exists());
    assert!(root.join("notes.done").exists());
    assert!(batch_path(root.as_path(), 0, DONE_EXTENSION).is_dir());
    assert_eq!(prune_done(root.as_path(), retained), 0, "a second pruning has nothing to remove");
    assert_eq!(prune_done(root.join("absent").as_path(), retained), 0, "a journal that is not there has nothing to remove");
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
