//! Lifecycle checks against real Git:
//! applicability,
//! bypasses,
//! settlement,
//! the summary line and exit statuses.

use super::*;
use crate::worktree_copy_notices::UNKNOWN_DESTINATION_NOTICE;
use crate::test_support::{fixture, git, git_context, git_output, remove, repository};
use crate::worktree_copy_after_lock::trash_beside_repair_awaiting;
use crate::worktree_copy_journal::{create_journal_for, journal_root, read_pending_journals};
use crate::test_support::dead_copy_owner;
use std::process::Output;

/// A function-like value with the right shape is a Git runner in these controls:
/// the shipped caller writes a small struct that holds the state.
impl<Run: FnMut(&GitStep<'_>) -> Result<GitStatus, WorktreeCopyError>> GitRunner for Run {
    fn run(&mut self, step: &GitStep<'_>, _child_started: &mut dyn ChildStarted) -> Result<GitStatus, WorktreeCopyError> {
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
    let root: PathBuf = fixture("lifecycle-trash-disposal");
    let (main, common) = source(root.as_path());
    // The trash the snapshot names is not there, so the disposal of it fails with the reason it names: a failure of
    //  the disposal is the add's to report, never the add's to fail.
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
        crate::lazy_runtime::run_on_runtime(trash_beside_repair_awaiting(&invocation, &after, &pending, &stream)).expect("this process can build its runtime");
    }
    crate::add_timing::finish(0);
    assert_eq!(stderr, Vec::<u8>::new(), "the person's stream carries the repair, and a disposal that fails says so in the log");
    let written: String = std::fs::read_to_string(common.join(crate::add_timing::ADD_TIMING_FILE)).expect("the add's record");
    let record: serde_json::Value = serde_json::from_str(written.trim_end()).expect("the record is one JSON object");
    assert_eq!(record["work"]["disposeTrash"]["calls"], 1, "{record}");
    assert_eq!(record["steps"]["disposeTrashWait"]["calls"], 1, "the wait is a step of its own: {record}");
    remove(root.as_path());
}

/// The names the private journal root holds right now,
/// sorted.
fn recorded_names(common: &Path) -> Vec<String> {
    let root: PathBuf = journal_root(common);
    let listing = match std::fs::read_dir(root.as_path()) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(error) => panic!("the journal root cannot be listed: {error}"),
    };
    let mut names: Vec<String> = listing
        .map(|entry| return entry.expect("entry").file_name().to_string_lossy().into_owned())
        .filter(|name| return !name.ends_with(crate::worktree_copy_journal_encoding_and_validation::TEMPORARY_SUFFIX))
        .collect();
    names.sort();
    return names;
}

/// An add aimed at a worktree that is registered already records nothing about it,
/// so a third add that copies from that worktree is not told it is still being filled.
#[cfg(unix)]
#[test]
fn an_add_aimed_at_a_registered_worktree_does_not_mark_it_as_being_filled() {
    let root: PathBuf = fixture("lifecycle-taken-destination");
    let (main, common) = source(root.as_path());
    // The repository's own exclusion file, so the registered worktree ignores `cache` without holding a .gitignore file.
    std::fs::write(common.join("info/exclude"), b"cache/\n").expect("exclude");
    let taken: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("taken");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "taken", taken.to_str().expect("utf8")]);
    // The registered worktree holds the source's ignored state, as one a finished copy leaves it.
    std::fs::create_dir_all(taken.join("cache")).expect("cache");
    std::fs::write(taken.join("cache/data"), b"payload").expect("data");
    let topic: PathBuf = root.join("topic");
    let mut records_during_git: Vec<String> = Vec::new();
    let mut nested_stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(taken.as_path())),
        &mut |_step: &GitStep<'_>| {
            records_during_git = recorded_names(common.as_path());
            // A third command adds from the registered worktree while this command is still pending.
            let identity: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
                common_dir: common.clone(),
                git_dir: common.join("worktrees").join("taken"),
                worktree_root: taken.clone(),
            };
            let nested: Invocation = Invocation {
                identity,
                effective_cwd: taken.clone(),
                creates: true,
                destination: Some(topic.clone()),
                opted_out: false,
                short_circuit: false,
                context: git_context(),
                add_plan: None,
                repair: crate::package_repair::RepairSwitch::default(),
            };
            let _settled = run_with_worktree_copy(
                &nested,
                &mut |_: &GitStep<'_>| return Ok(run_real(taken.as_path(), add_arguments("topic", topic.as_path()).as_slice())),
                &mut nested_stderr,
            )
            .expect("settled");
            // Real Git refuses this command's own destination, which another command registered already.
            return Ok(GitStatus { code: Some(1) });
        },
        &mut Vec::new(),
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 1, "Git's own refusal is kept");
    assert_eq!(records_during_git, Vec::<String>::new(), "an add aimed at a registered worktree records nothing about it");
    let copied: std::io::Result<Vec<u8>> = std::fs::read(topic.join("cache/data"));
    assert_eq!(
        copied.ok(),
        Some(b"payload".to_vec()),
        "the third add copies from a worktree that is whole: {}",
        String::from_utf8_lossy(nested_stderr.as_slice())
    );
    assert!(
        !String::from_utf8_lossy(nested_stderr.as_slice()).contains("being filled"),
        "the third add is not told the registered worktree is being filled: {:?}",
        String::from_utf8_lossy(nested_stderr.as_slice())
    );
    remove(root.as_path());
}

/// Two adds at one destination:
/// the command that reserved the directory fills it with its own source,
/// and the command whose own Git refused writes nothing into it,
/// even though a registration stands at the path it named.
#[cfg(unix)]
#[test]
fn two_adds_at_one_destination_copy_once_into_the_reserved_directory() {
    let root: PathBuf = fixture("lifecycle-two-adds-one-destination");
    let (main, common) = source(root.as_path());
    // The repository's own exclusion file, so every worktree of it ignores `cache` whether or not it holds the file.
    std::fs::write(common.join("info/exclude"), b"cache/\n").expect("exclude");
    let canonical_root: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root");
    let destination: PathBuf = canonical_root.join("taken");
    // This command's source is a linked worktree holding its own ignored entry, so a copy of it can be told apart.
    let source_root: PathBuf = canonical_root.join("own-source");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "own", source_root.to_str().expect("utf8")]);
    std::fs::create_dir_all(source_root.join("cache")).expect("cache");
    std::fs::write(source_root.join("cache/from-this-command"), b"mine").expect("own entry");
    let mut other_stderr: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: WorktreeIdentity::LinkedWorktree {
                common_dir: common.clone(),
                git_dir: common.join("worktrees").join("own"),
                worktree_root: source_root.clone(),
            },
            effective_cwd: source_root.clone(),
            creates: true,
            destination: Some(destination.clone()),
            opted_out: false,
            short_circuit: false,
            context: git_context(),
            add_plan: None,
            repair: crate::package_repair::RepairSwitch::default(),
        },
        &mut |_step: &GitStep<'_>| {
            // A whole add of another command at the same destination, run while this command's Git runs.
            let other: Invocation = creation_from(main.as_path(), common.as_path(), Some(destination.as_path()));
            let _settled = run_with_worktree_copy(
                &other,
                &mut |_: &GitStep<'_>| return Ok(run_real(main.as_path(), add_arguments("other", destination.as_path()).as_slice())),
                &mut other_stderr,
            )
            .expect("settled");
            // This command's own Git then refuses, because the destination exists now.
            return Ok(GitStatus { code: Some(1) });
        },
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 1, "Git's own refusal is kept");
    let said: String = String::from_utf8(stderr).expect("utf8");
    assert_eq!(
        std::fs::read(destination.join("cache/from-this-command")).ok(),
        None,
        "this command wrote nothing into a worktree its own Git did not register: {said}"
    );
    let other_said: String = String::from_utf8(other_stderr).expect("utf8");
    assert_eq!(
        std::fs::read(destination.join("cache/data")).ok(),
        Some(b"payload".to_vec()),
        "the command whose Git succeeded filled it: {other_said}"
    );
    assert!(said.contains("did not succeed"), "the person is told why nothing was copied: {said}");
    remove(root.as_path());
}

/// A copy that has not finished is visible to the source check from before real Git:
/// the destination record the creating command wrote names its worktree until the copy's journal does,
/// so a worktree whose copy is still staging can never be another command's source.
#[cfg(unix)]
#[test]
fn a_worktree_whose_copy_has_not_finished_is_a_source_that_is_being_filled() {
    let root: PathBuf = fixture("lifecycle-staging-is-visible");
    let (main, common) = source(root.as_path());
    let topic: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    let mut judged: Option<bool> = None;
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(topic.as_path())),
        &mut |_: &GitStep<'_>| {
            let status: GitStatus = run_real(main.as_path(), add_arguments("topic", topic.as_path()).as_slice());
            // Real Git has registered the worktree and the copy has not staged a byte yet: the record is the proof.
            judged = Some(
                crate::worktree_copy_source_still_being_filled::source_is_still_being_filled(common.as_path(), topic.as_path())
                    .expect("judged")
                    .is_some(),
            );
            return Ok(status);
        },
        &mut Vec::new(),
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0, "{:?}", settlement.copy_failure);
    assert_eq!(judged, Some(true), "the destination is recorded from before real Git until its journal exists");
    // Once the copy has finished, nothing names the worktree any more.
    let after_the_copy: bool =
        crate::worktree_copy_source_still_being_filled::source_is_still_being_filled(common.as_path(), topic.as_path())
            .expect("judged")
            .is_some();
    assert!(!after_the_copy, "a finished worktree is a source");
    assert_eq!(std::fs::read(topic.join("cache/data")).expect("copied"), b"payload");
    remove(root.as_path());
}

/// An error before the copy starts leaves the record of the worktree that was never filled,
/// so a later command reports it instead of finding a worktree that looks whole.
#[cfg(unix)]
#[test]
fn a_failure_before_the_copy_keeps_the_record_of_the_worktree_that_was_never_filled() {
    let root: PathBuf = fixture("lifecycle-failure-keeps-record");
    let (main, common) = source(root.as_path());
    let topic: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    // The registration lookup works, and the Git question that follows it fails: the copy stops before it stages anything.
    let mut broken: crate::transaction_git::GitContext = git_context();
    broken.real_git = std::fs::canonicalize(root.as_path()).expect("canonical root").join("no-git-here");
    let mut invocation: Invocation = creation_from(main.as_path(), common.as_path(), Some(topic.as_path()));
    invocation.context = broken;
    let settlement: Settlement = run_with_worktree_copy(
        &invocation,
        &mut |_: &GitStep<'_>| return Ok(run_real(main.as_path(), add_arguments("topic", topic.as_path()).as_slice())),
        &mut Vec::new(),
    )
    .expect("settled");
    let failure: &WorktreeCopyError = settlement.copy_failure.as_ref().expect("the git question after the lookup cannot be asked");
    assert!(!failure.0.is_empty());
    let records: Vec<String> = recorded_names(common.as_path());
    assert_eq!(
        records.iter().filter(|name| return name.ends_with(".add")).count(),
        1,
        "the record of the worktree that was never filled is kept: {records:?}"
    );
    assert_eq!(std::fs::read(topic.join("cache").join("data")).ok(), None, "nothing was copied");
    remove(root.as_path());
}


/// One destination,
/// one copy:
/// the command whose own Git succeeded fills it,
/// and the command Git refused writes nothing and says so.
#[cfg(unix)]
#[test]
fn one_of_two_adds_at_one_destination_fills_it_and_the_other_writes_nothing() {
    let root: PathBuf = fixture("lifecycle-refused-clean");
    let (main, common) = source(root.as_path());
    std::fs::write(common.join("info/exclude"), b"cache/\n").expect("exclude");
    let canonical_root: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root");
    let destination: PathBuf = canonical_root.join("taken");
    let source_root: PathBuf = canonical_root.join("own-source");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "own", source_root.to_str().expect("utf8")]);
    std::fs::create_dir_all(source_root.join("cache")).expect("cache");
    std::fs::write(source_root.join("cache/from-this-command"), b"mine").expect("own entry");
    let mut other_stderr: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity: WorktreeIdentity::LinkedWorktree {
                common_dir: common.clone(),
                git_dir: common.join("worktrees").join("own"),
                worktree_root: source_root.clone(),
            },
            effective_cwd: source_root.clone(),
            creates: true,
            destination: Some(destination.clone()),
            opted_out: false,
            short_circuit: false,
            context: git_context(),
            add_plan: None,
            repair: crate::package_repair::RepairSwitch::default(),
        },
        &mut |_: &GitStep<'_>| {
            // Another command at the same destination, whose own Git registers and fills the worktree.
            let other: Invocation = creation_from(main.as_path(), common.as_path(), Some(destination.as_path()));
            let _settled = run_with_worktree_copy(
                &other,
                &mut |_: &GitStep<'_>| return Ok(run_real(main.as_path(), add_arguments("other", destination.as_path()).as_slice())),
                &mut other_stderr,
            )
            .expect("settled");
            // This command's own Git then refuses, because the destination exists.
            return Ok(GitStatus { code: Some(1) });
        },
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 1, "Git's own refusal is kept");
    let said: String = String::from_utf8(stderr).expect("utf8");
    assert_eq!(
        std::fs::read(destination.join("cache/from-this-command")).ok(),
        None,
        "this command wrote nothing into a worktree its own Git did not register: {said}"
    );
    assert_eq!(
        std::fs::read(destination.join("cache/data")).ok(),
        Some(b"payload".to_vec()),
        "the command whose Git succeeded filled it: {}",
        String::from_utf8_lossy(other_stderr.as_slice())
    );
    assert!(said.contains("did not succeed"), "the person is told why nothing was copied: {said}");
    remove(root.as_path());
}


/// Two adds that both find the destination unregistered leave one copy and no unfilled,
/// unrecorded worktree:
/// the command that reserved the destination fills it,
/// and the other one says it wrote nothing.
#[cfg(unix)]
#[test]
fn two_adds_at_one_absent_destination_fill_it_once() {
    let root: PathBuf = fixture("lifecycle-two-adds-fill-once");
    let (main, common) = source(root.as_path());
    std::fs::write(common.join("info/exclude"), b"cache/\n").expect("exclude");
    let canonical_root: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root");
    let destination: PathBuf = canonical_root.join("taken");
    let mut other_stderr: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(destination.as_path())),
        &mut |_: &GitStep<'_>| {
            // A second command at the same destination, whose own Git creates the worktree inside this one's reservation.
            let other: Invocation = creation_from(main.as_path(), common.as_path(), Some(destination.as_path()));
            let _settled = run_with_worktree_copy(
                &other,
                &mut |_: &GitStep<'_>| return Ok(run_real(main.as_path(), add_arguments("other", destination.as_path()).as_slice())),
                &mut other_stderr,
            )
            .expect("settled");
            // This command's own Git then refuses, because the destination exists.
            return Ok(GitStatus { code: Some(1) });
        },
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 1);
    assert_eq!(
        std::fs::read(destination.join("cache/data")).ok(),
        Some(b"payload".to_vec()),
        "exactly one command filled the worktree: {}",
        String::from_utf8_lossy(stderr.as_slice())
    );
    // The refused command reads the other command's proof on the registration and settles its own record.
    let records: Vec<String> = recorded_names(common.as_path());
    assert!(records.is_empty(), "the refused command keeps no record for a worktree another completed: {records:?}");
    // Nothing of this command's record stays, so the line must not promise that a later command will report one.
    let said: String = String::from_utf8(stderr).expect("utf8");
    assert!(!said.contains("the record of this add stays"), "no record stays, so the line must not promise one: {said}");
    assert!(said.contains("was made by another command"), "the line says whose worktree stands there: {said}");
    remove(root.as_path());
}

/// A destination that another command moved away and made again is not this command's to fill,
/// even though this command's own Git succeeded and a registration stands at its path.
#[cfg(unix)]
#[test]
fn a_destination_made_again_before_the_lookup_is_left_alone() {
    let root: PathBuf = fixture("lifecycle-made-again-before-lookup");
    let (main, common) = source(root.as_path());
    let canonical_root: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root");
    let destination: PathBuf = canonical_root.join("taken");
    let moved: PathBuf = canonical_root.join("moved-away");
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(destination.as_path())),
        &mut |_: &GitStep<'_>| {
            let status: GitStatus = run_real(main.as_path(), add_arguments("mine", destination.as_path()).as_slice());
            // Another command moves this worktree away and registers a different one at the same path.
            git(main.as_path(), &["worktree", "move", destination.to_str().expect("utf8"), moved.to_str().expect("utf8")]);
            git(
                main.as_path(),
                &["worktree", "add", "--quiet", "--no-checkout", "-b", "replacement", destination.to_str().expect("utf8")],
            );
            return Ok(status);
        },
        &mut stderr,
    )
    .expect("settled");
    let said: String = String::from_utf8(stderr).expect("utf8");
    assert_eq!(exit_code(&settlement), 0, "Git's own success is kept: {said}");
    assert_eq!(
        std::fs::read(destination.join("cache/data")).ok(),
        None,
        "the replacement registration received nothing: {said}"
    );
    assert!(said.contains("no longer at that path"), "the person is told why: {said}");
    remove(root.as_path());
}

/// A worktree that receives no copy because its source is still being filled keeps its own unfinished record,
/// so a later add from it is held back instead of treating its partial ignored state as complete.
#[cfg(unix)]
#[test]
fn a_skipped_unfinished_source_keeps_the_record_of_the_worktree_it_did_not_fill() {
    let root: PathBuf = fixture("lifecycle-skipped-source-keeps-record");
    let (main, common) = source(root.as_path());
    std::fs::write(common.join("info/exclude"), b"cache/\n").expect("exclude");
    let filling: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("filling");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "filling", filling.to_str().expect("utf8")]);
    // The state a copy in progress leaves: one of the source's ignored entries is there and the rest are not.
    std::fs::create_dir_all(filling.join("cache")).expect("half-filled cache");
    std::fs::write(filling.join("cache/half"), b"partial").expect("half");
    let live = journal_for(main.as_path(), common.as_path(), filling.as_path(), &crate::worktree_copy_owner::current_copy_owner().expect("owner"));
    let topic: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    let identity: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: common.clone(),
        git_dir: common.join("worktrees").join("filling"),
        worktree_root: filling.clone(),
    };
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity,
            effective_cwd: filling.clone(),
            creates: true,
            destination: Some(topic.clone()),
            opted_out: false,
            short_circuit: false,
            context: git_context(),
            add_plan: None,
            repair: crate::package_repair::RepairSwitch::default(),
        },
        &mut |_: &GitStep<'_>| return Ok(run_real(filling.as_path(), add_arguments("topic", topic.as_path()).as_slice())),
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0, "the add itself succeeds: {:?}", settlement.copy_failure);
    let said: String = String::from_utf8(stderr).expect("utf8");
    assert!(said.contains("is itself being filled"), "the person is told why nothing was copied: {said}");
    assert!(!topic.join("cache").exists(), "the half-filled ignored state is not copied");
    let records: Vec<String> = recorded_names(common.as_path());
    let add_records: usize = records.iter().filter(|name| return name.ends_with(".add")).count();
    assert_eq!(
        add_records, 1,
        "this command's own unfinished record stays, so a later add from the new worktree is held back: {records:?} {live:?}"
    );
    remove(root.as_path());
}

/// A destination that stood there before this command's Git and was moved away and made again is not this command's:
/// its own Git succeeds,
/// and what stands at its path is another command's registration.
#[cfg(unix)]
#[test]
fn a_destination_that_stood_before_the_add_and_was_made_again_is_left_alone() {
    let root: PathBuf = fixture("lifecycle-stood-before-made-again");
    let (main, common) = source(root.as_path());
    let canonical_root: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root");
    let destination: PathBuf = canonical_root.join("taken");
    let moved: PathBuf = canonical_root.join("moved-away");
    // The destination stands there as an empty directory before this command runs, as another command's reservation does.
    std::fs::create_dir(destination.as_path()).expect("an empty destination");
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(destination.as_path())),
        &mut |_: &GitStep<'_>| {
            let status: GitStatus = run_real(main.as_path(), add_arguments("mine", destination.as_path()).as_slice());
            // Another command moves this worktree away and registers a different one at the same path.
            git(main.as_path(), &["worktree", "move", destination.to_str().expect("utf8"), moved.to_str().expect("utf8")]);
            git(
                main.as_path(),
                &["worktree", "add", "--quiet", "--no-checkout", "-b", "replacement", destination.to_str().expect("utf8")],
            );
            return Ok(status);
        },
        &mut stderr,
    )
    .expect("settled");
    let said: String = String::from_utf8(stderr).expect("utf8");
    assert_eq!(exit_code(&settlement), 0, "Git's own success is kept: {said}");
    assert_eq!(
        std::fs::read(destination.join("cache/data")).ok(),
        None,
        "the replacement registration received nothing: {said}"
    );
    assert!(said.contains("received no ignored-state copy"), "the person is told why: {said}");
    // The worktree this command made was moved away and never filled, so its record stays and a later add from it is held
    // back instead of treating its missing ignored state as whole.
    let records: Vec<String> = recorded_names(common.as_path());
    assert_eq!(
        records.iter().filter(|name| return name.ends_with(".add")).count(),
        1,
        "the record of the worktree this command never filled stays: {records:?}"
    );
    remove(root.as_path());
}

/// A refused add leaves no unfinished record for the worktree another command completed in its place,
/// so a later add from that worktree copies its ignored state instead of being told it is only partly there.
/// The durability writes of its three settlements stay in the page cache,
/// because this control checks nothing about what a crash leaves.
#[cfg(unix)]
#[test]
fn a_refused_add_leaves_no_record_for_the_worktree_another_command_completed() {
    keep_the_durability_writes_in_the_page_cache();
    let root: PathBuf = fixture("lifecycle-refused-no-record");
    let (main, common) = source(root.as_path());
    std::fs::write(common.join("info/exclude"), b"cache/\n").expect("exclude");
    let canonical_root: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root");
    let destination: PathBuf = canonical_root.join("taken");
    let mut stderr: Vec<u8> = Vec::new();
    let mut second_stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(destination.as_path())),
        &mut |_: &GitStep<'_>| {
            // The other command registers and fills the worktree inside this command's reserved directory.
            let other: Invocation = creation_from(main.as_path(), common.as_path(), Some(destination.as_path()));
            let _settled = run_with_worktree_copy(
                &other,
                &mut |_: &GitStep<'_>| return Ok(run_real(main.as_path(), add_arguments("other", destination.as_path()).as_slice())),
                &mut Vec::new(),
            )
            .expect("settled");
            // This command's own Git then refuses, because the worktree exists.
            return Ok(GitStatus { code: Some(1) });
        },
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 1, "Git's own refusal is kept");
    let records: Vec<String> = recorded_names(common.as_path());
    assert!(
        records.is_empty(),
        "a refused add leaves no record for a worktree another command completed: {records:?}"
    );
    // A later add from that worktree, which is whole, copies its ignored state.
    let later: PathBuf = canonical_root.join("later");
    let nested: Invocation = Invocation {
        identity: WorktreeIdentity::LinkedWorktree {
            common_dir: common.clone(),
            git_dir: common.join("worktrees").join("other"),
            worktree_root: destination.clone(),
        },
        effective_cwd: destination.clone(),
        creates: true,
        destination: Some(later.clone()),
        opted_out: false,
        short_circuit: false,
        context: git_context(),
        add_plan: None,
        repair: crate::package_repair::RepairSwitch::default(),
    };
    let _settled = run_with_worktree_copy(
        &nested,
        &mut |_: &GitStep<'_>| return Ok(run_real(destination.as_path(), add_arguments("later", later.as_path()).as_slice())),
        &mut second_stderr,
    )
    .expect("settled");
    assert_eq!(
        std::fs::read(later.join("cache/data")).ok(),
        Some(b"payload".to_vec()),
        "the later add copies from a worktree that is whole: {}",
        String::from_utf8_lossy(second_stderr.as_slice())
    );
    remove(root.as_path());
}

/// A destination that cannot be read is not a destination that holds nothing:
/// the worktree Git registered and never filled keeps its unfinished record.
#[cfg(unix)]
#[test]
fn an_unreadable_destination_keeps_the_record_of_the_worktree_that_was_not_filled() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("lifecycle-unreadable-destination-keeps-record");
    let (main, common) = source(root.as_path());
    let topic: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(topic.as_path())),
        &mut |_: &GitStep<'_>| {
            // Real Git registers the worktree, and then the destination cannot be read by this command at all.
            run_real(main.as_path(), add_arguments("topic", topic.as_path()).as_slice());
            std::fs::set_permissions(topic.as_path(), std::fs::Permissions::from_mode(0o000)).expect("unreadable");
            return Ok(GitStatus { code: Some(1) });
        },
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 1, "Git's own failure is kept");
    let records: Vec<String> = recorded_names(common.as_path());
    let add_records: usize = records.iter().filter(|name| return name.ends_with(".add")).count();
    // Restoring the mode before the fixture is removed, whatever the assertion said.
    std::fs::set_permissions(topic.as_path(), std::fs::Permissions::from_mode(0o755)).expect("readable again");
    assert_eq!(
        add_records, 1,
        "the worktree Git registered and never filled keeps its unfinished record: {records:?}"
    );
    assert_eq!(std::fs::read(topic.join("cache/data")).ok(), None, "nothing was copied into it");
    remove(root.as_path());
}

/// A control whose subject is neither what a crash leaves nor the flush itself keeps the durability writes of its
/// fixtures in the page cache,
/// and the settlement still records every sync it asked for.
/// Why:
///   One settlement asks for more than thirty flushes,
/// and on this repository's fixture volume (btrfs on dm-crypt with `compress=zstd:3`) each of them waits for a log
/// commit that every other process of the machine waits for too:
/// measured 2026-11-10,
/// one record flush of a `worktree_copy` fixture took up to 20.0 s while the host ran at a load average of 27 to 33
/// (`doc/troubleshooting/git-policy-cli-unit-test-watchdog-judges-progress.md`,
/// "The flushes the controls did not need").
#[cfg(unix)]
#[test]
fn a_settlement_whose_control_checks_no_crash_keeps_its_writes_in_the_page_cache() {
    keep_the_durability_writes_in_the_page_cache();
    let root: PathBuf = fixture("lifecycle-writes-kept-in-memory");
    let (main, common) = source(root.as_path());
    let topic: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    let mut stderr: Vec<u8> = Vec::new();
    let mut faithful_in_the_settlement: Vec<bool> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(topic.as_path())),
        &mut |_step: &GitStep<'_>| {
            // Read in the middle of the settlement: the journal is written and the copy is not installed yet.
            faithful_in_the_settlement.push(crate::private_storage::syncing_is_faithful());
            return Ok(run_real(main.as_path(), add_arguments("topic", topic.as_path()).as_slice()));
        },
        &mut stderr,
    )
    .expect("settled");
    let synced_directories: Vec<PathBuf> = crate::private_storage::take_directory_syncs();
    let synced_files: Vec<PathBuf> = crate::private_storage::take_file_syncs();
    assert_eq!(exit_code(&settlement), 0, "{}", String::from_utf8_lossy(stderr.as_slice()));
    assert_eq!(std::fs::read(topic.join("cache/data")).expect("copied"), b"payload");
    assert_eq!(
        faithful_in_the_settlement,
        vec![false],
        "the settlement's durability writes stay in the page cache for a control that checks nothing about a crash"
    );
    // The records stay whatever the switch says, so a control whose subject is the order of the flushes still reads them.
    assert!(
        synced_directories.iter().any(|path| return path.ends_with("v1")),
        "the settlement's own directory syncs are still recorded: {synced_directories:?}"
    );
    assert!(
        synced_files.iter().any(|path| return path.ends_with("install-log.jsonl")),
        "the install log's own file sync is still recorded: {synced_files:?}"
    );
    remove(root.as_path());
}

/// What:
///  Keep the durability writes of this control's fixtures in the page cache,
/// for a control whose subject is neither what a crash leaves nor the flush itself.
/// Why:
///   One settlement asks for more than thirty flushes,
/// and each of them waits for a log commit that every other process of the machine waits for too,
/// so the disk decides whether the control finishes inside the run's own bound.
/// Only the wait goes:
/// every sync is still recorded,
/// so a control whose subject is the order of the flushes reads them as before.
fn keep_the_durability_writes_in_the_page_cache() {
    crate::private_storage::skip_the_durability_syncs_here();
}

/// The arguments of a quiet `git worktree add -b <name> <destination>`.
fn add_arguments(name: &str, destination: &Path) -> Vec<String> {
    return ["worktree", "add", "--quiet", "-b", name].iter().map(|word| { return (*word).to_string(); }).chain([destination.to_string_lossy().into_owned()]).collect();
}

/// A creating command records the worktree it is about to make before it starts real Git,
/// so a command killed right after Git leaves a record the next command can act on,
/// and the record and its claim are gone once the copy ends.
#[cfg(unix)]
#[test]
fn a_creating_command_records_its_destination_before_real_git_runs() {
    use crate::worktree_add_binding::{Recorded, decode_record};
    let root: PathBuf = fixture("lifecycle-record-before-git");
    let (main, common) = source(root.as_path());
    let destination: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    let mut during_git: Vec<String> = Vec::new();
    let mut recorded_destination: Option<PathBuf> = None;
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(destination.as_path())),
        &mut |_step: &GitStep<'_>| {
            during_git = recorded_names(common.as_path());
            for name in &during_git {
                if let Some(recorded) = name.strip_suffix(".add") {
                    let text: String = std::fs::read_to_string(journal_root(common.as_path()).join(format!("{recorded}.add"))).expect("record");
                    if let Some(Recorded::Planned(planned)) = decode_record(text.as_str()) {
                        recorded_destination = Some(planned);
                    }
                }
            }
            return Ok(run_real(main.as_path(), add_arguments("topic", destination.as_path()).as_slice()));
        },
        &mut Vec::new(),
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0, "{:?}", settlement.copy_failure);
    assert_eq!(recorded_destination, Some(destination.clone()), "the destination is recorded before Git runs: {during_git:?}");
    assert_eq!(during_git.iter().filter(|name| return name.ends_with(".stage")).count(), 1, "one claim names the stage: {during_git:?}");
    assert_eq!(std::fs::read(destination.join("cache/data")).expect("copied"), b"payload");
    assert_eq!(recorded_names(common.as_path()), Vec::<String>::new(), "the record and its claim are gone once the worktree is whole");
    assert_nothing_left(root.as_path(), common.as_path());
    remove(root.as_path());
}

/// A creating command whose real Git refused leaves no record and no claim,
/// and tells the person nothing about an add that never happened.
#[cfg(unix)]
#[test]
fn a_creation_that_git_refused_leaves_no_record() {
    let root: PathBuf = fixture("lifecycle-record-refused");
    let (main, common) = source(root.as_path());
    let destination: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(destination.as_path())),
        // Git refuses a branch that already exists, and creates nothing.
        &mut |_step: &GitStep<'_>| {
            return Ok(run_real(
                main.as_path(),
                add_arguments("main", destination.as_path()).as_slice(),
            ));
        },
        &mut stderr,
    )
    .expect("settled");
    assert!(!settlement.git.succeeded(), "Git refused the add");
    assert_eq!(recorded_names(common.as_path()), Vec::<String>::new(), "nothing is left to act on");
    assert_eq!(String::from_utf8(stderr).expect("utf8"), String::new(), "an add Git never made is not reported");
    remove(root.as_path());
}

/// The arguments of a quiet `git worktree add -b <name> <destination> <revision>`.
fn add_at_revision(name: &str, destination: &Path, revision: &str) -> Vec<String> {
    let mut arguments: Vec<String> = add_arguments(name, destination);
    arguments.push(revision.to_string());
    return arguments;
}

/// The arguments of a quiet `git worktree add <destination> <branch>`,
/// the form that names a branch instead of making one.
fn add_existing_branch(branch: &str, destination: &Path) -> Vec<String> {
    return ["worktree", "add", "--quiet"].iter().map(|word| { return (*word).to_string(); }).chain([destination.to_string_lossy().into_owned(), branch.to_string()]).collect();
}

/// A refused add leaves no folder where the worktree would have been:
/// an empty folder inside a repository is acted on by any Git command run in it,
/// as if the enclosing checkout were the worktree.
#[cfg(unix)]
#[test]
fn a_refused_add_leaves_no_folder_where_the_worktree_would_have_been() {
    let root: PathBuf = fixture("lifecycle-refused-removes");
    let (main, common) = source(root.as_path());
    let canonical_root: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root");
    let destination: PathBuf = canonical_root.join("topic");
    // Real Git's own answer for the same refusal is the oracle for the status the wrapper must keep.
    let oracle: GitStatus = run_real(main.as_path(), add_arguments("main", canonical_root.join("oracle").as_path()).as_slice());
    assert!(!oracle.succeeded(), "Git refuses a branch that already exists");
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(destination.as_path())),
        &mut |_step: &GitStep<'_>| {
            return Ok(run_real(
                main.as_path(),
                add_arguments("main", destination.as_path()).as_slice(),
            ));
        },
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), oracle.code.unwrap_or(1), "Git's own status is kept");
    assert_eq!(String::from_utf8(stderr).expect("utf8"), String::new(), "an add Git never made is not reported");
    assert!(!destination.exists(), "the directory this command made is removed again while it is empty");
    remove(root.as_path());
}

/// Every refusal real Git makes before it fills a destination leaves no folder behind either:
/// a start point that does not exist,
/// and a branch another worktree has checked out.
#[cfg(unix)]
#[test]
fn a_refusal_before_the_worktree_leaves_no_folder() {
    let root: PathBuf = fixture("lifecycle-refused-routes");
    let (main, common) = source(root.as_path());
    let canonical_root: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root");
    // A branch another worktree holds, so this command's Git refuses the add before it makes anything.
    let holder: PathBuf = canonical_root.join("holder");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "locked", holder.to_str().expect("utf8")]);
    let bad_start: PathBuf = canonical_root.join("bad-start");
    let locked_copy: PathBuf = canonical_root.join("locked-copy");
    let routes: [(&str, &Path, Vec<String>); 2] = [
        ("a start point that does not exist", bad_start.as_path(), add_at_revision("fresh", bad_start.as_path(), "no-such-revision")),
        ("a branch another worktree holds", locked_copy.as_path(), add_existing_branch("locked", locked_copy.as_path())),
    ];
    for (route, destination, arguments) in routes {
        let oracle: GitStatus = run_real(main.as_path(), arguments.as_slice());
        assert!(!oracle.succeeded(), "Git refuses {route}");
        let mut stderr: Vec<u8> = Vec::new();
        let settlement: Settlement = run_with_worktree_copy(
            &creation_from(main.as_path(), common.as_path(), Some(destination)),
            &mut |_step: &GitStep<'_>| return Ok(run_real(main.as_path(), arguments.as_slice())),
            &mut stderr,
        )
        .expect("settled");
        assert_eq!(exit_code(&settlement), oracle.code.unwrap_or(1), "Git's own status is kept for {route}");
        assert_eq!(String::from_utf8(stderr).expect("utf8"), String::new(), "an add Git never made is not reported for {route}");
        assert!(!destination.exists(), "no folder is left for {route}");
    }
    remove(root.as_path());
}

/// A destination that stood before the command is the person's:
/// the refused add leaves it,
/// empty or holding what they put there.
#[cfg(unix)]
#[test]
fn a_refused_add_leaves_a_destination_that_stood_before_it() {
    let root: PathBuf = fixture("lifecycle-refused-before");
    let (main, common) = source(root.as_path());
    let canonical_root: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root");
    let empty: PathBuf = canonical_root.join("empty");
    std::fs::create_dir(empty.as_path()).expect("an empty destination the person made");
    let holding: PathBuf = canonical_root.join("holding");
    std::fs::create_dir_all(holding.join("deeper")).expect("a destination the person made");
    std::fs::write(holding.join("deeper/note"), b"mine").expect("the person's file");
    for destination in [empty.as_path(), holding.as_path()] {
        let settlement: Settlement = run_with_worktree_copy(
            &creation_from(main.as_path(), common.as_path(), Some(destination)),
            &mut |_step: &GitStep<'_>| {
                return Ok(run_real(
                    main.as_path(),
                    add_arguments("main", destination).as_slice(),
                ));
            },
            &mut Vec::new(),
        )
        .expect("settled");
        assert!(!settlement.git.succeeded(), "Git refused the add into {}", destination.display());
        assert!(destination.is_dir(), "{} stood before the command and stays", destination.display());
    }
    assert_eq!(std::fs::read(holding.join("deeper/note")).expect("the person's file"), b"mine");
    assert_eq!(std::fs::read_dir(empty.as_path()).expect("listed").count(), 0, "the destination the person made stays empty");
    remove(root.as_path());
}

/// A destination this command made and another program wrote into while real Git ran stays with what it holds,
/// and the line for the person names the path it left there.
#[cfg(unix)]
#[test]
fn a_destination_written_into_is_left_there_and_named() {
    let root: PathBuf = fixture("lifecycle-refused-written-into");
    let (main, common) = source(root.as_path());
    let destination: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(destination.as_path())),
        &mut |_step: &GitStep<'_>| {
            // Another program writes into the directory this command made, before real Git refuses the add.
            std::fs::write(destination.join("someone-elses-file"), b"theirs").expect("written by another program");
            return Ok(GitStatus { code: Some(1) });
        },
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 1, "Git's own refusal is kept");
    assert_eq!(
        std::fs::read(destination.join("someone-elses-file")).expect("the other program's file"),
        b"theirs",
        "a destination that holds anything is never removed"
    );
    let said: String = String::from_utf8(stderr).expect("utf8");
    assert!(said.contains("left in place"), "the person is told the directory stayed: {said}");
    assert!(
        said.contains(destination.to_str().expect("utf8 destination")),
        "the line names the path it left: {said}"
    );
    remove(root.as_path());
}

/// A refused add leaves the folder the person already had above the destination,
/// and makes none where they had none.
#[cfg(unix)]
#[test]
fn a_refused_add_leaves_the_folders_above_the_destination_to_the_person() {
    let root: PathBuf = fixture("lifecycle-refused-parents");
    let (main, common) = source(root.as_path());
    let canonical_root: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root");
    let parents: PathBuf = canonical_root.join("parents");
    std::fs::create_dir(parents.as_path()).expect("the person's folder");
    let below_a_parent: PathBuf = parents.join("topic");
    let below_nothing: PathBuf = canonical_root.join("absent/topic");
    for destination in [below_a_parent.as_path(), below_nothing.as_path()] {
        let settlement: Settlement = run_with_worktree_copy(
            &creation_from(main.as_path(), common.as_path(), Some(destination)),
            &mut |_step: &GitStep<'_>| {
                return Ok(run_real(
                    main.as_path(),
                    add_arguments("main", destination).as_slice(),
                ));
            },
            &mut Vec::new(),
        )
        .expect("settled");
        assert!(!settlement.git.succeeded(), "Git refused the add into {}", destination.display());
        assert!(!destination.exists(), "the destination this command made is gone: {}", destination.display());
    }
    assert!(parents.is_dir(), "the folder the person already had stays");
    assert_eq!(std::fs::read_dir(parents.as_path()).expect("listed").count(), 0, "and holds nothing new");
    assert!(!canonical_root.join("absent").exists(), "the wrapper makes no folder of its own above the destination");
    remove(root.as_path());
}

/// A destination Git registered before the command is that command's worktree,
/// so the refused add leaves it and its contents exactly as they were.
#[cfg(unix)]
#[test]
fn a_refused_add_leaves_a_registered_destination_untouched() {
    let root: PathBuf = fixture("lifecycle-refused-registered");
    let (main, common) = source(root.as_path());
    let destination: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("taken");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "taken", destination.to_str().expect("utf8")]);
    let pointer_before: Vec<u8> = std::fs::read(destination.join(".git")).expect("the registration pointer");
    let mut names_before: Vec<String> = std::fs::read_dir(destination.as_path())
        .expect("listed")
        .map(|entry| return entry.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    names_before.sort();
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(destination.as_path())),
        &mut |_step: &GitStep<'_>| {
            return Ok(run_real(
                main.as_path(),
                add_arguments("other", destination.as_path()).as_slice(),
            ));
        },
        &mut stderr,
    )
    .expect("settled");
    assert!(!settlement.git.succeeded(), "Git refused the second add at that path");
    assert_eq!(std::fs::read(destination.join(".git")).expect("the registration pointer"), pointer_before, "the registered worktree keeps its pointer");
    let mut names_after: Vec<String> = std::fs::read_dir(destination.as_path())
        .expect("listed")
        .map(|entry| return entry.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    names_after.sort();
    assert_eq!(names_after, names_before, "the registered worktree keeps everything that was in it");
    remove(root.as_path());
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

/// A snapshot that carried the source's ignored state needs no copy,
/// so a successful snapshot add is never reported as one that received none,
/// and the record of the add is settled because it is whole.
#[test]
fn btrfs_a_snapshot_add_that_carried_its_ignored_state_reports_no_missing_copy() {
    let Some(root) = crate::fs_btrfs_test_support::btrfs_fixture("lifecycle-snapshot-carried") else { return };
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
    // The snapshot carried the ignored state itself, so the worktree holds it and nothing is owed.
    assert_eq!(std::fs::read(topic.join("ignored/pkg/index.js")).expect("carried ignored state"), b"module.exports = 1;\n");
    assert!(!said.contains("received no ignored-state copy"), "a whole worktree is not reported as unfilled: {said:?}");
    assert_eq!(recorded_names(common.as_path()), Vec::<String>::new(), "a whole add leaves no record naming it unfilled");
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

/// A process that lives until the control ends,
/// killed and reaped however the control ends.
struct LiveProcess(std::process::Child);

impl LiveProcess {
    /// Start a `sleep` that outlives the call the control makes,
    /// which may take minutes on a contended disk:
    /// measured on 2026-11-09,
    /// one run of this control took 217 s,
    /// longer than the minute its owner was first given,
    /// so the owner's own ending would have been read as a wait.
    fn sleeping(seconds: &str) -> LiveProcess {
        let mut command: std::process::Command = std::process::Command::new("sleep");
        command.arg(seconds);
        let child: std::process::Child = crate::test_support::spawn_owned(&mut command).expect("the owner starts");
        return LiveProcess(child);
    }
}

impl Drop for LiveProcess {
    fn drop(&mut self) {
        let _killed = self.0.kill();
        let _reaped = self.0.wait();
    }
}

/// The owner a live process names,
/// as a journal of the copy that process is making would.
fn owner_named_by(child: &LiveProcess) -> crate::worktree_copy_owner::CopyOwner {
    let pid: i64 = i64::from(child.0.id());
    return crate::worktree_copy_owner::CopyOwner {
        pid,
        birth_identity: crate::process_identity::process_birth_identity(pid).expect("identity").expect("running"),
        pid_namespace: crate::pid_namespace::current_pid_namespace(),
    };
}

/// What:
///  A journal whose owner still runs belongs to a copy in progress:
///  a creation neither waits for it nor recovers it.
/// Why:
///   The clock this control used (under 30 s) could not tell a wait from a slow disk:
/// another agent reports it failing at 38.9 s and at 71 to 75 s in a suite while it passed alone in 6.0 to 8.9 s,
/// and this host's fixture disk stalls any step for seconds at a time.
/// The wait it was there to catch could only end when the copy's owner went away,
/// so the owner is now a process of its own that outlives the call:
/// the creation returning while that process still runs is the event that says nothing waited for it.
/// A wait that a later change added would not return while that process lives,
/// so this control would hang rather than fail,
/// and the unit-test watchdog ends a run that stops moving (`run_progress`).
#[cfg(unix)]
#[test]
fn a_creation_leaves_a_live_journal_alone_and_does_not_wait() {
    let root: PathBuf = fixture("lifecycle-live-journal");
    let (main, common) = source(root.as_path());
    let older: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("older");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "older", older.to_str().expect("utf8")]);
    let copy_in_progress: LiveProcess = LiveProcess::sleeping("86400");
    let owner: crate::worktree_copy_owner::CopyOwner = owner_named_by(&copy_in_progress);
    let live = journal_for(main.as_path(), common.as_path(), older.as_path(), &owner);
    let topic: PathBuf = root.join("topic");
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(topic.as_path())),
        &mut |_step: &GitStep<'_>| return Ok(run_real(main.as_path(), add_arguments("topic", topic.as_path()).as_slice())),
        &mut stderr,
    )
    .expect("settled");
    assert!(owner.is_alive(live.path.as_path()).expect("the owner is judged"), "the creator waited for the live copy's owner, which only ends when that process ends");
    assert_eq!(exit_code(&settlement), 0, "{:?}", settlement.copy_failure);
    assert!(live.path.exists(), "the live journal is whole");
    assert!(live.journal.stage_container.as_path().exists(), "its stage is whole");
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
    let claims_left: Vec<String> = recorded_names(common.as_path())
        .into_iter()
        .filter(|name: &String| return !name.ends_with(".json"))
        .collect();
    assert_eq!(
        claims_left.iter().filter(|name| return name.ends_with(".add")).count(),
        1,
        "the record of the destination this command did not fill stays, so a later add from it is held back: {claims_left:?}"
    );
    remove(root.as_path());
}

/// A journal whose owner is dead is recovered before the creation runs Git,
/// and reported.
/// The durability writes of its two settlements stay in the page cache,
/// because this control checks nothing about what a crash leaves.
#[cfg(unix)]
#[test]
fn a_creation_recovers_a_dead_journal_first() {
    keep_the_durability_writes_in_the_page_cache();
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

/// A Git that failed after registering the worktree leaves it unfilled and keeps the record that says so:
/// only a Git that returned success registered the worktree this command asked for,
/// and a later command reports the
///  unfilled one instead of treating it as whole.
#[cfg(unix)]
#[test]
fn a_creation_that_failed_after_registering_keeps_the_record_and_copies_nothing() {
    let root: PathBuf = fixture("lifecycle-failed-after-registering");
    let (main, common) = source(root.as_path());
    let topic: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &creation_from(main.as_path(), common.as_path(), Some(topic.as_path())),
        &mut |_: &GitStep<'_>| {
            run_real(main.as_path(), add_arguments("topic", topic.as_path()).as_slice());
            return Ok(GitStatus { code: Some(1) });
        },
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 1);
    assert_eq!(std::fs::read(topic.join("cache/data")).ok(), None, "nothing was copied into a worktree Git did not complete");
    let said: String = String::from_utf8(stderr).expect("utf8");
    assert!(said.contains("did not succeed"), "the person is told why: {said}");
    let records: Vec<String> = recorded_names(common.as_path());
    assert_eq!(records.iter().filter(|name| return name.ends_with(".add")).count(), 1, "the record of the unfilled worktree stays: {records:?}");
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

/// A worktree added from a source whose path is not text,
/// into a destination whose path is not text,
/// receives its ignored state exactly as it would on text paths.
#[cfg(unix)]
#[test]
fn a_worktree_add_between_paths_that_are_not_text_copies_the_ignored_state() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("lifecycle-add-bytes");
    let (main, common) = source(root.as_path());
    std::fs::write(common.join("info/exclude"), b"cache/\n").expect("exclude");
    let source_parent: PathBuf = root.join(std::ffi::OsStr::from_bytes(b"src\xff"));
    std::fs::create_dir_all(source_parent.as_path()).expect("source parent");
    let source_root: PathBuf = source_parent.join("source");
    let arguments: Vec<OsString> = ["worktree", "add", "--quiet", "-b", "source"]
        .iter()
        .map(OsString::from)
        .chain([source_root.clone().into_os_string()])
        .collect();
    git(main.as_path(), arguments.as_slice());
    let source_canonical: PathBuf = std::fs::canonicalize(source_root).expect("canonical source");
    std::fs::create_dir_all(source_canonical.join("cache")).expect("cache");
    std::fs::write(source_canonical.join("cache/data"), b"payload").expect("data");
    let destination_parent: PathBuf = root.join(std::ffi::OsStr::from_bytes(b"dst\xff"));
    std::fs::create_dir_all(destination_parent.as_path()).expect("destination parent");
    let topic: PathBuf = destination_parent.join("topic");
    let identity: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: common.clone(),
        git_dir: common.join("worktrees").join("source"),
        worktree_root: source_canonical.clone(),
    };
    let mut stderr: Vec<u8> = Vec::new();
    let settlement: Settlement = run_with_worktree_copy(
        &Invocation {
            identity,
            effective_cwd: source_canonical.clone(),
            creates: true,
            destination: Some(topic.clone()),
            opted_out: false,
            short_circuit: false,
            context: git_context(),
            add_plan: None,
            repair: crate::package_repair::RepairSwitch::default(),
        },
        &mut |_step: &GitStep<'_>| {
            let add_arguments: Vec<OsString> = ["worktree", "add", "--quiet", "-b", "topic"]
                .iter()
                .map(OsString::from)
                .chain([topic.clone().into_os_string()])
                .collect();
            let output: Output = git_output(source_canonical.as_path(), add_arguments.as_slice());
            return Ok(GitStatus { code: output.status.code() });
        },
        &mut stderr,
    )
    .expect("settled");
    assert_eq!(exit_code(&settlement), 0, "{}", String::from_utf8_lossy(stderr.as_slice()));
    assert_eq!(std::fs::read(topic.join("cache/data")).expect("copied"), b"payload");
    let replacement: PathBuf = PathBuf::from(topic.to_string_lossy().into_owned());
    assert!(!replacement.exists(), "nothing is written at the replacement spelling");
    assert!(read_pending_journals(common.as_path()).expect("pending").is_empty());
    remove(root.as_path());
}
