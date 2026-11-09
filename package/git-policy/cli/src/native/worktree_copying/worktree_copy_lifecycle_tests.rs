//! Lifecycle checks against real Git:
//! applicability,
//! bypasses,
//! settlement,
//! the summary line and exit statuses.

use super::*;
use crate::add_timing::{begin_work, end_work};
use crate::side_by_side_threads::in_thread_scope;
use crate::test_support::{fixture, git, git_context, git_output, remove, repository};
use crate::worktree_copy_after_lock::TrashBesideRepair;
use crate::worktree_copy_journal::{create_journal_for, journal_root, read_pending_journals};
use crate::test_support::dead_copy_owner;
use std::process::Output;

/// A function-like value with the right shape is a Git runner in these controls:
/// the shipped caller writes a small struct that holds the state.
impl<Run: FnMut(&GitStep<'_>) -> Result<GitStatus, WorktreeCopyError>> GitRunner for Run {
    fn run(&mut self, step: &GitStep<'_>) -> Result<GitStatus, WorktreeCopyError> {
        return self(step);
    }
}

/// A repository with an ignored `cache` directory,
/// returned with its canonical common directory.
fn source(root: &Path) -> (PathBuf, PathBuf) {
    let main: PathBuf = repository(root, "main");
    std::fs::write(main.join(".gitignore"), b"cache/\n").expect("gitignore");
    std::fs::create_dir_all(main.join("cache")).expect("cache");
    std::fs::write(main.join("cache/data"), b"payload").expect("data");
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("common");
    return (std::fs::canonicalize(main).expect("canonical main"), common);
}

/// Runs real Git with the given arguments in a directory and reports its status.
fn run_real(directory: &Path, arguments: &[String]) -> GitStatus {
    let output: Output = git_output(directory, arguments);
    return GitStatus {
        code: output.status.code(),
    };
}

/// The identity of the main worktree of a source.
fn main_identity(main: &Path, common: &Path) -> WorktreeIdentity {
    return WorktreeIdentity::MainWorktree {
        common_dir: common.to_path_buf(),
        git_dir: common.to_path_buf(),
        worktree_root: main.to_path_buf(),
    };
}

/// The summary line names the count,
/// the source,
/// and the destinations,
/// with the incumbent's nouns.
#[test]
fn summary_line_uses_exact_nouns() {
    assert_eq!(
        summary_line(1, 1, Some(Path::new("/repo"))),
        "monochromatic-git-policy-cli: copied 1 ignored filesystem entry from \"/repo\" into 1 new worktree.\n"
    );
    assert_eq!(
        summary_line(3, 2, None),
        "monochromatic-git-policy-cli: copied 3 ignored filesystem entries from bare repository with an empty source set into 2 new worktrees.\n"
    );
}

/// The exit status follows the incumbent:
/// a copy-only failure is 2,
/// Git's own status is kept,
/// and a signal is 1.
#[test]
fn exit_status_follows_the_settlement_table() {
    let ok: GitStatus = GitStatus { code: Some(0) };
    let failed: GitStatus = GitStatus { code: Some(5) };
    let signal: GitStatus = GitStatus { code: None };
    assert_eq!(
        exit_code(&Settlement {
            git: ok,
            copy_failure: None
        }),
        0
    );
    assert_eq!(
        exit_code(&Settlement {
            git: failed,
            copy_failure: None
        }),
        5
    );
    assert_eq!(
        exit_code(&Settlement {
            git: signal,
            copy_failure: None
        }),
        1
    );
    let copy_failure: WorktreeCopyError = WorktreeCopyError(String::from("copy failed"));
    assert_eq!(
        exit_code(&Settlement {
            git: ok,
            copy_failure: Some(copy_failure.clone())
        }),
        2
    );
    assert_eq!(
        exit_code(&Settlement {
            git: failed,
            copy_failure: Some(copy_failure.clone())
        }),
        5
    );
    assert_eq!(
        exit_code(&Settlement {
            git: signal,
            copy_failure: Some(copy_failure)
        }),
        1
    );
}

/// A worktree added from a linked worktree gets its ignored files and the summary line,
/// and takes no lock.
#[cfg(unix)]
#[test]
fn added_worktree_receives_ignored_state_without_a_lock() {
    let root: PathBuf = fixture("lifecycle-add");
    let (main, common) = source(root.as_path());
    std::fs::write(common.join("info/exclude"), b"cache/\n").expect("exclude");
    let source_root: PathBuf = root.join("source");
    git(
        main.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "source",
            source_root.to_str().expect("utf8"),
        ],
    );
    let source_canonical: PathBuf = std::fs::canonicalize(source_root).expect("canonical source");
    std::fs::create_dir_all(source_canonical.join("cache")).expect("cache");
    std::fs::write(source_canonical.join("cache/data"), b"payload").expect("data");
    let topic: PathBuf = root.join("topic");
    let identity: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: common.clone(),
        git_dir: common.join("worktrees").join("source"),
        worktree_root: source_canonical.clone(),
    };
    let mut stderr: Vec<u8> = Vec::new();
    let args: Vec<String> = vec![
        String::from("worktree"),
        String::from("add"),
        String::from("--quiet"),
        String::from("-b"),
        String::from("topic"),
        topic.to_string_lossy().into_owned(),
    ];
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: identity.clone(),
            effective_cwd: source_canonical.clone(),
            creates: true,
            destination: Some(topic.clone()),
            opted_out: false,
            short_circuit: false,
            context: git_context(),
            add_plan: None,
            repair: crate::package_repair::RepairSwitch::default(),
        },
        &mut |_step: &GitStep<'_>| return Ok(run_real(source_canonical.as_path(), args.as_slice())),
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0);
    assert_eq!(
        std::fs::read(topic.join("cache/data")).expect("copied"),
        b"payload"
    );
    assert_eq!(
        String::from_utf8(stderr).expect("utf8"),
        format!(
            "monochromatic-git-policy-cli: copied 2 ignored filesystem entries from {} into 1 new worktree.\n",
            quote(source_canonical.to_string_lossy().as_ref())
        )
    );
    assert!(!journal_root(common.as_path()).join("settlement.lock").exists(), "no lock is ever taken");
    assert!(
        read_pending_journals(common.as_path())
            .expect("pending")
            .is_empty()
    );
    remove(root.as_path());
}

/// The opt-out flag skips synchronization and recovery,
/// and the flag never reaches the runner.
#[cfg(unix)]
#[test]
fn opted_out_invocation_skips_synchronization() {
    let root: PathBuf = fixture("lifecycle-opt-out");
    let (main, common) = source(root.as_path());
    let topic: PathBuf = root.join("topic");
    let identity: WorktreeIdentity = main_identity(main.as_path(), common.as_path());
    let mut runs: usize = 0;
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: identity.clone(),
            effective_cwd: main.clone(),
            creates: true,
            destination: Some(topic.clone()),
            opted_out: true,
            short_circuit: false,
            context: git_context(),
            add_plan: None,
            repair: crate::package_repair::RepairSwitch::default(),
        },
        &mut |_step: &GitStep<'_>| {
            runs += 1;
            return Ok(run_real(
                main.as_path(),
                &[
                    String::from("worktree"),
                    String::from("add"),
                    String::from("--quiet"),
                    String::from("-b"),
                    String::from("topic"),
                    topic.to_string_lossy().into_owned(),
                ],
            ));
        },
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0);
    assert_eq!(runs, 1, "Git runs once and untouched");
    assert!(!topic.join("cache").exists());
    assert!(stderr.is_empty());
    remove(root.as_path());
}

/// A main worktree command that creates no worktree runs without any settlement.
#[test]
fn main_worktree_command_that_creates_nothing_is_not_settled() {
    let root: PathBuf = fixture("lifecycle-main");
    let (main, common) = source(root.as_path());
    let identity: WorktreeIdentity = main_identity(main.as_path(), common.as_path());
    let mut runs: usize = 0;
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: identity.clone(),
            effective_cwd: main.clone(),
            creates: false,
            destination: None,
            opted_out: false,
            short_circuit: false,
            context: git_context(),
            add_plan: None,
            repair: crate::package_repair::RepairSwitch::default(),
        },
        &mut |_step: &GitStep<'_>| {
            runs += 1;
            return Ok(GitStatus { code: Some(7) });
        },
        &mut Vec::new(),
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 7);
    assert_eq!(runs, 1);
    assert!(!journal_root(common.as_path()).exists(), "nothing is created in the repository");
    remove(root.as_path());
}

/// A command that creates no worktrees runs once,
/// untouched,
/// and creates nothing in the repository.
#[cfg(unix)]
#[test]
fn non_worktree_command_runs_without_settlement() {
    let root: PathBuf = fixture("lifecycle-status");
    let (main, common) = source(root.as_path());
    let identity: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: common.clone(),
        git_dir: common.clone(),
        worktree_root: main.clone(),
    };
    let mut runs: usize = 0;
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: identity.clone(),
            effective_cwd: main.clone(),
            creates: false,
            destination: None,
            opted_out: false,
            short_circuit: false,
            context: git_context(),
            add_plan: None,
            repair: crate::package_repair::RepairSwitch::default(),
        },
        &mut |_step: &GitStep<'_>| {
            runs += 1;
            return Ok(GitStatus { code: Some(0) });
        },
        &mut Vec::new(),
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0);
    assert_eq!(runs, 1);
    assert!(!journal_root(common.as_path()).exists(), "nothing is created in the repository");
    remove(root.as_path());
}

/// A staged journal of a registered worktree,
/// written for `owner`,
/// as a copy that stopped after staging leaves it.
#[cfg(unix)]
fn journal_for(main: &Path, common: &Path, destination: &Path, owner: &crate::worktree_copy_owner::CopyOwner) -> crate::worktree_copy_journal_encoding_and_validation::PendingJournal {
    use crate::worktree_copy_selection::CopySelection;
    use crate::worktree_copy_stage::{new_stage_container, stage_ignored_snapshot};
    let snapshot = stage_ignored_snapshot(
        &git_context(),
        main,
        new_stage_container(destination).expect("container").as_path(),
        &[],
        &CopySelection::MainCheckout,
    )
    .expect("staged");
    return create_journal_for(common, destination, &snapshot, owner).expect("journal");
}

/// The invocation of a creating `git worktree add` from the main checkout,
/// naming `destination`.
fn creation_from(main: &Path, common: &Path, destination: Option<&Path>) -> Invocation {
    return Invocation {
        identity: main_identity(main, common),
        effective_cwd: main.to_path_buf(),
        creates: true,
        destination: destination.map(Path::to_path_buf),
        opted_out: false,
        short_circuit: false,
        context: git_context(),
        add_plan: None,
        repair: crate::package_repair::RepairSwitch::default(),
    };
}

/// The disposal of a snapshot's trash runs and is joined before the add ends:
/// what it did is recorded as work of its own,
/// the wait for it is a step of its own,
/// and a disposal that fails is reported instead of failing the add.
#[cfg(target_os = "linux")]
#[test]
fn a_snapshot_trash_is_emptied_before_the_add_ends_and_a_failure_of_it_is_not_the_adds() {
    use crate::fs_beneath::ObjectId;
    use crate::worktree_trash::TrashManifest;
    use std::sync::atomic::{AtomicUsize, Ordering};
    /// How many times the stand-in emptied the trash.
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    /// The stand-in records the work as the shipped job does,
    /// and leaves the trash where it is.
    fn refuse(_state: (&GitContext, &PendingTrash, &crate::shared_notice_stream::NoticeStream<'_>)) -> Result<(), String> {
        let began: std::time::Instant = begin_work();
        CALLS.fetch_add(1, Ordering::SeqCst);
        end_work("disposeTrash", began);
        return Err(String::from("the trash stays"));
    }
    let root: PathBuf = fixture("lifecycle-trash-disposal");
    let (main, common) = source(root.as_path());
    let pending: PendingTrash = PendingTrash {
        destination: main.clone(),
        git_dir: common.clone(),
        manifest: TrashManifest { name: "ab".repeat(20), trash: ObjectId { device: 7, inode: 1, birth: None }, entries: Vec::new() },
        quiet: true,
    };
    let after: AfterLock = AfterLock { source_root: None, created: Vec::new(), trash: Some(pending.clone()) };
    let invocation: Invocation = creation_from(main.as_path(), common.as_path(), Some(main.as_path()));
    let mut stderr: Vec<u8> = Vec::new();
    crate::add_timing::begin(common.clone());
    {
        let stream: crate::shared_notice_stream::NoticeStream<'_> = std::sync::Mutex::new(&mut stderr);
        in_thread_scope(TrashBesideRepair { invocation: &invocation, after: &after, pending: &pending, stream: &stream, dispose: refuse });
    }
    crate::add_timing::finish(0);
    assert_eq!(CALLS.load(Ordering::SeqCst), 1, "the trash was emptied once, and the add waited for it");
    assert_eq!(stderr, Vec::<u8>::new(), "the person's stream carries the repair, and a disposal that fails says so in the log");
    let written: String = std::fs::read_to_string(common.join(crate::add_timing::ADD_TIMING_FILE)).expect("the add's record");
    let record: serde_json::Value = serde_json::from_str(written.trim_end()).expect("the record is one JSON object");
    assert_eq!(record["work"]["disposeTrash"]["calls"], 1, "{record}");
    assert_eq!(record["steps"]["disposeTrashWait"]["calls"], 1, "the wait is a step of its own: {record}");
    remove(root.as_path());
}

/// The arguments of a quiet `git worktree add -b <name> <destination>`.
fn add_arguments(name: &str, destination: &Path) -> Vec<String> {
    return ["worktree", "add", "--quiet", "-b", name].iter().map(|word| { return (*word).to_string(); }).chain([destination.to_string_lossy().into_owned()]).collect();
}

/// The arguments of a quiet `git worktree add --no-checkout -b <name> <destination>`.
fn add_without_checkout(name: &str, destination: &Path) -> Vec<String> {
    return ["worktree", "add", "--quiet", "--no-checkout", "-b", name].iter().map(|word| { return (*word).to_string(); }).chain([destination.to_string_lossy().into_owned()]).collect();
}

/// The note a snapshot of the main checkout writes reaches the stream the caller handed in,
/// where the rest of the add's report goes.
/// Why:
///   A caller that hands in its own stream reads the whole report;
/// a line written to the process's standard error beside it is one that caller never sees.
#[test]
fn btrfs_a_main_checkout_snapshot_notes_on_the_stream_the_caller_handed_in() {
    let Some(root) = crate::fs_btrfs_test_support::btrfs_fixture("lifecycle-main-note") else { return };
    let main: PathBuf = crate::worktree_ladder_test_support::subvolume_repository(root.as_path());
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("common");
    let topic: PathBuf = root.join("topic");
    let mut invocation: Invocation = creation_from(main.as_path(), common.as_path(), Some(topic.as_path()));
    invocation.add_plan = Some(crate::worktree_ladder_test_support::plan_to(topic.clone()));
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &invocation,
        &mut |_step: &GitStep<'_>| return Ok(run_real(main.as_path(), add_without_checkout("topic", topic.as_path()).as_slice())),
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0, "{:?}", settlement.copy_failure);
    let said: String = String::from_utf8(stderr).expect("utf8");
    assert!(said.contains(crate::worktree_trash::MAIN_SNAPSHOT_NOTE), "the note reaches the stream the caller handed in: {said:?}");
    remove(root.as_path());
}

/// Another command registers a worktree while this one's Git runs:
/// only the destination this command named is copied into,
/// so two concurrent adds each get exactly their own copy.
#[cfg(unix)]
#[test]
fn a_creation_copies_only_into_the_destination_it_named() {
    let root: PathBuf = fixture("lifecycle-own-destination");
    let (main, common) = source(root.as_path());
    let topic: PathBuf = root.join("topic");
    let other: PathBuf = root.join("other");
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(topic.as_path())),
        &mut |_step: &GitStep<'_>| {
            // The other command's registration lands inside this command's Git run, as a concurrent add's would.
            run_real(main.as_path(), add_arguments("other", other.as_path()).as_slice());
            return Ok(run_real(main.as_path(), add_arguments("topic", topic.as_path()).as_slice()));
        },
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0, "{:?}", settlement.copy_failure);
    assert_eq!(std::fs::read(topic.join("cache/data")).expect("own copy"), b"payload");
    assert!(!other.join("cache").exists(), "another command's worktree is not this command's to fill");
    assert!(String::from_utf8(stderr).expect("utf8").ends_with(" into 1 new worktree.\n"));
    assert_nothing_left(root.as_path(), common.as_path());
    remove(root.as_path());
}

/// A journal whose owner still runs belongs to a copy in progress:
/// a creation neither waits for it nor recovers it.
#[cfg(unix)]
#[test]
fn a_creation_leaves_a_live_journal_alone_and_does_not_wait() {
    let root: PathBuf = fixture("lifecycle-live-journal");
    let (main, common) = source(root.as_path());
    let older: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("older");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "older", older.to_str().expect("utf8")]);
    let live = journal_for(main.as_path(), common.as_path(), older.as_path(), &crate::worktree_copy_owner::current_copy_owner().expect("owner"));
    let topic: PathBuf = root.join("topic");
    let mut stderr: Vec<u8> = Vec::new();
    let started: std::time::Instant = std::time::Instant::now();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(topic.as_path())),
        &mut |_step: &GitStep<'_>| return Ok(run_real(main.as_path(), add_arguments("topic", topic.as_path()).as_slice())),
        &mut stderr,
    )
    .expect("settled");
    assert!(started.elapsed() < std::time::Duration::from_secs(30), "the creation must not wait for the live copy");
    assert_eq!(exit_code(&settlement), 0, "{:?}", settlement.copy_failure);
    assert!(live.path.exists(), "the live journal is whole");
    assert!(Path::new(live.journal.stage_container.as_str()).exists(), "its stage is whole");
    assert!(!older.join("cache").exists(), "its copy is not installed by anyone else");
    assert!(!String::from_utf8(stderr).expect("utf8").contains("recovered"));
    remove(root.as_path());
}

/// A source whose own ignored-state copy still runs is no source:
/// the worktree is made,
/// nothing is staged from the half-filled source,
/// and the person is told why and how to have the copy made later.
#[cfg(unix)]
#[test]
fn a_creation_from_a_source_still_being_filled_copies_nothing_and_says_so() {
    let root: PathBuf = fixture("lifecycle-source-still-filling");
    let (main, common) = source(root.as_path());
    // The repository's own exclusion file, so every worktree of it ignores `cache` whether or not it holds the .gitignore file.
    std::fs::write(common.join("info/exclude"), b"cache/\n").expect("exclude");
    let older: PathBuf = root.join("older");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "older", older.to_str().expect("utf8")]);
    let older_canonical: PathBuf = std::fs::canonicalize(older.as_path()).expect("canonical older");
    // The state a copy in progress leaves: one of the source's ignored entries is there and the rest are not.
    std::fs::create_dir_all(older_canonical.join("cache")).expect("half-filled cache");
    std::fs::write(older_canonical.join("cache/half"), b"partial").expect("half");
    let live = journal_for(main.as_path(), common.as_path(), older_canonical.as_path(), &crate::worktree_copy_owner::current_copy_owner().expect("owner"));
    let topic: PathBuf = root.join("topic");
    let identity: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: common.clone(),
        git_dir: common.join("worktrees").join("older"),
        worktree_root: older_canonical.clone(),
    };
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: identity.clone(),
            effective_cwd: older_canonical.clone(),
            creates: true,
            destination: Some(topic.clone()),
            opted_out: false,
            short_circuit: false,
            context: git_context(),
            add_plan: None,
            repair: crate::package_repair::RepairSwitch::default(),
        },
        &mut |_step: &GitStep<'_>| return Ok(run_real(older_canonical.as_path(), add_arguments("topic", topic.as_path()).as_slice())),
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0, "the add itself still succeeds: {:?}", settlement.copy_failure);
    assert!(settlement.copy_failure.is_none(), "a source that is busy is not the add's failure");
    assert!(topic.join(".git").exists(), "the worktree is made");
    assert!(!topic.join("cache").exists(), "the half-filled ignored state is not copied");
    assert_eq!(
        String::from_utf8(stderr).expect("utf8"),
        format!(
            "monochromatic-git-policy-cli: {} was made, but it received no ignored-state copy, because {} is itself being filled by another `git worktree add`, so its ignored files are only partly there. Once that add has finished, `git worktree remove {}` and run this `git worktree add` line again to have the ignored state copied; `git worktree remove` leaves a worktree that holds changes alone, so save them first.\n",
            quote(topic.to_string_lossy().as_ref()),
            quote(older_canonical.to_string_lossy().as_ref()),
            quote(topic.to_string_lossy().as_ref())
        )
    );
    assert!(live.path.exists(), "the running copy's journal is whole");
    assert!(read_pending_journals(common.as_path()).expect("pending").len() == 1, "nothing of this command's own stays pending");
    remove(root.as_path());
}

/// A journal whose owner is dead is recovered before the creation runs Git,
/// and reported.
#[cfg(unix)]
#[test]
fn a_creation_recovers_a_dead_journal_first() {
    let root: PathBuf = fixture("lifecycle-dead-journal");
    let (main, common) = source(root.as_path());
    let older: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("older");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "older", older.to_str().expect("utf8")]);
    let dead = journal_for(main.as_path(), common.as_path(), older.as_path(), &dead_copy_owner());
    let topic: PathBuf = root.join("topic");
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(topic.as_path())),
        &mut |_step: &GitStep<'_>| return Ok(run_real(main.as_path(), add_arguments("topic", topic.as_path()).as_slice())),
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0, "{:?}", settlement.copy_failure);
    let text: String = String::from_utf8(stderr).expect("utf8");
    assert!(text.starts_with("monochromatic-git-policy-cli: recovered ignored-state copies for 1 worktree transaction.\n"), "{text}");
    assert!(!dead.path.exists());
    assert_eq!(std::fs::read(older.join("cache/data")).expect("recovered copy"), b"payload");
    assert_eq!(std::fs::read(topic.join("cache/data")).expect("own copy"), b"payload");
    remove(root.as_path());
}

/// A creation whose command line names no readable destination copies nothing and says so once Git succeeded.
#[cfg(unix)]
#[test]
fn a_creation_with_an_unread_destination_copies_nothing_and_says_so() {
    let root: PathBuf = fixture("lifecycle-unread-destination");
    let (main, common) = source(root.as_path());
    let topic: PathBuf = root.join("topic");
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), None),
        &mut |_step: &GitStep<'_>| return Ok(run_real(main.as_path(), add_arguments("topic", topic.as_path()).as_slice())),
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0);
    assert_eq!(String::from_utf8(stderr).expect("utf8"), UNKNOWN_DESTINATION_NOTICE);
    assert!(!topic.join("cache").exists());
    // Git's own failure is its own message: the notice is only for a worktree that was made.
    let mut failed_stderr: Vec<u8> = Vec::new();
    let failed: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), None),
        &mut |_step: &GitStep<'_>| return Ok(GitStatus { code: Some(128) }),
        &mut failed_stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&failed), 128);
    assert!(failed_stderr.is_empty());
    remove(root.as_path());
}

/// A destination that was a registered worktree before Git ran is not copied into again when Git refuses the add.
#[cfg(unix)]
#[test]
fn a_destination_registered_before_is_not_copied_into_again() {
    let root: PathBuf = fixture("lifecycle-registered-before");
    let (main, common) = source(root.as_path());
    let topic: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "topic", topic.to_str().expect("utf8")]);
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(topic.as_path())),
        &mut |_step: &GitStep<'_>| return Ok(run_real(main.as_path(), add_arguments("again", topic.as_path()).as_slice())),
        &mut stderr,
    )
    .expect("settled");
    assert_ne!(exit_code(&settlement), 0, "Git refuses a destination that exists");
    assert!(stderr.is_empty(), "{}", String::from_utf8_lossy(stderr.as_slice()));
    assert!(!topic.join("cache").exists());
    remove(root.as_path());
}

/// Git failing after it registered the worktree keeps Git's status and still copies the ignored state.
#[cfg(unix)]
#[test]
fn a_creation_that_failed_after_registering_still_copies() {
    let root: PathBuf = fixture("lifecycle-failed-after-registering");
    let (main, common) = source(root.as_path());
    let topic: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(topic.as_path())),
        &mut |_step: &GitStep<'_>| {
            run_real(main.as_path(), add_arguments("topic", topic.as_path()).as_slice());
            return Ok(GitStatus { code: Some(1) });
        },
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 1);
    assert_eq!(std::fs::read(topic.join("cache/data")).expect("copied anyway"), b"payload");
    remove(root.as_path());
}

/// The repository path of the generated rules file that sits in the repository root's own `.cache`.
const RULES_PATH: &str = ".cache/forbidden-strings.rules.txt";

/// A main checkout whose ignored state has the shapes the rule must tell apart,
/// returned with its canonical root and common
/// directory:
/// installed dependencies with a pnpm-style link and a hard-linked store file,
/// build output with a cache inside it,
/// caches,
/// run records,
/// a local file,
/// a separate checkout and a registered worktree nested in an ignored directory.
#[cfg(unix)]
fn main_with_state(root: &Path) -> (PathBuf, PathBuf) {
    let main: PathBuf = repository(root, "main");
    std::fs::write(
        main.join(".gitignore"),
        b".cache/\nnode_modules/\ntarget/\n.pnpm-store/\n.claude/\nmutants.out/\nclone/\n*.local.*\n",
    )
    .expect("gitignore");
    let write = |path: &str, bytes: &[u8]| {
        let full: PathBuf = main.join(path);
        std::fs::create_dir_all(full.parent().expect("parent")).expect("directories");
        std::fs::write(full, bytes).expect("file");
    };
    write(RULES_PATH, b"rules");
    write(".cache/test-scratch/keep.txt", b"scratch");
    write("node_modules/.pnpm/dep@1.0.0/node_modules/dep/index.js", b"module");
    write("node_modules/.pnpm/dep@1.0.0/node_modules/dep/data.bin", b"data");
    write("node_modules/.pnpm-store/v11/files/ab", b"stored");
    write("node_modules/.cache/jiti/x.js", b"jiti");
    write("node_modules/.monochromatic/2026-11-01/log.jsonl", b"log");
    write("package/a/node_modules/.monochromatic/log.jsonl", b"log");
    write("package/a/node_modules/.bin/tool", b"#!/bin/sh\n");
    write("package/a/.cache/typescript/tsbuildinfo", b"cache");
    write("target/debug/deps/libx.rlib", b"rlib");
    write("target/debug/.fingerprint/x/fp", b"fp");
    write("target/debug/incremental/x/state", b"incremental");
    write("target/verification/run/out.txt", b"record");
    write("mutants.out/outcomes.json", b"{}");
    write(".pnpm-store/v11/files/cd", b"store");
    write("mise.local.toml", b"[env]\n");
    write("clone/.git/HEAD", b"ref: refs/heads/main\n");
    write("clone/file.txt", b"another checkout");
    // The pnpm layout: a package link to the virtual store, relative, and a store file with a second hard link.
    std::os::unix::fs::symlink(
        ".pnpm/dep@1.0.0/node_modules/dep",
        main.join("node_modules/dep"),
    )
    .expect("link");
    std::os::unix::fs::symlink(
        "../../../node_modules/.pnpm/dep@1.0.0/node_modules/dep",
        main.join("package/a/node_modules/dep"),
    )
    .expect("package link");
    std::fs::hard_link(
        main.join("node_modules/.pnpm/dep@1.0.0/node_modules/dep/data.bin"),
        main.join("target/debug/deps/data.bin"),
    )
    .expect("hard link");
    git(main.as_path(), &["add", ".gitignore"]);
    git(main.as_path(), &["commit", "--message", "ignore"]);
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "nested", ".claude/worktrees/nested"]);
    write(".claude/worktrees/nested/inside.txt", b"nested worktree file");
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("common");
    return (std::fs::canonicalize(main).expect("canonical main"), common);
}

/// Runs `git worktree add` for a new `topic` worktree from the main checkout through the lifecycle.
fn add_from_main(root: &Path, main: &Path, common: &Path, flags: (bool, bool)) -> (Settlement, String, PathBuf) {
    let topic: PathBuf = root.join("topic");
    let identity: WorktreeIdentity = main_identity(main, common);
    let args: Vec<String> = vec![
        String::from("worktree"),
        String::from("add"),
        String::from("--quiet"),
        String::from("-b"),
        String::from("topic"),
        topic.to_string_lossy().into_owned(),
    ];
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: identity.clone(),
            effective_cwd: main.to_path_buf(),
            creates: flags.0,
            destination: Some(topic.clone()),
            opted_out: flags.1,
            short_circuit: false,
            context: git_context(),
            add_plan: None,
            repair: crate::package_repair::RepairSwitch::default(),
        },
        &mut |_step: &GitStep<'_>| return Ok(run_real(main, args.as_slice())),
        &mut stderr,
    )
    .expect("settled");
    return (settlement, String::from_utf8(stderr).expect("utf8"), topic);
}

/// Nothing of the copy stays behind:
/// no lock file,
/// no pending journal and no private stage beside the worktree.
fn assert_nothing_left(root: &Path, common: &Path) {
    assert!(!journal_root(common).join("settlement.lock").exists(), "no lock is ever taken");
    assert!(read_pending_journals(common).expect("pending").is_empty());
    let leftovers: Vec<String> = std::fs::read_dir(root)
        .expect("listing")
        .map(|entry| return entry.expect("entry").file_name().to_string_lossy().into_owned())
        .filter(|name| return name.starts_with(crate::worktree_copy_selection::STAGE_PREFIX))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

/// A worktree added from the main checkout receives the dependencies and builds it needs,
/// links and all.
#[cfg(unix)]
#[test]
fn main_checkout_source_carries_dependencies_and_builds() {
    let root: PathBuf = fixture("lifecycle-main-carries");
    let (main, common) = main_with_state(root.as_path());
    let (settlement, stderr, topic) = add_from_main(root.as_path(), main.as_path(), common.as_path(), (true, false));
    assert_eq!(exit_code(&settlement), 0, "{stderr} {:?}", settlement.copy_failure);
    assert!(settlement.copy_failure.is_none());
    let dependency: &str = "node_modules/.pnpm/dep@1.0.0/node_modules/dep/index.js";
    assert_eq!(std::fs::read(topic.join(dependency)).expect("root dependency"), b"module");
    assert_eq!(
        std::fs::read_link(topic.join("node_modules/dep")).expect("root link"),
        PathBuf::from(".pnpm/dep@1.0.0/node_modules/dep"),
        "a link into the virtual store stays a link with the same target"
    );
    assert_eq!(
        std::fs::read_link(topic.join("package/a/node_modules/dep")).expect("package link"),
        PathBuf::from("../../../node_modules/.pnpm/dep@1.0.0/node_modules/dep"),
    );
    assert_eq!(std::fs::read(topic.join("package/a/node_modules/dep/index.js")).expect("through the link"), b"module");
    assert!(topic.join("package/a/node_modules/.bin/tool").is_file());
    assert_eq!(std::fs::read(topic.join("target/debug/deps/libx.rlib")).expect("build"), b"rlib");
    assert!(topic.join("target/debug/.fingerprint/x/fp").is_file());
    assert_eq!(std::fs::read(topic.join("target/debug/deps/data.bin")).expect("hard linked file"), b"data");
    assert_eq!(std::fs::read(topic.join("node_modules/.pnpm/dep@1.0.0/node_modules/dep/data.bin")).expect("its twin"), b"data");
    assert_eq!(std::fs::read(topic.join(RULES_PATH)).expect("the root cache"), b"rules");
    assert!(topic.join(".cache/test-scratch/keep.txt").is_file());
    assert!(topic.join("mise.local.toml").is_file(), "local files are carried");
    assert!(
        stderr.starts_with("monochromatic-git-policy-cli: copied ")
            && stderr.contains(quote(main.to_string_lossy().as_ref()).as_str())
            && stderr.ends_with(" into 1 new worktree.\n"),
        "{stderr}"
    );
    assert_nothing_left(root.as_path(), common.as_path());
    remove(root.as_path());
}

/// A worktree added from the main checkout never receives its nested worktrees,
/// another checkout,
/// caches or run records.
#[cfg(unix)]
#[test]
fn main_checkout_source_leaves_out_checkouts_caches_and_run_records() {
    let root: PathBuf = fixture("lifecycle-main-leaves-out");
    let (main, common) = main_with_state(root.as_path());
    let (settlement, stderr, topic) = add_from_main(root.as_path(), main.as_path(), common.as_path(), (true, false));
    assert_eq!(exit_code(&settlement), 0, "{stderr} {:?}", settlement.copy_failure);
    for absent in [
        ".claude/worktrees/nested/inside.txt",
        "clone",
        "node_modules/.pnpm-store",
        "node_modules/.cache",
        "node_modules/.monochromatic",
        "package/a/node_modules/.monochromatic",
        "package/a/.cache",
        "target/debug/incremental",
        "target/verification",
        "mutants.out",
        ".pnpm-store",
    ] {
        assert!(std::fs::symlink_metadata(topic.join(absent)).is_err(), "{absent} must not be carried");
    }
    assert_nothing_left(root.as_path(), common.as_path());
    remove(root.as_path());
}

/// A file the user cannot read is left out with one notice line,
/// and the add still succeeds.
#[cfg(unix)]
#[test]
fn main_checkout_source_skips_unreadable_entries_with_a_notice() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("lifecycle-main-unreadable");
    let (main, common) = main_with_state(root.as_path());
    let hidden: PathBuf = main.join("node_modules/.pnpm/dep@1.0.0/node_modules/dep/secret.bin");
    std::fs::write(hidden.as_path(), b"secret").expect("secret");
    std::fs::set_permissions(hidden.as_path(), std::fs::Permissions::from_mode(0o000)).expect("mode");
    // A process with the privilege to read anything cannot show the case, so the control proves the file is unreadable.
    if std::fs::File::open(hidden.as_path()).is_ok() {
        remove(root.as_path());
        return;
    }
    let (settlement, stderr, topic) = add_from_main(root.as_path(), main.as_path(), common.as_path(), (true, false));
    assert_eq!(exit_code(&settlement), 0, "{stderr} {:?}", settlement.copy_failure);
    assert!(settlement.copy_failure.is_none());
    assert!(!topic.join("node_modules/.pnpm/dep@1.0.0/node_modules/dep/secret.bin").exists());
    assert!(topic.join("node_modules/.pnpm/dep@1.0.0/node_modules/dep/index.js").is_file());
    assert!(stderr.contains("monochromatic-git-policy-cli: left out 1 unreadable ignored entry from"), "{stderr}");
    assert!(stderr.contains("secret.bin"), "{stderr}");
    assert_eq!(stderr.matches("left out").count(), 1, "one notice line: {stderr}");
    assert_nothing_left(root.as_path(), common.as_path());
    std::fs::set_permissions(hidden.as_path(), std::fs::Permissions::from_mode(0o600)).expect("restore");
    remove(root.as_path());
}

/// The opt-out flag leaves the main checkout's ignored state behind and prints nothing.
#[cfg(unix)]
#[test]
fn main_checkout_source_honors_the_opt_out() {
    let root: PathBuf = fixture("lifecycle-main-opt-out");
    let (main, common) = main_with_state(root.as_path());
    let (settlement, stderr, topic) = add_from_main(root.as_path(), main.as_path(), common.as_path(), (true, true));
    assert_eq!(exit_code(&settlement), 0);
    assert!(stderr.is_empty(), "{stderr}");
    assert!(topic.exists(), "Git itself still created the worktree");
    assert!(!topic.join("node_modules").exists());
    assert!(!topic.join(RULES_PATH).exists());
    remove(root.as_path());
}

/// A command the caller reports as creating no worktree is never settled,
/// even from a main checkout with ignored state.
#[cfg(unix)]
#[test]
fn main_checkout_source_ignores_commands_that_create_no_worktree() {
    let root: PathBuf = fixture("lifecycle-main-not-creating");
    let (main, common) = main_with_state(root.as_path());
    let (settlement, stderr, topic) = add_from_main(root.as_path(), main.as_path(), common.as_path(), (false, false));
    assert_eq!(exit_code(&settlement), 0);
    assert!(stderr.is_empty(), "{stderr}");
    assert!(topic.exists(), "the runner still created the worktree");
    assert!(!topic.join("node_modules").exists());
    remove(root.as_path());
}

/// A destination Git made whose registration does not name it back gets no copy,
/// and the command says so instead of
/// succeeding in silence.
#[cfg(unix)]
#[test]
fn a_made_destination_whose_registration_cannot_be_confirmed_is_told() {
    let root: PathBuf = fixture("lifecycle-unconfirmed");
    let (main, common) = source(root.as_path());
    let topic: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(topic.as_path())),
        &mut |_step: &GitStep<'_>| {
            let status: GitStatus = run_real(main.as_path(), add_arguments("topic", topic.as_path()).as_slice());
            std::fs::write(common.join("worktrees/topic/gitdir"), b"/somewhere/else/.git\n").expect("registration edited");
            return Ok(status);
        },
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0);
    let text: String = String::from_utf8(stderr).expect("utf8");
    assert!(text.contains("registration could not be confirmed"), "{text}");
    assert!(text.contains(topic.to_str().expect("utf8")), "{text}");
    assert!(!topic.join("cache").exists());
    remove(root.as_path());
}

/// The worktrees handed on for the package-manager repair are exactly the destination whose registration the binding
/// confirmed:
/// never another add's worktree registered meanwhile,
/// and none when the destination is unknown or unconfirmed.
#[cfg(unix)]
#[test]
fn the_repair_is_handed_exactly_the_confirmed_destination() {
    let root: PathBuf = fixture("lifecycle-repair-destination");
    let (main, common) = source(root.as_path());
    let canonical_root: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root");
    let topic: PathBuf = canonical_root.join("topic");
    let other: PathBuf = canonical_root.join("other");
    let observation: WorktreeCopyObservation = observe_copy_source(&main_identity(main.as_path(), common.as_path()), main.as_path())
        .expect("observed")
        .expect("applicable");
    let (_first, after) = settle_creation(
        &creation_from(main.as_path(), common.as_path(), Some(topic.as_path())),
        &observation,
        &mut |_step: &GitStep<'_>| {
            run_real(main.as_path(), add_arguments("other", other.as_path()).as_slice());
            return Ok(run_real(main.as_path(), add_arguments("topic", topic.as_path()).as_slice()));
        },
        &mut Vec::new(),
    )
    .expect("settled");
    assert_eq!(after.created, vec![topic.clone()]);
    assert_eq!(after.source_root, Some(main.clone()));
    // An unknown destination hands nothing on, even though Git made a worktree.
    let unknown: PathBuf = canonical_root.join("unknown");
    let (_second, none) = settle_creation(
        &creation_from(main.as_path(), common.as_path(), None),
        &observation,
        &mut |_step: &GitStep<'_>| return Ok(run_real(main.as_path(), add_arguments("unknown", unknown.as_path()).as_slice())),
        &mut Vec::new(),
    )
    .expect("settled");
    assert!(none.created.is_empty());
    remove(root.as_path());
}
