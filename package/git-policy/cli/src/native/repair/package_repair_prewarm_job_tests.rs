//! Controls of the service's filling:
//! where it stages,
//! what it refuses,
//! and what it never replaces.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// A Cargo program that is not there,
/// so a control reaches the clean and never runs a build.
const NO_CARGO: &str = "/nonexistent/monochromatic-prewarm-control/cargo";

/// What a control fills with when it expects the clean to stop it.
fn fill_of_nowhere() -> Fill {
    return Fill { cargo: PathBuf::from(NO_CARGO), limits: ToolLimits { window: std::time::Duration::from_millis(200), ceiling: std::time::Duration::from_secs(5) } };
}

/// One workspace whose source build directory holds one dependency build,
/// so a clone has something to copy.
fn entry_of(root: &Path) -> PrewarmEntry {
    return PrewarmEntry {
        root: root.join("package/linter/rust"),
        from: root.join("source-build"),
        to: root.join("worktree-build"),
        members: vec![String::from("monochromatic-lint")],
    };
}

/// What:
///  The staging directory sits beside the build directory it fills,
/// so publishing it is one rename,
/// and its name is that directory's plus a suffix of this job's own.
/// Why:
///   A name inside the build directory would publish it under the wrong one;
/// a name elsewhere could be on another filesystem,
/// where the rename that never replaces cannot work.
#[test]
fn the_staging_directory_sits_beside_the_build_directory_it_fills() {
    let destination: &Path = Path::new("/var/mnt/encrypted/cargo-build/6f/0123456789abcd");
    let staged: PathBuf = staging_of(destination);
    assert_eq!(staged.parent(), destination.parent(), "{}", staged.display());
    assert_eq!(staged.file_name().and_then(|name| return name.to_str()), Some(format!("0123456789abcd{STAGING_SUFFIX}").as_str()));
}

/// What:
///  The clean deletes from directories of this job's own,
/// never from the worktree's target directory.
/// Why:
///  `cargo clean --package` removes a package's final artifacts from the target directory as well as
/// its intermediates,
/// and a worktree's target directory may hold what a build made.
#[test]
fn the_clean_deletes_from_directories_of_its_own() {
    let destination: &Path = Path::new("/var/mnt/encrypted/cargo-build/6f/0123456789abcd");
    let staged: PathBuf = staging_of(destination);
    let target: PathBuf = target_of(staged.as_path());
    assert_eq!(target.parent(), staged.parent(), "{} is not beside {}", target.display(), staged.display());
    assert_ne!(target, destination.join("target"), "the worktree's own target directory is not the one the clean names");
}

/// What:
///  A destination a build already filled is kept byte for byte,
/// and nothing is staged beside it.
/// Why:
///   The filling only ever adds state a build would otherwise compile;
/// replacing what a build made would discard work the person did not ask anyone to touch.
#[test]
fn a_build_directory_that_exists_is_never_replaced() {
    let root: PathBuf = fixture("prewarm-kept");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.to.as_path()).expect("the build directory a build made");
    std::fs::write(entry.to.join("kept"), b"a build's own artifact").expect("the file");
    let filled: Filled = super::fill_one_in_a_control(&fill_of_nowhere(), &entry).expect("a filled destination is kept");
    assert_eq!(filled, Filled::Kept, "the worktree keeps what it made");
    assert_eq!(std::fs::read(entry.to.join("kept")).expect("the file"), b"a build's own artifact");
    assert!(!staging_of(entry.to.as_path()).exists(), "nothing is staged beside a directory that is already there");
    remove(root.as_path());
}

/// What:
///  A staging directory a killed run left is removed before the copy is made,
/// and neither the destination nor the staging directory survives a clean that fails.
/// Why:
///   The staging name belongs to this job alone,
/// so what it holds is never a person's work,
/// and a failure must leave no half-filled directory for a later run to trip over.
#[test]
fn a_failed_clean_leaves_no_staging_directory_and_no_destination() {
    let root: PathBuf = fixture("prewarm-failed-clean");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.from.join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    let staged: PathBuf = staging_of(entry.to.as_path());
    std::fs::create_dir_all(staged.join("left-by-a-killed-run")).expect("a staging directory an earlier run left");
    let failed: Result<Filled, String> = super::fill_one_in_a_control(&fill_of_nowhere(), &entry);
    assert!(failed.is_err(), "a clean that could not run is reported: {failed:?}");
    assert!(!staged.exists(), "the staging directory is removed");
    assert!(!entry.to.exists(), "and nothing is published under the worktree's name");
    remove(root.as_path());
}

/// What:
///  A staging copy is published under the build directory's name when nothing is there,
/// and a second publish over an existing directory answers that it was not published.
/// Why:
///   Two runs of the filling can overlap when a service restarts,
/// and the second must not replace the first,
/// nor the directory a build made.
#[test]
fn publishing_a_staging_copy_never_replaces_a_directory() {
    let root: PathBuf = fixture("prewarm-publish");
    let destination: PathBuf = root.join("build");
    let staged: PathBuf = staging_of(destination.as_path());
    std::fs::create_dir_all(staged.as_path()).expect("the staging copy");
    std::fs::write(staged.join("a-dependency-build"), b"dependency").expect("the file");
    assert_eq!(super::publish(staged.as_path(), destination.as_path()), Ok(true), "a directory nothing had is published");
    assert_eq!(std::fs::read(destination.join("a-dependency-build")).expect("the published file"), b"dependency");
    std::fs::create_dir_all(staged.as_path()).expect("a second staging copy");
    assert_eq!(super::publish(staged.as_path(), destination.as_path()), Ok(false), "a directory that exists is not replaced");
    assert_eq!(std::fs::read(destination.join("a-dependency-build")).expect("the kept file"), b"dependency");
    remove(root.as_path());
}

/// The Cargo this control builds its fixture with,
/// which the gate image provides.
fn cargo_on_the_search_path() -> PathBuf {
    let environment: RepairEnvironment = RepairEnvironment::from_process();
    return super::cargo_for(Path::new("/nonexistent"), &environment).expect("this control needs cargo on the search path");
}

/// The member the fixture's workspace builds,
/// which the filling must clean out of the copy.
const MEMBER: &str = "own";

/// The registry dependency the fixture builds,
/// which the copy exists to carry and must keep.
const DEPENDENCY: &str = "serde";

/// What:
///  The two Cargo settings the filling is checked under:
///  incremental compilation off,
/// which is what the container gate sets and what leaves a crate's fingerprint and dep-info alone,
///  and on,
/// which leaves a directory of its own per crate as well.
/// Why:
///   The first release of this control looked only where incremental compilation puts a crate's state,
/// so it passed on a host that has incremental on,
/// and failed in the gate's image,
/// which sets `CARGO_INCREMENTAL=0` (`bin/native-test-containerfile.ts`).
/// Both settings are checked here,
/// wherever this runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const INCREMENTAL_SETTINGS = ['0', '1'];
/// ```
const INCREMENTAL_SETTINGS: [&str; 2] = ["0", "1"];

/// What:
///  Every entry of a build directory whose file name starts with `prefix`.
/// Why:
///   The promise is about a crate's artifacts,
/// not about the directory
/// Cargo happens to keep them in:
/// a crate's fingerprint,
/// dep-info and incremental directory are all keyed by its name,
/// and which of them exist depends on settings this control does not own.
/// The walk is one pass over a stack and follows no link,
/// so a copy cannot lead it outside the directory it was given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entriesNamed(build: string, prefix: string): string[];
/// ```
fn entries_named(build: &Path, prefix: &str) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![build.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(listing) = std::fs::read_dir(directory.as_path()) else { continue };
        for entry in listing.flatten() {
            let path: PathBuf = entry.path();
            if entry.file_name().to_string_lossy().starts_with(prefix) {
                found.push(path.clone());
            }
            // A link is not followed, so the walk stays inside the build directory it was given.
            if entry.file_type().is_ok_and(|kind: std::fs::FileType| return kind.is_dir()) {
                pending.push(path);
            }
        }
    }
    found.sort();
    return found;
}

/// What:
///  A record naming one workspace is filled:
/// the copy is published under the worktree's build directory name,
/// the worktree's own crate is cleaned out of it,
/// the dependency builds it exists for are kept,
/// and the source checkout's build directory is untouched.
/// Why:
///   This is the whole guarantee of the change,
/// end to end and with a real Cargo:
/// the add stopped doing this work,
/// so the service's filling must do the same clean the add used to,
/// and it must never clean the source's own state,
/// which every other worktree of the repository shares.
/// Both incremental settings run,
/// because the gate's image turns incremental compilation off and a host has it on,
/// and the crate's state lives in different places under the two.
#[test]
fn the_filling_publishes_a_clean_copy_and_leaves_the_source_alone() {
    let own_prefix: String = format!("{MEMBER}-");
    for incremental in INCREMENTAL_SETTINGS {
        let root: PathBuf = fixture(format!("prewarm-filling-incremental-{incremental}").as_str());
        let workspace: PathBuf = root.join("worktree/package/own-ws");
        std::fs::create_dir_all(workspace.join("src")).expect("the workspace");
        let manifest: String = format!("[workspace]\nmembers = [\".\"]\n\n[package]\nname = \"{MEMBER}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n{DEPENDENCY} = \"1\"\n");
        std::fs::write(workspace.join("Cargo.toml"), manifest).expect("the manifest");
        std::fs::write(workspace.join("src/lib.rs"), format!("pub fn {MEMBER}() -> &'static str {{ \"{MEMBER}\" }}\n")).expect("the crate");
        let source_build: PathBuf = root.join("source-build");
        let cargo: PathBuf = cargo_on_the_search_path();
        let built: std::process::Output = std::process::Command::new(cargo)
            .args(["build", "--offline", "--quiet"])
            .current_dir(workspace.as_path())
            .env(BUILD_DIRECTORY_VARIABLE, source_build.as_os_str())
            .env(TARGET_DIRECTORY_VARIABLE, root.join("source-target").as_os_str())
            .env("CARGO_INCREMENTAL", incremental)
            .output()
            .expect("cargo runs");
        assert!(built.status.success(), "the fixture builds with CARGO_INCREMENTAL={incremental}: {}", String::from_utf8_lossy(built.stderr.as_slice()));
        let destination_build: PathBuf = root.join("worktree-build");
        let entry: PrewarmEntry = PrewarmEntry { root: workspace.clone(), from: source_build.clone(), to: destination_build.clone(), members: vec![String::from(MEMBER)] };
        let item: WorkItem = crate::package_repair_prewarm::item_of(root.as_path(), workspace.as_path(), std::slice::from_ref(&entry)).expect("the record");
        let verdict: Verdict = crate::lazy_runtime::run_on_runtime(super::prewarm_awaiting(&item)).expect("the filling runs");
        assert_eq!(verdict, Verdict::Done, "the filling published the copy (CARGO_INCREMENTAL={incremental})");
        assert!(!entries_named(source_build.as_path(), own_prefix.as_str()).is_empty(), "the source checkout's own crate is still built there (CARGO_INCREMENTAL={incremental})");
        assert!(
            entries_named(destination_build.as_path(), own_prefix.as_str()).is_empty(),
            "the copy holds no artifact of the worktree's own crate (CARGO_INCREMENTAL={incremental}): {:?}",
            entries_named(destination_build.as_path(), own_prefix.as_str())
        );
        assert!(!entries_named(destination_build.as_path(), DEPENDENCY).is_empty(), "the copy keeps the dependency builds it exists for (CARGO_INCREMENTAL={incremental})");
        assert!(!staging_of(destination_build.as_path()).exists(), "the staging directory is published and gone (CARGO_INCREMENTAL={incremental})");
        remove(root.as_path());
    }
}
