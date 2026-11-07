//! The directory unit-test fixtures are created in,
//! placed in the build directory so no test writes under `/tmp`.

/// `Path` to walk up from the test executable to its build directory,
/// and `PathBuf` for the scratch root handed to tests.
use std::path::{Path, PathBuf};

/// The directory fixtures are created in:
/// `test-scratch` in the build directory of the running test executable
///  (three levels above it,
/// the package's build directory in the current Cargo layout),
/// never the system temporary
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

/// The unit test of the scratch root's location.
#[cfg(test)]
#[path = "test_scratch_tests.rs"]
mod tests;
