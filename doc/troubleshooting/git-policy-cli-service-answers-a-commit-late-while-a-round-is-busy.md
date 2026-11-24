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
  Done:
  the beat is written before its note,
  a trash's entry removals take every place of the blocking pool but one (`places_a_round_may_take`),
  and a round starts housekeeping for no longer than `HOUSEKEEPING_OF_ONE_ROUND`
  ("Fixed,
  2026-11-11" in this file).
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
housekeeping,
and the cause it removed was narrower than the inference above:

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
  so the bound says what a round starts,
  never how long one step of it lasts.
  That is the limit that stays:
  a single trash of a worktree with a large `target/` directory still holds its round for as long as the removal takes,
  and what keeps a person's commit served meanwhile is the place the removals leave free and the beat.

The reproduction,
on a wrapper built from the branch before the change,
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
19,558 ms after its landing,
`answer_ms=19558 total_ms=49279`),
and it is a separate wait from the one this document is about.

The controls that keep this from coming back are
`service_server_connection_handling::tests::a_client_is_greeted_and_beaten_while_every_place_of_the_pool_is_held` (fails
after 5 s before the reply change),
`worktree_trash_entries::tests::a_trash_removal_leaves_one_place_for_a_step_that_answers_a_client` (fails after 10 s
before the place change) and
`service_work::tests::a_round_that_has_spent_its_housekeeping_leaves_the_rest_for_a_later_round`.

## What the three silences of 2026-11-11 had in common

`0.6.1-alpha.10` was the installed release for that whole day and does not carry the change above
(`git merge-base --is-ancestor 28f5f7b14f monochromatic-git-policy-cli-v0.6.1-alpha.10` fails).
Three commits of that day printed the sentence,
and each one's records name the same two facts:
the service
reached the end of its work on that commit while the client heard nothing,
and the same service logged the
device under its journal taking seconds to answer a few small writes.

- 18:05:32 local (`23:05:32Z`),
  the main checkout,
  `totalMs=6645`,
  `submitMs=5453`,
  `answerWaitMs=5428`,
  outcome `landed`,
  exit `0`.
  The service line for its request (`3ef36edc-cc1c-48ea-9250-e005673b9c45`)
  has `read_ms=0`,
  `landing_ms=4737` (`index_update_ms=3447`,
  `ref_ms=1241`) and `answer_ms=8`:
  the request was read at once,
  the landing worked for 4.7 s,
  and the answer was written 8 ms after it ended.
  That landing's six record syncs began 19.3 s after its answer and took 547 ms.
- 19:50:09,
  a worktree (`pi-kimi-findings-gate-and-install-scripts-round-210-findings`),
  `totalMs=14391`,
  `submitMs=13812`,
  `answerWaitMs=12590`,
  outcome `landed`,
  exit `0`.
  Its service line
  (`ba969533-1db1-4499-a58f-3001ec423879`) has `read_ms=3506`,
  so the request sat 3.5 s before the service
  read it,
  then `landing_ms=6253` (`reads_ms=5638`) and `total_ms=13651`;
  the same instance logged its
  journal syncs at 4678 ms for one record and 18861 ms for six.
- 19:54:31,
  the main checkout,
  `totalMs=11491`,
  `submitMs=5575`,
  `answerWaitMs=5527`,
  outcome `landed`,
  exit `0`.
  Its line (`6389996c-e1d6-4cbd-afda-5bd48d2e433d`) has `read_ms=0`,
  `queued_ms=50`,
  `landing_ms=5418` (`objects_ms=3870`,
  `reads_ms=563`) and `answer_ms=0`,
  and the instance's next line is
  "the background sync of 6 records of a landing took 10680 ms",
  begun 10 s after that landing's answer.
  A `git merge --ff-only` of that checkout had been running for more than a quarter of an hour while this
  commit ran,
  starting one short Git child after another.

So "what made the service silent" is not one step of the landing:
in all three the landing was working or had
just finished while the client heard nothing.
The one step between two beats in that release was the note,
a
read of the journal's board through the blocking pool awaited with no ceiling,
so a device that slow can hold
a beat past the five seconds a client allows even when nothing else is wrong.
The 19:50 record shows the pool
occupied as well (`read_ms=3506` before the read of a request that names another worktree),
which is the
mechanism the "Fixed" section above removed on two fronts.

The controls that hold it are the three the "Fixed" section names,
and `service_reply_tests.rs` adds
`a_note_that_takes_a_while_is_written_after_the_beat` and
`a_note_that_outlives_its_beat_is_left_for_the_next_beat`.

What the person read is now `silent_notice` in
`package/git-policy/cli/src/native/landing/service_messages_of_a_wait_for_the_service.rs`:
it names the seconds
the service has said nothing,
says that the same commit is sent again,
and says that the command goes on by
itself,
so nothing is asked of the reader.
The older words claimed the service had stopped answering,
which the
records above show it had not,
and named no duration.

## The change in the field, later the same day

`0.6.1-alpha.11` carries the change above and was installed at about 20:01 local.
From then to 21:41 the
repository's timing log holds 91 commits of that release:
18 of them waited five seconds or more for their
answer (the longest 27,145 ms of `answerWaitMs`),
and none of the 91 printed the notice.
So what a slow answer
costs is still there to fix,
and the false claim that the service had stopped did not come back.

From lines 48490 to 48573 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Two commits told "not known whether the commit was made", one of which landed 73 s later (2026-11-09T07:44:20Z)

Reported from `.claude/worktrees/pi-repo-read-and-index-route-findings-on-main`.
Nothing was lost and nothing doubled;
the person was told something untrue by omission while the service still held the commit.

The records,
all for that worktree:

- The wrapper's timing log (`.git/monochromatic-git-policy-cli-commit-timing.jsonl`),
  two `warn` entries,
  each naming `outcome: "stopped"` and `sends: 6`:
  `fdca390c` at 02:43:35 (total 62,307 ms,
  `answerWaitMs` 31,066) and `b4a0ab9a` at 02:44:20 (total 32,403 ms,
  `answerWaitMs` 31,124).
  Both hold `positions: []`:
  no queue position ever reached the person.
- The service journal (`journalctl --user`),
  six `answered` lines for `b4a0ab9a` at 02:44:48.608-609,
  all `outcome=branch-moved`,
  and one `landed` for `fdca390c` at 02:44:48.607 with `landing_ms=74`,
  `queued_ms=104022`.
- The reflog:
  `1ad4ad25d7` at 02:44:48,
  which is that landing.
- A third run at 02:47:41 landed in 6,014 ms with one send.

What `read_ms` measures is in `service/service_timing.rs`
(`Arrival.read`):
the time from the service accepting the connection to the whole request line being read.
It is not a landing figure and not a figure about Git.
It reached 78,085 ms on one of those connections because the service process read no request line between 02:43:30 and 02:44:48,
not because the client was slow to send:
six connections of two runs were accepted over that window and all were read within 60 ms of 02:44:48.
`queued_ms` (88,471 to 116,478 ms on the same burst) is the other half:
a request that was read at once and then waited for its batch.

Why the service stopped for 105 s:
the whole user journal has no line between 02:42:35 and 02:44:48,
for every unit,
not only the service,
and every queued landing then ran in one burst.
The service's own logging cannot explain it:
its lines go through `send_without_waiting` (`service/journald_native.rs`),
which is `MSG_DONTWAIT` on a socket and a polled page-sized write on a pipe,
so a journal that stops drinking drops lines instead of blocking the service.
The stall was the host's.
This host's disk and scheduler pressure is `NHR`'s normal,
and the wrapper cannot see it.

What the wrapper does with it is right and what it says is not:
`HEARTBEAT_SILENCE` is 5 s and `RECONNECTS` sends are tried
(`service/service_client.rs`),
so two runs each spent about 31 s on six sends and then printed
`ClientFailure::ServiceSilent`:
"so it is not known whether the commit was made".
The service sends a `Heartbeat` every `HEARTBEAT_EVERY` and a `Position` when a batch starts
(`service/service_reply.rs`,
`service_server_land_group.rs`),
so the client's silence bound is a bound on a *silent service*,
not on a slow commit:
extending it would be waiting without evidence,
which the owner's "nothing should ever wait forever or indefinitely" rules out.

The two honest improvements,
neither built:

1.  Before printing the silence message,
    read the branch once (a bounded `git rev-parse`) and say what it shows:
    the branch is at the commit this command prepared,
    or it is not.
    That turns "not known whether the commit was made" into a fact in the common case,
    costs one Git start,
    and waits for nothing.
    In this incident it would have told run A that its commit was made and run B that its prepared commit was not on the branch.
2.  Make a waiting person see the queue:
    `Position` lines exist and `positions` stayed empty here,
    so a request that waits minutes shows nothing at all.
    A position notice is a heartbeat with meaning,
    and it is what lets a bound on silence be tight without hiding a long queue.

Not done here because both are the client's reach and message path (`service/client` and `landing/`),
not the ceiling and notice this branch changes.

From lines 48574 to 48594 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

What happened to the two,
measured (2026-11-09T10:21:14-05:00):

- The service sent no place,
  and this is why.
  `service_server_land_group.rs` told a request where it stood only when it was not the first of its batch
  (`fresh.iter().enumerate().skip(1)`),
  and every request of both runs was alone in its batch.
  The journal since 2026-11-08 holds 2,241 `answered` lines:
  2,223 with `size=1 position=1`,
  18 with `size=0` (a repeat answered from memory),
  and none with `size` of two or more.
  So `Position` was never sent to anybody in this window,
  and it could not have been:
  the send whose line the journal holds at `read_ms=78085` had not been read at all,
  so no batch held it.
- The answer is that it was never sent,
  not sent and unread:
  a place that is not in a reply reaches no client,
  and every one of the 919 records of both timing-log generations holds `positions: []`.
