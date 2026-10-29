//! The directory unit-test fixtures are created in, placed in the build directory so no test writes under `/tmp`.

use std::path::{Path, PathBuf};

/// The directory fixtures are created in: `test-scratch` in the build directory of the running test executable
///  (three levels above it, the package's build directory in the current Cargo layout), never the system temporary
///  directory.
/// A shared tmpfs with a per-user quota fills up when tests leave fixtures behind (issue 1809).
pub(crate) fn scratch_root() -> PathBuf {
    let target: Option<PathBuf> = std::env::current_exe()
        .ok()
        .and_then(|executable| { return executable.ancestors().nth(3).map(Path::to_path_buf); });
    let root: PathBuf = target
        .unwrap_or_else(|| { return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"); })
        .join("test-scratch");
    std::fs::create_dir_all(&root).expect("make the scratch root");
    return root;
}

/// One empty directory under the scratch root, removed when the returned guard drops, however the test ends.
/// Returns the `io::Result` of the call it replaces, so call sites keep their `expect` or `?`.
pub(crate) fn scratch() -> std::io::Result<tempfile::TempDir> {
    return tempfile::tempdir_in(scratch_root());
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
