//! Entry creation checks on real filesystem trees.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::{MetadataExt, PermissionsExt};

/// A manifest entry shorthand.
fn entry(kind: EntryKind, mode: u32, relative_path: &[u8]) -> WorktreeCopyEntry {
    return WorktreeCopyEntry {
        kind,
        mode,
        relative_path: relative_path.to_vec(),
    };
}

/// The identity captured after creation names the object that exists at the path,
/// and carries its selection flag.
#[test]
fn captured_identity_names_the_created_object() {
    let root: PathBuf = fixture("install-entry-identity");
    std::fs::write(root.join("file"), b"x").expect("file");
    let metadata: std::fs::Metadata =
        std::fs::symlink_metadata(root.join("file")).expect("metadata");
    let captured: InstalledPath =
        capture_installed_path(root.join("file").as_path(), b"file", true).expect("captured");
    assert_eq!(captured.device, metadata.dev());
    assert_eq!(captured.inode, metadata.ino());
    assert!(captured.selected);
    assert_eq!(captured.relative_path, b"file");
    remove(root.as_path());
}

/// Missing parents are created shallow first and recorded as unselected;
/// an existing directory parent is kept unrecorded.
#[test]
fn missing_parents_are_created_and_recorded() {
    let root: PathBuf = fixture("install-entry-parents");
    std::fs::create_dir(root.join("a")).expect("existing parent");
    let mut created: Vec<InstalledPath> = Vec::new();
    ensure_parents(
        root.as_path(),
        &entry(EntryKind::File, 0o644, b"a/b/c/file"),
        &mut created,
    )
    .expect("parents");
    let paths: Vec<(&[u8], bool)> = created
        .iter()
        .map(|installed| return (installed.relative_path.as_slice(), installed.selected))
        .collect();
    assert_eq!(paths, vec![(b"a/b".as_slice(), false), (b"a/b/c".as_slice(), false)]);
    assert!(root.join("a/b/c").is_dir());
    remove(root.as_path());
}

/// A parent that exists as a file refuses the entry with the parent named.
#[test]
fn file_in_parent_position_is_refused() {
    let root: PathBuf = fixture("install-entry-file-parent");
    std::fs::write(root.join("a"), b"x").expect("file parent");
    let mut created: Vec<InstalledPath> = Vec::new();
    let failure: WorktreeCopyError = ensure_parents(
        root.as_path(),
        &entry(EntryKind::File, 0o644, b"a/file"),
        &mut created,
    )
    .expect_err("file parent");
    assert_eq!(
        failure.0,
        format!(
            "monochromatic-git-policy-cli: ignored-state parent is not a directory: \"a\" in {}.",
            quote(root.to_string_lossy().as_ref())
        )
    );
    assert!(created.is_empty());
    remove(root.as_path());
}

/// A selected directory is created private,
/// a link keeps its exact target,
/// and a file is a hard link to its stage copy.
#[cfg(unix)]
#[test]
fn selected_entries_are_created_by_kind() {
    let root: PathBuf = fixture("install-entry-kinds");
    let stage: PathBuf = root.join("stage");
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir_all(stage.join("dir")).expect("stage dir");
    std::fs::write(stage.join("file"), b"bytes").expect("stage file");
    std::os::unix::fs::symlink("../target text", stage.join("link")).expect("stage link");
    std::fs::create_dir(destination.as_path()).expect("destination");
    create_selected_entry(
        stage.as_path(),
        destination.as_path(),
        &entry(EntryKind::Directory, 0o755, b"dir"),
    )
    .expect("directory");
    assert_eq!(
        std::fs::metadata(destination.join("dir"))
            .expect("dir")
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    create_selected_entry(
        stage.as_path(),
        destination.as_path(),
        &entry(EntryKind::File, 0o640, b"file"),
    )
    .expect("file");
    let staged_inode: u64 = std::fs::metadata(stage.join("file")).expect("stage").ino();
    let installed_inode: u64 = std::fs::metadata(destination.join("file"))
        .expect("installed")
        .ino();
    assert_eq!(
        staged_inode, installed_inode,
        "a hard link shares the staged object"
    );
    create_selected_entry(
        stage.as_path(),
        destination.as_path(),
        &entry(EntryKind::Symlink, 0o777, b"link"),
    )
    .expect("link");
    assert_eq!(
        std::fs::read_link(destination.join("link")).expect("target"),
        PathBuf::from("../target text")
    );
    remove(root.as_path());
}

/// Only the link-refusal codes fall back to a copy;
/// any other operating-system error is reported as it is.
#[cfg(unix)]
#[test]
fn only_link_refusals_fall_back_to_copy() {
    for code in [
        libc::EXDEV,
        libc::EPERM,
        libc::EMLINK,
        libc::ENOSYS,
        libc::EOPNOTSUPP,
    ] {
        assert!(
            is_link_unsupported(&std::io::Error::from_raw_os_error(code)),
            "code {code}"
        );
    }
    assert!(!is_link_unsupported(&std::io::Error::from_raw_os_error(
        libc::EIO
    )));
    assert!(!is_link_unsupported(&std::io::Error::other("no code")));
}

/// A file whose target already exists is never replaced by the link.
#[cfg(unix)]
#[test]
fn existing_destination_file_is_not_replaced() {
    let root: PathBuf = fixture("install-entry-exists");
    let stage: PathBuf = root.join("stage");
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir_all(stage.as_path()).expect("stage");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    std::fs::write(stage.join("file"), b"staged").expect("staged");
    std::fs::write(destination.join("file"), b"existing").expect("existing");
    let failure: WorktreeCopyError = create_selected_entry(
        stage.as_path(),
        destination.as_path(),
        &entry(EntryKind::File, 0o644, b"file"),
    )
    .expect_err("exists");
    assert!(failure.0.contains("File exists"));
    assert_eq!(
        std::fs::read(destination.join("file")).expect("kept"),
        b"existing"
    );
    remove(root.as_path());
}

/// The link path publishes the staged object itself,
/// so the staged file takes the entry's mode before anything is linked:
/// a destination never shows a wider mode than its source,
/// not even in the instant its name first exists.
#[cfg(unix)]
#[test]
fn a_linked_file_never_shows_a_wider_mode_than_its_source() {
    let root: PathBuf = fixture("install-entry-link-mode");
    let stage: PathBuf = root.join("stage");
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir_all(stage.as_path()).expect("stage");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    std::fs::write(stage.join("key.pem"), b"KEY=credential\n").expect("staged");
    // The staged copy was made with the process's umask, as a copy of a file is, and holds a credential.
    std::fs::set_permissions(stage.join("key.pem"), std::os::unix::fs::PermissionsExt::from_mode(0o644)).expect("umask mode");
    create_selected_entry(
        stage.as_path(),
        destination.as_path(),
        &entry(EntryKind::File, 0o600, b"key.pem"),
    )
    .expect("installed");
    let staged: std::fs::Metadata = std::fs::symlink_metadata(stage.join("key.pem")).expect("staged");
    let installed: std::fs::Metadata = std::fs::symlink_metadata(destination.join("key.pem")).expect("installed");
    assert_eq!(staged.ino(), installed.ino(), "this control takes the link path");
    assert_eq!(
        installed.permissions().mode() & 0o777,
        0o600,
        "the destination's first instant is its source's mode"
    );
    remove(root.as_path());
}

/// A destination on another device than the stage,
/// as a btrfs subvolume beside the stage is:
/// the link is refused,
/// and the copy carries the entry's mode at the destination name's first look.
/// Where no second device is available,
/// the control states that and checks nothing.
#[cfg(unix)]
#[test]
fn a_copied_file_on_another_device_keeps_its_mode() {
    let shared_memory: &Path = Path::new("/dev/shm");
    let stage: PathBuf = fixture("install-entry-other-device-stage");
    let on_other_device: bool = match (std::fs::metadata(shared_memory), std::fs::metadata(stage.as_path())) {
        (Ok(inside), Ok(outside)) => {
            use std::os::unix::fs::MetadataExt as _;
            inside.dev() != outside.dev()
        }
        _ => false,
    };
    if !on_other_device {
        eprintln!("DEVICE CONTROL NOT RUN: /dev/shm is not a separate device here, so the copy fallback of the install had no destination on another device");
        remove(stage.as_path());
        return;
    }
    let destination: PathBuf = shared_memory.join(format!("monochromatic-install-entry-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(destination.as_path());
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    std::fs::write(stage.join("key.pem"), b"KEY=credential\n").expect("staged");
    std::fs::set_permissions(stage.join("key.pem"), std::os::unix::fs::PermissionsExt::from_mode(0o600)).expect("stage mode");
    create_selected_entry(
        stage.as_path(),
        destination.as_path(),
        &entry(EntryKind::File, 0o600, b"key.pem"),
    )
    .expect("copied");
    let installed: std::fs::Metadata = std::fs::symlink_metadata(destination.join("key.pem")).expect("installed");
    assert_eq!(
        installed.permissions().mode() & 0o777,
        0o600,
        "a copied credential carries its source's mode"
    );
    assert_eq!(std::fs::read(destination.join("key.pem")).expect("bytes"), b"KEY=credential\n");
    let left: Vec<String> = std::fs::read_dir(destination.as_path())
        .expect("destination")
        .flatten()
        .map(|listed| return listed.file_name().to_string_lossy().into_owned())
        .filter(|name| return name.starts_with(".monochromatic-git-policy-cli-partial-"))
        .collect();
    assert_eq!(left, Vec::<String>::new(), "no private name of the copy stays beside the entry");
    remove(stage.as_path());
    let _ = std::fs::remove_dir_all(destination.as_path());
}
