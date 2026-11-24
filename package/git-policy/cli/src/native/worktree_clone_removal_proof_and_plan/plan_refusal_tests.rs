//! What:
//!  The controls of the refusal a reader prints:
//!  the file,
//!  the field and the value,
//!  and the promise that nothing was destroyed.
//! Why:
//!  The project decided an unreadable record is refused,
//!  named and left for a person,
//!  so a message that omitted the file or the value would leave that person with nothing to read
//!  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
//!  "A record that cannot be read").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(new PlanRefusal(...).message(path)).toContain('Nothing was destroyed');
//! ```

use super::{PlanRefusal, PlanRefusalCause};
use std::path::Path;

#[test]
fn a_refusal_names_the_file_the_field_the_value_and_that_nothing_was_destroyed() {
    let path: &Path = Path::new("/repo/.git/monochromatic-git-policy-cli/clone-destruction-plans/7");
    let refusal: PlanRefusal = PlanRefusal::at(PlanRefusalCause::NotCanonical, "cloneGuid", "007");
    let message: String = refusal.message(path);
    assert!(message.contains(path.to_str().expect("the path is text")), "the message names the file: {message}");
    assert!(message.contains("cloneGuid"), "the message names the field: {message}");
    assert!(message.contains("`007`"), "the message names the value it read: {message}");
    assert!(message.contains("Nothing was destroyed by this refusal"), "the message says nothing was destroyed: {message}");
    assert!(message.contains("neither moved nor deleted"), "the message says the record stays: {message}");
    assert!(message.contains("Ways forward:"), "the message names the way forward: {message}");
}

#[test]
fn a_refusal_that_names_no_field_still_says_which_file_and_what_it_read() {
    let path: &Path = Path::new("/repo/.git/monochromatic-git-policy-cli/clone-destruction-plans/7");
    let message: String = PlanRefusal::of(PlanRefusalCause::Oversized, "12000 bytes").message(path);
    assert!(message.contains("in its bytes"), "the message says where it looked: {message}");
    assert!(message.contains("larger than its declared attempt count allows"), "the message says what it found: {message}");
    assert!(message.contains("12000 bytes"), "the message says what it read: {message}");
}

#[test]
fn each_cause_prints_its_own_clause_so_two_causes_never_read_alike() {
    let path: &Path = Path::new("/plan");
    let causes: [PlanRefusalCause; 26] = [
        PlanRefusalCause::NotThisRecord,
        PlanRefusalCause::UnknownVersion,
        PlanRefusalCause::EndsEarly,
        PlanRefusalCause::MissingAttemptLines,
        PlanRefusalCause::UnknownKey,
        PlanRefusalCause::OutOfOrderKey,
        PlanRefusalCause::RepeatedKey,
        PlanRefusalCause::MissingKey,
        PlanRefusalCause::NotCanonical,
        PlanRefusalCause::CountDisagrees,
        PlanRefusalCause::WrongAttemptKind,
        PlanRefusalCause::UnknownOutcome,
        PlanRefusalCause::UnknownHead,
        PlanRefusalCause::DuplicateAttempt,
        PlanRefusalCause::AttemptsOutOfOrder,
        PlanRefusalCause::DestroyedHistoryReopened,
        PlanRefusalCause::LineTooWide,
        PlanRefusalCause::Oversized,
        PlanRefusalCause::ForeignName,
        PlanRefusalCause::NameDisagreesWithTheRecord,
        PlanRefusalCause::NotARegularFile,
        PlanRefusalCause::Unreadable,
        PlanRefusalCause::FolderMode,
        PlanRefusalCause::WrongOwner,
        PlanRefusalCause::FieldTooWide,
        PlanRefusalCause::WriterWouldNotReadItBack,
    ];
    let mut seen: Vec<String> = Vec::new();
    for cause in causes {
        let message: String = PlanRefusal::of(cause, "x").message(path);
        assert!(!seen.contains(&message), "two causes print one sentence: {message}");
        seen.push(message);
    }
    assert_eq!(seen.len(), causes.len(), "every cause printed its own sentence");
}
