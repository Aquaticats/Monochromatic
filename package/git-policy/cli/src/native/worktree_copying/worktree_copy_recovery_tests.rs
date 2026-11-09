//! Recovery checks against real Git repositories:
//! registration,
//! discards with their notices,
//! and resumption.

use super::*;
use crate::test_support::{dead_copy_owner, fixture, git, git_context, remove, repository};
use crate::worktree_copy_install_entry::InstalledPath;
use crate::worktree_copy_journal::{
    adopt_journal, create_journal, create_journal_for, ensure_journal_root, journal_root, read_pending_journals, write_journal,
};
use crate::worktree_copy_journal_encoding_and_validation::Journal;
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_stage::{StagedSnapshot, new_stage_container, stage_ignored_snapshot};
use crate::worktree_copy_stage_claim::{claim_stage, claim_stage_for, stage_claims_exist};
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

/// Creates a staged journal for a destination,
/// as an interrupted copy would have left it.
fn staged_journal(source: &Path, destination: &Path) -> PendingJournal {
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let snapshot: StagedSnapshot =
        stage_ignored_snapshot(&git_context(), source, new_stage_container(destination).expect("container").as_path(), &[], &CopySelection::Ignored).expect("staged");
    return create_journal_for(common.as_path(), destination, &snapshot, &dead_copy_owner()).expect("journal");
}

/// A repository's registrations decide whether a destination is registered:
/// the linked worktree it holds is,
/// and a directory no registration names is not,
/// whatever the destination's own `.git` file holds.
#[cfg(unix)]
#[test]
fn registration_answers_from_the_repository_registrations() {
    let root: PathBuf = fixture("recovery-registration");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    assert_eq!(
        destination_registration(common.as_path(), topic.as_path()),
        Registration::Registered
    );
    let absent: PathBuf = root.join("absent");
    assert_eq!(
        destination_registration(common.as_path(), absent.as_path()),
        Registration::Unregistered
    );
    let other: PathBuf = root.join("other");
    std::fs::create_dir(other.as_path()).expect("other");
    std::fs::write(other.join(".git"), b"gitdir: /nonexistent/admin\n").expect("dangling pointer");
    assert_eq!(
        destination_registration(common.as_path(), other.as_path()),
        Registration::Unregistered
    );
    let plain: PathBuf = root.join("plain");
    std::fs::create_dir(plain.as_path()).expect("plain");
    std::fs::write(plain.join(".git"), b"not a pointer\n").expect("plain pointer");
    assert_eq!(
        destination_registration(common.as_path(), plain.as_path()),
        Registration::Unregistered
    );
    remove(root.as_path());
}

/// Git strips every trailing line feed and carriage return from a `.git` pointer (`read_gitfile_raw` in its `setup.c`),
/// so a pointer written by a Windows tool still names the administrative directory it holds:
/// the pending copy of the registered worktree is resumed instead of discarded.
#[test]
fn a_pointer_that_ends_in_a_carriage_return_is_still_followed() {
    let root: PathBuf = fixture("recovery-pointer-crlf");
    let (source, topic) = linked_source(root.as_path());
    let pending: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    let pointer: PathBuf = topic.join(".git");
    let recorded: String = std::fs::read_to_string(pointer.as_path()).expect("the pointer Git wrote");
    std::fs::write(pointer.as_path(), format!("{}\r\n", recorded.trim_end_matches(['\n', '\r'])).as_bytes()).expect("the same pointer, ended the way a Windows tool ends a line");
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let report: RecoveryReport = recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 1, "{:?}", report.notices);
    assert!(report.notices.is_empty(), "{:?}", report.notices);
    assert_eq!(std::fs::read(topic.join("cache/data")).expect("resumed copy"), b"payload");
    assert!(read_pending_journals(common.as_path()).expect("pending").is_empty());
    assert!(!PathBuf::from(pending.journal.stage_container.as_path()).exists(), "a finished copy takes its stage with it");
    remove(root.as_path());
}

/// A destination whose pointer names a directory that is not there is still registered,
/// so its pending copy is kept and reported with its journal and stage whole.
#[test]
fn a_pointer_that_cannot_be_followed_keeps_the_pending_copy() {
    let root: PathBuf = fixture("recovery-pointer-unfollowable");
    let (source, topic) = linked_source(root.as_path());
    let pending: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    let stage: PathBuf = PathBuf::from(pending.journal.stage_container.as_path());
    std::fs::write(topic.join(".git"), b"gitdir: /nonexistent/admin\n").expect("a pointer to nothing");
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let report: RecoveryReport = recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 0);
    let notice: &String = report.notices.first().expect("one notice");
    assert!(notice.contains("still registers"), "{notice}");
    assert!(stage.exists(), "the stage stays for the copy that may still be registered");
    assert_eq!(read_pending_journals(common.as_path()).expect("pending").len(), 1, "the journal stays");
    assert!(!topic.join("cache/data").exists(), "nothing was written into the destination");
    remove(root.as_path());
}

/// A copy whose stage recorded the registration it was made for,
/// as an add's copy does.
fn staged_journal_of_a_bound_copy(source: &Path, topic: &Path) -> (PendingJournal, PathBuf) {
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let binding = crate::worktree_add_binding::bind_registration(common.as_path(), topic).expect("binding").binding;
    let pending: PendingJournal = staged_journal(source, topic);
    let container: PathBuf = PathBuf::from(pending.journal.stage_container.as_path());
    crate::worktree_copy_stage_binding::record_stage_binding(container.as_path(), &binding).expect("recorded");
    return (pending, container);
}

/// A rewrite killed after `gitdir: /repo/.git` leaves a pointer that resolves to an existing directory outside
/// `worktrees/`,
/// while the registration under `worktrees/` still names the destination:
/// recovery keeps the pending copy and reports the pointer it could not confirm.
#[test]
fn a_pointer_left_naming_the_repository_directory_keeps_the_pending_copy() {
    let root: PathBuf = fixture("recovery-pointer-repository");
    let (source, topic) = linked_source(root.as_path());
    let (_pending, stage) = staged_journal_of_a_bound_copy(source.as_path(), topic.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    // The path a rewrite reaches before the last component of the registration is written; it exists, and it is not
    // the registrations directory.
    std::fs::write(topic.join(".git"), format!("gitdir: {}\n", common.display()).as_bytes()).expect("a pointer cut before its last component");
    let report: RecoveryReport = recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 0);
    let notice: &String = report.notices.first().expect("one notice");
    assert!(notice.contains("still registers"), "{notice}");
    assert!(stage.exists(), "the stage stays for the copy of a worktree that is still registered");
    assert_eq!(read_pending_journals(common.as_path()).expect("pending").len(), 1, "the journal stays");
    assert!(!topic.join("cache/data").exists(), "nothing was written into the destination");
    remove(root.as_path());
}

/// A command that replaces a pointer by unlinking it and creating it again leaves the destination without one for a
/// moment,
/// and its registration still names it:
/// recovery keeps the pending copy instead of discarding it.
#[test]
fn a_pointer_missing_while_it_is_replaced_keeps_the_pending_copy() {
    let root: PathBuf = fixture("recovery-pointer-replacing");
    let (source, topic) = linked_source(root.as_path());
    let (_pending, stage) = staged_journal_of_a_bound_copy(source.as_path(), topic.as_path());
    std::fs::remove_file(topic.join(".git")).expect("the pointer is gone while it is replaced");
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let report: RecoveryReport = recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 0);
    let notice: &String = report.notices.first().expect("one notice");
    assert!(notice.contains("still registers"), "{notice}");
    assert!(stage.exists(), "the stage stays for the copy of a worktree that is still registered");
    assert_eq!(read_pending_journals(common.as_path()).expect("pending").len(), 1, "the journal stays");
    assert!(!topic.join("cache/data").exists(), "nothing was written into the destination");
    remove(root.as_path());
}

/// A pointer that names a symbolic link pointing at itself cannot be resolved at all,
/// and the registration under `worktrees/` still names the destination:
/// the recovery check answers instead of failing,
/// and the pending copy stays.
#[test]
fn a_pointer_naming_a_symlink_loop_keeps_the_pending_copy_and_still_answers() {
    let root: PathBuf = fixture("recovery-pointer-symlink-loop");
    let (source, topic) = linked_source(root.as_path());
    let (_pending, stage) = staged_journal_of_a_bound_copy(source.as_path(), topic.as_path());
    std::os::unix::fs::symlink("loop", topic.join("loop")).expect("a link to itself");
    std::fs::write(topic.join(".git"), b"gitdir: loop\n").expect("a pointer into the loop");
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let report: RecoveryReport = recover_worktree_copy_transactions(common.as_path()).expect("the recovery check answers instead of failing");
    assert_eq!(report.recovered, 0);
    let notice: &String = report.notices.first().expect("one notice");
    assert!(notice.contains("still registers"), "{notice}");
    assert!(stage.exists(), "the stage stays for the copy of a worktree that is still registered");
    assert_eq!(read_pending_journals(common.as_path()).expect("pending").len(), 1, "the journal stays");
    remove(root.as_path());
}

/// The registrations answer for a destination whose pointer names nothing,
/// names a directory outside the repository,
/// or names anything at all while the common directory has no linked worktree:
/// no registration names it,
/// so no copy is its.
#[test]
fn registration_answers_a_pointer_that_names_nothing_and_one_that_names_elsewhere() {
    let root: PathBuf = fixture("recovery-registration-pointers");
    let (source, _topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let naming_nothing: PathBuf = root.join("naming-nothing");
    std::fs::create_dir(naming_nothing.as_path()).expect("destination");
    std::fs::write(naming_nothing.join(".git"), b"gitdir: \r\n").expect("a pointer with no directory");
    assert_eq!(
        destination_registration(common.as_path(), naming_nothing.as_path()),
        Registration::Unregistered
    );
    let elsewhere: PathBuf = root.join("admin");
    std::fs::create_dir(elsewhere.as_path()).expect("a directory that is no administrative directory");
    let foreign: PathBuf = root.join("foreign");
    std::fs::create_dir(foreign.as_path()).expect("destination");
    std::fs::write(foreign.join(".git"), format!("gitdir: {}\n", elsewhere.display()).as_bytes()).expect("a pointer outside the repository");
    assert_eq!(
        destination_registration(common.as_path(), foreign.as_path()),
        Registration::Unregistered
    );
    let without_worktrees: PathBuf = root.join("common-without-worktrees");
    std::fs::create_dir(without_worktrees.as_path()).expect("a common directory with no worktrees");
    assert_eq!(
        destination_registration(without_worktrees.as_path(), foreign.as_path()),
        Registration::Unregistered
    );
    remove(root.as_path());
}

/// How long an answer that must not wait may take in a control.
const PATIENCE: std::time::Duration = std::time::Duration::from_secs(20);

/// A named pipe at `path`,
/// which blocks whoever opens it for reading until a writer comes.
fn make_pipe(path: &Path) {
    use std::os::unix::ffi::OsStrExt;
    let name = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("path text");
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let result: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(result, 0, "mkfifo {}", path.display());
}

/// A command killed while it rewrote the pointer of a registered worktree leaves the file empty,
/// and the worktree and its registration under the common directory are still there:
/// recovery is told nothing it can act on,
/// so it keeps the pending copy and reports the unknown state.
#[test]
fn a_pointer_left_empty_by_an_interrupted_rewrite_keeps_the_pending_copy() {
    let root: PathBuf = fixture("recovery-pointer-empty");
    let (source, topic) = linked_source(root.as_path());
    let pending: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    let stage: PathBuf = PathBuf::from(pending.journal.stage_container.as_path());
    std::fs::write(topic.join(".git"), b"").expect("a pointer cut off before its path");
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let report: RecoveryReport = recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 0);
    let notice: &String = report.notices.first().expect("one notice");
    assert!(notice.contains("still registers"), "{notice}");
    assert!(stage.exists(), "the stage stays for the copy that is still registered");
    assert_eq!(read_pending_journals(common.as_path()).expect("pending").len(), 1, "the journal stays");
    assert!(!topic.join("cache/data").exists(), "nothing was written into the destination");
    remove(root.as_path());
}

/// A `.git` that another command replaced with a named pipe must be answered without waiting:
/// the pending check runs before every forwarded command,
/// so a pipe with no writer would stick a command in an unrelated worktree.
#[test]
fn a_pointer_that_is_a_named_pipe_is_answered_without_waiting() {
    let root: PathBuf = fixture("recovery-pointer-pipe");
    let (source, _topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let destination: PathBuf = root.join("piped");
    std::fs::create_dir(destination.as_path()).expect("destination");
    make_pipe(destination.join(".git").as_path());
    let (sender, receiver) = std::sync::mpsc::channel::<Registration>();
    std::thread::spawn(move || {
        let answer: Registration = destination_registration(common.as_path(), destination.as_path());
        let _sent: Result<(), _> = sender.send(answer);
    });
    let answer: Registration = receiver
        .recv_timeout(PATIENCE)
        .expect("the registration is answered without waiting for a writer");
    assert_eq!(answer, Registration::Unregistered);
    remove(root.as_path());
}

/// A pointer an interrupted rewrite left half written proves nothing about the registration either:
/// the file holds part of a pointer,
/// so neither its absence nor a registration elsewhere can be read from it.
#[test]
fn registration_answers_a_pointer_left_half_written() {
    let root: PathBuf = fixture("recovery-registration-half-written");
    let (source, _topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    for (name, content) in [("cut-in-the-word", b"gitdir".as_slice()), ("cut-before-the-prefix", b"g".as_slice())] {
        let destination: PathBuf = root.join(name);
        std::fs::create_dir(destination.as_path()).expect("destination");
        std::fs::write(destination.join(".git"), content).expect("a pointer cut off mid-word");
        assert_eq!(
            destination_registration(common.as_path(), destination.as_path()),
            Registration::Unregistered,
            "{name}: no registration names this directory, so nothing is kept for it"
        );
    }
    remove(root.as_path());
}

/// A journal whose destination is no longer registered is discarded with the notice,
/// and its stage goes too.
#[cfg(unix)]
#[test]
fn unregistered_destination_is_discarded_with_notice() {
    let root: PathBuf = fixture("recovery-unregistered");
    let (source, topic) = linked_source(root.as_path());
    let pending: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    let stage: PathBuf = PathBuf::from(pending.journal.stage_container.as_path());
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
            "monochromatic-git-policy-cli: discarded an interrupted ignored-state copy for {}, which is no longer a registered worktree.",
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
    std::fs::remove_dir_all(pending.journal.stage_container.as_path()).expect("stage removed");
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let report: RecoveryReport =
        recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(
        report.notices,
        vec![format!(
            "monochromatic-git-policy-cli: discarded an interrupted ignored-state copy into {} because its private stage is missing; ignored files there may be incomplete.",
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

/// A copy from a main checkout that was killed after staging is resumed from its stage:
/// the dependencies,
/// the pnpm-style
/// links and the build output arrive,
/// and the caches the rule leaves out never do.
#[cfg(unix)]
#[test]
fn an_interrupted_main_checkout_copy_is_resumed_from_its_stage() {
    let root: PathBuf = fixture("recovery-resume-main");
    let (source, topic) = linked_source(root.as_path());
    std::fs::write(source.join(".gitignore"), b"cache/\nnode_modules/\ntarget/\n.pnpm-store/\n").expect("gitignore");
    for (path, bytes) in [
        ("node_modules/.pnpm/dep@1/node_modules/dep/index.js", b"module".as_slice()),
        ("node_modules/.cache/jiti/x", b"cache"),
        ("target/debug/deps/libx.rlib", b"rlib"),
        ("target/debug/incremental/x", b"incremental"),
        (".pnpm-store/v11/ab", b"store"),
    ] {
        std::fs::create_dir_all(source.join(path).parent().expect("parent")).expect("directories");
        std::fs::write(source.join(path), bytes).expect("file");
    }
    std::os::unix::fs::symlink(".pnpm/dep@1/node_modules/dep", source.join("node_modules/dep")).expect("link");
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let snapshot: StagedSnapshot = stage_ignored_snapshot(
        &git_context(),
        source.as_path(),
        new_stage_container(topic.as_path()).expect("container").as_path(),
        std::slice::from_ref(&topic),
        &CopySelection::MainCheckout,
    )
    .expect("staged");
    let pending: PendingJournal = create_journal_for(common.as_path(), topic.as_path(), &snapshot, &dead_copy_owner()).expect("journal");
    write_journal(pending.path.as_path(), &Journal { phase: Phase::Installing, ..pending.journal.clone() }).expect("installing phase");
    let report: RecoveryReport = recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 1);
    assert!(report.notices.is_empty(), "{:?}", report.notices);
    assert_eq!(std::fs::read(topic.join("node_modules/.pnpm/dep@1/node_modules/dep/index.js")).expect("dependency"), b"module");
    assert_eq!(
        std::fs::read_link(topic.join("node_modules/dep")).expect("link"),
        PathBuf::from(".pnpm/dep@1/node_modules/dep")
    );
    assert_eq!(std::fs::read(topic.join("target/debug/deps/libx.rlib")).expect("build"), b"rlib");
    assert!(topic.join("cache/data").exists(), "the earlier ignored directory arrives too");
    assert!(!topic.join("node_modules/.cache").exists());
    assert!(!topic.join("target/debug/incremental").exists());
    assert!(!topic.join(".pnpm-store").exists());
    assert!(read_pending_journals(common.as_path()).expect("pending").is_empty());
    remove(root.as_path());
}

/// A journal already marked complete only has its cleanup finished,
/// and counts as recovered.
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

/// The pending check does nothing,
/// and creates nothing,
/// when no journal is pending;
/// it leaves a live owner's journal whole
/// and recovers a dead owner's.
#[cfg(unix)]
#[test]
fn pending_check_leaves_a_live_journal_and_recovers_a_dead_one() {
    let root: PathBuf = fixture("recovery-pending-check");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    recover_pending_worktree_copies(common.as_path(), &mut Vec::new()).expect("nothing pending");
    assert!(!journal_root(common.as_path()).exists(), "the check creates no journal root");
    let snapshot: StagedSnapshot =
        stage_ignored_snapshot(&git_context(), source.as_path(), new_stage_container(topic.as_path()).expect("container").as_path(), &[], &CopySelection::Ignored).expect("staged");
    let live: PendingJournal = create_journal(common.as_path(), topic.as_path(), &snapshot).expect("journal of this process");
    let mut notices: Vec<u8> = Vec::new();
    recover_pending_worktree_copies(common.as_path(), &mut notices).expect("a live owner's journal is skipped");
    assert!(live.path.exists(), "a live owner keeps the journal for itself");
    assert!(notices.is_empty(), "{}", String::from_utf8_lossy(notices.as_slice()));
    assert!(Path::new(live.journal.stage_container.as_path()).exists(), "its stage stays");
    let dead: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    recover_pending_worktree_copies(common.as_path(), &mut notices).expect("a dead owner's journal is recovered");
    assert!(!dead.path.exists());
    assert!(live.path.exists(), "the live journal is still whole after the dead one went");
    remove(root.as_path());
}

/// Taking over a dead owner's journal is one rename:
/// the first command gets it,
/// and a second command that still holds the
/// old name gets nothing,
/// so a journal is never installed twice.
#[cfg(unix)]
#[test]
fn only_one_command_takes_over_a_dead_journal() {
    let root: PathBuf = fixture("recovery-single-winner");
    let (source, topic) = linked_source(root.as_path());
    let dead: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    let first = dead_copy_owner();
    let second = dead_copy_owner();
    let won: PendingJournal = adopt_journal(&dead, &first).expect("rename works").expect("the first command wins");
    assert_eq!(won.owner_token(), Some(first.token()));
    assert!(won.path.exists());
    assert!(!dead.path.exists());
    assert!(adopt_journal(&dead, &second).expect("rename works").is_none(), "the old name is gone");
    remove(root.as_path());
}

/// A stage left by a copy that died before its journal existed is removed by recovery,
/// with a notice and no count.
#[cfg(unix)]
#[test]
fn recovery_removes_a_stage_no_journal_names() {
    let root: PathBuf = fixture("recovery-abandoned-stage");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let claim = claim_stage_for(common.as_path(), topic.as_path(), &dead_copy_owner()).expect("claim");
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

/// A forwarded command that creates no worktree reaps the stage of a dead owner's claim,
/// and leaves alone the stage of a claim
/// whose owner still runs.
#[cfg(unix)]
#[test]
fn the_pending_check_reaps_abandoned_stages_only_without_a_live_owner() {
    let root: PathBuf = fixture("recovery-pending-claim");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let live = claim_stage(common.as_path(), topic.as_path()).expect("claim of this process");
    let dead = claim_stage_for(common.as_path(), topic.as_path(), &dead_copy_owner()).expect("claim of a dead process");
    for claim in [&live, &dead] {
        std::fs::create_dir(claim.container()).expect("stage");
        std::fs::set_permissions(claim.container(), std::fs::Permissions::from_mode(0o700)).expect("private");
    }
    let mut notices: Vec<u8> = Vec::new();
    recover_pending_worktree_copies(common.as_path(), &mut notices).expect("reaped");
    assert!(live.container().exists(), "a live copy may still be staging");
    assert!(!dead.container().exists(), "a dead owner's stage is reaped");
    assert!(stage_claims_exist(common.as_path()).expect("claims"), "the live claim stays");
    assert!(String::from_utf8_lossy(notices.as_slice()).contains("removed the stage"));
    remove(root.as_path());
}

/// The summary line counts transactions with the singular or plural noun,
/// after the notices.
#[test]
fn report_writes_notices_then_summary() {
    let mut out: Vec<u8> = Vec::new();
    report_recovered(
        &RecoveryReport {
            recovered: 1,
            notices: vec![String::from("monochromatic-git-policy-cli: notice.")],
        },
        &mut out,
    )
    .expect("written");
    assert_eq!(
        String::from_utf8(out).expect("utf8"),
        "monochromatic-git-policy-cli: notice.\nmonochromatic-git-policy-cli: recovered ignored-state copies for 1 worktree transaction.\n"
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
        "monochromatic-git-policy-cli: recovered ignored-state copies for 2 worktree transactions.\n"
    );
}

/// The bytes the TypeScript wrapper's own `writeJournal` wrote for a phase-installing journal,
/// with the base directory as `@BASE@`.
/// Generated by running `createWorktreeCopyJournal` and `writeJournal` from
/// `package/git-policy/cli/src/worktree-copy/journal.ts` against a scratch base directory;
/// that file was deleted on 2026-10-13 in `642b6d72cb`,
/// so `git show 642b6d72cb^` holds it.
const INCUMBENT_JOURNAL: &str = "{\"createdEntries\":[{\"device\":\"7\",\"inode\":\"42\",\"relativePath\":\"cache\",\"selected\":true}],\"intendedEntries\":[\"cache\"],\"destinationRoot\":\"@BASE@/topic\",\"phase\":\"installing\",\"selectedRoots\":[\"cache\"],\"sourceRoot\":\"@BASE@/source\",\"stageContainer\":\"@BASE@/.cli-git-worktree-copy-abc\",\"stageRoot\":\"@BASE@/.cli-git-worktree-copy-abc/payload\",\"version\":1}\n";

/// A journal the incumbent wrote is read by the native wrapper with the same fields,
/// and recovered:
/// its destination is not a worktree,
/// so it is discarded with the incumbent's notice and its stage goes too.
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
            "monochromatic-git-policy-cli: discarded an interrupted ignored-state copy for {}, which is no longer a registered worktree.",
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
///  while a registered worktree whose `.git` file cannot be read as a plain file is unconfirmed:
///  it is neither discarded nor a fault that stops the command that found it.
#[cfg(unix)]
#[test]
fn registration_tells_a_missing_destination_from_an_unreadable_pointer() {
    let root: PathBuf = std::fs::canonicalize(fixture("recovery-registration-faults")).expect("canonical");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let file: PathBuf = root.join("a-file");
    std::fs::write(file.as_path(), b"not a directory").expect("file");
    assert_eq!(
        destination_registration(common.as_path(), file.join("below").as_path()),
        Registration::Unregistered
    );
    std::fs::remove_file(topic.join(".git")).expect("the pointer goes");
    std::fs::create_dir(topic.join(".git")).expect("a directory where the pointer file belongs");
    assert_eq!(
        destination_registration(common.as_path(), topic.as_path()),
        Registration::Unconfirmed
    );
    remove(root.as_path());
}

/// A stage that cannot be examined for any reason other than being absent is a fault,
/// not a missing stage.
#[cfg(unix)]
#[test]
fn a_stage_that_cannot_be_examined_is_a_fault_and_not_a_missing_stage() {
    let root: PathBuf = fixture("recovery-stage-blocked");
    let (source, topic) = linked_source(root.as_path());
    let mut pending: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    let blocker: PathBuf = root.join("blocker");
    std::fs::write(blocker.as_path(), b"a file").expect("blocker");
    pending.journal.stage_container = blocker.join("stage");
    let failure: WorktreeCopyError = validate_stage(&pending).expect_err("the stage cannot be examined");
    assert!(
        failure.0.contains("could not validate worktree-copy recovery state"),
        "{}",
        failure.0
    );
    remove(root.as_path());
}

/// Commands that race for one dead journal at the same instant produce exactly one winner,
/// and the winner's journal is the
/// only one left under any name.
#[cfg(unix)]
#[test]
fn racing_commands_take_a_dead_journal_exactly_once() {
    const RACERS: usize = 4;
    let root: PathBuf = fixture("recovery-race");
    let (source, topic) = linked_source(root.as_path());
    let dead: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    let owners: Vec<_> = (0..RACERS).map(|_racer| { return dead_copy_owner(); }).collect();
    let start: std::sync::Barrier = std::sync::Barrier::new(RACERS);
    let wins: usize = std::thread::scope(|scope| {
        let handles: Vec<_> = owners
            .iter()
            .map(|owner| {
                return scope.spawn(|| {
                    start.wait();
                    return adopt_journal(&dead, owner).expect("rename works").is_some();
                });
            })
            .collect();
        return handles.into_iter().map(|handle| { return handle.join().expect("racer ends"); }).filter(|won| { return *won; }).count();
    });
    assert_eq!(wins, 1, "exactly one command takes the journal");
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    assert_eq!(read_pending_journals(common.as_path()).expect("pending").len(), 1, "one journal remains, under the winner's name");
    remove(root.as_path());
}

/// A journal whose owner is in another PID namespace and whose file is older than the patience is neither alive nor dead (owner,
/// 2026-11-04,
/// "Refuse and name it (Recommended)"):
/// recovery stops with an error that names the journal,
/// the owner,
/// its namespace and
/// the `rm`,
/// and nothing is taken over,
/// finished or removed:
/// the journal and its stage stay.
#[cfg(unix)]
#[test]
fn a_journal_of_another_namespace_older_than_the_patience_stops_recovery_and_changes_nothing() {
    let root: PathBuf = fixture("recovery-foreign-expired");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let snapshot: StagedSnapshot =
        stage_ignored_snapshot(&git_context(), source.as_path(), new_stage_container(topic.as_path()).expect("container").as_path(), &[], &CopySelection::Ignored).expect("staged");
    let foreign = crate::worktree_copy_owner::CopyOwner { pid: 31337, birth_identity: String::from("linux:1"), pid_namespace: Some(String::from("pid:[1]")) };
    let pending: PendingJournal = create_journal_for(common.as_path(), topic.as_path(), &snapshot, &foreign).expect("journal");
    std::fs::File::options().write(true).open(pending.path.as_path()).expect("open").set_modified(std::time::SystemTime::now() - crate::pid_namespace::FOREIGN_OWNER_PATIENCE - std::time::Duration::from_secs(300)).expect("age");
    let stage: PathBuf = PathBuf::from(pending.journal.stage_container.as_path());
    let refused = recover_worktree_copy_transactions(common.as_path()).expect_err("an expired foreign journal stops recovery");
    assert!(refused.0.contains(pending.path.to_string_lossy().as_ref()) && refused.0.contains("31337") && refused.0.contains("pid:[1]") && refused.0.contains("rm -- "), "{}", refused.0);
    assert!(pending.path.exists() && stage.exists(), "the journal and its stage are untouched");
    assert_eq!(read_pending_journals(common.as_path()).expect("pending").len(), 1);
    remove(root.as_path());
}

/// Pending journals `[dead A, expired-foreign B]`:
/// the command refuses before it adopts,
/// finishes or removes A (sol-1 round 3,
/// P1 C).
#[cfg(unix)]
#[test]
fn an_expired_foreign_journal_after_a_dead_one_stops_recovery_before_the_dead_one_is_touched() {
    let root: PathBuf = fixture("recovery-foreign-after-dead");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let foreign = crate::worktree_copy_owner::CopyOwner { pid: 31337, birth_identity: String::from("linux:1"), pid_namespace: Some(String::from("pid:[1]")) };
    let other: PathBuf = root.join("other");
    // Journals are read in name order, which holds random identifiers: journals are made until the dead one sorts before the foreign one.
    let (dead, expired): (PendingJournal, PendingJournal) = loop {
        let dead: PendingJournal = staged_journal(source.as_path(), topic.as_path());
        let snapshot: StagedSnapshot =
            stage_ignored_snapshot(&git_context(), source.as_path(), new_stage_container(other.as_path()).expect("container").as_path(), &[], &CopySelection::Ignored).expect("staged");
        let expired: PendingJournal = create_journal_for(common.as_path(), other.as_path(), &snapshot, &foreign).expect("journal");
        if dead.path.file_name() < expired.path.file_name() {
            break (dead, expired);
        }
        std::fs::remove_file(dead.path.as_path()).expect("discard the pair");
        std::fs::remove_file(expired.path.as_path()).expect("discard the pair");
    };
    std::fs::File::options().write(true).open(expired.path.as_path()).expect("open").set_modified(std::time::SystemTime::now() - crate::pid_namespace::FOREIGN_OWNER_PATIENCE - std::time::Duration::from_secs(300)).expect("age");
    let dead_stage: PathBuf = PathBuf::from(dead.journal.stage_container.as_path());
    let refused = recover_worktree_copy_transactions(common.as_path()).expect_err("an expired foreign journal stops recovery");
    assert!(refused.0.contains("31337"), "{}", refused.0);
    assert!(dead.path.exists() && dead_stage.exists(), "the dead owner's journal and stage are untouched: nothing changed");
    assert!(expired.path.exists());
    remove(root.as_path());
}

/// A recoverable dead journal and an expired foreign stage claim:
/// recovery refuses before it adopts the journal (sol-1 round 4:
/// the
/// stage owners are judged before the journals are touched,
/// not only before the sweep).
/// Planted:
/// judging the stages after the journal
/// loop recovers the journal first and fails this control.
#[cfg(unix)]
#[test]
fn an_expired_foreign_stage_claim_stops_recovery_before_a_dead_journal_is_touched() {
    let root: PathBuf = fixture("recovery-foreign-stage-with-dead-journal");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let dead: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    let dead_stage: PathBuf = PathBuf::from(dead.journal.stage_container.as_path());
    let foreign = crate::worktree_copy_owner::CopyOwner { pid: 31337, birth_identity: String::from("linux:1"), pid_namespace: Some(String::from("pid:[1]")) };
    let _claim = claim_stage_for(common.as_path(), root.join("elsewhere").as_path(), &foreign).expect("a foreign claim");
    let claim_path: PathBuf = std::fs::read_dir(crate::worktree_copy_journal::journal_root(common.as_path())).expect("the journal root").flatten().map(|entry| { return entry.path(); }).find(|path| { return path.extension().is_some_and(|extension| { return extension == "stage"; }); }).expect("the claim file");
    std::fs::File::options().write(true).open(claim_path.as_path()).expect("open").set_modified(std::time::SystemTime::now() - crate::pid_namespace::FOREIGN_OWNER_PATIENCE - std::time::Duration::from_secs(300)).expect("age");
    let refused = recover_worktree_copy_transactions(common.as_path()).expect_err("an expired foreign claim stops recovery");
    assert!(refused.0.contains("31337"), "{}", refused.0);
    assert!(dead.path.exists() && dead_stage.exists(), "the dead journal and its stage are untouched");
    assert_eq!(read_pending_journals(common.as_path()).expect("pending").len(), 1);
    remove(root.as_path());
}

/// The registration a stage records is the one the copy was made for:
/// a destination another registration holds by the time recovery runs receives nothing,
/// and the notice names both registrations.
#[cfg(unix)]
#[test]
fn a_recorded_registration_that_is_gone_is_discarded_with_a_notice() {
    let root: PathBuf = fixture("recovery-registration-replaced");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let binding = crate::worktree_add_binding::bind_registration(common.as_path(), topic.as_path()).expect("binding").binding;
    let pending: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    let container: PathBuf = PathBuf::from(pending.journal.stage_container.as_path());
    crate::worktree_copy_stage_binding::record_stage_binding(container.as_path(), &binding).expect("recorded");
    // Another command removes this worktree and registers another one at the same path, whose checkout has not run.
    git(
        source.as_path(),
        &[
            "worktree",
            "remove",
            "--force",
            topic.to_str().expect("utf8"),
        ],
    );
    git(
        source.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            "-b",
            "second",
            topic.to_str().expect("utf8"),
        ],
    );
    let report: RecoveryReport = recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 0);
    let notice: &String = report.notices.first().expect("one notice");
    assert!(notice.contains(binding.admin.to_str().expect("utf8")), "the registration the copy was made for is named: {notice}");
    assert!(notice.contains("is registered by"), "{notice}");
    assert!(!topic.join("cache").exists(), "the replacement registration received nothing");
    assert!(!topic.join("tracked.txt").exists(), "the replacement registration's own checkout never ran");
    assert!(
        read_pending_journals(common.as_path())
            .expect("pending")
            .is_empty()
    );
    remove(root.as_path());
}

/// A recorded registration that still holds is resumed,
/// so the check never costs a copy that is still the right one.
#[cfg(unix)]
#[test]
fn a_recorded_registration_that_still_holds_is_resumed() {
    let root: PathBuf = fixture("recovery-registration-holds");
    let (source, topic) = linked_source(root.as_path());
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let binding = crate::worktree_add_binding::bind_registration(common.as_path(), topic.as_path()).expect("binding").binding;
    let pending: PendingJournal = staged_journal(source.as_path(), topic.as_path());
    let container: PathBuf = PathBuf::from(pending.journal.stage_container.as_path());
    crate::worktree_copy_stage_binding::record_stage_binding(container.as_path(), &binding).expect("recorded");
    let report: RecoveryReport = recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 1);
    assert!(report.notices.is_empty(), "{:?}", report.notices);
    assert_eq!(std::fs::read(topic.join("cache/data")).expect("resumed copy"), b"payload");
    remove(root.as_path());
}

/// A registration whose destination path holds bytes that are not text is read as registered:
/// the pointer Git wrote names the administrative directory with those bytes.
#[test]
fn a_registration_under_a_path_that_is_not_text_is_read() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("recovery-registration-bytes");
    // The repository itself sits under a name that is not text, so the pointer Git writes in the destination holds those bytes.
    let repository_parent: PathBuf = root.join(std::ffi::OsStr::from_bytes(b"repo\xff"));
    std::fs::create_dir_all(repository_parent.as_path()).expect("repository parent");
    let source: PathBuf = repository(repository_parent.as_path(), "source");
    let parent: PathBuf = root.join(std::ffi::OsStr::from_bytes(b"dst\xfe"));
    std::fs::create_dir_all(parent.as_path()).expect("destination parent");
    let destination: PathBuf = parent.join("topic");
    let arguments: Vec<OsString> = ["worktree", "add", "--quiet", "-b", "topic"]
        .iter()
        .map(OsString::from)
        .chain([destination.clone().into_os_string()])
        .collect();
    git(source.as_path(), arguments.as_slice());
    let canonical: PathBuf = std::fs::canonicalize(destination).expect("canonical destination");
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    assert_eq!(
        destination_registration(common.as_path(), canonical.as_path()),
        Registration::Registered,
        "the registration is read from the pointer Git wrote"
    );
    remove(root.as_path());
}
