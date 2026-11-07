//! A push's destinations and updates read side by side:
//! the Git queries of independent destinations and updates overlap in time
//!  (timestamps written by a stand-in Git),
//! the answers keep the order Git printed them,
//! and one that fails decides the error.

use super::*;
use crate::manual_push_probe::{MOST_CONCURRENT_LOCAL_QUERIES, probe_manual_push_updates_on};
use crate::parallel_budget::HostResources;
use crate::test_support::{fixture, git, remove};
use std::path::{Path, PathBuf};

/// A host large enough that the bound of the SPEC,
/// not the threads,
/// limits the overlap.
const LARGE_HOST: HostResources = HostResources { parallelism: 64, memory_available: None };

/// How long the stand-in Git holds a query it delays,
/// in whole and fractional seconds as `sleep` reads it.
const HOLD: &str = "0.6";

/// Environment of every Git child of these fixtures.
fn fixture_environment() -> Vec<(OsString, OsString)> {
    return [("PATH", "/usr/bin:/bin"), ("GIT_CONFIG_NOSYSTEM", "1"), ("GIT_CONFIG_GLOBAL", "/nonexistent-global-config"), ("GIT_AUTHOR_NAME", "Fixture"), ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"), ("GIT_COMMITTER_NAME", "Fixture"), ("GIT_COMMITTER_EMAIL", "fixture@example.invalid")]
        .iter()
        .map(|(name, value)| { return (OsString::from(name), OsString::from(value)); })
        .collect();
}

/// A Git that holds every query of the subcommand `held` for `HOLD` and appends the start and the end of the hold,
/// in
///  nanoseconds since the epoch,
/// to `log`;
/// every other query goes straight to the real Git.
fn timing_git(root: &Path, held: &str, log: &Path) -> PathBuf {
    let stand_in: PathBuf = root.join("git-timing");
    let script: String = format!(
        "#!/bin/sh\nfor word in \"$@\"; do\n  if [ \"$word\" = {held} ]; then\n    started=$(date +%s%N)\n    sleep {HOLD}\n    /usr/bin/git \"$@\"\n    status=$?\n    echo \"$started $(date +%s%N)\" >> {log}\n    exit $status\n  fi\ndone\nexec /usr/bin/git \"$@\"\n",
        log = log.display()
    );
    std::fs::write(&stand_in, script).expect("stand-in");
    std::fs::set_permissions(&stand_in, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("executable");
    return stand_in;
}

/// The held intervals the stand-in wrote,
/// sorted by start.
fn intervals(log: &Path) -> Vec<(u128, u128)> {
    let text: String = std::fs::read_to_string(log).unwrap_or_default();
    let mut found: Vec<(u128, u128)> = text
        .lines()
        .filter_map(|line: &str| { return line.split_once(' ').and_then(|(start, end)| { return Some((start.parse().ok()?, end.parse().ok()?)); }); })
        .collect();
    found.sort_unstable();
    return found;
}

/// Whether every interval starts before the first one ends:
/// all of them were held at one moment.
fn all_overlap(found: &[(u128, u128)]) -> bool {
    let first_end: u128 = found.iter().map(|(_start, end)| { return *end; }).min().unwrap_or(0);
    return found.iter().all(|(start, _end)| { return *start < first_end; });
}

/// The most intervals that were held at one moment.
fn most_at_once(found: &[(u128, u128)]) -> usize {
    return found.iter().map(|(start, _end)| { return found.iter().filter(|(other_start, other_end)| { return other_start <= start && start < other_end; }).count(); }).max().unwrap_or(0);
}

/// Coordinator Git over a stand-in Git.
fn git_over(root: &Path, stand_in: &Path) -> PushGit {
    let mut environment = fixture_environment();
    environment.push((OsString::from("HOME"), root.parent().unwrap_or(root).join("home").into_os_string()));
    return PushGit { real_git: stand_in.to_path_buf(), cwd: root.to_path_buf(), environment, lfs: false };
}

/// A repository with one published commit on `main`,
/// `branches` unpublished branches `b0`,
/// `b1`,
/// ...,
/// and one remote `origin`
///  whose push URLs are `destinations` bare repositories.
fn repository_with(name: &str, destinations: usize, branches: usize) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir_all(&repo).expect("repo");
    git(&repo, &["init", "--quiet", "--initial-branch=main"]);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=base"]);
    for index in 0..destinations {
        let bare: PathBuf = root.join(format!("origin{index}.git"));
        git(&root, &["init", "--quiet", "--bare", bare.to_str().expect("utf-8")]);
        git(&repo, &["push", "--quiet", bare.to_str().expect("utf-8"), "HEAD:refs/heads/main"]);
        let address: &str = bare.to_str().expect("utf-8");
        git(&repo, &if index == 0 { vec!["remote", "add", "origin", address] } else { vec!["remote", "set-url", "--add", "origin", address] });
        git(&repo, &["remote", "set-url", "--add", "--push", "origin", address]);
    }
    // The remote-tracking `main` is what bounds each update's range, so the scope runs `rev-list` against it.
    git(&repo, &["fetch", "--quiet", "origin"]);
    for index in 0..branches {
        git(&repo, &["checkout", "--quiet", "-b", format!("b{index}").as_str(), "main"]);
        git(&repo, &["commit", "--quiet", "--allow-empty", format!("--message=branch {index}").as_str()]);
    }
    return (root, repo);
}

/// Two push URLs of one remote are confirmed with their `ls-remote` queries held at the same moment,
/// and the updates come back in
///  the order of the destinations.
#[test]
fn the_destinations_of_a_push_are_confirmed_side_by_side() {
    let (root, repo) = repository_with("mp-parallel-destinations", 2, 1);
    let log: PathBuf = root.join("ls-remote.log");
    let stand_in: PathBuf = timing_git(&root, "ls-remote", &log);
    let updates = probe_manual_push_updates_on(&LARGE_HOST, &git_over(&repo, &stand_in), &[], &[OsString::from("origin"), OsString::from("b0")]).expect("probes");
    assert_eq!(updates.len(), 2, "one update per destination: {updates:?}");
    let held = intervals(&log);
    assert_eq!(held.len(), 2, "{held:?}");
    assert!(all_overlap(&held), "the two confirmations were held at one moment: {held:?}");
    remove(root.as_path());
}

/// The updates of one push are scoped with their queries held at the same moment,
/// and the scopes keep the order of the updates.
#[test]
fn the_updates_of_a_push_are_scoped_side_by_side() {
    let branches: usize = 3;
    let (root, repo) = repository_with("mp-parallel-updates", 1, branches);
    let log: PathBuf = root.join("rev-list.log");
    let stand_in: PathBuf = timing_git(&root, "rev-list", &log);
    let names: Vec<OsString> = std::iter::once(OsString::from("origin")).chain((0..branches).map(|index: usize| { return OsString::from(format!("b{index}")); })).collect();
    let git_stand_in: PushGit = git_over(&repo, &stand_in);
    let updates = probe_manual_push_updates_on(&LARGE_HOST, &git_stand_in, &[], &names).expect("probes");
    assert_eq!(updates.len(), branches);
    let scopes = scopes_of_updates_on(&LARGE_HOST, &git_stand_in, &updates).expect("scopes");
    assert_eq!(scopes.len(), branches);
    for (update, scope) in updates.iter().zip(&scopes) {
        let tip: String = update.local_oid.clone().expect("a local object");
        assert!(matches!(scope, UpdateScope::Commits(commits) if commits == &vec![tip.clone()]), "{scope:?} for {tip}");
    }
    let held = intervals(&log);
    assert_eq!(held.len(), branches, "{held:?}");
    assert!(all_overlap(&held), "the scopes were read at one moment: {held:?}");
    remove(root.as_path());
}

/// An update that cannot be scoped decides the error,
/// and the others do not turn it into a success.
#[test]
fn an_update_that_cannot_be_scoped_fails_the_push_scope() {
    let (root, repo) = repository_with("mp-parallel-failure", 1, 2);
    let git_plain: PushGit = git_over(&repo, Path::new("/usr/bin/git"));
    let mut updates = probe_manual_push_updates_on(&LARGE_HOST, &git_plain, &[], &[OsString::from("origin"), OsString::from("b0"), OsString::from("b1")]).expect("probes");
    updates[0].local_oid = Some(String::from("0000000000000000000000000000000000000001"));
    let failure = scopes_of_updates_on(&LARGE_HOST, &git_plain, &updates).expect_err("the first update names no object");
    assert!(!failure.message.is_empty());
    remove(root.as_path());
}

/// More updates than the SPEC allows local queries at once are read at most that many at a time,
/// however large the host.
#[test]
fn no_more_than_the_specified_number_of_updates_are_read_at_once() {
    let branches: usize = MOST_CONCURRENT_LOCAL_QUERIES + 2;
    let (root, repo) = repository_with("mp-parallel-bound", 1, branches);
    let log: PathBuf = root.join("rev-list.log");
    let stand_in: PathBuf = timing_git(&root, "rev-list", &log);
    let names: Vec<OsString> = std::iter::once(OsString::from("origin")).chain((0..branches).map(|index: usize| { return OsString::from(format!("b{index}")); })).collect();
    let git_stand_in: PushGit = git_over(&repo, &stand_in);
    let updates = probe_manual_push_updates_on(&LARGE_HOST, &git_stand_in, &[], &names).expect("probes");
    scopes_of_updates_on(&LARGE_HOST, &git_stand_in, &updates).expect("scopes");
    let held = intervals(&log);
    assert_eq!(held.len(), branches, "{held:?}");
    assert_eq!(most_at_once(&held), MOST_CONCURRENT_LOCAL_QUERIES, "{held:?}");
    remove(root.as_path());
}

/// Local queries run as many at once as the measured gain allows,
/// and no more than the remote queries unless measured to gain.
#[test]
fn local_queries_run_as_many_at_once_as_the_measured_gain_allows() {
    assert_eq!(crate::manual_push_probe::MOST_CONCURRENT_LOCAL_QUERIES, crate::manual_push_probe::LOCAL_QUERY_GAIN_STOPS_AT);
}
