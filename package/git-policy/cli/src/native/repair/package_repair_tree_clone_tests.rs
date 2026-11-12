//! The tree clone:
//! contents,
//! modes,
//! times,
//! links kept verbatim,
//! an existing destination refused,
//! parallel and serial agree.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::PathBuf;
use std::time::Duration;

/// More files than any single worker slice,
/// so the parallel path really splits the work.
const MANY_FILES: usize = 64;

/// A source with nested directories,
/// an executable,
/// a link with a relative target,
/// and an old modification time.
fn build_source(root: &Path) -> PathBuf {
    let source: PathBuf = root.join("source");
    std::fs::create_dir_all(source.join("deps/nested")).expect("directories");
    std::fs::write(source.join("deps/nested/lib.rlib"), b"library bytes").expect("file");
    std::fs::write(source.join("tool"), b"binary bytes").expect("tool");
    std::fs::set_permissions(source.join("tool"), std::fs::Permissions::from_mode(0o750)).expect("mode");
    symlink("../outside/target", source.join("deps/link")).expect("link");
    for index in 0..MANY_FILES {
        std::fs::write(source.join(format!("deps/file-{index}")), format!("{index}")).expect("many");
    }
    let old: std::time::SystemTime = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000_000);
    let file: std::fs::File = std::fs::File::options().write(true).open(source.join("deps/nested/lib.rlib")).expect("open for times");
    file.set_modified(old).expect("set modified");
    return source;
}

/// Contents,
/// modes and modification times are the source's;
/// a link keeps its exact target text.
#[test]
fn clones_contents_modes_times_and_links() {
    let root: PathBuf = fixture("tree-clone-faithful");
    let source: PathBuf = build_source(&root);
    let destination: PathBuf = root.join("destination");
    let made: usize = clone_tree(&source, &destination, 1).expect("clone").made;
    assert!(made > MANY_FILES, "made {made}");
    assert_eq!(std::fs::read(destination.join("deps/nested/lib.rlib")).expect("lib"), b"library bytes");
    let tool = std::fs::metadata(destination.join("tool")).expect("tool metadata");
    assert_eq!(tool.permissions().mode() & 0o777, 0o750);
    let before = std::fs::metadata(source.join("deps/nested/lib.rlib")).expect("source");
    let after = std::fs::metadata(destination.join("deps/nested/lib.rlib")).expect("destination");
    assert_eq!((after.mtime(), after.mtime_nsec()), (before.mtime(), before.mtime_nsec()));
    assert_eq!(std::fs::read_link(destination.join("deps/link")).expect("link"), PathBuf::from("../outside/target"));
    remove(&root);
}

/// The clone shares no hard link with the source:
/// changing the clone leaves the source alone.
#[test]
fn a_changed_clone_leaves_the_source_alone() {
    let root: PathBuf = fixture("tree-clone-independent");
    let source: PathBuf = build_source(&root);
    let destination: PathBuf = root.join("destination");
    clone_tree(&source, &destination, 1).expect("clone");
    std::fs::write(destination.join("tool"), b"changed in the clone").expect("write clone");
    assert_eq!(std::fs::read(source.join("tool")).expect("source tool"), b"binary bytes");
    remove(&root);
}

/// An existing destination is refused and left as it was.
#[test]
fn refuses_an_existing_destination() {
    let root: PathBuf = fixture("tree-clone-existing");
    let source: PathBuf = build_source(&root);
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir_all(&destination).expect("destination");
    std::fs::write(destination.join("keep"), b"kept").expect("keep");
    assert!(clone_tree(&source, &destination, 1).is_err());
    assert_eq!(std::fs::read(destination.join("keep")).expect("kept"), b"kept");
    assert!(!destination.join("tool").exists());
    remove(&root);
}

/// Several workers make the same tree as one worker.
#[test]
fn parallel_and_serial_clones_agree() {
    let root: PathBuf = fixture("tree-clone-parallel");
    let source: PathBuf = build_source(&root);
    let serial: PathBuf = root.join("serial");
    let parallel: PathBuf = root.join("parallel");
    let one: usize = clone_tree(&source, &serial, 1).expect("serial").made;
    let several: usize = clone_tree(&source, &parallel, 4).expect("parallel").made;
    assert_eq!(one, several);
    for index in 0..MANY_FILES {
        let name: String = format!("deps/file-{index}");
        assert_eq!(std::fs::read(parallel.join(&name)).expect("parallel file"), format!("{index}").into_bytes());
    }
    remove(&root);
}

/// A missing source is an error naming it.
#[test]
fn a_missing_source_is_an_error() {
    let root: PathBuf = fixture("tree-clone-missing");
    let error: String = clone_tree(root.join("absent").as_path(), root.join("destination").as_path(), 1).expect_err("missing source");
    assert!(error.contains("absent"), "{error}");
    remove(&root);
}

/// Every entry of a tree as its path below `root`,
/// its mode and,
/// for a readable file,
/// its bytes;
/// the source must come out of a
///  clone exactly as it went in.
fn describe(root: &Path) -> Vec<(PathBuf, u32, Option<Vec<u8>>)> {
    let mut found: Vec<(PathBuf, u32, Option<Vec<u8>>)> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(listing) = std::fs::read_dir(&directory) else { continue };
        for item in listing.flatten() {
            let metadata = std::fs::symlink_metadata(item.path()).expect("metadata");
            if metadata.is_dir() {
                pending.push(item.path());
            }
            found.push((item.path().strip_prefix(root).expect("below").to_path_buf(), metadata.permissions().mode(), if metadata.is_file() { std::fs::read(item.path()).ok() } else { None }));
        }
    }
    found.sort();
    return found;
}

/// A file a test left with no read permission stops nothing:
/// every other file is cloned,
/// the unreadable one is named as left
///  out and is not in the destination,
/// and the source is exactly as it was.
#[test]
fn an_unreadable_file_is_left_out_and_everything_else_is_cloned() {
    let root: PathBuf = fixture("tree-clone-unreadable-file");
    let source: PathBuf = build_source(&root);
    std::fs::create_dir_all(source.join("scratch/deep")).expect("scratch");
    std::fs::write(source.join("scratch/deep/cmdline"), b"private").expect("unreadable file");
    std::fs::set_permissions(source.join("scratch/deep/cmdline"), std::fs::Permissions::from_mode(0o000)).expect("no access");
    std::fs::write(source.join("scratch/deep/after"), b"readable").expect("neighbour");
    let before = describe(&source);
    let destination: PathBuf = root.join("destination");
    let cloned: Cloned = clone_tree(&source, &destination, 1).expect("the clone finishes past the unreadable file");
    assert_eq!(cloned.left_out, vec![PathBuf::from("scratch/deep/cmdline")]);
    assert!(!destination.join("scratch/deep/cmdline").exists(), "no placeholder and no partial file");
    assert_eq!(std::fs::read(destination.join("scratch/deep/after")).expect("neighbour"), b"readable");
    assert_eq!(std::fs::read(destination.join("deps/nested/lib.rlib")).expect("library"), b"library bytes");
    assert_eq!(describe(&source), before, "the source is untouched");
    std::fs::set_permissions(source.join("scratch/deep/cmdline"), std::fs::Permissions::from_mode(0o600)).expect("give access back");
    remove(&root);
}

/// A directory with no read permission is made empty with its own mode and named as left out;
/// its siblings are cloned.
#[test]
fn an_unreadable_directory_is_made_empty_and_left_out() {
    let root: PathBuf = fixture("tree-clone-unreadable-directory");
    let source: PathBuf = build_source(&root);
    std::fs::create_dir_all(source.join("scratch/locked")).expect("locked");
    std::fs::write(source.join("scratch/locked/inside"), b"hidden").expect("inside");
    std::fs::set_permissions(source.join("scratch/locked"), std::fs::Permissions::from_mode(0o000)).expect("no access");
    let destination: PathBuf = root.join("destination");
    let cloned: Cloned = clone_tree(&source, &destination, 1).expect("the clone finishes past the unreadable directory");
    assert_eq!(cloned.left_out, vec![PathBuf::from("scratch/locked")]);
    let made = std::fs::metadata(destination.join("scratch/locked")).expect("the directory exists");
    assert_eq!(made.permissions().mode() & 0o777, 0o000);
    assert_eq!(std::fs::read(destination.join("tool")).expect("sibling"), b"binary bytes");
    std::fs::set_permissions(source.join("scratch/locked"), std::fs::Permissions::from_mode(0o700)).expect("give access back");
    std::fs::set_permissions(destination.join("scratch/locked"), std::fs::Permissions::from_mode(0o700)).expect("give access back");
    remove(&root);
}

/// A tree with nothing unreadable leaves nothing out (the control that the new field is not always filled).
#[test]
fn a_readable_tree_leaves_nothing_out() {
    let root: PathBuf = fixture("tree-clone-readable");
    let source: PathBuf = build_source(&root);
    assert_eq!(clone_tree(&source, &root.join("destination"), 1).expect("clone").left_out, Vec::<PathBuf>::new());
    remove(&root);
}

/// A clone goes parallel from the measured break-even number of leaves and not before.
#[test]
fn the_clone_threshold_is_the_measured_break_even() {
    assert_eq!(PARALLEL_MIN_ENTRIES, crate::parallel_budget::PARALLEL_BREAK_EVEN_ENTRIES);
}

/// Where the system starts no thread the clone is made by the calling thread and equals the serial one.
/// The body runs in a child that
/// is refused every thread.
#[test]
fn a_parallel_clone_where_no_thread_can_start_is_made_in_turn() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "a_parallel_clone_where_no_thread_can_start_is_made_in_turn") {
        return;
    }
    parallel_and_serial_clones_agree();
}

/// How long the control holds the places slow work may take,
/// several times the moment the clone is given to answer in.
const HELD_SLOW: Duration = Duration::from_millis(500);

/// How long a clone that takes the places slow work may take is given to answer while no slow place is free:
/// a moment,
/// which a clone of a few dozen files finishes in many times over when it is given a place at once.
const A_MOMENT: Duration = Duration::from_millis(100);

/// What:
///  The clone of a build directory takes the places slow work may take,
///  so one place of the process's bound stays free of it for the steps that serve a person.
/// Why:
///   The clone is the heaviest step of a new worktree's prewarm
/// (111 s of one add's 160 s repair phase,
/// `doc/planning/cli-git-worktree-add-package-repair.md`),
/// and the prewarm is slow work nobody waits for.
/// Prewarm waves run four wide and the service's pool has four places,
/// so a wave that took general places held every one of them
/// and the round's own storage steps waited for the whole clone:
/// a person's pending push was claimed only when the wave ended.
/// The slow kinds and the snapshot ahead already ask for a slow place;
/// the clone is the one step of them that did not.
#[test]
fn a_clone_takes_the_places_slow_work_may_take_and_leaves_one_free_for_a_person() {
    crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to(2);
    assert_eq!(crate::blocking_jobs_within_the_budget::budgeted_blocking_jobs(), 2, "this control is the first to ask for the process's places, so the bound is the two it states");
    let root: PathBuf = fixture("tree-clone-slow-places");
    let source: PathBuf = build_source(&root);
    let destination: PathBuf = root.join("destination");
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime of this thread");
    runtime.block_on(async move {
        let (holding, held) = tokio::sync::oneshot::channel::<()>();
        let holder = tokio::spawn(async move {
            let _outcome = crate::blocking_jobs_within_the_budget::run_a_slow_job_within_the_budget(move || {
                let _told = holding.send(());
                std::thread::sleep(HELD_SLOW);
            })
            .await;
        });
        held.await.expect("the job that holds the place slow work may take started");
        let mut cloning = tokio::spawn(async move { return clone_tree_awaiting(source.as_path(), destination.as_path(), 1).await; });
        // The only place slow work may take is held, so a clone that takes slow places cannot answer yet.
        assert!(
            tokio::time::timeout(A_MOMENT, &mut cloning).await.is_err(),
            "the clone waits for a place slow work may take instead of taking one the steps that serve a person need"
        );
        // The place kept free of slow work is still free while the clone waits.
        assert!(
            matches!(tokio::time::timeout(A_MOMENT, crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget(|| ())).await, Ok(Ok(()))),
            "a place of the process's bound stayed free of the clone"
        );
        let cloned: Cloned = cloning.await.expect("the clone task ends").expect("the clone is made");
        assert!(cloned.made > MANY_FILES, "made {}", cloned.made);
        let _joined = holder.await;
    });
    remove(&root);
}
