//! What:
//!  Controls of the names one attempt is filed under:
//!  the entry's own name,
//!  the staging name a writer links it from,
//!  and the number the next claim takes.
//! Why:
//!  Two writers of one burst must never share a name,
//!  and a name this reader would misread would hand out an attempt that stands
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "The record:
//! one file per clone").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(EntryName.parse('42.3').attempt).toBe(3);
//! ```

use super::{
    EntryName,
    NameRefusal,
    fresh_staging_name,
    is_a_staging_name,
    one_past_the_highest,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;

/// An attempt's name reads back as its GUID and number.
#[test]
fn an_attempt_s_name_reads_back() {
    let name: EntryName = EntryName {
        guid: DatasetGuid::parse("42").expect("the control GUID is canonical"),
        attempt: 3,
    };
    assert_eq!(name.file_name(), "42.3");
    assert_eq!(EntryName::parse("42.3"), Ok(name));
}

/// A staging name is not an entry,
/// and its shape is the slot identity store's:
/// a dot and 32 lowercase hexadecimal digits.
#[test]
fn a_staging_name_is_not_an_entry() {
    let staging: String = format!(".{}", "0123456789abcdef".repeat(2));
    assert!(is_a_staging_name(staging.as_str()));
    assert_eq!(
        EntryName::parse(staging.as_str()),
        Err(NameRefusal::Staging)
    );
    for wrong in [
        format!(".{}", "0123456789ABCDEF".repeat(2)),
        format!(".{}", "0".repeat(31)),
        format!(".{}", "0".repeat(33)),
        String::from("..parent.abc.partial"),
    ] {
        assert!(!is_a_staging_name(wrong.as_str()), "{wrong}");
    }
}

/// A name whose GUID or attempt part is not canonical is refused,
/// so one attempt has exactly one name.
#[test]
fn a_name_of_another_shape_is_refused() {
    for wrong in [
        "", "42", "42.", "042.1", "42.01", "42.0", "42.one", "x.1", ".1",
    ] {
        let refusal: NameRefusal = EntryName::parse(wrong).expect_err("the name is not an attempt");
        assert!(
            matches!(refusal, NameRefusal::Attempt | NameRefusal::Guid),
            "{wrong}"
        );
    }
}

/// Every fresh staging name has the designed shape,
/// and two of them differ,
/// so a killed writer's leftover blocks no later writer.
#[test]
fn a_fresh_staging_name_is_the_writer_s_own() {
    let first: String = fresh_staging_name().expect("the random source answers");
    let second: String = fresh_staging_name().expect("the random source answers again");
    assert!(is_a_staging_name(first.as_str()), "{first}");
    assert_ne!(first, second);
}

/// The next attempt is one past the highest name present,
/// empty or readable,
/// and the first attempt of a clone is one.
#[test]
fn the_next_attempt_is_one_past_the_highest() {
    assert_eq!(one_past_the_highest(&[]), 1);
    assert_eq!(one_past_the_highest(&[1]), 2);
    assert_eq!(one_past_the_highest(&[1, 3]), 4);
    assert_eq!(one_past_the_highest(&[2, 1]), 3);
    assert_eq!(one_past_the_highest(&[u32::MAX]), u32::MAX);
}
