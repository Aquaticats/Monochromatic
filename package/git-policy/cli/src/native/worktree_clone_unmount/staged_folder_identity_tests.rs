//! What:
//!  Controls for one staging folder's five recorded fields and their compare.
//! Why:
//!  The compare is what tells a folder the route recorded from a replacement the
//!  account made under the same name,
//!  and each field must be able to fail alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(identity.differences(other)).toEqual(['inode']);
//! ```

use crate::staged_folder_identity::{
    StagedFolderField,
    StagedFolderIdentity,
};

/// One identity every control changes in one field.
fn identity() -> StagedFolderIdentity {
    return StagedFolderIdentity {
        device: 42,
        inode: 7,
        birth: Some((1_700_000_000, 12)),
        owner: 1000,
        mode: 0o700,
    };
}

/// The same identity gives no field.
#[test]
fn the_same_identity_differs_in_nothing() {
    assert_eq!(identity().differences(&identity()), Vec::new());
}

/// Each field fails alone and is named.
#[test]
fn each_field_is_named_when_it_moves() {
    let cases: [(StagedFolderIdentity, StagedFolderField); 5] = [
        (
            StagedFolderIdentity {
                device: 43,
                ..identity()
            },
            StagedFolderField::Device,
        ),
        (
            StagedFolderIdentity {
                inode: 8,
                ..identity()
            },
            StagedFolderField::Inode,
        ),
        (
            StagedFolderIdentity {
                birth: Some((1_700_000_001, 12)),
                ..identity()
            },
            StagedFolderField::Birth,
        ),
        (
            StagedFolderIdentity {
                owner: 1001,
                ..identity()
            },
            StagedFolderField::Owner,
        ),
        (
            StagedFolderIdentity {
                mode: 0o755,
                ..identity()
            },
            StagedFolderField::Mode,
        ),
    ];
    for (changed, field) in cases {
        assert_eq!(identity().differences(&changed), vec![field]);
    }
}

/// A folder's descriptor gives the identity its metadata holds.
#[test]
fn a_folder_descriptor_gives_its_identity() {
    let root: std::path::PathBuf =
        crate::testing::test_support::fixture_root().join("staging-folder-identity");
    std::fs::create_dir_all(root.as_path()).expect("the fixture folder is made");
    let folder: std::fs::File =
        std::fs::File::open(root.as_path()).expect("the fixture folder opens");
    let identity: StagedFolderIdentity =
        StagedFolderIdentity::of(&folder).expect("the identity is read");
    let facts: std::fs::Metadata = folder
        .metadata()
        .expect("the fixture folder's metadata is read");
    use std::os::unix::fs::MetadataExt;
    assert_eq!(identity.device, facts.dev());
    assert_eq!(identity.inode, facts.ino());
    assert_eq!(identity.owner, facts.uid());
    assert_eq!(identity.mode, facts.mode() & 0o7777);
}

/// More than one moved field is spelled as one phrase.
#[test]
fn several_fields_are_spelled_together() {
    assert_eq!(
        StagedFolderIdentity::field_names(&[StagedFolderField::Inode, StagedFolderField::Mode]),
        "inode, mode"
    );
}
