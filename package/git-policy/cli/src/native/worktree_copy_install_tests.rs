//! Installation checks on real filesystem trees: preflight, batching, modes and rollback on failure.

use super::*;
use crate::test_support::{fixture, remove};
use crate::worktree_copy_manifest::collect_entry_manifest;

/// A recorder that counts its calls and can fail one call on demand.
#[derive(Debug, Default)]
struct CountingRecorder {
    /// Sizes of each intent batch.
    intents: Vec<usize>,
    /// Sizes of each creation batch.
    creations: Vec<usize>,
    /// Creation call number that fails, counted from one.
    fail_creation_call: Option<usize>,
    /// Creation call number that first rewrites a path in place, then fails.
    tamper: Option<(usize, PathBuf)>,
}

impl InstallRecorder for CountingRecorder {
    /// Counts one intent batch.
    fn record_intents(&mut self, relative_paths: &[String]) -> Result<(), WorktreeCopyError> {
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

/// A stage directory holding one tree, with its manifest.
fn stage_tree(root: &Path) -> (PathBuf, Vec<WorktreeCopyEntry>) {
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(stage.join("cache/deep")).expect("stage tree");
    std::fs::write(stage.join("cache/deep/data"), b"payload").expect("data");
    std::fs::write(stage.join("cache/top"), b"top").expect("top");
    let entries: Vec<WorktreeCopyEntry> =
        collect_entry_manifest(stage.as_path(), &[String::from("cache")], &[]).expect("manifest");
    return (stage, entries);
}

/// Every existing entry must be identical; absent entries are reported in manifest order.
#[test]
fn preflight_accepts_identical_entries_and_lists_absent_ones() {
    let root: PathBuf = fixture("install-preflight");
    let (stage, entries) = stage_tree(root.as_path());
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir_all(destination.join("cache")).expect("cache");
    std::fs::write(destination.join("cache/top"), b"top").expect("identical");
    let absent: BTreeSet<String> = preflight_destination(
        stage.as_path(),
        destination.as_path(),
        &entries,
        &BTreeSet::new(),
    )
    .expect("preflight");
    assert_eq!(
        absent,
        BTreeSet::from([String::from("cache/deep"), String::from("cache/deep/data")])
    );
    remove(root.as_path());
}

/// A differing entry stops the installation, names the entry and the destination, and is never overwritten.
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
            "cli-git: ignored-state copy would overwrite differing destination entry \"cache/top\" in {}.",
            quote(destination.to_string_lossy().as_ref())
        )
    );
    assert_eq!(
        std::fs::read(destination.join("cache/top")).expect("kept"),
        b"other"
    );
    remove(root.as_path());
}

/// A directory this transaction claimed is accepted whatever its contents, so an interrupted run can resume.
#[test]
fn claimed_directory_is_accepted_for_resumption() {
    let root: PathBuf = fixture("install-preflight-claimed");
    let (stage, entries) = stage_tree(root.as_path());
    let destination: PathBuf = root.join("destination");
    std::fs::create_dir_all(destination.join("cache/deep")).expect("claimed");
    let claimed: BTreeSet<String> = BTreeSet::from([String::from("cache")]);
    let absent: BTreeSet<String> =
        preflight_destination(stage.as_path(), destination.as_path(), &entries, &claimed)
            .expect("claimed");
    assert_eq!(
        absent,
        BTreeSet::from([String::from("cache/deep/data"), String::from("cache/top")])
    );
    remove(root.as_path());
}

/// A complete installation copies content, links and modes, and reports the selected entries it created.
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
        collect_entry_manifest(stage.as_path(), &[String::from("cache")], &[]).expect("manifest");
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

/// Selected paths are claimed and settled in batches of at most the batch size, so the record count grows by batch, not by entry.
#[cfg(unix)]
#[test]
fn records_grow_by_batch_not_by_entry() {
    let root: PathBuf = fixture("install-batches");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(stage.join("many")).expect("many");
    for index in 0..1_100 {
        std::fs::write(stage.join(format!("many/file-{index:04}")), b"x").expect("file");
    }
    let entries: Vec<WorktreeCopyEntry> =
        collect_entry_manifest(stage.as_path(), &[String::from("many")], &[]).expect("manifest");
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
    )
    .expect("installed");
    // The manifest holds the directory and 1,100 files; the batches split it at 512.
    assert_eq!(INSTALL_BATCH_ENTRIES, 512);
    assert_eq!(recorder.intents, vec![512, 512, 77]);
    assert_eq!(recorder.creations, vec![512, 512, 77]);
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
    )
    .expect_err("recorder failure");
    assert_eq!(
        failure.0,
        format!(
            "cli-git: ignored-state installation failed for {}. Cause: recorder refused the batch",
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

/// A path replaced after its creation is kept by rollback, and the failure names it.
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
    )
    .expect_err("tampered");
    assert_eq!(
        failure.0,
        format!(
            "cli-git: ignored-state installation failed for {}. Rollback retained: \"cache/top: filesystem identity changed\", \"cache: Directory not empty (os error 39)\". Cause: recorder refused the batch",
            quote(destination.to_string_lossy().as_ref())
        )
    );
    assert_eq!(std::fs::read(edited).expect("kept"), b"tampered");
    remove(root.as_path());
}
