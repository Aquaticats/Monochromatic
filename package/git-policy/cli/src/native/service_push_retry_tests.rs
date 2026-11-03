//! Controls for the service's own retry of a push that was stopped or failed: a local bare remote whose `pre-receive` hook
//! stalls the first push and accepts the next, real Git pushing, and the service's work loop driven by hand.
//! Why:
//!  At 05:40 on 2026-11-03 a stopped auto-push of `main` was never tried again until a person pushed by hand 15 minutes
//!  later, which also blocked a release (the owner's rule, `SPEC.md` "Merges and commits never fail": auto-push must
//!  land). The retry needs no new commit: the work loop looks at the repository again when the pause is over.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await drain(gitDir); const wait = await drain(gitDir); await sleep(wait); await drain(gitDir); expect(remoteHas(commit)).toBe(true);
//! ```
#![cfg(unix)]

use super::*;
use crate::push_failure_notice::notice_directory;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::work_item::enqueue;
use std::os::unix::fs::PermissionsExt;

/// The stall window of the controls, short enough to wait for and longer than a healthy local push.
const STALL_SECONDS: &str = "2";

/// The first pause before a push is tried again, in the controls.
const PAUSE: Duration = Duration::from_millis(800);

/// A repository tracking a bare remote whose hook stalls the first push it sees after the setup.
struct Stage {
    /// The fixture directory, removed at the end.
    root: PathBuf,
    /// The working repository.
    repo: PathBuf,
    /// The bare remote.
    remote: PathBuf,
    /// The commit to deliver.
    landed: String,
}

/// Build the stage: an initial commit pushed with no hook, the hook installed, a new commit landed, its work item written.
fn stage(name: &str) -> Stage {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let remote: PathBuf = root.join("remote.git");
    git(root.as_path(), &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")]);
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    std::fs::write(repo.join("a.txt"), b"one\n").expect("file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    git(repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "main"]);
    let marker: PathBuf = root.join("stalled-once");
    let hook: PathBuf = remote.join("hooks").join("pre-receive");
    std::fs::write(hook.as_path(), format!("#!/bin/sh\nif [ ! -e '{}' ]; then : > '{}'; exec sleep 600; fi\nexit 0\n", marker.display(), marker.display())).expect("hook");
    std::fs::set_permissions(hook.as_path(), std::fs::Permissions::from_mode(0o755)).expect("hook is executable");
    std::fs::write(repo.join("a.txt"), b"two\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=two"]);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let work_item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repo.display().to_string(),
        git_dir: repo.join(".git").display().to_string(),
        branch_ref: String::from("refs/heads/main"),
        oid: landed.clone(),
        parent: String::new(),
        created: String::from("2026-11-03T05:40:00.000Z"),
        tries: 0,
    };
    enqueue(work_directory(repo.join(".git").as_path()).as_path(), &work_item).expect("enqueue");
    return Stage { root, repo, remote, landed };
}

/// The branch tip the remote holds.
fn remote_tip(stage: &Stage) -> String {
    return git_text(stage.remote.as_path(), &["rev-parse", "refs/heads/main"]).trim().to_owned();
}

/// How many records of failed pushes the repository holds for the person.
fn notices(stage: &Stage) -> usize {
    return std::fs::read_dir(notice_directory(stage.repo.join(".git").as_path())).map_or(0, |entries| { return entries.flatten().count(); });
}

/// A push the service stopped is tried again by the service alone, after a pause, with no new commit: the first drain stops
///  the stalled push and says when it will try again, the second (the work loop's wake after the pause) delivers, and the
///  person's failure record is gone.
#[test]
fn a_stopped_push_is_tried_again_without_a_new_commit_and_lands() {
    let stage: Stage = stage("push-retry-stall-once");
    let git_dir: PathBuf = stage.repo.join(".git");
    let handlers: GitHandlers = GitHandlers::new(git_context()).with_push_environment(&[("CLI_GIT_PUSH_DEADLINE_SECONDS", STALL_SECONDS)]).with_retry_pause(PAUSE);
    let first: Option<Duration> = drain(git_dir.as_path(), &handlers, &ClaimTime::read());
    assert_ne!(remote_tip(&stage), stage.landed, "the first push stalled and was stopped");
    assert_eq!(notices(&stage), 1, "the person is told the push failed");
    let wait: Duration = first.expect("the service says when to look at the repository again");
    assert!(wait <= PAUSE + Duration::from_secs(5), "the wake is for the retry: {wait:?}");
    std::thread::sleep(wait);
    let second: Option<Duration> = drain(git_dir.as_path(), &handlers, &ClaimTime::read());
    assert_eq!(remote_tip(&stage), stage.landed, "the retry delivered the commit");
    assert_eq!(notices(&stage), 0, "the record of the failure is gone once the push landed");
    assert_eq!(second, None, "nothing is left to try again");
    remove(stage.root.as_path());
}
