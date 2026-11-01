//! Controls for single-flight push coordination against real disposable repositories.
//! The TypeScript corpus is `src/auto-push-single-flight.unit.test.ts`;
//! every case there has a test below with the same behavior, and the injected push counts invocations.

use super::*;
use crate::bounded_push::LOCK_WAIT_VARIABLE;
use crate::owner_lock::{SilentWait, acquire_owner_lock as acquire_lock};
use crate::owner_lock_record::{
    OWNER_LOCK_RECORD_FILENAME, OwnerLockRecord, encode_owner_lock_record,
};
use crate::test_support::{fixture, git, remove};
use std::sync::mpsc;
use std::time::Duration;

/// The branch every case pushes.
const BRANCH: &str = "refs/heads/main";

/// Seconds a fixture coordinator waits for a lock before it reports the lock as stuck:
///  a regression that waits behind a lock the test holds fails here instead of hanging the run.
const FIXTURE_LOCK_WAIT_SECONDS: &str = "20";

/// Environment of the fixture Git, matching the test support fixtures.
fn fixture_environment() -> Vec<(OsString, OsString)> {
    let mut environment: Vec<(OsString, OsString)> = [
        (LOCK_WAIT_VARIABLE, FIXTURE_LOCK_WAIT_SECONDS),
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
        lfs_pre_push: None,
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

/// The successful attempt every counting push reports.
fn success() -> PushAttempt {
    return PushAttempt {
        exit_code: 0,
        output: "remote: ok\n".to_owned(),
    };
}

/// The coordination paths of the fixture branch.
fn paths_of(root: &Path) -> PushCoordinationPaths {
    return resolve_push_coordination_paths(&push_git(root), BRANCH).expect("paths");
}

/// Run the coordinator with a push closure and no wait observer.
fn coordinate(
    root: &Path,
    landed: &str,
    push: &mut dyn FnMut() -> Result<PushAttempt, SingleFlightError>,
) -> Result<SingleFlightOutcome, SingleFlightError> {
    return run_single_flight_push(&push_git(root), BRANCH, landed, push, &mut || {});
}

/// A record naming `tip` with the given outcome and owner.
fn record(
    tip: &str,
    outcome: LastPushedOutcome,
    owner_token: &str,
    output: &str,
) -> LastPushedRecord {
    return LastPushedRecord {
        tip: tip.to_owned(),
        outcome,
        owner_token: owner_token.to_owned(),
        owner_pid: 1,
        exit_code: if outcome == LastPushedOutcome::Pushed {
            0
        } else {
            1
        },
        output: output.to_owned(),
    };
}

/// Read the record of the fixture branch, which must exist.
fn read_record(root: &Path) -> LastPushedRecord {
    return match read_last_pushed_record(&paths_of(root).record_path).expect("reads") {
        LastPushedRead::Record(record) => record,
        LastPushedRead::Absent => panic!("record absent"),
    };
}

/// Entries of the coordination directory.
fn entries(root: &Path) -> Vec<String> {
    let directory = paths_of(root)
        .record_path
        .parent()
        .expect("directory")
        .to_path_buf();
    let mut names: Vec<String> = std::fs::read_dir(directory)
        .expect("list")
        .map(|entry| {
            return entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned();
        })
        .collect();
    names.sort();
    return names;
}

/// Plant a lock directory whose owner exited, as the incumbent's dead-pusher case does.
fn plant_dead_lock(lock_directory: &Path) {
    let mut child = std::process::Command::new("/bin/true")
        .spawn()
        .expect("spawn");
    let pid = i64::from(child.id());
    child.wait().expect("wait");
    std::fs::create_dir(lock_directory).expect("planted lock");
    let planted = OwnerLockRecord {
        token: "dead".to_owned(),
        owner_pid: pid,
        owner_birth_identity: "dead-identity".to_owned(),
        transaction_id: None,
        owner_pid_namespace: None,
    };
    std::fs::write(
        lock_directory.join(OWNER_LOCK_RECORD_FILENAME),
        encode_owner_lock_record(&planted),
    )
    .expect("planted record");
}

/// The first push of a fresh repository records the tip it pushed.
#[test]
fn pushes_the_branch_tip_itself_when_no_push_was_recorded() {
    let root = repository("sf-first-push");
    let landed = commit(&root, "one");
    let mut tips: Vec<String> = Vec::new();
    let outcome = coordinate(&root, &landed, &mut || {
        tips.push(head_of(&root, BRANCH));
        return Ok(success());
    })
    .expect("coordinates");
    assert_eq!(
        outcome,
        SingleFlightOutcome {
            kind: SingleFlightKind::Pushed,
            tip: landed.clone(),
            exit_code: 0,
            output: "remote: ok\n".to_owned(),
            covered: true,
        }
    );
    assert_eq!(tips, vec![landed.clone()]);
    let written = read_record(&root);
    assert_eq!(written.tip, landed);
    assert_eq!(written.outcome, LastPushedOutcome::Pushed);
    assert_eq!(written.output, "");
    assert_eq!(written.owner_pid, i64::from(std::process::id()));
    assert_eq!(
        entries(&root),
        vec!["refs%2Fheads%2Fmain.last-pushed.json".to_owned()]
    );
    remove(&root);
}

/// A recorded successful push that covers the commit joins without taking the lock.
#[test]
fn joins_a_recorded_successful_push_covering_the_commit_without_taking_the_lock() {
    let root = repository("sf-fast-join");
    let older = commit(&root, "older");
    let tip = commit(&root, "tip");
    let paths = paths_of(&root);
    write_last_pushed_record(
        &paths.record_path,
        &record(&tip, LastPushedOutcome::Pushed, "earlier", ""),
    )
    .expect("write");
    let held =
        acquire_lock(&paths.lock_directory, DEFAULT_POLL_DELAY, &mut SilentWait).expect("held");
    let mut pushes = 0;
    let outcome = coordinate(&root, &older, &mut || {
        pushes += 1;
        return Ok(success());
    })
    .expect("joins");
    assert_eq!(
        outcome,
        SingleFlightOutcome {
            kind: SingleFlightKind::JoinedPushed,
            tip,
            exit_code: 0,
            output: String::new(),
            covered: true,
        }
    );
    assert_eq!(pushes, 0);
    drop(held);
    remove(&root);
}

/// A recorded success that does not cover the commit leads to a push of its own.
#[test]
fn pushes_itself_when_the_recorded_successful_push_does_not_cover_the_commit() {
    let root = repository("sf-not-covering");
    let pushed = commit(&root, "pushed");
    let landed = commit(&root, "newer");
    write_last_pushed_record(
        &paths_of(&root).record_path,
        &record(&pushed, LastPushedOutcome::Pushed, "earlier", ""),
    )
    .expect("write");
    let mut tips: Vec<String> = Vec::new();
    let outcome = coordinate(&root, &landed, &mut || {
        tips.push(head_of(&root, BRANCH));
        return Ok(success());
    })
    .expect("coordinates");
    assert_eq!(outcome.kind, SingleFlightKind::Pushed);
    assert_eq!(tips, vec![landed]);
    remove(&root);
}

/// A failed push recorded before the commit waited is not reported as this commit's outcome.
#[test]
fn retries_a_failed_push_recorded_before_it_waited() {
    let root = repository("sf-stale-failure");
    let landed = commit(&root, "one");
    write_last_pushed_record(
        &paths_of(&root).record_path,
        &record(&landed, LastPushedOutcome::Failed, "earlier", "old failure"),
    )
    .expect("write");
    let mut pushes = 0;
    let outcome = coordinate(&root, &landed, &mut || {
        pushes += 1;
        return Ok(success());
    })
    .expect("coordinates");
    assert_eq!(outcome.kind, SingleFlightKind::Pushed);
    assert_eq!(pushes, 1);
    remove(&root);
}

/// Run a coordinator behind a lock the test holds: returns once the coordinator reports the wait.
///
/// The test releases the lock after it writes whatever record the case needs,
/// so the coordinator then reads that record, as a live pusher would leave it.
fn coordinate_behind_lock<F>(root: &Path, landed: &str, prepare: F) -> SingleFlightOutcome
where
    F: FnOnce(&Path, &str),
{
    let paths = paths_of(root);
    let held =
        acquire_lock(&paths.lock_directory, DEFAULT_POLL_DELAY, &mut SilentWait).expect("held");
    let (wait_tx, wait_rx) = mpsc::channel::<()>();
    let git_context = push_git(root);
    let landed_oid = landed.to_owned();
    let outcome = std::thread::scope(|scope| {
        let handle = scope.spawn(|| {
            let mut pushes = 0;
            return run_single_flight_push(
                &git_context,
                BRANCH,
                &landed_oid,
                &mut || {
                    pushes += 1;
                    return Ok(success());
                },
                &mut || {
                    let _sent = wait_tx.send(());
                },
            );
        });
        if wait_rx
            .recv_timeout(Duration::from_secs(10))
            .is_err()
        {
            // Releasing the lock lets the coordinator finish so the scope can end before the failure is raised.
            drop(held);
            panic!("the coordinator never reported waiting for the held lock");
        }
        prepare(&paths.record_path, held.token());
        drop(held);
        return handle
            .join()
            .expect("coordinator thread")
            .expect("coordinates");
    });
    return outcome;
}

/// A live pusher's successful push that covers the commit is joined, not repeated.
#[test]
fn waits_for_a_live_pusher_and_joins_its_successful_push_when_it_covers_the_commit() {
    let root = repository("sf-join-success");
    let landed = commit(&root, "one");
    let outcome = coordinate_behind_lock(&root, &landed, |record_path, token| {
        write_last_pushed_record(
            record_path,
            &record(&landed, LastPushedOutcome::Pushed, token, ""),
        )
        .expect("write");
    });
    assert_eq!(
        outcome,
        SingleFlightOutcome {
            kind: SingleFlightKind::JoinedPushed,
            tip: landed.clone(),
            exit_code: 0,
            output: String::new(),
            covered: true,
        }
    );
    remove(&root);
}

/// A live pusher's failed push that covers the commit is reported with its output.
#[test]
fn waits_for_a_live_pusher_and_reports_its_failed_push_when_it_covered_the_commit() {
    let root = repository("sf-join-failure");
    let landed = commit(&root, "one");
    let outcome = coordinate_behind_lock(&root, &landed, |record_path, token| {
        write_last_pushed_record(
            record_path,
            &LastPushedRecord {
                exit_code: 1,
                ..record(
                    &landed,
                    LastPushedOutcome::Failed,
                    token,
                    "pre-push hook declined\n",
                )
            },
        )
        .expect("write");
    });
    assert_eq!(
        outcome,
        SingleFlightOutcome {
            kind: SingleFlightKind::JoinedFailed,
            tip: landed.clone(),
            exit_code: 1,
            output: "pre-push hook declined\n".to_owned(),
            covered: true,
        }
    );
    remove(&root);
}

/// A live pusher's failed push that does not cover the commit leaves the commit to push itself.
#[test]
fn pushes_itself_after_a_live_pusher_failed_a_push_that_did_not_cover_the_commit() {
    let root = repository("sf-failure-not-covering");
    let failed_tip = commit(&root, "failed");
    let landed = commit(&root, "newer");
    let outcome = coordinate_behind_lock(&root, &landed, |record_path, token| {
        write_last_pushed_record(
            record_path,
            &record(&failed_tip, LastPushedOutcome::Failed, token, "declined\n"),
        )
        .expect("write");
    });
    assert_eq!(outcome.kind, SingleFlightKind::Pushed);
    assert_eq!(outcome.tip, landed);
    remove(&root);
}

/// A lock holder that released without recording a push leaves the commit to push itself.
#[test]
fn pushes_itself_when_the_lock_holder_released_without_recording_a_push() {
    let root = repository("sf-holder-nothing");
    let landed = commit(&root, "one");
    let outcome = coordinate_behind_lock(&root, &landed, |_record_path, _token| {});
    assert_eq!(outcome.kind, SingleFlightKind::Pushed);
    assert_eq!(outcome.tip, landed);
    remove(&root);
}

/// The lock of a dead pusher is retired and the commit pushes itself.
#[test]
fn takes_over_the_push_lock_of_a_dead_pusher_and_pushes_itself() {
    let root = repository("sf-dead-pusher");
    let landed = commit(&root, "one");
    plant_dead_lock(&paths_of(&root).lock_directory);
    let mut pushes = 0;
    let outcome = coordinate(&root, &landed, &mut || {
        pushes += 1;
        return Ok(success());
    })
    .expect("coordinates");
    assert_eq!(outcome.kind, SingleFlightKind::Pushed);
    assert_eq!(pushes, 1);
    assert_eq!(
        entries(&root),
        vec!["refs%2Fheads%2Fmain.last-pushed.json".to_owned()]
    );
    remove(&root);
}

/// A failed push of this commit is recorded with its complete output and reported.
#[test]
fn records_and_reports_its_own_failed_push_with_the_complete_output() {
    let root = repository("sf-own-failure");
    let landed = commit(&root, "one");
    let outcome = coordinate(&root, &landed, &mut || {
        return Ok(PushAttempt {
            exit_code: 1,
            output: "error: failed to push some refs\n".to_owned(),
        });
    })
    .expect("coordinates");
    assert_eq!(
        outcome,
        SingleFlightOutcome {
            kind: SingleFlightKind::Failed,
            tip: landed.clone(),
            exit_code: 1,
            output: "error: failed to push some refs\n".to_owned(),
            covered: true,
        }
    );
    let written = read_record(&root);
    assert_eq!(written.outcome, LastPushedOutcome::Failed);
    assert_eq!(written.output, "error: failed to push some refs\n");
    assert_eq!(written.exit_code, 1);
    remove(&root);
}

/// A pushed tip that no longer contains the landed commit is reported as uncovered.
#[test]
fn reports_a_pushed_tip_that_no_longer_contains_the_landed_commit_as_uncovered() {
    let root = repository("sf-uncovered");
    let landed = commit(&root, "dropped");
    git(&root, &["reset", "--quiet", "--hard", "HEAD~1"]);
    let outcome = coordinate(&root, &landed, &mut || {
        return Ok(success());
    })
    .expect("coordinates");
    assert_eq!(outcome.kind, SingleFlightKind::Pushed);
    assert!(!outcome.covered);
    remove(&root);
}

/// Concurrent landings coalesce: the first push covers all five commits and the others join it.
#[test]
fn coalesces_concurrent_landings_into_fewer_pushes_than_commits() {
    let root = repository("sf-coalesce");
    let landed: Vec<String> = ["c1", "c2", "c3", "c4", "c5"]
        .iter()
        .map(|message| {
            return commit(&root, message);
        })
        .collect();
    let pushes = std::sync::atomic::AtomicUsize::new(0);
    let git_context = push_git(&root);
    let outcomes: Vec<SingleFlightOutcome> = std::thread::scope(|scope| {
        let handles: Vec<_> = landed
            .iter()
            .map(|oid| {
                let context = &git_context;
                let counter = &pushes;
                return scope.spawn(move || {
                    return run_single_flight_push(
                        context,
                        BRANCH,
                        oid,
                        &mut || {
                            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            std::thread::sleep(std::time::Duration::from_millis(200));
                            return Ok(success());
                        },
                        &mut || {},
                    )
                    .expect("coordinates");
                });
            })
            .collect();
        return handles
            .into_iter()
            .map(|handle| {
                return handle.join().expect("thread");
            })
            .collect();
    });
    assert_eq!(pushes.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert!(outcomes.iter().all(|outcome| {
        return outcome.covered
            && matches!(
                outcome.kind,
                SingleFlightKind::Pushed | SingleFlightKind::JoinedPushed
            );
    }));
    remove(&root);
}

/// Ancestry is answered for ancestors and equal tips, and a Git failure is an error.
#[test]
fn tip_contains_reports_ancestry_and_rethrows_other_git_failures() {
    let root = repository("sf-tip-contains");
    let older = commit(&root, "older");
    let newer = commit(&root, "newer");
    let context = push_git(&root);
    assert!(tip_contains(&context, &older, &newer).expect("ancestor"));
    assert!(tip_contains(&context, &newer, &newer).expect("equal"));
    assert!(!tip_contains(&context, &newer, &older).expect("not ancestor"));
    assert!(tip_contains(&context, &"f".repeat(40), &newer).is_err());
    remove(&root);
}

/// A linked worktree resolves its coordination files under the shared common Git directory.
#[test]
fn coordination_files_live_under_the_common_git_directory_shared_by_linked_worktrees() {
    let root = repository("sf-linked");
    commit(&root, "one");
    let linked = root.with_extension("linked");
    git(
        &root,
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "side",
            linked.to_str().expect("utf-8 path"),
        ],
    );
    let paths =
        resolve_push_coordination_paths(&push_git(&linked), "refs/heads/side").expect("paths");
    let common = root.join(".git").join("cli-git").join("push");
    assert_eq!(
        paths.lock_directory,
        common.join("refs%2Fheads%2Fside.lock")
    );
    assert_eq!(
        paths.record_path,
        common.join("refs%2Fheads%2Fside.last-pushed.json")
    );
    remove(&linked);
    remove(&root);
}

/// A lock held by a live process that never lets go ends the wait at the configured limit
///  with a refusal naming the lock,
///  without pushing and without touching the holder's lock.
#[test]
fn a_lock_held_past_the_wait_limit_is_reported_and_left_in_place() {
    let root = repository("sf-lock-timeout");
    let landed = commit(&root, "one");
    let paths = paths_of(&root);
    let held =
        acquire_lock(&paths.lock_directory, DEFAULT_POLL_DELAY, &mut SilentWait).expect("held");
    let mut git = push_git(&root);
    git.environment
        .push((OsString::from(LOCK_WAIT_VARIABLE), OsString::from("1")));
    let mut pushes = 0;
    let mut waits = 0;
    let started = std::time::Instant::now();
    let (result_tx, result_rx) = mpsc::channel::<()>();
    let (failure, still_held) = std::thread::scope(|scope| {
        let handle = scope.spawn(|| {
            let result = run_single_flight_push(
                &git,
                BRANCH,
                &landed,
                &mut || {
                    pushes += 1;
                    return Ok(success());
                },
                &mut || {
                    waits += 1;
                },
            );
            let _sent = result_tx.send(());
            return result;
        });
        let in_time = result_rx.recv_timeout(Duration::from_secs(10)).is_ok();
        if !in_time {
            // Releasing the lock lets a coordinator without a limit finish so the scope can end before the failure is raised.
            drop(held);
            let _finished = handle.join();
            panic!("the coordinator kept waiting past its wait limit");
        }
        let failure = handle
            .join()
            .expect("coordinator thread")
            .expect_err("the held lock refuses the coordinator");
        return (failure, held);
    });
    let elapsed = started.elapsed();
    assert!(elapsed >= Duration::from_secs(1), "{elapsed:?}");
    assert!(elapsed < Duration::from_secs(15), "{elapsed:?}");
    assert_eq!((pushes, waits), (0, 1));
    match &failure {
        SingleFlightError::Lock(OwnerLockError::WaitTimedOut {
            path, holder_pid, ..
        }) => {
            assert_eq!(path, &paths.lock_directory);
            assert_eq!(*holder_pid, Some(i64::from(std::process::id())));
        }
        other => panic!("expected a lock timeout, got {other}"),
    }
    assert!(
        failure.to_string().contains("gave up after waiting 1 seconds"),
        "{failure}"
    );
    assert_eq!(still_held.directory(), paths.lock_directory.as_path());
    assert!(paths.lock_directory.exists(), "the holder's lock stays");
    drop(still_held);
    remove(&root);
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
    let started = std::time::Instant::now();
    let failure = git_output(&git, &argv(&["rev-parse", "HEAD"])).expect_err("stalls");
    assert!(started.elapsed() < Duration::from_secs(7), "{:?}", started.elapsed());
    assert!(
        matches!(&failure, SingleFlightError::TimedOut { arguments, seconds }
            if arguments == "rev-parse HEAD" && *seconds == 1),
        "{failure}"
    );
    let ancestry = tip_contains(&git, "a", "b").expect_err("stalls too");
    assert!(matches!(ancestry, SingleFlightError::TimedOut { .. }), "{ancestry}");
    remove(&root);
}
