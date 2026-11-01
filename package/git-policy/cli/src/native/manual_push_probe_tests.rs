//! Controls for the manual-push probe against real repositories and bare remotes, with Git writing the report.
//! The TypeScript corpus is `src/policy-engine/manual-push.integration.unit.test.ts` (probe and ls-remote cases)
//! and `src/policy-engine/manual-push-scope.unit.test.ts`; the update-discovery cases are reproduced here.

use super::*;
use crate::auto_push_single_flight::PushGit;
use crate::manual_push_porcelain::{PorcelainUpdate, RemoteBefore};
use crate::test_support::{fixture, git, remove};
use std::path::{Path, PathBuf};

/// Environment every Git child of these fixtures runs with.
fn fixture_environment() -> Vec<(OsString, OsString)> {
    return [
        ("PATH", "/usr/bin:/bin"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/nonexistent-global-config"),
        ("GIT_AUTHOR_NAME", "Fixture"),
        ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
        ("GIT_COMMITTER_NAME", "Fixture"),
        ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
    ]
    .iter()
    .map(|(name, value)| {
        return (OsString::from(name), OsString::from(value));
    })
    .collect();
}

/// The home directory of a fixture repository at `root`, beside it, where the per-user scratch directory is made.
fn home_beside(root: &Path) -> PathBuf {
    return root.parent().expect("fixture root").join("home");
}

/// Coordinator Git for a fixture repository at `root`, with a home beside the repository.
fn git_for(root: &Path) -> PushGit {
    let mut environment = fixture_environment();
    environment.push((OsString::from("HOME"), home_beside(root).into_os_string()));
    return PushGit {
        real_git: PathBuf::from("/usr/bin/git"),
        cwd: root.to_path_buf(),
        environment,
        lfs: false,
    };
}

/// A repository with one commit on `main` and a bare origin that holds the same commit.
fn repository_with_origin(name: &str) -> (PathBuf, PathBuf, String) {
    let root = fixture(name);
    let origin = root.join("origin.git");
    let repo = root.join("repo");
    git(
        &root,
        &["init", "--quiet", "--bare", origin.to_str().expect("utf-8")],
    );
    std::fs::create_dir_all(&repo).expect("repo");
    git(&repo, &["init", "--quiet", "--initial-branch=main"]);
    git(
        &repo,
        &["commit", "--quiet", "--allow-empty", "--message=base"],
    );
    git(
        &repo,
        &["remote", "add", "origin", origin.to_str().expect("utf-8")],
    );
    git(
        &repo,
        &["push", "--quiet", "origin", "HEAD:refs/heads/main"],
    );
    let base = head_of(&repo);
    return (repo, origin, base);
}

/// The commit `HEAD` names.
fn head_of(repo: &Path) -> String {
    return String::from_utf8_lossy(&git(repo, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_owned();
}

/// The object a ref names in a bare remote, or nothing.
fn remote_value(origin: &Path, reference: &str) -> Option<String> {
    let output = std::process::Command::new("/usr/bin/git")
        .current_dir(origin)
        .env_clear()
        .envs(fixture_environment())
        .args(["rev-parse", "--verify", "--quiet", reference])
        .output()
        .expect("rev-parse");
    if !output.status.success() {
        return None;
    }
    return Some(String::from_utf8_lossy(&output.stdout).trim().to_owned());
}

/// Post-subcommand arguments as owned strings.
fn post(arguments: &[&str]) -> Vec<OsString> {
    return arguments.iter().map(OsString::from).collect();
}

/// A new branch pushed by the probe is one update: a local object, no remote object, the named remote.
#[test]
fn a_new_branch_is_one_update_with_no_remote_object() {
    let (repo, origin, _base) = repository_with_origin("mp-probe-new");
    git(&repo, &["checkout", "--quiet", "-b", "feature"]);
    let landed = head_of(&repo);
    git(
        &repo,
        &["commit", "--quiet", "--allow-empty", "--message=feature"],
    );
    let tip = head_of(&repo);
    let updates = probe_manual_push_updates(&git_for(&repo), &[], &post(&["origin", "feature"]))
        .expect("probes");
    assert_eq!(
        updates,
        vec![PushUpdate {
            local_oid: Some(tip.clone()),
            remote_oid: None,
            remote_name: "origin".to_owned(),
            remote_ref: "refs/heads/feature".to_owned(),
        }]
    );
    assert_eq!(
        remote_value(&origin, "refs/heads/feature"),
        None,
        "the probe must not publish"
    );
    assert_ne!(landed, tip);
    remove(repo.parent().expect("root"));
}

/// A fast-forward names the remote's current object as the prior value.
#[test]
fn a_fast_forward_names_the_prior_remote_value() {
    let (repo, origin, base) = repository_with_origin("mp-probe-ff");
    git(
        &repo,
        &["commit", "--quiet", "--allow-empty", "--message=next"],
    );
    let tip = head_of(&repo);
    let updates = probe_manual_push_updates(&git_for(&repo), &[], &post(&["origin", "main"]))
        .expect("probes");
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].local_oid.as_deref(), Some(tip.as_str()));
    assert_eq!(updates[0].remote_oid.as_deref(), Some(base.as_str()));
    assert_eq!(
        remote_value(&origin, "refs/heads/main").as_deref(),
        Some(base.as_str())
    );
    remove(repo.parent().expect("root"));
}

/// Deleting a remote branch is an update with no local object.
#[test]
fn a_deletion_has_no_local_object() {
    let (repo, _origin, base) = repository_with_origin("mp-probe-delete");
    let updates = probe_manual_push_updates(&git_for(&repo), &[], &post(&["origin", ":main"]))
        .expect("probes");
    assert_eq!(
        updates,
        vec![PushUpdate {
            local_oid: None,
            remote_oid: Some(base),
            remote_name: "origin".to_owned(),
            remote_ref: "refs/heads/main".to_owned(),
        }]
    );
    remove(repo.parent().expect("root"));
}

/// A push with nothing to send has no updates to scan.
#[test]
fn an_up_to_date_push_has_no_updates() {
    let (repo, _origin, _base) = repository_with_origin("mp-probe-noop");
    let updates = probe_manual_push_updates(&git_for(&repo), &[], &post(&["origin", "main"]))
        .expect("probes");
    assert_eq!(updates, Vec::new());
    remove(repo.parent().expect("root"));
}

/// A destination Git cannot reach yields no capture, so the probe fails instead of reporting no updates.
#[test]
fn an_unreachable_destination_fails_the_probe() {
    let (repo, _origin, _base) = repository_with_origin("mp-probe-unreachable");
    git(
        &repo,
        &["commit", "--quiet", "--allow-empty", "--message=next"],
    );
    let missing = repo.parent().expect("root").join("missing.git");
    let result = probe_manual_push_updates(
        &git_for(&repo),
        &[],
        &post(&[missing.to_str().expect("utf-8"), "main"]),
    );
    assert!(result.is_err());
    remove(repo.parent().expect("root"));
}

/// The probe's arguments keep Git's options before the terminator, switch every hook off and ask for full names.
#[test]
fn probe_arguments_keep_git_options_before_the_terminator() {
    let arguments: Vec<OsString> = probe_arguments(&post(&["-C", "dir"]), &post(&["--force", "origin", "--", "main"]));
    let text: Vec<String> = arguments
        .iter()
        .map(|argument| {
            return argument.to_string_lossy().into_owned();
        })
        .collect();
    let position = |needle: &str| -> usize {
        return text
            .iter()
            .position(|word| {
                return word == needle;
            })
            .expect(needle);
    };
    assert!(position("--dry-run") < position("--"), "{text:?}");
    assert!(position("--porcelain") < position("--"), "{text:?}");
    assert!(position("push") < position("--force") && position("--force") < position("--dry-run"), "{text:?}");
    assert!(text.contains(&String::from("hook.pre-push.enabled=false")), "{text:?}");
    assert!(text.contains(&String::from("core.abbrev=no")), "{text:?}");
    assert!(
        text.iter().any(|word| {
            return word.starts_with("core.hooksPath=");
        }),
        "{text:?}"
    );
    assert_eq!(&text[..2], ["-C", "dir"], "the caller's global options come first");
}

/// A configured `pre-push` hook of the repository does not run for the probe, and a dry run publishes nothing.
#[test]
fn a_configured_pre_push_hook_does_not_run_for_the_probe() {
    let (repo, origin, _base) = repository_with_origin("mp-probe-hook-off");
    let marker = repo.parent().expect("root").join("hook-ran");
    git(&repo, &["config", "hook.mine.event", "pre-push"]);
    git(&repo, &["config", "hook.mine.command", &format!("touch {}", marker.display())]);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=next"]);
    let updates = probe_manual_push_updates(&git_for(&repo), &[], &post(&["origin", "main"])).expect("probes");
    assert_eq!(updates.len(), 1);
    assert!(!marker.exists(), "the caller's hook ran for the probe");
    assert_ne!(remote_value(&origin, "refs/heads/main"), Some(head_of(&repo)), "the probe published");
    remove(repo.parent().expect("root"));
}

/// A bare push resolves the remote the way Git does, and a tag is reported with its own object.
#[test]
fn a_bare_push_and_a_tag_are_reported_with_the_remote_git_picks() {
    let (repo, _origin, _base) = repository_with_origin("mp-probe-bare");
    git(&repo, &["config", "push.default", "current"]);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=next"]);
    git(&repo, &["tag", "--annotate", "--message=tag", "v1"]);
    let tip = head_of(&repo);
    let bare = probe_manual_push_updates(&git_for(&repo), &[], &[]).expect("probes");
    assert_eq!(bare.len(), 1);
    assert_eq!((bare[0].remote_name.as_str(), bare[0].local_oid.as_deref()), ("origin", Some(tip.as_str())));
    let tag_object = String::from_utf8_lossy(&git(&repo, &["rev-parse", "refs/tags/v1"]).stdout).trim().to_owned();
    let tags = probe_manual_push_updates(&git_for(&repo), &[], &post(&["origin", "v1"])).expect("probes");
    assert_eq!(tags[0].local_oid.as_deref(), Some(tag_object.as_str()), "the tag object, not the commit");
    assert_eq!(tags[0].remote_oid, None);
    remove(repo.parent().expect("root"));
}

/// Confirmation refuses a destination whose value differs from the reported one.
#[test]
fn confirmation_refuses_a_remote_ref_that_changed_after_the_report() {
    let (repo, origin, base) = repository_with_origin("mp-confirm-changed");
    let location: &str = origin.to_str().expect("utf-8");
    let reported = |before: RemoteBefore, to: &str| -> Vec<(PorcelainUpdate, Option<String>)> {
        let update = PorcelainUpdate { from: String::from("refs/heads/main"), to: to.to_owned(), before, after: None };
        return vec![(update, Some(head_of(&repo)))];
    };
    let changed = reported(RemoteBefore::Known("1".repeat(base.len())), "refs/heads/main");
    assert!(confirm_group(&git_for(&repo), location, "origin", changed.as_slice()).is_err());
    let matching = reported(RemoteBefore::Known(base.clone()), "refs/heads/main");
    let confirmed = confirm_group(&git_for(&repo), location, "origin", matching.as_slice()).expect("confirms");
    assert_eq!(confirmed[0].remote_oid.as_deref(), Some(base.as_str()));
    let deleted_since = reported(RemoteBefore::Known(base), "refs/heads/gone");
    assert!(confirm_group(&git_for(&repo), location, "origin", deleted_since.as_slice()).is_err(), "the reported tip no longer exists");
    let appeared_since = reported(RemoteBefore::Absent, "refs/heads/main");
    assert!(confirm_group(&git_for(&repo), location, "origin", appeared_since.as_slice()).is_err(), "a new ref that now exists");
    remove(repo.parent().expect("root"));
}

/// The probe error displays its message.
#[test]
fn the_probe_error_displays_its_message() {
    assert_eq!(probe_error(String::from("why")).to_string(), "why");
}
