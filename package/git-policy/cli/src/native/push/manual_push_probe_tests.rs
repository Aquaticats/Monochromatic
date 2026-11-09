//! Controls for the manual-push probe against real repositories and bare remotes,
//! with Git writing the report.
//! The corpus is `package/git-policy/cli/src/manual-push.integration.unit.test.ts` (probe and
//! ls-remote cases) and `package/git-policy/cli/src/policy-engine/manual-push-scope.unit.test.ts`,
//! both deleted on 2026-10-13 in `642b6d72cb` with the TypeScript wrapper,
//! so `git show 642b6d72cb^` holds each.
//! The update-discovery cases are reproduced here.

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

/// The home directory of a fixture repository at `root`,
/// beside it,
/// where the per-user scratch directory is made.
fn home_beside(root: &Path) -> PathBuf {
    return root.parent().expect("fixture root").join("home");
}

/// Coordinator Git for a fixture repository at `root`,
/// with a home beside the repository.
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

/// The object a ref names in a bare remote,
/// or nothing.
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

/// A new branch pushed by the probe is one update:
/// a local object,
/// no remote object,
/// the named remote.
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

/// A destination Git cannot reach yields no capture,
/// so the probe fails instead of reporting no updates.
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

/// A push Git refuses before it reports any destination carries Git's own status and words,
/// taken from a plain run.
#[test]
fn a_push_git_refuses_carries_gits_status_and_words() {
    let (repo, _origin, _base) = repository_with_origin("mp-probe-refusal");
    for arguments in [
        vec!["origin", "--delete", "no-such-branch"],
        vec!["origin", "no-such-source:main"],
        vec!["no-such-remote", "main"],
        vec!["--no-such-option", "origin"],
    ] {
        let plain = std::process::Command::new("/usr/bin/git")
            .current_dir(&repo)
            .env_clear()
            .envs(fixture_environment())
            .env("HOME", home_beside(&repo))
            .arg("push")
            .args(&arguments)
            .output()
            .expect("plain push");
        let error = probe_manual_push_updates(&git_for(&repo), &[], &post(&arguments)).expect_err("Git refuses this push");
        let refusal = error.refusal.expect("Git's refusal is carried");
        assert_eq!(Some(refusal.code), plain.status.code(), "{arguments:?}");
        assert_eq!(refusal.stderr, String::from_utf8_lossy(&plain.stderr), "{arguments:?}");
    }
    remove(repo.parent().expect("root"));
}

/// A failure of the probe's own reading is no refusal by Git:
/// a Git that fails without a word,
/// and a report the probe
///  cannot parse,
/// stay failures of the wrapper.
#[test]
fn a_failure_without_gits_words_stays_a_failure_of_the_probe() {
    let (repo, _origin, _base) = repository_with_origin("mp-probe-no-refusal");
    let mut silent = git_for(&repo);
    silent.real_git = PathBuf::from("/usr/bin/false");
    let silent_error = probe_manual_push_updates(&silent, &[], &post(&["origin", "main"])).expect_err("a silent failure");
    assert_eq!(silent_error.refusal, None, "{silent_error}");
    let mut garbled = git_for(&repo);
    garbled.real_git = PathBuf::from("/usr/bin/echo");
    let error = probe_manual_push_updates(&garbled, &[], &post(&["origin", "main"])).expect_err("an unreadable report");
    assert_eq!(error.refusal, None, "{error}");
    remove(repo.parent().expect("root"));
}

/// A `--` that is the value of an option is not the terminator,
/// and `--end-of-options` ends the options as `--` does:
/// the
///  probe's own options go before the one Git reads,
/// never between an option and its value or among the refs.
#[test]
fn probe_arguments_place_their_options_before_the_terminator_git_reads() {
    let own: [&str; 3] = ["--no-quiet", "--dry-run", "--porcelain"];
    // The typed words, and how many of them come before the probe's options.
    for (typed, before) in [
        (&["--receive-pack", "--", "origin", "--", "main"][..], 3),
        (&["--exec", "--", "origin", "main"][..], 4),
        (&["origin", "--end-of-options", "main"][..], 1),
        (&["origin", "main"][..], 2),
    ] {
        let arguments: Vec<OsString> = probe_arguments(&[], &post(typed));
        let text: Vec<String> = arguments.iter().map(|argument| { return argument.to_string_lossy().into_owned(); }).collect();
        let push: usize = text.iter().position(|word| { return word == "push"; }).expect("push");
        let options: &[String] = &text[push + 1 + before..push + 1 + before + own.len()];
        assert_eq!(options, own, "{text:?}");
        let rest: Vec<&str> = text[push + 1..].iter().map(String::as_str).filter(|word| { return !own.contains(word); }).collect();
        assert_eq!(rest, typed, "{text:?}");
    }
}

/// The probe's arguments keep Git's options before the terminator,
/// switch every hook off and ask for full names.
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

/// A configured `pre-push` hook of the repository does not run for the probe,
/// and a dry run publishes nothing.
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

/// A bare push resolves the remote the way Git does,
/// and a tag is reported with its own object.
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
    let moved = confirm_group(&git_for(&repo), location, "origin", changed.as_slice()).expect_err("the remote moved");
    assert_eq!(moved.cause, ManualPushCause::RemoteMoved, "{moved}");
    let matching = reported(RemoteBefore::Known(base.clone()), "refs/heads/main");
    let confirmed = confirm_group(&git_for(&repo), location, "origin", matching.as_slice()).expect("confirms");
    assert_eq!(confirmed[0].remote_oid.as_deref(), Some(base.as_str()));
    let deleted_since = reported(RemoteBefore::Known(base), "refs/heads/gone");
    assert!(confirm_group(&git_for(&repo), location, "origin", deleted_since.as_slice()).is_err(), "the reported tip no longer exists");
    let appeared_since = reported(RemoteBefore::Absent, "refs/heads/main");
    assert!(confirm_group(&git_for(&repo), location, "origin", appeared_since.as_slice()).is_err(), "a new ref that now exists");
    remove(repo.parent().expect("root"));
}

/// A Git stand-in that answers `ls-remote` as an unreachable remote would,
/// with a password in the address it names,
/// and
///  runs the real Git for every other query.
/// Returns its path.
fn git_that_cannot_list_the_remote(root: &Path) -> PathBuf {
    let stand_in = root.join("git-stand-in");
    let script = "#!/bin/sh\nfor word in \"$@\"; do\n  if [ \"$word\" = ls-remote ]; then\n    echo \"fatal: unable to access 'https://person:hunter2@example.invalid/r.git/': could not resolve host\" >&2\n    exit 128\n  fi\ndone\nexec /usr/bin/git \"$@\"\n";
    std::fs::write(&stand_in, script).expect("stand-in");
    std::fs::set_permissions(&stand_in, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("executable");
    return stand_in;
}

/// A remote whose refs cannot be read is the unreachable-remote cause,
/// and the message never carries the address's password.
#[test]
fn an_unreadable_remote_is_the_unreachable_cause_and_masks_the_password() {
    let (repo, _origin, _base) = repository_with_origin("mp-cause-unreachable");
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=next"]);
    let mut stand_in = git_for(&repo);
    stand_in.real_git = git_that_cannot_list_the_remote(repo.parent().expect("root"));
    let error = probe_manual_push_updates(&stand_in, &[], &post(&["origin", "main"])).expect_err("the remote cannot be listed");
    assert_eq!(error.cause, ManualPushCause::RemoteUnreachable, "{error}");
    assert_eq!(error.refusal, None);
    assert!(!error.message.contains("hunter2"), "the password must not appear: {error}");
    assert!(error.message.contains("example.invalid"), "the address is still named: {error}");
    remove(repo.parent().expect("root"));
}

/// A Git that cannot be started is its own cause,
/// a silent failure and an unreadable report are theirs.
#[test]
fn git_that_cannot_start_or_say_why_or_be_read_has_its_own_cause() {
    let (repo, _origin, _base) = repository_with_origin("mp-cause-git");
    let arguments = post(&["origin", "main"]);
    let mut absent = git_for(&repo);
    absent.real_git = repo.parent().expect("root").join("no-such-git");
    let not_started = probe_manual_push_updates(&absent, &[], &arguments).expect_err("Git cannot start");
    assert_eq!(not_started.cause, ManualPushCause::GitNotStarted, "{not_started}");
    let mut silent = git_for(&repo);
    silent.real_git = PathBuf::from("/usr/bin/false");
    let not_said = probe_manual_push_updates(&silent, &[], &arguments).expect_err("Git says nothing");
    assert_eq!(not_said.cause, ManualPushCause::GitSilent, "{not_said}");
    let mut garbled = git_for(&repo);
    garbled.real_git = PathBuf::from("/usr/bin/echo");
    let unreadable = probe_manual_push_updates(&garbled, &[], &arguments).expect_err("an unreadable report");
    assert_eq!(unreadable.cause, ManualPushCause::UnexpectedOutput, "{unreadable}");
    remove(repo.parent().expect("root"));
}

/// The probe error displays its message.
#[test]
fn the_probe_error_displays_its_message() {
    assert_eq!(probe_error(ManualPushCause::SelfExplained, String::from("why")).to_string(), "why");
}

/// `-u`,
/// `--set-upstream` and `push.autoSetupRemote` add a line to Git's own report;
/// the probe still reads the update
///  (real Git output,
/// in a repository whose branch has no upstream yet).
#[test]
fn the_three_ways_to_set_an_upstream_do_not_stop_the_probe() {
    for (name, prefix, region) in [
        ("short", Vec::new(), vec!["-u", "origin", "feature"]),
        ("long", Vec::new(), vec!["--set-upstream", "origin", "feature"]),
        ("auto", vec!["-c", "push.autoSetupRemote=true"], Vec::new()),
    ] {
        let (repo, _origin, _base) = repository_with_origin(&format!("mp-probe-upstream-{name}"));
        git(&repo, &["checkout", "--quiet", "-b", "feature"]);
        git(&repo, &["commit", "--quiet", "--allow-empty", "--message=feature"]);
        let updates = probe_manual_push_updates(&git_for(&repo), &post(&prefix), &post(&region))
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(updates.len(), 1, "{name}");
        assert_eq!(updates[0].remote_ref, "refs/heads/feature", "{name}");
        remove(repo.parent().expect("root"));
    }
}

/// `-q` and `--quiet` silence the report the probe reads unless the probe cancels them;
/// every spelling still reports.
#[test]
fn a_quiet_push_is_probed_like_any_other() {
    let (repo, _origin, _base) = repository_with_origin("mp-probe-quiet");
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=next"]);
    let tip = head_of(&repo);
    for region in [vec!["-q", "origin", "main"], vec!["--quiet", "origin", "main"], vec!["origin", "main", "-q"]] {
        let updates = probe_manual_push_updates(&git_for(&repo), &[], &post(&region)).expect("probes");
        assert_eq!(updates.len(), 1, "{region:?}");
        assert_eq!(updates[0].local_oid.as_deref(), Some(tip.as_str()), "{region:?}");
    }
    remove(repo.parent().expect("root"));
}

/// A destination typed as a path or URL is its own location and starts no query for push URLs;
/// a remote name does,
///  because Git's printed address has credentials removed and is never used to reach the remote.
#[test]
fn only_a_remote_name_needs_the_push_url_query() {
    let (repo, origin, _base) = repository_with_origin("mp-probe-locations");
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=next"]);
    let root = repo.parent().expect("root").to_path_buf();
    let log = root.join("starts.log");
    let wrapper = root.join("counting-git");
    crate::test_support::executable(
        &wrapper,
        format!("#!/bin/sh\necho \"$@\" >> '{}'\nexec /usr/bin/git \"$@\"\n", log.display()).as_bytes(),
    );
    let mut counting = git_for(&repo);
    counting.real_git = wrapper;
    let starts_after = |repository: &str| -> String {
        let _ = std::fs::remove_file(&log);
        probe_manual_push_updates(&counting, &[], &post(&[repository, "main"])).expect("probes");
        return std::fs::read_to_string(&log).expect("log");
    };
    let typed: String = starts_after(origin.to_str().expect("utf-8"));
    assert!(!typed.contains("remote get-url"), "{typed}");
    assert!(typed.contains("ls-remote"), "the destination is still confirmed: {typed}");
    let named: String = starts_after("origin");
    assert!(named.contains("remote get-url --push --all origin"), "{named}");
    remove(&root);
}

/// Text typed as the repository is told from a remote name by its shape.
#[test]
fn a_location_is_told_from_a_remote_name_by_its_shape() {
    for location in [
        "https://user:token@example.invalid/r.git",
        "ssh://git@example.invalid/r.git",
        "git@example.invalid:r.git",
        "/srv/r.git",
        "./r.git",
        "../r.git",
    ] {
        assert!(looks_like_location(location), "{location}");
    }
    for name in ["origin", "upstream", "my-remote", "team/fork"] {
        assert!(!looks_like_location(name), "{name}");
    }
}

/// A source selected by a message search holds a colon;
/// the probe still reports the commit it names (real Git output).
#[test]
fn a_revision_expression_with_a_colon_is_probed_as_the_commit_it_names() {
    let (repo, _origin, _base) = repository_with_origin("mp-probe-colon-source");
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=uniquemessage"]);
    let tip = head_of(&repo);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=later"]);
    let updates = probe_manual_push_updates(&git_for(&repo), &[], &post(&["origin", ":/uniquemessage:refs/heads/new"]))
        .expect("probes");
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].local_oid.as_deref(), Some(tip.as_str()), "not a deletion");
    assert_eq!(updates[0].remote_ref, "refs/heads/new");
    remove(repo.parent().expect("root"));
}

/// The estimate covers the measured peak of a query for the whole history,
/// and is that measurement rounded up to a power of two.
#[test]
fn the_query_estimate_is_the_measured_whole_history_peak_rounded_up_to_a_power_of_two() {
    const { assert!(QUERY_MEMORY >= MEASURED_WHOLE_HISTORY_QUERY, "the query estimate must cover the measured whole-history query") };
    assert_eq!(QUERY_MEMORY, MEASURED_WHOLE_HISTORY_QUERY.next_power_of_two());
}

/// A Git stand-in that answers `ls-remote` with one record per ref it was asked for:
/// the ref's own name as its object name,
/// so an update that reads the wrong record says so.
/// Returns its path.
fn git_that_lists_every_ref_asked_for(root: &Path) -> PathBuf {
    let stand_in = root.join("git-listing-stand-in");
    let script = "#!/bin/sh\nfor word in \"$@\"; do\n  case \"$word\" in\n    refs/*) printf '%s\\t%s\\n' \"$word\" \"$word\" ;;\n  esac\ndone\n";
    std::fs::write(&stand_in, script).expect("stand-in");
    std::fs::set_permissions(&stand_in, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("executable");
    return stand_in;
}

/// What:
///  Confirming a destination reads each listed ref once and gives each update its own record.
/// Why:
///  `AGENTS.md` `RG2`:
/// a mirror or `--all` push publishes hundreds of refs,
///  and asking every earlier listed record whether the candidate is its ref
///  is quadratic in them.
///  The seam counts the collected records a question looks at,
///  which is one per record and one per update when a map answers
///  (`foundation/collected_elements_looked_at_tests.rs` proves the count separates the two).
#[test]
fn confirming_a_destination_reads_each_listed_ref_once() {
    let root = fixture("mp-probe-confirm-once");
    std::fs::create_dir_all(root.as_path()).expect("fixture root");
    let mut environment: Vec<(OsString, OsString)> = fixture_environment();
    environment.push((OsString::from("HOME"), home_beside(root.as_path()).into_os_string()));
    let coordinator = PushGit {
        real_git: git_that_lists_every_ref_asked_for(root.as_path()),
        cwd: root.clone(),
        environment,
        lfs: false,
    };
    let width: usize = 2048;
    let mut updates: Vec<(PorcelainUpdate, Option<String>)> = Vec::new();
    for number in 0..width {
        let to: String = format!("refs/heads/branch-{number:05}");
        updates.push((
            PorcelainUpdate { from: to.clone(), to: to.clone(), before: RemoteBefore::Unknown, after: None },
            Some(format!("{number:040x}")),
        ));
    }
    let before: u64 = crate::collected_elements_looked_at::looked_at();
    let confirmed: Vec<PushUpdate> = confirm_group(&coordinator, "origin", "origin", updates.as_slice()).expect("confirms");
    let looked: u64 = crate::collected_elements_looked_at::looked_at() - before;
    assert_eq!(confirmed.len(), width);
    for (index, update) in confirmed.iter().enumerate() {
        assert_eq!(update.remote_ref, updates[index].0.to, "the updates keep their order");
        assert_eq!(update.remote_oid.as_deref(), Some(update.remote_ref.as_str()), "each update read its own record");
    }
    assert_eq!(
        looked,
        2 * width as u64,
        "{looked} collected records looked at for {width} listed refs and {width} updates; one pass over the listing and one lookup per update is linear"
    );
    remove(root.as_path());
}
