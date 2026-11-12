//! Controls for the words a commit reads out of its own command line.

use super::arguments_after_subcommand;
use crate::service_commit_input_notes_and_embedded_commit::switched_branch_name;
use std::ffi::OsString;


/// The command's own words start after the subcommand,
/// whatever stands before it,
/// and a command line that ends at the
/// subcommand,
/// or at which the subcommand is not,
/// has none.
#[test]
fn the_words_of_the_command_start_after_the_subcommand() {
    let line: Vec<OsString> = ["-C", "somewhere", "commit", "-m", "message", "--", "file"].iter().map(OsString::from).collect();
    let after: Vec<OsString> = ["-m", "message", "--", "file"].iter().map(OsString::from).collect();
    assert_eq!(arguments_after_subcommand(line.as_slice(), 2), after.as_slice());
    assert_eq!(arguments_after_subcommand(line.as_slice(), 6), &[] as &[OsString], "the subcommand is the last word");
    assert_eq!(arguments_after_subcommand(line.as_slice(), 40), &[] as &[OsString], "no such word");
    let bare: Vec<OsString> = vec![OsString::from("commit"), OsString::from("-a")];
    assert_eq!(arguments_after_subcommand(bare.as_slice(), 0), &bare[1..], "a subcommand in first place leaves everything after it, and not itself");
}

/// A branch that switched under a commit is named as the person knows it:
/// a detached head by those words,
/// a branch without the
/// `refs/heads/` of its reference,
/// and any other reference as it is.
#[test]
fn a_switched_branch_is_named_as_the_person_knows_it() {
    assert_eq!(switched_branch_name(&crate::ref_name::RefName::from_text("HEAD")), "a detached head");
    assert_eq!(switched_branch_name(&crate::ref_name::RefName::from_text("refs/heads/main")), "main");
    assert_eq!(switched_branch_name(&crate::ref_name::RefName::from_text("refs/heads/topic/deep")), "topic/deep");
    assert_eq!(switched_branch_name(&crate::ref_name::RefName::from_text("refs/tags/v1")), "refs/tags/v1");
    assert_ne!(switched_branch_name(&crate::ref_name::RefName::from_text("HEADS")), "a detached head", "only the word HEAD is a detached head");
}
