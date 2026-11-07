//! The unit tests of the scratch root's location:
//! outside a checkout and outside the system temporary directory.

use super::{assumed_absent_above, scratch_root};
use std::path::PathBuf;

/// Tests such as `outside_a_work_tree_nothing_is_ignored` and
///  `a_worktree_that_is_not_above_the_project_does_not_serve_it` need a folder with no repository,
/// no toolchain entry
///  point,
/// and no TypeScript install above it,
/// so the scratch root must not sit inside a checkout,
/// whatever the build
///  directory is.
#[test]
fn the_scratch_root_has_no_repository_entry_point_or_typescript_install_above_it() {
    let created: PathBuf = std::fs::canonicalize(scratch_root()).expect("scratch root");
    assert_eq!(
        assumed_absent_above(&created),
        None,
        "{} has a checkout marker above it",
        created.display()
    );
}

/// The test reads the system temporary directory only to prove the scratch root is not inside it.
#[test]
#[expect(clippy::disallowed_methods, reason = "compares against the banned location without writing there")]
fn the_scratch_root_is_not_under_the_system_temporary_directory() {
    let temporary: PathBuf = std::fs::canonicalize(std::env::temp_dir()).expect("system temporary directory");
    let created: PathBuf = std::fs::canonicalize(scratch_root()).expect("scratch root");
    assert!(
        !created.starts_with(&temporary),
        "{} is under {}",
        created.display(),
        temporary.display()
    );
}
