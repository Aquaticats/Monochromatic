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

/// The member the fixture's workspace builds,
/// which the filling must clean out of the copy.
const MEMBER: &str = "own";

/// The registry dependency the fixture builds,
/// which the copy exists to carry and must keep.
const DEPENDENCY: &str = "serde";

/// The dependency the lock fixture resolves from a directory of its own,
/// so Cargo writes that fixture's lock without reading a registry.
const DIRECTORY_DEPENDENCY: &str = "local-dependency";

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

/// The lock file beside a workspace's root manifest.
const LOCK_FILE: &str = "Cargo.lock";

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

/// What the operating system records about one file,
/// so a control can tell a file nothing wrote from one that was written again,
/// even when the bytes came back the same.
/// Why:
///   Reading the bytes cannot show that a file was not written:
/// a rewrite and a put-back leave the same ones.
///   The inode,
/// the size and the two timestamps do change,
/// and no writer restores them.
#[derive(Debug, Eq, PartialEq)]
struct FileIdentity {
    inode: u64,
    size: u64,
    modified: (i64, i64),
    changed: (i64, i64),
}

/// What:
///  What the filesystem records about the file at `path`.
/// Why:
///   The four fields a write changes are read together,
/// so a control compares one value.
fn identity_of(path: &Path) -> FileIdentity {
    /// The Unix metadata fields that tell a rewrite from an untouched file.
    use std::os::unix::fs::MetadataExt as _;
    let metadata: std::fs::Metadata = std::fs::metadata(path).expect("the file's metadata");
    return FileIdentity { inode: metadata.ino(), size: metadata.len(), modified: (metadata.mtime(), metadata.mtime_nsec()), changed: (metadata.ctime(), metadata.ctime_nsec()) };
}

/// What:
///  A fixture workspace with a lock Cargo itself resolved,
///  and a source build directory for the copy to carry,
///  without compiling anything.
/// Why:
///   The control below is about the lock file and about what the clean refuses,
/// and a clean reads the manifest and the lock and writes only to directories of this job's own,
/// so compiling the fixture would cost the gate a minute and prove nothing more.
/// `cargo generate-lockfile` is what makes the lock one Cargo accepts as matching the manifest.
/// The one dependency is a directory inside the fixture,
/// so resolving the lock reads no registry and never waits on the package cache every other
/// build on this host holds.
fn a_workspace_with_a_resolved_lock(root: &Path) -> PrewarmEntry {
    let named: PrewarmEntry = entry_of(root);
    let entry: PrewarmEntry = PrewarmEntry { members: vec![String::from(MEMBER)], ..named };
    std::fs::create_dir_all(entry.root.join("src")).expect("the workspace");
    std::fs::create_dir_all(entry.root.join("dep/src")).expect("the dependency beside the workspace");
    let manifest: String = format!(
        "[workspace]\nmembers = [\".\"]\n\n[package]\nname = \"{MEMBER}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n{DIRECTORY_DEPENDENCY} = {{ path = \"dep\" }}\n"
    );
    std::fs::write(entry.root.join("Cargo.toml"), manifest).expect("the manifest");
    std::fs::write(entry.root.join("src/lib.rs"), format!("pub fn {MEMBER}() -> &'static str {{ \"{MEMBER}\" }}\n")).expect("the crate");
    std::fs::write(entry.root.join("dep/Cargo.toml"), format!("[package]\nname = \"{DIRECTORY_DEPENDENCY}\"\nversion = \"1.0.0\"\nedition = \"2024\"\n"))
        .expect("the dependency's manifest");
    std::fs::write(entry.root.join("dep/src/lib.rs"), "pub fn dependency() {}\n").expect("the dependency's crate");
    std::fs::create_dir_all(entry.from.join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    let resolved: std::process::Output = std::process::Command::new(cargo_on_the_search_path())
        .args(["generate-lockfile", "--offline", "--quiet"])
        .current_dir(entry.root.as_path())
        .output()
        .expect("cargo resolves the fixture's lock");
    assert!(resolved.status.success(), "the fixture's lock resolves: {}", String::from_utf8_lossy(resolved.stderr.as_slice()));
    return entry;
}

/// What:
///  The filling never writes the worktree's `Cargo.lock`,
/// in any of the three states a checked-out lock can be in,
/// so a kill anywhere in the filling leaves the person's bytes as they were.
/// Why:
///   The clean resolves the worktree's workspace even when it deletes only from directories of
/// this job's own,
/// and a lock that no longer matches its manifest is rewritten where it lies,
/// and a missing one is created.
/// The filling runs after the add has returned,
/// while the person works in the worktree,
/// so a lock it writes is a tracked file changed behind their back,
/// and a put-back compares only the bytes it read with the bytes it finds:
/// a person's own write during the clean is overwritten,
/// and a kill before the put-back leaves the rewritten lock.
/// A clean that is told to leave the lock alone cannot write it at all:
/// Cargo refuses the lock it would have to change,
/// and the filling reports that instead of filling the copy.
#[test]
fn the_filling_never_writes_the_worktrees_lock() {
    // The three states a worktree's lock can be in when the filling starts.
    const LOCK_STATES: [&str; 3] = ["matching", "stale", "absent"];
    for state in LOCK_STATES {
        let root: PathBuf = fixture(format!("prewarm-lock-never-written-{state}").as_str());
        let entry: PrewarmEntry = a_workspace_with_a_resolved_lock(root.as_path());
        let lock: PathBuf = entry.root.join(LOCK_FILE);
        let left: Option<Vec<u8>> = if state == "absent" {
            std::fs::remove_file(lock.as_path()).expect("a worktree with no lock");
            None
        } else {
            let built: String = std::fs::read_to_string(lock.as_path()).expect("the lock the build wrote");
            if state == "stale" {
                // A manifest that moved on and a lock that did not: the shape several locks in this repository have.
                let stale: String = built.replace("name = \"own\"\nversion = \"0.1.0\"", "name = \"own\"\nversion = \"0.0.9\"");
                std::fs::write(lock.as_path(), stale.as_bytes()).expect("the stale lock");
                Some(stale.into_bytes())
            } else {
                Some(built.into_bytes())
            }
        };
        let before: Option<FileIdentity> = left.as_ref().map(|_unused| return identity_of(lock.as_path()));
        let item: WorkItem = item_for(root.as_path(), entry.root.as_path(), &entry);
        let verdict: Verdict = crate::lazy_runtime::run_on_runtime(super::prewarm_awaiting(&item)).expect("the filling runs");
        let after: Option<Vec<u8>> = std::fs::read(lock.as_path()).ok();
        let described: String = after.as_ref().map_or(String::from("no lock"), |bytes| return format!("{} bytes", bytes.len()));
        assert_eq!(after, left, "the worktree's {LOCK_FILE} holds exactly the bytes the filling found ({state}): {described}");
        if let Some(before) = before {
            assert_eq!(identity_of(lock.as_path()), before, "nothing wrote the worktree's {LOCK_FILE}, so no kill can leave it rewritten or cut short ({state})");
        }
        if state == "matching" {
            assert_eq!(verdict, Verdict::Done, "a lock that matches its manifest lets the clean run, and the copy is published");
            assert!(entry.to.exists(), "the copy reached the worktree's build directory name (matching)");
        } else {
            let Verdict::Failed(reason) = verdict else { panic!("the filling reports the lock Cargo would have had to change ({state}): {verdict:?}") };
            assert!(reason.contains(LOCK_FILE), "the report names the lock Cargo would have had to change ({state}): {reason}");
            assert!(!entry.to.exists(), "a refused clean publishes no copy ({state})");
        }
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
///  A directory at the staging name that this filling did not make,
/// and a file at the marker name that is not this job's,
/// are never removed,
/// and the worktree is filled anyway.
/// Why:
///   The staging and marker names are this job's by convention,
///   not by proof:
/// only the text this job writes at the marker name says a directory there is its own leftover.
/// Anything else at those names is somebody's work,
/// which "never discard user data" forbids deleting,
/// so the filling stages under a name of its own and leaves what it found alone.
#[test]
fn a_directory_at_the_staging_name_that_this_job_did_not_make_is_never_removed() {
    // What a stranger leaves at each of the two names.
    const STRANGERS: [&str; 2] = ["staging", "marker"];
    for stranger in STRANGERS {
        let root: PathBuf = fixture(format!("prewarm-stranger-{stranger}").as_str());
        let entry: PrewarmEntry = entry_of(root.as_path());
        std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
        std::fs::create_dir_all(entry.from.join("debug/deps")).expect("the source's dependency build");
        std::fs::write(entry.from.join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
        let staged: PathBuf = staging_of(entry.to.as_path());
        std::fs::create_dir_all(staged.join("keep-me")).expect("a directory this filling did not make");
        std::fs::write(staged.join("keep-me/a-file"), b"someone's work").expect("the file");
        let marker: PathBuf = super::marker_of(staged.as_path());
        if stranger == "marker" {
            // A file at the marker name that does not hold this job's text claims nothing.
            std::fs::write(marker.as_path(), b"a person's own note").expect("a file this filling did not make");
        }
        let fill: Fill = Fill { cargo: cargo_that_succeeds(root.as_path()), limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: std::time::Duration::from_secs(5) } };
        let filled: Filled = super::fill_one_in_a_control(&fill, &entry).expect("the worktree is filled");
        assert_eq!(filled, Filled::Published, "the copy is published under the worktree's build directory name ({stranger})");
        assert_eq!(std::fs::read(staged.join("keep-me/a-file")).expect("the file this filling did not make is still there"), b"someone's work");
        assert_eq!(std::fs::read(marker.as_path()).unwrap_or_default(), if stranger == "marker" { b"a person's own note".to_vec() } else { Vec::new() }, "a file this filling did not make is never removed ({stranger})");
        remove(root.as_path());
    }
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
    let entry: PrewarmEntry = PrewarmEntry { root: nested.clone(), from: root.join("source-build"), to: destination_build, members: vec![String::from(MEMBER)] };
    let item: WorkItem = item_for(root.as_path(), worktree.as_path(), &entry);
    let verdict: Verdict = crate::lazy_runtime::run_on_runtime(super::prewarm_awaiting_with(&item, &an_environment_with_no_programs())).expect("the filling runs");
    assert_eq!(verdict, Verdict::Done, "the worktree's own Cargo link is found from the worktree root");
    remove(root.as_path());
}

/// The deadline one control gives a filling whose whole work is a fixture's clone and a stand-in
/// Cargo,
/// so a filling that waits on something it must not wait on fails here instead of holding the suite.
const FILLING_DEADLINE: std::time::Duration = std::time::Duration::from_secs(10);

/// What:
///  Make a named pipe at `path`.
/// Why:
///   A named pipe is the one foreign object whose plain open waits for a writer that may never come,
///   which is what the filling must never do at a name anyone can write.
fn named_pipe_at(path: &Path) {
    /// The bytes of a pathname, for the C interface.
    use std::os::unix::ffi::OsStrExt as _;
    let named: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("a fixture path holds no NUL byte");
    // SAFETY: the string is NUL-terminated and outlives the call, and the name is a fresh path in the fixture.
    let made: libc::c_int = unsafe { libc::mkfifo(named.as_ptr(), 0o600) };
    assert_eq!(made, 0, "make a named pipe at {}: {}", path.display(), std::io::Error::last_os_error());
}

/// What:
///  An entry whose source build directory holds one dependency build,
///  so a clone has something to copy,
///  and whose workspace exists,
///  so the clean has a directory to run in.
/// Why:
///   The three controls below are about a name beside the destination and about the worktree's lock,
/// and none of them reads a manifest.
fn an_entry_with_a_source_build(root: &Path) -> PrewarmEntry {
    let entry: PrewarmEntry = entry_of(root);
    std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
    std::fs::create_dir_all(entry.from.join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    return entry;
}

/// What:
///  A named pipe at the marker name,
///  which this filling did not make,
///  never holds the filling:
///  the copy is published under names of this run's own and the pipe stays where it is.
/// Why:
///   The marker name is this job's by convention,
/// and anyone can put anything there.
/// A reader-less pipe blocks a plain read inside `open` for ever,
/// so one foreign object at that name would hold the service's filling with no end
/// (the owner's "nothing should ever wait forever or indefinitely"),
/// and the worktree would keep waiting for a build directory nothing fills.
#[test]
fn a_named_pipe_at_the_marker_name_never_holds_the_filling() {
    let root: PathBuf = fixture("prewarm-marker-pipe");
    let entry: PrewarmEntry = an_entry_with_a_source_build(root.as_path());
    let staged: PathBuf = staging_of(entry.to.as_path());
    let marker: PathBuf = super::marker_of(staged.as_path());
    named_pipe_at(marker.as_path());
    let covered: PathBuf = root.clone();
    let filled: Option<Result<Filled, String>> = crate::test_support::finished_within_at(FILLING_DEADLINE, move || {
        let fill: Fill = Fill { cargo: cargo_that_succeeds(root.as_path()), limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: FILLING_DEADLINE } };
        return super::fill_one_in_a_control(&fill, &entry);
    });
    let filled: Result<Filled, String> = filled.expect("the filling must not wait on a named pipe at the marker name");
    assert_eq!(filled, Ok(Filled::Published), "the copy is published under names of this run's own");
    assert!(marker.exists(), "the named pipe this filling did not make is left where it is");
    remove(covered.as_path());
}

/// What:
///  A person's own update of the worktree's `Cargo.lock`,
///  written while the filling's clean runs,
///  is still the worktree's lock when the filling ends.
/// Why:
///   The filling runs after the add has returned,
/// while the person works in the worktree,
/// so the clean can overlap their `cargo update`.
/// Comparing the lock's bytes with the bytes read before the clean cannot tell Cargo's write from
/// theirs,
/// and writing the earlier bytes back would discard their newer ones
/// ("never discard user data").
/// A filling that never writes the lock cannot lose it.
#[test]
fn the_filling_never_writes_a_lock_a_person_updates_while_the_clean_runs() {
    /// What the person's own command writes while the clean runs.
    const THE_PERSONS_UPDATE: &[u8] = b"the person's own update\n";
    let root: PathBuf = fixture("prewarm-lock-updated-while-clean-runs");
    let entry: PrewarmEntry = an_entry_with_a_source_build(root.as_path());
    let lock: PathBuf = entry.root.join(LOCK_FILE);
    std::fs::write(lock.as_path(), b"checked out\n").expect("the checked-out lock");
    // A Cargo that says when it started and then waits to be let go,
    // so the person's write lands inside the clean rather than before or after it.
    let started: PathBuf = root.join("cargo-started");
    let release: PathBuf = root.join("release-cargo");
    named_pipe_at(release.as_path());
    let cargo: PathBuf = root.join("blocking-cargo");
    crate::test_support::executable(cargo.as_path(), format!("#!/bin/sh\nprintf started > '{}'\nread released < '{}'\nexit 0\n", started.display(), release.display()).as_bytes());
    let persons_lock: PathBuf = lock.clone();
    let persons_started: PathBuf = started.clone();
    let persons_release: PathBuf = release.clone();
    let person = std::thread::spawn(move || {
        let deadline: std::time::Instant = std::time::Instant::now() + FILLING_DEADLINE;
        while !persons_started.exists() {
            assert!(std::time::Instant::now() < deadline, "the clean did not start within {FILLING_DEADLINE:?}");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        std::fs::write(persons_lock.as_path(), THE_PERSONS_UPDATE).expect("the person's own update while the clean runs");
        /// Writing a line to the pipe is what lets the stand-in Cargo exit.
        use std::io::Write as _;
        let mut letting_go: std::fs::File = std::fs::OpenOptions::new().write(true).open(persons_release.as_path()).expect("let the clean finish");
        letting_go.write_all(b"go\n").expect("let the clean finish");
    });
    let fill: Fill = Fill { cargo, limits: ToolLimits { window: FILLING_DEADLINE, ceiling: FILLING_DEADLINE } };
    let filled: Result<Filled, String> = super::fill_one_in_a_control(&fill, &entry);
    person.join().expect("the person's own command finished");
    assert_eq!(filled, Ok(Filled::Published), "the copy is cleaned and published");
    assert_eq!(std::fs::read(lock.as_path()).expect("read the worktree's lock"), THE_PERSONS_UPDATE, "the person's newer lock is still the worktree's lock");
    remove(root.as_path());
}

/// What:
///  A file that another command writes at the marker name,
///  after this filling has looked at the name and before it publishes its own marker there,
///  is never removed,
///  and the worktree is filled under names of this run's own.
/// Why:
///   The filling looks at the three usual names once and then claims the marker by an exclusive
/// write,
/// and another command can create that name in between.
/// An exclusive write that answers "it is there already" proves only that somebody's file is there,
/// never that this run's is,
/// so removing it would delete a file this job never wrote
/// ("never discard user data").
#[test]
fn a_file_that_wins_the_race_to_the_marker_name_is_never_removed() {
    /// What the other command writes at the marker name.
    const ANOTHER_COMMANDS_FILE: &[u8] = b"a person's own note";
    let root: PathBuf = fixture("prewarm-marker-race");
    let entry: PrewarmEntry = an_entry_with_a_source_build(root.as_path());
    let staged: PathBuf = staging_of(entry.to.as_path());
    let marker: PathBuf = super::marker_of(staged.as_path());
    let planted: PathBuf = marker.clone();
    crate::private_storage::AFTER_THE_STAGING_FILE_IS_WRITTEN.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            std::fs::write(planted.as_path(), ANOTHER_COMMANDS_FILE)?;
            return Ok(());
        }));
    });
    let covered: PathBuf = root.clone();
    let fill: Fill = Fill { cargo: cargo_that_succeeds(root.as_path()), limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: FILLING_DEADLINE } };
    let filled: Result<Filled, String> = super::fill_one_in_a_control(&fill, &entry);
    crate::private_storage::AFTER_THE_STAGING_FILE_IS_WRITTEN.with(|hook| { *hook.borrow_mut() = None; });
    assert_eq!(filled, Ok(Filled::Published), "the worktree is filled under names of this run's own");
    assert_eq!(std::fs::read(marker.as_path()).expect("the other command's file"), ANOTHER_COMMANDS_FILE, "a file this filling did not write is never removed");
    remove(covered.as_path());
}
