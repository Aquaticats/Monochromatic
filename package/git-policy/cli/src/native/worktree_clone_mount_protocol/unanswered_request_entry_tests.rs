//! What:
//!  Controls of one entry of the folder of unanswered requests:
//!  what a wait writes,
//!  every outcome word a helper records,
//!  and each field a reader refuses.
//! Why:
//!  The entry is what tells a later command that a clone still waits,
//!  so a field this route would misread is a trigger lost or a request acted
//!  on twice
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "The record:
//! one file per clone").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(parseEntry(entryText(entry))).toEqual(entry);
//! ```

use super::{
    EntryRefusal,
    MAXIMUM_ENTRY_BYTES,
    RequestEntry,
    RequestOutcome,
    parse_entry,
};
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_property_names::NONCE_HEX_DIGITS;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::MAXIMUM_DATASET_NAME_BYTES;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// The GUID every control writes.
const GUID: u64 = 1234567890123456789;

/// The stamp every control writes,
/// which is the shape the routes' own records carry.
const WRITTEN_AT: &str = "2026-11-27T21:14:14.000Z";

/// The dataset every control writes.
const DATASET: &str = "zpcachyos/nosnap/monochromatic-git-worktree-clone-datasets/one";

/// One entry a wait writes.
fn waiting() -> RequestEntry {
    return RequestEntry {
        guid: DatasetGuid::parse(GUID.to_string().as_str()).expect("the control GUID is canonical"),
        nonce: RequestNonce::parse("0123456789abcdef".repeat(2).as_str())
            .expect("the control nonce is 32 hexadecimal digits"),
        operation: MountOperation::Mount,
        written_at: String::from(WRITTEN_AT),
        outcome: RequestOutcome::Waiting,
        dataset: String::from(DATASET),
    };
}

/// A wait's entry reads back as itself,
/// and its text is the one line the design gives.
#[test]
fn a_wait_s_entry_reads_back_as_itself() {
    let entry: RequestEntry = waiting();
    let text: String = entry.text();
    assert_eq!(
        text,
        format!(
            "v1 {GUID} {} mount {WRITTEN_AT} waiting {DATASET}\n",
            "0123456789abcdef".repeat(2)
        )
    );
    assert_eq!(parse_entry(text.as_str()), Ok(entry));
}

/// Every outcome word the route writes reads back,
/// so a helper's record is never read as another attempt's state.
#[test]
fn every_outcome_word_reads_back() {
    for outcome in [
        RequestOutcome::Waiting,
        RequestOutcome::Claimed,
        RequestOutcome::Nudged,
        RequestOutcome::RefusedGate,
        RequestOutcome::RefusedDevice,
        RequestOutcome::CloneGone,
    ] {
        let written: RequestEntry = RequestEntry {
            outcome,
            ..waiting()
        };
        assert_eq!(
            parse_entry(written.text().as_str()).map(|read| return read.outcome),
            Ok(outcome)
        );
        assert_eq!(RequestOutcome::from_word(outcome.word()), Some(outcome));
    }
}

/// A dataset name with a space in it survives the round trip,
/// because the line is split at its first six separators and not at seven.
#[test]
fn a_dataset_name_with_a_space_survives_the_round_trip() {
    let written: RequestEntry = RequestEntry {
        dataset: String::from("zpcachyos/nosnap/two clones/one"),
        ..waiting()
    };
    assert_eq!(parse_entry(written.text().as_str()), Ok(written));
}

/// A file longer than one page is refused before any field is split.
#[test]
fn a_file_longer_than_one_page_is_refused() {
    let padded: String = format!(
        "{} {}",
        waiting().text().trim_end(),
        "x".repeat(MAXIMUM_ENTRY_BYTES)
    );
    let refusal: EntryRefusal = parse_entry(padded.as_str()).expect_err("a page is the bound");
    assert!(
        matches!(refusal, EntryRefusal::TooLong { .. }),
        "{refusal:?}"
    );
    assert!(refusal.message().contains(&MAXIMUM_ENTRY_BYTES.to_string()));
}

/// A line of another shape is refused,
/// and each refusal names the field and what to write.
#[test]
fn a_line_of_another_shape_is_refused() {
    let good: String = waiting().text();
    let line: &str = good.trim_end();
    let named: Vec<(String, EntryRefusal)> = vec![
        (
            String::from("v1 42.1"),
            EntryRefusal::Fields {
                found: 2,
            },
        ),
        (
            line.replace("v1 ", "v2 "),
            EntryRefusal::Version {
                found: String::from("v2"),
            },
        ),
        (
            line.replace(GUID.to_string().as_str(), "042"),
            EntryRefusal::Guid(
                crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuidRefusal::LeadingZero {
                    value: String::from("042"),
                },
            ),
        ),
        (
            line.replace(&"0123456789abcdef".repeat(2), "ABCDEF0123456789abcdef0123456789"),
            EntryRefusal::Nonce(
                crate::worktree_clone_mount_protocol::request_nonce::NonceRefusal::NotLowercaseHex {
                    place: 1,
                    shown: String::from("A"),
                },
            ),
        ),
        (
            line.replace(" mount ", " detach "),
            EntryRefusal::Operation {
                found: String::from("detach"),
            },
        ),
        (
            line.replace(WRITTEN_AT, "yesterday"),
            EntryRefusal::Time {
                found: String::from("yesterday"),
            },
        ),
        (
            line.replace(" waiting ", " pending "),
            EntryRefusal::Outcome {
                found: String::from("pending"),
            },
        ),
        (
            format!("{} ", line.trim_end_matches(DATASET).trim_end()),
            EntryRefusal::Dataset {
                found: 0,
            },
        ),
        (
            format!(
                "{}{}",
                line.trim_end_matches(DATASET),
                "d".repeat(MAXIMUM_DATASET_NAME_BYTES + 1)
            ),
            EntryRefusal::Dataset {
                found: MAXIMUM_DATASET_NAME_BYTES + 1,
            },
        ),
    ];
    for (text, expected) in named {
        let refusal: EntryRefusal =
            parse_entry(text.as_str()).expect_err("the line is not an entry");
        assert_eq!(refusal, expected, "for `{text}`");
        assert!(
            refusal
                .message()
                .contains("Nothing was read as an unanswered request"),
            "{}",
            refusal.message()
        );
    }
}

/// A file that holds two lines is refused rather than read as its first,
/// so one name cannot hold two attempts.
#[test]
fn a_file_that_holds_two_lines_is_refused() {
    let good: String = waiting().text();
    let twice: String = format!("{good}{good}");
    assert!(matches!(
        parse_entry(twice.as_str()),
        Err(EntryRefusal::Fields { .. })
    ));
}

/// A nonce of the wrong length is refused with the length it held,
/// which is the one refusal that would otherwise read as a short entry.
#[test]
fn a_short_nonce_is_refused() {
    let good: String = waiting().text();
    let short: String = good.replace(&"0123456789abcdef".repeat(2), "0123");
    let refusal: EntryRefusal = parse_entry(short.as_str()).expect_err("four digits are no nonce");
    assert_eq!(
        refusal,
        EntryRefusal::Nonce(
            crate::worktree_clone_mount_protocol::request_nonce::NonceRefusal::Length {
                found: 4,
            }
        )
    );
    assert_eq!(NONCE_HEX_DIGITS, 32);
}
