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
/// Some tests need a folder with no repository,
/// toolchain entry point,
/// or TypeScript install above it,
/// and a build
///  directory inside a checkout (a host build with `CARGO_BUILD_BUILD_DIR=target` in a worktree) has all of them
///  above it;
/// then the fixtures go in the home directory's cache directory instead,
/// which no checkout contains.
/// The container mounts only the package at `/work`,
/// so its build directory stays the scratch location.
pub(crate) fn scratch_root() -> PathBuf {
    let target: Option<PathBuf> = std::env::current_exe()
        .ok()
        .and_then(|executable| { return executable.ancestors().nth(3).map(Path::to_path_buf); });
    let in_build: PathBuf = target
        .unwrap_or_else(|| { return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"); })
        .join("test-scratch");
    let root: PathBuf = match std::env::var_os("HOME") {
        Some(home) if assumed_absent_above(&in_build).is_some() => {
            PathBuf::from(home).join(".cache").join("monochromatic-ide-test-scratch")
        }
        _ => in_build,
    };
    std::fs::create_dir_all(&root).expect("make the scratch root");
    return root;
}

/// One empty directory under the scratch root,
/// removed when the returned guard drops,
/// however the test ends.
/// Returns the `io::Result` of the call it replaces,
/// so call sites keep their `expect` or `?`.
pub(crate) fn scratch() -> std::io::Result<tempfile::TempDir> {
    return tempfile::tempdir_in(scratch_root());
}

/// The name of the first thing found at or above `directory` that tests on a disposable folder assume is absent:
///  a `.git` entry (work-tree discovery),
/// the toolchain plan file (`language::toolchain::PLAN_FILE`),
/// or the
///  TypeScript launcher (`language::config::typescript::LAUNCHER`).
/// The two paths are repeated here because the native test binary has no `language` module to import them from.
fn assumed_absent_above(directory: &Path) -> Option<String> {
    for ancestor in directory.ancestors() {
        for marker in [
            ".git",
            "toolchain.plan.jsonc",
            "node_modules/typescript/bin/tsc",
        ] {
            let found: PathBuf = ancestor.join(marker);
            if found.exists() {
                return Some(found.display().to_string());
            }
        }
    }
    return None;
}

/// The unit tests of the scratch root's location.
#[cfg(test)]
#[path = "test_scratch_tests.rs"]
mod tests;
