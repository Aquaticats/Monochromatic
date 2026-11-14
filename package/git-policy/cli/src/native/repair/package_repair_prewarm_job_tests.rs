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
    return Fill { cargo: PathBuf::from(NO_CARGO), limits: ToolLimits { window: std::time::Duration::from_millis(200), ceiling: std::time::Duration::from_secs(5) }, kept_in: None, worktree: PathBuf::from("/nonexistent/worktree"), filling: String::from("a-controls-filling") };
}

/// One workspace whose source build directory holds one dependency build,
/// so a clone has something to copy.
fn entry_of(root: &Path) -> PrewarmEntry {
    return PrewarmEntry {
        root: root.join("package/linter/rust"),
        from: Some(root.join("source-build")),
        to: root.join("worktree-build"),
        members: vec![String::from("monochromatic-lint")],
        staged_in: None,
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
    std::fs::create_dir_all(entry.from.as_ref().expect("the source's build directory").join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.as_ref().expect("the source's build directory").join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    let staged: PathBuf = staging_of(entry.to.as_path());
    std::fs::create_dir_all(staged.join("left-by-a-killed-run")).expect("a staging directory an earlier run left");
    super::write_marker(super::marker_of(staged.as_path()).as_path(), entry.to.as_path()).expect("the marker an earlier run left");
    let failed: Result<Filled, String> = super::fill_one_in_a_control(&fill_of_nowhere(), &entry);
    assert!(failed.is_err(), "a clean that could not run is reported: {failed:?}");
    assert!(!staged.exists(), "the staging directory this job's marker claims is removed");
    assert!(!super::marker_of(staged.as_path()).exists(), "and the marker goes with it");
    assert!(!entry.to.exists(), "and nothing is published under the worktree's name");
    remove(root.as_path());
}

/// What:
///  A filling killed after it published leaves nothing of its own:
/// the marker and the empty staging copy beside a build directory a build now holds are given back by the next filling,
/// and the directory the build made is kept byte for byte.
/// Why:
///   A publish is one rename that leaves no trace of the filling that made it,
/// so a kill between the publish and the giving back of the names stranded a directory and its marker for ever:
/// every later filling answered `Filled::Kept` at the destination and never looked at the names again.
/// A kill at any instruction must leave a worktree the next command completes by itself,
/// and nothing of the wrapper's left behind beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const filled = fillOne(fill, entry); expect(filled).toBe('kept'); expect(existsSync(staging)).toBe(false)
/// ```
#[test]
fn a_filling_killed_after_it_published_leaves_nothing_of_its_own() {
    let root: PathBuf = fixture("prewarm-killed-after-publish");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.to.as_path()).expect("the build directory the killed filling published");
    std::fs::write(entry.to.join("kept"), b"a build's own artifact").expect("the file");
    let staged: PathBuf = staging_of(entry.to.as_path());
    std::fs::create_dir_all(staged.as_path()).expect("the staging directory the publish left empty");
    super::write_marker(super::marker_of(staged.as_path()).as_path(), entry.to.as_path()).expect("the marker the killed filling wrote first");
    let filled: Filled = super::fill_one_in_a_control(&fill_of_nowhere(), &entry).expect("a filled destination is kept");
    assert_eq!(filled, Filled::Kept, "the worktree keeps what the build made");
    assert_eq!(std::fs::read(entry.to.join("kept")).expect("the file"), b"a build's own artifact");
    assert!(!staged.exists(), "the staging directory the killed filling left is given back: {}", staged.display());
    assert!(!super::marker_of(staged.as_path()).exists(), "and the marker that claimed it goes with it");
    remove(root.as_path());
}

/// What:
///  The state a worktree's own copy carried is moved aside by the add,
/// and the filling cleans it there and publishes it back,
/// so no build trusts the copy's artifacts and no `cargo clean` ever runs in the add.
/// Why:
///   Cargo treats the copied artifacts of the worktree's own crates as fresh, so a build that started before a clean would
///  reuse them at the wrong paths;
/// the add answers within its bound only if the state is out of the way before it returns.
/// A rename does that in one step, and the clean of that state is the filling's work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stageTheWorktreesOwnState(destination); fillOne(fill, { ...entry, staged: true, from: undefined }); expect(published).toBe(true)
/// ```
#[test]
fn a_staged_state_the_add_moved_aside_is_cleaned_and_published() {
    let root: PathBuf = fixture("prewarm-staged-by-the-add");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
    // The copy carried the worktree's own build state, which the add moves aside in one rename.
    std::fs::create_dir_all(entry.to.join("debug")).expect("the copied build state");
    std::fs::write(entry.to.join("debug/copied"), b"the copy's state\n").expect("a file of the copy");
    let git_directory: PathBuf = root.join("git-directory");
    let staged: Staged = super::stage_the_worktrees_own_state(entry.to.as_path(), Some(git_directory.as_path())).expect("the state is moved aside");
    let Staged::Staged(staged_in) = staged else {
        panic!("the state is moved aside");
    };
    assert!(!entry.to.exists(), "the worktree's build directory is out of the way before the add answers");
    assert_eq!(staged_in, staging_of(entry.to.as_path()), "the state waits beside the build directory it was taken from");
    assert!(!entry.to.exists(), "and out of the build directory itself");
    let working: PrewarmEntry = PrewarmEntry { staged_in: Some(staged_in.clone()), from: None, ..entry.clone() };
    let cargo: PathBuf = root.join("cargo");
    crate::test_support::executable(cargo.as_path(), b"#!/bin/sh\nexit 0\n");
    let fill: Fill = Fill { cargo, limits: ToolLimits { window: std::time::Duration::from_millis(500), ceiling: std::time::Duration::from_secs(5) }, kept_in: None, worktree: root.clone(), filling: String::from("a-controls-filling") };
    let filled: Filled = super::fill_one_in_a_control(&fill, &working).expect("the staged state is cleaned and published");
    assert_eq!(filled, Filled::Published);
    assert!(entry.to.join("debug/copied").is_file(), "the cleaned copy is published under the worktree's build directory name");
    assert!(!staged_in.exists(), "and the staging name is given back");
    assert!(!super::marker_of(staged_in.as_path()).exists(), "with the marker that claimed it");
    remove(root.as_path());
}

/// What:
///  A build that made the worktree's build directory while the filling ran keeps everything it made,
/// and the state the add moved aside is given back.
/// Why:
///   The filling only ever adds state a build would otherwise compile, and it must never replace what a build made.
/// A staged entry is the one case where the state it would publish is slower to make than the build itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stageTheWorktreesOwnState(destination); mkdirSync(destination); expect(fillOne(fill, staged)).toBe('kept'); expect(staging).not.toExist()
/// ```
#[test]
fn a_build_that_got_there_first_keeps_its_own_and_the_staged_state_is_given_back() {
    let root: PathBuf = fixture("prewarm-staged-taken");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
    std::fs::create_dir_all(entry.to.join("debug")).expect("the copied build state");
    let git_directory: PathBuf = root.join("git-directory");
    let Staged::Staged(staged_in) = super::stage_the_worktrees_own_state(entry.to.as_path(), Some(git_directory.as_path())).expect("the state is moved aside") else {
        panic!("the state is moved aside");
    };
    let staging: PathBuf = staged_in.clone();
    // A build got there first, in the second between the add's rename and this filling.
    std::fs::create_dir_all(entry.to.join("debug")).expect("the build's own directory");
    std::fs::write(entry.to.join("debug/built"), b"the build's own artifact\n").expect("the build's file");
    let working: PrewarmEntry = PrewarmEntry { staged_in: Some(staged_in), from: None, ..entry.clone() };
    let filled: Filled = super::fill_one_in_a_control(&fill_of_nowhere(), &working).expect("a directory a build made is kept");
    assert_eq!(filled, Filled::Kept);
    assert!(entry.to.join("debug/built").is_file(), "the build keeps what it made");
    assert!(!staging.exists(), "the staged state is given back");
    remove(root.as_path());
}

/// What:
///  A staging name another writer holds is never renamed over or removed,
/// and the worktree says so instead of leaving state a build would trust wrongly.
/// Why:
///   The names are the job's by convention, and only this job's own marker proves what stands there is its own;
/// anything else is somebody's work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdirSync(staging); expect(stageTheWorktreesOwnState(destination)).toThrow(); expect(existsSync(destination)).toBe(true)
/// ```
#[test]
fn a_staging_name_another_writer_holds_is_never_renamed_over() {
    let root: PathBuf = fixture("prewarm-staged-taken-name");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.to.join("debug")).expect("the copied build state");
    let git_directory: PathBuf = root.join("git-directory");
    let staging: PathBuf = staging_of(entry.to.as_path());
    std::fs::create_dir_all(staging.as_path()).expect("somebody else\'s directory at the staging name");
    let refused: Result<Staged, String> = super::stage_the_worktrees_own_state(entry.to.as_path(), Some(git_directory.as_path()));
    assert!(refused.is_err(), "a name this wrapper did not take is refused: {refused:?}");
    assert!(entry.to.join("debug").is_dir(), "the copied state stays where it is");
    remove(root.as_path());
}

/// What:
///  A worktree whose build directory is not there has nothing to move aside.
/// Why:
///   The plan asks this before it says the state is staged, and a worktree that never had one must not be sent to the
/// service for a clean of nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(stageTheWorktreesOwnState(missing)).toBe('nothing')
/// ```
#[test]
fn a_worktree_with_no_build_state_has_nothing_to_move_aside() {
    let root: PathBuf = fixture("prewarm-nothing-to-stage");
    let entry: PrewarmEntry = entry_of(root.as_path());
    let git_directory: PathBuf = root.join("git-directory");
    let staged: Staged = super::stage_the_worktrees_own_state(entry.to.as_path(), Some(git_directory.as_path())).expect("nothing is nothing");
    assert_eq!(staged, Staged::Nothing);
    assert!(!staging_of(entry.to.as_path()).exists(), "and no staging directory is made");
    assert!(!git_directory.join("info/exclude").exists(), "and no exclude file is written for a worktree with nothing to stage");
    remove(root.as_path());
}

/// What:
///  A marker a copy carried from another worktree claims nothing here:
///  the add stages the state it copied and hands it over to the filling,
///  so the new worktree is never left holding build state no repair will clean.
/// Why:
///   The ignored-state copy carries the staging name and the marker from the source worktree,
///  so a worktree added from one whose add was killed wakes up with a marker for a build directory
///  that is not here.
///  A marker an add cannot place means the repair is done again, never skipped:
///  a skip leaves the copy's artifacts where a build calls them fresh and names nothing that would
///  clean them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeMarker(carried); expect(stageTheWorktreesOwnState(destination)).toEqual({ staged: staging })
/// ```
#[test]
fn a_marker_a_copy_carried_from_another_worktree_never_skips_the_repair() {
    let root: PathBuf = fixture("prewarm-carried-marker");
    let entry: PrewarmEntry = entry_of(root.as_path());
    // What the copy carried: the source's build state under the worktree's build directory name, and the marker its
    //  killed add wrote before the rename that never happened.
    std::fs::create_dir_all(entry.to.join("debug")).expect("the copied build state");
    std::fs::write(entry.to.join("debug/copied"), b"the source's state\n").expect("a file of the copy");
    let staging: PathBuf = staging_of(entry.to.as_path());
    let marker: PathBuf = super::marker_of(staging.as_path());
    super::write_marker(marker.as_path(), &root.join("another-worktree/worktree-build")).expect("the marker the source worktree's killed add left");
    assert!(!staging.exists(), "the copy carried the marker and the build state, and no staging directory");
    let git_directory: PathBuf = root.join("git-directory");
    let staged: Staged = super::stage_the_worktrees_own_state(entry.to.as_path(), Some(git_directory.as_path())).expect("the carried marker is placed and the repair runs");
    assert_eq!(staged, Staged::Staged(staging.clone()), "the copied state is staged instead of skipped");
    assert!(!entry.to.exists(), "and the worktree holds no build directory a build would call fresh");
    assert!(staging.join("debug/copied").is_file(), "the copied state waits at the staging name for the filling's clean");
    assert!(super::marker_claims(marker.as_path(), entry.to.as_path()), "the carried marker is replaced by this add's claim on this build directory");
    remove(root.as_path());
}

/// What:
///  A copy that carried another worktree's staged state as well as its build state is repaired
///  over this worktree's own state:
///  the carried staging copy goes,
///  and the build directory the copy carried is staged for the filling's clean.
/// Why:
///   Two directories at the names of one build directory come from one source worktree:
///  the state its add moved aside, and the state a build made there afterwards.
///  Both name the source's paths, so the newer one is what a clean has to remove before a build can
///  trust anything here,
///  and the older copy is a directory of this job's own that nothing else will ever read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeMarker(carriedForAnotherWorktree); mkdirSync(staging); expect(stageTheWorktreesOwnState(destination)).toEqual({ staged: staging }); expect(staging/debug/copied).toExist()
/// ```
#[test]
fn a_staged_copy_a_copy_carried_is_replaced_by_the_state_this_worktree_holds() {
    let root: PathBuf = fixture("prewarm-carried-staging");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.to.join("debug")).expect("the copied build state");
    std::fs::write(entry.to.join("debug/built"), b"the newer state of the same tree\n").expect("a file of the copy");
    let staging: PathBuf = staging_of(entry.to.as_path());
    std::fs::create_dir_all(staging.join("debug")).expect("the staging copy the source worktree's add made");
    std::fs::write(staging.join("debug/copied"), b"the older state of the same tree\n").expect("a file of the carried staging copy");
    let marker: PathBuf = super::marker_of(staging.as_path());
    super::write_marker(marker.as_path(), &root.join("another-worktree/worktree-build")).expect("the source worktree's marker, carried here by the copy");
    let git_directory: PathBuf = root.join("git-directory");
    let staged: Staged = super::stage_the_worktrees_own_state(entry.to.as_path(), Some(git_directory.as_path())).expect("the carried names are taken back and this worktree's state is staged");
    assert_eq!(staged, Staged::Staged(staging.clone()), "this worktree's own copied state is staged");
    assert!(staging.join("debug/built").is_file(), "the newer state of the tree is what the filling cleans");
    assert!(!staging.join("debug/copied").exists(), "and the older carried copy of it is gone");
    assert!(super::marker_claims(marker.as_path(), entry.to.as_path()), "under this add's claim on this build directory");
    remove(root.as_path());
}

/// What:
///  A marker of this very build directory whose rename never happened stages the state again.
/// Why:
///   A kill between the marker's write and the rename leaves the marker and the build state, and
///  no staging directory, which is the state a second add of the same worktree inherits.
///  Believing that marker would leave the state where a build trusts it and record nothing about it,
///  so the repair runs again instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeMarker(forThisWorktree); expect(stageTheWorktreesOwnState(destination)).toEqual({ staged: staging })
/// ```
#[test]
fn a_marker_whose_rename_never_happened_stages_the_state_again() {
    let root: PathBuf = fixture("prewarm-marker-without-a-rename");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.to.join("debug")).expect("the copied build state");
    std::fs::write(entry.to.join("debug/copied"), b"the source's state\n").expect("a file of the copy");
    let staging: PathBuf = staging_of(entry.to.as_path());
    let marker: PathBuf = super::marker_of(staging.as_path());
    super::write_marker(marker.as_path(), entry.to.as_path()).expect("the marker this worktree's killed add wrote before its rename");
    assert!(!staging.exists(), "the kill left the marker and no staging directory");
    let git_directory: PathBuf = root.join("git-directory");
    let staged: Staged = super::stage_the_worktrees_own_state(entry.to.as_path(), Some(git_directory.as_path())).expect("the state is staged again");
    assert_eq!(staged, Staged::Staged(staging.clone()), "the state is staged again rather than skipped");
    assert!(!entry.to.exists(), "and it is out of the build directory a build would trust");
    assert!(super::marker_claims(marker.as_path(), entry.to.as_path()), "this add's claim replaces the killed add's marker");
    remove(root.as_path());
}

/// What:
///  A build directory whose own folder is not there yet is still filled:
/// the folder Cargo's layout names before the hash is made before the marker and the staging copy beside it.
/// Why:
///  `build.build-dir` writes an external build directory as `<root>/<two characters>/<hash>`,
/// and a worktree that has
///  never been built has neither,
/// so the marker the filling writes first was refused with `No such file or directory` and
///  nothing of the workspace was filled
///  (the binary suite's `package_repair::real_cargo_rebuilds_own_crates_with_an_external_build_directory`,
/// 2026-11-11).
/// What:
///  The repository's exclude file keeps its own lines when a staging name is named in it,
/// and naming them twice adds nothing.
/// Why:
///   The file is the person's, so a write that replaced it would discard what they excluded by hand,
/// and two adds racing to name the same patterns must not lose each other's line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileSync(exclude, 'hand-written/\n'); excludeTheStagingNames(gitDir); excludeTheStagingNames(gitDir); expect(readFileSync(exclude)).toContain('hand-written/')
/// ```
#[test]
fn the_exclude_file_keeps_its_own_lines_and_names_the_staging_once() {
    let root: PathBuf = fixture("prewarm-exclude");
    let git_dir: PathBuf = root.join("git-directory");
    std::fs::create_dir_all(git_dir.join("info")).expect("the Git directory's info folder");
    std::fs::write(git_dir.join("info/exclude"), b"# a person's own line\nhand-written/\n").expect("the person's exclude file");
    super::exclude_the_staging_names(git_dir.as_path()).expect("the patterns are named");
    super::exclude_the_staging_names(git_dir.as_path()).expect("naming them again is nothing");
    let named: String = std::fs::read_to_string(git_dir.join("info/exclude")).expect("the exclude file");
    assert!(named.contains("hand-written/"), "the person's own line stays: {named}");
    assert_eq!(named.matches(super::EXCLUDED_STAGING_PATTERNS[0]).count(), 1, "the pattern is named once: {named}");
    remove(root.as_path());
}

#[test]
fn a_build_directory_beside_a_folder_that_is_not_there_yet_is_still_filled() {
    let root: PathBuf = fixture("prewarm-missing-folder");
    let mut entry: PrewarmEntry = entry_of(root.as_path());
    // The two characters Cargo's own layout puts above the hash.
    entry.to = root.join("worktree-build/6f/0123456789abcd");
    std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
    std::fs::create_dir_all(entry.from.as_ref().expect("the source's build directory").join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.as_ref().expect("the source's build directory").join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    let filled: Result<Filled, String> = super::fill_one_in_a_control(&fill_of_nowhere(), &entry);
    let Err(reason) = filled else { panic!("a clean that cannot run is reported: {filled:?}") };
    // The clean could not run, so the repair gives way before it ever resolves the lock (there is no Git directory here), and the
    //  message says nothing of this workspace was cleaned: the marker and the staging copy were made, and nothing was published.
    assert!(reason.contains("nothing of this workspace was cleaned"), "the filing got as far as the clean, so the marker and the staging copy were made: {reason}");
    assert!(!entry.to.exists(), "a clean that failed publishes nothing");
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
        let entry: PrewarmEntry = PrewarmEntry { root: workspace.clone(), from: Some(source_build.clone()), to: destination_build.clone(), members: vec![String::from(MEMBER)], staged_in: None };
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
    let entry: PrewarmEntry = PrewarmEntry { members: vec![String::from(MEMBER)], ..named };
    std::fs::create_dir_all(entry.root.join("src")).expect("the workspace");
    let manifest: String = format!("[workspace]\nmembers = [\".\"]\n\n[package]\nname = \"{MEMBER}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n{DEPENDENCY} = \"1\"\n");
    std::fs::write(entry.root.join("Cargo.toml"), manifest).expect("the manifest");
    std::fs::write(entry.root.join("src/lib.rs"), format!("pub fn {MEMBER}() -> &'static str {{ \"{MEMBER}\" }}\n")).expect("the crate");
    let cargo: PathBuf = cargo_on_the_search_path();
    let built: std::process::Output = std::process::Command::new(cargo)
        .args(["build", "--offline", "--quiet"])
        .current_dir(entry.root.as_path())
        .env(BUILD_DIRECTORY_VARIABLE, entry.from.as_ref().expect("the source's build directory").as_os_str())
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
/// whether it was there and stale or absent,
/// when nothing else writes it.
/// Why:
///   `cargo clean` resolves the worktree's workspace even when it deletes only from directories of
/// this job's own,
/// which rewrites a lock that no longer matches its manifest and creates one that is missing.
/// The add's own clean puts the lock back (`package_repair_cargo_lock.rs`),
/// and the service's clean runs after the add has returned,
/// so a lock it left rewritten would change a tracked file in the person's worktree behind their back.
/// The clean itself is given `--locked`,
/// the rewrite is asked for as a resolution of its own,
/// and the checked-out bytes go back over it.
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

/// What the clean's own resolution writes into the worktree's lock,
/// and what the person's own `cargo add` writes while that clean runs.
/// Why:
///   The filling's clean runs after the add returned,
/// so the two texts say which one stands in the file afterwards:
/// the person's must.
const THE_RESOLUTIONS_LOCK: &str = "# what the worktree resolution wrote";

/// The text the person's own write leaves,
/// which is never Cargo's rewrite.
const THE_PERSONS_LOCK: &str = "# what the person wrote instead";

/// What:
///  A stand-in Cargo for the controls about the lock:
/// its resolution writes Cargo's rewrite,
/// its first clean refuses the stale lock,
/// and its second clean has the person's own write land while it runs.
/// Why:
///   The sequence is the whole rule:
/// a clean that refuses starts no write,
/// the resolution is what writes the lock,
/// and the clean that runs afterwards cannot write it.
/// None of that needs a real Cargo,
/// and a stand-in makes a person's write land at a known moment inside the clean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const program = standIn({ metadata: resolutionsLock, refusesOnce: true, thenPersonWrites: personsLock });
/// ```
fn cargo_that_takes_the_persons_lock_as_it_runs(root: &Path) -> PathBuf {
    let record: PathBuf = root.join("cargo-record");
    let program: PathBuf = root.join("cargo-that-resolves");
    let body: String = format!(
        "#!/bin/sh\necho \"$*\" >> '{record}'\nif [ \"$1\" = metadata ]; then\n  printf '%s' '{resolved}' > Cargo.lock\n  exit 0\nfi\nif [ ! -e refused-once ]; then\n  : > refused-once\n  echo 'error: cannot update the lock file because --locked was passed to prevent this' >&2\n  exit 101\nfi\nprintf '%s' '{persons}' > Cargo.lock\nexit 0\n",
        record = record.display(),
        resolved = THE_RESOLUTIONS_LOCK,
        persons = THE_PERSONS_LOCK,
    );
    crate::test_support::executable(program.as_path(), body.as_bytes());
    return program;
}

/// The lines a stand-in Cargo recorded.
fn recorded(record: &Path) -> Vec<String> {
    return std::fs::read_to_string(record).map(|text: String| { return text.lines().map(str::to_owned).collect(); }).unwrap_or_default();
}

/// What:
///  A lock the person writes while the filling's clean runs,
///  stale before or not there at all,
///  is left exactly as they wrote it,
/// and the copy is still filled and published.
/// Why:
///   This clean runs after the add returned,
/// so the person is working in the worktree and a write standing there when the clean ends may be theirs,
/// and replacing it or removing it would discard a file they wrote.
/// The clean is `--locked` and the resolution is a step of its own,
/// so the file is only ever put back over bytes that resolution wrote.
#[test]
fn a_lock_the_person_writes_while_a_clean_runs_is_left_exactly_as_they_wrote_it() {
    // The two states the checked-out worktree can be in when the person starts.
    const LOCK_STATES: [&str; 2] = ["stale", "absent"];
    for state in LOCK_STATES {
        let root: PathBuf = fixture(format!("prewarm-person-lock-{state}").as_str());
        let entry: PrewarmEntry = entry_of(root.as_path());
        std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
        std::fs::create_dir_all(entry.from.as_ref().expect("the source's build directory").join("debug/deps")).expect("the source's dependency build");
        std::fs::write(entry.from.as_ref().expect("the source's build directory").join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
        let lock: PathBuf = entry.root.join(LOCK_FILE);
        if state == "stale" {
            std::fs::write(lock.as_path(), b"# a lock that no longer matches its manifest\n").expect("the checked-out lock");
        }
        let program: PathBuf = cargo_that_takes_the_persons_lock_as_it_runs(root.as_path());
        let fill: Fill = Fill { cargo: program, limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: std::time::Duration::from_secs(5) }, kept_in: Some(root.join("git-dir")), worktree: root.clone(), filling: String::from("a-controls-filling") };
        let filled: Filled = super::fill_one_in_a_control(&fill, &entry).expect("the copy is filled");
        assert_eq!(filled, Filled::Published, "the copy is published under the worktree's build directory name ({state})");
        let after: Option<Vec<u8>> = std::fs::read(lock.as_path()).ok();
        assert_eq!(after.as_deref(), Some(THE_PERSONS_LOCK.as_bytes()), "the person's own lock is exactly what stands there ({state})");
        let lines: Vec<String> = recorded(root.join("cargo-record").as_path());
        assert_eq!(lines.len(), 3, "a refused clean, the resolution, and the clean that runs ({state}): {lines:?}");
        assert!(lines[0].starts_with("clean ") && lines[1].starts_with("metadata ") && lines[2].starts_with("clean "), "the lock is resolved between the two cleans ({state}): {lines:?}");
        assert!(lines[0].contains("--locked") && lines[2].contains("--locked"), "neither clean may write the lock ({state}): {lines:?}");
        remove(root.as_path());
    }
}

/// What:
///  A lock that still matches its manifest costs no resolution and is never written.
/// Why:
///   Most worktrees have a lock Cargo agrees with,
/// and asking for a resolution there would start a second Cargo for nothing;
/// the clean alone runs,
/// with `--locked`,
///  so the person's file is not opened for writing at all.
#[test]
fn a_lock_that_is_current_costs_no_resolution_and_is_never_written() {
    let root: PathBuf = fixture("prewarm-lock-current");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
    std::fs::create_dir_all(entry.from.as_ref().expect("the source's build directory").join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.as_ref().expect("the source's build directory").join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    let lock: PathBuf = entry.root.join(LOCK_FILE);
    std::fs::write(lock.as_path(), THE_RESOLUTIONS_LOCK.as_bytes()).expect("a lock the clean agrees with");
    let record: PathBuf = root.join("cargo-record");
    let program: PathBuf = root.join("cargo-that-cleans-once");
    let body: String = format!("#!/bin/sh\necho \"$*\" >> '{record}'\nexit 0\n", record = record.display());
    crate::test_support::executable(program.as_path(), body.as_bytes());
    let fill: Fill = Fill { cargo: program, limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: std::time::Duration::from_secs(5) }, kept_in: Some(root.join("git-dir")), worktree: root.clone(), filling: String::from("a-controls-filling") };
    let filled: Filled = super::fill_one_in_a_control(&fill, &entry).expect("the copy is filled");
    assert_eq!(filled, Filled::Published, "the copy is published under the worktree's build directory name");
    assert_eq!(std::fs::read(lock.as_path()).ok().as_deref(), Some(THE_RESOLUTIONS_LOCK.as_bytes()), "a current lock is left exactly as it was");
    let lines: Vec<String> = recorded(record.as_path());
    assert_eq!(lines.len(), 1, "one clean and no resolution: {lines:?}");
    assert!(lines[0].starts_with("clean ") && lines[0].contains("--locked"), "the clean runs with the lock held still: {lines:?}");
    remove(root.as_path());
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
        std::fs::create_dir_all(entry.from.as_ref().expect("the source's build directory").join("debug/deps")).expect("the source's dependency build");
        std::fs::write(entry.from.as_ref().expect("the source's build directory").join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
        let staged: PathBuf = staging_of(entry.to.as_path());
        std::fs::create_dir_all(staged.join("keep-me")).expect("a directory this filling did not make");
        std::fs::write(staged.join("keep-me/a-file"), b"someone's work").expect("the file");
        let marker: PathBuf = super::marker_of(staged.as_path());
        if stranger == "marker" {
            // A file at the marker name that does not hold this job's text claims nothing.
            std::fs::write(marker.as_path(), b"a person's own note").expect("a file this filling did not make");
        }
        let fill: Fill = Fill { cargo: cargo_that_succeeds(root.as_path()), limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: std::time::Duration::from_secs(5) }, kept_in: None, worktree: root.clone(), filling: String::from("a-controls-filling") };
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
    std::fs::create_dir_all(entry.from.as_ref().expect("the source's build directory").join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.as_ref().expect("the source's build directory").join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    // A clean that makes the target directory it is given, as a real Cargo does.
    let program: PathBuf = root.join("cargo-that-cleans");
    crate::test_support::executable(
        program.as_path(),
        b"#!/bin/sh\nmkdir --parents \"$CARGO_TARGET_DIR/debug\"\necho cleaned > \"$CARGO_TARGET_DIR/debug/state\"\nexit 0\n",
    );
    let fill: Fill = Fill { cargo: program, limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: std::time::Duration::from_secs(5) }, kept_in: None, worktree: root.clone(), filling: String::from("a-controls-filling") };
    let filled: Filled = super::fill_one_in_a_control(&fill, &entry).expect("the copy is cleaned and published");
    assert_eq!(filled, Filled::Published, "the copy is published under the worktree's build directory name");
    let staged: PathBuf = staging_of(entry.to.as_path());
    assert!(!super::target_of(staged.as_path()).exists(), "the clean's target directory {} is left behind", super::target_of(staged.as_path()).display());
    assert!(entry.to.exists(), "the published copy is the worktree's build directory");
    remove(root.as_path());
}

/// What:
///  A worktree's build directory that sits below a directory nothing has made yet is filled:
///  the filling makes the directory the destination and its staging name live in,
///  and publishes the copy there.
/// Why:
///  Cargo keys an external build directory by the workspace's path,
///  and the name it keys it by is two levels below the root the person's configuration names:
///  `build-dir = "~/build/{workspace-path-hash}"` resolves to `~/build/8d/0e503e8832e433`,
///  as the record of a wrapped add names it.
///  Neither level exists before the first build of a new worktree,
///  and the marker that claims the staging name sits beside the destination,
///  so a filling that took its names first failed writing that marker with `ENOENT` and filled nothing
///  (`real_cargo_rebuilds_own_crates_with_an_external_build_directory`,
///  binary tier).
#[test]
fn a_build_directory_below_directories_that_do_not_exist_yet_is_filled() {
    let root: PathBuf = fixture("prewarm-missing-parent");
    let named: PrewarmEntry = entry_of(root.as_path());
    // The shape Cargo resolves an external build directory to, and the source's own build directory beside it.
    let entry: PrewarmEntry = PrewarmEntry { to: root.join("cargo-build/8d/0e503e8832e433"), ..named };
    std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
    std::fs::create_dir_all(entry.from.as_ref().expect("the source's build directory").join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.as_ref().expect("the source's build directory").join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    std::fs::create_dir_all(root.join("cargo-build")).expect("the root the person's configuration names");
    assert!(!entry.to.parent().expect("the destination has a directory").exists(), "the destination's own directory is not there before the filling");
    let fill: Fill = Fill { cargo: cargo_that_succeeds(root.as_path()), limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: std::time::Duration::from_secs(5) }, kept_in: None, worktree: root.clone(), filling: String::from("a-controls-filling") };
    let filled: Filled = super::fill_one_in_a_control(&fill, &entry).expect("the worktree is filled");
    assert_eq!(filled, Filled::Published, "the copy is published under the worktree's build directory name");
    assert_eq!(std::fs::read(entry.to.join("debug/deps/libserde.rlib")).expect("the published copy"), b"a dependency build");
    assert!(!staging_of(entry.to.as_path()).exists(), "the staging directory is published and gone");
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
    let entry: PrewarmEntry = PrewarmEntry { root: nested.clone(), from: Some(root.join("source-build")), to: destination_build, members: vec![String::from(MEMBER)], staged_in: None };
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
    /// The bytes of a pathname,
    /// for the C interface.
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
///   The controls below are about a name beside the destination,
/// and neither of them reads a manifest.
fn an_entry_with_a_source_build(root: &Path) -> PrewarmEntry {
    let entry: PrewarmEntry = entry_of(root);
    std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
    std::fs::create_dir_all(entry.from.as_ref().expect("the source's build directory").join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.as_ref().expect("the source's build directory").join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
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
    let waiting: Option<Result<Filled, String>> = crate::test_support::finished_within_at(FILLING_DEADLINE, move || {
        let fill: Fill = Fill { cargo: cargo_that_succeeds(root.as_path()), limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: FILLING_DEADLINE }, kept_in: None, worktree: root.clone(), filling: String::from("a-controls-filling") };
        return super::fill_one_in_a_control(&fill, &entry);
    });
    let filled: Result<Filled, String> = waiting.expect("the filling must not wait on a named pipe at the marker name");
    assert_eq!(filled, Ok(Filled::Published), "the copy is published under names of this run's own");
    assert!(marker.exists(), "the named pipe this filling did not make is left where it is");
    remove(covered.as_path());
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
    let fill: Fill = Fill { cargo: cargo_that_succeeds(root.as_path()), limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: FILLING_DEADLINE }, kept_in: None, worktree: root.clone(), filling: String::from("a-controls-filling") };
    let filled: Result<Filled, String> = super::fill_one_in_a_control(&fill, &entry);
    crate::private_storage::AFTER_THE_STAGING_FILE_IS_WRITTEN.with(|hook| { *hook.borrow_mut() = None; });
    assert_eq!(filled, Ok(Filled::Published), "the worktree is filled under names of this run's own");
    assert_eq!(std::fs::read(marker.as_path()).expect("the other command's file"), ANOTHER_COMMANDS_FILE, "a file this filling did not write is never removed");
    remove(covered.as_path());
}

/// What:
///  A clean stopped because Cargo refused the lock names the lock's own command,
///  the workspace and the clean the add would have run,
///  and a clean stopped for any other reason says only what the tool said.
/// Why:
///   The remedy is chosen from Cargo's own words,
/// and a later Cargo that stops naming the lock there would send every lock refusal to the generic
/// sentence without anything failing,
/// so which words take which branch is pinned here.
#[test]
fn a_stopped_clean_names_the_lock_command_only_when_cargo_named_the_lock() {
    let root: PathBuf = fixture("prewarm-remedy-branches");
    let entry: PrewarmEntry = entry_of(root.as_path());
    // What Cargo prints for a lock it would have had to change, measured on this host.
    let cargo_refusal: String = format!("error: cannot update the lock file {}/{LOCK_FILE} because --locked was passed to prevent this", entry.root.display());
    let remedy: String = super::what_to_do_next(&entry, cargo_refusal.as_str());
    let own_clean: String = manual_command(entry.root.as_path(), entry.members.as_slice());
    assert!(remedy.contains("cargo update --offline --workspace"), "the remedy names the lock's own command: {remedy}");
    assert!(remedy.contains(entry.root.to_string_lossy().as_ref()), "the remedy names the workspace whose lock it is: {remedy}");
    assert!(remedy.contains(own_clean.as_str()), "the remedy names the clean the add would have run ({own_clean}): {remedy}");
    let elsewhere: String = super::what_to_do_next(&entry, "a program that is not there");
    assert!(!elsewhere.contains("cargo update"), "a failure that names no lock is not answered with one: {elsewhere}");
    assert!(!elsewhere.contains(own_clean.as_str()), "a failure that names no lock is not answered with the clean: {elsewhere}");
    assert!(elsewhere.contains("the next add fills the copy"), "a failure that names no lock still says the copy is filled next time: {elsewhere}");
    remove(root.as_path());
}


/// What:
///  A write a person made to the lock while the repair resolved it is handed back,
///  and the repair gives way instead of replacing it.
/// Why:
///   The resolution is made in a mirror of the workspace, and the only write to the person's file is the
///  exchange that hands back whatever stood there:
/// a write made at any moment before that exchange is in this run's hands,
/// and it is put back rather than truncated.
/// The stand-in Cargo writes the person's bytes into the worktree while its resolution runs,
/// which is the moment no write-by-write comparison can see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeDuringTheResolution(personsBytes); const filled = fillOne(fill, entry); expect(filled).toBeFailed(); expect(read(lock)).toBe(personsBytes)
/// ```
#[test]
fn a_write_the_resolution_meets_is_handed_back_and_the_repair_gives_way() {
    let root: PathBuf = fixture("prewarm-lock-write-during-the-resolution");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
    std::fs::create_dir_all(entry.from.as_ref().expect("the source's build directory").join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.as_ref().expect("the source's build directory").join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    let lock: PathBuf = entry.root.join(LOCK_FILE);
    let checked_out: &str = "# the checked-out stale lock\n";
    let persons: &str = "# the person wrote this while the repair resolved\n";
    let resolution: &str = "# what the resolution wrote\n";
    std::fs::write(lock.as_path(), checked_out.as_bytes()).expect("the checked-out lock");
    let program: PathBuf = root.join("cargo-that-resolves-while-a-person-writes");
    let body: String = format!(
        "#!/bin/sh\nDIR='{dir}'\nif [ \"$1\" = clean ]; then\n  if [ ! -e \"$DIR/refused-once\" ]; then : > \"$DIR/refused-once\"; echo 'error: cannot update the lock file because --locked was passed to prevent this' >&2; exit 101; fi\n  exit 0\nfi\nif [ \"$1\" = metadata ]; then\n  printf '%s' '{persons}' > '{lock}'\n  printf '%s' '{resolution}' > Cargo.lock\n  exit 0\nfi\nexit 0\n",
        dir = root.display(),
        persons = persons.trim_end(),
        lock = lock.display(),
        resolution = resolution.trim_end(),
    );
    crate::test_support::executable(program.as_path(), body.as_bytes());
    let fill: Fill = Fill {
        cargo: program,
        limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: std::time::Duration::from_secs(10) },
        kept_in: Some(root.join("git-dir")),
        worktree: root.clone(),
        filling: String::from("a-controls-filling"),
    };
    let filled: Result<Filled, String> = super::fill_one_in_a_control(&fill, &entry);
    let Err(reason) = filled else { panic!("the repair gives way rather than replace the write: {filled:?}") };
    assert!(reason.contains("gives way"), "the message says the repair gave way: {reason}");
    assert_eq!(std::fs::read(lock.as_path()).ok().as_deref(), Some(persons.trim_end().as_bytes()), "the person's bytes stand in their file");
    assert!(!entry.to.exists(), "nothing is published");
    assert!(!staging_of(entry.to.as_path()).exists(), "and the staging copy is removed");
    remove(root.as_path());
}

/// What:
///  A lock a person creates while the repair resolves one the checkout does not have is never written over.
/// Why:
///   With no lock at the start the resolution is created with `create_new`,
///  which refuses rather than replaces the file a person made in the meantime;
/// the repair gives way and their file stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const filled = fillOne(fill, entry); expect(filled).toBeFailed(); expect(read(lock)).toBe(personsBytes)
/// ```
#[test]
fn a_lock_a_person_creates_while_the_repair_resolves_one_is_never_written_over() {
    let root: PathBuf = fixture("prewarm-lock-created-while-resolving");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
    std::fs::create_dir_all(entry.from.as_ref().expect("the source's build directory").join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.as_ref().expect("the source's build directory").join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    let lock: PathBuf = entry.root.join(LOCK_FILE);
    let persons: &str = "# the person created this while the repair resolved\n";
    let program: PathBuf = root.join("cargo-that-resolves-while-a-person-creates");
    let body: String = format!(
        "#!/bin/sh\nDIR='{dir}'\nif [ \"$1\" = clean ]; then\n  if [ ! -e \"$DIR/refused-once\" ]; then : > \"$DIR/refused-once\"; echo 'error: cannot update the lock file because --locked was passed to prevent this' >&2; exit 101; fi\n  exit 0\nfi\nif [ \"$1\" = metadata ]; then\n  printf '%s' '{persons}' > '{lock}'\n  printf '%s' '# what the resolution wrote' > Cargo.lock\n  exit 0\nfi\nexit 0\n",
        dir = root.display(),
        persons = persons.trim_end(),
        lock = lock.display(),
    );
    crate::test_support::executable(program.as_path(), body.as_bytes());
    let fill: Fill = Fill {
        cargo: program,
        limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: std::time::Duration::from_secs(10) },
        kept_in: Some(root.join("git-dir")),
        worktree: root.clone(),
        filling: String::from("a-controls-filling"),
    };
    let filled: Result<Filled, String> = super::fill_one_in_a_control(&fill, &entry);
    assert!(filled.is_err(), "the repair gives way rather than write over the file a person made: {filled:?}");
    assert_eq!(std::fs::read(lock.as_path()).ok().as_deref(), Some(persons.trim_end().as_bytes()), "the file the person created stands as they left it");
    assert!(!entry.to.exists(), "nothing is published");
    remove(root.as_path());
}

/// What:
///  The resolution is made in a mirror below the Git directory,
///  and the worktree's own lock is untouched while it runs.
/// Why:
///   A resolution that runs in the worktree is a write of Cargo's to the person's file,
/// and a write of theirs made a moment earlier is truncated by it with no copy kept;
/// the mirror is what takes that write out of their file altogether.
/// The stand-in records the directory it resolved in and what the lock held at that moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const filled = fillOne(fill, entry); expect(filled).toBePublished(); expect(recorded.cwd).not.toBe(entry.root); expect(recorded.lockWhileResolving).toBe(checkedOut)
/// ```
#[test]
fn the_resolution_is_made_in_a_mirror_and_never_in_the_worktrees_lock() {
    let root: PathBuf = fixture("prewarm-resolution-in-a-mirror");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
    std::fs::create_dir_all(entry.from.as_ref().expect("the source's build directory").join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.as_ref().expect("the source's build directory").join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    let lock: PathBuf = entry.root.join(LOCK_FILE);
    let checked_out: &str = "# the checked-out stale lock\n";
    std::fs::write(lock.as_path(), checked_out.as_bytes()).expect("the checked-out lock");
    let record: PathBuf = root.join("the-resolution-record");
    let git_dir: PathBuf = root.join("git-dir");
    let program: PathBuf = root.join("cargo-that-records-where-it-resolves");
    let body: String = format!(
        "#!/bin/sh\nDIR='{dir}'\nif [ \"$1\" = clean ]; then\n  if [ ! -e \"$DIR/refused-once\" ]; then : > \"$DIR/refused-once\"; echo 'error: cannot update the lock file because --locked was passed to prevent this' >&2; exit 101; fi\n  exit 0\nfi\nif [ \"$1\" = metadata ]; then\n  printf '%s\\n%s\\n' \"$(pwd)\" \"$(cat '{lock}')\" >> '{record}'\n  printf '%s' '# what the resolution wrote' > Cargo.lock\n  exit 0\nfi\nexit 0\n",
        dir = root.display(),
        lock = lock.display(),
        record = record.display(),
    );
    crate::test_support::executable(program.as_path(), body.as_bytes());
    let fill: Fill = Fill {
        cargo: program,
        limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: std::time::Duration::from_secs(10) },
        kept_in: Some(git_dir.clone()),
        worktree: root.clone(),
        filling: String::from("a-controls-filling"),
    };
    let filled: Filled = super::fill_one_in_a_control(&fill, &entry).expect("the copy is cleaned and published");
    assert_eq!(filled, Filled::Published);
    let recorded: String = std::fs::read_to_string(record.as_path()).expect("the resolution recorded itself");
    let mut lines = recorded.lines();
    let resolved_in: &str = lines.next().expect("the directory the resolution ran in");
    let standing_then: &str = lines.next().expect("what the lock held while it resolved");
    assert_ne!(resolved_in, entry.root.to_str().expect("the workspace path"), "the resolution did not run in the worktree");
    assert!(resolved_in.starts_with(git_dir.to_str().expect("the Git directory")), "it ran in a mirror under the Git directory: {resolved_in}");
    assert_eq!(standing_then, checked_out.trim_end(), "the person's lock was untouched while the resolution ran");
    assert_eq!(std::fs::read(lock.as_path()).ok().as_deref(), Some(checked_out.as_bytes()), "and the checked-out bytes are back after the filling");
    remove(root.as_path());
}

/// What:
///  A filling that is still at work keeps its names:
///  the next filling stages under names of its own and removes nothing of the live one's.
/// Why:
///   The marker alone says a filling wrote the names,
/// and a killed filling leaves the same marker a live one holds,
/// so the lease is what tells them apart:
/// a second filling that finds it taken never takes the names for a dead filling's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const live = claimTheNames(staging); const filled = fillOne(fill, entry); expect(filled).toBePublished(); expect(stagedByTheLiveFilling).toExist()
/// ```
#[test]
fn a_filling_that_is_alive_keeps_its_names_from_the_next_one() {
    let root: PathBuf = fixture("prewarm-live-filling-keeps-its-names");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
    std::fs::create_dir_all(entry.from.as_ref().expect("the source's build directory").join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.as_ref().expect("the source's build directory").join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    let usual: PathBuf = staging_of(entry.to.as_path());
    let marker: PathBuf = super::marker_of(usual.as_path());
    let live: super::TheLease = match super::claim_the_names(usual.as_path()) {
        super::TheClaim::Held(lease) => lease,
        other => panic!("the first filling takes the lease: {other:?}"),
    };
    std::fs::create_dir_all(usual.join("debug")).expect("the live filling's staging copy");
    std::fs::write(usual.join("debug/the-live-fillings-state"), b"held\n").expect("a file of the live filling's copy");
    super::write_marker(marker.as_path(), entry.to.as_path()).expect("the live filling's marker");
    let fill: Fill = Fill {
        cargo: cargo_that_succeeds(root.as_path()),
        limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: FILLING_DEADLINE },
        kept_in: Some(root.join("git-dir")),
        worktree: root.clone(),
        filling: String::from("a-controls-filling"),
    };
    let filled: Filled = super::fill_one_in_a_control(&fill, &entry).expect("the second filling fills under names of its own");
    assert_eq!(filled, Filled::Published, "the worktree's build directory is filled");
    assert!(usual.join("debug/the-live-fillings-state").is_file(), "the live filling's staging copy is not removed");
    assert!(marker.is_file(), "nor its marker");
    drop(live);
    remove(root.as_path());
}

/// What:
///  The same leftovers with no filling holding them are a dead filling's,
///  and the next filling takes the names and removes them.
/// Why:
///   This is the state a kill leaves,
/// and a proof that never lets the names be taken would strand every killed filling's copy for ever.
/// The pair with `a_filling_that_is_alive_keeps_its_names_from_the_next_one` is what shows the lease, and
///  not the marker, decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantLeftoversWithoutALease(); const filled = fillOne(fill, entry); expect(filled).toBePublished(); expect(staging).not.toExist()
/// ```
#[test]
fn a_filling_a_kill_left_has_its_names_taken_and_its_leftovers_removed() {
    let root: PathBuf = fixture("prewarm-dead-filling-loses-its-names");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.root.as_path()).expect("the workspace the clean runs in");
    std::fs::create_dir_all(entry.from.as_ref().expect("the source's build directory").join("debug/deps")).expect("the source's dependency build");
    std::fs::write(entry.from.as_ref().expect("the source's build directory").join("debug/deps/libserde.rlib"), b"a dependency build").expect("the dependency");
    let usual: PathBuf = staging_of(entry.to.as_path());
    let marker: PathBuf = super::marker_of(usual.as_path());
    std::fs::create_dir_all(usual.join("debug")).expect("a killed filling's staging copy");
    std::fs::write(usual.join("debug/the-killed-fillings-state"), b"left behind\n").expect("a file of the killed filling's copy");
    super::write_marker(marker.as_path(), entry.to.as_path()).expect("the killed filling's marker");
    let fill: Fill = Fill {
        cargo: cargo_that_succeeds(root.as_path()),
        limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: FILLING_DEADLINE },
        kept_in: Some(root.join("git-dir")),
        worktree: root.clone(),
        filling: String::from("a-controls-filling"),
    };
    let filled: Filled = super::fill_one_in_a_control(&fill, &entry).expect("a lease nobody holds is the next filling's");
    assert_eq!(filled, Filled::Published, "the worktree's build directory is filled");
    assert!(!usual.exists(), "the killed filling's staging copy is removed with the names it held");
    assert!(!marker.exists(), "and its marker goes with it");
    assert!(!crate::package_repair_prewarm_claim::lease_of(usual.as_path()).exists(), "and no lease is left behind");
    remove(root.as_path());
}

/// What:
///  An add never takes the names of a filling that is alive:
///  the copied build state stays where it is and the add says why.
/// Why:
///   The add runs while the service may be filling the same build directory,
/// and the add deletes leftovers a marker claims;
/// with the lease held by a live filling, taking its marker for a dead one's would remove a copy in use,
/// so the add gives way and the test shows it happening.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const live = claimTheNames(staging); expect(stageTheWorktreesOwnState(destination)).toBeRefused(); expect(copiedState).toExist()
/// ```
#[test]
fn an_add_never_takes_the_names_of_a_filling_that_is_alive() {
    let root: PathBuf = fixture("prewarm-add-meets-a-live-filling");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.to.join("debug")).expect("the copied build state");
    std::fs::write(entry.to.join("debug/copied"), b"the source's state\n").expect("a file of the copy");
    let usual: PathBuf = staging_of(entry.to.as_path());
    let marker: PathBuf = super::marker_of(usual.as_path());
    let live: super::TheLease = match super::claim_the_names(usual.as_path()) {
        super::TheClaim::Held(lease) => lease,
        other => panic!("the filling takes the lease: {other:?}"),
    };
    super::write_marker(marker.as_path(), entry.to.as_path()).expect("the live filling's marker");
    let git_directory: PathBuf = root.join("git-directory");
    let staged: Result<Staged, String> = super::stage_the_worktrees_own_state(entry.to.as_path(), Some(git_directory.as_path()));
    assert!(staged.is_err(), "the add gives way rather than take a live filling's names: {staged:?}");
    let Err(reason) = staged else { unreachable!() };
    assert!(reason.contains("still at work"), "the refusal names the filling that is at work: {reason}");
    assert!(entry.to.join("debug/copied").is_file(), "the copied state stays where it is");
    assert!(marker.is_file(), "and the live filling's marker stands");
    drop(live);
    remove(root.as_path());
}


/// What:
///  A line of an exclude file as this job reads it,
///  so a control compares the file with the patterns the way the helper does.
/// Why:
///   The controls below state exactly what stands in the file afterwards,
/// and a comparison through the same trimming keeps the control about the helper and not about a second reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linesIn = (path) => readFileSync(path, "utf8").split("\n").map((line) => line.trimEnd());
/// ```
fn lines_in(path: &Path) -> Vec<String> {
    let bytes: Vec<u8> = std::fs::read(path).unwrap_or_default();
    return String::from_utf8_lossy(bytes.as_slice()).split('\n').map(|line| return String::from(line.trim_end())).collect();
}

/// What:
///  Every pattern the exclude file lacks is named in it, and none it already has is written twice,
///  whichever subset of them an older release, a cut-short append or a person's hand left there.
/// Why:
///   The check that once looked for the first pattern alone took a file that named it as complete for good,
/// so the lease pattern never reached a repository an older release had named the others in:
/// the lease file then showed in the person's `git status` and could be added by name.
/// The empty subset is the left edge and the full set the right one,
/// and the person's own line must survive every one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const subset of subsets(PATTERNS)) { write(subset.join("\n") + "\n"); excludeTheStagingNames(gitDir); expect(eachPatternOnce(read())).toBe(true) }
/// ```
#[test]
fn every_pattern_the_exclude_file_lacks_is_named_and_none_is_written_twice() {
    let root: PathBuf = fixture("prewarm-exclude-subsets");
    let git_dir: PathBuf = root.join("git-directory");
    std::fs::create_dir_all(git_dir.join("info")).expect("the Git directory's info folder");
    let exclude: PathBuf = git_dir.join("info/exclude");
    let subsets: usize = 1 << super::EXCLUDED_STAGING_PATTERNS.len();
    for subset in 0..subsets {
        let mut present: Vec<&str> = Vec::new();
        for (index, pattern) in super::EXCLUDED_STAGING_PATTERNS.iter().enumerate() {
            if subset & (1 << index) != 0 {
                present.push(pattern);
            }
        }
        let mut text: String = String::from("# a person's own line\n");
        for pattern in &present {
            text.push_str(pattern);
            text.push('\n');
        }
        std::fs::write(exclude.as_path(), text.as_bytes()).expect("the exclude file an earlier release or a person left");
        super::exclude_the_staging_names(git_dir.as_path()).expect("the patterns are named");
        let lines: Vec<String> = lines_in(exclude.as_path());
        assert!(lines.iter().any(|line| return line == "# a person's own line"), "subset {subset:04b}: the person's own line stays: {lines:?}");
        for pattern in super::EXCLUDED_STAGING_PATTERNS {
            let count: usize = lines.iter().filter(|line| return line.as_str() == pattern).count();
            assert_eq!(count, 1, "subset {subset:04b}: {pattern} stands on a line of its own exactly once: {lines:?}");
        }
        // Naming them again writes nothing more, whatever the subset left behind.
        super::exclude_the_staging_names(git_dir.as_path()).expect("naming them again is nothing");
        let after: Vec<String> = lines_in(exclude.as_path());
        assert_eq!(after, lines, "subset {subset:04b}: a second call appends nothing");
    }
    remove(root.as_path());
}

/// What:
///  An exclude file whose last line has no line end gains one before the patterns this job writes,
///  so a line a kill cut short never carries the next pattern.
/// Why:
///   Appending "…target/\n" to a file that ends with "*.monochromatic-prewarm" would make one line of both,
/// and neither pattern would stand on a line of its own:
/// the ignored names would then show, and the file would hold a line Git reads as neither.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write("*.monochromatic-prewarm.owner"); excludeTheStagingNames(gitDir); expect(lines).toEqual(["*.monochromatic-prewarm.owner", ...theOtherThree])
/// ```
#[test]
fn a_last_line_without_a_line_end_never_carries_the_next_pattern() {
    // Two states of a file a kill cut short: its last line is one of this job's patterns, and it is a person's.
    for last_line in [super::EXCLUDED_STAGING_PATTERNS[2], "# a person's cut-short line"] {
        let root: PathBuf = fixture("prewarm-exclude-no-line-end");
        let git_dir: PathBuf = root.join("git-directory");
        std::fs::create_dir_all(git_dir.join("info")).expect("the Git directory's info folder");
        let exclude: PathBuf = git_dir.join("info/exclude");
        std::fs::write(exclude.as_path(), last_line.as_bytes()).expect("a file whose last line has no line end");
        super::exclude_the_staging_names(git_dir.as_path()).expect("the patterns are named");
        let lines: Vec<String> = lines_in(exclude.as_path());
        assert_eq!(lines[0], last_line, "the cut-short line is a line of its own afterwards: {lines:?}");
        for pattern in super::EXCLUDED_STAGING_PATTERNS {
            let count: usize = lines.iter().filter(|line| return line.as_str() == pattern).count();
            assert_eq!(count, 1, "{pattern} stands on a line of its own exactly once after {last_line:?}: {lines:?}");
        }
        remove(root.as_path());
    }
}

/// What:
///  A named pipe at the repository's exclude file refuses the naming at once,
///  and names the file and what stands there.
/// Why:
///   The filling calls this before the future's first await,
/// and a plain read of a pipe with no writer never returns:
/// no ceiling could expire on the thread, so the record would never settle
/// (`SPEC.md`,
/// "Every wait has a bound").
/// The marker beside it is read through the same non-waiting reader,
/// and this control is what says the exclude file is too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(join(gitDir, "info/exclude")); expect(within(10_000, () => excludeTheStagingNames(gitDir))).toContain("is not a regular file")
/// ```
#[test]
fn a_named_pipe_at_the_exclude_file_never_holds_the_naming() {
    let root: PathBuf = fixture("prewarm-exclude-pipe");
    let git_dir: PathBuf = root.join("git-directory");
    std::fs::create_dir_all(git_dir.join("info")).expect("the Git directory's info folder");
    let exclude: PathBuf = git_dir.join("info/exclude");
    named_pipe_at(exclude.as_path());
    let covered: PathBuf = root.clone();
    let naming: Option<Result<(), String>> = crate::test_support::finished_within_at(FILLING_DEADLINE, move || return super::exclude_the_staging_names(git_dir.as_path()));
    let named: Result<(), String> = naming.expect("naming the patterns must not wait on a named pipe at the exclude file");
    let Err(reason) = named else { panic!("a named pipe at the exclude file is refused: {named:?}") };
    assert!(reason.contains("exclude") && reason.contains("not a regular file"), "the refusal names the file and what stands there: {reason}");
    remove(covered.as_path());
}

/// What:
///  A link at the exclude file is refused rather than written through,
///  and a file larger than any exclude list is refused rather than read whole.
/// Why:
///   The person's file is written where Git wrote it and nowhere a link points,
/// and a read whose size somebody else decides is the hazard the limit exists for;
/// both refusals happen before anything of this job's is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// symlink(target, exclude); expect(excludeTheStagingNames(gitDir)).toBeRefused(); expect(read(target)).toBe("")
/// ```
#[test]
fn a_link_or_an_oversized_exclude_file_is_refused_before_anything_is_written() {
    let root: PathBuf = fixture("prewarm-exclude-refused-shapes");
    let git_dir: PathBuf = root.join("git-directory");
    std::fs::create_dir_all(git_dir.join("info")).expect("the Git directory's info folder");
    let elsewhere: PathBuf = root.join("a-persons-file-elsewhere");
    std::fs::write(elsewhere.as_path(), b"# not Git's exclude file\n").expect("a file a link points at");
    let exclude: PathBuf = git_dir.join("info/exclude");
    std::os::unix::fs::symlink(elsewhere.as_path(), exclude.as_path()).expect("a link at the exclude name");
    let linked: Result<(), String> = super::exclude_the_staging_names(git_dir.as_path());
    assert!(linked.is_err(), "a link at the exclude file is refused: {linked:?}");
    assert_eq!(std::fs::read(elsewhere.as_path()).expect("the file the link points at"), b"# not Git's exclude file\n", "nothing was written through the link");
    std::fs::remove_file(exclude.as_path()).expect("the link goes");
    let oversized: Vec<u8> = vec![b'#'; (super::EXCLUDE_LIMIT as usize) + 1];
    std::fs::write(exclude.as_path(), oversized.as_slice()).expect("a file past the limit");
    let refused: Result<(), String> = super::exclude_the_staging_names(git_dir.as_path());
    let Err(reason) = refused else { panic!("a file past the limit is refused: {refused:?}") };
    assert!(reason.contains("more than"), "the refusal says the file is too large: {reason}");
    assert_eq!(std::fs::read(exclude.as_path()).expect("the oversized file"), oversized, "and nothing was appended to it");
    remove(root.as_path());
}

/// What:
///  A filling whose worktree has a named pipe where its `Cargo.lock` belongs gives that workspace up at once,
///  says what stands there,
///  and leaves the pipe untouched.
/// Why:
///   The filling reads the lock before any bounded step of the clean,
/// and a plain read of a pipe with no writer never returns:
/// no ceiling could expire on the thread that polls the runtime
/// (`SPEC.md`, "Every wait has a bound").
/// The lock is the person's file, so nothing is read of it, nothing is placed over it and nothing of it is removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(join(entry.root, 'Cargo.lock')); expect(within(10_000, () => fillOne(fill, entry))).toBeFailedNamingThePipe()
/// ```
#[test]
fn a_named_pipe_at_the_lock_never_holds_the_filling() {
    let root: PathBuf = fixture("prewarm-lock-pipe");
    let entry: PrewarmEntry = an_entry_with_a_source_build(root.as_path());
    let lock: PathBuf = entry.root.join(LOCK_FILE);
    named_pipe_at(lock.as_path());
    let covered: PathBuf = root.clone();
    let filling_root: PathBuf = root.clone();
    let covered_entry: PrewarmEntry = entry.clone();
    let filling: Option<Result<Filled, String>> = crate::test_support::finished_within_at(FILLING_DEADLINE, move || {
        let fill: Fill = Fill {
            cargo: cargo_that_succeeds(filling_root.as_path()),
            limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: FILLING_DEADLINE },
            kept_in: Some(filling_root.join("git-dir")),
            worktree: filling_root.clone(),
            filling: String::from("a-controls-filling"),
        };
        return super::fill_one_in_a_control(&fill, &entry);
    });
    let filled: Result<Filled, String> = filling.expect("the filling must not wait on a named pipe at the lock's name");
    let Err(reason) = filled else { panic!("a named pipe at the lock is refused: {filled:?}") };
    assert!(reason.contains("not a regular file"), "the answer names what stands there: {reason}");
    let standing: std::fs::Metadata = std::fs::symlink_metadata(lock.as_path()).expect("the pipe is still there");
    {
        use std::os::unix::fs::FileTypeExt as _;
        assert!(standing.file_type().is_fifo(), "the person's pipe is where it stood, untouched");
    }
    assert!(!covered_entry.to.exists(), "nothing was published");
    remove(covered.as_path());
}

/// What:
///  A staging pattern a person left a tab after,
///  or a space before,
///  is not taken for one already named:
///  the pattern is appended,
///  and Git then ignores the staging name by it.
/// Why:
///   Git keeps a pattern's leading spaces and its trailing tabs,
/// so `*.monochromatic-prewarm/\t` is not the staging pattern and ignores nothing:
/// taking it for the pattern would leave the staging copy visible in `git status`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const form of ["*.monochromatic-prewarm/\t", " *.monochromatic-prewarm/"]) { put(exclude, form + "\n"); excludeTheStagingNames(gitDir); expect(lines).toContain("*.monochromatic-prewarm/") }
/// ```
#[test]
fn a_staging_pattern_git_does_not_ignore_by_is_named_again() {
    for form in ["*.monochromatic-prewarm/\t", " *.monochromatic-prewarm/"] {
        let root: PathBuf = fixture("prewarm-exclude-trimmed-form");
        let git_dir: PathBuf = root.join("git-directory");
        std::fs::create_dir_all(git_dir.join("info")).expect("the Git directory's info folder");
        let exclude: PathBuf = git_dir.join("info/exclude");
        std::fs::write(exclude.as_path(), format!("{form}\n").as_bytes()).expect("the person's line");
        super::exclude_the_staging_names(git_dir.as_path()).expect("the patterns are named");
        // The file as it stands, byte for byte: the tabs and spaces in it are what this control is about.
        let raw: String = String::from_utf8_lossy(std::fs::read(exclude.as_path()).expect("the exclude file").as_slice()).into_owned();
        let lines: Vec<&str> = raw.split('\n').collect();
        assert!(lines.contains(&form), "{form:?}: the person's line stays as they wrote it: {raw:?}");
        for pattern in super::EXCLUDED_STAGING_PATTERNS {
            let count: usize = lines.iter().filter(|line| return **line == pattern).count();
            assert_eq!(count, 1, "{form:?}: {pattern} is a line of its own exactly once: {raw:?}");
        }
        remove(root.as_path());
    }
}

/// What:
///  A stand-in Cargo that takes a while to resolve a workspace,
///  refuses the first `--locked` clean it is asked for,
///  and writes the resolution into the workspace it was given.
/// Why:
///   The window finding 4 names needs one filling's resolution to be slow enough that a second filling reaches its own
/// resolution while the first's Cargo is still working inside the mirror,
/// and a real Cargo would make the control take minutes.
/// The refusal is what sends the filling to the resolution at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slowCargo = (root, sleepSeconds) => { const program = join(root, 'cargo-that-resolves-slowly'); write(program, `#!/bin/sh ... sleep ${sleepSeconds} ... printf '%s' "$RESOLUTION" > Cargo.lock`); return program; };
/// ```
fn cargo_that_resolves_slowly(root: &Path, sleep_seconds: &str) -> PathBuf {
    const RESOLUTION: &str = "# the resolution both fillings would make\n";
    std::fs::create_dir_all(root).expect("the stand-in's directory");
    let program: PathBuf = root.join("cargo-that-resolves-slowly");
    let refused: PathBuf = root.join("refused-once");
    let body: String = format!(
        "#!/bin/sh\nif [ \"$1\" = clean ]; then\n  if [ ! -e '{refused}' ]; then\n    : > '{refused}'\n    echo 'error: cannot update the lock file because --locked was passed to prevent this' >&2\n    exit 101\n  fi\n  exit 0\nfi\nif [ \"$1\" = metadata ]; then\n  sleep {sleep_seconds}\n  printf '%s' '{resolution}' > Cargo.lock\n  exit $?\nfi\nexit 0\n",
        refused = refused.display(),
        sleep_seconds = sleep_seconds,
        resolution = RESOLUTION,
    );
    crate::test_support::executable(program.as_path(), body.as_bytes());
    return program;
}

/// What:
///  Two fillings of one workspace at once each resolve in a mirror of their own,
///  so neither removes the folder under the other's Cargo,
///  and both complete.
/// Why:
///   A worktree removed and added again at the same path while its first record is queued gives two fillings of one
/// workspace;
/// the lease keeps them out of each other's staging names,
/// but a mirror named after the workspace alone was removed by the second filling under the first one's running Cargo,
/// so a healthy workspace's filling failed and told the person to update a lock that was fine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const a = fillOne(slowFill, entry); await sleep(200); const b = fillOne(fastFill, entry); expect(await a).toBeOk(); expect(await b).toBeOk()
/// ```
#[test]
fn two_fillings_of_one_workspace_resolve_in_mirrors_of_their_own() {
    let root: PathBuf = fixture("prewarm-two-fillings-of-one-workspace");
    let entry: PrewarmEntry = an_entry_with_a_source_build(root.as_path());
    // A lock the first `--locked` clean refuses, so each filling has to resolve the workspace.
    let lock: PathBuf = entry.root.join(LOCK_FILE);
    std::fs::write(lock.as_path(), b"# a stale committed lock\n").expect("the stale lock");
    let slow: PathBuf = cargo_that_resolves_slowly(root.as_path(), "0.6");
    let fast: PathBuf = cargo_that_resolves_slowly(root.join("fast").as_path(), "0");
    let git_dir: PathBuf = root.join("git-dir");
    let first_root: PathBuf = root.clone();
    let second_root: PathBuf = root.clone();
    let first_entry: PrewarmEntry = entry.clone();
    let second_entry: PrewarmEntry = entry.clone();
    let second_git_dir: PathBuf = git_dir.clone();
    let (first_filled, second_filled): (Result<Filled, String>, Result<Filled, String>) = crate::lazy_runtime::run_on_runtime(async move {
        let slow_fill: Fill = Fill {
            cargo: slow,
            limits: ToolLimits { window: std::time::Duration::from_secs(60), ceiling: FILLING_DEADLINE },
            kept_in: Some(git_dir),
            worktree: first_root,
            filling: String::from("the-first-filling"),
        };
        let fast_fill: Fill = Fill {
            cargo: fast,
            limits: ToolLimits { window: std::time::Duration::from_secs(60), ceiling: FILLING_DEADLINE },
            kept_in: Some(second_git_dir),
            worktree: second_root,
            filling: String::from("the-second-filling"),
        };
        let first = tokio::spawn(async move { return super::fill_one(&slow_fill, &first_entry).await });
        let second = tokio::spawn(async move {
            // Long enough for the first filling to be inside its resolution, and inside its mirror.
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            return super::fill_one(&fast_fill, &second_entry).await;
        });
        return (first.await.expect("the first filling joins"), second.await.expect("the second filling joins"));
    })
    .expect("the runtime");
    assert!(first_filled.is_ok(), "the first filling of a healthy workspace completes: {first_filled:?}");
    assert!(second_filled.is_ok(), "and so does the second: {second_filled:?}");
    assert!(lock.is_file(), "the workspace's lock stands after both fillings: {}", lock.display());
    remove(root.as_path());
}

/// What:
///  A filling that placed a resolution and then put the lock back leaves no copy of the bytes the placement displaced,
///  and the lock holds them again.
/// Why:
///   The placement keeps what its exchange displaced until the restore has put it back,
/// so a kill in between leaves a person's bytes on disk;
/// once the restore has done that, the copy is this filling's own leftover and goes,
/// or every resolution would leave a copy of the lock beside it for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const filled = fillOne(fill, entry); expect(filled).toBePublished(); expect(read(lock)).toBe(stale); expect(copiesWith(gitDir, stale).length).toBe(0)
/// ```
#[test]
fn a_filling_that_placed_a_resolution_leaves_no_copy_of_the_lock_it_displaced() {
    const RESOLUTION: &str = "# what the resolution wrote\n";
    let root: PathBuf = fixture("prewarm-placed-resolution-leaves-no-copy");
    let entry: PrewarmEntry = an_entry_with_a_source_build(root.as_path());
    let lock: PathBuf = entry.root.join(LOCK_FILE);
    let stale: &[u8] = b"# a stale committed lock\n";
    std::fs::write(lock.as_path(), stale).expect("the stale lock");
    let git_dir: PathBuf = root.join("git-dir");
    let refused: PathBuf = root.join("refused-once");
    let program: PathBuf = root.join("cargo-that-resolves");
    let body: String = format!(
        "#!/bin/sh\nif [ \"$1\" = clean ]; then\n  if [ ! -e '{refused}' ]; then\n    : > '{refused}'\n    echo 'error: cannot update the lock file because --locked was passed to prevent this' >&2\n    exit 101\n  fi\n  exit 0\nfi\nif [ \"$1\" = metadata ]; then\n  printf '%s' '{resolution}' > Cargo.lock\n  exit 0\nfi\nexit 0\n",
        refused = refused.display(),
        resolution = RESOLUTION,
    );
    crate::test_support::executable(program.as_path(), body.as_bytes());
    let fill: Fill = Fill {
        cargo: program,
        limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: FILLING_DEADLINE },
        kept_in: Some(git_dir.clone()),
        worktree: root.clone(),
        filling: String::from("the-filling-that-placed-a-resolution"),
    };
    let filled: Filled = super::fill_one_in_a_control(&fill, &entry).expect("the copy is cleaned and published");
    assert_eq!(filled, Filled::Published);
    assert_eq!(std::fs::read(lock.as_path()).ok().as_deref(), Some(stale), "the lock holds what it held before the resolution");
    // The restore keeps the resolution it displaced, which is its own record; what must not stay is the placement's copy of the
    //  bytes that stood at the lock, because the lock holds them again.
    let copies: PathBuf = git_dir.join(crate::package_repair_cargo_lock::DISPLACED_DIRECTORY);
    let mut left: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![copies];
    while let Some(directory) = pending.pop() {
        let Ok(listing) = std::fs::read_dir(directory.as_path()) else { continue };
        for found in listing.flatten() {
            let path: PathBuf = found.path();
            if path.is_dir() {
                pending.push(path);
            } else if std::fs::read(path.as_path()).ok().as_deref() == Some(stale) {
                left.push(path);
            }
        }
    }
    assert!(left.is_empty(), "no copy of the bytes the placement displaced is left after the restore: {left:?}");
    remove(root.as_path());
}

/// What:
///  A lock the filling cannot read stops that workspace before the clean,
///  and the file is left exactly as it is found:
///  both a lock past the read limit and a named pipe at the name.
/// Why:
///   "The lock could not be read" and "the checkout has no lock" are two answers:
/// taking the first for the second let a clean that rewrote the lock smaller have the restore take the tracked
/// file away as a file the clean had created,
/// and the worktree was then left with no `Cargo.lock` at all.
/// A file past the limit is a shape a committed lock can have;
/// a named pipe at the name is one nothing may wait on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const shape of ['over-the-limit', 'a-named-pipe']) { expect(fillOne(fill, entry)).toBeFailed(); expect(theLockIsUnchanged()).toBe(true) }
/// ```
#[test]
fn a_lock_the_filling_cannot_read_stops_its_clean_and_stays_as_it_is_found() {
    for shape in ["over-the-limit", "a-named-pipe"] {
        let root: PathBuf = fixture("prewarm-lock-that-cannot-be-read");
        let entry: PrewarmEntry = an_entry_with_a_source_build(root.as_path());
        let lock: PathBuf = entry.root.join(LOCK_FILE);
        let over_the_limit: Vec<u8> = vec![b'#'; (crate::package_repair_cargo_lock::LOCK_READ_LIMIT as usize) + 1];
        if shape == "over-the-limit" {
            std::fs::write(lock.as_path(), over_the_limit.as_slice()).expect("a lock past the read limit");
        } else {
            named_pipe_at(lock.as_path());
        }
        // A Cargo that is not there: the clean must never be reached, so a run of it would show as a failure of another shape.
        let fill: Fill = Fill {
            cargo: PathBuf::from(NO_CARGO),
            limits: ToolLimits { window: std::time::Duration::from_millis(200), ceiling: FILLING_DEADLINE },
            kept_in: Some(root.join("git-dir")),
            worktree: root.clone(),
            filling: String::from("a-controls-filling"),
        };
        let filled: Result<Filled, String> = super::fill_one_in_a_control(&fill, &entry);
        let Err(reason) = filled else { panic!("{shape}: the workspace gives way rather than clean a lock it cannot read: {filled:?}") };
        assert!(reason.contains(LOCK_FILE) && reason.contains("so nothing of this workspace was cleaned"), "{shape}: the answer names the lock and says nothing was cleaned: {reason}");
        if shape == "over-the-limit" {
            assert_eq!(std::fs::read(lock.as_path()).expect("the lock"), over_the_limit, "the lock past the limit is exactly as it was found");
        } else {
            use std::os::unix::fs::FileTypeExt as _;
            let standing: std::fs::Metadata = std::fs::symlink_metadata(lock.as_path()).expect("the pipe is still there");
            assert!(standing.file_type().is_fifo(), "the person's pipe is exactly as it was found");
        }
        remove(root.as_path());
    }
}

/// The variable that makes this test's child run the filling instead of asserting, and the one that tells it the fixture.
const A_KILLED_FILLINGS_CHILD: &str = "CLI_GIT_TEST_SIXTH_REVIEW_KILLED_FILLING";

/// What:
///  A filling killed between placing a resolution and putting a person's lock bytes back is finished by the next filling:
///  their bytes are back at the lock.
/// Why:
///   A kill at any instruction is ordinary on this machine, and this is the one window of the design whose kept copy nothing
/// scanned:
/// the lock held Cargo's resolution, the person's edit sat at a kept name under the Git directory,
/// and the next filling took the resolution as its baseline, so their edit left the worktree without a word.
/// The kill is real here: the child process runs the filling with a stand-in Cargo whose second clean waits, and the
/// parent kills it with `SIGKILL` while it waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const child = spawn(theTestBinary, ['--exact', name, '--nocapture'], { env: fixture }); await marker('sleeping'); child.kill('SIGKILL'); expect(lockHoldsThePersonsBytes()).toBe(true)
/// ```
#[test]
fn a_filling_killed_between_placing_a_resolution_and_restoring_is_finished_by_the_next_one() {
    // The child: the filling itself, with the stand-in Cargo and the fixture the parent made.
    if std::env::var(A_KILLED_FILLINGS_CHILD).is_ok() {
        let root: PathBuf = PathBuf::from(std::env::var("CLI_GIT_TEST_SIXTH_REVIEW_ROOT").expect("the fixture root"));
        let entry: PrewarmEntry = an_entry_with_a_source_build(root.as_path());
        let fill: Fill = Fill {
            cargo: root.join("cargo-that-waits-in-the-second-clean"),
            limits: ToolLimits { window: std::time::Duration::from_secs(30), ceiling: std::time::Duration::from_secs(30) },
            kept_in: Some(root.join("git-dir")),
            worktree: root.clone(),
            filling: String::from("the-filling-the-parent-kills"),
        };
        let _filled: Result<Filled, String> = super::fill_one_in_a_control(&fill, &entry);
        return;
    }
    let root: PathBuf = fixture("prewarm-killed-between-placement-and-restore");
    let entry: PrewarmEntry = an_entry_with_a_source_build(root.as_path());
    // The person's own lock, which no clean accepts and only a resolution makes current.
    let persons: &[u8] = b"# the person's own lock edit, made after the add returned\n";
    let lock: PathBuf = entry.root.join(LOCK_FILE);
    std::fs::write(lock.as_path(), persons).expect("the person's write");
    // The stand-in Cargo: refuses the first `--locked` clean, resolves in the mirror, and in the second clean says it is waiting
    //  and sleeps, which is the moment the parent kills the filling.
    let refused: PathBuf = root.join("refused-once");
    let sleeping: PathBuf = root.join("the-second-clean-waits");
    let resolution: &str = "# what the resolution wrote\n";
    let cargo: PathBuf = root.join("cargo-that-waits-in-the-second-clean");
    let body: String = format!(
        "#!/bin/sh\nif [ \"$1\" = clean ]; then\n  if [ ! -e '{refused}' ]; then\n    : > '{refused}'\n    echo 'error: cannot update the lock file because --locked was passed to prevent this' >&2\n    exit 101\n  fi\n  : > '{sleeping}'\n  sleep 30\n  exit 0\nfi\nif [ \"$1\" = metadata ]; then\n  printf '%s' '{resolution}' > Cargo.lock\n  exit 0\nfi\nexit 0\n",
        refused = refused.display(),
        sleeping = sleeping.display(),
        resolution = resolution,
    );
    crate::test_support::executable(cargo.as_path(), body.as_bytes());
    let child: std::process::Child = std::process::Command::new(crate::test_support::this_test_binary().expect("the test binary"))
        .args(["--exact", "repair::package_repair_prewarm_job::tests::a_filling_killed_between_placing_a_resolution_and_restoring_is_finished_by_the_next_one", "--nocapture", "--test-threads=1"])
        .env(A_KILLED_FILLINGS_CHILD, "1")
        .env("CLI_GIT_TEST_SIXTH_REVIEW_ROOT", root.to_str().expect("the fixture root is text"))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("the child filling starts");
    // Wait, bounded, for the second clean to be the step that is running: the placement has happened and the restore has not.
    let began: std::time::Instant = std::time::Instant::now();
    while !sleeping.is_file() {
        assert!(began.elapsed() < std::time::Duration::from_secs(60), "the child never reached its second clean");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let mut child: std::process::Child = child;
    let _killed = child.kill();
    let _waited = child.wait();
    // The state the kill leaves: the lock holds the resolution, and the person's bytes are at a kept name with a note beside it.
    assert_eq!(std::fs::read(lock.as_path()).expect("the lock"), resolution.as_bytes(), "the kill left the resolution at the lock");
    assert!(!copies_with_the_lock_module(root.join("git-dir").as_path(), persons).is_empty(), "and the person's bytes are kept under the Git directory");
    // The next filling, with a Cargo that accepts the resolution: without the recovery it takes the resolution as its baseline and
    //  publishes, and the person's edit never comes back.
    let quiet: PathBuf = root.join("cargo-that-accepts-the-lock");
    crate::test_support::executable(quiet.as_path(), b"#!/bin/sh\nexit 0\n");
    let fill: Fill = Fill {
        cargo: quiet,
        limits: ToolLimits { window: std::time::Duration::from_secs(5), ceiling: FILLING_DEADLINE },
        kept_in: Some(root.join("git-dir")),
        worktree: root.clone(),
        filling: String::from("the-filling-that-looks-next"),
    };
    let next: Result<Filled, String> = super::fill_one_in_a_control(&fill, &entry);
    assert!(next.is_ok(), "the next filling completes: {next:?}");
    assert_eq!(
        std::fs::read(lock.as_path()).expect("the lock"),
        persons,
        "the person's bytes are back at the lock after the next filling; the kept copies are {:?}",
        copies_with_the_lock_module(root.join("git-dir").as_path(), persons)
    );
    remove(root.as_path());
}

/// What:
///  Every file under one lock's displaced directory whose bytes are `bytes`.
/// Why:
///   A control has to see whether a person's bytes are kept, without depending on the name the step chose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copiesWithTheLockModule = (gitDir, bytes) => walk(join(gitDir, DISPLACED_DIRECTORY)).filter((path) => readFileSync(path).equals(bytes))
/// ```
fn copies_with_the_lock_module(git_dir: &Path, bytes: &[u8]) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![git_dir.join(crate::package_repair_cargo_lock::DISPLACED_DIRECTORY)];
    while let Some(directory) = pending.pop() {
        let Ok(listing) = std::fs::read_dir(directory.as_path()) else { continue };
        for found_entry in listing.flatten() {
            let path: PathBuf = found_entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if std::fs::read(path.as_path()).ok().as_deref() == Some(bytes) {
                found.push(path);
            }
        }
    }
    found.sort();
    return found;
}
