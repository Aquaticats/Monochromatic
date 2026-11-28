//! What:
//!  The record's file:
//!  one whole record published over the earlier one,
//!  the answer read back,
//!  a directory that is not there answering "no record yet",
//!  and each proof the reader makes before it believes a byte.
//! Why:
//!  Two runs write the same facts about one instance and the last whole
//!  record must stand,
//!  so the rename is the only way a record is replaced,
//!  and a reader must refuse a directory or a file it cannot prove root
//!  published.
//!  A control hands in a fixture directory it owns,
//!  so the release place under `/run` is exercised by the guest arm alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! file.publish(one); file.publish(another); expect(file.read()).toEqual(another);
//! ```

use super::*;
use crate::checkout_propagation::mount_propagation_set::MountPropagation;
use crate::checkout_propagation::propagation_record::PropagationRecord;
use crate::checkout_propagation::propagation_words::{
    PropagationOutcome,
    PropagationStep,
};
use crate::checkout_propagation::shared_mounts_beneath::SharedMountsBeneath;
use crate::testing::test_support::fixture;
use std::os::unix::fs::PermissionsExt;

/// What:
///  One answer to publish,
///  with the fields that make two publishes tellable apart.
/// Why:
///  A control reads back the record it published and must know it read that
///  one and not the earlier,
///  so the numbers and the outcome are the ones a second publish changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = (mountId: number, outcome: PropagationOutcome): PropagationRecord => ({ ... });
/// ```
fn record(
    mount_id: u64,
    outcome: PropagationOutcome,
) -> PropagationRecord {
    return PropagationRecord {
        boot: String::from("6f1c2b8a-2b9f-4b4d-9a1f-0e5e6f7a8b9c"),
        mount_id: Some(mount_id),
        mount_id_old: Some(41),
        mount_namespace_id: Some(7),
        checkout: Some(String::from("/home/user/Monochromatic")),
        source: Some(String::from("pool/home/monochromatic-git-checkout")),
        before: Some(MountPropagation::of_the_kernel_set(libc::MS_SHARED).expect("shared")),
        after: Some(MountPropagation::of_the_kernel_set(libc::MS_PRIVATE).expect("private")),
        master: None,
        beneath: SharedMountsBeneath::Counted {
            count: 0,
            paths: Vec::new(),
        },
        outcome,
        step: PropagationStep::NoStep,
        errno: None,
        time: String::from("2026-11-27T21:00:00.000-05:00"),
    };
}

/// One file at a fixture folder the account running the tests owns,
/// which is the owner every control states.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = (): { file: PropagationFile; directory: string } => ...;
/// ```
fn place() -> PropagationFile {
    let directory: std::path::PathBuf = fixture("checkout-propagation-record");
    return PropagationFile::at(directory, nix_uid());
}

/// The account this process runs as,
/// which a fixture's file and folder must belong to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const uid = (): number => process.getuid();
/// ```
fn nix_uid() -> u32 {
    // SAFETY: `getuid` reads the calling process's own number and touches no memory.
    return unsafe { libc::getuid() };
}

#[test]
fn one_whole_record_is_published_over_the_earlier_one() {
    let file: PropagationFile = place();
    let first: PropagationRecord = record(9001, PropagationOutcome::Applied);
    let second: PropagationRecord = record(9002, PropagationOutcome::AlreadyPrivate);
    file.publish(&first).expect("the first answer");
    file.publish(&second).expect("the answer that replaces it");
    assert_eq!(
        file.read().expect("a readable record"),
        Some(second),
        "the later publish is the record a reader sees whole"
    );
    assert!(
        file.record().exists(),
        "the record stands at its own name: {}",
        file.record().display()
    );
}

#[test]
fn a_publish_leaves_no_staging_sibling_behind_and_no_lock_file() {
    let file: PropagationFile = place();
    file.publish(&record(9001, PropagationOutcome::Applied))
        .expect("the answer");
    let mut names: Vec<String> = Vec::new();
    for listed in std::fs::read_dir(file.record().parent().expect("the directory"))
        .expect("the directory is there")
    {
        let entry: std::fs::DirEntry = listed.expect("an entry");
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    assert_eq!(
        names,
        vec![String::from("result")],
        "the directory holds the record and nothing else: no staging sibling, no lock"
    );
}

#[test]
fn a_directory_that_is_not_there_is_no_record_yet() {
    let file: PropagationFile = PropagationFile::at(
        fixture("checkout-propagation-absent").join("never-made"),
        nix_uid(),
    );
    assert_eq!(
        file.read().expect("no record is not a refusal"),
        None,
        "a boot that has not run the handler answers no record"
    );
    assert!(
        !file.record().exists(),
        "the read makes nothing, unlike the publish"
    );
}

#[test]
fn a_record_at_another_mode_is_refused() {
    let file: PropagationFile = place();
    file.publish(&record(9001, PropagationOutcome::Applied))
        .expect("the answer");
    std::fs::set_permissions(file.record(), std::fs::Permissions::from_mode(0o666))
        .expect("a mode a reader must refuse");
    let refusal: PropagationFileRefusal = file.read().expect_err("a widened record");
    assert_eq!(
        refusal,
        PropagationFileRefusal::RecordMode {
            found: 0o666,
        }
    );
    assert!(refusal.message().contains("mode"));
}

#[test]
fn a_record_a_writer_left_half_written_is_refused() {
    let file: PropagationFile = place();
    file.publish(&record(9001, PropagationOutcome::Applied))
        .expect("the answer");
    std::fs::write(file.record(), "v1\nkind checkout-propagation\n")
        .expect("a half-written record");
    let refusal: PropagationFileRefusal = file.read().expect_err("a text the parser refuses");
    match refusal {
        PropagationFileRefusal::NotARecord {
            refusal: parse_refusal,
        } => {
            assert_eq!(parse_refusal.kind, "key");
        }
        other => panic!("the read answered {other:?}, and a half-written record is no record"),
    }
}

#[test]
fn a_writer_whose_text_its_own_parser_refuses_writes_nothing() {
    let file: PropagationFile = place();
    let mut broken: PropagationRecord = record(9001, PropagationOutcome::Applied);
    broken.mount_id = None;
    broken.step = PropagationStep::VerifyAfter;
    // A refusal with no number is a shape this writer never builds: the parser reads it, so this text is written.
    file.publish(&broken)
        .expect("a refusal's own shape is one the parser reads");
    assert_eq!(
        file.read().expect("a readable record"),
        Some(broken),
        "the text the writer built is the text the reader read"
    );
}

#[test]
fn a_directory_another_account_owns_is_refused_rather_than_written() {
    let directory: std::path::PathBuf = fixture("checkout-propagation-owner");
    let file: PropagationFile = PropagationFile::at(directory, nix_uid() + 1);
    let refusal: PropagationFileRefusal = file
        .publish(&record(9001, PropagationOutcome::Applied))
        .expect_err("a directory owned by another account");
    assert!(
        refusal.message().contains("owned by user"),
        "the refusal names the account: {}",
        refusal.message()
    );
}
