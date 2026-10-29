//! Controls for the transaction registry:
//!  names,
//!  publication,
//!  listing and removal.

use super::*;
use crate::private_storage::take_directory_syncs;
use crate::test_support::{fixture, remove};

/// The canonical transaction ID used by these tests.
const ID: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";

/// Only the canonical lowercase layout is a transaction ID.
#[test]
fn transaction_ids_are_canonical_uuids() {
    assert!(is_transaction_id(ID));
    assert!(is_transaction_id("00000000-0000-0000-0000-000000000000"));
    assert!(is_transaction_id("ffffffff-ffff-ffff-ffff-ffffffffffff"));
    for refused in [
        "",
        "0B6C2C1E-6F5B-4D0E-9A55-3F5D8E2F6A10",
        "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a1",
        "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a100",
        "0b6c2c1e_6f5b-4d0e-9a55-3f5d8e2f6a10",
        "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a1g",
        "0b6c2c1e-6f5b-4d0e-9a5-53f5d8e2f6a10",
        "0b6c2c1e6f5b-4d0e-9a55--3f5d8e2f6a10",
        "-b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10",
    ] {
        assert!(!is_transaction_id(refused), "{refused}");
    }
}

/// The registry's locks and their candidates are not transactions.
#[test]
fn lock_names_are_recognized() {
    for name in [
        "landing.lock",
        "reservation.lock",
        "landing.lock.x.pending",
        "reservation.lock.y.stale",
    ] {
        assert!(is_registry_lock_name(name), "{name}");
    }
    for name in [
        "landing.locks",
        "landing",
        "hook.lock",
        ID,
        "reservation.lockx",
    ] {
        assert!(!is_registry_lock_name(name), "{name}");
    }
}

/// Names classify into the three kinds,
///  and anything else is unexpected.
#[test]
fn names_classify_by_suffix() {
    let root: &Path = Path::new("/repo/.git/cli-git-transactions");
    let published: RegistryEntry = classify_entry_name(root, ID).expect("published");
    assert_eq!(published.kind, EntryKind::Transaction);
    assert_eq!(published.transaction_id, ID);
    assert_eq!(published.path, root.join(ID));
    let staging: RegistryEntry =
        classify_entry_name(root, format!("{ID}.pending").as_str()).expect("staging");
    assert_eq!(
        (staging.kind, staging.transaction_id.as_str()),
        (EntryKind::Staging, ID)
    );
    let retired: RegistryEntry =
        classify_entry_name(root, format!("{ID}.retired").as_str()).expect("retired");
    assert_eq!(
        (retired.kind, retired.transaction_id.as_str()),
        (EntryKind::Retired, ID)
    );
    for name in [
        "junk",
        "x.pending",
        "x.retired",
        format!("{ID}.other").as_str(),
    ] {
        assert_eq!(
            classify_entry_name(root, name).expect_err("unexpected").0,
            format!(
                "Unexpected transaction registry entry: /repo/.git/cli-git-transactions/{name}"
            )
        );
    }
}

/// Publication renames a complete candidate;
///  listing sees it and nothing half-built.
#[test]
fn publication_and_listing() {
    let git_dir: PathBuf = fixture("registry-publish");
    let root: PathBuf = git_dir.join(TRANSACTION_ROOT_NAME);
    assert!(
        list_transaction_entries(root.as_path())
            .expect("absent")
            .is_empty()
    );
    ensure_transaction_root(root.as_path()).expect("create");
    ensure_transaction_root(root.as_path()).expect("exists already");
    let directory: PathBuf =
        publish_transaction_directory(root.as_path(), ID, b"{\"owner\":1}\n").expect("publish");
    assert_eq!(directory, root.join(ID));
    assert_eq!(
        std::fs::read(directory.join(OWNER_FILENAME)).expect("owner"),
        b"{\"owner\":1}\n"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode: u32 = std::fs::metadata(&directory)
            .expect("meta")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o700);
        let root_mode: u32 = std::fs::metadata(&root).expect("meta").permissions().mode() & 0o777;
        assert_eq!(root_mode, 0o700);
    }
    std::fs::create_dir(root.join("landing.lock")).expect("lock");
    std::fs::create_dir(root.join(format!("{ID}.retired").replace("0b6c", "1b6c")))
        .expect("retired");
    let entries: Vec<RegistryEntry> = list_transaction_entries(root.as_path()).expect("listing");
    assert_eq!(entries.len(), 2, "locks are left out: {entries:?}");
    assert_eq!(entries[0].kind, EntryKind::Transaction);
    assert_eq!(entries[1].kind, EntryKind::Retired);
    assert!(
        publish_transaction_directory(root.as_path(), "not-an-id", b"").is_err(),
        "a malformed ID is refused"
    );
    let second: Result<PathBuf, RecoveryError> =
        publish_transaction_directory(root.as_path(), ID, b"{}");
    assert!(second.is_err(), "an existing transaction is never replaced");
    assert!(
        !root.join(format!("{ID}.pending")).exists(),
        "the failed candidate was removed"
    );
    remove(git_dir.as_path());
}

/// Unsafe registries and entries fail closed with the path named.
#[cfg(unix)]
#[test]
fn unsafe_state_fails_closed() {
    let git_dir: PathBuf = fixture("registry-unsafe");
    let root: PathBuf = git_dir.join(TRANSACTION_ROOT_NAME);
    std::fs::write(&root, b"x").expect("file registry");
    assert_eq!(
        list_transaction_entries(root.as_path())
            .expect_err("file")
            .0,
        format!("Unsafe transaction registry: {}", root.display())
    );
    assert!(ensure_transaction_root(root.as_path()).is_err());
    std::fs::remove_file(&root).expect("remove");
    let elsewhere: PathBuf = git_dir.join("elsewhere");
    std::fs::create_dir(&elsewhere).expect("elsewhere");
    std::os::unix::fs::symlink(&elsewhere, &root).expect("linked registry");
    assert!(list_transaction_entries(root.as_path()).is_err());
    assert!(ensure_transaction_root(root.as_path()).is_err());
    std::fs::remove_file(&root).expect("unlink");
    std::fs::create_dir(&root).expect("registry");
    std::os::unix::fs::symlink(&elsewhere, root.join(ID)).expect("linked entry");
    assert_eq!(
        list_transaction_entries(root.as_path())
            .expect_err("link")
            .0,
        format!(
            "Unsafe transaction recovery directory: {}",
            root.join(ID).display()
        )
    );
    std::fs::remove_file(root.join(ID)).expect("unlink entry");
    std::fs::create_dir(root.join("junk")).expect("junk");
    assert!(list_transaction_entries(root.as_path()).is_err());
    std::fs::remove_dir(root.join("junk")).expect("rmdir");
    // A link on the way to the registry makes its parent noncanonical.
    let linked_parent: PathBuf = git_dir.join("linked-git");
    std::os::unix::fs::symlink(&git_dir, &linked_parent).expect("linked parent");
    assert!(
        ensure_transaction_root(linked_parent.join(TRANSACTION_ROOT_NAME).as_path())
            .expect_err("noncanonical")
            .0
            .ends_with("has a noncanonical administrative parent.")
    );
    remove(git_dir.as_path());
}

/// A non-UTF-8 entry name is unexpected state.
#[cfg(target_os = "linux")]
#[test]
fn a_non_utf8_entry_is_unexpected() {
    use std::os::unix::ffi::OsStrExt;
    let git_dir: PathBuf = fixture("registry-bytes");
    let root: PathBuf = git_dir.join(TRANSACTION_ROOT_NAME);
    std::fs::create_dir(&root).expect("registry");
    std::fs::create_dir(root.join(std::ffi::OsStr::from_bytes(b"\xff"))).expect("odd entry");
    assert!(
        list_transaction_entries(root.as_path())
            .expect_err("odd")
            .0
            .starts_with("Unexpected transaction registry entry: ")
    );
    remove(git_dir.as_path());
}

/// Removal retires a registry directory first;
///  other directories are removed in place.
#[test]
fn removal_retires_registry_directories() {
    let git_dir: PathBuf = fixture("registry-remove");
    let root: PathBuf = git_dir.join(TRANSACTION_ROOT_NAME);
    ensure_transaction_root(root.as_path()).expect("create");
    let directory: PathBuf =
        publish_transaction_directory(root.as_path(), ID, b"{}\n").expect("publish");
    std::fs::write(directory.join("preparing.json"), b"{}").expect("state");
    remove_transaction_directory(directory.as_path()).expect("remove");
    assert!(!directory.exists());
    assert!(!root.join(format!("{ID}.retired")).exists());
    assert!(
        list_transaction_entries(root.as_path())
            .expect("listing")
            .is_empty()
    );
    let legacy: PathBuf = git_dir.join(LEGACY_TRANSACTION_DIRECTORY_NAME);
    std::fs::create_dir(&legacy).expect("legacy");
    std::fs::write(legacy.join("journal.json"), b"{}").expect("journal");
    remove_transaction_directory(legacy.as_path()).expect("legacy removal");
    assert!(!legacy.exists());
    remove(git_dir.as_path());
}

/// Removing a registry directory syncs nothing: a retired directory that survives a power loss is removed by
///  recovery, and a published one that survives is cleaned as a landed transaction.
#[test]
fn removing_a_registry_directory_syncs_nothing() {
    let git_dir: PathBuf = fixture("registry-remove-sync");
    let root: PathBuf = git_dir.join(TRANSACTION_ROOT_NAME);
    ensure_transaction_root(root.as_path()).expect("create");
    let directory: PathBuf =
        publish_transaction_directory(root.as_path(), ID, b"{}\n").expect("publish");
    std::fs::write(directory.join("preparing.json"), b"{}").expect("state");
    take_directory_syncs();
    remove_transaction_directory(directory.as_path()).expect("remove");
    assert_eq!(take_directory_syncs(), Vec::<PathBuf>::new());
    remove(git_dir.as_path());
}

/// Two recoverers retire one dead transaction:
///  the one that loses the rename finds the directory gone and has nothing left to do.
#[test]
fn removing_a_directory_a_peer_already_retired_succeeds() {
    let git_dir: PathBuf = fixture("registry-remove-peer");
    let root: PathBuf = git_dir.join(TRANSACTION_ROOT_NAME);
    ensure_transaction_root(root.as_path()).expect("create");
    assert!(remove_transaction_directory(root.join(ID).as_path()).is_ok());
    let directory: PathBuf =
        publish_transaction_directory(root.as_path(), ID, b"{}\n").expect("publish");
    remove_transaction_directory(directory.as_path()).expect("first removal");
    assert!(remove_transaction_directory(directory.as_path()).is_ok());
    assert!(
        list_transaction_entries(root.as_path())
            .expect("listing")
            .is_empty()
    );
    remove(git_dir.as_path());
}

/// Whether a directory is removed in place or retired first shows only when its parent is missing:
///  the retiring rename finds nothing and succeeds, while the in-place removal must still sync the parent and fails.
///  Only a registry directory named by a transaction ID is retired.
#[test]
fn only_a_registry_directory_with_a_transaction_name_is_retired() {
    let git_dir: PathBuf = fixture("registry-remove-kinds");
    // A transaction ID under the registry's name is retired, so the missing registry is not an error.
    remove_transaction_directory(git_dir.join(TRANSACTION_ROOT_NAME).join(ID).as_path())
        .expect("a retired transaction whose registry vanished");
    // A transaction ID under another name is removed in place, and the missing parent cannot be synced.
    let plain: String = remove_transaction_directory(git_dir.join("plain").join(ID).as_path())
        .expect_err("another parent")
        .0;
    assert!(plain.starts_with("syncing the transaction registry"), "{plain}");
    // Another name under the registry's name is removed in place as well.
    let legacy: String = remove_transaction_directory(
        git_dir.join(TRANSACTION_ROOT_NAME).join("legacy").as_path(),
    )
    .expect_err("another entry name")
    .0;
    assert!(legacy.starts_with("syncing the transaction registry"), "{legacy}");
    remove(git_dir.as_path());
}

/// A path whose parent is a file cannot be inspected,
///  and that is a failure rather than an absent directory.
#[cfg(unix)]
#[test]
fn an_uninspectable_path_is_not_an_absent_directory() {
    let git_dir: PathBuf = fixture("registry-real-directory");
    let file: PathBuf = git_dir.join("file");
    std::fs::write(&file, b"x").expect("file");
    assert!(real_directory(file.join("child").as_path()).is_err());
    assert_eq!(
        real_directory(git_dir.join("absent").as_path()).expect("absent"),
        None
    );
    assert_eq!(real_directory(git_dir.as_path()).expect("directory"), Some(true));
    assert_eq!(real_directory(file.as_path()).expect("file"), Some(false));
    remove(git_dir.as_path());
}

/// The directory holding the registry is synced when the registry is created and at no other time:
///  finding a registry, whether this wrapper or another process made it, changes nothing to make durable.
#[test]
fn the_registry_parent_is_synced_only_when_the_registry_is_created() {
    let git_dir: PathBuf = fixture("registry-sync-on-create");
    let registry: PathBuf = git_dir.join(TRANSACTION_ROOT_NAME);
    take_directory_syncs();
    ensure_transaction_root(registry.as_path()).expect("created");
    assert_eq!(take_directory_syncs(), vec![git_dir.clone()]);
    ensure_transaction_root(registry.as_path()).expect("found");
    assert_eq!(take_directory_syncs(), Vec::<PathBuf>::new());
    let foreign: PathBuf = git_dir.join("made-elsewhere");
    std::fs::create_dir(&foreign).expect("foreign registry");
    ensure_transaction_root(foreign.as_path()).expect("found foreign");
    assert_eq!(take_directory_syncs(), Vec::<PathBuf>::new());
    remove(git_dir.as_path());
}

/// A registry that cannot be created for a reason other than already existing is reported as a creation failure,
///  not as an unsafe registry.
#[cfg(unix)]
#[test]
fn a_registry_that_cannot_be_created_is_reported_as_such() {
    use std::os::unix::fs::PermissionsExt;
    let git_dir: PathBuf = fixture("registry-create-refused");
    let canonical: PathBuf = std::fs::canonicalize(&git_dir).expect("canonical");
    std::fs::set_permissions(&canonical, std::fs::Permissions::from_mode(0o500)).expect("lock");
    let outcome = ensure_transaction_root(canonical.join(TRANSACTION_ROOT_NAME).as_path());
    std::fs::set_permissions(&canonical, std::fs::Permissions::from_mode(0o700)).expect("unlock");
    // A superuser creates the directory anyway, so only an ordinary user shows the refusal.
    if outcome.is_ok() {
        remove(git_dir.as_path());
        return;
    }
    assert!(
        outcome
            .expect_err("refused")
            .0
            .contains("creating the transaction registry")
    );
    remove(git_dir.as_path());
}
