//! What:
//!  Controls of the places a shutdown pass reads:
//!  this boot's identifier trim,
//!  a file that is not there,
//!  an empty file,
//!  and the installed place's own names.
//! Why:
//!  The pass tells this boot's readiness record from an earlier boot's by that
//!  identifier,
//!  so an identifier this code guessed would judge another boot's record as
//!  this one's
//!  (`crate::worktree_clone_mount_protocol::clone_mount_readiness_admission`).
//!  No control here opens a device or a mount:
//!  the identifier is one file of the test's fixture.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const places = fixturePlaces('boot'); writeFile(bootFile, 'abc\n'); expect(places.thisBoot()).toBe('abc');
//! ```

use super::ShutdownUnmountPlaces;
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount::slot_identity_store::SLOT_IDENTITY_STORE_PATH;
use crate::worktree_clone_mount_handler::handler_install_paths::CONFIGURATION_PATH;
use crate::worktree_clone_mount_handler::per_slot_root_claim::{
    SLOT_CLAIM_DIRECTORY,
    SlotClaimDirectory,
};
use crate::worktree_clone_mount_protocol::clone_mount_readiness::READINESS_DIRECTORY;
use crate::worktree_clone_mount_protocol::clone_mount_readiness_file::ReadinessFile;
use crate::worktree_clone_unmount::staging_calls::BOOT_ID_PATH;
use std::path::PathBuf;

/// This account's own number.
fn this_uid() -> u32 {
    // SAFETY: `getuid` takes no argument and changes nothing.
    return unsafe { libc::getuid() };
}

/// The places of one control,
/// all of them under the test's fixture root.
fn fixture_places(name: &str) -> ShutdownUnmountPlaces {
    let root: PathBuf = fixture(name);
    return ShutdownUnmountPlaces {
        readiness: ReadinessFile::at(root.join("readiness"), this_uid()),
        configuration: root.join("configuration.jsonc"),
        claims: SlotClaimDirectory::at(root.join("claims"), this_uid()),
        store: root.join("store"),
        boot_identity: root.join("boot_id"),
        proc_root: root.join("proc"),
        self_task: root.join("proc").join("self"),
    };
}

/// The boot identifier is read without the line's own newline.
#[test]
fn the_boot_identifier_is_read_without_its_newline() {
    let places: ShutdownUnmountPlaces = fixture_places("shutdown-places-boot");
    std::fs::write(
        places.boot_identity.as_path(),
        "0f3a5c7e-1b2d-4f60-8a9b-0c1d2e3f4a5b\n",
    )
    .expect("the fixture identifier is written");
    assert_eq!(
        places.this_boot().expect("the identifier reads"),
        "0f3a5c7e-1b2d-4f60-8a9b-0c1d2e3f4a5b"
    );
}

/// A file that is not there refuses, and says which file.
#[test]
fn a_missing_boot_identifier_refuses_and_names_the_file() {
    let places: ShutdownUnmountPlaces = fixture_places("shutdown-places-missing");
    let said: String = places.this_boot().expect_err("an absent file is a refusal");
    assert!(
        said.contains(
            places
                .boot_identity
                .to_str()
                .expect("the fixture path is text")
        ),
        "the refusal names the file a person inspects, and this one says {said}"
    );
    assert!(
        said.contains("Ways forward:"),
        "a refusal says what to do next, and this one says {said}"
    );
}

/// A file holding nothing is a refusal rather than an empty identifier.
#[test]
fn an_empty_boot_identifier_refuses() {
    let places: ShutdownUnmountPlaces = fixture_places("shutdown-places-empty");
    std::fs::write(places.boot_identity.as_path(), "\n").expect("the empty file is written");
    let said: String = places.this_boot().expect_err("an empty file is a refusal");
    assert!(
        said.contains("holds no characters"),
        "the refusal says the file is empty, and this one says {said}"
    );
}

/// The installed places are the ones the boot unit's own words reach.
#[test]
fn the_live_places_are_the_installed_ones() {
    let places: ShutdownUnmountPlaces = ShutdownUnmountPlaces::live();
    assert_eq!(
        places.readiness_directory().to_str(),
        Some(READINESS_DIRECTORY)
    );
    assert_eq!(places.configuration.to_str(), Some(CONFIGURATION_PATH));
    assert_eq!(
        places.claims.directory().to_str(),
        Some(SLOT_CLAIM_DIRECTORY)
    );
    assert_eq!(places.store.to_str(), Some(SLOT_IDENTITY_STORE_PATH));
    assert_eq!(places.boot_identity.to_str(), Some(BOOT_ID_PATH));
    assert_eq!(places.readiness.owner(), 0);
}
