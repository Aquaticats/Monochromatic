//! Controls for the post-commit run: the author environment, the disabled event, the hook lock,
//! the hook's environment and status, and the failures before the hook starts.

use super::*;
use crate::owner_lock::{DEFAULT_POLL_DELAY, SilentWait, acquire_owner_lock};
use crate::hook_dispatch_plan::ConfigParameters;
use crate::test_support::{executable, fixture, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// Builds a plan whose only meaningful fields are the disabled events and the hook lock.
fn plan(lock: &Path, skip_lock: bool, disabled: &[&str]) -> HookDispatchPlan {
    return HookDispatchPlan {
        git_path: String::from(crate::test_support::REAL_GIT),
        hooks_path: String::new(),
        disabled_events: disabled
            .iter()
            .map(|name| {
                return String::from(*name);
            })
            .collect(),
        config_parameters: ConfigParameters::Absent,
        worktree_root: String::new(),
        lease: String::new(),
        hook_lock_path: lock.display().to_string(),
        skip_hook_lock: skip_lock,
        named_hooks_possible: true,
    };
}

/// Installs a `post-commit` hook that records its environment, whether the lock exists, and its own exit.
fn install_hook(repo: &Path, lock: &Path, record: &Path, exit_code: u8) {
    let script: String = format!(
        "#!/bin/sh\n{{\n  printf 'name=%s\\n' \"$GIT_AUTHOR_NAME\"\n  printf 'email=%s\\n' \"$GIT_AUTHOR_EMAIL\"\n  printf 'date=%s\\n' \"$GIT_AUTHOR_DATE\"\n  printf 'index=%s\\n' \"$GIT_INDEX_FILE\"\n  printf 'editor=%s\\n' \"$GIT_EDITOR\"\n  if [ -d '{}' ]; then printf 'locked=yes\\n'; else printf 'locked=no\\n'; fi\n}} > '{}'\nexit {exit_code}\n",
        lock.display(),
        record.display(),
    );
    executable(
        repo.join(".git")
            .join("hooks")
            .join("post-commit")
            .as_path(),
        script.as_bytes(),
    );
}

/// The fields the hook recorded, as text.
fn recorded(record: &Path) -> String {
    return std::fs::read_to_string(record).expect("hook record");
}

/// Writes a stand-in Git that logs each invocation's arguments and then runs real Git with them.
fn logging_git(root: &Path) -> (PathBuf, PathBuf) {
    let log: PathBuf = root.join("calls.log");
    let path: PathBuf = root.join("git");
    let content: String = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexec {} \"$@\"\n",
        log.display(),
        crate::test_support::REAL_GIT,
    );
    executable(path.as_path(), content.as_bytes());
    return (path, log);
}

/// Author fields are read as Git's NUL-separated format, and one trailing newline is dropped.
#[test]
fn author_environment_reads_name_email_and_raw_date() {
    assert_eq!(
        author_environment("Jane Roe\u{0}jane@example.invalid\u{0}1700000000 +0000\n"),
        vec![
            (String::from("GIT_AUTHOR_NAME"), String::from("Jane Roe")),
            (
                String::from("GIT_AUTHOR_EMAIL"),
                String::from("jane@example.invalid")
            ),
            (
                String::from("GIT_AUTHOR_DATE"),
                String::from("@1700000000 +0000")
            ),
        ]
    );
}

/// Missing fields are empty strings, as the incumbent's destructuring defaults make them.
#[test]
fn author_environment_defaults_missing_fields_to_empty() {
    assert_eq!(
        author_environment("Only\0"),
        vec![
            (String::from("GIT_AUTHOR_NAME"), String::from("Only")),
            (String::from("GIT_AUTHOR_EMAIL"), String::new()),
            (String::from("GIT_AUTHOR_DATE"), String::from("@")),
        ]
    );
}

/// A disabled event runs nothing: no author lookup, no lock, no hook.
#[test]
fn disabled_post_commit_runs_nothing() {
    let root: PathBuf = fixture("post-commit-disabled");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let lock: PathBuf = repo.join(".git").join("cli-git").join("hook.lock");
    let record: PathBuf = root.join("hook.txt");
    install_hook(repo.as_path(), lock.as_path(), record.as_path(), 0);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let disabled_plan: HookDispatchPlan = plan(lock.as_path(), false, &["post-commit"]);
    let context = git_context();
    let outcome: PostCommitHookOutcome = run_post_commit_hook(PostCommitHookRequest {
        context: &context,
        worktree: repo.as_path(),
        real_index: repo.join(".git").join("index").as_path(),
        landed_oid: landed.as_str(),
        plan: &disabled_plan,
    })
    .expect("disabled run");
    assert_eq!(outcome, PostCommitHookOutcome::Disabled);
    assert!(!record.exists());
    remove(root.as_path());
}

/// An event with no user hook runs nothing: no author lookup, no lock and no hook runner start.
#[test]
fn an_absent_post_commit_hook_starts_nothing() {
    let root: PathBuf = fixture("post-commit-absent");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let lock: PathBuf = repo.join(".git").join("cli-git").join("hook.lock");
    let (stand_in, log) = logging_git(root.as_path());
    let context: GitContext = GitContext {
        real_git: stand_in,
        overlay: Vec::new(),
        global_prefix: Vec::new(),
    };
    let absent_plan: HookDispatchPlan = HookDispatchPlan {
        hooks_path: repo.join(".git").join("hooks").display().to_string(),
        named_hooks_possible: false,
        ..plan(lock.as_path(), false, &[])
    };
    let outcome: PostCommitHookOutcome = run_post_commit_hook(PostCommitHookRequest {
        context: &context,
        worktree: repo.as_path(),
        real_index: repo.join(".git").join("index").as_path(),
        landed_oid: "unused",
        plan: &absent_plan,
    })
    .expect("absent run");
    assert_eq!(outcome, PostCommitHookOutcome::Absent);
    assert!(!log.exists(), "nothing started: {:?}", std::fs::read_to_string(log.as_path()));
    assert!(!lock.exists());
    remove(root.as_path());
}

/// A post-commit file that is not executable, or a configuration that names hooks, still runs the hook runner,
///  which prints Git's hint or runs the named hook.
#[test]
fn a_non_executable_or_named_post_commit_hook_still_starts_the_runner() {
    let root: PathBuf = fixture("post-commit-kept");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let lock: PathBuf = repo.join(".git").join("cli-git").join("hook.lock");
    let hooks: PathBuf = repo.join(".git").join("hooks");
    std::fs::write(hooks.join("post-commit"), b"#!/bin/sh\n").expect("plain file");
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let context = git_context();
    for named_hooks_possible in [false, true] {
        let kept_plan: HookDispatchPlan = HookDispatchPlan {
            hooks_path: hooks.display().to_string(),
            named_hooks_possible,
            ..plan(lock.as_path(), true, &[])
        };
        let outcome: PostCommitHookOutcome = run_post_commit_hook(PostCommitHookRequest {
            context: &context,
            worktree: repo.as_path(),
            real_index: repo.join(".git").join("index").as_path(),
            landed_oid: landed.as_str(),
            plan: &kept_plan,
        })
        .expect("run");
        assert_eq!(outcome, PostCommitHookOutcome::Ran(0), "named {named_hooks_possible}");
    }
    std::fs::remove_file(hooks.join("post-commit")).expect("remove the file");
    let named_plan: HookDispatchPlan = HookDispatchPlan {
        hooks_path: hooks.display().to_string(),
        named_hooks_possible: true,
        ..plan(lock.as_path(), true, &[])
    };
    let outcome: PostCommitHookOutcome = run_post_commit_hook(PostCommitHookRequest {
        context: &context,
        worktree: repo.as_path(),
        real_index: repo.join(".git").join("index").as_path(),
        landed_oid: landed.as_str(),
        plan: &named_plan,
    })
    .expect("run");
    assert_eq!(outcome, PostCommitHookOutcome::Ran(0));
    remove(root.as_path());
}

/// The hook sees the author, the real index, the editor override, and the held lock.
#[test]
fn hook_runs_under_the_lock_with_the_native_environment() {
    let root: PathBuf = fixture("post-commit-environment");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let lock: PathBuf = repo.join(".git").join("cli-git").join("hook.lock");
    let record: PathBuf = root.join("hook.txt");
    install_hook(repo.as_path(), lock.as_path(), record.as_path(), 0);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let index: PathBuf = repo.join(".git").join("index");
    let locked_plan: HookDispatchPlan = plan(lock.as_path(), false, &[]);
    let context = git_context();
    let outcome: PostCommitHookOutcome = run_post_commit_hook(PostCommitHookRequest {
        context: &context,
        worktree: repo.as_path(),
        real_index: index.as_path(),
        landed_oid: landed.as_str(),
        plan: &locked_plan,
    })
    .expect("run");
    assert_eq!(outcome, PostCommitHookOutcome::Ran(0));
    let text: String = recorded(record.as_path());
    assert!(text.contains("locked=yes"), "{text}");
    assert!(
        text.contains(format!("index={}", index.display()).as_str()),
        "{text}"
    );
    assert!(text.contains("editor=:"), "{text}");
    let author: String = git_text(
        repo.as_path(),
        &["show", "--no-patch", "--format=%an", landed.as_str()],
    );
    assert!(text.contains(format!("name={author}").as_str()), "{text}");
    assert!(!lock.exists(), "the hook lock is released after the hook");
    remove(root.as_path());
}

/// A skipped hook lock, from concurrent commits or an inherited lease, runs the hook without the lock.
#[test]
fn skipped_hook_lock_runs_without_the_lock() {
    let root: PathBuf = fixture("post-commit-skip-lock");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let lock: PathBuf = repo.join(".git").join("cli-git").join("hook.lock");
    let record: PathBuf = root.join("hook.txt");
    install_hook(repo.as_path(), lock.as_path(), record.as_path(), 0);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let skipped_plan: HookDispatchPlan = plan(lock.as_path(), true, &[]);
    let context = git_context();
    run_post_commit_hook(PostCommitHookRequest {
        context: &context,
        worktree: repo.as_path(),
        real_index: repo.join(".git").join("index").as_path(),
        landed_oid: landed.as_str(),
        plan: &skipped_plan,
    })
    .expect("run");
    assert!(recorded(record.as_path()).contains("locked=no"));
    remove(root.as_path());
}

/// A non-zero hook status is returned for the caller to ignore; the run itself succeeds.
#[test]
fn a_failing_hook_status_is_reported_not_raised() {
    let root: PathBuf = fixture("post-commit-status");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let lock: PathBuf = repo.join(".git").join("cli-git").join("hook.lock");
    let record: PathBuf = root.join("hook.txt");
    install_hook(repo.as_path(), lock.as_path(), record.as_path(), 3);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let skipped_plan: HookDispatchPlan = plan(lock.as_path(), true, &[]);
    let context = git_context();
    let outcome: PostCommitHookOutcome = run_post_commit_hook(PostCommitHookRequest {
        context: &context,
        worktree: repo.as_path(),
        real_index: repo.join(".git").join("index").as_path(),
        landed_oid: landed.as_str(),
        plan: &skipped_plan,
    })
    .expect("run");
    assert_eq!(outcome, PostCommitHookOutcome::Ran(3));
    remove(root.as_path());
}

/// A lock that cannot be created fails the run before the hook starts.
#[test]
fn a_lock_that_cannot_be_taken_fails_before_the_hook() {
    let root: PathBuf = fixture("post-commit-lock-fails");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let blocker: PathBuf = repo.join(".git").join("blocker");
    std::fs::write(blocker.as_path(), b"not a directory").expect("blocker");
    let lock: PathBuf = blocker.join("hook.lock");
    let record: PathBuf = root.join("hook.txt");
    install_hook(repo.as_path(), lock.as_path(), record.as_path(), 0);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let locked_plan: HookDispatchPlan = plan(lock.as_path(), false, &[]);
    let context = git_context();
    let failure: PostCommitHookError = run_post_commit_hook(PostCommitHookRequest {
        context: &context,
        worktree: repo.as_path(),
        real_index: repo.join(".git").join("index").as_path(),
        landed_oid: landed.as_str(),
        plan: &locked_plan,
    })
    .expect_err("lock failure");
    assert!(!failure.0.is_empty());
    assert!(!record.exists(), "the hook must not start without its lock");
    remove(root.as_path());
}

/// An author lookup that Git refuses fails the run, and the hook never starts.
#[test]
fn an_unreadable_landed_commit_fails_before_the_hook() {
    let root: PathBuf = fixture("post-commit-author-fails");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let lock: PathBuf = repo.join(".git").join("cli-git").join("hook.lock");
    let record: PathBuf = root.join("hook.txt");
    install_hook(repo.as_path(), lock.as_path(), record.as_path(), 0);
    let missing: &str = "0123456789abcdef0123456789abcdef01234567";
    let skipped_plan: HookDispatchPlan = plan(lock.as_path(), true, &[]);
    let context = git_context();
    let failure: PostCommitHookError = run_post_commit_hook(PostCommitHookRequest {
        context: &context,
        worktree: repo.as_path(),
        real_index: repo.join(".git").join("index").as_path(),
        landed_oid: missing,
        plan: &skipped_plan,
    })
    .expect_err("author lookup");
    assert!(!failure.0.is_empty());
    assert!(!record.exists());
    remove(root.as_path());
}

/// Each invocation the run makes goes through the logging stand-in, so the order and the arguments are visible.
#[test]
fn the_run_starts_show_then_the_hook_runner_with_the_global_options() {
    let root: PathBuf = fixture("post-commit-argv");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let lock: PathBuf = repo.join(".git").join("cli-git").join("hook.lock");
    let record: PathBuf = root.join("hook.txt");
    install_hook(repo.as_path(), lock.as_path(), record.as_path(), 0);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let (stand_in, log): (PathBuf, PathBuf) = logging_git(root.as_path());
    let mut context = git_context();
    context.real_git = stand_in;
    context.global_prefix = vec![
        std::ffi::OsString::from("-c"),
        std::ffi::OsString::from("maintenance.auto=false"),
    ];
    let skipped_plan: HookDispatchPlan = plan(lock.as_path(), true, &[]);
    run_post_commit_hook(PostCommitHookRequest {
        context: &context,
        worktree: repo.as_path(),
        real_index: repo.join(".git").join("index").as_path(),
        landed_oid: landed.as_str(),
        plan: &skipped_plan,
    })
    .expect("run");
    let calls: String = std::fs::read_to_string(log).expect("calls");
    let lines: Vec<&str> = calls.lines().collect();
    assert_eq!(lines.len(), 2, "{calls}");
    assert!(
        lines[0].starts_with("show --no-patch --date=raw"),
        "{calls}"
    );
    assert_eq!(
        lines[1], "-c maintenance.auto=false hook run --ignore-missing post-commit",
        "{calls}"
    );
    remove(root.as_path());
}

/// A hook lock a live owner holds delays the hook until the owner releases it, so concurrent commits' hooks never overlap.
#[test]
fn the_hook_waits_while_another_owner_holds_the_lock() {
    let root: PathBuf = fixture("post-commit-waits");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let lock: PathBuf = repo.join(".git").join("cli-git").join("hook.lock");
    let record: PathBuf = root.join("hook.txt");
    install_hook(repo.as_path(), lock.as_path(), record.as_path(), 0);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::create_dir_all(lock.parent().expect("lock parent")).expect("lock parent");
    let holder: OwnerLock =
        acquire_owner_lock(lock.as_path(), DEFAULT_POLL_DELAY, &mut SilentWait).expect("held");
    let waiter_repo: PathBuf = repo.clone();
    let waiter_lock: PathBuf = lock.clone();
    let waiter = std::thread::spawn(move || {
        let locked_plan: HookDispatchPlan = plan(waiter_lock.as_path(), false, &[]);
        let context = git_context();
        return run_post_commit_hook(PostCommitHookRequest {
            context: &context,
            worktree: waiter_repo.as_path(),
            real_index: waiter_repo.join(".git").join("index").as_path(),
            landed_oid: landed.as_str(),
            plan: &locked_plan,
        });
    });
    std::thread::sleep(std::time::Duration::from_millis(600));
    assert!(
        !record.exists(),
        "the hook must not run while another owner holds the lock"
    );
    holder.release().expect("released");
    let outcome = waiter.join().expect("waiter thread").expect("run");
    assert_eq!(outcome, PostCommitHookOutcome::Ran(0));
    assert!(recorded(record.as_path()).contains("locked=yes"));
    remove(root.as_path());
}
