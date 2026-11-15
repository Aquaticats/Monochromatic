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
    super::write_marker(super::marker_of(staged.as_path()).as_path(), entry.to.as_path(), super::WhatTheMarkerRecords::TheStaging).expect("the marker an earlier run left");
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
    super::write_marker(super::marker_of(staged.as_path()).as_path(), entry.to.as_path(), super::WhatTheMarkerRecords::TheStaging).expect("the marker the killed filling wrote first");
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
    super::write_marker(marker.as_path(), &root.join("another-worktree/worktree-build"), super::WhatTheMarkerRecords::TheStaging).expect("the marker the source worktree's killed add left");
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
///  A copy that carried another worktree's staged state as well as its build state is refused,
///  with the state left exactly as it was found:
///  nothing at the staging name is removed for this worktree's own state,
///  and the add says what to run instead.
/// Why:
///   The marker that came with the copy names another worktree's build directory,
///  so it proves nothing about the names beside this one,
///  and a directory at the staging name is somebody's unless this job's own marker for this very
///  build directory records making it.
///  Removing it would be removing bytes by where they stand rather than by proof,
///  so the names are left, the add refuses, and the person's own `cargo clean` is what completes
///  the repair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeMarker(carriedForAnotherWorktree); mkdirSync(staging); expect(stageTheWorktreesOwnState(destination)).toBeErr(); expect(staging/debug/copied).toExist()
/// ```
#[test]
fn a_staged_copy_a_copy_carried_is_left_and_the_add_says_what_to_run() {
    let root: PathBuf = fixture("prewarm-carried-staging");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.to.join("debug")).expect("the copied build state");
    std::fs::write(entry.to.join("debug/built"), b"the newer state of the same tree\n").expect("a file of the copy");
    let staging: PathBuf = staging_of(entry.to.as_path());
    std::fs::create_dir_all(staging.join("debug")).expect("the staging copy the source worktree's add made");
    std::fs::write(staging.join("debug/copied"), b"the older state of the same tree\n").expect("a file of the carried staging copy");
    let marker: PathBuf = super::marker_of(staging.as_path());
    super::write_marker(marker.as_path(), &root.join("another-worktree/worktree-build"), super::WhatTheMarkerRecords::TheStaging).expect("the source worktree's marker, carried here by the copy");
    let git_directory: PathBuf = root.join("git-directory");
    let staged: Result<Staged, String> = super::stage_the_worktrees_own_state(entry.to.as_path(), Some(git_directory.as_path()));
    assert!(staged.is_err(), "the add refuses a name it did not make: {staged:?}");
    assert!(staging.join("debug/copied").is_file(), "the carried staging copy is left exactly as it was found");
    assert!(entry.to.join("debug/built").is_file(), "and this worktree's own state stays where it is for the person's clean");
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
    super::write_marker(marker.as_path(), entry.to.as_path(), super::WhatTheMarkerRecords::TheStaging).expect("the marker this worktree's killed add wrote before its rename");
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
    let patterns: Vec<String> = super::excluded_staging_patterns();
    assert_eq!(named.matches(patterns[0].as_str()).count(), 1, "the pattern is named once: {named}");
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
///  A stand-in Cargo that succeeds at everything it is asked,
///  which is the clean of a workspace whose state needs nothing resolved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cargoThatSucceeds = (root) => executable(join(root, 'cargo-that-succeeds'), '#!/bin/sh\nexit 0\n')
/// ```
fn cargo_that_succeeds(root: &Path) -> PathBuf {
    let program: PathBuf = root.join("cargo-that-succeeds");
    crate::test_support::executable(program.as_path(), b"#!/bin/sh\nexit 0\n");
    return program;
}

/// The deadline one control gives a filling,
/// so a filling that waits on something it must not wait on fails here instead of holding the suite.
const FILLING_DEADLINE: std::time::Duration = std::time::Duration::from_secs(10);

/// What:
///  Make a named pipe at `path`.
/// Why:
///  A named pipe is the one foreign object whose plain open waits for a writer that may never come,
/// which is what a read at a name anyone can write must never do.
fn named_pipe_at(path: &Path) {
    use std::os::unix::ffi::OsStrExt as _;
    let named: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("a fixture path holds no NUL byte");
    // SAFETY: the string is NUL-terminated and outlives the call, and the name is a fresh path in the fixture.
    let made: libc::c_int = unsafe { libc::mkfifo(named.as_ptr(), 0o600) };
    assert_eq!(made, 0, "make a named pipe at {}: {}", path.display(), std::io::Error::last_os_error());
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
    super::write_marker(marker.as_path(), entry.to.as_path(), super::WhatTheMarkerRecords::TheStaging).expect("the live filling's marker");
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
    super::write_marker(marker.as_path(), entry.to.as_path(), super::WhatTheMarkerRecords::TheStaging).expect("the killed filling's marker");
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
    assert!(crate::package_repair_prewarm_claim::lease_of(usual.as_path()).is_file(), "the lease file stands, as the next holder needs it to: nothing unlinks it while it is held");
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
    super::write_marker(marker.as_path(), entry.to.as_path(), super::WhatTheMarkerRecords::TheStaging).expect("the live filling's marker");
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
    let patterns: Vec<String> = super::excluded_staging_patterns();
    let subsets: usize = 1 << patterns.len();
    for subset in 0..subsets {
        let mut present: Vec<&str> = Vec::new();
        for (index, pattern) in patterns.iter().enumerate() {
            if subset & (1 << index) != 0 {
                present.push(pattern.as_str());
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
        for pattern in &patterns {
            let count: usize = lines.iter().filter(|line| return **line == pattern.as_str()).count();
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
    let patterns: Vec<String> = super::excluded_staging_patterns();
    for last_line in [patterns[2].as_str(), "# a person's cut-short line"] {
        let root: PathBuf = fixture("prewarm-exclude-no-line-end");
        let git_dir: PathBuf = root.join("git-directory");
        std::fs::create_dir_all(git_dir.join("info")).expect("the Git directory's info folder");
        let exclude: PathBuf = git_dir.join("info/exclude");
        std::fs::write(exclude.as_path(), last_line.as_bytes()).expect("a file whose last line has no line end");
        super::exclude_the_staging_names(git_dir.as_path()).expect("the patterns are named");
        let lines: Vec<String> = lines_in(exclude.as_path());
        assert_eq!(lines[0], last_line, "the cut-short line is a line of its own afterwards: {lines:?}");
        for pattern in &patterns {
            let count: usize = lines.iter().filter(|line| return **line == pattern.as_str()).count();
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
        let patterns: Vec<String> = super::excluded_staging_patterns();
        for pattern in &patterns {
            let count: usize = lines.iter().filter(|line| return **line == pattern.as_str()).count();
            assert_eq!(count, 1, "{form:?}: {pattern} is a line of its own exactly once: {raw:?}");
        }
        remove(root.as_path());
    }
}


/// What:
///  A filling whose workspace has a stale lock cleans it through a mirror of the workspace,
///  and the person's `Cargo.lock` is exactly as they left it:
///  the same bytes,
///  the same inode,
///  the same file time,
///  and no copy of it anywhere under the Git directory.
/// Why:
///   This is where every lock finding of seven review rounds lived:
/// the filling used to place a resolution in that file and put the person's bytes back,
/// and every kill or concurrent write in between lost or mislabelled them.
/// The design removes the steps:
/// Cargo resolves and cleans in a mirror of the workspace
/// (`package_repair_cargo_clean_in_mirror.rs`), so the person's file is read and never written.
/// The real Cargo here builds the copy down to the member's own artifacts,
/// so the clean has something to delete and this control shows it did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeStaleLock(); const filled = fillOne(fill, entry); expect(filled).toBePublished(); expect(readFileSync(lock)).toEqual(persons); expect(inode).toBe(before); expect(displacedCopies()).toBe(0)
/// ```
#[test]
fn a_filling_with_a_stale_lock_leaves_the_persons_lock_exactly_as_it_is() {
    let root: PathBuf = fixture("prewarm-stale-lock-is-never-written");
    let entry: PrewarmEntry = built_workspace(root.as_path());
    let lock: PathBuf = entry.root.join(LOCK_FILE);
    // A lock the `--locked` clean refuses: the resolution has to happen, and it happens in the mirror.
    let persons: Vec<u8> = b"# the person's own lock, made after the add returned\n".to_vec();
    std::fs::write(lock.as_path(), persons.as_slice()).expect("the person's lock");
    let before: std::fs::Metadata = std::fs::metadata(lock.as_path()).expect("the lock");
    let git_dir: PathBuf = root.join("git-dir");
    std::fs::create_dir_all(git_dir.as_path()).expect("the Git directory");
    let item: WorkItem = item_for(root.as_path(), entry.root.as_path(), &entry);
    let verdict: Verdict = crate::lazy_runtime::run_on_runtime(super::prewarm_awaiting(&item)).expect("the filling runs");
    assert_eq!(verdict, Verdict::Done, "the filling completes");
    assert!(entry.to.exists(), "the cleaned copy reached the worktree's build directory name");
    assert!(entries_named(entry.to.as_path(), format!("{MEMBER}-").as_str()).is_empty(), "the member's own artifacts are gone from the copy: {:?}", entries_named(entry.to.as_path(), format!("{MEMBER}-").as_str()));
    let after: std::fs::Metadata = std::fs::metadata(lock.as_path()).expect("the lock is still there");
    assert_eq!(std::fs::read(lock.as_path()).expect("the person's lock"), persons, "the person's lock keeps its bytes");
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        assert_eq!(after.ino(), before.ino(), "the file is the same file: nothing renamed or exchanged it");
        assert_eq!(after.mtime(), before.mtime(), "and nothing wrote it");
    }
    assert!(kept_lock_copies(git_dir.as_path()).is_empty(), "no copy of the lock is left under the Git directory: {:?}", kept_lock_copies(git_dir.as_path()));
    remove(root.as_path());
}

/// What:
///  Every byte any step kept under one Git directory in the tree of displaced locks,
///  which the design no longer writes at all.
/// Why:
///   A control has to see whether anything of the person's was moved aside,
///  and that tree is the only place the repair ever kept such a copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keptLockCopies = (gitDir) => walk(join(gitDir, 'monochromatic-git-policy-cli-displaced-locks')).map(readFile)
/// ```
fn kept_lock_copies(git_dir: &Path) -> Vec<Vec<u8>> {
    let mut found: Vec<Vec<u8>> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![git_dir.join("monochromatic-git-policy-cli-displaced-locks")];
    while let Some(directory) = pending.pop() {
        let Ok(listing) = std::fs::read_dir(directory.as_path()) else { continue };
        for entry in listing.flatten() {
            let path: PathBuf = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if let Ok(bytes) = std::fs::read(path.as_path()) {
                found.push(bytes);
            }
        }
    }
    return found;
}

/// What:
///  A lease a filling gives back is not taken twice:
///  a second filling that opened the lease file before the give-back and locks it after holds the names,
///  and a third filling that claims the name is refused.
/// Why:
///   Unlinking the lease file while its lock is still held let exactly that happen:
/// the taker that opened the name before the unlink locked the inode the name no longer pointed at,
/// a taker after it made a fresh file and locked that one,
/// and both believed they held one build directory's names,
/// so one removed the other's staging copy while its Cargo ran.
/// The file stays where it is, so the name and the inode are one for every taker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = claimTheNames(staging); const opened = open(leaseOf(staging)); giveBack(names); expect(flock(opened, EX|NB)).toBe(0); expect(claimTheNames(staging).kind).toBe('taken')
/// ```
#[test]
fn a_lease_a_filling_gives_back_is_not_taken_twice() {
    let root: PathBuf = fixture("prewarm-claim-given-back-once");
    std::fs::create_dir_all(root.as_path()).expect("the fixture");
    let staging: PathBuf = root.join("worktree-build.monochromatic-prewarm");
    let lease: TheLease = match super::claim_the_names(staging.as_path()) {
        TheClaim::Held(lease) => lease,
        other => panic!("the first filling takes the lease: {other:?}"),
    };
    // A second filling opens the lease file while the first still holds it, and locks it only after the give-back.
    let opened: std::fs::File = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(crate::package_repair_prewarm_claim::lease_of(staging.as_path()))
        .expect("the lease file is there to open");
    super::give_back(&super::TheNames { staging: staging.clone(), target: super::target_of(staging.as_path()), marker: None, lease: Some(lease) });
    use std::os::unix::io::AsRawFd as _;
    // SAFETY: the descriptor belongs to `opened`, which outlives the call.
    let locked: libc::c_int = unsafe { libc::flock(opened.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    assert_eq!(locked, 0, "the filling that opened the name before the give-back holds the lease: {}", std::io::Error::last_os_error());
    let third: TheClaim = super::claim_the_names(staging.as_path());
    assert!(matches!(third, TheClaim::Taken), "and a filling that claims the name now is refused, so two fillings never hold it: {third:?}");
    remove(root.as_path());
}

/// What:
///  A pattern another command appends between this job's reading of the exclude file and its own write
///  keeps its own line,
///  and every staging pattern is a line of its own.
/// Why:
///   The reading decides which patterns are missing,
/// and whether the batch needs a leading line end,
/// and both decisions are stale by the time of the write:
/// an unterminated pattern appended in the window would carry the first staging pattern onto its own line,
/// so that pattern would not exist and the staging name would be ignored by nothing.
/// The batch begins with a line end whatever was read, and it is one write,
/// so the other writer's line ends where it ended and every staging pattern stands alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hook(() => appendFileSync(exclude, 'important-file')); expect(excludeTheStagingNames(gitDir)).toBeOk(); expect(everyPatternIsItsOwnLine()).toBe(true)
/// ```
#[test]
fn a_pattern_another_writer_appends_between_the_read_and_the_write_keeps_both_lines_apart() {
    let root: PathBuf = fixture("prewarm-exclude-append-race");
    let git_dir: PathBuf = root.join("git-directory");
    std::fs::create_dir_all(git_dir.join("info")).expect("the Git directory's info folder");
    let exclude: PathBuf = git_dir.join("info/exclude");
    std::fs::write(exclude.as_path(), b"node_modules/\n").expect("a person's own line");
    let planted: PathBuf = exclude.clone();
    super::AFTER_THE_EXCLUDE_IS_READ.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            // Another command's pattern, with no line end after it, exactly as a person's editor may leave it.
            let mut appended: Vec<u8> = std::fs::read(planted.as_path()).expect("the exclude file");
            appended.extend_from_slice(b"important-file");
            std::fs::write(planted.as_path(), appended.as_slice()).expect("the other command's append");
        }));
    });
    super::exclude_the_staging_names(git_dir.as_path()).expect("the patterns are named");
    super::AFTER_THE_EXCLUDE_IS_READ.with(|hook| { *hook.borrow_mut() = None; });
    let raw: String = String::from_utf8_lossy(std::fs::read(exclude.as_path()).expect("the exclude file").as_slice()).into_owned();
    let lines: Vec<&str> = raw.split('\n').collect();
    assert!(lines.contains(&"important-file"), "the other command's pattern is a line of its own: {raw:?}");
    for pattern in super::excluded_staging_patterns() {
        let count: usize = lines.iter().filter(|line| return **line == pattern.as_str()).count();
        assert_eq!(count, 1, "{pattern} is a line of its own exactly once: {raw:?}");
    }
    assert!(!raw.contains("important-file*.monochromatic-prewarm/"), "nothing joins the other command's pattern and the first staging pattern: {raw:?}");
    remove(root.as_path());
}

/// What:
///  A directory that stood at the target name before the filling claimed it keeps every file in it,
///  through the filling's clean and the giving back of the names,
///  and the clean is given a directory this run made instead.
/// Why:
///   The names this job may remove are the names this job made,
/// and a directory at the target name was not made by any operation of this job's:
/// the filling reads the add's marker, hands the target name to Cargo as its target directory,
/// and the giving back of the names then removes it whole.
/// A marker that records making the target is the proof a later run removes one by, and the add's
/// marker records the staging alone, so a directory that stood there before the claim is left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stage(destination); writeFileSync(join(targetOf(staging), 'whoever.txt'), 'their build output'); fillOne(fillOfNowhere(), entryOf(staged)); expect(existsSync(join(targetOf(staging), 'whoever.txt'))).toBe(true)
/// ```
#[test]
fn a_target_directory_that_stood_there_before_the_claim_is_left_whole() {
    let root: PathBuf = fixture("prewarm-foreign-target");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.to.as_path()).expect("the build directory");
    std::fs::write(entry.to.join("artifact"), b"a build's artifact").expect("the artifact");
    let staging: PathBuf = staging_of(entry.to.as_path());
    let staged: Staged = super::stage_the_worktrees_own_state(entry.to.as_path(), None).expect("the add stages its state");
    assert_eq!(staged, Staged::Staged(staging.clone()), "the state moves aside under this job's marker");
    let target: PathBuf = super::target_of(staging.as_path());
    std::fs::create_dir_all(target.as_path()).expect("a directory that appeared at the target name after the add staged");
    let theirs: PathBuf = target.join("whoever.txt");
    std::fs::write(theirs.as_path(), b"another command's build output").expect("a file of theirs");
    let staged_entry: PrewarmEntry = PrewarmEntry { staged_in: Some(staging.clone()), from: None, ..entry.clone() };
    let filled: Result<Filled, String> = super::fill_one_in_a_control(&fill_of_nowhere(), &staged_entry);
    assert!(filled.is_err(), "a clean that could not run is reported: {filled:?}");
    assert!(theirs.exists(), "the directory that stood at the target name keeps its files: {}", theirs.display());
    remove(root.as_path());
}

/// What:
///  The add refuses a target name something this job did not make already holds,
///  and leaves it and the build directory whole.
/// Why:
///   The add stages the copied state under three names, and a directory at the target name is
///  neither the staging name nor the marker:
///  a guard that looks at those two alone stages the state with a name beside it that it will then
///  hand to a clean and remove, so the person's directory goes without a word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdirSync(targetOf(stagingOf(destination))); writeFileSync(join(targetOf(stagingOf(destination)), 'whoever.txt'), 'x'); expect(stageTheWorktreesOwnState(destination)).toBeErr()
/// ```
#[test]
fn the_add_refuses_a_target_name_it_did_not_make() {
    let root: PathBuf = fixture("prewarm-foreign-target-at-the-add");
    let entry: PrewarmEntry = entry_of(root.as_path());
    std::fs::create_dir_all(entry.to.as_path()).expect("the build directory");
    std::fs::write(entry.to.join("artifact"), b"a build's artifact").expect("the artifact");
    let target: PathBuf = super::target_of(staging_of(entry.to.as_path()).as_path());
    std::fs::create_dir_all(target.as_path()).expect("a directory at the target name");
    let theirs: PathBuf = target.join("whoever.txt");
    std::fs::write(theirs.as_path(), b"another command's build output").expect("a file of theirs");
    let staged: Result<Staged, String> = super::stage_the_worktrees_own_state(entry.to.as_path(), None);
    assert!(staged.is_err(), "the add refuses a name it did not make: {staged:?}");
    assert!(theirs.exists(), "the directory at the target name keeps its files: {}", theirs.display());
    assert!(entry.to.join("artifact").exists(), "and the build directory is where it was");
    remove(root.as_path());
}

/// What:
///  A symbolic link at the staging name that points at nothing is still that link after the add stages its state.
/// Why:
///   A link whose target does not exist answers `exists` with false,
/// and a rename replaces the name it lands on,
/// so a guard that asks only whether the name exists takes the person's link for nothing there and unlinks it.
/// A name is not proof that a thing is this job's:
/// the staging name must be free of any entry at all,
/// and the move aside must fail rather than replace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// symlinkSync(join(root, 'nothing-here'), staging); const staged = stageTheWorktreesOwnState(destination, undefined); expect(staged).toBeErr(); expect(lstatSync(staging).isSymbolicLink()).toBe(true)
/// ```
#[test]
fn a_link_to_nothing_at_the_staging_name_is_never_replaced_by_the_move_aside() {
    let root: PathBuf = fixture("prewarm-staging-link");
    let destination: PathBuf = root.join("worktree-build");
    std::fs::create_dir_all(destination.as_path()).expect("the build directory");
    std::fs::write(destination.join("artifact"), b"a build's artifact").expect("the artifact");
    let staging: PathBuf = staging_of(destination.as_path());
    std::os::unix::fs::symlink(root.join("nothing-here"), staging.as_path()).expect("the person's link");
    let staged: Result<Staged, String> = super::stage_the_worktrees_own_state(destination.as_path(), None);
    eprintln!("THE ANSWER OF THE MOVE ASIDE: {staged:?}");
    assert!(staged.is_err(), "a name somebody else's link holds is refused rather than replaced: {staged:?}");
    let standing: std::fs::Metadata = std::fs::symlink_metadata(staging.as_path()).expect("the name is still there");
    assert!(standing.file_type().is_symlink(), "the person's link is still the link it was: {:?}", standing.file_type());
    assert!(destination.join("artifact").exists(), "and the build directory is where it was");
    remove(root.as_path());
}

/// What:
///  Every name the makers of a staging name can produce is ignored by real Git,
///  the names of a run's own included,
///  and the person's `git add --all` stages none of them.
/// Why:
///   The names stand inside the worktree,
/// so Git is what decides whether a commit sees them,
/// and a pattern written for the usual name alone left the names a second filling takes for itself
/// visible while that filling was at work:
/// the person's next `git add --all` then staged a partial copy of the build state.
/// The patterns are read from the functions that make the names,
/// so a form one of them can produce is a form this control asks Git about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const form of everyFormOf(destination)) expect(checkIgnore(form)).toBe(true); git('add', '--all'); expect(staged()).toEqual([])
/// ```
#[test]
fn every_form_the_name_makers_make_is_ignored_by_real_git() {
    let root: PathBuf = fixture("prewarm-patterns-real-git");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let destination: PathBuf = repo.join("target");
    std::fs::create_dir_all(destination.as_path()).expect("the build directory");
    let mut exclude: String = super::excluded_staging_patterns().join("\n");
    exclude.push('\n');
    std::fs::write(repo.join(".git/info/exclude"), exclude.as_bytes()).expect("the repository's exclude file");
    std::fs::write(repo.join("tracked.txt"), b"the person's file\n").expect("a file of theirs");
    crate::test_support::git(repo.as_path(), &["add", "--", "tracked.txt"]);
    crate::test_support::git(repo.as_path(), &["commit", "--quiet", "--message=the-person's-file"]);
    // Every form beside this build directory, made by the functions that make them: the directories hold an artifact and the
    //  marker and the lease are files.
    let usual: PathBuf = super::staging_of(destination.as_path());
    let own: PathBuf = super::a_name_of_this_runs_own(usual.as_path());
    let mut forms: Vec<PathBuf> = Vec::new();
    for staging in [usual, own] {
        for (named, directory) in [
            (staging.clone(), true),
            (super::target_of(staging.as_path()), true),
            (super::marker_of(staging.as_path()), false),
            (super::lease_of(staging.as_path()), false),
        ] {
            let artifact: PathBuf = if directory { named.join("artifact") } else { named };
            std::fs::create_dir_all(artifact.parent().expect("a parent")).expect("the directories of the name");
            std::fs::write(artifact.as_path(), b"a partial copy of the build state\n").expect("the file of the name");
            forms.push(artifact);
        }
    }
    for form in &forms {
        let relative: &std::ffi::OsStr = form.strip_prefix(repo.as_path()).expect("inside the repository").as_os_str();
        let checked: std::process::Output = crate::test_support::git_output(repo.as_path(), &[
            std::ffi::OsStr::new("check-ignore"),
            std::ffi::OsStr::new("--no-index"),
            std::ffi::OsStr::new("--quiet"),
            std::ffi::OsStr::new("--"),
            relative,
        ]);
        assert!(checked.status.success(), "Git ignores {}: {:?}", form.display(), String::from_utf8_lossy(checked.stderr.as_slice()));
    }
    // A positive control for the control: a file that is nobody's staging name is not ignored.
    assert!(
        !crate::test_support::git_output(repo.as_path(), &["check-ignore", "--no-index", "--quiet", "--", "tracked.txt"]).status.success(),
        "a file of the person's is not ignored, so the patterns are what ignore the staging names"
    );
    crate::test_support::git(repo.as_path(), &["add", "--all"]);
    let staged: String = crate::test_support::git_text(repo.as_path(), &["diff", "--cached", "--name-only"]);
    assert!(staged.trim().is_empty(), "the person's `git add --all` stages nothing of this job's names: {staged:?}");
    remove(root.as_path());
}

/// What:
///  The words for a resolution that a mirror could not make name the mirror and never send a person to
///  update their lock.
/// Why:
///   A dependency whose path leaves the worktree is not carried into a mirror of it,
/// and the person's lock is read and never written,
/// so a lock is never that cause:
/// advising an update would send them to change the one file that cannot be the reason,
/// and the change would not help.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = theWordsForAMirrorThatGaveWay('it exited with 101: failed to load manifest for dependency `shared`', entry); expect(words).toContain('leaves the worktree'); expect(words).not.toContain('cargo update')
/// ```
#[test]
fn the_words_for_a_mirror_that_gave_way_name_the_mirror_and_never_the_lock() {
    let root: PathBuf = fixture("prewarm-mirror-words");
    let entry: PrewarmEntry = entry_of(root.as_path());
    let reason: &str = "it exited with 101: failed to load manifest for dependency `shared`";
    let words: String = super::the_words_for_a_mirror_that_gave_way(reason, &entry);
    assert!(words.contains(reason), "the tool's own words are kept: {words}");
    assert!(words.contains("leaves the worktree"), "and the cause a mirror has is named: {words}");
    assert!(!words.contains("cargo update"), "a lock is never what stopped it, so no advice sends a person to one: {words}");
    assert!(words.contains("removed"), "and what this job did with the copy is said: {words}");
    // The contrast: a reason that is the lock's own keeps the lock's advice.
    let the_locks_own: String = format!("cannot update {LOCK_FILE} as --locked requires");
    assert!(super::what_to_do_next(&entry, the_locks_own.as_str()).contains("cargo update"), "a lock's own refusal keeps its advice");
    remove(root.as_path());
}
