//! What:
//!  The record's words and bounds:
//!  every outcome word and every step word reads back as itself,
//!  no word is longer than the bound its field carries,
//!  and the file bound covers a record with every field at its longest and
//!  four paths named.
//! Why:
//!  The record's writer proves its own parser reads the text back before a
//!  byte is written,
//!  so a word longer than its field's bound would make every record
//!  unwritable,
//!  and the bound that matters is the one the writer's own fields need
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "The propagation record",
//!  which states each bound and derives the file's).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('every step word fits its field', () => expect(STEP_WORDS.every(w => w.length <= MAXIMUM_STEP_BYTES)).toBe(true));
//! ```

use super::*;
use crate::checkout_propagation::mount_propagation_set::SHARED_AND_SLAVE_WORD;

/// The four outcome words,
/// in the order the record lists them.
const OUTCOMES: [PropagationOutcome; 4] = [
    PropagationOutcome::Applied,
    PropagationOutcome::AlreadyPrivate,
    PropagationOutcome::Left,
    PropagationOutcome::Refused,
];

/// The words the `step` field can carry,
/// which is every act the step makes.
const STEPS: [PropagationStep; 12] = [
    PropagationStep::NoStep,
    PropagationStep::Resolve,
    PropagationStep::Open,
    PropagationStep::Statmount,
    PropagationStep::Point,
    PropagationStep::Type,
    PropagationStep::Source,
    PropagationStep::Namespace,
    PropagationStep::BeforeTable,
    PropagationStep::Set,
    PropagationStep::VerifyAfter,
    PropagationStep::Record,
];

#[test]
fn every_outcome_word_reads_back_as_itself() {
    for outcome in OUTCOMES {
        assert_eq!(
            PropagationOutcome::from_word(outcome.word()),
            Some(outcome),
            "the outcome word {} reads back",
            outcome.word()
        );
        assert!(
            outcome.word().len() <= MAXIMUM_OUTCOME_BYTES,
            "the outcome word {} fits its field's bound of {MAXIMUM_OUTCOME_BYTES}",
            outcome.word()
        );
    }
    assert_eq!(PropagationOutcome::from_word("repaired"), None);
}

#[test]
fn every_step_word_reads_back_as_itself_and_fits_its_bound() {
    for step in STEPS {
        assert_eq!(
            PropagationStep::from_word(step.word()),
            Some(step),
            "the step word {} reads back",
            step.word()
        );
        assert!(
            step.word().len() <= MAXIMUM_STEP_BYTES,
            "the step word {} fits its field's bound of {MAXIMUM_STEP_BYTES}",
            step.word()
        );
    }
    assert_eq!(PropagationStep::from_word("wandering"), None);
    assert_eq!(
        PropagationStep::from_word(NONE_WORD),
        Some(PropagationStep::NoStep),
        "the word a successful run carries is the one `NoStep` names"
    );
}

#[test]
fn the_longest_set_word_is_the_bound_the_record_uses() {
    assert_eq!(LONGEST_WORD, SHARED_AND_SLAVE_WORD);
    assert_eq!(MAXIMUM_SET_WORD_BYTES, LONGEST_WORD.len());
}

#[test]
fn the_file_bound_covers_a_record_with_every_field_at_its_longest() {
    // One field at each bound, four paths at the path bound, and every key: nothing a writer builds may exceed it.
    let longest: usize = MAXIMUM_PROPAGATION_BYTES;
    let needed: usize = "v1".len()
        + 1
        + "kind".len()
        + 1
        + PROPAGATION_KIND.len()
        + 1
        + "boot".len()
        + 1
        + MAXIMUM_BOOT_BYTES
        + 1
        + "mount-id".len()
        + 1
        + MAXIMUM_NUMBER_BYTES
        + 1
        + "mount-id-old".len()
        + 1
        + MAXIMUM_NUMBER_BYTES
        + 1
        + "mount-ns-id".len()
        + 1
        + MAXIMUM_NUMBER_BYTES
        + 1
        + "checkout".len()
        + 1
        + MAXIMUM_CHECKOUT_PATH_BYTES
        + 1
        + "source".len()
        + 1
        + MAXIMUM_DATASET_BYTES
        + 1
        + "before".len()
        + 1
        + MAXIMUM_SET_WORD_BYTES
        + 1
        + "after".len()
        + 1
        + MAXIMUM_SET_WORD_BYTES
        + 1
        + "master".len()
        + 1
        + MAXIMUM_NUMBER_BYTES
        + 1
        + "beneath".len()
        + 1
        + MAXIMUM_NUMBER_BYTES
        + 1
        + "outcome".len()
        + 1
        + MAXIMUM_OUTCOME_BYTES
        + 1
        + "step".len()
        + 1
        + MAXIMUM_STEP_BYTES
        + 1
        + "errno".len()
        + 1
        + MAXIMUM_ERRNO_BYTES
        + 1
        + "time".len()
        + 1
        + MAXIMUM_TIME_BYTES
        + 1
        + SHARED_MOUNTS_NAMED * (BENEATH_PATH_KEY.len() + 1 + MAXIMUM_BENEATH_PATH_BYTES + 1);
    assert_eq!(
        longest, needed,
        "the file bound is exactly what a record with every field at its bound takes"
    );
    assert_eq!(
        MAXIMUM_BENEATH_PATH_BYTES,
        4 * MAXIMUM_CHECKOUT_PATH_BYTES,
        "a path escaped at the kernel's worst four bytes each fits its line's bound"
    );
}

#[test]
fn the_record_names_one_directory_under_run_and_one_file_in_it() {
    assert!(
        PROPAGATION_DIRECTORY.starts_with("/run/"),
        "the record lives under `/run`, which every boot empties"
    );
    assert_eq!(PROPAGATION_NAME, "result");
    assert_eq!(PROPAGATION_DIRECTORY_MODE, 0o755);
    assert_eq!(PROPAGATION_MODE, 0o644);
    assert_eq!(PROPAGATION_VERSION, "v1");
    assert_eq!(PROPAGATION_KIND, "checkout-propagation");
    assert_eq!(
        SHARED_MOUNTS_NAMED, 4,
        "the record names four paths and counts the rest"
    );
    assert_eq!(BENEATH_UNREAD, "unread");
    assert_eq!(NONE_WORD, "none");
}
