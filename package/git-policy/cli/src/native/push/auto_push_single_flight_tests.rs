//! Controls for the service's Git queries against real disposable repositories and a Git that never answers.

use super::*;
use crate::test_support::{fixture, git, remove};
use std::path::Path;

/// Argument list from string literals.
fn argv(parts: &[&str]) -> Vec<OsString> {
    return parts.iter().map(OsString::from).collect();
}

/// Environment of the fixture Git,
/// matching the test support fixtures.
fn fixture_environment() -> Vec<(OsString, OsString)> {
    let mut environment: Vec<(OsString, OsString)> = [
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
    // The per-user scratch directory of the child runners is made under a home beside the fixtures.
    environment.extend(crate::test_support::scratch_environment());
    return environment;
}

/// Coordinator Git for a fixture repository.
fn push_git(root: &Path) -> PushGit {
    return PushGit {
        real_git: PathBuf::from("/usr/bin/git"),
        cwd: root.to_path_buf(),
        environment: fixture_environment(),
        lfs: false,
    };
}

/// A new empty fixture repository on `main`.
fn repository(name: &str) -> PathBuf {
    let root = fixture(name);
    git(&root, &["init", "--quiet", "--initial-branch=main"]);
    commit(&root, "base");
    return root;
}

/// Land an empty commit with real Git and return its OID.
fn commit(root: &Path, message: &str) -> String {
    let message_argument = format!("--message={message}");
    git(
        root,
        &[
            "commit",
            "--quiet",
            "--allow-empty",
            message_argument.as_str(),
        ],
    );
    return head_of(root, "HEAD");
}

/// The OID a revision names in a fixture.
fn head_of(root: &Path, revision: &str) -> String {
    return String::from_utf8_lossy(&git(root, &["rev-parse", revision]).stdout)
        .trim()
        .to_owned();
}

/// A failed Git query names the arguments it ran in its error text.
#[test]
fn a_failed_query_names_its_arguments() {
    let root = repository("sf-query-failure");
    let failure = git_output(
        &push_git(&root),
        &argv(&["rev-parse", "--verify", "no-such-ref^{commit}"]),
    )
    .expect_err("an unknown ref fails");
    match &failure {
        SingleFlightError::Git {
            arguments, code, ..
        } => {
            assert_eq!(arguments, "rev-parse --verify no-such-ref^{commit}");
            assert_eq!(*code, Some(128));
        }
        other => panic!("expected a Git failure, got {other}"),
    }
    assert!(
        failure
            .to_string()
            .starts_with("git rev-parse --verify no-such-ref^{commit} exited with Some(128): "),
        "{failure}"
    );
    remove(&root);
}

/// A Git query that stalls is stopped at the query deadline and reported as a timeout,
///  so a stuck query cannot hold the commit.
#[test]
fn a_stalled_query_is_reported_instead_of_waited_for() {
    use std::os::unix::fs::PermissionsExt;
    let root = fixture("sf-stalled-query");
    let fake = root.join("fake-git");
    std::fs::write(&fake, "#!/bin/sh\nexec sleep 8\n").expect("fake git");
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).expect("mode");
    let mut git = push_git(&root);
    git.real_git = fake;
    git.environment.push((
        OsString::from(crate::bounded_push::PUSH_DEADLINE_VARIABLE),
        OsString::from("1"),
    ));
    let failure = git_output(&git, &argv(&["rev-parse", "HEAD"])).expect_err("stalls");
    assert!(
        matches!(&failure, SingleFlightError::TimedOut { arguments, seconds }
            if arguments == "rev-parse HEAD" && *seconds == 1),
        "{failure}"
    );
    remove(&root);
}

/// What:
///  The awaited query answers what the synchronous one answers,
/// for the in-process read and for a real Git child.
/// Why:
///   The landing round's push asks through the awaited face,
/// and a difference between the faces would be a push that
///  reads a different repository from its own.
#[test]
fn the_awaited_query_answers_what_the_synchronous_query_answers() {
    let root = repository("sf-query-awaiting-same-answer");
    let git = push_git(&root);
    for arguments in [
        vec!["rev-parse", "--abbrev-ref", "--symbolic-full-name", "main@{upstream}"],
        vec!["rev-parse", "HEAD"],
    ] {
        let asked: Vec<OsString> = argv(arguments.as_slice());
        let synchronous = git_output(&git, &asked);
        let (awaited, _ticks): (Result<String, SingleFlightError>, u32) =
            crate::ticker_beside_a_wait::with_a_ticker_beside(git_output_awaiting(&git, &asked));
        assert_eq!(format!("{awaited:?}"), format!("{synchronous:?}"), "{arguments:?}");
    }
    remove(&root);
}

/// What:
///  A failed awaited query names its arguments,
/// as the synchronous one does.
/// Why:
///   The words of a failure are what the person reads when a landing's push could not ask its question.
#[test]
fn a_failed_awaited_query_names_its_arguments() {
    let root = repository("sf-query-awaiting-failure");
    let (failure, _ticks): (Result<String, SingleFlightError>, u32) =
        crate::ticker_beside_a_wait::with_a_ticker_beside(git_output_awaiting(
            &push_git(&root),
            &argv(&["rev-parse", "--verify", "no-such-ref^{commit}"]),
        ));
    let failed = failure.expect_err("an unknown ref fails");
    match &failed {
        SingleFlightError::Git {
            arguments, code, ..
        } => {
            assert_eq!(arguments, "rev-parse --verify no-such-ref^{commit}");
            assert_eq!(*code, Some(128));
        }
        other => panic!("expected a Git failure, got {other}"),
    }
    remove(&root);
}

/// What:
///  A query whose Git never answers is stopped at its window on the awaited face too,
/// and a task beside it keeps running while the window passes.
/// Why:
///   A wait that held its thread would stop every other directory of the round for the
///  window,
/// and one that never ended would stop them for good.
#[test]
fn a_stalled_awaited_query_is_reported_and_lets_other_tasks_run() {
    use std::os::unix::fs::PermissionsExt;
    let root = fixture("sf-query-awaiting-stalled");
    let fake = root.join("fake-git");
    std::fs::write(&fake, "#!/bin/sh\nexec sleep 8\n").expect("fake git");
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).expect("mode");
    let mut git = push_git(&root);
    git.real_git = fake;
    git.environment.push((
        OsString::from(crate::bounded_push::PUSH_DEADLINE_VARIABLE),
        OsString::from("1"),
    ));
    let (failure, ticks): (Result<String, SingleFlightError>, u32) =
        crate::ticker_beside_a_wait::with_a_ticker_beside(git_output_awaiting(&git, &argv(&["rev-parse", "HEAD"])));
    let stalled = failure.expect_err("stalls");
    assert!(
        matches!(&stalled, SingleFlightError::TimedOut { arguments, seconds }
            if arguments == "rev-parse HEAD" && *seconds == 1),
        "{stalled}"
    );
    assert!(ticks > 0, "the wait for the query held the thread: the task beside it never ran");
    remove(&root);
}
