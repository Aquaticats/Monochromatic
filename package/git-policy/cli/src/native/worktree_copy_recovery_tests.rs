//! Recovery checks against real Git repositories: registration, discards with their notices, and resumption.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::worktree_copy_install_entry::InstalledPath;
use crate::worktree_copy_journal::{
    Journal, create_journal, ensure_journal_root, read_pending_journals, write_journal,
};
use crate::worktree_copy_lock::{SETTLEMENT_LOCK_NAME, acquire_worktree_copy_lock};
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_stage::{new_stage_container, stage_ignored_snapshot};
use crate::worktree_copy_stage_claim::{claim_stage, stage_claims_exist};
use std::os::unix::fs::PermissionsExt;

/// A repository with an ignored `cache` file and a linked worktree at `topic` that the copy would fill.
fn linked_source(root: &Path) -> (PathBuf, PathBuf) {
    let source: PathBuf = repository(root, "source");
    std::fs::write(source.join(".gitignore"), b"cache/\n").expect("gitignore");
    std::fs::create_dir_all(source.join("cache")).expect("cache");
    std::fs::write(source.join("cache/data"), b"payload").expect("data");
    let topic: PathBuf = root.join("topic");
    git(
        source.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            topic.to_str().expect("utf8"),
        ],
    );
    return (
        std::fs::canonicalize(source).expect("canonical source"),
        std::fs::canonicalize(topic).expect("canonical topic"),
    );
}

/// Creates a staged journal for a destination, as an interrupted copy would have left it.
fn staged_journal(source: &Path, destination: &Path) -> PendingJournal {
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let snapshot: StagedSnapshot =
        stage_ignored_snapshot(&git_context(), source, new_stage_container(destination).expect("container").as_path(), &[], &CopySelection::Ignored).expect("staged");
    return create_journal(common.as_path(), destination, &snapshot).expect("journal");
}

/// A linked worktree of the repository is registered; a removed or re-pointed one is not.
#[cfg(unix)]
#[test]
fn registration_follows_the_linked_pointer() {
    let root: PathBuf = fixture("recovery-registration");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    assert_eq!(
        destination_registration(common.as_path(), topic.as_path()).expect("registered"),
        Registration::Registered
    );
    let absent: PathBuf = root.join("absent");
    assert_eq!(
        destination_registration(common.as_path(), absent.as_path()).expect("absent"),
        Registration::Unregistered
    );
    let other: PathBuf = root.join("other");
    std::fs::create_dir(other.as_path()).expect("other");
    std::fs::write(other.join(".git"), b"gitdir: /nonexistent/admin\n").expect("dangling pointer");
    assert_eq!(
        destination_registration(common.as_path(), other.as_path()).expect("dangling"),
        Registration::Unregistered
    );
    let plain: PathBuf = root.join("plain");
    std::fs::create_dir(plain.as_path()).expect("plain");
    std::fs::write(plain.join(".git"), b"not a pointer\n").expect("plain pointer");
    assert_eq!(
        destination_registration(common.as_path(), plain.as_path()).expect("plain"),
        Registration::Unregistered
    );
    remove(root.as_path());
}

/// A journal whose destination is no longer registered is discarded with the notice, and its stage goes too.
#[cfg(unix)]
#[test]
fn unregistered_destination_is_discarded_with_notice() {
    let root: PathBuf = fixture("recovery-unregistered");
    let (source, topic) = linked_source(root.as_path());
    let pending: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    let stage: PathBuf = PathBuf::from(pending.journal.stage_container.as_str());
    git(
        source.as_path(),
        &[
            "worktree",
            "remove",
            "--force",
            topic.to_str().expect("utf8"),
        ],
    );
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let report: RecoveryReport =
        recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 0);
    assert_eq!(
        report.notices,
        vec![format!(
            "cli-git: discarded an interrupted ignored-state copy for {}, which is no longer a registered worktree.",
            quote(topic.to_string_lossy().as_ref())
        )]
    );
    assert!(!stage.exists());
    assert!(
        read_pending_journals(common.as_path())
            .expect("pending")
            .is_empty()
    );
    remove(root.as_path());
}

/// A journal whose stage is gone is discarded with the incomplete-ignored-files notice.
#[cfg(unix)]
#[test]
fn missing_stage_is_discarded_with_notice() {
    let root: PathBuf = fixture("recovery-missing-stage");
    let (source, topic) = linked_source(root.as_path());
    let pending: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    std::fs::remove_dir_all(pending.journal.stage_container.as_str()).expect("stage removed");
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let report: RecoveryReport =
        recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(
        report.notices,
        vec![format!(
            "cli-git: discarded an interrupted ignored-state copy into {} because its private stage is missing; ignored files there may be incomplete.",
            quote(topic.to_string_lossy().as_ref())
        )]
    );
    assert!(
        read_pending_journals(common.as_path())
            .expect("pending")
            .is_empty()
    );
    remove(root.as_path());
}

/// A journal left in the installing phase with an intact stage is resumed and completed.
#[cfg(unix)]
#[test]
fn interrupted_installation_is_resumed() {
    let root: PathBuf = fixture("recovery-resume");
    let (source, topic) = linked_source(root.as_path());
    let pending: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    let installing: Journal = Journal {
        phase: Phase::Installing,
        ..pending.journal.clone()
    };
    write_journal(pending.path.as_path(), &installing).expect("installing phase");
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let report: RecoveryReport =
        recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 1);
    assert!(report.notices.is_empty(), "{:?}", report.notices);
    assert_eq!(
        std::fs::read(topic.join("cache/data")).expect("resumed copy"),
        b"payload"
    );
    assert!(
        read_pending_journals(common.as_path())
            .expect("pending")
            .is_empty()
    );
    remove(root.as_path());
}

/// A journal already marked complete only has its cleanup finished, and counts as recovered.
#[cfg(unix)]
#[test]
fn completed_journal_only_cleans_up() {
    let root: PathBuf = fixture("recovery-complete");
    let (source, topic) = linked_source(root.as_path());
    let pending: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    write_journal(
        pending.path.as_path(),
        &Journal {
            phase: Phase::Complete,
            ..pending.journal.clone()
        },
    )
    .expect("complete phase");
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let report: RecoveryReport =
        recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 1);
    assert!(!topic.join("cache").exists());
    assert!(
        read_pending_journals(common.as_path())
            .expect("pending")
            .is_empty()
    );
    remove(root.as_path());
}

/// Recovery skips the settlement lock when no journal is pending, and skips recovery while a live owner holds it.
#[cfg(unix)]
#[test]
fn pending_check_takes_the_lock_only_for_a_journal() {
    let root: PathBuf = fixture("recovery-pending-check");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    recover_pending_worktree_copies(common.as_path(), &mut Vec::new()).expect("nothing pending");
    assert!(
        !ensure_journal_root(common.as_path())
            .expect("root")
            .join(SETTLEMENT_LOCK_NAME)
            .exists()
    );
    let pending: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    let mut stderr: Vec<u8> = Vec::new();
    let held = acquire_worktree_copy_lock(common.as_path(), &mut stderr).expect("held by us");
    recover_pending_worktree_copies(common.as_path(), &mut Vec::new())
        .expect("held by a live owner");
    assert!(
        pending.path.exists(),
        "a live owner keeps the journal for itself"
    );
    held.release().expect("released");
    remove(root.as_path());
}

/// A stage left by a copy that died before its journal existed is removed by recovery, with a notice and no count.
#[cfg(unix)]
#[test]
fn recovery_removes_a_stage_no_journal_names() {
    let root: PathBuf = fixture("recovery-abandoned-stage");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let claim = claim_stage(common.as_path(), topic.as_path()).expect("claim");
    std::fs::create_dir(claim.container()).expect("stage");
    std::fs::set_permissions(claim.container(), std::fs::Permissions::from_mode(0o700)).expect("private");
    std::fs::write(claim.container().join("half"), b"copied so far").expect("partial copy");
    let report = recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 0);
    assert_eq!(report.notices.len(), 1);
    assert!(report.notices[0].contains("removed the stage"), "{:?}", report.notices);
    assert!(!claim.container().exists());
    assert!(!stage_claims_exist(common.as_path()).expect("claims"));
    remove(root.as_path());
}

/// A forwarded command that creates no worktree reaps an abandoned stage when no live copy holds the lock,
/// and leaves it alone while one does.
#[cfg(unix)]
#[test]
fn the_pending_check_reaps_abandoned_stages_only_without_a_live_owner() {
    let root: PathBuf = fixture("recovery-pending-claim");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let claim = claim_stage(common.as_path(), topic.as_path()).expect("claim");
    std::fs::create_dir(claim.container()).expect("stage");
    std::fs::set_permissions(claim.container(), std::fs::Permissions::from_mode(0o700)).expect("private");
    let mut stderr: Vec<u8> = Vec::new();
    let held = acquire_worktree_copy_lock(common.as_path(), &mut stderr).expect("held by us");
    recover_pending_worktree_copies(common.as_path(), &mut stderr).expect("held by a live owner");
    assert!(claim.container().exists(), "a live copy may still be staging");
    held.release().expect("released");
    let mut notices: Vec<u8> = Vec::new();
    recover_pending_worktree_copies(common.as_path(), &mut notices).expect("reaped");
    assert!(!claim.container().exists());
    assert!(String::from_utf8_lossy(notices.as_slice()).contains("removed the stage"));
    remove(root.as_path());
}

/// The summary line counts transactions with the singular or plural noun, after the notices.
#[test]
fn report_writes_notices_then_summary() {
    let mut out: Vec<u8> = Vec::new();
    report_recovered(
        &RecoveryReport {
            recovered: 1,
            notices: vec![String::from("cli-git: notice.")],
        },
        &mut out,
    )
    .expect("written");
    assert_eq!(
        String::from_utf8(out).expect("utf8"),
        "cli-git: notice.\ncli-git: recovered ignored-state copies for 1 worktree transaction.\n"
    );
    let mut plural: Vec<u8> = Vec::new();
    report_recovered(
        &RecoveryReport {
            recovered: 2,
            notices: Vec::new(),
        },
        &mut plural,
    )
    .expect("written");
    assert_eq!(
        String::from_utf8(plural).expect("utf8"),
        "cli-git: recovered ignored-state copies for 2 worktree transactions.\n"
    );
}

/// The bytes the incumbent's own `writeJournal` wrote for a phase-installing journal, with the base directory as `@BASE@`.
/// Generated by running `createWorktreeCopyJournal` and `writeJournal` from `src/worktree-copy/journal.ts` against a scratch base directory.
const INCUMBENT_JOURNAL: &str = "{\"createdEntries\":[{\"device\":\"7\",\"inode\":\"42\",\"relativePath\":\"cache\",\"selected\":true}],\"intendedEntries\":[\"cache\"],\"destinationRoot\":\"@BASE@/topic\",\"phase\":\"installing\",\"selectedRoots\":[\"cache\"],\"sourceRoot\":\"@BASE@/source\",\"stageContainer\":\"@BASE@/.cli-git-worktree-copy-abc\",\"stageRoot\":\"@BASE@/.cli-git-worktree-copy-abc/payload\",\"version\":1}\n";

/// A journal the incumbent wrote is read by the native wrapper with the same fields, and recovered: its destination is not a worktree, so it is discarded with the incumbent's notice and its stage goes too.
#[cfg(unix)]
#[test]
fn incumbent_written_journal_is_recovered_natively() {
    let root: PathBuf = fixture("recovery-incumbent-journal");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let journal_root: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    let base: String = root.to_string_lossy().into_owned();
    let stage: PathBuf = root.join(".cli-git-worktree-copy-abc");
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(stage.join("payload"))
            .expect("private stage");
    }
    let text: String = INCUMBENT_JOURNAL.replace("@BASE@", base.as_str());
    let path: PathBuf = journal_root.join("6ff4c81f-f579-4616-b5d7-4c0694f77b6c.json");
    crate::private_storage::write_private_file(path.as_path(), text.as_bytes())
        .expect("incumbent journal");
    let pending: Vec<PendingJournal> = read_pending_journals(common.as_path()).expect("read");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].journal.phase, Phase::Installing);
    assert_eq!(
        pending[0].journal.created_entries,
        vec![InstalledPath {
            device: 7,
            inode: 42,
            relative_path: String::from("cache"),
            selected: true,
        }]
    );
    let report: RecoveryReport =
        recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(
        report.notices,
        vec![format!(
            "cli-git: discarded an interrupted ignored-state copy for {}, which is no longer a registered worktree.",
            quote(format!("{base}/topic").as_str())
        )]
    );
    assert!(!stage.exists());
    assert!(
        read_pending_journals(common.as_path())
            .expect("none left")
            .is_empty()
    );
    remove(root.as_path());
}

/// A destination under something that is not a directory is unregistered,
///  while a destination whose pointer cannot be read for any other reason is a fault to report.
#[cfg(unix)]
#[test]
fn registration_tells_a_missing_destination_from_an_unreadable_one() {
    let root: PathBuf = std::fs::canonicalize(fixture("recovery-registration-faults")).expect("canonical");
    let (source, _topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let file: PathBuf = root.join("a-file");
    std::fs::write(file.as_path(), b"not a directory").expect("file");
    assert_eq!(
        destination_registration(common.as_path(), file.join("below").as_path()).expect("under a file"),
        Registration::Unregistered
    );
    let unreadable: PathBuf = root.join("unreadable");
    std::fs::create_dir_all(unreadable.join(".git")).expect("a directory where the pointer file belongs");
    let failure: WorktreeCopyError =
        destination_registration(common.as_path(), unreadable.as_path()).expect_err("a directory is not a pointer");
    assert!(failure.0.contains("could not read worktree registration"), "{}", failure.0);
    remove(root.as_path());
}

/// A stage that cannot be examined for any reason other than being absent is a fault, not a missing stage.
#[cfg(unix)]
#[test]
fn a_stage_that_cannot_be_examined_is_a_fault_and_not_a_missing_stage() {
    let root: PathBuf = fixture("recovery-stage-blocked");
    let (source, topic) = linked_source(root.as_path());
    let mut pending: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    let blocker: PathBuf = root.join("blocker");
    std::fs::write(blocker.as_path(), b"a file").expect("blocker");
    pending.journal.stage_container = blocker.join("stage").to_string_lossy().into_owned();
    let failure: WorktreeCopyError = validate_stage(&pending).expect_err("the stage cannot be examined");
    assert!(
        failure.0.contains("could not validate worktree-copy recovery state"),
        "{}",
        failure.0
    );
    remove(root.as_path());
}
