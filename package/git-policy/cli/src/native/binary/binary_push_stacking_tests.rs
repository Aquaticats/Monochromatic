//! Binary-level controls for an explicit `git push` that lost a race to another session:
//!  the built executable merges what landed on the remote,
//! publishes the merge through its own push,
//! and leaves every
//!  other failure exactly as Git reported it.
//! A recording `pre-receive` hook in the bare origin (a server-side hook,
//! which a local-path remote runs itself) shows
//!  which pushes reached the origin and whether each carried the marker of a stacked push.
//! No hook of the pushing repository is used:
//! the wrapper runs no configured hook.

use super::support::{Fixture, git, remove, fixture, repository, run_wrapped, stderr_of};
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// A repository,
/// its bare origin,
/// a second clone that wins races,
/// and the hook's record.
struct Race {
    /// The pushing repository.
    repo: PathBuf,
    /// The bare origin.
    origin: PathBuf,
    /// The clone another session pushes from.
    rival: PathBuf,
    /// The file the origin's `pre-receive` hook appends to.
    record: PathBuf,
}

/// Install a `pre-receive` hook in the bare origin that records the marker and every received object,
/// and exits with
///  `exit_when_stacked` when it runs inside a stacked push and `0` otherwise.
fn install_hook(origin: &Path, record: &Path, exit_when_stacked: i32) {
    let hooks: PathBuf = origin.join("hooks");
    std::fs::create_dir_all(&hooks).expect("hooks directory");
    let hook: PathBuf = hooks.join("pre-receive");
    let source = format!(
        "#!/bin/sh\n\
         printf 'marker=%s\\n' \"${{CLI_GIT_STACKED_PUSH-unset}}\" >> '{record}'\n\
         while read old_oid new_oid ref_name; do printf 'pushed=%s\\n' \"$new_oid\" >> '{record}'; done\n\
         [ \"${{CLI_GIT_STACKED_PUSH-}}\" = 1 ] && exit {exit_when_stacked}\n\
         exit 0\n",
        record = record.display(),
    );
    std::fs::write(&hook, source).expect("hook source");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("hook mode");
}

/// Commit `text` to `name` with real Git,
/// so no auto-push runs.
fn commit(fixture: &Fixture, repo: &Path, name: &str, text: &str) {
    std::fs::write(repo.join(name), text).expect("fixture file");
    git(fixture, repo, &["add", "--", name]);
    git(fixture, repo, &["commit", "--quiet", &format!("--message={name}"), "--", name]);
}

/// Build the fixture:
/// `main` on the origin,
/// one clone that wins races,
/// and the pushing repository one commit ahead
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
    install_hook(origin.as_path(), record.as_path(), hook_exit_when_stacked);
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

/// The hook's record,
/// one entry per line.
fn record_lines(race: &Race) -> Vec<String> {
    return std::fs::read_to_string(&race.record)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect();
}

/// How many times the hook ran,
/// one `marker=` line each.
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

/// A push that lost a race is published as a merge,
/// and Git's own report of the loss is still on standard error.
#[test]
fn a_push_that_lost_a_race_is_published_as_a_merge_through_the_gates() {
    let fixture = fixture("stack-lost-race");
    let race = race(&fixture, 0);
    let landed = oid(&fixture, race.repo.as_path(), "HEAD");
    let observed = run_wrapped(&fixture, race.repo.as_path(), &["push", "origin", "HEAD:main"]);
    let stderr = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{stderr}");
    assert!(stderr.contains("[rejected]"), "Git's own report is unchanged: {stderr}");
    assert!(stderr.contains("monochromatic-git-policy-cli: push: origin/main had new commits"), "{stderr}");
    let published = oid(&fixture, race.origin.as_path(), "main");
    assert!(reachable(&fixture, &race, &landed, &published));
    assert!(reachable(&fixture, &race, &oid(&fixture, race.rival.as_path(), "HEAD"), &published));
    let lines = record_lines(&race);
    // Only the stacked merge push reaches the origin's hook: Git rejects the typed push before it sends anything,
    //  and the wrapper's dry-run probes send nothing.
    assert_eq!(hook_runs(&race), 1, "{lines:?}");
    assert!(stacked_push_ran(&race), "the merge went through a stacked push: {lines:?}");
    assert!(lines.contains(&format!("pushed={published}")), "the origin's hook saw the merge: {lines:?}");
    remove(&fixture);
}

/// A gate that refuses the merge stops the stacking:
/// the origin is untouched and the refusal's status stands.
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
        "the merge was pushed through a stacked push and the origin's hook refused it: {:?}",
        record_lines(&race)
    );
    remove(&fixture);
}

/// A dry run is forwarded as it is:
/// nothing is published and no stacked push runs.
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

/// A hook that holds the first push that reaches the origin and lets every later one through:
/// the push the person typed stands in the origin's `pre-receive` after advertising the value it sends,
/// which is the window the commit's own auto-push lands in.
/// `mkdir` decides which push is the first,
/// because it is the one step two hooks cannot both win.
fn install_holding_hook(origin: &Path, gate: &Path, started: &Path) {
    let hook: PathBuf = origin.join("hooks").join("pre-receive");
    std::fs::write(
        &hook,
        format!(
            "#!/bin/sh\n\
             if [ -e '{gate}' ] && mkdir '{first}' 2>/dev/null; then\n\
             : > '{started}'\n\
             while [ -e '{gate}' ]; do sleep 0.01; done\n\
             fi\n\
             exit 0\n",
            gate = gate.display(),
            started = started.display(),
            first = started.with_extension("first").display(),
        ),
    )
    .expect("hook source");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("hook mode");
}

/// Wait,
/// bounded,
/// until `path` exists.
fn wait_for(path: &Path) {
    let began = std::time::Instant::now();
    while !path.exists() {
        assert!(
            began.elapsed() < std::time::Duration::from_secs(60),
            "the origin's hook never took the push: {}",
            path.display()
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

/// A push held in the origin's hook while another push of the very same commit lands ends with success,
/// because the origin then holds what it sends:
/// the commit's own auto-push is what landed it first in production.
#[test]
fn a_push_the_auto_push_landed_while_it_waited_ends_with_success() {
    let fixture = fixture("stack-published-first");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let origin: PathBuf = fixture.root.join("origin.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "origin.git"]);
    git(&fixture, repo.as_path(), &["remote", "add", "origin", origin.to_str().expect("utf-8")]);
    git(&fixture, repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "main"]);
    commit(&fixture, repo.as_path(), "next.txt", "next\n");
    let pushed = oid(&fixture, repo.as_path(), "HEAD");
    let gate: PathBuf = fixture.root.join("gate");
    let started: PathBuf = fixture.root.join("hook-started.txt");
    std::fs::write(&gate, b"").expect("gate");
    install_holding_hook(origin.as_path(), gate.as_path(), started.as_path());
    let held: std::process::Child = super::support::bounded_for(
        &fixture,
        fixture.root.join("bin/git").as_path(),
        super::support::wrapper_first_path(&fixture).as_os_str(),
        "120",
    )
    .current_dir(repo.as_path())
    .args(["push"])
    .stdin(std::process::Stdio::null())
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .expect("the wrapped push starts");
    wait_for(started.as_path());
    // The auto-push of the same commit lands it, so the origin no longer stands where the held push found it.
    git(&fixture, repo.as_path(), &["push", "--quiet", "origin", "HEAD:refs/heads/main"]);
    std::fs::remove_file(&gate).expect("gate released");
    let output: std::process::Output = held.wait_with_output().expect("the wrapped push ends");
    let report: String = format!(
        "{}{}",
        String::from_utf8_lossy(output.stdout.as_slice()),
        String::from_utf8_lossy(output.stderr.as_slice())
    );
    assert_eq!(oid(&fixture, origin.as_path(), "main"), pushed, "the origin holds the pushed commit");
    assert!(report.contains("cannot lock ref"), "the origin refused the held push's stale expectation: {report}");
    assert_eq!(output.status.code(), Some(0), "the origin holds what this push sends: {report}");
    assert!(report.contains("already holds everything this push sends"), "{report}");
    remove(&fixture);
}
