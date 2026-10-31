//! The directory unit-test fixtures are created in, placed in the build directory so no test writes under `/tmp`.

use std::path::{Path, PathBuf};

/// The directory fixtures are created in: `test-scratch` in the build directory of the running test executable
///  (three levels above it, the package's build directory in the current Cargo layout), never the system temporary
///  directory.
/// A shared tmpfs with a per-user quota fills up when tests leave fixtures behind (issue 1809).
/// Some tests need a folder with no repository, toolchain entry point, or TypeScript install above it, and a build
///  directory inside a checkout (a host build with `CARGO_BUILD_BUILD_DIR=target` in a worktree) has all of them
///  above it; then the fixtures go in the home directory's cache directory instead, which no checkout contains.
/// The container mounts only the package at `/work`, so its build directory stays the scratch location.
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

/// One empty directory under the scratch root, removed when the returned guard drops, however the test ends.
/// Returns the `io::Result` of the call it replaces, so call sites keep their `expect` or `?`.
pub(crate) fn scratch() -> std::io::Result<tempfile::TempDir> {
    return tempfile::tempdir_in(scratch_root());
}

/// The name of the first thing found at or above `directory` that tests on a disposable folder assume is absent:
///  a `.git` entry (work-tree discovery), the toolchain entry point script (`language::toolchain::ENTRY_POINT`), or the
///  TypeScript launcher (`language::config::typescript::LAUNCHER`).
/// The two paths are repeated here because the native test binary has no `language` module to import them from.
fn assumed_absent_above(directory: &Path) -> Option<String> {
    for ancestor in directory.ancestors() {
        for marker in [
            ".git",
            "package/file-enforcer/core/src/toolchain/bin-paths.ts",
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

/// Tests such as `outside_a_work_tree_nothing_is_ignored` and
///  `a_worktree_that_is_not_above_the_project_does_not_serve_it` need a folder with no repository, no toolchain entry
///  point, and no TypeScript install above it, so the scratch root must not sit inside a checkout, whatever the build
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
