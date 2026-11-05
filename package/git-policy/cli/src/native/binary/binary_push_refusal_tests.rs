//! Binary-level controls for a push that Git itself refuses: the wrapper reports Git's refusal and exit status, and never
//! an engine failure of its own with a hint about missing content.
//!
//! What:
//!  Each case runs a push that Git's dry run refuses before it transfers anything, once with plain Git and once through the
//!  wrapper with a manual-push policy configured, and requires the same exit status and Git's message.
//! Why:
//!  The wrapper asks Git for the push's updates with `push --dry-run --porcelain` before any policy reads content. When
//!  Git refuses the push there (a branch to delete that the remote does not have, a refspec that names nothing, a remote
//!  that does not exist, an option Git does not know), nothing could be published, and Git's own words are the answer. A
//!  wrapper failure that said the content was unavailable, with a hint to fetch history, named a cause that was not the case.

use super::commit::remove_settled;
use super::lfs_pre_push::{World, commit_directly, ref_of, wrapped, world};
use super::support::{Fixture, Observed, direct, fixture, git, observe, stderr_of};

/// What the wrapper's own failure report carries for this refusal, which a refusal by Git must never show.
const ENGINE_FAILURE_MARKERS: [&str; 3] = ["engine-failure", "content-unavailable", "Hint: if you are committing or pushing content"];

/// A repository whose policy file turns a manual-push policy on, with one commit published to `origin`.
fn policy_world(fixture: &Fixture) -> World {
    let world: World = world(fixture, false, None);
    let policy: &str = "{ \"policies\": { \"final-newline\": \"error\" } }\n";
    commit_directly(fixture, world.repo.as_path(), "monochromatic-git-policy-cli.config.jsonc", policy);
    git(fixture, world.repo.as_path(), &["push", "--quiet", "--no-verify", "origin", "main"]);
    return world;
}

/// Runs `region` with plain Git in the repository, for the status and message Git gives.
fn plain(fixture: &Fixture, world: &World, region: &[&str]) -> Observed {
    return observe(direct(fixture).current_dir(world.repo.as_path()).args(region), b"");
}

/// Requires that the wrapped `region` ends exactly as plain Git ends it, with no engine failure and no hint.
fn assert_ends_as_git(fixture: &Fixture, world: &World, region: &[&str]) {
    let expected: Observed = plain(fixture, world, region);
    assert_ne!(expected.code, Some(0), "{region:?}: the case must be a push Git refuses: {}", stderr_of(&expected));
    let before: String = ref_of(fixture, world.origin.as_path(), "refs/heads/main");
    let pushed: Observed = wrapped(fixture, world.repo.as_path(), region);
    let stderr: String = stderr_of(&pushed);
    assert_eq!(pushed.code, expected.code, "{region:?}: {stderr}");
    for marker in ENGINE_FAILURE_MARKERS {
        assert!(!stderr.contains(marker), "{region:?} shows the wrapper's {marker:?}: {stderr}");
    }
    assert_eq!(stderr, stderr_of(&expected), "{region:?}: Git's message must be the whole answer");
    assert_eq!(ref_of(fixture, world.origin.as_path(), "refs/heads/main"), before, "{region:?}: nothing may be published");
}

/// A branch to delete that the remote does not have ends as Git ends it.
#[test]
fn deleting_a_branch_the_remote_does_not_have_ends_as_git_ends_it() {
    let fixture: Fixture = fixture("refusal-delete-absent");
    let world: World = policy_world(&fixture);
    assert_ends_as_git(&fixture, &world, &["push", "origin", "--delete", "no-such-branch"]);
    remove_settled(&fixture);
}

/// A refspec whose source names nothing in the repository ends as Git ends it.
#[test]
fn a_source_that_names_nothing_ends_as_git_ends_it() {
    let fixture: Fixture = fixture("refusal-source");
    let world: World = policy_world(&fixture);
    assert_ends_as_git(&fixture, &world, &["push", "origin", "no-such-source:main"]);
    remove_settled(&fixture);
}

/// A destination that is not a full ref name and cannot be guessed ends as Git ends it.
#[test]
fn a_destination_that_cannot_be_qualified_ends_as_git_ends_it() {
    let fixture: Fixture = fixture("refusal-destination");
    let world: World = policy_world(&fixture);
    assert_ends_as_git(&fixture, &world, &["push", "origin", "HEAD~0:unqualified"]);
    remove_settled(&fixture);
}

/// A remote that does not exist ends as Git ends it.
#[test]
fn a_remote_that_does_not_exist_ends_as_git_ends_it() {
    let fixture: Fixture = fixture("refusal-remote");
    let world: World = policy_world(&fixture);
    assert_ends_as_git(&fixture, &world, &["push", "no-such-remote", "main"]);
    remove_settled(&fixture);
}

/// An option Git does not know ends as Git ends it, with its usage text.
#[test]
fn an_option_git_does_not_know_ends_as_git_ends_it() {
    let fixture: Fixture = fixture("refusal-option");
    let world: World = policy_world(&fixture);
    assert_ends_as_git(&fixture, &world, &["push", "--no-such-option", "origin"]);
    remove_settled(&fixture);
}

/// Options Git does not allow together end as Git ends them.
#[test]
fn options_that_cannot_be_combined_end_as_git_ends_them() {
    let fixture: Fixture = fixture("refusal-combination");
    let world: World = policy_world(&fixture);
    assert_ends_as_git(&fixture, &world, &["push", "--all", "origin", "main"]);
    remove_settled(&fixture);
}

/// A tag that the remote has at another commit is rejected by Git's own push after the policies passed, with Git's message.
///  (A branch that is not a fast-forward is not this case: the wrapper integrates it by design.)
#[test]
fn a_rejected_update_ends_as_git_ends_it() {
    let fixture: Fixture = fixture("refusal-rejected");
    let world: World = policy_world(&fixture);
    git(&fixture, world.repo.as_path(), &["tag", "v1"]);
    git(&fixture, world.repo.as_path(), &["push", "--quiet", "--no-verify", "origin", "v1"]);
    commit_directly(&fixture, world.repo.as_path(), "later.txt", "later\n");
    git(&fixture, world.repo.as_path(), &["tag", "--force", "v1"]);
    let expected: Observed = plain(&fixture, &world, &["push", "origin", "v1"]);
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "origin", "v1"]);
    let stderr: String = stderr_of(&pushed);
    assert_eq!(pushed.code, expected.code, "{stderr}");
    for marker in ENGINE_FAILURE_MARKERS {
        assert!(!stderr.contains(marker), "the rejection shows the wrapper's {marker:?}: {stderr}");
    }
    // Git's own line for the rejected ref; the wrapper's atomic push adds a line of its own before it.
    let plain_text: String = stderr_of(&expected);
    let rejection: &str = plain_text.lines().find(|line| { return line.contains("[rejected]"); }).expect("Git rejects the tag");
    assert!(stderr.contains(rejection), "Git's rejection must be shown: {stderr}");
    remove_settled(&fixture);
}
