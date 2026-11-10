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
///   A killed run leaves the marker that claims the staging name and the target beside it,
/// so what the next filling finds there is its own,
/// and a failure must leave no half-filled directory for a later run to trip over.
#[test]
fn a_failed_clean_leaves_no_staging_directory_and_no_destination() {
    let root: PathBuf = fixture("prewarm-failed-clean");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
    std::fs::create_dir_all(entry.from.join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    let staged: PathBuf = staging_of(entry.to.as_path());
    std::fs::create_dir_all(staged.join("left-by-a-killed-run")).expect("a staging directory an earlier run left");
    super::write_marker(super::marker_of(staged.as_path()).as_path()).expect("the marker an earlier run left");
    let failed: Result<Filled, String> = super::fill_one_in_a_control(&fill_of_nowhere(), &entry);
    assert!(failed.is_err(), "a clean that could not run is reported: {failed:?}");
    assert!(!staged.exists(), "the staging directory this job's marker claims is removed");
    assert!(!super::marker_of(staged.as_path()).exists(), "and the marker goes with it");
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

/// What:
///  A record naming one workspace is filled:
/// the copy is published under the worktree's build directory name,
/// the worktree's own crate is cleaned out of the copy,
/// and the source checkout's build directory is untouched.
/// Why:
///   This is the whole guarantee of the change,
/// end to end and with a real Cargo:
/// the add stopped doing this work,
/// so the service's filling must do the same clean the add used to,
/// and it must never clean the source's own state,
/// which every other worktree of the repository shares.
#[test]
fn the_filling_publishes_a_clean_copy_and_leaves_the_source_alone() {
    let root: PathBuf = fixture("prewarm-filling");
    let workspace: PathBuf = root.join("worktree/package/own-ws");
    std::fs::create_dir_all(workspace.join("src")).expect("the workspace");
    std::fs::write(workspace.join("Cargo.toml"), "[workspace]\nmembers = [\".\"]\n\n[package]\nname = \"own\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nserde = \"1\"\n").expect("the manifest");
    std::fs::write(workspace.join("src/lib.rs"), "pub fn own() -> &'static str { \"own\" }\n").expect("the crate");
    let source_build: PathBuf = root.join("source-build");
    let cargo: PathBuf = cargo_on_the_search_path();
    let built: std::process::Output = std::process::Command::new(cargo)
        .args(["build", "--offline", "--quiet"])
        .current_dir(workspace.as_path())
        .env(BUILD_DIRECTORY_VARIABLE, source_build.as_os_str())
        .env(TARGET_DIRECTORY_VARIABLE, root.join("source-target").as_os_str())
        .output()
        .expect("cargo runs");
    assert!(built.status.success(), "the fixture builds: {}", String::from_utf8_lossy(built.stderr.as_slice()));
    let destination_build: PathBuf = root.join("worktree-build");
    let entry: PrewarmEntry = PrewarmEntry { root: workspace.clone(), from: source_build.clone(), to: destination_build.clone(), members: vec![String::from("own")] };
    let item: WorkItem = crate::package_repair_prewarm::item_of(root.as_path(), workspace.as_path(), std::slice::from_ref(&entry)).expect("the record");
    let verdict: Verdict = crate::lazy_runtime::run_on_runtime(super::prewarm_awaiting(&item)).expect("the filling runs");
    assert_eq!(verdict, Verdict::Done, "the filling published the copy");
    let own_of = |build: &Path| -> Vec<String> {
        return std::fs::read_dir(build.join("debug/incremental"))
            .map(|listing| return listing.flatten().filter_map(|each| return each.file_name().into_string().ok()).filter(|name| return name.starts_with("own-")).collect())
            .unwrap_or_default();
    };
    assert!(!own_of(source_build.as_path()).is_empty(), "the source checkout's own crate is still built there");
    assert!(own_of(destination_build.as_path()).is_empty(), "the copy holds no artifact of the worktree's own crate");
    assert!(!staging_of(destination_build.as_path()).exists(), "the staging directory is published and gone");
    remove(root.as_path());
}

/// The lock file beside a workspace's root manifest.
const LOCK_FILE: &str = "Cargo.lock";

/// The member the fixture workspace builds.
const FIXTURE_MEMBER: &str = "own";

/// The manifest of a workspace whose only member depends on one registry crate,
/// so a build leaves dependency state a copy is worth making from.
const FIXTURE_MANIFEST: &str = "[workspace]\nmembers = [\".\"]\n\n[package]\nname = \"own\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nserde = \"1\"\n";

/// What:
///  Write the fixture workspace `entry_of` names,
/// and build it into the source build directory,
/// so the entry has state worth copying.
/// Why:
///  The two controls below need a workspace a real Cargo resolves,
/// and the copy exists only when the source holds dependency builds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function builtWorkspace(root: Path): PrewarmEntry;
/// ```
fn built_workspace(root: &Path) -> PrewarmEntry {
    let named: PrewarmEntry = entry_of(root);
    let entry: PrewarmEntry = PrewarmEntry { members: vec![String::from(FIXTURE_MEMBER)], ..named };
    std::fs::create_dir_all(entry.root.join("src")).expect("the workspace");
    std::fs::write(entry.root.join("Cargo.toml"), FIXTURE_MANIFEST).expect("the manifest");
    std::fs::write(entry.root.join("src/lib.rs"), format!("pub fn {FIXTURE_MEMBER}() -> &'static str {{ \"{FIXTURE_MEMBER}\" }}\n")).expect("the crate");
    let cargo: PathBuf = cargo_on_the_search_path();
    let built: std::process::Output = std::process::Command::new(cargo)
        .args(["build", "--offline", "--quiet"])
        .current_dir(entry.root.as_path())
        .env(BUILD_DIRECTORY_VARIABLE, entry.from.as_os_str())
        .env(TARGET_DIRECTORY_VARIABLE, root.join("source-target").as_os_str())
        .output()
        .expect("cargo runs");
    assert!(built.status.success(), "the fixture builds: {}", String::from_utf8_lossy(built.stderr.as_slice()));
    return entry;
}

/// What:
///  The record of one worktree,
/// as the add writes it:
/// the repository's common directory,
/// the new worktree,
/// and the workspace it names.
/// Why:
///   The record's repository is the worktree root,
/// which the service resolves the toolchain's Cargo from,
/// and the entry says which workspace below it is filled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const itemFor = (common, worktree, entry) => itemOf(common, worktree, [entry]);
/// ```
fn item_for(common: &Path, worktree: &Path, entry: &PrewarmEntry) -> WorkItem {
    return crate::package_repair_prewarm::item_of(common, worktree, std::slice::from_ref(entry)).expect("the record");
}

/// What:
///  The worktree's `Cargo.lock` is as checked out after the service's clean,
/// whether it was there and stale or absent.
/// Why:
///   `cargo clean` resolves the worktree's workspace even when it deletes only from directories of
/// this job's own,
/// which rewrites a lock that no longer matches its manifest and creates one that is missing.
/// The add's own clean puts the lock back (`package_repair_cargo_lock.rs`),
/// and the service's clean runs after the add has returned,
/// so a lock it left rewritten would change a tracked file in the person's worktree behind their back.
#[test]
fn the_filling_puts_the_lock_back_as_checked_out() {
    // The two states a checked-out lock can be in.
    const LOCK_STATES: [&str; 2] = ["stale", "absent"];
    for state in LOCK_STATES {
        let root: PathBuf = fixture(format!("prewarm-lock-{state}").as_str());
        let entry: PrewarmEntry = built_workspace(root.as_path());
        let lock: PathBuf = entry.root.join(LOCK_FILE);
        let checked_out: Option<Vec<u8>> = if state == "stale" {
            // A manifest that moved on and a lock that did not: the shape several locks in this repository have.
            let built: String = std::fs::read_to_string(lock.as_path()).expect("the lock the build wrote");
            let stale: String = built.replace("name = \"own\"\nversion = \"0.1.0\"", "name = \"own\"\nversion = \"0.0.9\"");
            std::fs::write(lock.as_path(), stale.as_bytes()).expect("the stale lock");
            Some(stale.into_bytes())
        } else {
            std::fs::remove_file(lock.as_path()).expect("a worktree with no lock");
            None
        };
        let item: WorkItem = item_for(root.as_path(), entry.root.as_path(), &entry);
        let verdict: Verdict = crate::lazy_runtime::run_on_runtime(super::prewarm_awaiting(&item)).expect("the filling runs");
        assert_eq!(verdict, Verdict::Done, "the filling published the copy ({state})");
        assert!(entry.to.exists(), "the copy reached the worktree's build directory name ({state})");
        let after: Option<Vec<u8>> = std::fs::read(lock.as_path()).ok();
        let described: String = after.as_ref().map_or(String::from("no lock"), |bytes| return format!("{} bytes", bytes.len()));
        assert_eq!(after.as_deref(), checked_out.as_deref(), "the worktree's {LOCK_FILE} is as checked out after the clean ({state}): {described}");
        remove(root.as_path());
    }
}

/// A Cargo that succeeds without deleting anything,
/// for a control that is not about the clean.
fn cargo_that_succeeds(root: &Path) -> PathBuf {
    let program: PathBuf = root.join("cargo-that-succeeds");
    crate::test_support::executable(program.as_path(), b"#!/bin/sh\nexit 0\n");
    return program;
}

/// What:
///  A directory at the staging name that this filling did not make is never removed,
/// and the worktree is filled anyway.
/// Why:
///   The staging name is this job's by convention,
///   not by proof:
/// only a marker this job writes says a directory there is its own leftover.
/// A directory without one is somebody's work,
/// which "never discard user data" forbids deleting,
/// so the filling stages under a name of its own and leaves that directory alone.
#[test]
fn a_directory_at_the_staging_name_that_this_job_did_not_make_is_never_removed() {
    let root: PathBuf = fixture("prewarm-stranger-staging");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
    std::fs::create_dir_all(entry.from.join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    let staged: PathBuf = staging_of(entry.to.as_path());
    std::fs::create_dir_all(staged.join("keep-me")).expect("a directory this filling did not make");
    std::fs::write(staged.join("keep-me/a-file"), b"someone's work").expect("the file");
    let fill: Fill = Fill { cargo: cargo_that_succeeds(root.as_path()), limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: std::time::Duration::from_secs(5) } };
    let filled: Filled = super::fill_one_in_a_control(&fill, &entry).expect("the worktree is filled");
    assert_eq!(filled, Filled::Published, "the copy is published under the worktree's build directory name");
    assert_eq!(std::fs::read(staged.join("keep-me/a-file")).expect("the file this filling did not make is still there"), b"someone's work");
    remove(root.as_path());
}

/// What:
///  A successful filling leaves no directory of its own beside the build directory it fills.
/// Why:
///   The clean is given a target directory of this job's own,
/// so it deletes nothing from the worktree's target directory,
/// and that directory is this job's to remove when the filling ends.
/// A published filling that kept it would leave one behind for every build directory it ever fills.
#[test]
fn a_published_filling_leaves_no_target_directory_of_its_own() {
    let root: PathBuf = fixture("prewarm-target-given-back");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
    std::fs::create_dir_all(entry.from.join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    // A clean that makes the target directory it is given, as a real Cargo does.
    let program: PathBuf = root.join("cargo-that-cleans");
    crate::test_support::executable(
        program.as_path(),
        b"#!/bin/sh\nmkdir --parents \"$CARGO_TARGET_DIR/debug\"\necho cleaned > \"$CARGO_TARGET_DIR/debug/state\"\nexit 0\n",
    );
    let fill: Fill = Fill { cargo: program, limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: std::time::Duration::from_secs(5) } };
    let filled: Filled = super::fill_one_in_a_control(&fill, &entry).expect("the copy is cleaned and published");
    assert_eq!(filled, Filled::Published, "the copy is published under the worktree's build directory name");
    let staged: PathBuf = staging_of(entry.to.as_path());
    assert!(!super::target_of(staged.as_path()).exists(), "the clean's target directory {} is left behind", super::target_of(staged.as_path()).display());
    assert!(entry.to.exists(), "the published copy is the worktree's build directory");
    remove(root.as_path());
}

/// An environment with nothing on its search path,
/// as the service's own may have,
/// and short bounds so a control never waits.
fn an_environment_with_no_programs() -> RepairEnvironment {
    return RepairEnvironment { search_path: None, quiet_window: std::time::Duration::from_millis(200), ceiling: std::time::Duration::from_secs(5), workers: 1 };
}

/// What:
///  A worktree's own link to the toolchain's Cargo is found from the worktree root,
/// whatever workspace the record's first entry names.
/// Why:
///   The link sits at the worktree's root,
///   which the toolchain repair names `OWN_LINK` (`repair/package_repair_toolchain.rs`),
/// and a record's workspaces can be nested anywhere below it,
/// so a lookup at the first workspace's root finds no link and fails the whole filling.
#[test]
fn the_worktrees_own_cargo_link_is_found_from_the_worktree_root_for_a_nested_workspace() {
    let root: PathBuf = fixture("prewarm-own-cargo-link");
    let worktree: PathBuf = root.join("worktree");
    let nested: PathBuf = worktree.join("package/example");
    std::fs::create_dir_all(nested.as_path()).expect("a workspace below the worktree root");
    let own: PathBuf = worktree.join(".cache/toolchain-bin/cargo");
    std::fs::create_dir_all(own.parent().expect("the link has a directory")).expect("the toolchain directory");
    crate::test_support::executable(own.as_path(), b"#!/bin/sh\nexit 0\n");
    let destination_build: PathBuf = root.join("worktree-build");
    // A build got there first, so nothing is cloned or cleaned and only the lookup is under test.
    std::fs::create_dir_all(destination_build.as_path()).expect("a build directory a build made");
    let entry: PrewarmEntry = PrewarmEntry { root: nested.clone(), from: root.join("source-build"), to: destination_build, members: vec![String::from(FIXTURE_MEMBER)] };
    let item: WorkItem = item_for(root.as_path(), worktree.as_path(), &entry);
    let verdict: Verdict = crate::lazy_runtime::run_on_runtime(super::prewarm_awaiting_with(&item, &an_environment_with_no_programs())).expect("the filling runs");
    assert_eq!(verdict, Verdict::Done, "the worktree's own Cargo link is found from the worktree root");
    remove(root.as_path());
}
