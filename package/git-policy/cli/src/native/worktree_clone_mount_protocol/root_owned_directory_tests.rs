//! What:
//!  Controls of the root-owned `/run` directory:
//!  a fresh parent gets its directory at its designed mode,
//!  a directory another account owns is refused,
//!  and a refused and then corrected mode is reported.
//! Why:
//!  A publish on a fresh boot meets no directory at all,
//!  so the writer must make one
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "Where the verdict is enforced,
//! and where the record lives"),
//!  and it must never write into another account's folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const directory = makeAndProveDirectory(freshChild, uid, 0o755);
//! ```

use super::{
    RootDirectoryRefusal,
    make_and_prove,
};
use crate::testing::test_support::fixture;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// This account's own number,
/// which owns every fixture folder.
fn this_uid() -> u32 {
    // SAFETY: `getuid` takes no argument and changes nothing.
    return unsafe { libc::getuid() };
}

/// A fresh directory below the fixture root,
/// which does not exist yet.
fn absent_child(name: &str) -> PathBuf {
    return fixture(name).join("fresh").join("child");
}

/// A directory below a parent that does not exist yet is made,
/// owned by the expected account and at its designed mode.
#[test]
fn a_directory_below_an_absent_parent_is_made() {
    let path: PathBuf = absent_child("directory-made");
    let directory: std::fs::File =
        make_and_prove(path.as_path(), this_uid(), 0o755).expect("the directory is made");
    assert!(path.is_dir(), "the directory exists");
    let mode: u32 = std::fs::metadata(path.as_path())
        .expect("the directory is readable")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o755);
    let facts: std::fs::Metadata = directory
        .metadata()
        .expect("the descriptor names the directory");
    assert!(facts.is_dir(), "the descriptor is the directory's own");
}

/// A directory another account owns is refused and left at its own mode,
/// so nothing is written into a folder this route does not own.
#[test]
fn a_directory_of_another_account_is_refused() {
    let path: PathBuf = absent_child("directory-other-owner");
    std::fs::create_dir_all(path.as_path()).expect("the fixture directory is made");
    std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o700))
        .expect("the fixture takes a private mode");
    let refusal: RootDirectoryRefusal = make_and_prove(path.as_path(), this_uid() + 1, 0o755)
        .expect_err("another account's folder is refused");
    assert!(
        matches!(refusal, RootDirectoryRefusal::Owner { wanted, .. } if wanted == this_uid() + 1),
        "{refusal:?}"
    );
    let mode: u32 = std::fs::metadata(path.as_path())
        .expect("the directory is readable")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o700, "a refused call changed no mode");
}

/// A link where the directory belongs is refused without changing the folder it
/// names,
/// because the open follows no link.
#[test]
fn a_link_where_the_directory_belongs_is_refused() {
    let target: PathBuf = fixture("directory-link-target");
    std::fs::set_permissions(target.as_path(), std::fs::Permissions::from_mode(0o700))
        .expect("the target takes a private mode");
    let holder: PathBuf = fixture("directory-link-holder");
    let link: PathBuf = holder.join("link");
    std::os::unix::fs::symlink(target.as_path(), link.as_path()).expect("the link is made");
    let refusal: RootDirectoryRefusal =
        make_and_prove(link.as_path(), this_uid(), 0o755).expect_err("a link is refused");
    assert!(
        matches!(refusal, RootDirectoryRefusal::Open { .. }),
        "{refusal:?}"
    );
    let mode: u32 = std::fs::metadata(target.as_path())
        .expect("the target exists")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o700, "the folder the link named was not written");
}
