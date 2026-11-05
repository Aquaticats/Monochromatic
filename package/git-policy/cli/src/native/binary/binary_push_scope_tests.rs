//! Binary-level controls for a push whose scope the wrapper cannot establish: the engine failure's hint names the real
//! cause, and only the case where content is missing keeps the hint about missing content.
//!
//! What:
//!  Each case makes a push that Git's dry run accepts but whose scan range the wrapper cannot read, and requires the
//!  failure event's message and hint to be those of its cause.
//! Why:
//!  One engine-failure code covers causes the person answers differently. A hint chosen by the code alone told a person
//!  who had pushed a tag of a tree to fetch history, which is not the case.

use super::commit::remove_settled;
use super::lfs_pre_push::{World, commit_directly, ref_of, world};
use super::support::{Fixture, Observed, fixture, git, observe_raw, stderr_of, wrapped};
use git_policy_cli::event_hints::{engine_failure_hint, manual_push_failure_hint};
use git_policy_cli::manual_push_cause::ManualPushCause;

/// Runs one wrapped command in the repository with its hint fields kept, which the other controls strip.
fn wrapped_with_hints(fixture: &Fixture, directory: &std::path::Path, arguments: &[&str]) -> Observed {
    return observe_raw(wrapped(fixture).current_dir(directory).args(arguments), b"");
}

/// A repository whose policy file turns a manual-push policy on, with one commit published to `origin`.
fn policy_world(fixture: &Fixture) -> World {
    let world: World = world(fixture, false, None);
    commit_directly(fixture, world.repo.as_path(), "monochromatic-git-policy-cli.config.jsonc", "{ \"policies\": { \"final-newline\": \"error\" } }\n");
    git(fixture, world.repo.as_path(), &["push", "--quiet", "--no-verify", "origin", "main"]);
    return world;
}

/// The hint field of an event line for `cause`, as the wrapper writes it, or nothing when the cause has none.
fn hint_field(cause: ManualPushCause) -> Option<String> {
    return manual_push_failure_hint(cause).map(|hint| { return format!("\"hint\":\"{hint}\""); });
}

/// Requires a `content-unavailable` failure on the manual-push trigger whose hint is the one of `cause`.
fn assert_failure_of(pushed: &Observed, cause: ManualPushCause, message_part: &str) {
    let stderr: String = stderr_of(pushed);
    assert_eq!(pushed.code, Some(2), "{stderr}");
    assert!(stderr.contains("\"code\":\"content-unavailable\""), "{stderr}");
    assert!(stderr.contains("\"trigger\":\"manual-push\""), "{stderr}");
    assert!(stderr.contains(message_part), "the message must say {message_part:?}: {stderr}");
    match hint_field(cause) {
        Some(field) => assert!(stderr.contains(field.as_str()), "the hint of {cause:?} is missing: {stderr}"),
        None => assert!(!stderr.contains("\"hint\""), "{cause:?} takes no hint: {stderr}"),
    }
}

/// A tag of a tree is not a commit: the message says so and the hint says to push a commit, not to fetch history.
#[test]
fn a_tag_of_a_tree_gets_the_hint_about_commits() {
    let fixture: Fixture = fixture("scope-tree-tag");
    let world: World = policy_world(&fixture);
    git(&fixture, world.repo.as_path(), &["tag", "tree-tag", "HEAD^{tree}"]);
    let pushed: Observed = wrapped_with_hints(&fixture, world.repo.as_path(), &["push", "origin", "tree-tag"]);
    assert_failure_of(&pushed, ManualPushCause::UnsupportedObject, "is not a commit");
    let by_code: &str = engine_failure_hint("content-unavailable").expect("the code's hint");
    assert!(!stderr_of(&pushed).contains(by_code), "a tag of a tree is not missing content: {}", stderr_of(&pushed));
    remove_settled(&fixture);
}

/// A commit whose parent object is gone from the clone is content that is missing: the hint about fetching history stays.
#[test]
fn a_commit_whose_parent_object_is_gone_keeps_the_hint_about_missing_content() {
    let fixture: Fixture = fixture("scope-missing-parent");
    let world: World = policy_world(&fixture);
    commit_directly(&fixture, world.repo.as_path(), "one.txt", "one\n");
    let parent: String = ref_of(&fixture, world.repo.as_path(), "HEAD");
    commit_directly(&fixture, world.repo.as_path(), "two.txt", "two\n");
    let object: std::path::PathBuf = world.repo.join(".git/objects").join(&parent[..2]).join(&parent[2..]);
    assert!(object.exists(), "the parent is a loose object: {}", object.display());
    std::fs::remove_file(&object).expect("remove the parent object");
    let pushed: Observed = wrapped_with_hints(&fixture, world.repo.as_path(), &["push", "origin", "HEAD:refs/heads/unpublished"]);
    assert_failure_of(&pushed, ManualPushCause::ContentMissing, "failed");
    remove_settled(&fixture);
}
