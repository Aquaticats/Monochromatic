//! Controls for the questions an isolated merge asks Git:
//! the one head a `FETCH_HEAD` marks for merging,
//! and a `FETCH_HEAD` that is a named pipe instead,
//! which must answer at once.
//! Why:
//!  A pull or a merge that reads `FETCH_HEAD` with a call that waits is held for ever by a pipe at that name,
//!  after `git fetch` has already written it (`crate::plain_file`;
//! issue 6415 for the same shape elsewhere).

/// The rule under test.
use super::read_fetch_head;
/// Fixture folders,
/// their removal and a bound on the wait.
use crate::test_support::{
    finished_within,
    fixture,
    remove,
};
use std::path::PathBuf;

/// A line marked for merging is one head,
/// and the file's own text comes back with it,
/// because the merge message is worded from all of it.
#[test]
fn one_merge_head_comes_back_with_the_files_text() {
    let root: PathBuf = fixture("isolation-fetch-head-one");
    let line: &str = "1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b\t\tbranch 'main' of somewhere\n";
    std::fs::write(root.join("FETCH_HEAD"), line).expect("the fetched head");
    let Ok((head, text)) = read_fetch_head(root.as_path()) else {
        panic!("one line marked for merging is one head to merge")
    };
    assert_eq!(
        head,
        String::from("1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b")
    );
    assert_eq!(
        text,
        String::from(line),
        "the merge message is worded from the whole file"
    );
    remove(root.as_path());
}

/// A `FETCH_HEAD` that marks several heads is left to native Git,
/// which words that refusal itself.
#[test]
fn several_merge_heads_are_left_to_native_git() {
    let root: PathBuf = fixture("isolation-fetch-head-several");
    let two: &str = "1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b\t\tbranch 'a' of somewhere\n2222222222222222222222222222222222222222\t\tbranch 'b' of somewhere\n";
    std::fs::write(root.join("FETCH_HEAD"), two).expect("the fetched heads");
    assert!(
        read_fetch_head(root.as_path()).is_err(),
        "two heads are native Git's to refuse"
    );
    remove(root.as_path());
}

/// A `FETCH_HEAD` that is a named pipe sends the merge to native Git at once:
/// opening a named pipe for reading waits for a writer that may never come,
/// which held every pull in such a repository before the file was read as a plain one.
#[test]
fn a_fetched_head_that_is_a_named_pipe_goes_to_native_git_at_once() {
    let root: PathBuf = fixture("isolation-fetch-head-named-pipe");
    crate::subvolume_sync_attrs::make_fifo(root.join("FETCH_HEAD").as_path(), 0o600)
        .expect("a named pipe where a fetched head goes");
    let inside: PathBuf = root.clone();
    let read = finished_within(move || return read_fetch_head(inside.as_path()));
    let Some(answer) = read else {
        panic!("a `FETCH_HEAD` that is a named pipe held the merge")
    };
    assert!(
        answer.is_err(),
        "a pipe is no list of heads to merge here, so native Git runs the pull"
    );
    // The read this module made before, on the same fixture under the same bound, so the bound above is shown to see a wait.
    let fetched: PathBuf = root.join("FETCH_HEAD");
    let waited = finished_within(move || return std::fs::read_to_string(fetched).ok());
    assert!(
        waited.is_none(),
        "a plain read of the same name waits, so the bound above can see a wait"
    );
    remove(root.as_path());
}
