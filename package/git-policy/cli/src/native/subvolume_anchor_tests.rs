//! Controls for the anchored directory: components opened without following links, children addressed through the
//!  descriptor, and a directory that was replaced after it was opened.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::{MetadataExt, PermissionsExt};

/// A directory is opened below a root, and a child path made from it reaches the real directory.
#[test]
fn a_directory_below_the_root_is_opened_and_its_children_are_reached() {
    let parent: PathBuf = fixture("anchor-open");
    std::fs::create_dir_all(parent.join("a/b")).expect("directories");
    let anchor: Anchor = Anchor::open_below(parent.as_path(), Path::new("a/b")).expect("open");
    std::fs::write(anchor.child(OsStr::new("file")), b"x").expect("write through the anchor");
    assert_eq!(std::fs::read(parent.join("a/b/file")).expect("read"), b"x");
    assert!(Anchor::open_below(parent.as_path(), Path::new("")).is_ok());
    remove(parent.as_path());
}

/// A component that is a symbolic link, a file or missing is refused, and a path that climbs out of the root is refused.
#[test]
fn a_linked_or_missing_component_is_refused() {
    let parent: PathBuf = fixture("anchor-refuse");
    std::fs::create_dir_all(parent.join("real")).expect("directory");
    std::os::unix::fs::symlink(parent.join("real"), parent.join("linked")).expect("link");
    std::fs::write(parent.join("plain"), b"x").expect("file");
    let code = |name: &str| -> Option<i32> { return Anchor::open_below(parent.as_path(), Path::new(name)).expect_err("refused").raw_os_error(); };
    // The kernel answers `ENOTDIR` for a link opened with `O_DIRECTORY | O_NOFOLLOW`, `ELOOP` without `O_DIRECTORY`.
    assert!(matches!(code("linked"), Some(libc::ELOOP | libc::ENOTDIR)), "{:?}", code("linked"));
    assert_eq!(code("plain"), Some(libc::ENOTDIR));
    assert_eq!(code("absent"), Some(libc::ENOENT));
    assert_eq!(Anchor::open_below(parent.as_path(), Path::new("../escape")).expect_err("refused").raw_os_error(), Some(libc::EINVAL));
    remove(parent.as_path());
}

/// A directory replaced by a link after it was opened is still the directory that was checked.
#[test]
fn a_replaced_directory_does_not_redirect_the_children_of_an_open_anchor() {
    let parent: PathBuf = fixture("anchor-replaced");
    std::fs::create_dir_all(parent.join("d")).expect("directory");
    std::fs::create_dir_all(parent.join("outside")).expect("outside");
    let anchor: Anchor = Anchor::open_below(parent.as_path(), Path::new("d")).expect("open");
    std::fs::rename(parent.join("d"), parent.join("moved")).expect("move aside");
    std::os::unix::fs::symlink(parent.join("outside"), parent.join("d")).expect("link in its place");
    std::fs::write(anchor.child(OsStr::new("file")), b"x").expect("write through the anchor");
    assert!(parent.join("moved/file").exists() && !parent.join("outside/file").exists());
    remove(parent.as_path());
}

/// A directory the owner cannot read is made searchable so a pass can fill it.
#[test]
fn a_directory_without_owner_permission_is_opened_after_widening_it() {
    // SAFETY: `geteuid` takes no arguments and cannot fail; root opens any directory, so there is nothing to widen.
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    let parent: PathBuf = fixture("anchor-closed");
    std::fs::create_dir_all(parent.join("closed")).expect("directory");
    std::fs::set_permissions(parent.join("closed"), std::fs::Permissions::from_mode(0o000)).expect("closed");
    let anchor: Anchor = Anchor::open_below(parent.as_path(), Path::new("closed")).expect("open");
    std::fs::write(anchor.child(OsStr::new("file")), b"x").expect("write");
    assert_eq!(std::fs::metadata(parent.join("closed")).expect("metadata").mode() & 0o700, 0o700);
    remove(parent.as_path());
}

/// A directory the owner cannot read is widened by adding the owner's bits, not by flipping them: a mode that keeps some
///  of them (write, search, or both) ends with all three, and a child can be created in it.
#[test]
fn a_directory_that_keeps_some_owner_bits_gets_all_of_them() {
    // SAFETY: `geteuid` takes no arguments and cannot fail; root opens any directory, so there is nothing to widen.
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    let parent: PathBuf = fixture("anchor-partly-closed");
    for (name, mode) in [("search", 0o100u32), ("write", 0o200), ("both", 0o300)] {
        std::fs::create_dir_all(parent.join(name)).expect("directory");
        std::fs::set_permissions(parent.join(name), std::fs::Permissions::from_mode(mode)).expect("closed");
        let anchor: Anchor = Anchor::open_below(parent.as_path(), Path::new(name)).expect("open");
        assert_eq!(std::fs::metadata(parent.join(name)).expect("metadata").mode() & 0o700, 0o700, "{name}");
        std::fs::write(anchor.child(OsStr::new("file")), b"x").expect("write");
    }
    remove(parent.as_path());
}
