//! The directory unit-test fixtures are created in,
//! placed in the build directory so no test writes under `/tmp`.

/// Import the path types the scratch root is built from.
use std::path::{Path, PathBuf};

/// Whether a directory,
/// or an ancestor of it,
/// holds a Cargo.toml and so would own a file created below it.
fn has_owning_manifest(directory: &Path) -> bool {
    return directory.ancestors().any(|ancestor| {
        return ancestor.join("Cargo.toml").is_file();
    });
}

/// The directory fixtures are created in:
/// `test-scratch` in the build directory of the running test executable
///  (three levels above it,
/// the package's build directory in the current Cargo layout),
/// never the system temporary
///  directory.
/// A shared tmpfs with a per-user quota fills up when tests leave fixtures behind (issue 1809).
/// Some tests need a file that no Cargo.toml owns,
/// and a build directory inside the package (the default layout,
/// as in
///  the test container) sits below the package's own manifest;
/// then the fixtures go in the home directory's cache
///  directory instead,
/// which no manifest owns.
pub(crate) fn scratch_root() -> PathBuf {
    let target: Option<PathBuf> = std::env::current_exe().ok().and_then(|executable| {
        return executable.ancestors().nth(3).map(Path::to_path_buf);
    });
    let in_build: PathBuf = target
        .unwrap_or_else(|| {
            return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");
        })
        .join("test-scratch");
    let root: PathBuf = match std::env::var_os("HOME") {
        Some(home) if has_owning_manifest(&in_build) => PathBuf::from(home)
            .join(".cache")
            .join("monochromatic-lint-test-scratch"),
        _ => in_build,
    };
    std::fs::create_dir_all(&root).expect("make the scratch root");
    return root;
}

/// Controls of the scratch root stay outside release artifacts.
#[cfg(test)]
#[path = "test_scratch_tests.rs"]
mod tests;
