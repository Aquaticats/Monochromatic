//! The manifest text a work item carries:
//! what round-trips and what is refused.

use super::*;

/// An identity with every field set.
fn identity(inode: u64) -> ObjectId {
    return ObjectId { device: 7, inode, birth: Some((1_700_000_000, 5)) };
}

/// A manifest survives the text a work item carries,
/// entries in order,
/// and an identity without a birth time does too.
#[test]
fn a_manifest_round_trips_through_its_text() {
    let name: String = "ab".repeat(20);
    let manifest: TrashManifest = TrashManifest { name: name.clone(), trash: identity(1), entries: vec![identity(2), ObjectId { device: 7, inode: 3, birth: None }] };
    assert_eq!(TrashManifest::decode(name.as_str(), manifest.encode().as_str()), Ok(manifest));
}

/// A manifest with no entries is just the identity of the trash.
#[test]
fn a_manifest_without_entries_round_trips() {
    let name: String = "cd".repeat(20);
    let manifest: TrashManifest = TrashManifest { name: name.clone(), trash: identity(9), entries: Vec::new() };
    assert_eq!(TrashManifest::decode(name.as_str(), manifest.encode().as_str()), Ok(manifest));
}

/// A name that is not forty lowercase hexadecimal digits,
/// and text that is not identities,
/// are refused with a reason.
#[test]
fn a_name_or_an_identity_that_is_not_one_is_refused() {
    let name: String = "ef".repeat(20);
    let text: String = identity(1).encode();
    for bad in ["../../etc", "", "EF".repeat(20).as_str(), "ef".repeat(19).as_str()] {
        assert!(TrashManifest::decode(bad, text.as_str()).is_err(), "{bad:?}");
    }
    for bad in ["", "other", "1:2:3", "1:2:-:-;nonsense"] {
        assert!(TrashManifest::decode(name.as_str(), bad).is_err(), "{bad:?}");
    }
}

/// Each move gets its own name,
/// and the name carries the index,
/// so the disposal finds exactly the entry the move made.
#[test]
fn entry_names_are_distinct_per_move() {
    let names: std::collections::HashSet<String> = (0..50).map(entry_name).collect();
    assert_eq!(names.len(), 50, "no two moves share a name");
    assert!(entry_name(12).ends_with("12"));
}
