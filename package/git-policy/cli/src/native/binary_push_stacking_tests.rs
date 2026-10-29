//! Binary-level controls for an explicit `git push` that lost a race to another session:
//!  the built executable merges what landed on the remote, publishes the merge through its own push, and leaves every
//!  other failure exactly as Git reported it.
//! A recording pre-push hook shows which pushes Git ran and whether each carried the marker of a stacked push.

use super::support::{Fixture, git, remove, fixture, repository, run_wrapped, stderr_of};
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// A repository, its bare origin, a second clone that wins races, and the hook's record.
struct Race {
    /// The pushing repository.
    repo: PathBuf,
    /// The bare origin.
    origin: PathBuf,
    /// The clone another session pushes from.
    rival: PathBuf,
    /// The file the pre-push hook appends to.
    record: PathBuf,
}

/// Install a pre-push hook that records the marker and every pushed object, and exits with `exit_when_stacked`
///  when it runs inside a stacked push and `0` otherwise.
fn install_hook(fixture: &Fixture, repo: &Path, record: &Path, exit_when_stacked: i32) {
    let hooks: PathBuf = fixture.root.join("hooks");
    std::fs::create_dir_all(&hooks).expect("hooks directory");
    let hook: PathBuf = hooks.join("pre-push");
    let source = format!(
        "#!/bin/sh\n\
         printf 'marker=%s\\n' \"${{CLI_GIT_STACKED_PUSH-unset}}\" >> '{record}'\n\
         while read local_ref local_oid remote_ref remote_oid; do printf 'pushed=%s\\n' \"$local_oid\" >> '{record}'; done\n\
         [ \"${{CLI_GIT_STACKED_PUSH-}}\" = 1 ] && exit {exit_when_stacked}\n\
         exit 0\n",
        record = record.display(),
    );
    std::fs::write(&hook, source).expect("hook source");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("hook mode");
    // A configuration hook: traditional hooks never run through the wrapper's real-Git starts.
    git(fixture, repo, &["config", "hook.control.event", "pre-push"]);
    git(fixture, repo, &["config", "hook.control.command", hook.to_str().expect("utf-8")]);
}

/// Commit `text` to `name` with real Git, so no auto-push runs.
fn commit(fixture: &Fixture, repo: &Path, name: &str, text: &str) {
    std::fs::write(repo.join(name), text).expect("fixture file");
    git(fixture, repo, &["add", "--", name]);
    git(fixture, repo, &["commit", "--quiet", &format!("--message={name}"), "--", name]);
}

/// Build the fixture: `main` on the origin, one clone that wins races, and the pushing repository one commit ahead
///  of an origin that has since moved.
fn race(fixture: &Fixture, hook_exit_when_stacked: i32) -> Race {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    let origin: PathBuf = fixture.root.join("origin.git");
    git(
        fixture,
        fixture.root.as_path(),
        &["init", "--quiet", "--bare", "--initial-branch=main", "origin.git"],
    );
    git(fixture, repo.as_path(), &["remote", "add", "origin", origin.to_str().expect("utf-8")]);
    git(fixture, repo.as_path(), &["push", "--quiet", "origin", "main"]);
    let rival: PathBuf = fixture.root.join("rival");
    git(
        fixture,
        fixture.root.as_path(),
        &["clone", "--quiet", origin.to_str().expect("utf-8"), "rival"],
    );
    commit(fixture, rival.as_path(), "rival.txt", "rival\n");
    git(fixture, rival.as_path(), &["push", "--quiet", "origin", "HEAD:main"]);
    commit(fixture, repo.as_path(), "local.txt", "local\n");
    let record: PathBuf = fixture.root.join("hook-record.txt");
    install_hook(fixture, repo.as_path(), record.as_path(), hook_exit_when_stacked);
    return Race {
        repo,
        origin,
        rival,
        record,
    };
}

/// The commit a ref names in a repository.
fn oid(fixture: &Fixture, repo: &Path, name: &str) -> String {
    return String::from_utf8_lossy(&git(fixture, repo, &["rev-parse", name]).stdout)
        .trim()
        .to_owned();
}

/// Whether `ancestor` is reachable from `descendant` in the origin.
fn reachable(fixture: &Fixture, race: &Race, ancestor: &str, descendant: &str) -> bool {
    return super::support::direct(fixture)
        .current_dir(&race.origin)
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .status()
        .expect("git")
        .success();
}

/// The hook's record, one entry per line.
fn record_lines(race: &Race) -> Vec<String> {
    return std::fs::read_to_string(&race.record)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect();
}

/// How many times the hook ran, one `marker=` line each.
fn hook_runs(race: &Race) -> usize {
    return record_lines(race)
        .iter()
        .filter(|line| {
            return line.starts_with("marker=");
        })
        .count();
}

/// Whether a push that carried the stacked-push marker reached the hook.
fn stacked_push_ran(race: &Race) -> bool {
    return record_lines(race).iter().any(|line| {
        return line == "marker=1";
    });
}

/// A push that lost a race is published as a merge, and Git's own report of the loss is still on standard error.
#[test]
fn a_push_that_lost_a_race_is_published_as_a_merge_through_the_gates() {
    let fixture = fixture("stack-lost-race");
    let race = race(&fixture, 0);
    let landed = oid(&fixture, race.repo.as_path(), "HEAD");
    let observed = run_wrapped(&fixture, race.repo.as_path(), &["push", "origin", "HEAD:main"]);
    let stderr = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{stderr}");
    assert!(stderr.contains("[rejected]"), "Git's own report is unchanged: {stderr}");
    assert!(stderr.contains("cli-git: push: origin/main had new commits"), "{stderr}");
    let published = oid(&fixture, race.origin.as_path(), "main");
    assert!(reachable(&fixture, &race, &landed, &published));
    assert!(reachable(&fixture, &race, &oid(&fixture, race.rival.as_path(), "HEAD"), &published));
    let lines = record_lines(&race);
    // A configuration hook also runs in the wrapper's dry-run probes (the probe replaces only the hooks directory):
    //  the scope probe and the push, then the integration probe, the stacked push's scope probe and the stacked push.
    assert_eq!(hook_runs(&race), 5, "{lines:?}");
    assert_eq!(lines.first().map(String::as_str), Some("marker=unset"), "{lines:?}");
    assert!(stacked_push_ran(&race), "the merge went through a stacked push: {lines:?}");
    assert!(lines.contains(&format!("pushed={published}")), "the hook saw the merge: {lines:?}");
    remove(&fixture);
}

/// A gate that refuses the merge stops the stacking: the origin is untouched and the refusal's status stands.
#[test]
fn a_gate_that_refuses_the_merge_stops_the_stacking() {
    let fixture = fixture("stack-gate-refuses");
    let race = race(&fixture, 1);
    let before = oid(&fixture, race.origin.as_path(), "main");
    let observed = run_wrapped(&fixture, race.repo.as_path(), &["push", "origin", "HEAD:main"]);
    let stderr = stderr_of(&observed);
    assert_eq!(observed.code, Some(1), "{stderr}");
    assert_eq!(oid(&fixture, race.origin.as_path(), "main"), before);
    assert!(!stderr.contains("had new commits"), "{stderr}");
    assert!(
        stacked_push_ran(&race),
        "the merge was pushed through a stacked push and the hook refused it: {:?}",
        record_lines(&race)
    );
    remove(&fixture);
}

/// A dry run is forwarded as it is: nothing is published and no stacked push runs.
#[test]
fn a_dry_run_runs_no_stacked_push() {
    let fixture = fixture("stack-ineligible");
    let race = race(&fixture, 0);
    let before = oid(&fixture, race.origin.as_path(), "main");
    let observed = run_wrapped(&fixture, race.repo.as_path(), &["push", "--dry-run", "origin", "HEAD:main"]);
    assert_eq!(oid(&fixture, race.origin.as_path(), "main"), before);
    assert!(!stderr_of(&observed).contains("had new commits"));
    assert!(
        !stacked_push_ran(&race),
        "a dry run runs no stacked push: {:?}",
        record_lines(&race)
    );
    remove(&fixture);
}

/// A real conflict publishes nothing and names the paths.
#[test]
fn a_conflict_publishes_nothing_and_names_the_paths() {
    let fixture = fixture("stack-conflict");
    let race = race(&fixture, 0);
    commit(&fixture, race.rival.as_path(), "same.txt", "rival side\n");
    git(&fixture, race.rival.as_path(), &["push", "--quiet", "origin", "HEAD:main"]);
    commit(&fixture, race.repo.as_path(), "same.txt", "local side\n");
    let before = oid(&fixture, race.origin.as_path(), "main");
    let observed = run_wrapped(&fixture, race.repo.as_path(), &["push", "origin", "HEAD:main"]);
    let stderr = stderr_of(&observed);
    assert_eq!(observed.code, Some(1), "{stderr}");
    assert!(stderr.contains("conflicts in: same.txt"), "{stderr}");
    assert_eq!(oid(&fixture, race.origin.as_path(), "main"), before);
    remove(&fixture);
}
