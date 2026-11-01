//! Journaled installation checks against real Git repositories: completion, failure, and resumption from the log.

use super::*;
use crate::worktree_copy_stage::new_stage_container;
use crate::worktree_copy_stage_claim::stage_claims_exist;
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::worktree_copy_install_entry::capture_installed_path;
use crate::worktree_copy_install_log::{INSTALL_LOG_NAME, open_install_log};
use crate::worktree_copy_journal::{ensure_journal_root, read_pending_journals};

/// A source repository whose ignored `cache` holds one nested file, and a destination directory beside it.
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

/// A staged snapshot of the source, as the copy would produce it.
fn stage(source: &Path, destination: &Path) -> StagedSnapshot {
    return stage_ignored_snapshot(&git_context(), source, new_stage_container(destination).expect("container").as_path(), &[], &CopySelection::Ignored).expect("staged");
}

/// Entries of the destination with their content, sorted, for comparison.
fn tree(root: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for listed in std::fs::read_dir(&directory).expect("read") {
            let path: PathBuf = listed.expect("entry").path();
            let relative: String = path
                .strip_prefix(root)
                .expect("prefix")
                .to_string_lossy()
                .into_owned();
            if path.is_dir() {
                found.push(format!("{relative}/"));
                pending.push(path);
            } else {
                found.push(relative);
            }
        }
    }
    found.sort();
    return found;
}

/// A complete synchronization installs the ignored state and leaves neither a journal nor a stage behind.
#[cfg(unix)]
#[test]
fn completed_synchronization_leaves_no_journal_or_stage() {
    let root: PathBuf = fixture("transaction-complete");
    let (source, destination) = ignored_source(root.as_path());
    git(
        source.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            destination.to_str().expect("utf8"),
        ],
    );
    let destination_canonical: PathBuf =
        std::fs::canonicalize(&destination).expect("canonical destination");
    let common: PathBuf = common_of(source.as_path());
    let copied: usize = synchronize_created_worktree(
        &git_context(),
        common.as_path(),
        source.as_path(),
        destination_canonical.as_path(),
        &[],
        &CopySelection::Ignored,
    )
    .expect("synchronized");
    assert_eq!(copied, 4);
    assert_eq!(
        std::fs::read(destination_canonical.join("cache/deep/data")).expect("data"),
        b"payload"
    );
    assert!(
        read_pending_journals(common.as_path())
            .expect("pending")
            .is_empty()
    );
    let leftovers: Vec<String> = std::fs::read_dir(destination_canonical.parent().expect("parent"))
        .expect("list")
        .map(|entry| {
            return entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned();
        })
        .filter(|name| return name.starts_with(".cli-git-worktree-copy-"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
    assert!(
        !stage_claims_exist(common.as_path()).expect("claims"),
        "a settled copy releases its stage claim"
    );
    remove(root.as_path());
}

/// A copy that fails while staging removes its stage and releases its claim, so nothing is left to reap.
#[cfg(unix)]
#[test]
fn failed_staging_releases_its_claim() {
    let root: PathBuf = fixture("transaction-staging-failure");
    let (source, _destination) = ignored_source(root.as_path());
    let common: PathBuf = common_of(source.as_path());
    let destination: PathBuf = root.join("absent").join("topic");
    let failure: WorktreeCopyError = synchronize_created_worktree(
        &git_context(),
        common.as_path(),
        source.as_path(),
        destination.as_path(),
        &[],
        &CopySelection::Ignored,
    )
    .expect_err("the stage parent is absent");
    assert!(!failure.0.is_empty());
    assert!(!stage_claims_exist(common.as_path()).expect("claims"), "the failed copy released its claim");
    assert!(read_pending_journals(common.as_path()).expect("pending").is_empty());
    remove(root.as_path());
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
        .record_intents(&[String::from("cache"), String::from("cache/top")])
        .expect("first");
    state
        .record_intents(&[String::from("cache")])
        .expect("repeat");
    state
        .record_creations(&[InstalledPath {
            device: 1,
            inode: 2,
            relative_path: String::from("cache"),
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

/// A failed installation is removed with its journal and stage, and the destination keeps its own differing entry.
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
    let outcome: Outcome = complete_journal(pending, &|_intended| return Ok(staged.clone()))
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

/// An interrupted installation resumes from its log: claimed and created paths are honored, and the rest is installed.
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
        .append_intents(&[String::from("cache")])
        .expect("claim");
    let created_cache: InstalledPath =
        capture_installed_path(destination.join("cache").as_path(), "cache", true)
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
    let stage_root: PathBuf = PathBuf::from(resumable.journal.stage_root.as_str());
    let outcome: Outcome = complete_journal(resumable, &|intended| {
        return snapshot_from_journal(
            &read_pending_journals(common.as_path())
                .expect("pending")
                .remove(0),
            intended,
        );
    })
    .expect("completed");
    match outcome {
        // The claimed directory counts as created by the interrupted owner, so all four selected entries are counted.
        Outcome::Installed(copied) => assert_eq!(copied, 4),
        Outcome::Ended(failure) => panic!("resumption must finish: {}", failure.0),
    }
    assert_eq!(
        tree(destination.as_path()),
        vec![
            String::from("cache/"),
            String::from("cache/deep/"),
            String::from("cache/deep/data"),
            String::from("cache/top"),
        ]
    );
    assert_eq!(
        std::fs::read(destination.join("cache/top")).expect("top"),
        b"top"
    );
    assert!(!stage_root.exists());
    remove(root.as_path());
}

/// A stage that lost a claimed path cannot be recovered, and the failure names the stage.
#[cfg(unix)]
#[test]
fn stage_missing_a_claim_cannot_be_recovered() {
    let root: PathBuf = fixture("transaction-stage-claim");
    let (source, destination) = ignored_source(root.as_path());
    let snapshot: StagedSnapshot = stage(source.as_path(), destination.as_path());
    let common: PathBuf = common_of(source.as_path());
    let pending: PendingJournal =
        create_journal(common.as_path(), destination.as_path(), &snapshot).expect("journal");
    let claims: BTreeSet<String> = BTreeSet::from([String::from("nowhere")]);
    let failure: WorktreeCopyError =
        snapshot_from_journal(&pending, &claims).expect_err("claim absent");
    assert!(
        failure
            .0
            .starts_with("cli-git: could not recover staged ignored state at "),
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
