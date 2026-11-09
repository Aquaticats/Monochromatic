//! What:
//!  Controls for the clearing that precedes the person's own forwarded `git fetch` and `git push`.
//! Why:
//!  The forwarded command becomes Git,
//! so the notice is all the wrapper can add;
//! it must name what it removed,
//! describe what
//!  it will not remove,
//! and leave every other command and repository alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(forwardedLockNotice({ word: 'fetch', globalPrefix: ['-C', repo], environment: [], judge })).toContain('removed the reference lock');
//! ```
#![cfg(unix)]

use super::*;
use crate::stale_ref_lock_killed_git as killed;
use crate::stale_ref_lock_killed_git::{Held, Stage};

/// The prefix that points the command at the stage's repository.
fn prefix(stage: &Stage) -> Vec<OsString> {
    return vec![OsString::from("-C"), stage.repo.clone().into_os_string()];
}

/// A killed fetch's locks are named before a forwarded fetch,
/// and each carries the command that clears it,
/// because a number in a pid file proves nothing about its writer.
#[test]
fn a_forwarded_fetch_names_what_stays_and_says_so() {
    let stage: Stage = killed::stage("forwarded-notice-proven", true);
    let mut held: Held = killed::hold_fetch(&stage);
    held.kill();
    let notice: String = forwarded_lock_notice(b"fetch", prefix(&stage).as_slice(), &[], &killed::judge());
    assert!(notice.contains("monochromatic-git-policy-cli: left the reference lock"), "{notice}");
    assert!(notice.contains("rm -- "), "{notice}");
    assert!(stage.lock_of(killed::GONE_REF).exists() && stage.packed_lock().exists());
    stage.finish();
}

/// A lock that is not proven stale stays,
/// and the notice gives the command that clears it.
#[test]
fn a_forwarded_push_describes_a_lock_it_will_not_remove() {
    let stage: Stage = killed::stage("forwarded-notice-unproven", false);
    let mut held: Held = killed::hold_fetch(&stage);
    held.kill();
    let notice: String = forwarded_lock_notice(b"push", prefix(&stage).as_slice(), &[], &killed::judge());
    assert!(notice.contains(&format!("rm -- {} ", stage.lock_of(killed::GONE_REF).display())) || notice.contains("rm -- "), "{notice}");
    assert!(stage.lock_of(killed::GONE_REF).exists());
    stage.finish();
}

/// Other commands are not looked at,
/// and nothing is removed for them.
#[test]
fn other_commands_are_left_alone() {
    let stage: Stage = killed::stage("forwarded-notice-others", true);
    let mut held: Held = killed::hold_fetch(&stage);
    held.kill();
    for word in [&b"status"[..], b"log", b"pull", b"commit"] {
        assert_eq!(forwarded_lock_notice(word, prefix(&stage).as_slice(), &[], &killed::judge()), "");
    }
    assert!(stage.lock_of(killed::GONE_REF).exists());
    stage.finish();
}

/// A repository named by an option the wrapper does not read is not guessed at.
#[test]
fn a_repository_named_by_another_option_is_not_touched() {
    let stage: Stage = killed::stage("forwarded-notice-git-dir", true);
    let mut held: Held = killed::hold_fetch(&stage);
    held.kill();
    let named: Vec<OsString> = vec![OsString::from("--git-dir"), stage.git_dir.clone().into_os_string()];
    assert_eq!(forwarded_lock_notice(b"fetch", named.as_slice(), &[], &killed::judge()), "");
    assert!(stage.lock_of(killed::GONE_REF).exists());
    stage.finish();
}

/// `GIT_DIR` in the environment names the repository,
/// as it does for Git.
#[test]
fn git_dir_in_the_environment_names_the_repository() {
    let stage: Stage = killed::stage("forwarded-notice-env", true);
    let mut held: Held = killed::hold_fetch(&stage);
    held.kill();
    let environment: Vec<(OsString, OsString)> = vec![(OsString::from("GIT_DIR"), stage.git_dir.clone().into_os_string())];
    let notice: String = forwarded_lock_notice(b"fetch", prefix(&stage).as_slice(), environment.as_slice(), &killed::judge());
    assert!(notice.contains("left the reference lock"), "{notice}");
    stage.finish();
}
