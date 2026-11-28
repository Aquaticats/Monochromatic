//! What:
//!  The record's text and its parser:
//!  a full answer read back as the record it was written from,
//!  a refusal's record with `none` in every field it never reached and
//!  `unread` for the mounts beneath,
//!  and each malformed shape the parser refuses.
//! Why:
//!  The writer proves its own parser reads the text back before a byte is
//!  written,
//!  so the two must agree on every field and on every word,
//!  and a reader must refuse a half-written file,
//!  another build's file,
//!  and a word this build does not know
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "The propagation record",
//!  and its test 11).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(parseRecord(record.text())).toEqual(record);
//! expect(() => parseRecord(text.replace('kind checkout-propagation', 'kind clone-mount-readiness'))).toThrow();
//! ```

use super::*;
use crate::checkout_propagation::mount_propagation_set::MountPropagation;

/// What:
///  One answer a step reached,
///  with every field the writer fills.
/// Why:
///  A control that changes one field must not change another,
///  so the whole record is stated once and each malformed shape takes its
///  text and breaks one line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const applied = (): PropagationRecord => ({ boot: 'b0ot', mountId: 9001, ... });
/// ```
fn applied() -> PropagationRecord {
    return PropagationRecord {
        boot: String::from("6f1c2b8a-2b9f-4b4d-9a1f-0e5e6f7a8b9c"),
        mount_id: Some(9001),
        mount_id_old: Some(41),
        mount_namespace_id: Some(4_022_345_678),
        checkout: Some(String::from("/home/user/Monochromatic")),
        source: Some(String::from("pool/home/monochromatic-git-checkout")),
        before: Some(MountPropagation::of_the_kernel_set(libc::MS_SHARED).expect("shared")),
        after: Some(MountPropagation::of_the_kernel_set(libc::MS_PRIVATE).expect("private")),
        master: None,
        beneath: SharedMountsBeneath::Counted {
            count: 1,
            paths: vec![String::from(
                "/home/user/Monochromatic/package/vm-builder/cli/output",
            )],
        },
        outcome: PropagationOutcome::Applied,
        step: PropagationStep::NoStep,
        errno: None,
        time: String::from("2026-11-27T21:00:00.000-05:00"),
    };
}

/// What:
///  One refusal a run filed before it read the mount.
/// Why:
///  The refusal's record is the other shape the writer makes,
///  and every field it never reached must read `none`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = (): PropagationRecord => ({ boot: 'b0ot', mountId: undefined, ..., beneath: { kind: 'unread' }, outcome: 'refused', step: 'open', errno: 2 });
/// ```
fn refused() -> PropagationRecord {
    return PropagationRecord {
        boot: String::from("6f1c2b8a-2b9f-4b4d-9a1f-0e5e6f7a8b9c"),
        mount_id: None,
        mount_id_old: None,
        mount_namespace_id: None,
        checkout: None,
        source: None,
        before: None,
        after: None,
        master: None,
        beneath: SharedMountsBeneath::Unread,
        outcome: PropagationOutcome::Refused,
        step: PropagationStep::Open,
        errno: Some(libc::ENOENT),
        time: String::from("2026-11-27T21:00:00.000-05:00"),
    };
}

#[test]
fn a_full_answer_reads_back_as_the_record_it_was_written_from() {
    let record: PropagationRecord = applied();
    let text: String = record.text();
    assert_eq!(
        parse_record(text.as_str()).expect("the writer's own text"),
        record,
        "the text and the record are one answer"
    );
    assert!(text.starts_with("v1\nkind checkout-propagation\n"));
    assert!(text.ends_with('\n'), "the file ends with one newline");
}

#[test]
fn a_refusals_record_carries_none_where_it_never_looked_and_unread_for_the_mounts_beneath() {
    let record: PropagationRecord = refused();
    let text: String = record.text();
    assert!(text.contains("\nmount-id none\n"), "the text: {text}");
    assert!(text.contains("\nbeneath unread\n"), "the text: {text}");
    assert!(text.contains("\nstep open\n"));
    assert!(text.contains("\nerrno 2\n"));
    assert_eq!(
        parse_record(text.as_str()).expect("a refusal's text"),
        record
    );
}

#[test]
fn the_paths_follow_the_count_and_a_reader_reads_them_back() {
    let mut record: PropagationRecord = applied();
    record.beneath = SharedMountsBeneath::Counted {
        count: 5,
        paths: vec![
            String::from("/home/user/Monochromatic/a"),
            String::from("/home/user/Monochromatic/b"),
            String::from("/home/user/Monochromatic/c"),
            String::from("/home/user/Monochromatic/d"),
        ],
    };
    let text: String = record.text();
    assert_eq!(
        text.matches("\nbeneath-path ").count(),
        4,
        "the record names four paths and counts five: {text}"
    );
    assert_eq!(parse_record(text.as_str()).expect("five counted"), record);
}

#[test]
fn a_missing_key_is_refused() {
    let record: PropagationRecord = applied();
    let text: String = record.text().replace("mount-ns-id 4022345678\n", "");
    let refusal: PropagationParseRefusal =
        parse_record(text.as_str()).expect_err("no mount-ns-id line");
    assert_eq!(refusal.kind, "key");
    assert!(refusal.message().contains("mount-ns-id"));
}

#[test]
fn a_version_and_a_kind_this_build_does_not_write_are_refused() {
    let record: PropagationRecord = applied();
    let version: String = record.text().replacen("v1\n", "v2\n", 1);
    let kind: String = record
        .text()
        .replace("kind checkout-propagation", "kind clone-mount-readiness");
    assert_eq!(
        parse_record(version.as_str()).expect_err("v2").kind,
        "version"
    );
    assert_eq!(
        parse_record(kind.as_str()).expect_err("another kind").kind,
        "kind"
    );
}

#[test]
fn an_over_long_field_is_refused_rather_than_written() {
    let mut record: PropagationRecord = applied();
    record.checkout = Some("x".repeat(MAXIMUM_CHECKOUT_PATH_BYTES + 1));
    let refusal: PropagationParseRefusal =
        parse_record(record.text().as_str()).expect_err("an over-long path");
    assert_eq!(refusal.kind, "too-long");
    assert!(refusal.message().contains("checkout"));
}

#[test]
fn an_unknown_word_in_a_set_an_outcome_or_a_step_is_refused() {
    let record: PropagationRecord = applied();
    let text: String = record.text();
    for (from, to) in [
        ("\nbefore shared\n", "\nbefore passing\n"),
        ("\noutcome applied\n", "\noutcome repaired\n"),
        ("\nstep none\n", "\nstep wandering\n"),
    ] {
        let broken: String = text.replace(from, to);
        let refusal: PropagationParseRefusal =
            parse_record(broken.as_str()).expect_err("a word this build does not write");
        assert_eq!(refusal.kind, "word", "the word {to} is refused as a word");
    }
}

#[test]
fn a_count_below_the_paths_it_names_is_refused() {
    let record: PropagationRecord = applied();
    let broken: String = record.text().replace("\nbeneath 1\n", "\nbeneath 0\n");
    let refusal: PropagationParseRefusal =
        parse_record(broken.as_str()).expect_err("a wrong count");
    assert_eq!(refusal.kind, "number");
    assert!(
        refusal.message().contains("count"),
        "the refusal says the count is wrong: {}",
        refusal.message()
    );
}

#[test]
fn an_empty_text_and_a_text_of_another_shape_are_refused() {
    assert_eq!(parse_record("").expect_err("empty").kind, "not-a-record");
    assert_eq!(
        parse_record("v1\nkind checkout-propagation\nnokey\n").expect_err("no key"),
        PropagationParseRefusal {
            kind: "not-a-record",
            what: String::from("the line `nokey` holds no key and value"),
        }
    );
}
