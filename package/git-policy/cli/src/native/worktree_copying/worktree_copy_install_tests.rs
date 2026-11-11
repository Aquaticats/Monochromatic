//! Installation checks on real filesystem trees:
//! preflight,
//! batching,
//! modes and rollback on failure.

use super::*;
use crate::test_support::{fixture, remove};
use crate::worktree_copy_ceiling::CopyCeiling;
use crate::worktree_copy_manifest::collect_entry_manifest;

/// A recorder that counts its calls and can fail one call on demand.
#[derive(Debug, Default)]
struct CountingRecorder {
    /// Sizes of each intent batch.
    intents: Vec<usize>,
    /// Sizes of each creation batch.
    creations: Vec<usize>,
    /// Creation call number that fails,
    /// counted from one.
    fail_creation_call: Option<usize>,
    /// Creation call number that first rewrites a path in place,
    /// then fails.
    tamper: Option<(usize, PathBuf)>,
}

impl InstallRecorder for CountingRecorder {
    /// Counts one intent batch.
    fn record_intents(&mut self, relative_paths: &[Vec<u8>]) -> Result<(), WorktreeCopyError> {
        self.intents.push(relative_paths.len());
        return Ok(());
    }

    /// Counts one creation batch and fails the chosen call.
    fn record_creations(&mut self, entries: &[InstalledPath]) -> Result<(), WorktreeCopyError> {
        self.creations.push(entries.len());
        if let Some((_, path)) = self
            .tamper
            .as_ref()
            .filter(|(call, _)| return *call == self.creations.len())
        {
            {
                // Replacing the file gives it a new identity, as an editor's save or a foreign write would.
                std::fs::remove_file(path).expect("detach the hard link");
                std::fs::write(path, b"tampered").expect("tamper");
                return Err(WorktreeCopyError(String::from(
                    "recorder refused the batch",
                )));
            }
        }
        if self.fail_creation_call == Some(self.creations.len()) {
            return Err(WorktreeCopyError(String::from(
                "recorder refused the batch",
            )));
        }
        return Ok(());
    }
}

/// A stage directory holding one tree,
/// with its manifest.
fn stage_tree(root: &Path) -> (PathBuf, Vec<WorktreeCopyEntry>) {
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(stage.join("cache/deep")).expect("stage tree");
    std::fs::write(stage.join("cache/deep/data"), b"payload").expect("data");
    std::fs::write(stage.join("cache/top"), b"top").expect("top");
    let entries: Vec<WorktreeCopyEntry> =
        collect_entry_manifest(stage.as_path(), &[b"cache".to_vec()], &[]).expect("manifest");
    return (stage, entries);
}

/// Every existing entry must be identical;
/// absent entries are reported in manifest order.
#[test]
fn preflight_accepts_identical_entries_and_lists_absent_ones() {
    let root: PathBuf = fixture("install-preflight");
    let (stage, entries) = stage_tree(root.as_path());
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir_all(destination.join("cache")).expect("cache");
    std::fs::write(destination.join("cache/top"), b"top").expect("identical");
    let absent: BTreeSet<Vec<u8>> = preflight_destination(
        stage.as_path(),
        destination.as_path(),
        &entries,
        &BTreeSet::new(),
    )
    .expect("preflight");
    assert_eq!(
        absent,
        BTreeSet::from([b"cache/deep".to_vec(), b"cache/deep/data".to_vec()])
    );
    remove(root.as_path());
}

/// A differing entry stops the installation,
/// names the entry and the destination,
/// and is never overwritten.
#[test]
fn preflight_refuses_differing_entries_without_overwriting() {
    let root: PathBuf = fixture("install-preflight-differs");
    let (stage, entries) = stage_tree(root.as_path());
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir_all(destination.join("cache")).expect("cache");
    std::fs::write(destination.join("cache/top"), b"other").expect("differs");
    let failure: WorktreeCopyError = preflight_destination(
        stage.as_path(),
        destination.as_path(),
        &entries,
        &BTreeSet::new(),
    )
    .expect_err("differs");
    assert_eq!(
        failure.0,
        format!(
            "monochromatic-git-policy-cli: ignored-state copy would overwrite differing destination entry \"cache/top\" in {}.",
            quote(destination.to_string_lossy().as_ref())
        )
    );
    assert_eq!(
        std::fs::read(destination.join("cache/top")).expect("kept"),
        b"other"
    );
    remove(root.as_path());
}

/// A directory this transaction claimed is accepted whatever its contents,
/// so an interrupted run can resume.
#[test]
fn claimed_directory_is_accepted_for_resumption() {
    let root: PathBuf = fixture("install-preflight-claimed");
    let (stage, entries) = stage_tree(root.as_path());
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir_all(destination.join("cache/deep")).expect("claimed");
    let claimed: BTreeSet<Vec<u8>> = BTreeSet::from([b"cache".to_vec()]);
    let absent: BTreeSet<Vec<u8>> =
        preflight_destination(stage.as_path(), destination.as_path(), &entries, &claimed)
            .expect("claimed");
    assert_eq!(
        absent,
        BTreeSet::from([b"cache/deep/data".to_vec(), b"cache/top".to_vec()])
    );
    remove(root.as_path());
}

/// A complete installation copies content,
/// links and modes,
/// and reports the selected entries it created.
#[cfg(unix)]
#[test]
fn installation_copies_every_entry_and_applies_modes() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("install-happy");
    let (stage, _entries) = stage_tree(root.as_path());
    std::os::unix::fs::symlink("deep/data", stage.join("cache/link")).expect("link");
    std::fs::set_permissions(
        stage.join("cache/top"),
        std::fs::Permissions::from_mode(0o751),
    )
    .expect("mode");
    let entries: Vec<WorktreeCopyEntry> =
        collect_entry_manifest(stage.as_path(), &[b"cache".to_vec()], &[]).expect("manifest");
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir(destination.as_path()).expect("destination");
    let mut recorder: CountingRecorder = CountingRecorder::default();
    let copied: usize = install_snapshot(
        stage.as_path(),
        destination.as_path(),
        &entries,
        &BTreeSet::new(),
        &[],
        &mut recorder,
        &CopyCeiling::unbounded(),
    )
    .expect("installed");
    assert_eq!(copied, entries.len());
    assert_eq!(
        std::fs::read(destination.join("cache/deep/data")).expect("data"),
        b"payload"
    );
    assert_eq!(
        std::fs::read_link(destination.join("cache/link")).expect("link"),
        PathBuf::from("deep/data")
    );
    assert_eq!(
        std::fs::metadata(destination.join("cache/top"))
            .expect("top")
            .permissions()
            .mode()
            & 0o777,
        0o751
    );
    remove(root.as_path());
}

/// Full batches of files that the batching control installs,
/// plus one file more so the last batch is a short one.
const FULL_BATCHES_OF_FILES: usize = 2;

/// The install's slices are jobs of the process's blocking pool:
/// a slice asks for the one place of the process's bound before its files are created,
/// which a slice on a thread of its own never does.
#[test]
fn the_install_slices_take_places_of_the_processs_bound() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::RecvTimeoutError;
    let (given_back, wanted): (std::sync::Arc<AtomicBool>, std::sync::mpsc::Receiver<bool>) = crate::parallel_test_support::hold_the_only_place();
    let root: PathBuf = fixture("install-the-bound");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(stage.join("cache")).expect("stage tree");
    for index in 0..crate::parallel_budget::PARALLEL_BREAK_EVEN_ENTRIES + 40 {
        std::fs::write(stage.join(format!("cache/file-{index:05}")), b"payload").expect("file");
    }
    let entries: Vec<WorktreeCopyEntry> = collect_entry_manifest(stage.as_path(), &[b"cache".to_vec()], &[]).expect("manifest");
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir(destination.as_path()).expect("destination");
    let mut recorder: CountingRecorder = CountingRecorder::default();
    install_snapshot(stage.as_path(), destination.as_path(), &entries, &BTreeSet::new(), &[], &mut recorder, &CopyCeiling::unbounded()).expect("installed");
    // The holder gives its place back when a job of the work asks for one, so this answer is the proof that the
    //  install's slices reached the process's bound instead of starting at once.
    match wanted.recv_timeout(crate::parallel_test_support::PATIENCE) {
        Ok(true) => {}
        Ok(false) => panic!("the job that holds the only place saw no ask, so no slice of the install took a place of the process's bound"),
        Err(RecvTimeoutError::Timeout) => panic!("no slice of the install asked for a place of the process's bound within the patience, so the slices did not run as jobs of the blocking pool"),
        Err(RecvTimeoutError::Disconnected) => panic!("the job that holds the only place ended without answering"),
    }
    assert!(given_back.load(Ordering::SeqCst), "the place of the process's bound was never given back to a slice of the install");
    remove(root.as_path());
}

/// Selected paths are claimed and settled in batches of at most the batch size,
/// so the record count grows by batch,
/// not by entry.
///  The tree holds more entries than the batch size several times over:
/// a tree that fits in one batch cannot tell batching from
///  its absence,
/// which is how this control once stopped noticing the removal of the split after the batch size grew.
#[cfg(unix)]
#[test]
fn records_grow_by_batch_not_by_entry() {
    let root: PathBuf = fixture("install-batches");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(stage.join("many")).expect("many");
    let file_count: usize = INSTALL_BATCH_ENTRIES * FULL_BATCHES_OF_FILES + 1;
    for index in 0..file_count {
        std::fs::write(stage.join(format!("many/file-{index:05}")), b"x").expect("file");
    }
    let entries: Vec<WorktreeCopyEntry> =
        collect_entry_manifest(stage.as_path(), &[b"many".to_vec()], &[]).expect("manifest");
    assert!(
        entries.len() > INSTALL_BATCH_ENTRIES * FULL_BATCHES_OF_FILES,
        "the tree must span several batches or the control cannot see a missing split"
    );
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir(destination.as_path()).expect("destination");
    let mut recorder: CountingRecorder = CountingRecorder::default();
    install_snapshot(
        stage.as_path(),
        destination.as_path(),
        &entries,
        &BTreeSet::new(),
        &[],
        &mut recorder,
        &CopyCeiling::unbounded(),
    )
    .expect("installed");
    // The batches split the manifest at the batch size, one record per batch.
    let mut expected: Vec<usize> = Vec::new();
    let mut remaining: usize = entries.len();
    while remaining > 0 {
        let batch: usize = remaining.min(INSTALL_BATCH_ENTRIES);
        expected.push(batch);
        remaining -= batch;
    }
    assert!(expected.len() > FULL_BATCHES_OF_FILES, "several batches: {expected:?}");
    assert_eq!(recorder.intents, expected);
    assert_eq!(recorder.creations, expected);
    // The batch is past the parallel threshold, so several workers created its files; every one arrived, intact.
    for index in 0..file_count {
        assert_eq!(
            std::fs::read(destination.join(format!("many/file-{index:05}"))).expect("installed file"),
            b"x"
        );
    }
    remove(root.as_path());
}

/// A recorder failure after a parallel batch rolls back every file the workers created,
/// none left behind.
#[cfg(unix)]
#[test]
fn a_failed_parallel_batch_rolls_back_every_file_its_workers_created() {
    let root: PathBuf = fixture("install-parallel-rollback");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(stage.join("many")).expect("many");
    for index in 0..1_100 {
        std::fs::write(stage.join(format!("many/file-{index:04}")), b"x").expect("file");
    }
    let entries: Vec<WorktreeCopyEntry> =
        collect_entry_manifest(stage.as_path(), &[b"many".to_vec()], &[]).expect("manifest");
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir(destination.as_path()).expect("destination");
    let mut recorder: CountingRecorder = CountingRecorder {
        fail_creation_call: Some(1),
        ..CountingRecorder::default()
    };
    install_snapshot(
        stage.as_path(),
        destination.as_path(),
        &entries,
        &BTreeSet::new(),
        &[],
        &mut recorder,
        &CopyCeiling::unbounded(),
    )
    .expect_err("the recorder fails after the batch was created");
    assert!(
        std::fs::read_dir(destination.as_path()).expect("destination").next().is_none(),
        "rollback left the destination as it was"
    );
    remove(root.as_path());
}

/// A recorder failure after creation rolls back every path this installation created and keeps the destination's own entries.
#[cfg(unix)]
#[test]
fn failed_installation_rolls_back_its_own_paths() {
    let root: PathBuf = fixture("install-rollback");
    let (stage, entries) = stage_tree(root.as_path());
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir(destination.as_path()).expect("destination");
    std::fs::write(destination.join("user-file"), b"mine").expect("user file");
    let mut recorder: CountingRecorder = CountingRecorder {
        fail_creation_call: Some(1),
        ..CountingRecorder::default()
    };
    let failure: WorktreeCopyError = install_snapshot(
        stage.as_path(),
        destination.as_path(),
        &entries,
        &BTreeSet::new(),
        &[],
        &mut recorder,
        &CopyCeiling::unbounded(),
    )
    .expect_err("recorder failure");
    assert_eq!(
        failure.0,
        format!(
            "monochromatic-git-policy-cli: ignored-state installation failed for {}. Cause: recorder refused the batch",
            quote(destination.to_string_lossy().as_ref())
        )
    );
    assert!(!destination.join("cache").exists());
    assert_eq!(
        std::fs::read(destination.join("user-file")).expect("kept"),
        b"mine"
    );
    remove(root.as_path());
}

/// A path replaced after its creation is kept by rollback,
/// and the failure names it.
#[cfg(unix)]
#[test]
fn retained_paths_are_named_in_the_failure() {
    let root: PathBuf = fixture("install-retained");
    let (stage, entries) = stage_tree(root.as_path());
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir(destination.as_path()).expect("destination");
    let edited: PathBuf = destination.join("cache/top");
    let mut recorder: CountingRecorder = CountingRecorder {
        tamper: Some((1, edited.clone())),
        ..CountingRecorder::default()
    };
    let failure: WorktreeCopyError = install_snapshot(
        stage.as_path(),
        destination.as_path(),
        &entries,
        &BTreeSet::new(),
        &[],
        &mut recorder,
        &CopyCeiling::unbounded(),
    )
    .expect_err("tampered");
    assert_eq!(
        failure.0,
        format!(
            "monochromatic-git-policy-cli: ignored-state installation failed for {}. Rollback retained: \"cache/top: filesystem identity changed\", \"cache: Directory not empty (os error 39)\". Cause: recorder refused the batch",
            quote(destination.to_string_lossy().as_ref())
        )
    );
    assert_eq!(std::fs::read(edited).expect("kept"), b"tampered");
    remove(root.as_path());
}

/// An install goes parallel from the measured break-even number of leaves and not before.
#[test]
fn the_install_threshold_is_the_measured_break_even() {
    assert_eq!(PARALLEL_INSTALL_MIN_LEAVES, crate::parallel_budget::PARALLEL_BREAK_EVEN_ENTRIES);
}

/// Where the system starts no thread the install workers' slices are installed in turn,
/// with the same rollback.
/// The body runs in a
/// child that is refused every thread.
#[cfg(unix)]
#[test]
fn a_parallel_install_where_no_thread_can_start_is_made_in_turn() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "a_parallel_install_where_no_thread_can_start_is_made_in_turn") {
        return;
    }
    a_failed_parallel_batch_rolls_back_every_file_its_workers_created();
}
