//! What:
//!  The queries and readings of "what would this abort discard",
//!  the record it restores its tree from,
//!  `src/native/repo_read/repository_facts_abort_discard.rs`,
//!  and the real repository answers they produce.
//! Why:
//!  The listing shapes are what the guard trusts,
//!  so each one is checked against real Git here:
//!  the merge result `AUTO_MERGE` leaves untouched conflict markers out,
//!  a hand resolution in,
//!  the fallback lists every uncommitted change,
//!  and the commit a rebase or a sequence restores is read from the record Git wrote.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const paths = await abortDiscard(facts, GuardedCommand.AbortMerge);
//! expect(paths.paths).toEqual(['a.txt']);
//! ```

use super::{
    AbortDiscard, RECORD_BYTES_READ, StateAnswer, both_listings, holds_an_entry, merge_result_queries, read_the_record, recorded_commit,
    state_answer, state_from_presence, state_query_arguments, uncommitted_queries, unresolved_query,
};
use crate::rule_abort_operation::{RESTORED_REVISION_NAMES, SEQUENCER_DIRECTORY};
use crate::rule_linked_worktree::GuardedCommand;
use crate::rule_abort_operation::OPERATION_STATE_NAMES;
use crate::test_support::{fixture, remove};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The words of an argument list,
///  for comparing queries as text.
fn words(arguments: &[OsString]) -> Vec<String> {
    return arguments.iter().map(|argument| { return argument.to_string_lossy().into_owned(); }).collect();
}

/// Every name the state query asks about, in the order it asks.
fn asked_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = OPERATION_STATE_NAMES.to_vec();
    names.push(SEQUENCER_DIRECTORY);
    names.extend_from_slice(RESTORED_REVISION_NAMES.as_slice());
    return names;
}

/// An answer of one path per name,
///  with every path inside `root`,
///  so a presence fact is read from a file the test can make exist.
fn answer_of(root: &Path, names: &[&str]) -> Vec<u8> {
    let mut text: Vec<u8> = Vec::<u8>::new();
    for name in names {
        text.extend_from_slice(format!("{}\n", root.join(name.replace('/', "-").as_str()).display().to_string().as_str()).as_bytes());
    }
    return text;
}

/// The whole answer every state question produces.
fn answered(root: &Path) -> Vec<u8> {
    return answer_of(root, asked_names().as_slice());
}

/// The state query asks for each state entry,
///  the sequencer directory,
///  and every record an abort restores a commit from.
#[test]
fn the_state_query_names_every_entry_once() {
    let asked: Vec<String> = words(state_query_arguments(&[OsString::from("-C"), OsString::from("/repo")]).as_slice());
    assert_eq!(asked.first().map(String::as_str), Some("-C"), "{asked:?}");
    assert_eq!(asked.get(1).map(String::as_str), Some("/repo"), "{asked:?}");
    assert_eq!(asked.get(2).map(String::as_str), Some("rev-parse"), "{asked:?}");
    assert_eq!(asked.get(3).map(String::as_str), Some("--path-format=absolute"), "{asked:?}");
    let mut names: Vec<&str> = Vec::new();
    for pair in asked[4..].chunks(2) {
        assert_eq!(pair.first().map(String::as_str), Some("--git-path"), "{asked:?}");
        names.push(pair.get(1).expect("a name"));
    }
    let mut expected: Vec<&str> = vec!["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply/rebasing", "rebase-apply/applying"];
    expected.push("sequencer");
    expected.extend_from_slice(RESTORED_REVISION_NAMES.as_slice());
    assert_eq!(names, expected);
}

/// A state answer of the wrong length is an error,
///  never "nothing waits".
#[test]
fn a_state_answer_of_the_wrong_length_is_an_error() {
    assert!(state_answer(b"MERGE_HEAD\n").is_err());
    assert!(state_answer(b"").is_err());
    // Six names and a trailing separator is still six.
    let six: &[u8] = b"/a/MERGE_HEAD\n/b/CHERRY_PICK_HEAD\n/c/REVERT_HEAD\n/d/rebase-merge\n/e/rebase-apply/rebasing\n/f/rebase-apply/applying\n";
    assert!(state_answer(six).is_err());
    // One name short of the full answer is still short.
    let names: Vec<&str> = asked_names();
    let short: Vec<u8> = answer_of(Path::new("/nowhere"), &names[..names.len() - 1]);
    assert!(state_answer(short.as_slice()).is_err());
}

/// A state answer of the right length becomes the state and the record paths,
///  in the order `RESTORED_REVISION_NAMES` names them.
#[test]
fn a_state_answer_of_the_right_length_becomes_the_state_and_the_records() {
    let answer: StateAnswer = state_answer(answered(Path::new("/nowhere")).as_slice()).expect("one answer per name");
    assert_eq!(answer.state, state_from_presence([false; 7]), "a path that is not there is not present");
    let mut names: Vec<&str> = Vec::new();
    for (index, (name, path)) in answer.restored_paths.iter().enumerate() {
        names.push(name);
        assert_eq!(
            path,
            &Path::new("/nowhere").join(RESTORED_REVISION_NAMES[index].replace('/', "-").as_str()),
            "the record keeps the path Git printed"
        );
    }
    assert_eq!(names, RESTORED_REVISION_NAMES.to_vec());
    let state = state_from_presence([true, false, false, false, false, false, false]);
    assert!(state.merge_head);
    assert!(!state.sequencer);
}

/// The commit an abort restores is the one Git wrote into its record.
#[test]
fn the_record_holds_the_commit_the_abort_restores() {
    let root: PathBuf = fixture("abort-record");
    let record: PathBuf = root.join("orig-head");
    let commit: &str = "0123456789abcdef0123456789abcdef01234567";
    std::fs::write(record.as_path(), format!("{commit}\n")).expect("the record");
    let answer: StateAnswer = StateAnswer {
        state: state_from_presence([false, false, false, true, false, false, false]),
        restored_paths: vec![("rebase-merge/orig-head", record.clone())],
    };
    assert_eq!(
        recorded_commit(&answer, OsStr::new("rebase-merge/orig-head")).expect("the commit"),
        OsString::from(commit)
    );
    remove(root.as_path());
}

/// A record the wrapper did not ask for,
///  a record it cannot read,
///  and a record that holds no commit id are failures,
///  never a quiet fall back to `HEAD`.
#[test]
fn an_unreadable_or_unasked_record_is_a_failure() {
    let root: PathBuf = fixture("abort-record-failures");
    let record: PathBuf = root.join("orig-head");
    std::fs::write(record.as_path(), "not a commit\n").expect("the record");
    let asked: StateAnswer = StateAnswer {
        state: state_from_presence([false; 7]),
        restored_paths: vec![("rebase-merge/orig-head", record.clone())],
    };
    assert!(recorded_commit(&asked, OsStr::new("sequencer/head")).is_err(), "a record the wrapper never asked for");
    assert!(recorded_commit(&asked, OsStr::new("rebase-merge/orig-head")).is_err(), "a value that is no commit id");
    std::fs::write(record.as_path(), "").expect("the record");
    assert!(recorded_commit(&asked, OsStr::new("rebase-merge/orig-head")).is_err(), "an empty record");
    std::fs::remove_file(record.as_path()).expect("the record");
    assert!(recorded_commit(&asked, OsStr::new("rebase-merge/orig-head")).is_err(), "a record that is not there");
    remove(root.as_path());
}

/// The merge-result queries compare the worktree and the index with `AUTO_MERGE`,
///  and only the index side drops unmerged entries.
#[test]
fn the_merge_result_queries_read_both_sides_against_the_recorded_result() {
    let queries = merge_result_queries(&[]);
    assert_eq!(
        words(queries[0].as_slice()),
        vec!["--no-optional-locks", "diff", "--name-only", "-z", "AUTO_MERGE"]
    );
    assert_eq!(
        words(queries[1].as_slice()),
        vec!["--no-optional-locks", "diff", "--cached", "--name-only", "-z", "--diff-filter=ACDMRTB", "AUTO_MERGE"]
    );
}

/// The fallback queries list every uncommitted tracked change,
///  and the index side leaves out the paths the operation is still stopped at.
#[test]
fn the_fallback_queries_list_every_uncommitted_change() {
    let queries = uncommitted_queries(&[]);
    assert_eq!(words(queries[0].as_slice()), vec!["--no-optional-locks", "diff", "--name-only", "-z"]);
    assert_eq!(
        words(queries[1].as_slice()),
        vec!["--no-optional-locks", "diff", "--cached", "--name-only", "-z", "--diff-filter=ACDMRTB"]
    );
}

/// The paths the operation is still stopped at are asked for on their own,
///  because no other filter separates the two records the worktree listing gives each of them.
#[test]
fn the_unresolved_query_asks_for_exactly_the_unmerged_entries() {
    assert_eq!(
        words(unresolved_query(&[]).as_slice()),
        vec!["--no-optional-locks", "diff", "--name-only", "-z", "--diff-filter=U"]
    );
}

/// Two listings become one set of paths,
///  each once and in one order.
#[test]
fn two_listings_become_one_set() {
    let both: Vec<Vec<u8>> = both_listings(
        vec![b"b.txt".to_vec(), b"a.txt".to_vec()],
        vec![b"a.txt".to_vec(), b"c.txt".to_vec()],
    );
    assert_eq!(both, vec![b"a.txt".to_vec(), b"b.txt".to_vec(), b"c.txt".to_vec()]);
    assert!(both_listings(Vec::new(), Vec::new()).is_empty());
}

/// A record that is not a regular file,
///  and one longer than any commit id,
///  are read in bounded time and refused with their own words.
///  A pipe is what a `mkfifo` leaves at the name,
///  and reading it by its own length never returns.
#[test]
fn a_record_that_is_no_regular_file_is_refused_without_waiting() {
    let root: PathBuf = fixture("abort-record-kinds");
    let pipe: PathBuf = root.join("orig-head");
    let made: std::process::ExitStatus = std::process::Command::new("mkfifo").arg(pipe.as_path()).status().expect("mkfifo");
    assert!(made.success(), "mkfifo must create {}", pipe.display());
    let read: Result<String, String> = read_the_record(pipe.as_path());
    assert!(read.is_err(), "a pipe is not a record: {read:?}");
    assert!(read.unwrap_err().contains("not a regular file"), "the refusal says what stands there");
    // A directory at the name, which is not a record either.
    let directory: PathBuf = root.join("rebase-merge");
    std::fs::create_dir(directory.as_path()).expect("a directory");
    assert!(read_the_record(directory.as_path()).is_err(), "a directory is not a record");
    // A record longer than any commit id is refused rather than read whole.
    let long: PathBuf = root.join("long-head");
    let mut text: String = String::with_capacity(usize::try_from(RECORD_BYTES_READ).unwrap_or(0) + 1);
    for _ in 0..=RECORD_BYTES_READ {
        text.push('a');
    }
    std::fs::write(long.as_path(), text.as_str()).expect("the long record");
    let too_long: Result<String, String> = read_the_record(long.as_path());
    assert!(too_long.is_err(), "a record longer than {RECORD_BYTES_READ} bytes holds no commit id: {too_long:?}");
    assert!(too_long.unwrap_err().contains("holds more than"), "the refusal says the file is too long");
    remove(root.as_path());
}

/// A form that aborts no operation asks nothing.
#[test]
fn a_form_that_aborts_no_operation_asks_nothing() {
    let mut facts = crate::repository_facts::git_facts(Path::new(crate::test_support::REAL_GIT), &[], &[]);
    let answer = crate::repository_facts_abort_discard::abort_discard(&mut facts, GuardedCommand::Stash).expect("nothing is asked");
    assert_eq!(answer, AbortDiscard::NothingWaiting);
    assert_eq!(facts.queries, 0, "no Git starts");
}

/// What stands at a path decides whether the path is work the command would discard:
///  nothing at all is not, and a directory, a link or a pipe is.
#[test]
fn only_a_path_the_worktree_still_holds_is_work() {
    let root: PathBuf = fixture("abort-entry-here");
    let file: PathBuf = root.join("file.txt");
    std::fs::write(file.as_path(), b"content\n").expect("a file");
    let directory: PathBuf = root.join("directory");
    std::fs::create_dir(directory.as_path()).expect("a directory");
    let link: PathBuf = root.join("link.txt");
    std::os::unix::fs::symlink(file.as_path(), link.as_path()).expect("a link");
    let pipe: PathBuf = root.join("pipe.txt");
    let made: std::process::ExitStatus = std::process::Command::new("mkfifo").arg(pipe.as_path()).status().expect("mkfifo runs");
    assert!(made.success(), "mkfifo must create {}", pipe.display());
    assert!(holds_an_entry(Some(root.as_path()), b"file.txt"), "a file is work the reset would replace");
    assert!(holds_an_entry(Some(root.as_path()), b"directory"), "a directory is work the reset would need");
    assert!(holds_an_entry(Some(root.as_path()), b"link.txt"), "a link is work the reset would replace");
    assert!(holds_an_entry(Some(root.as_path()), b"pipe.txt"), "a pipe is an entry the reset would replace");
    assert!(!holds_an_entry(Some(root.as_path()), b"gone.txt"), "nothing at all is nothing to lose");
    assert!(!holds_an_entry(Some(root.as_path()), b"below/gone.txt"), "nothing at all below the top either");
    assert!(holds_an_entry(None, b"gone.txt"), "a worktree Git did not name refuses rather than lets a loss through");
    remove(root.as_path());
}
