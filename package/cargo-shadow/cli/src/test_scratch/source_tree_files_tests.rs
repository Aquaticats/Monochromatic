//! What:
//! The control of the one walk a guard reads this crate's sources through:
//! a folder link is never entered,
//! a file link is never read as a file of its own,
//! and a folder that cannot be read is an error rather than an empty listing.
//! Why:
//! `AGENTS.md` rules `LYD`,
//! `LYL` and `LYF` put a `uses/` folder of links in a crate's source folders,
//! and a walk that followed them would read one source once per chain of links above it
//! (measured for the wrapper package on 2026-11-27,
//! issue 6580:
//! 165,978 entries and 163,498 `.rs` names where the tree holds 4,234 entries and 3,905 Rust sources).
//! This crate's `src` holds no link on 2026-11-27,
//! so this control is what holds the rule for the guards that read the sources.
//! A guard that read an unreadable folder as an empty one would pass while reading nothing,
//! which is why the second control asks for the error.

/// What:
/// `scratch` is one empty folder under the build output,
/// removed when its guard drops.
/// Why:
/// The fixture is a tree this test makes and can link into,
/// never a folder of the repository and never `/tmp`.
use crate::test_scratch::scratch;
/// What:
/// `skip_or_fail` ends a control whose schedule this host cannot build:
/// it fails the test,
/// unless `CARGO_SHADOW_ALLOW_SKIPS` is set.
/// Why:
/// A control that returned green without asking its question would be evidence of nothing.
use crate::test_scratch::skip_or_fail;
/// What:
/// `source_tree_files` is the walk under control,
/// which answers every regular file below one folder and never enters a link.
/// Why:
/// The guards of this crate read the sources through it,
/// so a defect in it is a defect in every one of them.
use crate::test_scratch::source_tree_files::source_tree_files;
/// What:
/// `fs` is the file system's own functions.
/// Why:
/// The fixture is made,
/// linked and listed with them.
use std::fs;
/// What:
/// `symlink` makes a symbolic link,
/// and `PermissionsExt` adds `from_mode` to a set of permissions.
/// Why:
/// The first control needs links and the second a folder this user may not read.
use std::os::unix::fs::{
    PermissionsExt,
    symlink,
};
/// What:
/// `Path` is a borrowed path and `PathBuf` an owned one.
/// Why:
/// The expected listing is written as paths and compared with the walk's owned answers.
use std::path::{
    Path,
    PathBuf,
};

/// A folder link is never entered and a file link is never read:
/// the walk answers each real file of the tree once.
#[test]
fn a_folder_link_is_never_entered_and_a_file_link_is_never_read() {
    // What: the fixture root, an empty folder this test owns until it ends.
    let root: tempfile::TempDir = scratch();
    fs::create_dir_all(root.path().join("real").join("next")).expect("make a folder");
    fs::create_dir_all(root.path().join("other")).expect("make a folder");
    fs::write(root.path().join("real").join("code.rs"), "the one source").expect("write a source");
    fs::write(
        root.path().join("real").join("next").join("deeper.rs"),
        "a deeper source",
    )
    .expect("write a source");
    fs::write(
        root.path().join("other").join("hidden.rs"),
        "another folder's source",
    )
    .expect("write a source");
    // What: a folder link that points back at the root of the walk.
    // Why: a walk that entered it would list the tree again under `real/loop` and never end.
    symlink(root.path(), root.path().join("real").join("loop")).expect("make a folder link");
    // What: a folder link to a folder of the same tree.
    // Why: a walk that entered it would answer `other/hidden.rs` a second time, as `real/alias/hidden.rs`.
    symlink(
        root.path().join("other"),
        root.path().join("real").join("alias"),
    )
    .expect("make a folder link");
    // What: a file link to a source the tree holds.
    // Why: a walk that read it would read the same bytes a second time under another name.
    symlink(
        root.path().join("other").join("hidden.rs"),
        root.path().join("real").join("twin.rs"),
    )
    .expect("make a file link");

    let found: Vec<PathBuf> = source_tree_files(root.path()).expect("walk the fixture");
    // What: the three real files of the fixture, in the one order the walk sorts them.
    let expected: Vec<PathBuf> = vec![
        Path::new("other").join("hidden.rs"),
        Path::new("real").join("code.rs"),
        Path::new("real").join("next").join("deeper.rs"),
    ];
    assert_eq!(
        found, expected,
        "the walk entered a link, read a file twice, or missed a real file"
    );
}

/// A folder this user may not read is an error from the walk,
/// never an empty listing.
#[test]
fn a_folder_that_cannot_be_read_is_an_error_and_not_an_empty_listing() {
    let root: tempfile::TempDir = scratch();
    // What: the folder the fixture closes after writing one source into it.
    let closed: PathBuf = root.path().join("closed");
    fs::create_dir_all(&closed).expect("make a folder");
    fs::write(closed.join("source.rs"), "a source").expect("write a source");
    // What: `from_mode(0o000)` takes every permission from the folder, so a plain user cannot list it.
    fs::set_permissions(&closed, fs::Permissions::from_mode(0o000)).expect("close the folder");
    // What: whether this user reads the folder anyway, which a privileged user does.
    // Why: the arm cannot be built for such a user, and a control that ran it anyway would prove nothing.
    let readable: bool = fs::read_dir(&closed).is_ok();
    let walked: std::io::Result<Vec<PathBuf>> = source_tree_files(root.path());
    // What: `from_mode(0o755)` gives the folder its ordinary permissions back, so the fixture can be removed.
    fs::set_permissions(&closed, fs::Permissions::from_mode(0o755)).expect("open the folder again");
    if readable {
        skip_or_fail(
            "this user reads a folder that grants no permission, so the walk cannot be shown to refuse it",
        );
        return;
    }
    assert!(
        walked.is_err(),
        "an unreadable folder was read as an empty listing instead of an error: {walked:?}"
    );
}
