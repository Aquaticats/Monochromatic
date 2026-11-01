//! Recovery checks against real Git repositories: registration, discards with their notices, and resumption.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::worktree_copy_install_entry::InstalledPath;
use crate::worktree_copy_journal::{
    Journal, create_journal, create_journal_for, ensure_journal_root, read_pending_journals, write_journal,
};
use crate::worktree_copy_registry::{copied_for, ensure_registration_token};
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_stage::{new_stage_container, stage_ignored_snapshot};

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

/// A journal that an older release wrote, with no registration identity, is recovered by its path as it always was.
#[cfg(unix)]
#[test]
fn a_journal_without_an_identity_is_recovered_by_its_path() {
    let root: PathBuf = fixture("recovery-legacy");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let pending: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    assert_eq!(pending.journal.admin_id, None);
    let report: RecoveryReport = recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 1);
    assert!(report.notices.is_empty(), "{:?}", report.notices);
    assert_eq!(std::fs::read(topic.join("cache/data")).expect("copy"), b"payload");
    assert!(read_pending_journals(common.as_path()).expect("pending").is_empty());
    remove(root.as_path());
}

/// A destination is the registration of a journal only when the path, the name and the identity agree.
#[cfg(unix)]
#[test]
fn a_registration_is_told_by_its_identity_and_not_by_its_path() {
    let root: PathBuf = fixture("recovery-identity");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let admin_root: PathBuf = common.join("worktrees");
    let token: String = ensure_registration_token(admin_root.as_path(), "topic", None).expect("identity").expect("registered");
    assert_eq!(registration_of(common.as_path(), topic.as_path(), "topic", token.as_str()).expect("same"), Registration::Registered);
    // Another identity under the same name and path: a registration that reused them.
    assert_eq!(registration_of(common.as_path(), topic.as_path(), "topic", "another").expect("other"), Registration::Unregistered);
    // The right identity, but a pointer file that names another administrative directory.
    assert_eq!(registration_of(common.as_path(), topic.as_path(), "elsewhere", token.as_str()).expect("name"), Registration::Unregistered);
    // A path that is no worktree.
    assert_eq!(
        registration_of(common.as_path(), root.join("absent").as_path(), "topic", token.as_str()).expect("absent"),
        Registration::Unregistered
    );
    remove(root.as_path());
}

/// A journal whose registration was replaced by another of the same name is discarded, not installed into the new one.
#[cfg(unix)]
#[test]
fn a_journal_of_a_replaced_registration_is_discarded() {
    let root: PathBuf = fixture("recovery-replaced");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let token: String = ensure_registration_token(common.join("worktrees").as_path(), "topic", None).expect("identity").expect("registered");
    let snapshot: StagedSnapshot = stage_ignored_snapshot(
        &git_context(),
        source.as_path(),
        new_stage_container(topic.as_path()).expect("container").as_path(),
        &[],
        &CopySelection::Ignored,
    )
    .expect("staged");
    let pending: PendingJournal =
        create_journal_for(common.as_path(), topic.as_path(), &snapshot, Some(("topic", token.as_str()))).expect("journal");
    git(source.as_path(), &["worktree", "remove", "--force", topic.to_str().expect("utf8")]);
    git(source.as_path(), &["worktree", "add", "--quiet", "-b", "again", topic.to_str().expect("utf8")]);
    let report: RecoveryReport = recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 0);
    assert!(report.notices[0].contains("no longer a registered worktree"), "{:?}", report.notices);
    assert!(!topic.join("cache").exists(), "the new registration was filled from the old one's stage");
    assert!(!pending.path.exists());
    remove(root.as_path());
}

/// Finishing a journal installs, marks the registration, and only then removes the journal; stopping after either step
///  leaves exactly what a kill there leaves.
#[cfg(unix)]
#[test]
fn finishing_a_journal_marks_before_it_removes() {
    let root: PathBuf = fixture("recovery-finish-order");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let admin_root: PathBuf = common.join("worktrees");
    let token: String = ensure_registration_token(admin_root.as_path(), "topic", None).expect("identity").expect("registered");
    let snapshot: StagedSnapshot = stage_ignored_snapshot(
        &git_context(),
        source.as_path(),
        new_stage_container(topic.as_path()).expect("container").as_path(),
        &[],
        &CopySelection::Ignored,
    )
    .expect("staged");
    let pending: PendingJournal =
        create_journal_for(common.as_path(), topic.as_path(), &snapshot, Some(("topic", token.as_str()))).expect("journal");
    let staged: StagedSnapshot = snapshot.clone();
    let mut steps: Vec<Step> = Vec::new();
    let finished = finish_journal(common.as_path(), pending.clone(), &|_intended| return Ok(staged.clone()), &mut |step| {
        steps.push(step);
        // At the first step nothing is marked; at the second the marker is there and the journal still is.
        match step {
            Step::Installed => assert!(!copied_for(admin_root.as_path(), "topic", token.as_str()).expect("marker")),
            Step::Marked => {
                assert!(copied_for(admin_root.as_path(), "topic", token.as_str()).expect("marker"));
                assert!(pending.path.exists());
            }
        }
        return true;
    })
    .expect("finished");
    assert!(matches!(finished, Finished::Done(Outcome::Installed(_))));
    assert_eq!(steps, vec![Step::Installed, Step::Marked]);
    assert!(!pending.path.exists());
    remove(root.as_path());
}

/// A failed installation is not marked: it rolled back its own paths and a later run may try again.
#[cfg(unix)]
#[test]
fn a_failed_installation_is_not_marked() {
    let root: PathBuf = fixture("recovery-failed-unmarked");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let admin_root: PathBuf = common.join("worktrees");
    let token: String = ensure_registration_token(admin_root.as_path(), "topic", None).expect("identity").expect("registered");
    std::fs::create_dir_all(topic.join("cache")).expect("cache");
    std::fs::write(topic.join("cache/data"), b"differs").expect("differing entry");
    let snapshot: StagedSnapshot = stage_ignored_snapshot(
        &git_context(),
        source.as_path(),
        new_stage_container(topic.as_path()).expect("container").as_path(),
        &[],
        &CopySelection::Ignored,
    )
    .expect("staged");
    let pending: PendingJournal =
        create_journal_for(common.as_path(), topic.as_path(), &snapshot, Some(("topic", token.as_str()))).expect("journal");
    let staged: StagedSnapshot = snapshot.clone();
    let finished = finish_journal(common.as_path(), pending, &|_intended| return Ok(staged.clone()), &mut |_step| return true)
        .expect("finished");
    assert!(matches!(finished, Finished::Done(Outcome::Ended(_))));
    assert!(!copied_for(admin_root.as_path(), "topic", token.as_str()).expect("marker"));
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

/// The journal is not complete when the installation returns: completeness is written only by the cleanup that follows the
/// marker, so a kill between the installation and the marker leaves a journal that recovery resumes and marks.
#[cfg(unix)]
#[test]
fn the_journal_is_not_complete_when_the_installation_returns() {
    let root: PathBuf = fixture("recovery-phase-before-marker");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let admin_root: PathBuf = common.join("worktrees");
    let token: String = ensure_registration_token(admin_root.as_path(), "topic", None).expect("identity").expect("registered");
    let snapshot: StagedSnapshot = stage_ignored_snapshot(
        &git_context(),
        source.as_path(),
        new_stage_container(topic.as_path()).expect("container").as_path(),
        &[],
        &CopySelection::Ignored,
    )
    .expect("staged");
    let pending: PendingJournal =
        create_journal_for(common.as_path(), topic.as_path(), &snapshot, Some(("topic", token.as_str()))).expect("journal");
    let staged: StagedSnapshot = snapshot.clone();
    let mut phases: Vec<Phase> = Vec::new();
    finish_journal(common.as_path(), pending, &|_intended| return Ok(staged.clone()), &mut |_step| {
        phases.extend(read_pending_journals(common.as_path()).expect("pending").into_iter().map(|journal| return journal.journal.phase));
        return false;
    })
    .expect("interrupted at the first step");
    assert_eq!(phases, vec![Phase::Installing], "the installation left the journal complete before the marker");
    remove(root.as_path());
}

/// A complete journal that carries no marker is the cleanup of a rolled-back copy or of a discard, which are not copies
/// that happened: recovery removes it and its stage and does not mark the registration.
#[cfg(unix)]
#[test]
fn a_complete_unmarked_journal_is_cleaned_up_and_never_marked() {
    let root: PathBuf = fixture("recovery-complete-unmarked");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let admin_root: PathBuf = common.join("worktrees");
    let token: String = ensure_registration_token(admin_root.as_path(), "topic", None).expect("identity").expect("registered");
    let snapshot: StagedSnapshot = stage_ignored_snapshot(
        &git_context(),
        source.as_path(),
        new_stage_container(topic.as_path()).expect("container").as_path(),
        &[],
        &CopySelection::Ignored,
    )
    .expect("staged");
    let pending: PendingJournal =
        create_journal_for(common.as_path(), topic.as_path(), &snapshot, Some(("topic", token.as_str()))).expect("journal");
    write_journal(pending.path.as_path(), &Journal { phase: Phase::Complete, ..pending.journal.clone() }).expect("complete");
    let report: RecoveryReport = recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert!(report.notices.is_empty(), "{:?}", report.notices);
    assert!(read_pending_journals(common.as_path()).expect("pending").is_empty());
    assert!(!Path::new(pending.journal.stage_container.as_str()).exists(), "the stage stays");
    assert!(!copied_for(admin_root.as_path(), "topic", token.as_str()).expect("marker"), "a cleanup marked a registration");
    assert!(!topic.join("cache").exists(), "a cleanup installed files");
    remove(root.as_path());
}
