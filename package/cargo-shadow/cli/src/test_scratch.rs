//! Scratch directories for the unit tests,
//! placed beside the build output so no test writes under `/tmp`.

/// Imports `Path`,
/// `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};

/// The directory test scratch is created in:
/// `test-scratch` in the build directory of the running test executable
/// (three levels above it,
/// the package's build directory in the current Cargo layout),
/// never the system temporary
/// directory.
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

/// The variable that lets a test whose schedule cannot be built on this host end as a printed skip instead of a failure.
pub(crate) const ALLOW_SKIPS: &str = "CARGO_SHADOW_ALLOW_SKIPS";

/// What:
/// `pub(crate) fn skip_or_fail(reason: &str)` ends a test whose schedule cannot be built on this host: it fails the test, unless
/// `CARGO_SHADOW_ALLOW_SKIPS` is set, when it prints the skip.
/// Why:
/// A test that returns green without running its schedule is evidence of nothing (sixth pass); a certification run therefore fails on
/// such a host instead of counting the test as passed, and a host that cannot build the schedule says so explicitly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function skipOrFail(reason: string): void  // throws unless CARGO_SHADOW_ALLOW_SKIPS is set
/// ```
pub(crate) fn skip_or_fail(reason: &str) {
    assert!(std::env::var_os(ALLOW_SKIPS).is_some(), "this host cannot build the schedule ({reason}); set {ALLOW_SKIPS}=1 to skip the test");
    eprintln!("skipped: {reason}");
}
