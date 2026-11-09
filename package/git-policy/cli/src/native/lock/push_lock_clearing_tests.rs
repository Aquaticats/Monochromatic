//! What:
//!  Controls for the wrapper's own fetch and push starting after the stale reference locks of their repository are cleared.
//! Why:
//!  The locks are made by real Git killed with `SIGKILL`,
//! and the fetch and push are real Git started through the same runner
//!  the commit service uses,
//! so what the controls show is what the service meets.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! heldPush.kill('SIGKILL'); expect((await runBoundedPush(pushAgain)).exitCode).toBe(0);
//! ```
#![cfg(unix)]

use super::*;
use crate::bounded_push::STOP_GRACE;
use crate::stale_ref_lock_killed_git as killed;
use crate::stale_ref_lock_killed_git::{Held, Stage};
use crate::test_support::scratch_environment;
use std::time::Duration;

/// The patience of a run that is never meant to be stopped.
const PATIENCE: Patience = Patience::Fixed(Duration::from_secs(120));

/// The tracking reference a killed push leaves locked.
const TRACKING: &str = "refs/remotes/origin/main";

/// Run `arguments` as the wrapper's own runner does,
/// with `judge`.
fn run(stage: &Stage, arguments: &[&str], judge: &Judge) -> PushAttempt {
    let command: Command = killed::plain_git(stage, arguments);
    return run_clearing_locks(command, PATIENCE, STOP_GRACE, scratch_environment().as_slice(), judge).expect("the run");
}

/// A push killed after the remote took its commit leaves the tracking reference locked;
/// the next push names that lock and gives the command,
/// because a number in its pid file proves nothing about its writer.
#[test]
fn a_push_names_the_lock_of_a_killed_push_it_will_not_clear() {
    let stage: Stage = killed::stage("push-clearing-killed-push", true);
    let mut held: Held = killed::hold_push(&stage);
    assert!(stage.lock_of(TRACKING).exists(), "the push holds the tracking reference");
    held.kill();
    let _removed = std::fs::remove_file(stage.git_dir.join("hooks").join("reference-transaction"));
    let attempt: PushAttempt = run(&stage, &["push", "--quiet", "origin", "main"], &killed::judge());
    assert!(attempt.output.contains("monochromatic-git-policy-cli: left the reference lock"), "{}", attempt.output);
    assert!(attempt.output.contains("rm -- "), "{}", attempt.output);
    assert!(stage.lock_of(TRACKING).exists());
    stage.finish();
}

/// The same push,
/// with a judge that never sees a complete process table,
/// shows Git's refusal (Git exits 0 for it,
/// the
/// push having delivered) and says what was found.
#[test]
fn a_failed_push_carries_the_notice_of_the_lock_it_could_not_clear() {
    let stage: Stage = killed::stage("push-clearing-unproven", true);
    let mut held: Held = killed::hold_push(&stage);
    held.kill();
    let _removed = std::fs::remove_file(stage.git_dir.join("hooks").join("reference-transaction"));
    let partial: Judge = Judge { ..killed::judge() };
    let attempt: PushAttempt = run(&stage, &["push", "--quiet", "origin", "main"], &partial);
    assert!(attempt.output.contains("cannot lock ref"), "Git's own refusal stays: {}", attempt.output);
    assert!(attempt.output.contains("monochromatic-git-policy-cli: left the reference lock"), "{}", attempt.output);
    assert!(attempt.output.contains(&format!("rm -- {} ", stage.lock_of(TRACKING).display())), "{}", attempt.output);
    assert!(stage.lock_of(TRACKING).exists());
    stage.finish();
}

/// A run that meets no lock says nothing:
/// the notice is for a lock that stands in the way.
#[test]
fn a_run_with_no_lock_adds_no_words() {
    let stage: Stage = killed::stage("push-clearing-silent", true);
    let attempt: PushAttempt = run(&stage, &["fetch", "--prune", "--quiet", "origin"], &killed::judge());
    assert_eq!(attempt.exit_code, 0, "{}", attempt.output);
    assert_eq!(attempt.output, "");
    stage.finish();
}
