//! Controls for the record's home:
//!  the folder the design names,
//!  the slot a destination holds,
//!  the four name forms,
//!  both staging forms,
//!  and the folder the first command makes with mode `0700`.

use crate::worktree_clone_binding_record::registration_home::*;
use std::os::unix::fs::PermissionsExt;
use std::path::{
    Path,
    PathBuf,
};

/// The home is under the wrapper's own folder,
/// one family and one version down.
#[test]
fn the_home_is_the_wrappers_folder_one_family_and_one_version_down() {
    let home: PathBuf = home_folder(Path::new("/repo/.git"));
    assert_eq!(
        home,
        PathBuf::from("/repo/.git/monochromatic-git-policy-cli/worktree-clones/v1")
    );
    assert_eq!(
        plans_folder(Path::new("/repo/.git")),
        PathBuf::from("/repo/.git/monochromatic-git-policy-cli/clone-destruction-plans")
    );
}

/// A destination's last component is its slot,
/// and a name no plain slot may have refuses with its reason.
#[test]
fn a_destination_names_its_slot_and_refuses_another() {
    assert_eq!(
        slot_of(Path::new("/repo/.claude/worktrees/add-cache-layer")),
        Ok(String::from("add-cache-layer"))
    );
    assert!(slot_of(Path::new("/repo/.claude/worktrees/two words")).is_err());
    assert!(slot_of(Path::new("/repo/.claude/worktrees/.hidden")).is_err());
    assert!(slot_of(Path::new("/repo/.claude/worktrees/..")).is_err());
    assert!(slot_of(Path::new("/")).is_err());
}

/// The three record forms read back from their names,
/// and a staging name is none of them.
#[test]
fn the_three_record_forms_read_back_from_their_names() {
    let slot: &str = "add-cache-layer";
    for (name, kind) in [
        (binding_name(slot), RecordKind::Binding),
        (origin_inventory_name(slot), RecordKind::OriginInventory),
        (clone_inventory_name(slot), RecordKind::CloneInventory),
    ] {
        let read: RecordName = record_name_of(name.as_str()).expect("the record name reads");
        assert_eq!(read.slot, slot);
        assert_eq!(read.kind, kind);
    }
    assert_eq!(
        claim_prefix("add-cache-layer"),
        std::ffi::OsString::from("add-cache-layer.claim")
    );
    assert_eq!(record_name_of("add-cache-layer.claim.1"), None);
    assert_eq!(
        record_name_of(".add-cache-layer.binding.0123456789abcdef0123456789abcdef.partial"),
        None
    );
}

/// Both staging forms are known,
/// and a name of another shape is not.
#[test]
fn both_staging_forms_are_known() {
    assert!(is_known_staging_name(
        ".add-cache-layer.binding.0123456789abcdef0123456789abcdef.partial"
    ));
    assert!(is_known_staging_name(".cli-git-claim.tmp.4321.123456789.1"));
    assert!(!is_known_staging_name("add-cache-layer.binding"));
    assert!(!is_known_staging_name(
        ".add-cache-layer.binding.0123.partial"
    ));
    assert!(!is_known_staging_name(".cli-git-claim.tmp.4321.123456789"));
    assert!(!is_known_staging_name(".something.partial"));
}

/// A claim generation reads from its name and no other slot's does.
#[test]
fn a_claim_generation_reads_from_its_own_slots_name() {
    assert_eq!(
        claim_generation_of("add-cache-layer.claim.7", "add-cache-layer"),
        Some(7)
    );
    assert_eq!(
        claim_generation_of("add-cache-layer.claim.0", "add-cache-layer"),
        None
    );
    assert_eq!(
        claim_generation_of("add-cache-layer.claim.7", "another-slot"),
        None
    );
    assert_eq!(
        claim_generation_of("add-cache-layer.claim.x", "add-cache-layer"),
        None
    );
}

/// The first command that needs the folder makes it,
/// with mode `0700` and its two parents.
#[test]
fn the_first_command_makes_the_folder_private() {
    let root: PathBuf = crate::test_support::fixture("record-home");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir(common.as_path()).expect("the common directory is made");
    let folder: std::fs::File =
        open_or_create_home(common.as_path()).expect("the home folder is made");
    let metadata: std::fs::Metadata = folder.metadata().expect("the folder's metadata reads");
    assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
    assert!(home_folder(common.as_path()).is_dir());
    crate::test_support::remove(root.as_path());
}

/// What:
///  A home that is not there,
/// is empty,
/// or holds only an unrelated file answers
///  that it holds no record;
///  a binding record and a record still being written answer that it does.
/// Why:
///  The unknown-global refusal protects a slot that has a record,
///  so it reads this answer instead of asking whether the folder exists:
///  a folder the route made on an earlier command and left empty endangers no
///  slot,
/// and a record mid-write is the same home's record.
#[test]
fn the_home_says_whether_it_holds_a_record() {
    let root: PathBuf = crate::test_support::fixture("registration-home-records");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the common directory");
    assert!(
        !holds_a_record(common.as_path()),
        "a home that is not there holds no record"
    );
    let home: PathBuf = home_folder(common.as_path());
    std::fs::create_dir_all(home.as_path()).expect("the home");
    assert!(
        !holds_a_record(common.as_path()),
        "an empty home holds no record"
    );
    std::fs::write(home.join("notes.txt"), b"unrelated\n").expect("the unrelated file");
    assert!(
        !holds_a_record(common.as_path()),
        "an unrelated file is no record"
    );
    std::fs::write(home.join(binding_name("one")), b"a record\n").expect("the record");
    assert!(
        holds_a_record(common.as_path()),
        "a binding record answers yes"
    );
    std::fs::remove_file(home.join(binding_name("one"))).expect("the record goes");
    std::fs::write(
        home.join(".one.binding.0123456789abcdef0123456789abcdef.partial"),
        b"a record being written\n",
    )
    .expect("the staging record");
    assert!(
        holds_a_record(common.as_path()),
        "a record being written answers yes"
    );
    crate::test_support::remove(root.as_path());
}
