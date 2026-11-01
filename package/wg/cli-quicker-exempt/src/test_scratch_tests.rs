//! The unit test of the scratch root's location.

use super::scratch_root;
use std::path::PathBuf;

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
