//! What:
//!  Commit services whose own runtime build outlasts `BUILD_DEADLINE`,
//! and the person's commit through them.
//! Why:
//!  The gate of 2026-11-11T10:29-05:00 failed
//! `fault_trace::a_service_killed_at_each_ftruncate_leaves_a_recoverable_repository`
//! because one of its services logged `ready` and then,
//! 1.76 s later,
//! `cannot start the acceptor of connections: the runtime that runs it has no worker thread`:
//! a build that missed its one-second deadline stood in for the runtime,
//! and the service ended at the deadline itself instead of waiting for the build that is kept for it.
//! The commit of that case failed with the service.
//! The builds here are made slower than the deadline on purpose (`SLOW_BUILD_VARIABLE`),
//! so these controls reach the state the gate reached by load,
//! one of them past the six seconds a service used to wait before it ended.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const service = await serve({ buildDelayMillis: 3000 }); expect(await commit('mine')).toBe(0);
//! ```

use super::commit::{git_text, remove_settled, run_commit};
use super::support::{Fixture, fixture, give_service_environment, service_log, start_fixture_service};
use super::service_landing_selection::base_repository;
use git_policy_cli::lazy_runtime::SLOW_BUILD_VARIABLE;
use std::path::PathBuf;

/// The pause the build of the service's runtime is given:
/// three seconds,
/// three times `BUILD_DEADLINE`,
/// so the deadline passes with the build still asleep and the build arrives inside the patience a service gives it.
const THE_BUILD_OUTLASTS_ITS_DEADLINE_BY_MILLISECONDS: &str = "3000";

/// The pause the build of the service's runtime is given in the control below:
/// seven seconds,
/// a second past the six seconds at which a service used to end
/// (its one-second `BUILD_DEADLINE` and a patience of five deadlines),
/// and three seconds inside the ten seconds a client that cannot ask a service manager waits for the service's greeting.
const THE_BUILD_OUTLASTS_THE_OLD_PATIENCE_BY_MILLISECONDS: &str = "7000";

/// The commit lands through a service whose runtime build missed its deadline,
/// and the service says which way its build went.
#[test]
fn a_service_whose_runtime_build_outlasted_its_deadline_still_lands_the_commit() {
    let fixture: Fixture = fixture("service-late-runtime-build");
    let repo: PathBuf = base_repository(&fixture);
    give_service_environment(&fixture, SLOW_BUILD_VARIABLE, THE_BUILD_OUTLASTS_ITS_DEADLINE_BY_MILLISECONDS);
    let service: super::support::FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    // The service serves from the build that arrived late, so the person's commit lands and nothing else is needed.
    std::fs::write(repo.join("a.txt"), "mine\n").expect("a.txt");
    let served = run_commit(&fixture, &repo, &["commit", "-m", "mine", "--", "a.txt"]);
    assert_eq!(served.code, Some(0), "the commit through the late service lands: {}", String::from_utf8_lossy(&served.stderr));
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s"]).trim(), "mine", "the commit is on the branch");
    let said: String = service_log(fixture.root.as_path());
    assert!(said.contains("current-thread stand-in") && said.contains("outlasted"), "the service names its stand-in and its cause: {said}");
    assert!(!said.contains("cannot start the acceptor of connections"), "the service did not end for want of a worker: {said}");
    remove_settled(&fixture);
}

/// What:
///  A commit lands through a service whose runtime build arrives after the patience a service used to give it,
///  and the service says what it waits for while it waits.
/// Why:
///   A build that takes more than six seconds is an ordinary event on a machine whose load average is over a hundred,
/// and ending the service there loses a commit that a real Git would have made;
/// the waiting line is what a person reads in the service's log while the commit waits for its greeting.
#[test]
fn a_runtime_that_arrives_after_the_old_patience_still_lands_the_commit() {
    let fixture: Fixture = fixture("service-slow-runtime-build");
    let repo: PathBuf = base_repository(&fixture);
    give_service_environment(&fixture, SLOW_BUILD_VARIABLE, THE_BUILD_OUTLASTS_THE_OLD_PATIENCE_BY_MILLISECONDS);
    let service: super::support::FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    std::fs::write(repo.join("a.txt"), "mine\n").expect("a.txt");
    let served = run_commit(&fixture, &repo, &["commit", "-m", "mine", "--", "a.txt"]);
    let said: String = service_log(fixture.root.as_path());
    assert_eq!(served.code, Some(0), "the commit through the slow service lands: {}\n{said}", String::from_utf8_lossy(&served.stderr));
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s"]).trim(), "mine", "the commit is on the branch");
    assert!(said.contains("still being built"), "the service says what it waits for while it waits: {said}");
    assert!(said.contains("current-thread stand-in") && said.contains("outlasted"), "the service names its stand-in and its cause: {said}");
    assert!(!said.contains("cannot start the acceptor of connections"), "the service did not end for want of a worker: {said}");
    remove_settled(&fixture);
}
