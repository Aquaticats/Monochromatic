//! Scratch directories for the unit tests,
//! placed beside the build output so no test writes under `/tmp`.

/// What:
/// `mod fixture_root_taking_any_name;` declares the question whether this fixture root takes a name that is not UTF-8,
/// and the re-execution that mounts a tmpfs there when it does not.
/// Why:
/// A control that makes such a name cannot run where the file system refuses it,
/// as this host's build dataset does under `utf8only=on` (issue 6550),
/// and a control that quietly skipped there would pass a run in which it never ran.
/// It is a submodule of this one because the folder it asks about is this module's own,
/// and because the crate root's code lines are at their limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // test-scratch/fixture-root-taking-any-name.test-support.ts, imported by the tests that make such a name
/// ```
pub(crate) mod fixture_root_taking_any_name;

/// Imports `Path`,
/// `PathBuf` from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// The directory test scratch is created in:
/// `test-scratch` in the build directory of the running test executable
/// (three levels above it,
/// the package's build directory in the current Cargo layout),
/// never the system temporary
/// directory.
/// Why it is not private to this module:
/// `fixture_root_taking_any_name` asks this very directory whether it takes a name that is not UTF-8,
/// and mounts a tmpfs there when it does not (issue 6550).
pub(crate) fn scratch_root() -> PathBuf {
    let target: Option<PathBuf> = std::env::current_exe().ok().and_then(|executable| {
        return executable.ancestors().nth(3).map(Path::to_path_buf);
    });
    let root: PathBuf = target
        .unwrap_or_else(|| {
            return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");
        })
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
/// `pub(crate) fn skip_or_fail(reason: &str)` ends a test whose schedule cannot be built on this host:
/// it fails the test,
/// unless
/// `CARGO_SHADOW_ALLOW_SKIPS` is set,
/// when it prints the skip.
/// Why:
/// A test that returns green without running its schedule is evidence of nothing (sixth pass);
/// a certification run therefore fails on
/// such a host instead of counting the test as passed,
/// and a host that cannot build the schedule says so explicitly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function skipOrFail(reason: string): void  // throws unless CARGO_SHADOW_ALLOW_SKIPS is set
/// ```
pub(crate) fn skip_or_fail(reason: &str) {
    assert!(
        std::env::var_os(ALLOW_SKIPS).is_some(),
        "this host cannot build the schedule ({reason}); set {ALLOW_SKIPS}=1 to skip the test"
    );
    eprintln!("skipped: {reason}");
}
