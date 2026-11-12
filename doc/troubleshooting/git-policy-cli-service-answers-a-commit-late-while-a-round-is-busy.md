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

Two limits follow from the evidence.
The heartbeat one landed on
`fix/git-policy-cli-service-reads-its-socket-while-a-round-works`,
the push-jam one on
`fix/git-policy-cli-pushes-of-main-that-stalled-today`,
and what neither of them bounds is the length of one trash.
This is a defect on the commit's waiting path,
which the agent `commit-waiting-path-with-dozens-of-agents` owns.

- the socket's read and the heartbeats must not wait on the drain's work,
  so a round that is busy with a long removal cannot make a client call the service stopped.
  Done: the beat is written before its note,
  a trash's entry removals take every place of the blocking pool but one (`places_a_round_may_take`),
  and a round starts housekeeping for no longer than `HOUSEKEEPING_OF_ONE_ROUND`
  ("Fixed, 2026-11-11" in this file).
- the pushes of a worktree must not wait for a round's trash work.
  Done by claiming the push on the round that answers a wake and every other
  kind on a lane of its own task
  (`landing/service_work_slow_lane.rs`),
  which `a_push_is_taken_while_the_service_is_inside_a_trash` asserts.
  The measurement of the day's late pushes,
  and what is still unbounded,
  is in
  `doc/planning/git-policy-cli-push-jams-2026-11-10.md`,
  "A round that serves a push holds no slow step".
  The trash itself is still unbounded:
  a worktree's `target/` still takes as long as it takes,
  on the lane rather than on the push.

Until then,
a commit that reports the service silent has landed more often than not:
`git log -1` is the check,
and the message the wrapper prints names that.

## Related

The held push such a command leaves is settled by the verdict its command left
(`landing/held_push_holder_and_verdict.rs`),
and the push record the service holds is released by the next command of the worktree.

## Fixed, 2026-11-11

`fix/git-policy-cli-service-reads-its-socket-while-a-round-works` changed the waiting path and the round's
housekeeping, and the cause it removed was narrower than the inference above:

- the beat asked the landing's board for its note **before** writing anything,
  and that read is a job of the blocking pool
  (`service/service_reply.rs`),
  while a trash's removals are jobs of the same pool,
  one per entry
  (`worktree/worktree_trash_entries.rs`).
  A full pool therefore left a waiting client with no beat,
  and the request's own check of which instance it belongs to
  (`service/service_server_connection_handling.rs`) waited for a place as well,
  which is what `read_ms` measured.
  The beat now goes first,
  and the note is given one beat's interval to answer (`HEARTBEAT_EVERY`,
  `service/service_protocol.rs`) before it is left out for the next beat.
- a trash's entry removals now take every place of that pool but one
  (`places_a_round_may_take`,
  `foundation/blocking_jobs_within_the_budget.rs`),
  so the steps that answer a client run beside a round.
- one round starts housekeeping for at most `HOUSEKEEPING_OF_ONE_ROUND` (30 s,
  `landing/housekeeping_of_one_round.rs`):
  what it does not reach goes back to `pending`,
  is named in one journal line,
  and is woken for after `LOOK_AGAIN_AFTER_DEFERRING` (5 s).
  An item a round has started is not interrupted
  (a removal stopped halfway would need a resume the trash does not keep),
  so the bound says what a round starts, never how long one step of it lasts.
  That is the limit that stays:
  a single trash of a worktree with a large `target/` directory still holds its round for as long as the removal takes,
  and what keeps a person's commit served meanwhile is the place the removals leave free and the beat.

The reproduction, on a wrapper built from the branch before the change,
in a throwaway repository with a trash of four entries of 60,000 files each:

```text
commit exit=1 wall=32.3 s "stopped answering"=6 "asking it again"=5
The commit service stopped answering; asking it again with the same commit, which cannot land twice.
...
answered request="8c3f113f-..." outcome=landed read_ms=29346 ...
```

After the change the same probe reports,
with the same trash of four entries of 60,000 files:

```text
the trash round took 10.4 s
commit exit=0 wall=3.0 s "stopped answering"=0 "asking it again"=0
read_ms=0 ... landing_ms=22 answer_ms=745
```

and with a trash of four entries of 150,000 files,
whose round ran for 51.4 s while the commit was made:

```text
the trash round took 51.4 s
commit exit=0 wall=45.8 s "stopped answering"=0 "asking it again"=0
read_ms=0 queued_ms=15 landing_ms=479
```

The request reaches the landing at once and the client is beaten until its answer,
so a round that is removing 600,000 files no longer reads as a stopped service.
What still waits in that run is the answer's own path and not the socket:
`answer_ms=21041`,
and the journal of the same run shows the cause,
the disk the removals are emptying:

```text
the background sync of 1 records of a landing took 9059 ms
the background sync of 6 records of a landing took 62465 ms
```

That is the wrapper's own storage writes competing with the removals on one btrfs,
which it did before this change as well (the run that reported `commit exit=1 wall=32.3 s` wrote its landed answer
19,558 ms after its landing, `answer_ms=19558 total_ms=49279`),
and it is a separate wait from the one this document is about.

The controls that keep this from coming back are
`service_server_connection_handling::tests::a_client_is_greeted_and_beaten_while_every_place_of_the_pool_is_held` (fails
after 5 s before the reply change),
`worktree_trash_entries::tests::a_trash_removal_leaves_one_place_for_a_step_that_answers_a_client` (fails after 10 s
before the place change) and
`service_work::tests::a_round_that_has_spent_its_housekeeping_leaves_the_rest_for_a_later_round`.

## The same sentence on 2026-11-11, printed on a landing and not on a trash

The release installed for that whole day was `0.6.1-alpha.10`, which does not carry the change above
(`git merge-base --is-ancestor 28f5f7b14f monochromatic-git-policy-cli-v0.6.1-alpha.10` fails).
One commit of that day printed the sentence again, and its records put the silence on a landing:

- the day's only timing record with `sends=2` is `refs/heads/main` at 18:05:32 local (`23:05:32Z`),
  with `submitMs=5453`, `answerWaitMs=5428`, `totalMs=6645`, outcome `landed`.
- the service's own line for request `3ef36edc-cc1c-48ea-9250-e005673b9c45`
  (`journalctl --user --identifier=monochromatic-git-policy-cli`) answers it with `read_ms=0`,
  `landing_ms=4737` (`index_update_ms=3447`, `ref_ms=1241`) and `answer_ms=8`:
  the request was read at once, the landing worked for 4.7 s, and the answer was written 8 ms after it ended.
- no beat reached the client during those 4.7 s, so what it spent (5.4 s) was the landing plus the silence it
  gave up on, and neither the read of its request nor the write of its answer was late.
- the only step between two beats in that release was the note, and the note was a job of the blocking pool
  awaited with no ceiling. That pool was occupied then: the landing's own record syncs were queued when it
  answered and logged 19.7 s later ("the background sync of 6 records of a landing took 547 ms" at 18:05:58).

What those records do not say is which job held the pool, or whether the note's own read of the board was slow,
so the note cannot be blamed beyond this: it was the one step between beats, it had no ceiling, and the pool it
ran on was held past the five seconds a client allows. Both halves of the change above remove it, which is why a
`0.6.1-alpha.10` still shows it and the branch does not.

The controls that hold it are the three the "Fixed" section names, and `service_reply_tests.rs` adds
`a_note_that_takes_a_while_is_written_after_the_beat` and
`a_note_that_outlives_its_beat_is_left_for_the_next_beat`.

What the person read is now `silent_notice` in
`package/git-policy/cli/src/native/landing/service_messages_of_a_wait_for_the_service.rs`: it names the seconds the
service has said nothing, says that the same commit is sent again, and says that the command goes on by itself, so
nothing is asked of the reader. The older words claimed the service had stopped answering, which the records above
show it had not, and named no duration.
