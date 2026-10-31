//! Scratch directories for the unit tests,
//! placed beside the build output so no test writes under `/tmp`.

/// Imports `Path`,
/// `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};

/// The directory test scratch is created in:
/// `test-scratch` in the build directory of the running test executable
///  (three levels above it,
/// the package's build directory in the current Cargo layout),
/// never the system temporary
///  directory.
/// A shared tmpfs with a per-user quota fills up when failing tests leave directories behind (issue 1809).
fn scratch_root() -> PathBuf {
    let target: Option<PathBuf> = std::env::current_exe()
        .ok()
        .and_then(|executable| { return executable.ancestors().nth(3).map(Path::to_path_buf); });
    let root: PathBuf = target
        .unwrap_or_else(|| { return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"); })
        .join("test-scratch");
    std::fs::create_dir_all(&root).expect("make the scratch root");
    return root;
}

/// One empty directory under the scratch root,
/// removed when the returned guard drops,
/// however the test ends.
pub(crate) fn scratch() -> tempfile::TempDir {
    return tempfile::tempdir_in(scratch_root()).expect("make a scratch directory");
}

/// The unit test of the scratch directory's location.
#[cfg(test)]
#[path = "test_scratch_tests.rs"]
mod tests;
