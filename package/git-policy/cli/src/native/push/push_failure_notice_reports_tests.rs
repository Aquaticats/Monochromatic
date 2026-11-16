//! Controls for the record of what a push that succeeded did:
//! the lines it keeps,
//! and what a replacement leaves when its write is refused.
//! Why:
//!  The record is the only copy of what a server and a reconcile said about a push the remote accepted,
//! and nothing can make an accepted push say it again.

use super::*;
use crate::private_storage::AFTER_THE_STAGING_FILE_IS_WRITTEN;
use crate::push_failure_notice::{LONGEST_REASON, push_failures};
use crate::test_support::{fixture, remove};

/// A commit name for the records of these controls.
const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

/// The branch the records of these controls belong to.
const BRANCH: &str = "refs/heads/main";

/// The line a cut record ends with.
const CUT_MARKER: &str = "(Git's output is cut here.)";

/// What a server's line looks like in a push's output.
const REMOTE_PREFIX: &str = "remote: ";

/// The words of every record,
/// shown to a buffer and removed as a command shows and removes them.
fn shown_in(root: &Path) -> Vec<String> {
    return crate::test_support::shown_notices(push_failures(root));
}

/// What:
///  The account of an accepted push keeps every line the push chose to show,
/// the last one included.
/// Why:
///   A server's last line is often the one a person acts on:
/// a link to approve a deployment,
/// the branch a hook moved,
/// or the reference a hook refused.
/// An accepted push is never repeated,
/// so a line cut from this record is a line nobody can read again
/// (`SPEC.md`,
/// "Reconcile and bounds").
#[test]
fn an_accepted_pushs_account_keeps_every_line_it_was_given() {
    const LAST_LINE: &str = "remote: THE SERVERS LAST WORD, WHICH A PERSON ACTS ON";
    // One line more than the failure notice's display cap holds, so a cut shows here and not only in a much longer push.
    let filler: String = format!("{REMOTE_PREFIX}{}\n", "x".repeat(LONGEST_REASON / 8));
    let mut said: String = String::new();
    while said.len() <= LONGEST_REASON {
        said.push_str(filler.as_str());
    }
    said.push_str(LAST_LINE);
    assert!(said.len() > LONGEST_REASON, "the account is longer than the cap this control defends against");
    let root: PathBuf = fixture("push-report-every-line");
    record_push_report(root.as_path(), BRANCH, COMMIT, said.as_str()).expect("the record of an accepted push");
    let shown: Vec<String> = shown_in(root.as_path());
    assert_eq!(shown.len(), 1, "one record: {shown:?}");
    let notice: &str = shown[0].as_str();
    assert!(notice.contains(LAST_LINE), "the server's last line is missing from the only record of the push: {notice}");
    assert!(!notice.contains(CUT_MARKER), "the account was cut: {notice}");
    remove(root.as_path());
}

/// What:
///  A report whose replacement cannot be written leaves the record it would have replaced.
/// Why:
///   A second try of the same push writes its record again,
/// and a kill or a full disk in that write must leave one of the two records:
/// the record name is the only place the person's account of that commit is,
/// and a record removed before the new one is published leaves them nothing,
/// while a push of what the remote already holds never says it again.
#[test]
fn a_report_whose_replacement_is_refused_leaves_the_record_it_replaces() {
    const FIRST: &str = "remote: THE ACCOUNT NOBODY HAS READ YET";
    const SECOND: &str = "remote: THE ACCOUNT THAT COULD NOT BE WRITTEN";
    let root: PathBuf = fixture("push-report-replacement-refused");
    record_push_report(root.as_path(), BRANCH, COMMIT, FIRST).expect("the first record");
    AFTER_THE_STAGING_FILE_IS_WRITTEN.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(|| { return Err(std::io::Error::other("the disk filled")); }));
    });
    let refused: Result<(), String> = record_push_report(root.as_path(), BRANCH, COMMIT, SECOND);
    AFTER_THE_STAGING_FILE_IS_WRITTEN.with(|hook| { *hook.borrow_mut() = None; });
    refused.expect_err("a write the disk refused");
    let shown: Vec<String> = shown_in(root.as_path());
    assert_eq!(shown.len(), 1, "the record that stood there is gone: {shown:?}");
    assert!(shown[0].contains(FIRST), "the words nobody had read were kept: {:?}", shown[0]);
    assert!(!shown[0].contains(SECOND), "the refused words are not the record: {:?}", shown[0]);
    remove(root.as_path());
}
