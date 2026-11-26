//! What:
//!  Allowed and rejected fixtures for every deletion the cleanup makes.
//! Why:
//!  The cleanup deletes under an account's real cache,
//!  so an empty claimed folder must leave and a folder that holds a file,
//!  carries a mount or differs from its claim must stay with its record and
//!  claim;
//!  each fixture is its own folder under the test root.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(cleanUpStagingFolder(request).word).toBe('removed'); expect(folder.exists()).toBe(false);
//! ```

use super::{
    CleanupOutcome,
    CleanupRequest,
    clean_up_staging_folder,
};
use crate::fs_mount_options::MountFacts;
use crate::staged_folder_identity::StagedFolderIdentity;
use crate::staging_claim::{
    StagingClaim,
    write_claim,
};
use crate::staging_folder_name::{
    staging_claim_name,
    staging_folder_name,
    staging_record_name,
};
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::path::PathBuf;

/// The GUID the fixtures use.
fn guid() -> DatasetGuid {
    return DatasetGuid::parse("555").expect("the fixture GUID parses");
}

/// One fixture with a claim,
/// a folder and a record.
struct Fixture {
    cache_path: PathBuf,
    cache: std::fs::File,
    claims: std::fs::File,
    claims_path: PathBuf,
    claim_path: PathBuf,
    folder: PathBuf,
    record: PathBuf,
}

/// Make the cache,
/// its folder,
/// the record beside it and a claim of a dead owner.
fn fixture(name: &str) -> Fixture {
    let root: PathBuf = crate::testing::test_support::fixture_root().join(name);
    let cache_path: PathBuf = root.join("cache");
    let claim_folder: PathBuf = root.join("claims");
    std::fs::create_dir_all(cache_path.as_path()).expect("the fixture cache is made");
    std::fs::create_dir_all(claim_folder.as_path()).expect("the fixture claim folder is made");
    let folder: PathBuf = cache_path.join(staging_folder_name(guid()));
    std::fs::create_dir_all(folder.as_path()).expect("the fixture staging folder is made");
    let record: PathBuf = cache_path.join(staging_record_name(guid()));
    std::fs::write(record.as_path(), b"record\n").expect("the fixture record is written");
    let opened: std::fs::File =
        std::fs::File::open(folder.as_path()).expect("the fixture folder opens");
    let identity: StagedFolderIdentity =
        StagedFolderIdentity::of(&opened).expect("the fixture identity is read");
    let claim: StagingClaim = StagingClaim {
        guid: guid(),
        nonce: String::from("0123456789abcdef0123456789abcdef"),
        owner: String::from("bogus"),
        identity,
    };
    let claims: std::fs::File =
        std::fs::File::open(claim_folder.as_path()).expect("the claim folder opens");
    write_claim(&claims, &claim).expect("the claim is written");
    let claim_path: PathBuf = claim_folder.join(staging_claim_name(guid()));
    let cache: std::fs::File =
        std::fs::File::open(cache_path.as_path()).expect("the fixture cache opens");
    return Fixture {
        cache_path,
        cache,
        claims,
        claims_path: claim_folder,
        claim_path,
        folder,
        record,
    };
}

/// One request over the fixture,
/// with the mount rows the arm wants.
fn request<'a>(
    fixture: &'a Fixture,
    mounts: &'a dyn Fn() -> Vec<MountFacts>,
) -> CleanupRequest<'a> {
    return CleanupRequest {
        cache: &fixture.cache,
        cache_path: fixture.cache_path.as_path(),
        claim_directory: &fixture.claims,
        guid: guid(),
        owner_token: Some("bogus"),
        mounts,
        claim_path: fixture.claim_path.as_path(),
    };
}

/// A claim of a dead owner over an empty folder removes the folder,
/// its record and its claim.
#[test]
fn an_empty_claimed_folder_is_removed() {
    let fixture = fixture("cleanup-allowed");
    let no_mounts = || return Vec::new();
    let outcome: CleanupOutcome = clean_up_staging_folder(&request(&fixture, &no_mounts));
    assert_eq!(
        outcome,
        CleanupOutcome::Removed {
            record: true,
            claim: true
        }
    );
    assert!(!fixture.folder.exists());
    assert!(!fixture.record.exists());
    assert!(!fixture.claim_path.exists());
}

/// A folder that holds a file is kept with its record and claim.
#[test]
fn a_folder_that_holds_a_file_is_kept() {
    let fixture = fixture("cleanup-holds-file");
    std::fs::write(fixture.folder.join("sentinel"), b"keep me").expect("the sentinel is written");
    let no_mounts = || return Vec::new();
    let outcome: CleanupOutcome = clean_up_staging_folder(&request(&fixture, &no_mounts));
    assert!(
        matches!(outcome, CleanupOutcome::Kept { ref step, .. } if step == "holds-entries"),
        "{outcome:?}"
    );
    assert!(fixture.folder.join("sentinel").exists());
    assert!(fixture.record.exists());
    assert!(fixture.claim_path.exists());
}

/// A folder a mount row names is kept.
#[test]
fn a_folder_a_mount_names_is_kept() {
    let fixture = fixture("cleanup-mount");
    let row = |path: PathBuf| {
        return move || {
            return vec![MountFacts {
                mount_id: 9,
                parent_id: 1,
                root: PathBuf::from("/"),
                optional_fields: Vec::new(),
                mount_point: path.clone(),
                filesystem_type: String::from("zfs"),
                source: String::from("pool/cache"),
                mount_options: Vec::new(),
                super_options: Vec::new(),
            }];
        };
    };
    let mounts = row(fixture.folder.clone());
    let outcome: CleanupOutcome = clean_up_staging_folder(&request(&fixture, &mounts));
    assert!(
        matches!(outcome, CleanupOutcome::Kept { ref step, .. } if step == "mount"),
        "{outcome:?}"
    );
    assert!(fixture.folder.exists());
}

/// A folder whose identity moved from the claim is kept.
#[test]
fn a_folder_whose_identity_moved_is_kept() {
    let fixture = fixture("cleanup-identity");
    let claims: std::fs::File =
        std::fs::File::open(fixture.claims_path.as_path()).expect("the claim folder opens again");
    let mut claim: StagingClaim = crate::staging_claim::read_claim(&claims, guid())
        .expect("the claim is read")
        .expect("the claim is there");
    claim.identity.inode += 1;
    crate::staging_claim::write_claim(&claims, &claim).expect("the moved claim is written");
    let no_mounts = || return Vec::new();
    let outcome: CleanupOutcome = clean_up_staging_folder(&request(&fixture, &no_mounts));
    assert!(
        matches!(outcome, CleanupOutcome::Kept { ref step, .. } if step == "identity"),
        "{outcome:?}"
    );
    assert!(fixture.folder.exists());
}

/// A claim whose folder is absent is released,
/// not kept forever.
#[test]
fn a_claim_without_a_folder_is_released() {
    let fixture = fixture("cleanup-no-folder");
    std::fs::remove_dir(fixture.folder.as_path()).expect("the fixture folder is removed");
    let no_mounts = || return Vec::new();
    let outcome: CleanupOutcome = clean_up_staging_folder(&request(&fixture, &no_mounts));
    assert_eq!(outcome, CleanupOutcome::ReleasedNoFolder);
    assert!(!fixture.claim_path.exists());
}

/// A clone with no claim leaves the folder and reports it.
#[test]
fn a_folder_without_a_claim_is_kept() {
    let fixture = fixture("cleanup-no-claim");
    std::fs::remove_file(fixture.claim_path.as_path()).expect("the fixture claim is removed");
    let no_mounts = || return Vec::new();
    let outcome: CleanupOutcome = clean_up_staging_folder(&request(&fixture, &no_mounts));
    assert!(
        matches!(outcome, CleanupOutcome::Kept { ref step, .. } if step == "claim"),
        "{outcome:?}"
    );
    assert!(fixture.folder.exists());
}

/// A claim whose owner is still alive keeps the folder.
#[test]
fn a_folder_whose_owner_lives_is_kept() {
    let fixture = fixture("cleanup-owner-live");
    // The test process owns the claim, so its token names a live owner.
    let owner =
        crate::worktree_copy_owner::current_copy_owner().expect("this process's owner token");
    let no_mounts = || return Vec::new();
    let mut request = request(&fixture, &no_mounts);
    let token = owner.token();
    request.owner_token = Some(token.as_str());
    let outcome: CleanupOutcome = clean_up_staging_folder(&request);
    assert!(
        matches!(outcome, CleanupOutcome::Kept { ref step, .. } if step == "owner"),
        "{outcome:?}"
    );
    assert!(fixture.folder.exists());
}
