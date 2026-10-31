//! The directory unit-test fixtures are created in,
//! placed in the build directory so no test writes under `/tmp`.

/// Imports `Path` and `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};

/// The directory fixtures are created in:
/// `test-scratch` in the build directory of the running test executable,
///  three levels above it,
///  the package's build directory in the current Cargo layout,
///  never the system temporary directory.
/// A shared tmpfs with a per-user quota fills up when tests leave fixtures behind (issue 1809).
pub(crate) fn scratch_root() -> PathBuf {
    let target: Option<PathBuf> = std::env::current_exe()
        .ok()
        .and_then(executable_build_directory);
    let root: PathBuf = target
        .unwrap_or_else(manifest_target_directory)
        .join("test-scratch");
    std::fs::create_dir_all(&root).expect("make the scratch root");
    return root;
}

/// The build directory three levels above a test executable.
fn executable_build_directory(executable: PathBuf) -> Option<PathBuf> {
    return executable.ancestors().nth(3).map(Path::to_path_buf);
}

/// The package's `target` directory,
/// used when the executable path has too few ancestors.
fn manifest_target_directory() -> PathBuf {
    return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");
}

/// The unit test of the scratch root's location.
#[cfg(test)]
#[path = "test_scratch_tests.rs"]
mod tests;
