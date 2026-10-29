//! Controls for post-landing completion: the step order, the failure semantics of each step,
//! and the maintenance exemption from failing the landed commit.

use super::*;
use crate::hook_dispatch_plan::ConfigParameters;
use crate::recovery_landing::tests::Scene;
use crate::test_support::{executable, git_context, git_text};
use std::ffi::OsString;

/// Writes a stand-in Git that logs each invocation, answers `config` with `config_exit`, and otherwise runs real Git.
fn logging_git(root: &std::path::Path, config_exit: u8) -> (PathBuf, PathBuf) {
    let log: PathBuf = root.join("calls.log");
    let path: PathBuf = root.join("git");
    let config_line: String = if config_exit == 0 {
        String::new()
    } else {
        format!("case \" $* \" in *\" config \"*) exit {config_exit};; esac\n")
    };
    let content: String = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\n{config_line}exec {} \"$@\"\n",
        log.display(),
        crate::test_support::REAL_GIT,
    );
    executable(path.as_path(), content.as_bytes());
    return (path, log);
}

/// A plan whose hook lock sits at `lock`, with the given disabled events.
fn plan(lock: &std::path::Path, disabled: &[&str]) -> HookDispatchPlan {
    return HookDispatchPlan {
        git_path: String::from(crate::test_support::REAL_GIT),
        hooks_path: String::new(),
        disabled_events: disabled
            .iter()
            .map(|name| return String::from(*name))
            .collect(),
        config_parameters: ConfigParameters::Absent,
        worktree_root: String::new(),
        lease: String::new(),
        hook_lock_path: lock.display().to_string(),
        skip_hook_lock: false,
        named_hooks_possible: true,
    };
}

/// A landed transaction over a scene, with `HEAD` as the landed commit.
fn landed(scene: &Scene, records: Vec<AddedPath>) -> LandedTransaction {
    return LandedTransaction {
        landed_oid: git_text(scene.repo.as_path(), &["rev-parse", "HEAD"]),
        branch: String::from("main"),
        cwd: scene.repo.clone(),
        repository_root: scene.repo.clone(),
        directory: scene.directory.clone(),
        shadow: scene.shadow.clone(),
        git_dir: scene.git_dir.clone(),
        registry_root: scene.registry.clone(),
        real_index: scene.git_dir.join("index"),
        added_paths: Vec::new(),
        worktree_records: records,
        config_snapshot: None,
    };
}

/// The command names of the logged invocations, in order, so the test asserts the sequence.
fn sequence(log: &std::path::Path) -> Vec<String> {
    return std::fs::read_to_string(log)
        .unwrap_or_default()
        .lines()
        .map(String::from)
        .collect();
}

/// Position of the first logged invocation containing `needle`.
fn position(lines: &[String], needle: &str) -> Option<usize> {
    return lines.iter().position(|line| return line.contains(needle));
}

/// Every step runs in the incumbent's order: maintenance reads its config, runs, then `post-commit` reads the author and runs its hook.
#[test]
fn completion_runs_maintenance_then_the_hook_after_disposal() {
    let scene: Scene = Scene::new("post-landing-order", false);
    std::fs::create_dir_all(scene.shadow.join("objects")).expect("shadow");
    let (stand_in, log): (PathBuf, PathBuf) = logging_git(scene.root.as_path(), 0);
    let mut context = git_context();
    context.real_git = stand_in;
    context.global_prefix = vec![
        OsString::from("-c"),
        OsString::from("maintenance.autodetach=false"),
    ];
    let lock: PathBuf = scene.git_dir.join("cli-git").join("hook.lock");
    let hook_plan: HookDispatchPlan = plan(lock.as_path(), &[]);
    let transaction: LandedTransaction = landed(&scene, Vec::new());
    let outcome: PostLandingOutcome = complete_landed_transaction(PostLandingRequest {
        context: &context,
        plan: &hook_plan,
        landed: &transaction,
    })
    .expect("completed");
    assert!(!scene.shadow.exists());
    assert!(!scene.directory.exists());
    assert_eq!(outcome.landed_oid, transaction.landed_oid);
    assert_eq!(outcome.maintenance, Some(AutoMaintenanceOutcome::Exited(0)));
    assert_eq!(outcome.hook, PostCommitHookOutcome::Ran(0));
    let lines: Vec<String> = sequence(log.as_path());
    let maintenance: usize =
        position(lines.as_slice(), "maintenance run --auto").expect("maintenance started");
    let show: usize = position(lines.as_slice(), "show --no-patch").expect("author read");
    let hook: usize =
        position(lines.as_slice(), "hook run --ignore-missing post-commit").expect("hook started");
    assert!(maintenance < show && show < hook, "{lines:?}");
    scene.finish();
}

/// A copy that cannot be completed stops the sequence: the shadow and the directory stay for recovery,
/// and neither maintenance nor the hook runs.
#[test]
fn an_install_failure_keeps_the_shadow_and_the_directory() {
    let scene: Scene = Scene::new("post-landing-install-fails", false);
    std::fs::create_dir_all(scene.shadow.join("objects")).expect("shadow");
    let (stand_in, log): (PathBuf, PathBuf) = logging_git(scene.root.as_path(), 0);
    let mut context = git_context();
    context.real_git = stand_in;
    let lock: PathBuf = scene.git_dir.join("cli-git").join("hook.lock");
    let hook_plan: HookDispatchPlan = plan(lock.as_path(), &[]);
    let missing: String = String::from("1111111111111111111111111111111111111111");
    let records: Vec<AddedPath> = vec![AddedPath {
        path: b"added.txt".to_vec(),
        git_mode: String::from("100644"),
        original_oid: missing.clone(),
        intended_oid: missing,
    }];
    let transaction: LandedTransaction = landed(&scene, records);
    let failure: PostLandingError = complete_landed_transaction(PostLandingRequest {
        context: &context,
        plan: &hook_plan,
        landed: &transaction,
    })
    .expect_err("install failure");
    assert!(
        matches!(failure, PostLandingError::Install(_)),
        "{failure:?}"
    );
    assert!(scene.shadow.exists());
    assert!(scene.directory.exists());
    let lines: Vec<String> = sequence(log.as_path());
    assert!(
        position(lines.as_slice(), "maintenance run").is_none(),
        "{lines:?}"
    );
    assert!(
        position(lines.as_slice(), "hook run").is_none(),
        "{lines:?}"
    );
    scene.finish();
}

/// A maintenance config read that fails is logged and never fails the landed commit; the hook still runs.
#[test]
fn a_maintenance_failure_never_fails_the_landed_commit() {
    let scene: Scene = Scene::new("post-landing-maintenance-fails", false);
    std::fs::create_dir_all(scene.shadow.join("objects")).expect("shadow");
    let (stand_in, log): (PathBuf, PathBuf) = logging_git(scene.root.as_path(), 2);
    let mut context = git_context();
    context.real_git = stand_in;
    let lock: PathBuf = scene.git_dir.join("cli-git").join("hook.lock");
    let hook_plan: HookDispatchPlan = plan(lock.as_path(), &[]);
    let transaction: LandedTransaction = landed(&scene, Vec::new());
    let outcome: PostLandingOutcome = complete_landed_transaction(PostLandingRequest {
        context: &context,
        plan: &hook_plan,
        landed: &transaction,
    })
    .expect("completed despite maintenance");
    assert_eq!(outcome.maintenance, Some(AutoMaintenanceOutcome::Exited(2)));
    assert_eq!(outcome.hook, PostCommitHookOutcome::Ran(0));
    let lines: Vec<String> = sequence(log.as_path());
    assert!(
        position(lines.as_slice(), "hook run").is_some(),
        "{lines:?}"
    );
    scene.finish();
}

/// A hook lock that cannot be taken fails after disposal, as the incumbent's thrown error does after its cleanup.
#[test]
fn a_hook_lock_failure_is_reported_after_the_transaction_is_disposed() {
    let scene: Scene = Scene::new("post-landing-lock-fails", false);
    std::fs::create_dir_all(scene.shadow.join("objects")).expect("shadow");
    let (stand_in, _log): (PathBuf, PathBuf) = logging_git(scene.root.as_path(), 0);
    let mut context = git_context();
    context.real_git = stand_in;
    let blocker: PathBuf = scene.git_dir.join("blocker");
    std::fs::write(blocker.as_path(), b"not a directory").expect("blocker");
    let hook_plan: HookDispatchPlan = plan(blocker.join("hook.lock").as_path(), &[]);
    let transaction: LandedTransaction = landed(&scene, Vec::new());
    let failure: PostLandingError = complete_landed_transaction(PostLandingRequest {
        context: &context,
        plan: &hook_plan,
        landed: &transaction,
    })
    .expect_err("lock failure");
    assert!(matches!(failure, PostLandingError::Hook(_)), "{failure:?}");
    assert!(!scene.shadow.exists());
    assert!(!scene.directory.exists());
    scene.finish();
}

/// A disabled `post-commit` leaves the rest of the completion in place and reports the event as disabled.
#[test]
fn a_disabled_post_commit_still_completes_the_rest() {
    let scene: Scene = Scene::new("post-landing-disabled-hook", false);
    std::fs::create_dir_all(scene.shadow.join("objects")).expect("shadow");
    let (stand_in, log): (PathBuf, PathBuf) = logging_git(scene.root.as_path(), 0);
    let mut context = git_context();
    context.real_git = stand_in;
    let lock: PathBuf = scene.git_dir.join("cli-git").join("hook.lock");
    let hook_plan: HookDispatchPlan = plan(lock.as_path(), &["post-commit"]);
    let transaction: LandedTransaction = landed(&scene, Vec::new());
    let outcome: PostLandingOutcome = complete_landed_transaction(PostLandingRequest {
        context: &context,
        plan: &hook_plan,
        landed: &transaction,
    })
    .expect("completed");
    assert_eq!(outcome.hook, PostCommitHookOutcome::Disabled);
    assert!(!scene.directory.exists());
    let lines: Vec<String> = sequence(log.as_path());
    assert!(
        position(lines.as_slice(), "hook run").is_none(),
        "{lines:?}"
    );
    scene.finish();
}

/// Writes a blob with `content` into the real object store, then moves its loose file into the shadow's object store,
/// so only `shadow/objects` can serve it, as it does for an object a transaction wrote before landing.
fn blob_only_in_shadow(scene: &Scene, content: &str) -> String {
    let source: PathBuf = scene.root.join("blob-source");
    std::fs::write(source.as_path(), content).expect("blob source");
    let oid: String = git_text(
        scene.repo.as_path(),
        &[
            "hash-object",
            "-w",
            "--",
            source.to_str().expect("UTF-8 path"),
        ],
    );
    let (fan_out, rest): (&str, &str) = oid.split_at(2);
    let from: PathBuf = scene.git_dir.join("objects").join(fan_out).join(rest);
    let to_directory: PathBuf = scene.shadow.join("objects").join(fan_out);
    std::fs::create_dir_all(to_directory.as_path()).expect("shadow fan-out");
    std::fs::rename(from, to_directory.join(rest)).expect("moved loose object");
    return oid;
}

/// A record whose worktree copy holds `original` and whose landed bytes are `intended`, both served by the shadow only.
fn shadow_record(scene: &Scene, path: &str, original: &str, intended: &str) -> AddedPath {
    std::fs::write(scene.repo.join(path), original).expect("worktree copy");
    return AddedPath {
        path: path.as_bytes().to_vec(),
        git_mode: String::from("100644"),
        original_oid: blob_only_in_shadow(scene, original),
        intended_oid: blob_only_in_shadow(scene, intended),
    };
}

/// Both record lists are completed, the added paths and the worktree records, from the shadow's objects,
/// and the shadow is removed only afterwards.
#[test]
fn both_record_lists_are_copied_from_the_shadow_before_it_is_removed() {
    let scene: Scene = Scene::new("post-landing-copies", false);
    let (stand_in, _log): (PathBuf, PathBuf) = logging_git(scene.root.as_path(), 0);
    let mut context = git_context();
    context.real_git = stand_in;
    let lock: PathBuf = scene.git_dir.join("cli-git").join("hook.lock");
    let hook_plan: HookDispatchPlan = plan(lock.as_path(), &["post-commit"]);
    let mut transaction: LandedTransaction = landed(&scene, Vec::new());
    transaction.added_paths = vec![shadow_record(
        &scene,
        "added.txt",
        "added before\n",
        "added after\n",
    )];
    transaction.worktree_records = vec![shadow_record(
        &scene,
        "fixed.txt",
        "fixed before\n",
        "fixed after\n",
    )];
    let outcome: PostLandingOutcome = complete_landed_transaction(PostLandingRequest {
        context: &context,
        plan: &hook_plan,
        landed: &transaction,
    })
    .expect("completed");
    assert_eq!(
        outcome.worktree.rewritten,
        vec![b"added.txt".to_vec(), b"fixed.txt".to_vec()]
    );
    assert!(outcome.worktree.conflicted.is_empty());
    assert_eq!(
        std::fs::read_to_string(scene.repo.join("added.txt")).expect("added copy"),
        "added after\n"
    );
    assert_eq!(
        std::fs::read_to_string(scene.repo.join("fixed.txt")).expect("fixed copy"),
        "fixed after\n"
    );
    assert!(!scene.shadow.exists());
    scene.finish();
}

/// A copy the user changed meanwhile is kept and reported, and the completion still disposes and runs the hook.
#[test]
fn a_concurrently_changed_copy_is_kept_without_failing_the_commit() {
    let scene: Scene = Scene::new("post-landing-conflict", false);
    let (stand_in, _log): (PathBuf, PathBuf) = logging_git(scene.root.as_path(), 0);
    let mut context = git_context();
    context.real_git = stand_in;
    let lock: PathBuf = scene.git_dir.join("cli-git").join("hook.lock");
    let hook_plan: HookDispatchPlan = plan(lock.as_path(), &[]);
    let mut transaction: LandedTransaction = landed(&scene, Vec::new());
    let record: AddedPath = shadow_record(&scene, "edited.txt", "before\n", "after\n");
    std::fs::write(scene.repo.join("edited.txt"), "the user's edit\n").expect("user edit");
    transaction.added_paths = vec![record];
    let outcome: PostLandingOutcome = complete_landed_transaction(PostLandingRequest {
        context: &context,
        plan: &hook_plan,
        landed: &transaction,
    })
    .expect("completed");
    assert_eq!(
        outcome.worktree.conflicted,
        vec![b"edited.txt".to_vec()]
    );
    assert!(outcome.worktree.rewritten.is_empty());
    assert_eq!(
        std::fs::read_to_string(scene.repo.join("edited.txt")).expect("kept edit"),
        "the user's edit\n"
    );
    assert!(!scene.directory.exists());
    assert_eq!(outcome.hook, PostCommitHookOutcome::Ran(0));
    scene.finish();
}

/// A shadow that cannot be removed fails the completion before maintenance and the hook start.
#[cfg(unix)]
#[test]
fn a_disposal_failure_stops_before_maintenance_and_the_hook() {
    use std::os::unix::fs::PermissionsExt;
    let scene: Scene = Scene::new("post-landing-dispose-fails", false);
    std::fs::create_dir_all(scene.shadow.join("objects")).expect("shadow");
    let (stand_in, log): (PathBuf, PathBuf) = logging_git(scene.root.as_path(), 0);
    let mut context = git_context();
    context.real_git = stand_in;
    let lock: PathBuf = scene.git_dir.join("cli-git").join("hook.lock");
    let hook_plan: HookDispatchPlan = plan(lock.as_path(), &[]);
    let transaction: LandedTransaction = landed(&scene, Vec::new());
    let parent: PathBuf = scene.shadow.parent().expect("shadow parent").to_path_buf();
    std::fs::set_permissions(parent.as_path(), std::fs::Permissions::from_mode(0o555))
        .expect("read-only parent");
    let result = complete_landed_transaction(PostLandingRequest {
        context: &context,
        plan: &hook_plan,
        landed: &transaction,
    });
    std::fs::set_permissions(parent.as_path(), std::fs::Permissions::from_mode(0o755))
        .expect("restored parent");
    let failure: PostLandingError = result.expect_err("disposal failure");
    assert!(
        matches!(failure, PostLandingError::Dispose(_)),
        "{failure:?}"
    );
    assert!(scene.directory.exists());
    let lines: Vec<String> = sequence(log.as_path());
    assert!(
        position(lines.as_slice(), "maintenance run").is_none(),
        "{lines:?}"
    );
    assert!(
        position(lines.as_slice(), "hook run").is_none(),
        "{lines:?}"
    );
    scene.finish();
}

/// Maintenance whose config output cannot be decoded is logged and skipped; the hook still runs and the commit stands.
#[test]
fn an_undecodable_maintenance_config_is_skipped_and_the_hook_still_runs() {
    let scene: Scene = Scene::new("post-landing-maintenance-undecodable", false);
    std::fs::create_dir_all(scene.shadow.join("objects")).expect("shadow");
    let log: PathBuf = scene.root.join("calls.log");
    let path: PathBuf = scene.root.join("git");
    executable(
        path.as_path(),
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\ncase \" $* \" in *\" config \"*) printf '\\377\\376'; exit 0;; esac\nexec {} \"$@\"\n",
            log.display(),
            crate::test_support::REAL_GIT,
        )
        .as_bytes(),
    );
    let mut context = git_context();
    context.real_git = path;
    let lock: PathBuf = scene.git_dir.join("cli-git").join("hook.lock");
    let hook_plan: HookDispatchPlan = plan(lock.as_path(), &[]);
    let transaction: LandedTransaction = landed(&scene, Vec::new());
    let outcome: PostLandingOutcome = complete_landed_transaction(PostLandingRequest {
        context: &context,
        plan: &hook_plan,
        landed: &transaction,
    })
    .expect("completed despite maintenance");
    assert_eq!(outcome.maintenance, None);
    assert_eq!(outcome.hook, PostCommitHookOutcome::Ran(0));
    let lines: Vec<String> = sequence(log.as_path());
    assert!(
        position(lines.as_slice(), "maintenance run").is_none(),
        "{lines:?}"
    );
    scene.finish();
}

/// A hook that exits non-zero is reported in the outcome and never fails the landed commit.
#[test]
fn a_failing_hook_never_fails_the_landed_commit() {
    let scene: Scene = Scene::new("post-landing-hook-exits", false);
    std::fs::create_dir_all(scene.shadow.join("objects")).expect("shadow");
    executable(
        scene.git_dir.join("hooks").join("post-commit").as_path(),
        b"#!/bin/sh\nexit 3\n",
    );
    let context = git_context();
    let lock: PathBuf = scene.git_dir.join("cli-git").join("hook.lock");
    let hook_plan: HookDispatchPlan = plan(lock.as_path(), &[]);
    let transaction: LandedTransaction = landed(&scene, Vec::new());
    let outcome: PostLandingOutcome = complete_landed_transaction(PostLandingRequest {
        context: &context,
        plan: &hook_plan,
        landed: &transaction,
    })
    .expect("completed");
    assert_eq!(outcome.hook, PostCommitHookOutcome::Ran(3));
    assert!(!scene.directory.exists());
    scene.finish();
}

/// An author lookup that fails is reported after disposal and maintenance, and the hook never starts.
#[test]
fn an_unreadable_landed_commit_fails_after_disposal_and_maintenance() {
    let scene: Scene = Scene::new("post-landing-author-fails", false);
    std::fs::create_dir_all(scene.shadow.join("objects")).expect("shadow");
    let (stand_in, log): (PathBuf, PathBuf) = logging_git(scene.root.as_path(), 0);
    let mut context = git_context();
    context.real_git = stand_in;
    let lock: PathBuf = scene.git_dir.join("cli-git").join("hook.lock");
    let hook_plan: HookDispatchPlan = plan(lock.as_path(), &[]);
    let mut transaction: LandedTransaction = landed(&scene, Vec::new());
    transaction.landed_oid = String::from("0123456789abcdef0123456789abcdef01234567");
    let failure: PostLandingError = complete_landed_transaction(PostLandingRequest {
        context: &context,
        plan: &hook_plan,
        landed: &transaction,
    })
    .expect_err("author lookup failure");
    assert!(matches!(failure, PostLandingError::Hook(_)), "{failure:?}");
    assert!(!scene.directory.exists());
    let lines: Vec<String> = sequence(log.as_path());
    assert!(
        position(lines.as_slice(), "maintenance run").is_some(),
        "{lines:?}"
    );
    assert!(
        position(lines.as_slice(), "hook run").is_none(),
        "{lines:?}"
    );
    scene.finish();
}

/// Each failure variant prints its own message without a step prefix, as the incumbent prints a thrown error.
#[test]
fn failures_display_their_message_alone() {
    assert_eq!(
        PostLandingError::Install(RecoveryError(String::from("copy"))).to_string(),
        "copy"
    );
    assert_eq!(
        PostLandingError::Dispose(RecoveryError(String::from("remove"))).to_string(),
        "remove"
    );
    assert_eq!(
        PostLandingError::Hook(PostCommitHookError(String::from("hook"))).to_string(),
        "hook"
    );
}
