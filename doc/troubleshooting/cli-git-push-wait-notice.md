# Cli-git 0.1.4 announces a push wait before its outcome is known

## Symptom

A successful commit can include this advisory message while another process owns its branch's push:

```text
cli-git: waiting for process 762795 to finish pushing refs/heads/main; giving up after 665 seconds (CLI_GIT_PUSH_LOCK_WAIT_SECONDS changes this). The commit is already saved.
```

On 2026-10-15,
the agent initially described this message as a completed timeout.
That interpretation was wrong.
The message names a future wait limit;
it does not establish that the limit expired or that the push failed.

## Root cause of the misreading

The advisory originates in `package/git-policy/cli/src/native/auto_push.rs:182`:

```rust
// package/git-policy/cli/src/native/auto_push.rs:187
return format!(
    "cli-git: waiting for {holder} to finish pushing {branch_ref}; giving up after {} seconds \
     ({LOCK_WAIT_VARIABLE} changes this). The commit is already saved.",
    lock_wait_limit(&git.environment).as_secs()
);
```

`auto_push.rs:319` installs it as the coordinator's wait callback:

```rust
// package/git-policy/cli/src/native/auto_push.rs:319
let mut on_wait = || {
    let line = waiting_line(git, &branch_ref);
    debug(TAG, &line);
    // Terminal output error: the notice is advisory; a failed write to stderr must not stop the wait.
    let _written = writeln!(err, "{line}");
};
run_single_flight_push(git, &branch_ref, landed_oid, &mut push, &mut on_wait)
```

A coordination failure has separate reporting at `auto_push.rs:331`:

```rust
// package/git-policy/cli/src/native/auto_push.rs:331
Err(error) => {
    debug(TAG, &format!("auto-push coordination failed: {error}"));
    let _written = writeln!(
        err,
        "cli-git: auto-push of {landed_oid} could not run: {error}"
    );
    let _noted = writeln!(err, "{PUSH_FAILED_NOTE}");
```

The source's timeout test explicitly distinguishes the initial notice from the eventual failure:

```rust
// package/git-policy/cli/src/native/auto_push_tests.rs:471
assert_eq!(result.outcome, AutoPushOutcome::Failed);
assert!(shown.contains("gave up after waiting 1 seconds"), "{shown}");
assert!(shown.contains(PUSH_FAILED_NOTE), "{shown}");
```

This source test was inspected,
not rerun during the incident.
There is no evidence of a timeout in the observed commit output.

## Verification

The active wrapper resolved to the installed `cargo-monochromatic-git-policy-cli/0.1.4` executable.
Its SHA-256 was `0072cb8ac4e0748c2b4fb56d9e84d951363c180746e719bccdd1c91ca352a597`.
The repository source was inspected at `2255008ede69d4399d0e7859c2124de49cf4764b`.
The forwarded Git version was `2.56.0`.

These read-only commands verified the actual remote and landed commits:

```sh
# Run from the repository root; object IDs identify this recorded incident.
git branch --show-current
git rev-parse HEAD
git ls-remote origin refs/heads/main
git merge-base --is-ancestor 776f3486be55909811ecfe76c4ea2ecd7ae046cc 2255008ede69d4399d0e7859c2124de49cf4764b
git merge-base --is-ancestor dbb98f434861b369f1f703f2a036182a9a39cf71 2255008ede69d4399d0e7859c2124de49cf4764b
```

The branch was `main`.
Local `HEAD` and the remote `refs/heads/main` both named
`2255008ede69d4399d0e7859c2124de49cf4764b`.
Both ancestry checks exited zero.
The commits were already backed up;
no push was retried.

Observed successful patterns:

- A wait notice accompanied a completed commit command.
- The remote tip contained both landed preparation commits.
- The wrapper replayed the second prepared commit after a concurrent landing,
  producing final commit `776f3486be` from prepared commit `5d90f18fab`.

Failure patterns distinguished by source:

- An actual exhausted wait additionally reports `gave up after waiting`.
- Coordination failure reports `auto-push of ... could not run` and the saved-local-commit failure note.

The failure patterns were not observed in this incident.

## Verified handling

Read the terminal outcome,
query the remote ref,
and check whether it contains the landed commits before retrying a push or changing coordination state.
The commands in Verification perform these reads without changing the worktree,
branch,
or remote.
They establish a point-in-time result;
concurrent writers can move the branch after the observation.

No tool patch or configuration workaround was needed.

## What does not work

- Treating `giving up after` as proof that time already elapsed misreads the advisory.
- `git rev-list --left-right --count origin/main...HEAD` reported `0 20` here,
  but the direct remote query already matched local `HEAD`.
  A local tracking ref alone did not prove an unpushed commit.
- The named process had exited by inspection.
  Its absence did not establish whether the push succeeded or failed.

## Upstream filing decision

No external upstream artifact is needed.
The diagnostic is owned by this repository,
and the investigation corrected an interpretation without finding a tool defect.
The `.out-of-scope/` inventory was checked;
no listed exemption determined this result.

1.  Fault: no tool failure was established.
2.  Fixability: no corrective tool change is required by the evidence.
3.  Supported use: concurrent push waiting and eventual failure reporting have distinct source paths and tests.
4.  Contribution policy: this is repository-owned code under the repository's existing agent instructions.
5.  Maintainer disposition: no request for a tool change follows from this incident.
6.  Prototype: not applicable because the observed operation succeeded and the fault premise failed.

There is nothing to file or add to an external tracker.

## Concurrent main commits on 2026-10-17

The same advisory named processes `174748` and `173660` during concurrent scoped commits.
The owner of commit session `48904` retained terminal exit `0` for prepared commit `ce54`,
replayed as `1d27d625583d142722d8cd4df24db175f6f9a2dc`.
Session `57030` had saved `8dda808f086d26b6b1d0d9d75fb5c0e0bcc52bc8`,
but its terminal outcome was not available to this investigation.
The waiting notice alone did not establish a failure for either command.

Direct remote discovery returned `787f21579c67bd22869e5621f592111ad03d9587` for `refs/heads/main`.
Both affected landed commits were ancestors of that exact remote OID:

```sh
# Run from the repository root; both ancestry commands exited zero in this incident.
git ls-remote origin refs/heads/main
git merge-base --is-ancestor 1d27d625583d142722d8cd4df24db175f6f9a2dc 787f21579c67bd22869e5621f592111ad03d9587
git merge-base --is-ancestor 8dda808f086d26b6b1d0d9d75fb5c0e0bcc52bc8 787f21579c67bd22869e5621f592111ad03d9587
```

The contemporaneous `.git/cli-git/push/refs%2Fheads%2Fmain.last-pushed.json` independently recorded that tip,
`outcome: pushed`,
and `exitCode: 0`.
Neither named process existed when inspected with `ps --pid 174748,173660`.
The main push lock's `owner.json` appeared in the initial file listing,
then vanished before its contents could be read.
The investigation did not capture its owner identity or remove any lock,
interrupt a push,
retry publication,
or change a ref.
These snapshots establish publication and changing coordination state,
not why either original holder exited.

The active wrapper still resolved to `cargo-monochromatic-git-policy-cli/0.1.4`;
forwarded Git reported `2.56.0`.
Source inspected at `787f21579c67bd22869e5621f592111ad03d9587` retained the advisory and failure paths quoted here.
`package/git-policy/cli/src/native/bounded_push.rs:139` derives the default wait limit from
the `600` second push deadline,
`5` second stop grace,
and `60` second margin.
Thus `665 seconds` is a configured future limit,
not an elapsed-time measurement.
`package/git-policy/cli/src/native/owner_lock.rs:146` separately emits `gave up after waiting` on exhaustion.

Current-repository issue searches for `cli-git push lock`,
`CLI_GIT_PUSH_LOCK_WAIT_SECONDS`,
`"cli-git" "waiting"`,
and `"giving up after"` found no distinct matching defect to update.
No new issue or production patch was warranted by this recurrence.
The earlier diagnostic interpretation and verified handling remain applicable.
