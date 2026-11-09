//! Journaled installation checks against real Git repositories:
//! completion,
//! failure,
//! and resumption from the log.

use super::*;
use crate::worktree_copy_journal::create_journal;
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_stage::{new_stage_container, stage_ignored_snapshot};
use crate::test_support::{fixture, git, git_context, remove, repository};
use std::os::unix::ffi::OsStrExt;
use crate::worktree_copy_install_entry::capture_installed_path;
use crate::worktree_copy_install_log::{INSTALL_LOG_NAME, open_install_log};
use crate::worktree_copy_journal::{ensure_journal_root, read_pending_journals};

/// A source repository whose ignored `cache` holds one nested file,
/// and a destination directory beside it.
fn ignored_source(root: &Path) -> (PathBuf, PathBuf) {
    let source: PathBuf = repository(root, "source");
    std::fs::write(source.join(".gitignore"), b"cache/\n").expect("gitignore");
    std::fs::create_dir_all(source.join("cache/deep")).expect("cache");
    std::fs::write(source.join("cache/deep/data"), b"payload").expect("data");
    std::fs::write(source.join("cache/top"), b"top").expect("top");
    let destination: PathBuf = root.join("topic");
    return (
        std::fs::canonicalize(source).expect("canonical source"),
        destination,
    );
}

/// The common Git directory of a repository.
fn common_of(source: &Path) -> PathBuf {
    return std::fs::canonicalize(source.join(".git")).expect("common directory");
}

/// A staged snapshot of the source,
/// as the copy would produce it.
fn stage(source: &Path, destination: &Path) -> StagedSnapshot {
    return stage_ignored_snapshot(&git_context(), source, new_stage_container(destination).expect("container").as_path(), &[], &CopySelection::Ignored).expect("staged");
}

/// Entries of the destination with their content,
/// sorted,
/// as the bytes each name has,
/// for comparison.
fn tree_names(root: &Path) -> Vec<Vec<u8>> {
    let mut found: Vec<Vec<u8>> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for listed in std::fs::read_dir(&directory).expect("read") {
            let path: PathBuf = listed.expect("entry").path();
            let mut relative: Vec<u8> = path
                .strip_prefix(root)
                .expect("prefix")
                .as_os_str()
                .as_bytes()
                .to_vec();
            if path.is_dir() {
                relative.push(b'/');
                pending.push(path);
            }
            found.push(relative);
        }
    }
    found.sort();
    return found;
}


/// Recording claims writes only the ones no earlier record holds.
#[cfg(unix)]
#[test]
fn recorder_claims_each_path_once() {
    let root: PathBuf = fixture("transaction-recorder");
    let (source, destination) = ignored_source(root.as_path());
    let snapshot: StagedSnapshot = stage(source.as_path(), destination.as_path());
    let common: PathBuf = common_of(source.as_path());
    let pending: PendingJournal =
        create_journal(common.as_path(), destination.as_path(), &snapshot).expect("journal");
    let opened = open_install_log(snapshot.stage_container.as_path()).expect("log");
    let mut state: TransactionState = TransactionState {
        pending: pending.clone(),
        log: opened.log,
        intended: BTreeSet::new(),
        created_paths: BTreeSet::new(),
    };
    state
        .record_intents(&[b"cache".to_vec(), b"cache/top".to_vec()])
        .expect("first");
    state
        .record_intents(&[b"cache".to_vec()])
        .expect("repeat");
    state
        .record_creations(&[InstalledPath {
            device: 1,
            inode: 2,
            relative_path: b"cache".to_vec(),
            selected: true,
        }])
        .expect("creation");
    state.record_creations(&[]).expect("empty");
    let text: String =
        std::fs::read_to_string(snapshot.stage_container.join(INSTALL_LOG_NAME)).expect("log text");
    assert_eq!(text.lines().count(), 2, "{text}");
    assert_eq!(state.intended.len(), 2);
    remove(root.as_path());
}

/// A failed installation is removed with its journal and stage,
/// and the destination keeps its own differing entry.
#[cfg(unix)]
#[test]
fn failed_installation_never_stays_pending() {
    let root: PathBuf = fixture("transaction-failed");
    let (source, destination) = ignored_source(root.as_path());
    let snapshot: StagedSnapshot = stage(source.as_path(), destination.as_path());
    std::fs::create_dir_all(destination.join("cache")).expect("destination cache");
    std::fs::write(destination.join("cache/top"), b"differs").expect("differing entry");
    let common: PathBuf = common_of(source.as_path());
    let pending: PendingJournal =
        create_journal(common.as_path(), destination.as_path(), &snapshot).expect("journal");
    let staged: StagedSnapshot = snapshot.clone();
    let outcome: Outcome = complete_journal(pending, &SnapshotSource::Staged(&staged))
        .expect("journal fault-free");
    match outcome {
        Outcome::Ended(failure) => assert!(
            failure
                .0
                .contains("would overwrite differing destination entry \"cache/top\""),
            "{}",
            failure.0
        ),
        Outcome::Installed(_) => panic!("a differing entry must end the installation"),
    }
    assert!(
        read_pending_journals(common.as_path())
            .expect("pending")
            .is_empty()
    );
    assert!(!snapshot.stage_container.exists());
    assert_eq!(
        std::fs::read(destination.join("cache/top")).expect("kept"),
        b"differs"
    );
    remove(root.as_path());
}

/// An interrupted installation resumes from its log:
/// claimed and created paths are honored,
/// and the rest is installed.
#[cfg(unix)]
#[test]
fn interrupted_installation_resumes_from_its_log() {
    let root: PathBuf = fixture("transaction-resume");
    let (source, destination) = ignored_source(root.as_path());
    let snapshot: StagedSnapshot = stage(source.as_path(), destination.as_path());
    let common: PathBuf = common_of(source.as_path());
    ensure_journal_root(common.as_path()).expect("root");
    let pending: PendingJournal =
        create_journal(common.as_path(), destination.as_path(), &snapshot).expect("journal");
    // The interrupted owner claimed `cache` and created it before it died.
    std::fs::create_dir(destination.as_path()).expect("destination");
    std::fs::create_dir(destination.join("cache")).expect("created cache");
    let mut opened = open_install_log(snapshot.stage_container.as_path()).expect("log");
    opened
        .log
        .append_intents(&[b"cache".to_vec()])
        .expect("claim");
    let created_cache: InstalledPath =
        capture_installed_path(destination.join("cache").as_path(), b"cache", true)
            .expect("identity");
    opened
        .log
        .append_creations(&[created_cache])
        .expect("creation");
    drop(opened);
    write_journal(
        pending.path.as_path(),
        &Journal {
            phase: Phase::Installing,
            ..pending.journal.clone()
        },
    )
    .expect("installing");
    let resumable: PendingJournal = read_pending_journals(common.as_path())
        .expect("pending")
        .remove(0);
    let stage_root: PathBuf = resumable.journal.stage_root.clone();
    let rebuilt_from: PendingJournal = read_pending_journals(common.as_path())
        .expect("pending")
        .remove(0);
    let outcome: Outcome = complete_journal(resumable, &SnapshotSource::FromJournal(&rebuilt_from))
    .expect("completed");
    match outcome {
        // The claimed directory counts as created by the interrupted owner, so all four selected entries are counted.
        Outcome::Installed(copied) => assert_eq!(copied, 4),
        Outcome::Ended(failure) => panic!("resumption must finish: {}", failure.0),
    }
    assert_eq!(
        tree_names(destination.as_path()),
        vec![
            b"cache/".to_vec(),
            b"cache/deep/".to_vec(),
            b"cache/deep/data".to_vec(),
            b"cache/top".to_vec(),
        ]
    );
    assert_eq!(
        std::fs::read(destination.join("cache/top")).expect("top"),
        b"top"
    );
    assert!(!stage_root.exists());
    remove(root.as_path());
}

/// A stage that lost a claimed path cannot be recovered,
/// and the failure names the stage.
#[cfg(unix)]
#[test]
fn stage_missing_a_claim_cannot_be_recovered() {
    let root: PathBuf = fixture("transaction-stage-claim");
    let (source, destination) = ignored_source(root.as_path());
    let snapshot: StagedSnapshot = stage(source.as_path(), destination.as_path());
    let common: PathBuf = common_of(source.as_path());
    let pending: PendingJournal =
        create_journal(common.as_path(), destination.as_path(), &snapshot).expect("journal");
    let claims: BTreeSet<Vec<u8>> = BTreeSet::from([b"nowhere".to_vec()]);
    let failure: WorktreeCopyError =
        snapshot_from_journal(&pending, &claims).expect_err("claim absent");
    assert!(
        failure
            .0
            .starts_with("monochromatic-git-policy-cli: could not recover staged ignored state at "),
        "{}",
        failure.0
    );
    assert!(
        failure
            .0
            .contains("worktree-copy journal intent is absent from private stage"),
        "{}",
        failure.0
    );
    remove(root.as_path());
}

/// Beginning the installation writes the installing phase to the journal and to the state,
///  and a second call changes nothing.
#[cfg(unix)]
#[test]
fn beginning_the_installation_persists_the_installing_phase() {
    let root: PathBuf = fixture("transaction-begin-installing");
    let (source, destination) = ignored_source(root.as_path());
    let snapshot: StagedSnapshot = stage(source.as_path(), destination.as_path());
    let common: PathBuf = common_of(source.as_path());
    let pending: PendingJournal =
        create_journal(common.as_path(), destination.as_path(), &snapshot).expect("journal");
    assert_eq!(pending.journal.phase, Phase::Staged);
    let opened = open_install_log(snapshot.stage_container.as_path()).expect("log");
    let mut state: TransactionState = TransactionState {
        pending,
        log: opened.log,
        intended: BTreeSet::new(),
        created_paths: BTreeSet::new(),
    };
    begin_installing(&mut state).expect("begun");
    assert_eq!(state.pending.journal.phase, Phase::Installing);
    let written: Vec<PendingJournal> = read_pending_journals(common.as_path()).expect("pending");
    assert_eq!(written.len(), 1);
    assert_eq!(written[0].journal.phase, Phase::Installing);
    begin_installing(&mut state).expect("already begun");
    assert_eq!(state.pending.journal.phase, Phase::Installing);
    remove(root.as_path());
}

/// A destination another command moved and made again after the binding was checked receives nothing:
/// the identity of the registration the stage was made for is checked again where the first entry is written.
#[cfg(unix)]
#[test]
fn a_destination_replaced_after_the_check_is_not_installed_into() {
    use crate::worktree_add_binding::{bind_registration, path_identity};
    use crate::worktree_copy_stage_binding::record_stage_binding;
    let root: PathBuf = fixture("transaction-destination-replaced");
    let (source, destination) = ignored_source(root.as_path());
    git(source.as_path(), &["worktree", "add", "--quiet", "-b", "topic", destination.to_str().expect("utf8")]);
    let destination_canonical: PathBuf = std::fs::canonicalize(&destination).expect("canonical destination");
    let common: PathBuf = common_of(source.as_path());
    let binding = bind_registration(common.as_path(), destination_canonical.as_path()).expect("binding").binding;
    let snapshot: StagedSnapshot = stage(source.as_path(), destination_canonical.as_path());
    record_stage_binding(snapshot.stage_container.as_path(), &binding).expect("recorded");
    let pending: PendingJournal = create_journal(common.as_path(), destination_canonical.as_path(), &snapshot).expect("journal");
    // Another command moves this worktree away and registers a different one at the same path.
    let moved: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("moved");
    git(source.as_path(), &["worktree", "move", destination_canonical.to_str().expect("utf8"), moved.to_str().expect("utf8")]);
    git(
        source.as_path(),
        &["worktree", "add", "--quiet", "--no-checkout", "-b", "replacement", destination_canonical.to_str().expect("utf8")],
    );
    assert_ne!(
        path_identity(destination_canonical.as_path()),
        Some(binding.destination_id),
        "the path holds another directory now"
    );
    let failure: WorktreeCopyError = complete_journal(pending.clone(), &SnapshotSource::FromJournal(&pending)).expect_err("the registration is gone");
    assert!(failure.0.contains("no longer the registration the copy was made for"), "{}", failure.0);
    assert!(!destination_canonical.join("cache").exists(), "the replacement registration received nothing");
    remove(root.as_path());
}
