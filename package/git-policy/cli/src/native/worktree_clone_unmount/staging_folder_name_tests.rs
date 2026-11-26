//! What:
//!  Controls for the one staging component and its two beside-files.
//! Why:
//!  The wrapper,
//! the handler and `doctor` derive the same names from
//!  one GUID,
//!  and each reader must tell a folder from a record,
//! from a claim and from a
//!  name it did not derive.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(stagingRecordName(guid)).toBe(`${stagingFolderName(guid)}.json`);
//! expect(guidOfRecordName(stagingRecordName(guid))).toBe(guid);
//! ```

use crate::staging_folder_name::{
    StagingNameRefusal,
    guid_of_claim_name,
    guid_of_folder_name,
    guid_of_record_name,
    staging_claim_name,
    staging_folder_name,
    staging_record_name,
    staging_temporary_name,
};
use crate::worktree_clone_mount_protocol::DatasetGuid;

/// The GUID the controls use.
fn guid() -> DatasetGuid {
    return DatasetGuid::parse("18446744073709551615").expect("the largest GUID parses");
}

/// The component,
/// the record and the claim one GUID gives.
#[test]
fn the_three_names_share_one_component() {
    let folder: String = staging_folder_name(guid());
    assert_eq!(
        folder,
        "monochromatic-git-policy-cli-staging-18446744073709551615"
    );
    assert_eq!(staging_record_name(guid()), format!("{folder}.json"));
    assert_eq!(staging_temporary_name(guid()), format!("{folder}.json.tmp"));
    assert_eq!(staging_claim_name(guid()), "18446744073709551615.staged");
}

/// Each name reads back as the GUID it was derived from.
#[test]
fn each_name_reads_back_to_its_guid() {
    assert_eq!(
        guid_of_folder_name(staging_folder_name(guid()).as_str()),
        Ok(guid())
    );
    assert_eq!(
        guid_of_record_name(staging_record_name(guid()).as_str()),
        Ok(guid())
    );
    assert_eq!(
        guid_of_claim_name(staging_claim_name(guid()).as_str()),
        Ok(guid())
    );
}

/// A name that is not one this route derives is refused rather than guessed.
#[test]
fn a_foreign_name_is_refused() {
    assert!(matches!(
        guid_of_folder_name("some-other-folder"),
        Err(StagingNameRefusal::Prefix { .. })
    ));
    assert!(matches!(
        guid_of_folder_name("monochromatic-git-policy-cli-staging-0123"),
        Err(StagingNameRefusal::Guid { .. })
    ));
    assert!(matches!(
        guid_of_folder_name("monochromatic-git-policy-cli-staging-123.json"),
        Err(StagingNameRefusal::Suffix { .. })
    ));
    assert!(matches!(
        guid_of_record_name("123.staged"),
        Err(StagingNameRefusal::Suffix { .. })
    ));
    assert!(matches!(
        guid_of_claim_name("123.json"),
        Err(StagingNameRefusal::Suffix { .. })
    ));
    assert!(matches!(
        guid_of_claim_name("nonsense.staged"),
        Err(StagingNameRefusal::Guid { .. })
    ));
}

/// A GUID with a leading zero has one canonical spelling and is refused in the other.
#[test]
fn a_leading_zero_is_refused() {
    assert!(matches!(
        guid_of_folder_name("monochromatic-git-policy-cli-staging-007"),
        Err(StagingNameRefusal::Guid { .. })
    ));
}
