# The commit service answers a commit late while one round is busy, and the wrapper calls it silent

Date:
 2026-11-10.
Subject:
 a commit service that lands commits on time but reads their requests up to 32 s late,
 and answers `The commit service stopped answering` to a command whose commit landed a second later.
Owner's rules this bears on:
 "Commits should never fail" and
 "nothing should ever wait forever or indefinitely".

## Symptom

A commit of `refs/heads/main` in this repository printed:

```text
The commit service stopped answering; asking it again with the same commit, which cannot land twice.
```

and then,
on the same command:

```text
The commit service stopped answering while this commit waited, and the branch could not be read afterwards, so it is not known whether the commit was made.
```

The commit `7801a67ad0` was on the branch.
The service had landed it before the command gave up,
and answered six copies of the request in one burst afterwards.

The same service made no push,
no automatic maintenance and no shadow review of `refs/heads/main`
for 1 h 37 min (12:35:56 to 14:12:50),
while landings continued (12:50,
13:01,
13:02,
13:05,
13:06,
13:11).
`origin/main` fell 36 commits behind.

## What the journal proves

`journalctl --user -u 'monochromatic-git-policy-cli@f30a114aaabc977cf77309db0f77f1ee.service'`
(read only;
the service of this checkout was not touched):

- The landing of `7801a67ad0` took `read_ms=32019`:
  the request sat in the socket for 32 s before the service read it,
  and five re-sent copies of it were answered in the same instant,

  ```text
  12:50:28 answered request="c2b378d0-..." outcome=landed read_ms=32019 ... total_ms=35215
  12:50:28 answered request="c2b378d0-..." outcome=already read_ms=26873
  12:50:28 answered request="c2b378d0-..." outcome=already read_ms=37392
  12:50:28 answered request="c2b378d0-..." outcome=already read_ms=21728
  12:50:28 answered request="c2b378d0-..." outcome=already read_ms=16583
  12:50:28 answered request="c2b378d0-..." outcome=already read_ms=11443
  ```

  A client gives up after five silent windows of `HEARTBEAT_SILENCE`,
  5 s each
  (`service_protocol_messages_and_name_checks.rs`),
  so a service that stops reading its socket for half a minute is not distinguishable from a stopped one.
- The landing at 13:11 did it again,
  `read_ms=27706`.
- No `pushed`,
  `automatic maintenance ran` or `the review of ... is done` line appears between
  12:35:56 and 14:12:50,
  and the 14:12:50 push carried seven records at once (`items=7`),
  so the round that runs those steps was busy with something else for an hour and a half.

## What is inferred, not proven

The drain runs its work in one round,
and the trash of removed worktrees comes last in that round,
one blocking-pool job per entry
(`landing/service_work_drain_pending_claims.rs`).
The same service had logged,
at 10:24,
a trash it could not empty that named hundreds of thousands of entries under
`.claude/cli-git-trash/<snapshot>` of a cargo-heavy removed worktree,
and 15 trash records were claimed in its work directory at 14:24-14:40.
A trash of a worktree's `target/` directory is hundreds of thousands of removals,
so a round that starts one can hold every push,
review and upkeep behind it for that long,
and the jobs it queues share the blocking pool with the storage steps of the landings that arrive meanwhile.
That is consistent with both facts above,
and it is what this host's journals can say:
no line names a trash that finishes,
so the interval cannot be attributed to one with the records there are.

## What to do about it

The second limit below landed on
`fix/git-policy-cli-pushes-of-main-that-stalled-today`,
and the first is still open.
This is a defect on the commit's waiting path,
which the agent `commit-waiting-path-with-dozens-of-agents` owns.

- the socket's read and the heartbeats must not wait on the drain's work,
  so a round that is busy with a long removal cannot make a client call the service stopped.
  Still open.
- the pushes of a worktree must not wait for a round's trash work.
  Done by claiming the push on the round that answers a wake and every other
  kind on a lane of its own task
  (`landing/service_work_slow_lane.rs`),
  which `a_push_is_taken_while_the_service_is_inside_a_trash` asserts.
  The measurement of the day's late pushes, and what is still unbounded,
  is in
  `doc/planning/git-policy-cli-push-jams-2026-11-10.md`,
  "A round that serves a push holds no slow step".
  The trash itself is still unbounded: a worktree's `target/` still takes as long
  as it takes, on the lane rather than on the push.

Until then,
a commit that reports the service silent has landed more often than not:
`git log -1` is the check,
and the message the wrapper prints names that.

## Related

The held push such a command leaves is settled by the verdict its command left
(`landing/held_push_holder_and_verdict.rs`),
and the push record the service holds is released by the next command of the worktree.
