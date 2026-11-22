//! What:
//!  Controls for the staging claim's name,
//! text and one whole write.
//! Why:
//!  The claim is what proves the wrapper made a folder,
//!  so a claim of another GUID or another shape must be kept rather than read
//!  as one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeStagingClaim(directory, claim); expect(readStagingClaim(directory, guid)).toEqual(claim);
//! ```

use super::{claim_from_text, read_claim, staging_claim_path, write_claim, StagingClaim, CLAIM_FOLDER};
use crate::staged_folder_identity::StagedFolderIdentity;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::path::{Path, PathBuf};

/// The GUID the fixtures use.
fn guid() -> DatasetGuid {
    return DatasetGuid::parse("1234").expect("the fixture GUID parses");
}

/// One claim.
fn claim() -> StagingClaim {
    return StagingClaim {
        guid: guid(),
        nonce: String::from("0123456789abcdef0123456789abcdef"),
        owner: String::from("1234~1~2"),
        identity: StagedFolderIdentity { device: 4, inode: 5, birth: Some((1_700_000_000, 1)), owner: 1000, mode: 0o700 },
    };
}

/// The claim's place is beside the copy journal,
/// not inside it.
#[test]
fn the_claim_lives_beside_the_copy_journal() {
    let path: PathBuf = staging_claim_path(Path::new("/repo/.git"), guid());
    assert_eq!(path, Path::new("/repo/.git").join(CLAIM_FOLDER).join("1234.staged"));
}

/// One claim survives its text.
#[test]
fn one_claim_survives_its_text() {
    let text: String = claim().encode();
    assert_eq!(claim_from_text(text.as_str(), guid()), Ok(claim()));
}

/// A claim of another GUID is kept rather than read.
#[test]
fn a_claim_of_another_guid_is_refused() {
    let other: DatasetGuid = DatasetGuid::parse("9999").expect("the other GUID parses");
    assert!(claim_from_text(claim().encode().as_str(), other).is_err());
}

/// A claim of another shape is refused.
#[test]
fn a_foreign_shape_is_refused() {
    assert!(claim_from_text("[]", guid()).is_err());
    assert!(claim_from_text("{\"version\":\"v2\",\"kind\":\"staging-claim\"}", guid()).is_err());
}

/// One whole claim is written and read back.
#[test]
fn one_whole_claim_is_written() {
    let root: PathBuf = crate::testing::test_support::fixture_root().join("staging-claim-write");
    std::fs::create_dir_all(root.as_path()).expect("the claim folder is made");
    let directory: std::fs::File = std::fs::File::open(root.as_path()).expect("the claim folder opens");
    write_claim(&directory, &claim()).expect("the claim is written");
    assert_eq!(read_claim(&directory, guid()).expect("the claim is read"), Some(claim()));
    assert!(!root.join("1234.staged.tmp").exists());
}

/// An absent claim is no claim.
#[test]
fn an_absent_claim_is_nothing() {
    let root: PathBuf = crate::testing::test_support::fixture_root().join("staging-claim-absent");
    std::fs::create_dir_all(root.as_path()).expect("the claim folder is made");
    let directory: std::fs::File = std::fs::File::open(root.as_path()).expect("the claim folder opens");
    assert_eq!(read_claim(&directory, guid()), Ok(None));
}
