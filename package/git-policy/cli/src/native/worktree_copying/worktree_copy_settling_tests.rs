//! Controls for settling a staged copy against its source:
//! an entry the source removed while the copy ran,
//! an entry that changed once,
//! and an entry that a neighbouring copy keeps rewriting.

use super::*;
use crate::test_support::{fixture, remove};
use crate::worktree_copy_ceiling::CopyCeiling;
use crate::worktree_copy_manifest::{LeftOut, collect_pruned_manifest};
use crate::worktree_copy_stage_leaf_copying::{CopiedManifest, copy_manifest_entries_with, try_clone};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};

/// The mode of the clone request the control supplies:
/// nothing is changed under the copy.
const UNCHANGED: usize = 0;

/// A source file goes away between its listing and its copy.
const VANISHES: usize = 1;

/// A source file is rewritten once,
/// by the copy of its neighbour,
/// after it was copied itself.
const CHANGES_ONCE: usize = 2;

/// Each source file is rewritten by the copy of the other one,
/// so neither ever holds still between two comparisons.
const KEEPS_CHANGING: usize = 3;

/// A source directory goes away between its listing and its copy,
/// which takes the file it held with it.
const DIRECTORY_VANISHES: usize = 4;

/// A source directory's permission bits change under the copy.
const DIRECTORY_MODE_CHANGES: usize = 5;

/// A source file is replaced by a named pipe,
/// by the copy of its neighbour,
/// between its own listing and its own copy.
const A_PIPE_REPLACES_A_FILE: usize = 6;

/// One control at a time,
/// because the clone request a copy takes is a plain function and reads these.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// The source tree the clone request rewrites,
/// while one control runs.
static SOURCE_ROOT: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Which of the three changes the clone request makes.
static MODE: AtomicUsize = AtomicUsize::new(UNCHANGED);

/// How many entries the copy has reached.
static COPIES: AtomicUsize = AtomicUsize::new(0);

/// Whether the control that changes one file once has already done it.
static CHANGED: AtomicBool = AtomicBool::new(false);

/// A source with `cache/a.txt` and `cache/b.txt`,
/// a stage root beside it,
/// and the manifest of the two,
/// under the lock that keeps one control at a time.
fn two_ignored_files(label: &str, mode: usize) -> (MutexGuard<'static, ()>, PathBuf, PathBuf, PathBuf, Vec<WorktreeCopyEntry>) {
    let held: MutexGuard<'static, ()> = ONE_AT_A_TIME.lock().expect("no other control holds the clone request");
    let root: PathBuf = fixture(label);
    let source: PathBuf = root.join("source");
    std::fs::create_dir_all(source.join("cache")).expect("the ignored cache");
    std::fs::write(source.join("cache/a.txt"), b"the first one\n").expect("a");
    std::fs::write(source.join("cache/b.txt"), b"the second one\n").expect("b");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(&stage).expect("the stage");
    let mut left_out: LeftOut = LeftOut::default();
    let entries: Vec<WorktreeCopyEntry> =
        collect_pruned_manifest(source.as_path(), [b"cache".to_vec()].as_slice(), &[], Prune::Never, &mut left_out, &CopyCeiling::unbounded()).expect("the manifest");
    assert!(left_out.unreadable.is_empty(), "{:?}", left_out.unreadable);
    assert!(left_out.uncopiable.is_empty(), "{:?}", left_out.uncopiable);
    *SOURCE_ROOT.lock().expect("the source") = Some(source.clone());
    MODE.store(mode, Ordering::SeqCst);
    COPIES.store(0, Ordering::SeqCst);
    CHANGED.store(false, Ordering::SeqCst);
    return (held, root, source, stage, entries);
}

/// Put a named pipe where a file was,
/// so the entry the walk listed holds no bytes for the copy to read.
/// Why:
///   A pipe is what a dev server and a `mkfifo` leave in an ignored cache,
/// and opening one for reading waits
///  for a writer that may never come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// execFileSync('mkfifo', [path]);
/// ```
#[cfg(unix)]
fn make_pipe(path: &Path) {
    let status: std::process::ExitStatus = std::process::Command::new("mkfifo")
        .arg(path)
        .status()
        .expect("mkfifo");
    assert!(status.success(), "mkfifo must create {}", path.display());
}

/// The device and inode of a path,
/// to recognise the entry a copy is reading.
fn identity_of(path: &Path) -> Option<(u64, u64)> {
    return match std::fs::metadata(path) {
        Ok(found) => Some((std::os::unix::fs::MetadataExt::dev(&found), std::os::unix::fs::MetadataExt::ino(&found))),
        Err(_absent) => None,
    };
}

/// What:
///  The clone request the control supplies:
/// it makes the change of the control's mode and then asks the
///  filesystem for a clone,
/// so a control never depends on whether this filesystem clones.
/// Why:
///   The change has to land at a stated moment of the copy,
/// which is what the request's own call is;
/// the answer
///  stays the real one,
/// so the copy takes the byte path here whenever it would in use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hook(input, output) { const mode = MODE; const copies = COPIES++; const root = SOURCE_ROOT; if (mode === VANISHES && copies === 0) rmSync(join(root, 'cache/b.txt')); if (mode === CHANGES_ONCE && copies === 0) writeFileSync(join(root, 'cache/a.txt'), 'rewritten once'); if (mode === KEEPS_CHANGING) { if (reading(input) === identity('cache/a.txt')) writeFileSync(join(root, 'cache/b.txt'), `b ${copies}`); else writeFileSync(join(root, 'cache/a.txt'), `a ${copies}`); } return tryClone(input, output); }
/// ```
fn changing_clone(input: &std::fs::File, output: &std::fs::File) -> bool {
    let mode: usize = MODE.load(Ordering::SeqCst);
    let copies: usize = COPIES.fetch_add(1, Ordering::SeqCst);
    let Some(root) = SOURCE_ROOT.lock().expect("the source").clone() else { return try_clone(input, output) };
    let a: PathBuf = root.join("cache/a.txt");
    let b: PathBuf = root.join("cache/b.txt");
    let reading = match input.metadata() {
        Ok(found) => Some((std::os::unix::fs::MetadataExt::dev(&found), std::os::unix::fs::MetadataExt::ino(&found))),
        Err(_absent) => None,
    };
    // `match` on the mode keeps each control's change to the one it states.
    match mode {
        VANISHES if copies == 0 => {
            std::fs::remove_file(&b).expect("b goes away");
        }
        DIRECTORY_VANISHES if copies == 0 => {
            std::fs::remove_dir_all(root.join("cache/deep")).expect("the source's directory goes away");
        }
        DIRECTORY_MODE_CHANGES if copies == 0 => {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(root.join("cache/deep"), std::fs::Permissions::from_mode(0o750)).expect("the directory's mode changes");
        }
        A_PIPE_REPLACES_A_FILE if copies == 0 => {
            std::fs::remove_file(&b).expect("the file goes away");
            make_pipe(&b);
        }
        CHANGES_ONCE if reading.is_some() && reading == identity_of(b.as_path()) && !CHANGED.swap(true, Ordering::SeqCst) => {
            std::fs::write(&a, b"rewritten once\n").expect("a is rewritten by the copy of its neighbour");
        }
        KEEPS_CHANGING => {
            if reading.is_some() && reading == identity_of(a.as_path()) {
                std::fs::write(&b, format!("b {copies}\n").as_bytes()).expect("b is rewritten");
            } else if reading.is_some() && reading == identity_of(b.as_path()) {
                std::fs::write(&a, format!("a {copies}\n").as_bytes()).expect("a is rewritten");
            }
        }
        _ => {}
    }
    return try_clone(input, output);
}

/// What:
///  Copy the manifest and settle it,
/// with the clone request of the control's mode.
/// Why:
///   One shape for the three controls,
/// so each of them reads as its own change and its own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copied = copyManifestEntriesWith({ source, stage, entries, clone: changingClone }); return settleTheCopy({ ...copied, source, stage, entries, prune: 'never', clone: changingClone });
/// ```
fn copy_and_settle(source: &Path, stage: &Path, entries: Vec<WorktreeCopyEntry>) -> Result<Settled, WorktreeCopyError> {
    let copied: CopiedManifest = copy_manifest_entries_with(source, stage, entries.as_slice(), changing_clone, &CopyCeiling::unbounded())?;
    return settle_the_copy(SettlingRequest {
        source_root: source,
        stage_root: stage,
        selected_roots: [b"cache".to_vec()].as_slice(),
        excluded_source_roots: &[],
        prune: Prune::Never,
        entries,
        stamps: copied.stamps,
        clone: changing_clone,
        gone: copied.gone,
    });
}

/// Review round 5,
/// the owner's second live failure:
/// a file a build removed between the listing and its copy has nothing left to copy,
/// so the stage simply
///  does not hold it and the add goes on.
/// Why:
///   The previous code compared the manifest by position,
/// so one vanished entry made every later entry
///  differ and the add failed with "ignored source paths changed while snapshot was staged";
/// the person then
///  held a worktree with no ignored state and a branch they did not ask for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { entries } = await copyAndSettle(...VANISHES); expect(entries.map((e) => e.relativePath)).toEqual(['cache', 'cache/a.txt']); expect(existsSync(join(source, 'cache/b.txt'))).toBe(false);
/// ```
#[test]
fn a_file_the_source_removed_while_the_copy_ran_is_left_out_and_not_a_failure() {
    let (held, root, source, stage, entries) = two_ignored_files("settling-vanished", VANISHES);
    let settled: Settled = copy_and_settle(source.as_path(), stage.as_path(), entries).expect("a vanished entry is no failure");
    let staged: Vec<Vec<u8>> = settled.entries.iter().map(|entry: &WorktreeCopyEntry| { return entry.relative_path.clone(); }).collect();
    assert_eq!(staged, vec![b"cache".to_vec(), b"cache/a.txt".to_vec()], "the entry the source removed is not in the stage");
    assert!(!stage.join("cache/b.txt").exists(), "and nothing of it was staged");
    assert!(settled.unsettled.is_empty(), "nothing was left in the source for changing: {:?}", settled.unsettled);
    assert!(source.join("cache/a.txt").is_file(), "the source keeps its own files");
    drop(held);
    remove(root.as_path());
}

/// What:
///  A file that changed under the copy is copied again,
/// and the stage then holds the bytes the source has.
/// Why:
///   A build rewrites a stamp file while its own copy runs,
/// and the copy is seconds long;
/// one more copy of that
///  one entry is what makes the stage match the source it is installed from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { entries, unsettled } = await copyAndSettle(...CHANGES_ONCE); expect(unsettled).toEqual([]); expect(readFileSync(join(stage, 'cache/a.txt'), 'utf8')).toBe('rewritten once\n');
/// ```
#[test]
fn a_file_that_changed_under_the_copy_is_copied_again() {
    let (held, root, source, stage, entries) = two_ignored_files("settling-changed-once", CHANGES_ONCE);
    let settled: Settled = copy_and_settle(source.as_path(), stage.as_path(), entries).expect("a changed entry is copied again");
    assert!(settled.unsettled.is_empty(), "the entry settled at its second copy: {:?}", settled.unsettled);
    assert_eq!(settled.entries.len(), 3, "every entry of the manifest is staged: {:?}", settled.entries);
    assert_eq!(std::fs::read(stage.join("cache/a.txt")).expect("a staged a"), std::fs::read(source.join("cache/a.txt")).expect("a in the source"), "the stage holds the source's bytes");
    drop(held);
    remove(root.as_path());
}

/// What:
///  A file that a neighbouring copy keeps rewriting is copied again a bounded number of times,
/// then left in its
///  source and named,
/// while its neighbour is staged.
/// Why:
///   Nothing may fail an add for a cache file a build keeps writing,
/// and nothing may delete the
///  person's file to make the comparison succeed;
/// the stage does not hold the entry,
/// the source keeps it
///  untouched,
/// and the person is told which path it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { entries, unsettled } = await copyAndSettle(...KEEPS_CHANGING); expect(unsettled).toEqual(['cache/a.txt']); expect(existsSync(join(stage, 'cache/a.txt'))).toBe(false); expect(readFileSync(join(source, 'cache/a.txt'), 'utf8')).toMatch(/^a \d+$/);
/// ```
#[test]
fn a_file_that_keeps_changing_is_left_in_its_source_and_named() {
    let (held, root, source, stage, entries) = two_ignored_files("settling-keeps-changing", KEEPS_CHANGING);
    let settled: Settled = copy_and_settle(source.as_path(), stage.as_path(), entries).expect("an entry that will not settle is no failure");
    assert_eq!(settled.unsettled, vec![b"cache/a.txt".to_vec()], "the entry its neighbour kept rewriting is named");
    assert!(!stage.join("cache/a.txt").exists(), "it is not in the stage");
    assert!(source.join("cache/a.txt").is_file(), "its only copy stays in the source, untouched");
    assert_eq!(std::fs::read(stage.join("cache/b.txt")).expect("b staged"), std::fs::read(source.join("cache/b.txt")).expect("b in the source"), "the entry that settled is staged with the source's bytes");
    drop(held);
    remove(root.as_path());
}

/// What:
///  A source with `cache/a.txt` and the directory `cache/deep` holding `cache/deep/file.txt`,
/// a stage root beside it,
/// and the manifest of those four entries,
/// under the lock that keeps one control at a time.
/// Why:
///   The directory controls change what happens to an entry that is not a leaf,
/// which is the shape the file controls never reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { source, stage, entries } = directorySource('settling-directory', VANISH);
/// ```
fn source_with_a_directory(
    label: &str,
    mode: usize,
) -> (MutexGuard<'static, ()>, PathBuf, PathBuf, PathBuf, Vec<WorktreeCopyEntry>) {
    let held: MutexGuard<'static, ()> = ONE_AT_A_TIME.lock().expect("no other control holds the clone request");
    let root: PathBuf = fixture(label);
    let source: PathBuf = root.join("source");
    std::fs::create_dir_all(source.join("cache/deep")).expect("the ignored cache");
    std::fs::write(source.join("cache/a.txt"), b"the first one\n").expect("a");
    std::fs::write(source.join("cache/deep/file.txt"), b"the deep one\n").expect("deep file");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(&stage).expect("the stage");
    let mut left_out: LeftOut = LeftOut::default();
    let entries: Vec<WorktreeCopyEntry> =
        collect_pruned_manifest(source.as_path(), [b"cache".to_vec()].as_slice(), &[], Prune::Never, &mut left_out, &CopyCeiling::unbounded()).expect("the manifest");
    assert!(left_out.unreadable.is_empty(), "{:?}", left_out.unreadable);
    *SOURCE_ROOT.lock().expect("the source") = Some(source.clone());
    MODE.store(mode, Ordering::SeqCst);
    COPIES.store(0, Ordering::SeqCst);
    CHANGED.store(false, Ordering::SeqCst);
    return (held, root, source, stage, entries);
}

/// The permission bits of a staged or source entry.
fn mode_of(path: &Path) -> u32 {
    use std::os::unix::fs::MetadataExt;
    return std::fs::symlink_metadata(path).expect("the entry").mode() & 0o7777;
}

/// What:
///  A directory the source removed while the copy ran is left out with the entries below it,
/// and the add goes
///  on.
/// Why:
///   A build removes a whole stamp directory,
/// not one file at a time,
/// and the staged tree holds
///  that directory's children,
/// so dropping the directory has to take them with it instead of failing
///  the copy with "Is a directory".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { entries } = await copyAndSettle(...DIRECTORY_VANISHES); expect(entries.map((e) => e.relativePath)).toEqual(['cache', 'cache/a.txt']); expect(existsSync(join(stage, 'cache/deep'))).toBe(false);
/// ```
#[test]
fn a_directory_the_source_removed_while_the_copy_ran_is_left_out_and_not_a_failure() {
    let (held, root, source, stage, entries) = source_with_a_directory("settling-directory-vanished", DIRECTORY_VANISHES);
    let settled: Settled = copy_and_settle(source.as_path(), stage.as_path(), entries).expect("a vanished directory is no failure");
    let staged: Vec<Vec<u8>> = settled.entries.iter().map(|entry: &WorktreeCopyEntry| { return entry.relative_path.clone(); }).collect();
    assert_eq!(staged, vec![b"cache".to_vec(), b"cache/a.txt".to_vec()], "the directory the source removed and its file are not in the stage");
    assert!(!stage.join("cache/deep").exists(), "and nothing of them was staged");
    drop(held);
    remove(root.as_path());
}

/// What:
///  A directory whose permission bits changed under the copy is staged with the new bits,
/// and the add goes on.
/// Why:
///   A directory's only mutable property is its mode,
/// and its content belongs to the entries below it,
/// so a
///  changed mode is applied instead of removing the staged tree and copying nothing into its place
///  ("Is a directory").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { entries, unsettled } = await copyAndSettle(...DIRECTORY_MODE_CHANGES); expect(unsettled).toEqual([]); expect(modeOf(join(stage, 'cache/deep'))).toBe(0o750);
/// ```
#[cfg(unix)]
#[test]
fn a_directory_whose_mode_changed_under_the_copy_is_staged_with_the_new_mode() {
    let (held, root, source, stage, entries) = source_with_a_directory("settling-directory-mode", DIRECTORY_MODE_CHANGES);
    let settled: Settled = copy_and_settle(source.as_path(), stage.as_path(), entries).expect("a changed directory mode is no failure");
    assert!(settled.unsettled.is_empty(), "the entry settled at its second pass: {:?}", settled.unsettled);
    assert_eq!(settled.entries.len(), 4, "every entry of the manifest is staged: {:?}", settled.entries);
    assert_eq!(mode_of(stage.join("cache/deep").as_path()), mode_of(source.join("cache/deep").as_path()), "the stage holds the source's permission bits");
    assert_eq!(mode_of(stage.join("cache/deep").as_path()), 0o750);
    assert!(stage.join("cache/deep/file.txt").is_file(), "the directory's content is still staged");
    drop(held);
    remove(root.as_path());
}

/// What:
///  A named pipe that replaces a cache file between its listing and its own copy is left out,
/// and the copy answers
///  instead of waiting for a writer that never comes.
/// Why:
///   A dev server leaves a socket or a pipe in an ignored cache while an add copies the same directory,
/// and
///  opening a pipe for reading waits until some process writes to it,
/// which nothing promises;
/// a command
///  that waits there holds the checkout and every other command that wants it,
/// with nothing said and no
///  end ("nothing should ever wait forever or indefinitely").
/// The handle the copy opens is where the entry's type is
///  settled,
/// because the walk and the copy are two passes over a directory anything may write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copied = await finishedWithinAt(PATIENCE, () => copyManifestEntriesWith({ source, stage, entries, clone: changingClone })); expect(copied.gone).toEqual(['cache/b.txt']); expect(existsSync(join(stage, 'cache/b.txt'))).toBe(false);
/// ```
#[cfg(unix)]
#[test]
fn a_pipe_that_replaces_a_file_before_its_copy_is_left_out_without_waiting() {
    let (held, root, source, stage, entries) = two_ignored_files("settling-pipe-replaces", A_PIPE_REPLACES_A_FILE);
    let wanted: (PathBuf, PathBuf, Vec<WorktreeCopyEntry>) = (source.clone(), stage.clone(), entries.clone());
    let answered: Option<Result<CopiedManifest, WorktreeCopyError>> = crate::parallel_test_support::finished_within_at(
        crate::parallel_test_support::PATIENCE,
        move || {
            return copy_manifest_entries_with(wanted.0.as_path(), wanted.1.as_path(), wanted.2.as_slice(), changing_clone, &CopyCeiling::unbounded());
        },
    );
    let Some(staged) = answered else {
        panic!("the copy of cache/b.txt waited for a writer of the named pipe that replaced it and never came; opening a source entry for reading must not wait");
    };
    let copied: CopiedManifest = staged.expect("a pipe where a file was listed is no failure of the copy");
    assert_eq!(copied.gone, vec![b"cache/b.txt".to_vec()], "the entry the pipe replaced is named as gone");
    assert!(!stage.join("cache/b.txt").exists(), "and nothing of it was staged");
    assert_eq!(std::fs::read(stage.join("cache/a.txt")).expect("a is staged"), b"the first one\n", "its neighbour is copied as before");
    drop(held);
    remove(root.as_path());
}

/// What:
///  A named pipe that replaced a source file before its copy leaves the settled copy,
/// and the neighbours
///  of the entry are installed as they were.
/// Why:
///   A socket,
/// a named pipe and a device node cannot be copied into a worktree,
/// so the walk leaves
///  each of them out and nothing fails over one (SPEC,
/// "Linked-worktree ignored-state synchronization");
/// a name
///  that became one between the listing and its copy is the same entry,
/// so the pass that settles the stage
///  drops it by its path rather than failing an add whose worktree Git already made.
/// The settling answers instead of waiting for a writer that never comes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settled = await finishedWithinAt(PATIENCE, () => copyAndSettle(...A_PIPE_REPLACES_A_FILE)); expect(settled.entries.map((e) => e.relativePath)).toEqual(['cache/a.txt']); expect(existsSync(join(stage, 'cache/b.txt'))).toBe(false);
/// ```
#[cfg(unix)]
#[test]
fn settling_leaves_out_a_pipe_that_replaced_a_file_and_keeps_its_neighbours() {
    let (held, root, source, stage, entries) = two_ignored_files("settling-pipe-named", A_PIPE_REPLACES_A_FILE);
    let wanted: (PathBuf, PathBuf, Vec<WorktreeCopyEntry>) = (source.clone(), stage.clone(), entries.clone());
    let answered: Option<Result<Settled, WorktreeCopyError>> = crate::parallel_test_support::finished_within_at(
        crate::parallel_test_support::PATIENCE,
        move || {
            return copy_and_settle(wanted.0.as_path(), wanted.1.as_path(), wanted.2);
        },
    );
    let Some(outcome) = answered else {
        panic!("the staging of the copy waited for a writer of the named pipe that replaced cache/b.txt and never came");
    };
    let settled: Settled = outcome.expect("an entry no worktree can hold is left out of the settled copy, never fatal");
    let kept: Vec<&[u8]> = settled
        .entries
        .iter()
        .map(|entry| return entry.relative_path.as_slice())
        .collect();
    assert_eq!(
        kept,
        vec![b"cache".as_slice(), b"cache/a.txt".as_slice()],
        "the entry the pipe replaced is gone, and the directory that held it and its neighbour stay"
    );
    assert!(settled.unsettled.is_empty(), "the pipe is no entry that would not settle: {}", settled.unsettled.len());
    assert_eq!(settled.uncopiable, vec![b"cache/b.txt".to_vec()], "the entry the pipe replaced is named as one no worktree can hold");
    assert_eq!(std::fs::read(stage.join("cache/a.txt")).expect("a is staged"), b"the first one\n", "its neighbour was staged before the pass answered");
    assert!(!stage.join("cache/b.txt").exists(), "nothing of the entry that became a pipe is installed");
    drop(held);
    remove(root.as_path());
}

/// A named pipe that replaced a source file after the copy ran is named by the comparison's own pass,
/// so the entry a worktree cannot hold is said even when no copy ever met it.
/// Why:
///  the comparison's walks leave a named pipe out of the source's manifest,
/// so the entry answers as gone,
/// and only the look at the name tells the two answers apart.
#[cfg(unix)]
#[test]
fn a_pipe_that_replaces_a_file_after_the_copy_is_named_by_the_settling_pass() {
    let (held, root, source, stage, entries) = two_ignored_files("settling-pipe-after-copy", UNCHANGED);
    let copied: CopiedManifest = copy_manifest_entries_with(source.as_path(), stage.as_path(), entries.as_slice(), changing_clone, &CopyCeiling::unbounded()).expect("both entries are copied");
    assert!(copied.gone.is_empty(), "both files were there for their copy");
    std::fs::remove_file(source.join("cache/b.txt")).expect("the file goes away");
    make_pipe(source.join("cache/b.txt").as_path());
    let settled: Settled = settle_the_copy(SettlingRequest {
        source_root: source.as_path(),
        stage_root: stage.as_path(),
        selected_roots: [b"cache".to_vec()].as_slice(),
        excluded_source_roots: &[],
        prune: Prune::Never,
        entries,
        stamps: copied.stamps,
        clone: changing_clone,
        gone: copied.gone,
    })
    .expect("a pipe that replaced a file is no failure of the settling");
    assert_eq!(settled.uncopiable, vec![b"cache/b.txt".to_vec()], "the entry the pipe replaced is named as one no worktree can hold");
    assert!(!stage.join("cache/b.txt").exists(), "nothing of the entry that became a pipe is installed");
    drop(held);
    remove(root.as_path());
}
