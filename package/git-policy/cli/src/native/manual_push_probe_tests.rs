//! Controls for the manual-push probe against real repositories and bare remotes.
//! The TypeScript corpus is `src/policy-engine/manual-push.integration.unit.test.ts` (probe and ls-remote cases)
//! and `src/policy-engine/manual-push-scope.unit.test.ts`; the update-discovery cases are reproduced here.

use super::*;
use crate::auto_push_single_flight::PushGit;
use crate::test_support::{fixture, git, remove};

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

/// Coordinator Git for a fixture repository at `root`.
fn git_for(root: &Path) -> PushGit {
    return PushGit {
        real_git: PathBuf::from("/usr/bin/git"),
        cwd: root.to_path_buf(),
        environment: fixture_environment(),
        lfs_pre_push: None,
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

/// Pure parsing: four fields per record, empty lines ignored, other shapes refused.
#[test]
fn update_records_have_four_fields_and_other_shapes_are_refused() {
    let parsed = parse_update_records(
        "refs/heads/main abc refs/heads/main def\n\nrefs/heads/x 111 refs/heads/x 000\n",
        "origin",
        "/remote",
    )
    .expect("parses");
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].remote_name, "origin");
    assert_eq!(parsed[0].remote_location, "/remote");
    assert_eq!(parsed[0].remote_ref, "refs/heads/main");
    assert_eq!(parsed[0].advertised_remote_oid, "def");
    assert!(parse_update_records("refs/heads/main abc refs/heads/main\n", "o", "l").is_err());
    assert!(parse_update_records("a b c d e\n", "o", "l").is_err());
    assert_eq!(
        parse_update_records("", "o", "l").expect("empty"),
        Vec::new()
    );
}

/// Only the all-zero spelling is an absent object name.
#[test]
fn only_all_zero_names_are_absent() {
    assert!(is_zero_oid("0000000000000000000000000000000000000000"));
    assert!(!is_zero_oid("0000000000000000000000000000000000000001"));
    assert!(!is_zero_oid(""));
}

/// The probe inserts its hook override and `--dry-run --verify` before a `--` terminator, after the global options.
#[test]
fn probe_arguments_keep_git_options_before_the_terminator() {
    let arguments = probe_arguments(
        &post(&["-C", "/repo"]),
        &post(&["origin", "main", "--", "tail"]),
        Path::new("/hooks"),
    );
    let text: Vec<String> = arguments
        .iter()
        .map(|argument| {
            return argument.to_string_lossy().into_owned();
        })
        .collect();
    assert_eq!(
        text,
        vec![
            "-C",
            "/repo",
            "-c",
            "core.hooksPath=/hooks",
            "push",
            "origin",
            "main",
            "--dry-run",
            "--verify",
            "--",
            "tail"
        ]
    );
}

/// The recording hook refuses to overwrite a capture and limits its input.
#[test]
fn the_hook_writes_an_exclusive_capture_of_bounded_input() {
    let source = hook_source();
    assert!(source.starts_with("#!/bin/sh\n"), "{source}");
    assert!(source.contains("set -C"), "{source}");
    assert!(
        source.contains("$CLI_GIT_PUSH_FACTS_DIRECTORY/$$.capture"),
        "{source}"
    );
    assert!(
        source.contains(&format!("head -c {}", MAX_HOOK_INPUT_BYTES + 1)),
        "{source}"
    );
}

/// The limit itself is complete input; the one extra byte the hook reads is proof that input was cut short.
#[test]
fn only_input_past_the_limit_exceeds_the_capture_limit() {
    let limit = usize::try_from(MAX_HOOK_INPUT_BYTES).expect("the limit fits in memory");
    assert!(!exceeds_capture_limit(0));
    assert!(!exceeds_capture_limit(limit - 1));
    assert!(!exceeds_capture_limit(limit));
    assert!(exceeds_capture_limit(limit + 1));
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

/// Confirmation refuses a destination whose value differs from the negotiated one.
#[test]
fn confirmation_refuses_a_remote_ref_that_changed_after_negotiation() {
    let (repo, origin, base) = repository_with_origin("mp-confirm-changed");
    let changed = ProbedUpdate {
        local_ref: "refs/heads/main".to_owned(),
        local_oid: head_of(&repo),
        remote_name: "origin".to_owned(),
        remote_location: origin.to_str().expect("utf-8").to_owned(),
        remote_ref: "refs/heads/main".to_owned(),
        advertised_remote_oid: "1".repeat(40),
    };
    assert!(confirm_group(&git_for(&repo), origin.to_str().expect("utf-8"), &[changed]).is_err());
    let matching = ProbedUpdate {
        local_ref: "refs/heads/main".to_owned(),
        local_oid: head_of(&repo),
        remote_name: "origin".to_owned(),
        remote_location: origin.to_str().expect("utf-8").to_owned(),
        remote_ref: "refs/heads/main".to_owned(),
        advertised_remote_oid: base.clone(),
    };
    let confirmed = confirm_group(
        &git_for(&repo),
        origin.to_str().expect("utf-8"),
        &[matching],
    )
    .expect("confirms");
    assert_eq!(confirmed[0].remote_oid.as_deref(), Some(base.as_str()));
    remove(repo.parent().expect("root"));
}

/// A remote ref deleted after negotiation is refused: the advertised tip no longer exists.
#[test]
fn confirmation_refuses_a_remote_ref_deleted_after_negotiation() {
    let (repo, origin, base) = repository_with_origin("mp-confirm-deleted");
    let deleted = ProbedUpdate {
        local_ref: "refs/heads/main".to_owned(),
        local_oid: head_of(&repo),
        remote_name: "origin".to_owned(),
        remote_location: origin.to_str().expect("utf-8").to_owned(),
        remote_ref: "refs/heads/gone".to_owned(),
        advertised_remote_oid: base,
    };
    assert!(confirm_group(&git_for(&repo), origin.to_str().expect("utf-8"), &[deleted]).is_err());
    remove(repo.parent().expect("root"));
}

/// The scratch guard removes its directory when it is dropped, on every path.
#[test]
fn the_scratch_guard_removes_the_probe_directory() {
    let root = fixture("mp-scratch-guard");
    let scratch = root.join("probe");
    std::fs::create_dir_all(scratch.join("captures")).expect("scratch");
    drop(ScratchGuard(scratch.clone()));
    assert!(!scratch.exists());
    remove(&root);
}

/// A probe that fails before or after Git ran leaves no scratch directory: the captures name unpublished refs.
#[test]
fn a_failed_probe_removes_its_scratch_directory() {
    let root = fixture("mp-scratch-failure");
    let (repo, _origin, _base) = repository_with_origin("mp-scratch-failure-repo");
    // Git cannot start.
    let missing = PushGit {
        real_git: PathBuf::from("/nonexistent/git"),
        cwd: repo.clone(),
        environment: fixture_environment(),
        lfs_pre_push: None,
    };
    let not_started = root.join("not-started");
    assert!(capture_probed_updates(&missing, &[], &[], &not_started).is_err());
    assert!(!not_started.exists(), "the scratch directory of a probe that could not start remains");
    // Git runs, and the hook captures nothing because the remote cannot be reached.
    let nothing_captured = root.join("nothing-captured");
    let unreachable = post(&["/nonexistent-remote-for-the-probe", "main"]);
    assert!(capture_probed_updates(&git_for(&repo), &[], &unreachable, &nothing_captured).is_err());
    assert!(!nothing_captured.exists(), "the scratch directory of a probe with no capture remains");
    // A scratch path that exists already is not adopted and not removed.
    let present = root.join("present");
    std::fs::create_dir_all(&present).expect("present");
    assert!(capture_probed_updates(&git_for(&repo), &[], &[], &present).is_err());
    assert!(present.exists(), "a directory the probe did not create was removed");
    remove(repo.parent().expect("root"));
    remove(&root);
}

/// The probe error prints as its diagnostic, so a refused push names what could not be established.
#[test]
fn the_probe_error_displays_its_message() {
    let error = ManualPushProbeError {
        message: String::from("the probe could not run"),
    };
    assert_eq!(error.to_string(), "the probe could not run");
}

/// The recording hook accepts exactly 16 MiB of standard input, the incumbent's limit.
#[test]
fn the_hook_input_limit_is_sixteen_mebibytes() {
    assert_eq!(MAX_HOOK_INPUT_BYTES, 16_777_216);
}
