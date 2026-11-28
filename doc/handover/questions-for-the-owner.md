# Questions for the owner

Open questions from the coordinator session,
newest first.
This file is open in Zed,
which reloads it when it changes on disk and the tab has no unsaved edits.
Write your answer under a question and save;
the session re-reads this file before every update,
so a saved answer is never overwritten.
An answered question moves to the canonical doc it belongs to and leaves this file.

For this session the owner said:
"Please make any non-destructive options (when they are the ones I would likely have picked) yourself from now on for
this session."
So this file holds only choices that are destructive,
reach outside this machine,
or where the likely pick is unclear.
The non-destructive choices the session made itself are listed at the end,
so any of them can be undone.

## Open: four proposals from rounds, each with its own questions, recorded 2026-11-27T04:34:22-05:00

Four rounds each wrote a proposal on 2026-11-27 and changed no code.
Each document gives what happens today with the source lines,
what was measured,
the options with what a person would see,
a ranking,
and the questions it leaves to you.
They wait here and are not put in a dialog while rounds run,
because a dialog holds the session and no round merges until it is answered
(`doc/planning/pi-round-start-failures-and-what-would-stop-them.md`).

### The 60 s bound on the Git calls `cargo-shadow` makes before a command (issue 6555)

`doc/planning/cargo-shadow-bound-on-the-git-calls-before-a-command.md`.
The round measured the same call at 80 ms to 130.6 s on this repository under load.
Its ranking is D > A > C > B > E > F > G:
D caches the base and its lockfiles per commit and so removes 39 of the 40 Git starts it counted;
A judges the calls by the progress meter the wrapper already uses instead of a wall clock.
Its questions:
may a Git call before a command take minutes on a loaded host as long as it is working;
if a wall clock stays,
what number and from which measurement;
and whether the wrapper's "no sign of work for its quiet window" rule is the project's answer here too.

### An ignored-state copy owed to a worktree that is gone (issue 6557)

`doc/planning/cli-git-owed-ignored-state-copy-of-a-removed-worktree.md`.
Its ranking is 1 > 2 > 5 > 3 > 4,
and option 1 is that the worktree's absence settles the claim and the source settles the stage,
so the wrapper removes the stage folder itself.
It counted 20 stage folders under `.claude/worktrees/`,
42,628 MiB by `du` at 2026-11-27T03:56:10-05:00,
most of them left by the starts of that morning whose `git worktree add` was killed at its bound (issue 6563).
Its questions are under "What only the owner can answer" in the document.
Until this is answered the folders stay:
the wrapper says to remove them by hand once nothing needs them,
and the coordinator has not removed any.

### How starts from several launchers order their worktree adds (issue 6563)

`doc/planning/pi-driver-order-of-worktree-adds-across-launchers.md`.
It reads the morning of 2026-11-27 from what the machine left and starts no add of its own.
Its ranking is A > B > C > D > E:
A is an add cheap enough to need no order,
which is what the ZFS route does for a worktree,
and it says the copy of ignored files is 89% to 99% of an add that returned;
B moves that copy off the add's waiting path;
D and E are a claim per add or one queue.
Its first question is whether any rule about starts is wanted at all,
or whether the answer is the hold of 2026-11-18 read again
("We shouldn't use worktrees heavily before we implement the zfs parts for the git wrapper."),
and it notes that a bound on overlapping adds needs a figure nobody has measured,
and that measuring it would mean starting overlapping adds on purpose,
which is synthetic load and needs your word.
Its second is when the checkout's move to its own dataset runs and what holds until then.

### A push the server refuses with GH009 (issue 6559)

`doc/planning/cli-git-push-refused-by-the-server-with-gh009.md`.
Its ranking is B > E > C > A > D:
B gives the refusal the bounded retry GH006 already gets (five tries over two and a half minutes),
E is one more try after the first pause.
Its questions:
whether a `GH009` refusal gets a bound and which;
whether one rule should cover both shapes of the refusal,
the readable `GH009` and the unreadable `(failed)` decline;
whether the record should name GitHub's documented remedy above the server's words;
and whether a client-side upload check is wanted at all,
since no refused push of this boot referenced an LFS object.


## Open: five choices the rounds of 2026-11-27 left, recorded 2026-11-27T05:47:00-05:00 and revised 2026-11-27T06:24:00-05:00

Each comes from one round's report,
restated here because the reports are scratch and expire.
The rankings are the rounds',
except where one is marked the coordinator's.

### Where a merge the word-level pass finished says so (issue 6529)

The fix merged (`5914dd3810`):
the pass no longer writes back a line one side deleted.
The issue's third request is open:
a merge a person runs is committed with Git's prepared message,
so the commit says nothing about what the pass decided.
The pass already prints one line per merged part on standard error and writes a journal entry,
and the auto-push reconcile merge already appends those lines to its commit message.
`SPEC.md` fixes the message today ("the prepared message as it is"),
so each option changes the SPEC.

- A:
  append the part lines to the merge commit's message,
  as the reconcile merge does.
  A reader of `git show` gets the path,
  the three line spans and the blob names.
  Every merge the pass finishes carries those lines,
  and an `--edit` finish opens an editor holding them.
- B:
  append one trailer line naming the pass and the paths.
  Smaller in the log;
  it drops the spans and the blobs,
  and this repository uses trailers nowhere.
- C:
  leave the commit alone.
  Nothing is added,
  and a person reading the history still cannot tell the pass finished a merge.

Ranking:
A > B > C.
A beats B because the spans and blob names are what let a reader check the resolution.
B beats C because any durable record on the commit beats a line on standard error nobody kept.

### What closes the last file loop in `foundation/`

`diagnostic_lines_off_the_calling_thread.rs`,
`diagnostic_log.rs` and `lines_written_by_a_thread.rs` use each other in a loop the allow list tolerates in three lines.
The plan's move was applied and the linter run:
a loop of two replaces the loop of three,
because the log's door writes through those lines,
so any file under it that says a log line closes a loop with it.
The line in question is the notice that the queue's reader stopped.

- 1:
  the door says the line.
  Three functions answer whether that call stopped waiting.
  Three signatures change,
  and two callers that hand a line in without the door would say the notice late or never.
- 2:
  the queue records that a reader just stopped,
  and the door takes that record after each write.
  No signature changes;
  the same lateness as 1,
  and a state flag.
- 3:
  the words move below the queue,
  into a file of their own,
  and `diagnostic_log.rs` keeps only the door over them.
  No signature change and no lateness;
  it cuts `diagnostic_log.rs`'s public shape in two,
  and that file's controls for those words move with them.

Ranking:
3 > 1 > 2.
3 beats 1 because it alone keeps the notice said when it is said today without changing a signature.
1 beats 2 because a returned answer is more local than a flag another file has to take.
`foundation/lines_written_by_a_thread.rs` (391 code lines) waits for this before its split.

### What a landing writes for a path it has not read (issue 6566)

A landing records a path's stat data only when the file provably holds the bytes the commit stores,
and the proof reads and hashes the file.
For a change list of 4,097 paths that was 33.5 s and 34.9 s through the service,
against native Git's 0.595 s and 0.607 s for the same change,
measured once on 2026-11-27 on a loaded host
(`doc/troubleshooting/git-policy-cli-landing-index-reset-reads-every-changed-path.md`).
A proposal round wrote the options (`5bf07aef72`,
`doc/planning/cli-git-landing-index-reset-reads-every-changed-path.md`),
with Git's own behaviour cited from its source at `v2.56.1`.
It found the form the issue named changes nothing:
the entries it would spare are the ones the reset already leaves alone,
and a staged change list already costs no read.
The shapes that do read are `git commit -a`,
an amend,
a merge,
a rebase,
an import,
and every commit past `MAX_CHANGES` that real Git builds.

- A:
  today's proof,
  one read and one hash for every path the reset writes.
- B:
  keep the live index's stat data where the blob already matches,
  the form the issue named.
- C:
  carry the stat data of the index the commit was staged through.
  A landing costs one `lstat` a path and the next command finds the entries clean;
  it needs a field on the wire and in the journal.
- D:
  write the zero value,
  as `git reset --no-refresh` does,
  and let the person's next command pay the hash.
- E:
  write the zero value,
  then refresh through Git after the lock is released,
  which adds a second writer of the live index.
- F:
  a flag that picks between the proof and the zero value for one commit.

Ranking,
the round's:
C > D > E > A > F > B.
C beats D because both hold a landing to one `lstat` a path and only C leaves entries the next command finds clean.
D beats E because E buys the same speed with a writer that races what the person stages next.
E beats A because E hashes outside the index lock,
so another client's commit is not held behind a vendor drop.
A beats F because F is A plus a switch only a person who knows it gains from.
F beats B because B changes nothing for anyone.

The proposal's own questions,
which the choice rests on:
which measurement is the cost to beat (the record's 30 s,
or the round's 3.06 s and 7.00 s);
whether the landing may take stat data from the staging index (without that,
C cannot be built);
whether the zero value is acceptable for a path the landing cannot prove;
whether any of it is a flag;
which entries' stat data the differential controls must pin;
and whether `SPEC.md` gains the fifth condition the source has and it lacks
(the file's change time carries nanoseconds).

### Two bounds no round may size (issue 6556)

Seven Pi starts of 2026-11-26 ended without a run.
Four stopped at the sops decrypt of the checkout's `.env.local.json`
(`SOPS_TIMEOUT_MS`,
15,000 ms,
`package/env-local-json/core/src/constants.ts`),
before Pi was started.
Three stopped at the start-up answer bound
(`START_UP_ANSWER_MILLISECONDS`,
120,000 ms),
on a host at load 100 to 235.
Each now leaves a line naming the wait,
its bound and how long it waited (`faa52ab2af`).
Open:
whether either bound changes,
which needs a measurement under load;
and whether a start that stops before Pi starts writes a `summary.json`,
which needs an end reason of its own.

### Whether a stuck pipe ends a control's wait at once (issue 6548)

The control that proves a copy does not wait on a file replaced by a named pipe had borrowed a ten-second rendezvous patience as a deadline over real work;
its window is now sized over the whole run (`407adb173c`).
In a busy shared process a copy really stuck in a pipe's open is therefore waited for up to the four-minute ceiling.
The gate runs each test in a process of its own,
where 60 s ends it.
The choice:
whether the pipe's state should end the wait immediately,
at the price of a probe on every look
and a false reading if the probe catches the release code's own non-blocking open.
The coordinator's ranking:
leave it > probe,
because the gate's verdict is unchanged and a false failure costs more than a slow one.


## Open: does the crate split's step 4 run now? (recorded 2026-11-27T06:40:00-05:00)

The plan's rounds 1 to 3 are done or nearly done
(`doc/planning/cli-git-source-layout-target-tree.md`,
"Next"):
the single-item moves merged on 2026-11-26,
the splits and the loop breaks on 2026-11-27,
and the allow list `layout-allow-list/file-rule.txt` went from 544 lines to 52.
What is left of round 3 is one loop of twelve files in `worktree/`,
with a round running,
and three lines in `foundation/`,
which wait for the choice recorded under "five choices the rounds of 2026-11-27 left".

Round 4 is the plan's five call changes,
each a changed signature in release code:
finished results (`subvolume_command` and `doctor` stop building `Action` themselves),
system settings path (`repo_read_config_scan` stops calling `git_system_config::system_config_for`),
after-stop action (the watcher's callers stop passing a `LockLedger`),
current step record (`transaction_git` stops entering its step through `service_busy_step_record_on_this_thread`),
and run this next (`stacked_merge_push` stops planning a whole invocation in process).
On `main` today each call site the plan names is still there:
`plan_invocation` in `push/stacked_merge_push.rs`,
`LockLedger` in six files of `push/`,
`service_busy_step_record_on_this_thread` in `transaction/`,
`service/` and `push/`,
and `system_config_for` in two files of `repo_read/`.

The coordinator did not start it.
The plan calls itself a draft for the owner's review,
the only recorded answer about the split is when to cut the crate
("Window after live rounds (Recommended)",
`doc/planning/cli-git-source-layout.md`),
and the loop-break rounds were told to leave any loop that needed a changed signature,
which is what round 4 is made of.
An earlier entry of this file says rounds 1 to 6 need none of the owner's open points;
that is the coordinator's reading of which points hold which round,
not an acceptance of the changes.

- Run round 4 now,
  one round per call change.
  Three of the five meet work in flight
  (the push split cuts `child_watch_one_shot_run.rs`,
  issue 6573 changes the control that reads `repo_read/git_system_config.rs`,
  and the choice of issue 6529 edits `merge_second_pass_finish.rs`),
  so those three start after that work merges.
- Read the five signatures first
  (the plan's "The call changes,
  as signatures")
  and accept or change each.
- Leave round 4 and go to what needs no changed signature.

Ranking,
the coordinator's:
read first > run now > leave.
Read first beats run now because five signatures are five design choices nobody but a draft has stated.
Run now beats leave because rounds 5 to 7 build on these changes,
and the files over the budget in `invocation/` and `health/` wait for round 7's move.

## Open: where a worktree removal's Git reads its cache place (issue 6571, recorded 2026-11-27T07:08:00-05:00)

Controls whose service runs inside the test process wrote `git-system-attributes-<digest>`
under the real home cache.
The round measured the writers and fixed the ones it could without a signature the release's callers see
(`c1a3b33db5`):
a fixture names a cache place of its own,
and `for_context` now lets a variable its caller named win over the process's own,
where the caller's entries were appended and the first match won,
so a caller's variable never reached the place.
That last change is release code:
the release's overlay names no cache place,
and the built binary run with a throwaway home wrote its two cache files there.

One writer is left,
and reaching it changes a signature:
a worktree removal's service-side Git (`the_service_git`) builds a context of its own from the process's environment,
so a removal a control runs still writes `git-system-config-<digest>` under the real home cache.
The proposal is `doc/planning/cli-git-removal-work-cache-place-the-caller-hands.md`.

- A:
  the removal work runs under the `GitContext` its handlers already hold,
  and `the_service_git`'s own resolution of Git goes.
  One channel for the place;
  the removal stops resolving a second Git.
- B:
  the work keeps resolving its own Git,
  and its caller hands it the variables.
  It keeps the Git of the attempt for a service that restarts mid-record.
- C:
  a build of the controls refuses a write to the process's own home cache.
  No signature changes;
  two callers already catch a panic and decline,
  so the file can be written and the control stay green.

Ranking,
the round's:
A > B > C.
A beats B because the service already holds that one value for every child it runs.
B beats C because B fixes the place for every route that reaches the work,
where C only refuses the write.

## Open: how long cargo-shadow's stop waits for a killed scope (issue 6577, recorded 2026-11-27T07:45:45-05:00)

On 2026-11-26 `cargo-shadow`'s install step refused a command with
"a process started by cargo could not be stopped",
on a host with about sixty rounds building and a disk stall near `some avg10=75`,
and left the lockfile change cargo had written (issue 6528).
The round for it measured that the kernel does not count a member that has exited and waits to be reaped,
so the zombie the issue named was not the cause,
and made the refusal say how long the stop waited and name each member left (`41b9597adf`).
What refused that day is not known and cannot be recovered.
A live member past the bound is what is left:
a killed process inside an uninterruptible disk wait does not end until that wait returns.

The bound,
`QUIESCE_BOUND` in `package/cargo-shadow/cli/src/bounded_run.rs`,
is 5 s,
a delegated pick with veto open,
sized from probes of 2026-11-04 that took 1 ms to 5 ms on a host without that load.
The round left it:
a shipped bound needs a workload measurement or the owner's decision.

The options are the coordinator's and none is measured.

- A:
  keep 5 s and rely on the message.
  Nothing new to build,
  and the next refusal states the wait and each member's state,
  which is the measurement the bound lacks.
  The refusal recurs under the load that is normal on this host,
  and each one fails a command and leaves a lockfile change for the next run to settle.
- B:
  keep waiting while every listed member has the kill pending and is in an uninterruptible wait,
  saying so every few seconds,
  up to a ceiling sized from the worst stall measured here,
  then refuse.
  The wait follows the cause,
  so a command under disk load runs late where today it is refused.
  It is a new rule in release code with a ceiling that still needs its measurement,
  and a command can wait as long as that ceiling.
- C:
  raise the fixed bound to a figure sized from the next occurrence's message.
  One constant changes.
  One occurrence sizes it,
  so a longer stall refuses again,
  and until that occurrence it is A.

Ranking:
B > C > A.
B beats C because its wait ends when the member's own wait ends,
where C's figure holds only for stalls no longer than the one that sized it.
C beats A because A leaves the refusal recurring under load,
which this host has every day.

## Open: where the wrapper package's TypeScript unit tests and lint run (issue 6584, recorded 2026-11-27T08:32:56-05:00)

On 2026-11-27 the wrapper package's TypeScript side was found red on `main` with nothing running it:
`test:unit` failed two files and `lint:oxlint` reported 850 errors.
The unit tests are fixed (issues 6579 and 6580);
the lint has four rounds (issue 6581).
The release gate runs the Rust suite and Clippy,
and what the gate runs is recorded as the owner's.
The proposal is `doc/planning/git-policy-cli-where-the-typescript-unit-tests-and-lint-run.md`.

What the three checks cost,
measured once on this host:
`lint:types` 2.8 s and 0.45 GB,
`lint:oxlint` 12.8 s and 1.0 GB,
`test:unit` 22.2 s and 4.5 GB,
37.8 s one after another.

- A:
  inside the release gate,
  as a third check beside the suite and Clippy,
  run on the host inside the gate's unit.
  It runs whenever a batch's gate runs,
  and a refusal names it.
  A TypeScript failure then turns a batch's gate red.
- C:
  at the coordinator's tip check after a batch of merges,
  through the package's `native:ts-checks`.
  It answers before the gate starts;
  it is a routine,
  and a routine that lapses does so silently,
  which is what happened.
- B:
  a unit beside the gate,
  started with it.
  A second verdict to read,
  with nothing tying the two together.
- E:
  a CI workflow of its own,
  filtered by path.
  It reports without anyone deciding to run it,
  on a surface that was not read:
  `forbidden-strings` failed on `main` on each of the last three pushes.
- D:
  the advisory CI workflow's fast subset,
  whose budget of eight runs a day misses most merges.
- F:
  in every round that changes the package,
  before its branch merges.
  It cannot see a break two branches make together.
- G:
  required checks on pull requests,
  as comparable repositories do,
  which the recorded merge bar rules out.

Ranking,
the round's:
A > C > B > E > D > F > G,
by how soon and how surely a red check is seen by someone who was not watching.
A beats C because the gate is what a release waits for,
so its step cannot lapse unnoticed.
C beats B because it adds one command to a step that happens anyway and answers first.
B beats E because its answer is read where the batch's green is declared.
E beats D because it runs on every matching push.
D beats F because it needs no brief to be obeyed.
F beats G because G cannot run without pull requests.

The proposal's other questions:
whether a red check may hold a merge;
whether the placement lands before or after issue 6581 is fixed,
since a gate that runs the lint while it is red refuses every batch;
what memory bound the step gets;
and whether `native:ts-checks` should run `lint:oxlint` by default.

Until an answer,
the coordinator runs C by hand:
`test:unit`,
`lint:types` and `lint:oxlint` on `main` with each batch check.

## Open: four choices the release passes of 2026-11-27 left, with five picks beside them (recorded 2026-11-27T09:40:05-05:00)

Each comes from a round's report or a release pass
(Claude Sonnet reads,
same vendor as some of the authors,
so second passes and not independent reviews).
None holds a merge or a release.

### Where `mpm-shadow`'s two plan-insert modules live

The splits put the plan insert's place and spans code in two files beside `plan_insert.rs`
(`src/toolchain/plan_insert_place.rs`,
`src/toolchain/plan_insert_spans.rs`),
so `pub(super)` on their items means all of `toolchain`,
where before the split the same items were private to the plan insert.
The round narrowed seventeen declarations from `pub` to `pub(super)` (`cc61079f82`).

- B:
  move both files into a `plan_insert/` folder,
  so `pub(super)` means the plan insert again.
  The compiler then refuses any other module of `toolchain` that reaches them.
  It changes two module paths,
  and it adds folder findings under the layout rules,
  which this crate does not switch on.
- A:
  leave the files where they are.
  Nothing moves;
  the items stay reachable from every module of `toolchain`,
  held back only by review.

Ranking:
B > A,
because B makes the compiler hold the reach the code had before the split,
and A leaves it to readers.

### How far `mpm-shadow`'s five test-only names reach

Five names are `pub(crate)` only so tests under `src/toolchain/` can read them.

- A:
  narrow them to `pub(in crate::toolchain)`.
  The tests that read them still compile,
  and nothing outside `toolchain` can.
- B:
  leave them `pub(crate)`.

Ranking:
A > B,
because A serves the same readers with a smaller reach and changes no behaviour.

### What shows that an alpha of `cargo-shadow` was tested

The crate-release workflow runs no test for an alpha:
the run that published `0.1.5-alpha.11` holds no line of the `test` task
(`package/cargo-shadow/cli/README.md`,
"Which host each check runs on").
The evidence is a round's local run of the package's suite,
which needs a delegated control group of the account's service manager.

- C:
  the package's release task refuses to start unless a suite result is recorded for the commit it releases.
  The evidence is then tied to the released commit.
  It needs a place for that record and a rule for what counts.
- A:
  as now.
  The suite runs in the rounds that change the package,
  on their branches,
  and nothing ties a run to the commit that is released.
- B:
  the workflow runs the suite.
  It would report without anyone deciding to run it.
  Not measured:
  whether a hosted runner gives the suite the delegated control group it needs;
  the tests that cannot run without one are left out by the package's own task,
  so a green run there could cover less than a local one.

Ranking:
C > A > B.
C beats A because it ties the evidence to the commit.
A beats B until someone measures what B's runner can run,
because a green workflow that skips the scope tests reads as more than it is.

### Whether the two fence measurement scripts of 2026-11-21 stay

The two scripts under `package/git-policy/cli/perf/` can no longer measure what they measured:
the crate no longer reads their switch,
and this host takes another rung
(`doc/planning/cli-git-fence-size-read-beside-source-check.md`,
"The harness after 2026-11-27").
Their results are in that record.

- A:
  remove them.
  The record keeps what they found;
  nothing has to keep dead scripts lint-clean.
- B:
  keep them,
  as they are,
  for whoever measures the fence again and wants a starting point.

Ranking:
A > B,
because a script that cannot run its measurement misleads the next reader more than its absence would,
and the record says how the measurement was made.

### Picks made, veto open

- `cargo-shadow`'s library surface is alpha.10's without the `git_objects_packed_commit` path,
  which was internal layout,
  and with `ByteCount` (`d134b5f836`).
  The alternative is one `pub use` alias so the paths match exactly.
  Picked because the library goes to GitHub releases only and nothing outside this repository names it.
- `cargo-shadow`'s `bounded_run_end` stays `pub(crate)`.
- `LISTING_BOUND`,
  the time `cargo-shadow` gives a refusal to list what a scope still holds,
  is 500 ms (`c59a4f5fd5`).
  The round measured the listing at 37.5 to 45.6 ms for a scope of three thousand threads,
  and the value sits beside that measurement in `src/control_group_stop.rs`.
  It is the only delay a refusal adds after its own bound.
- A notice's record outlives a line that could not be written.
  `package/git-policy/cli/SPEC.md` says two things about the record of a failed push or a kept worktree:
  that one "whose words could not be written stays for the next command",
  and,
  since the fix for issue 6421,
  that these records are among the "lines a person can live without".
  Since `6ea539bc91` the code follows the second,
  and the release pass of `0.6.1-alpha.48` showed the record lost with standard error closed.
  No decision record chooses between them
  (`doc/decision/` and `doc/planning/` searched for both phrasings).
  The round `wrapper-alpha-48-release-pass-a-notice-record-outlives-a-line-that-was-not-written`
  builds the first:
  the command is never held,
  and the record stays until its line is known written.
  The alternative is to accept the loss and correct the first passage.

What proves the reload remedy,
now that no release this host holds comes from before it,
is its own proposal:
`doc/planning/git-wrapper-reload-proofs-control-arm-without-a-pre-remedy-release.md`
(issue 6587's second round).
The reload proof's control arm must end a desktop stand-in by a manager-wide reload,
and every release since the remedy of 2026-11-12 asks for none.
Its options:
A keeps two pre-remedy releases for the arm,
B has a stand-in ask the manager for the reload,
C retires the arm and plants one reload in the arm that remains,
which must then survive the steps;
D,
separable,
writes an earlier release's unit text into the machine,
since no arm meets one today.
Its ranking is C > A > B,
and D > A for the earlier text.

## Open: what a repository with slot records does with a shell alias the wrapper cannot read (recorded 2026-11-27T10:01:17-05:00)

The release pass of `0.6.1-alpha.48` found that a shell alias took a slot removal,
a move or a prune past the refusals.
The round for it made the refusals read every alias the destructive-command guard can follow
(`!git <words>`,
the bare `!git`,
a plain alias that chains to one;
merged as `cd62517318`).
What is left is an alias the wrapper cannot read,
such as `-c "alias.x=!sh -c 'git worktree remove -f -f <slot>'" x`,
which still reaches Git and deletes a bound slot's registration,
in alpha.47 and alpha.48 alike.
The proposal is `doc/planning/cli-git-slot-refusals-and-shell-aliases-they-cannot-read.md`.

- A:
  refuse every shell alias the wrapper cannot read,
  in a repository that holds slot records.
  No alias can then reach a slot command unread.
  It also refuses aliases that touch no worktree
  (`!sh -c 'git status'`),
  in every repository the ZFS route has served.
- B:
  refuse a shell alias whose value text names `worktree` with `remove`,
  `move` or `prune`.
  It catches the spelled-out case and leaves unrelated aliases alone.
  `eval`,
  a variable or a script file hides the command from it,
  and an alias that only mentions the words is refused.
- C:
  leave such an alias to Git,
  as now and as the guard's recorded rule does.
  Nothing unrelated stops working;
  the spelled-out case still deletes the registration.

The round's ranking is C > B > A,
with a note that A is the choice consistent with the four answers of 2026-11-26,
which refused every slot command the wrapper cannot prove
(`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`).
The coordinating session's reading differs in one place:
the proposal says A is "still defeated by the same `eval`",
and under A an alias that hides its command behind `eval` is an alias the wrapper cannot read,
so A refuses it.
Read that way,
A is the only option that closes the route,
and its price is the unrelated aliases.

## Open: five choices and three proposals from the rounds that ended by 09:45 on 2026-11-27 (recorded 2026-11-27T09:53:22-05:00)

None holds a merge or a release.

### Whether `mpm-shadow` repeats a start refused as busy

`0.2.7-alpha.13` explains a start refused with "Text file busy" and does not repeat it (issue 6585,
`c513ee060c`).
That was recorded here as a pick,
on the round's finding that every writer the tool starts publishes by rename.
The release pass of alpha.13 then showed the grounds were wider than the evidence:
this repository's own records measure a window of 0.5 to 3 ms in which a forked child holds a copied write descriptor
(`doc/troubleshooting/shadow-tests-that-write-a-program-and-start-it-are-refused-with-etxtbsy.md`,
`doc/troubleshooting/git-policy-cli-a-fixture-git-is-refused-with-etxtbsy.md`),
the round's probe was single-threaded and could not see it,
and `go install` writes its published path in place across file systems,
though no start of the shadow or the launcher resolves that path.
The record is corrected since
(`doc/troubleshooting/mpm-shadow-a-start-refused-because-the-program-is-being-written.md`),
and it ends with the same two options and the same ranking.
What its round measured:
with one thread writing a program and another forking,
5 to 12 of 20 starts were refused,
and none of 20 without the forking thread;
`go install` held its published path open for 3.5 ms across file systems,
and a prober starting that path was refused seven times in that interval;
`uv tool upgrade`,
which the self-upgrade runs,
stages the script and renames it.
Its counter-argument stands beside the ranking:
the window is measured in probes and test processes,
never in a production writer.

- B:
  repeat the start within a bound sized from the measured windows,
  then explain.
  The one cause anyone measured ends by itself in milliseconds,
  so the person's command works with no second try.
  It needs a bound with its measurement beside it,
  and a start held by a real writer waits that bound before it is explained.
- A:
  explain only,
  as released.
  Nothing waits;
  the person runs the command again by hand for a cause that was gone before they read the line.

Ranking:
B > A,
because B turns the only measured cause into a start that works and still explains the rest,
and A hands a transient to the person.

### What the five seconds of the start retry rest on

`START_RETRY_BUDGET` is five seconds in `mpm-shadow` and in the wrapper's `push/start_retry.rs`:
how long a start the system refused for want of a process is repeated.
It came with the design `doc/planning/cli-git-spawn-and-sync-reduction.md`,
"Process-start retry (slice R)",
which gives no measurement,
and no decision record names it (issue 6597).
The repository's rule asks a shipped bound to cite a measurement or a decision.
How long such a refusal lasts here cannot be measured on purpose,
since making that load is banned.

- C:
  both,
  in this order:
  record five seconds as decided now,
  and size it again from the first refusal met in ordinary work.
- A:
  record five seconds as decided,
  and leave it.
- B:
  leave it uncited until a refusal is met and measured.

Ranking:
C > A > B.
C beats A because it keeps the value answerable to a measurement when one exists.
A beats B because B leaves a shipped bound resting on nothing for an unknown time.

### Which program may provision the account's commit service

Issue 6586's round made a test build unable to offer itself to the account's service manager (`7d6bd90c92`),
and wrote the wider question as a proposal:
`doc/planning/cli-git-provisioning-from-a-program-that-is-not-an-installed-wrapper.md`.
Today whatever wrapper program runs points the account's shared link at itself,
and this machine's link names a build from `main`,
not an installation.
Its options:
A keeps that,
B refuses unless the program is the installed wrapper,
C refuses unless the build was made as a release,
D keeps it and prints which program the link now names.
Its ranking is A > D > B > C,
with the reason for each pair and five questions at its end.
B and C would refuse this machine's current practice.

### What the waiting notice says after a step's bound has come

Issue 6589's round found that the notice keeps promising a stop in the future tense after the stop was sent,
while a child in an uninterruptible wait holds the step
(a `gpg` held one 31.9 s under load).
`doc/planning/cli-git-notice-of-a-stopped-step-keeps-promising-a-stop.md` gives four options,
ranked A > B > C > D:
A says the stop was sent and that the command still waits for the kernel to let the child go.

### What a push does when its step's watch cannot start

Issue 6590's round found that a watch whose runtime is late reads as a refused thread,
and that the answer waits for a scan that names a leftover program.
`doc/planning/git-policy-cli-a-step-whose-watch-cannot-start.md` separates two choices:
for the watch,
A1 > A2 > A3
(A1 waits,
bounded,
for a runtime that is coming);
for the scan,
B1 > B3 > B2
(B1 moves it off the answer's path).

### What clears a scope unit a killed `cargo-shadow` leaves

A shadow killed while its launcher has not become a member of the scope leaves the unit loaded and empty,
and nothing in the package sweeps it (issue 6578,
whose two rounds fixed every path a living shadow controls).
`doc/planning/cargo-shadow-a-scope-unit-a-killed-shadow-leaves.md` gives five options,
ranked D > A > E > C > B:
D has a recovery stop the unit its own journal recorded,
A sweeps when a run starts,
E sweeps in the lean service,
which this host has disabled,
C leaves it and documents it,
and B,
a unit property that makes the manager collect an empty scope,
does not exist.

### Picks made, veto open

- The gate's suite container is bounded at 4 GiB (`2da7ccd13a`,
  issue 6583),
  2.76 times the 1,486.0 MiB peak one measured run reached,
  with that measurement beside the value.
  A gate at its bounds then holds 12 GiB at once.
- The container ledger lists Rust test files that name `podman`
  (two of the four Rust entries),
  where it skips TypeScript test files:
  a Rust test that starts a real container is then read for its limits (issue 6594).
- The ledger does not read a container start written through a shell in a Rust string
  (`Command::new("sh").arg("-c").arg("podman run ...")`).
  The file is still listed by the program name it holds,
  and its role is read by hand.
- Nothing that runs on `main` after a merge runs the ledger's test,
  so a file that names `podman` lands unlisted until someone runs it (issue 6588).
  This waits on the same answer as issue 6584's entry,
  "where the wrapper package's TypeScript unit tests and lint run":
  whichever place is chosen should run `package/container-limit/core`'s `test:unit` too.

## Open: choices the rounds that ended between 10:46 and 11:35 on 2026-11-27 left (recorded 2026-11-27T11:42:35-05:00)

None of these holds a round or a release:
each waits here while the release work moves,
and each names what was done meanwhile.
The reports they come from are in scratch and expire,
so what a choice needs is stated here.

### Which slice a TypeScript check's scope is counted in (issue 6605)

`native:ts-checks` starts each check in a scope of its own,
with bounds its round measured
(merged as `71d9ce980d`).
The scope sits in systemd's default slice,
`app.slice`,
which is where the recorded readings were taken.
A round starts the check from inside `agent-work.slice`,
the slice agent work is counted in,
and the check's own scope then leaves that slice.

- A:
  every check's scope goes in `agent-work.slice`.
  A round's checks are then counted with the rest of its work;
  a person's own run from a terminal is counted as agent work too.
- B:
  the scope takes the slice its caller is in.
  A round's checks are counted in `agent-work.slice` and a person's are not;
  it is one more thing the runner reads at its start.
- C:
  as now,
  `app.slice`.
  The readings stand as measured;
  a round's checks are outside the slice the machine's limits for agents are set on.

Ranking:
B > A > C.
B beats A because a person's run stays out of the agents' count.
A beats C because nearly every run of these checks is a round's,
and C leaves those uncounted.

### Whether a check that passed while held at its quota says so (issue 6605)

The runner prints the scope's counters only for a check that did not pass,
because its sentence is that a bound and not a test failed.

- A:
  one line for a passed check too,
  with the share of periods it was held.
  A slow pass explains itself;
  on a busy host most runs print it.
- B:
  as now.
  A pass prints nothing more;
  a pass that took several times as long says nothing about why.

Ranking:
A > B,
because the counters are read either way,
and the line is what tells a slow host from a slow check.

### One set of bounds for every check, or one per check (issue 6605)

One set serves every check:
1,024 tasks,
4 GiB,
800% of one CPU,
sized for the unit suite.
`lint:types` peaks at 39 tasks.

- A:
  as now,
  one set.
  One measurement to keep;
  a runaway in a small check is caught only at the large check's bound.
- B:
  each check carries bounds from its own measurement.
  A runaway is caught near the check's own need;
  every bound is one more that a grown check can meet,
  which is what issue 6605 was.

Ranking:
A > B,
because the defect on record is a bound set too low,
and no check has yet run away.

### Whether the package guide's rule on a test's children binds its TypeScript tests (issues 6596 and 6608)

`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md` states this for the Rust crate
(its section `Own every child, end it on every path`),
and in another section that every wait has a deadline:

```text
A helper whose state a control reads is one the control ends,
never one that ends by itself:
```

Two defects of that class were found in the package's TypeScript tests on 2026-11-27
(issue 6596,
merged as `a7dff54ebc`;
issue 6608,
whose round runs and was told the question is open).

- A:
  the guide says its sections on children and waits bind the package's TypeScript tests too,
  and one round reads those tests against them.
- B:
  the guide stays the Rust crate's,
  and each TypeScript defect of the class is fixed as it is found.

Ranking:
A > B,
because a brief can then cite the rule,
and two finds in one day say more are there.

### Whether the repository link's messages name this machine's drive (wrapper alpha.48 pass)

The pass found one machine's wording in what the wrapper prints.
Its round reworded the two messages the finding named
("so the repository link is not checked for this command",
merged as `14e54207de`)
and left the rest:
the walk's messages still say "the 860 drive is not mounted yet",
"ready (on the 860)",
"the link to the 860 is gone" and the like
(`repository_link_check_walk.rs`),
and a moved-aside copy is named with the suffix `.nvme-before-860-`.
Those sentences are pinned by `repository_link_shared_table.json`,
which `package/repository-link/check` reads too,
and that package's own README says it checks
"whether the repository link to the 860 drive is usable".

- A:
  both packages' messages and the shared table say
  "the drive recorded for this repository" and "the repository link";
  the suffix on disk stays.
  No message names one machine;
  the suffix still does,
  on copies only this machine's move has made.
- B:
  A,
  and new moves write a suffix that names no drive,
  with both suffixes read.
  Nothing new names the machine;
  every reader of old copies handles two names.
- C:
  as now.
  The check is this machine's and says so;
  another user of the wrapper reads about a drive they do not have.

Ranking:
A > B > C.
A beats B because the old suffix is on disk either way,
and a second one doubles what a reader must know.
B beats C because C keeps one machine's drive in messages every user of the wrapper can meet.

### Whether `pnpm-shadow` and `search-shadow` are released as alphas now (issue 6609, recorded 2026-11-27T11:46:49-05:00)

`mpm-shadow`'s copies of the rule the four shadow tools share had changed alone,
so the comparison in `search-shadow`'s tests failed on `main`.
The round for issue 6609 carried the change into the other three crates
(merged as `7e215a38c2`):
each package's own test task now runs the comparison,
and `pnpm-shadow` and `search-shadow` print a refused start with the system's own answer and its kind
(`could not start <path>: No such file or directory (os error 2)`,
where the path stood twice before).
The text that marks a program as a shadow did not change,
so the installed shadows still know each other.
The standing answer on alphas names the wrapper,
`cargo-shadow`,
`mpm-shadow` and the Pi driver packages
(`doc/decision/cli-git-wrapper-owner-answers-2026-11-12.md`,
item 4.9);
these two are not in it,
so neither was released.
`cargo-shadow` and `mpm-shadow` take the change in their next alphas.

- A:
  release `pnpm-shadow` `0.1.3-alpha.2` and `search-shadow` `0.2.3-alpha.2` now,
  lock and install them,
  and count both under the standing answer from here on.
  The installed tools then match `main`;
  two more tools release without a question each time.
- B:
  release both this once,
  and ask again next time.
- C:
  leave both at their installed alphas until a change of their own asks for a release.
  The installed tools print the older line;
  nothing else differs.

Ranking:
A > B > C.
A beats B because the four tools change together whenever their shared rule does,
and a question per release would hold two of the four behind the others.
B beats C because an installed tool that differs from `main` is one more thing a reader of a report must know.

### Picks the coordinating session made, veto open

- `cargo-shadow`,
  from the round that held the refusal's branches with controls (`9c4c5aac42`).
  The listing does not read each group a second time after it opens its descriptors:
  the refusal's decision does not rest on the names,
  and a second read doubles the reads under the listing's own bound.
  The README says what the descriptor proves in its place.
- `cargo-shadow`,
  same round.
  Two things it asked are taken as work,
  because the repository's rules already decide them:
  the refusal for a scope the manager collected goes on to say what the next run does with the kept journal,
  and the `Nothing` leftover gets a control that does not depend on a race.
  Both go to the round that follows the `0.1.5-alpha.13` pass.
- Issue 6605's readings stay where its round put them,
  beside the values and in the package README;
  no copy goes to `doc/planning/`.

### What the `cargo-shadow` and `mpm-shadow` passes of 2026-11-27 added, taken as work with veto open (recorded 2026-11-27T12:13:27-05:00)

Neither pass left a choice only the owner can make;
each point here is decided by a rule or a promise already on record,
and its round runs.

- `mpm-shadow`'s republish (issue 6600,
  released in `0.2.7-alpha.14`) left three points its round named,
  and the pass of that release confirmed each.
  A live installer the reader cannot see,
  in another process namespace or hidden by `hidepid`,
  is read as ended,
  and its directory removed or published.
  The package's README already promises the opposite,
  that a directory whose installer cannot be judged is left alone,
  so the code is made to keep it,
  without a lock
  (the pass suggests a boot and a namespace identifier in the hidden name).
  Hidden names written by earlier builds,
  and finished hidden copies of versions no lock names,
  are swept by `prune --toolchain` where their owner is provably ended,
  under the recorded "mpm-shadow cleans up after itself"
  (`doc/decision/mpm-shadow-and-cargo-shadow-clean-up-after-themselves-owner-answers-2026-11-13.md`).
  A publish that fails or is raced after a claim no longer costs the finished copy.
- The same pass found the repair publishes a hidden copy without the lock's SHA-256 behind it;
  the marker is made to carry what was verified,
  and the repair to compare it with the lock.
- The round on `mpm-shadow`'s publish (merged as `1add32a8fb`,
  recorded 2026-11-27T12:44:25-05:00) made two choices of its own:
  a finished hidden copy of a version the older-version rule forbids publishing is left hidden,
  not deleted,
  because it may be the only copy once the newer version is gone;
  and a recovery that failed is a warning when the install after it puts the version in place,
  and a failure when it does not.
  Each is one line to turn the other way.
- `cargo-shadow`'s `0.1.5-alpha.13` pass read in systemd's source three cases
  in which the manager never ends a scope that emptied.
  That is a second road to the leftover unit the section
  "What clears a scope unit a killed `cargo-shadow` leaves" puts to the owner:
  option D there,
  a recovery that stops the unit its journal recorded,
  may not reach a run that ended normally.
  A round showed the state in a guest and added it to the proposal
  (merged as `17a5efdf9a`,
  recorded 2026-11-27T12:25:24-05:00):
  with the account's inotify watches used up,
  the released `0.1.5-alpha.13` left its four scopes loaded and empty and exited 0;
  a re-execution of the manager collects such units and,
  in one of two timed attempts,
  lost two of forty.
  Option D cannot reach a run that finished,
  the two sweeps can,
  and the round keeps the ranking D > A > E > C > B with its first pair closer than before.
  It adds one question:
  whether a run's own last step should ask the manager about the unit it just started,
  which reaches this road before the run ends
  and brings back the question per scoped run that alpha.13 removed.
  The choice stays the owner's and stays open.

## Open: two choices the wrapper's alpha.49 reads left (recorded 2026-11-27T13:13:22-05:00)

Neither holds a round or a release.
Both come from Claude Sonnet reads of `0.6.1-alpha.49`,
same-vendor second passes and not independent reviews.
Each is to be filled in from its round's report before it is asked:
the rounds measure what the options cost.

### What a batch's width is when the host's free memory cannot be read

The service sizes each batch of its work from the CPUs and the free memory it reads when the batch starts
(`worker_budget` in `package/git-policy/cli/src/native/foundation/parallel_budget.rs`).
When the CPU count cannot be read the width is one.
When the memory reading fails there is no memory bound at all,
so the width is the CPUs less their margin;
the read of alpha.49 points out that one failure narrows and the other widens.
Alpha.48 did the same per round of work.
The round `wrapper-alpha-49-release-pass-four-small-points-of-the-behaviour-read` (merged `203bd65854`) holds what the code does now with a control,
`a_host_whose_memory_could_not_be_read_bounds_no_batch`,
which asserts a width of 28 on a stated host of 32 threads.
With no memory figure each caller comes to:

- a work directory's round of lanes (`service_round.rs`):
  the items and the CPUs less their margin,
  and nothing else;
- the pushes of different branches (`pushes_side_by_side`):
  the branches and the CPUs less their margin,
  with no ceiling;
- the slow kinds of one work directory (`slow_items_side_by_side`):
  the same,
  under `SLOW_ITEMS_CEILING` of four.

The reading is `System::new()` of the `sysinfo` crate,
and the round found no record of it failing.
The round's own ranking was to leave the code as it stands over a width of one,
which is B over C here;
it did not weigh A.

- A,
  keep the last good reading:
  a batch that cannot read memory uses the reading of the batch before it.
  It never widens past what memory last allowed and never drops to one for a passing failure.
  It costs a value kept between batches,
  and a first batch with no reading still needs one of the other two answers.
- B,
  no memory bound,
  as now:
  work runs as wide as the CPUs allow,
  which is what ran before the memory bound existed.
  On a host short of memory it can start more than memory holds.
- C,
  width one:
  the safe answer,
  and the slowest:
  a passing failure makes a whole batch run one item at a time.

Ranking:
A > B > C.
A beats B because it keeps the bound the reading exists to give,
at the price of one remembered number.
B beats C because C makes a whole batch run one item at a time behind a failure that may say nothing about memory:
nothing measured says a failed reading goes with a host short of memory,
and the coordinator has not looked for a failed reading in any log.
A and C are limits the shipped wrapper would carry,
and `AGENTS.md` asks such a limit to cite a workload measurement or the owner's decision beside its value.

### Whether the wrapper crate's published functions should be narrowed

The control that stops a split from widening an item's reach
(`source_guard/nothing_becomes_nameable_from_outside_tests.rs`)
did not see a function written `pub async fn`:
154 of them are nameable from outside the crate with no line in its list,
by the read's count.
(The round has since merged as `b929e8c39b`:
the list holds 12,395 entries,
12,295 of them names that were already nameable at `0.6.1-alpha.47` and that nobody has reviewed,
fields and methods among them.
Those counts are the list's on the day that round merged:
a later round widened the control again and narrowed ten names,
so the file's own length is the count now.)
The round `wrapper-alpha-49-release-pass-the-surface-control-sees-every-shape-of-published-name`
makes the control see them and gives each a line,
and narrows only an item a split widened.
Whether the rest should be public was never decided.

- A,
  a round narrows each one that still builds narrower:
  the compiler then warns about any that nothing uses,
  which it never does for a `pub` item,
  and the list shrinks to what is meant to be published.
  It touches many files at once,
  so it meets every other round at merge.
- B,
  leave them listed:
  the control holds the line from here,
  and nothing moves.

Ranking:
A > B,
because a narrower reach lets the compiler find dead code that a `pub` hides,
and the control already keeps the surface from growing either way.

## Open points of the `cargo-shadow` round that ended at 14:21 on 2026-11-27 (recorded 2026-11-27T14:30:09-05:00)

None holds a round or a release.
The round on the refusal's sentence is merged as `87399f8462`.

### A run that leaves a kept journal says nothing about it

A refusal keeps the run's journal,
and only a later run that resolves dependencies,
or `cargo-shadow update`,
settles it.
A pass-through command such as `cargo fmt` and a frozen run under `CI` leave it and print nothing,
so a journal can stay through any count of such runs unnoticed.
The round changed no run's output.

- A,
  a frozen run names each kept journal in one line:
  it already announces kept files at that place,
  and the cost is one directory read.
- B,
  a pass-through command does too:
  it has no such place today,
  so every `cargo fmt` would first walk up to find the catalog.
- C,
  as now.

Ranking:
A > C > B.
A beats C because a frozen run is where a CI log is read,
and the line costs one directory read there.
C beats B because B puts a directory walk and a line in front of commands that today hand straight over to `cargo`.

What the read of `0.1.5-alpha.15` adds:
a run on the fast path leaves the journal and says nothing too
(shown in a second crate of the same repository),
as does `cargo-shadow install --frozen-lockfile`.
The journal belongs to the catalog's root,
so a later run in any crate of the repository settles it.

What the round on the refusal's words added
(merged as `7eb08ea6b9`):
it staged a real refusal and made each kind of later run,
and the refusal now states the rule and names the kinds.
A later run of the catalog that checks or updates a lockfile settles the journal,
both dry runs included;
a frozen run,
a run on the fast path,
a run that resolves nothing and a run outside the catalog's repository leave it and print nothing.
The choice between A,
B and C stands as it was.

### A dry run settles a kept journal before it looks

`cargo-shadow install --dry-run` and `update --dry-run` say they report and then restore every `Cargo.lock`.
With a kept journal present each settles it first,
as every run that is not frozen does,
and that can keep or replace a lockfile
(the read of `0.1.5-alpha.15`,
on both releases).

- A,
  a dry run reports the kept journal and what settling it would do,
  and changes nothing:
  the word "dry" then holds,
  at the price of a recovery that can describe without acting,
  and of a report about lockfiles a real run would first change.
- B,
  as now,
  with the help saying so:
  what a dry run reports is true of the settled state,
  and a person who ran it only to look gets a changed lockfile.

Ranking:
A > B,
because a person reaches for a dry run exactly when they do not want anything changed.
The round did both
(merged as `7eb08ea6b9`).
Each dry run's help now says that a kept journal is settled first.
What A would take,
as the round returned it:
a branch beside the frozen one that reads the journals only to report them,
with a recovery that can decide without applying,
no transaction,
and no lockfile touched.
What A would cost a person whose journal waits:
the dry run would name the journal and leave it,
so the habit of running a dry run first to clear it stops working,
and the uncertainty lasts until a run that is not dry.
The round recommends leaving it as it is in this pass,
now that the help says what happens.
The coordinating session's ranking stays A > B for the reason given,
and nothing is built on A without an answer.

### The steps that merge a round's branch are written down and no command does them

The coordinator's merge script lived in scratch and went with the cleanup of 2026-11-27T15:26:37-05:00.
Its steps are in `doc/handover/cli-git-rust-implementation.md`,
with the check that a merge really landed
(a `git merge` once returned 0 with nothing merged).
The owner said on 2026-11-11 of the scripts for Pi runs:
"I don't wanna see you edit pi-runs outside the workspace again."
The coordinator read that as about the scripts that start rounds,
and asks whether it covers this.

- A,
  a task of the driver package (`package/pi-shared/subagent-driver`) that takes branches in order and makes every check:
  no step can be dropped by a session in a hurry,
  at the price of one more command to keep.
- B,
  as now,
  by hand from the handover.

Ranking:
A > B,
because the check that the merge landed exists for a failure that printed success.

### A scope whose directory was never made reads as a collected scope

The retire gate reads a missing directory as the manager having collected the scope,
asks nothing,
prints nothing and leaves the unit loaded;
a directory that was never made answers the same.
A control holds that it happens.
The one run-level occurrence came from a stand-in's race in the read of `0.1.5-alpha.14`.
The fix needs one of two facts
(`doc/troubleshooting/systemd-scope-adopting-an-exiting-pid-stays-running-empty.md`,
"A directory that was never made reads as a collected scope"):

- A,
  the run records the group it made,
  so an absent directory afterwards is a collection and one never recorded is not:
  no question to the manager.
- B,
  the `LoadState` question to the manager,
  which `0.1.5-alpha.12` took out for its cost on every call.

Ranking:
A > B,
because A tells the two cases apart without bringing back the cost that was taken out.
The read of `0.1.5-alpha.15` reached the silent state in a release build,
through a start the manager refuses and through a launcher that is killed,
and did not find a unit left loaded:
in 56 launcher kills and two stand-ins both unit names read not found afterwards.
So no round starts on A until a unit left loaded is shown outside a stand-in's race.

### What each shadow package's copies test compares

The test file is byte-identical in the four crates and names three of them,
so `search-shadow`'s suite fails for a changed copy in any of the four,
and the other three fail only for a change among those three
(measured:
with one line added to `search-shadow`'s copy alone,
three suites passed and its own failed).
The README says exactly that now.

- A,
  as now:
  a copy that changed alone fails the suite of the crate it was changed in,
  which is what the file's own header promises.
- B,
  every suite compares all four:
  a drift is seen from any package,
  and a suite then fails for a change it did not cause.

Ranking:
A > B,
because the change that drifts a copy is always made in some crate,
and that crate's suite fails.

## Picks and open points of the rounds that ended between 14:08 and 14:15 on 2026-11-27 (recorded 2026-11-27T14:24:29-05:00)

None holds a round or a release.

### `cargo-shadow`: an unreadable `populated` line no longer counts as a collected scope (pick, veto open)

The read of `0.1.5-alpha.14` asked for a comment or a check where the retire gate reads a group's `populated` line.
The round found a real input:
the kernel makes a group's directory before its control files,
so a read in that window answers "No such file" for a group that is there
(`kernel/cgroup/cgroup.c` at `v6.1`,
quoted in the round's commit;
read,
not observed).
Alpha.14 read that as the manager having collected the scope and said nothing while the unit stayed loaded.
The round's pick,
merged as `3bf99cd706`:
the run says what it could not read and keeps the unit.
No exit status changes.
The other way is one guard to revert,
which puts alpha.14's silence back.

What the read of `0.1.5-alpha.15` adds
(a same-vendor second pass):
by its reading of the kernel at three versions,
the window does not exist.
A new group's directory and its files are made invisible and become visible together,
so no read sees the directory without its `cgroup.events`.
The answer the kernel can give with the directory there is "No such device",
for a file opened before the group was removed and read after;
there alpha.14's silence was right,
and alpha.15 prints "may be left loaded" about a unit the manager is freeing.
The round's reason was a reading as well,
neither observed anything,
and this host's kernel was not read.
What the measurement found
(the round merged as `92a10aa7dc`,
on this host's kernel `7.3.5-2-cachyos`,
2026-11-27):
the read was right.
Of 3,090 reads made while a group was being created,
none answered "No such file" with the group's directory there,
while the same harness counted 1,640 such answers on a plain directory whose file is written after it.
Of 300 removals,
24 put the step's own read at "No such device",
and the directory was still there in 20 of those.
The method and the counts are in `doc/troubleshooting/systemd-scope-adopting-an-exiting-pid-stays-running-empty.md`,
in the section on which absence means what.
Three picks follow from it,
each with the veto open and each beside its other way.

- "No such device" with the directory there answers as a collected scope (taken),
  as alpha.14 did:
  the kernel removes a group only when it holds no task,
  so no warning is printed about a unit the manager is freeing.
  The cost is that an answer of "No such device" from anything other than a removal,
  which nobody has seen,
  would be read as silence too.
  The other way keeps alpha.15's warning in the one state the kernel was shown to produce.
- "No such file" with the directory there keeps its warning (taken),
  and the records say this kernel is not seen to produce it:
  the same check is the live one at the `pids.peak` read,
  where a group without that file must go to the manager.
  The cost is one branch on the `populated` read that nothing reaches today.
  The other way drops the check,
  and a kernel that did show the state would leave the unit loaded with nothing printed.
- The control of the bound at a run's end allows one second over the promise (taken),
  where it allowed five:
  measured,
  the helper took 5068,
  5073 and 5079 ms,
  and the control now fails when the call site's bound is raised by three seconds.
  The cost is a margin of one second on a busy host.
  The other way lets any raise under five seconds pass.

Ranking,
the round's and the coordinating session's on reading its report whole:
for each of the three,
the way taken > its other way.
The first because it is what the measurement shows;
the second because one check serves both reads and the other read needs it;
the third because a control that passes under the edit it guards protects nothing.
Not measured:
how often a real run's read meets a removal.

### The driver: who makes a run's scratch folder, and who clears one a launcher never reached

Issue 6618 is merged (`a40a99467c`):
the launcher clears a finished run's scratch folder beside `keep/`.
Two points of the record's sentence it left,
each a choice:

- The launcher does not make the folder;
  the brief tells the round to.
  A,
  leave it:
  nothing more to build,
  and a round that never makes one costs nothing.
  B,
  the launcher makes it and names it to the round in its prompt:
  the record's sentence becomes true as written,
  and a round cannot choose another place.
  Ranking:
  B > A,
  because a folder the launcher made is one it can always find and clear,
  where a round that writes somewhere else leaves a folder nothing clears.
- A launcher killed before its unit ends,
  and a run started by hand,
  clear nothing;
  those folders stay until `~/temp` expires them.
  A,
  `remove-finished-worktrees` also clears such folders,
  by the same conditions:
  nothing is left,
  at the price of a second owner of the same folder.
  B,
  leave them to the daily expiry.
  Ranking:
  A > B,
  because the conditions are already one function both can call,
  and a killed launcher is an ordinary event on this machine.

### The linter: a byte order mark is named, not taken off (pick, veto open)

The round for the `0.5.2-alpha.20` read (merged `2261d05b14`) chose that the list reader does not strip a byte order mark:
the message names the character,
its place and the edit that removes it.
Its reason:
a line no other tool reads as a `uses` line would start reading as one here,
and a key a macro prints may hold that character as content.
The other way is to strip one at the start of a list,
which makes a file saved by a Windows editor read with no message.

## Picks the round that joined two `mpm-shadow` rounds made, veto open (recorded 2026-11-27T13:23:53-05:00)

Two rounds of the `0.2.7-alpha.14` pass changed one module,
and their words could not both hold in two places.
The joining round chose (merge `d3f0628c0b`,
released in `0.2.7-alpha.15`),
and each choice is one line or one arm to turn round.

### Which hidden names hold back the copy a kill moved aside

The copy a kill moved aside is put back only when nothing may still hold that version.
One round said a name whose installer cannot be judged does not hold it back;
the other said a name from another process namespace may belong to a live installer this reader cannot see.
The choice:
a hidden name of this boot whose installer cannot be judged
(another process namespace,
a record `/proc` hides,
a start time of zero)
holds the copy back,
and a name in an earlier build's shape does not.
Holding costs no copy:
a later install puts the copy back once nothing may hold the version.
The cost taken,
which the README states:
a live install by an earlier build that holds an earlier-shape name loses the finished copy it holds,
because its own publish does not retry over a name that appeared.
The other way,
one line in `may_still_be_running`:
an earlier-shape name holds the put-back for ever,
since nothing removes such a name,
and the version's name stays empty after every run until a person deletes the hidden name by hand.

What the read of `0.2.7-alpha.15` adds
(2026-11-27T14:35:51-05:00,
a same-vendor second pass):
"Holding costs no copy" is true only while the holder lives.
For an installer that died in another process namespace the hold never ends:
the version's name stays empty on every offline run,
and the deletion `prune --toolchain` advises loses the only finished copy
(shown with a hand-made name;
a real second namespace was not built).
A name in the earlier build's shape is never judged at all,
so after a kill under `0.2.7-alpha.14` its finished copy is neither published nor removed by any command.
A round changes the sentences and the README,
and the rules stay as picked.
The reader's own suggestion,
untested:
hold the put-back while an earlier-shape name's process id and start time name a process running here,
a hold that ends when that process does.

### What becomes of a finished copy of an older version

A recovery never publishes an older version beside a newer installed one.
The choice:
such a finished copy stays under its hidden name only when its marker records the lock's row,
so a later install of that version publishes it with no download;
one whose marker the lock does not vouch for is removed,
since no install can ever publish it;
and a run that holds no row for the archive removes nothing.
The other way,
one arm of `a_finished_copy_nothing_here_publishes`:
keep every such copy.

What the read of `0.2.7-alpha.15` adds:
the removal asks only the running checkout's lock,
where `prune --toolchain` asks every known lock.
So one checkout's install removes a finished copy that another checkout's lock vouches for
(two checkouts that pin one version with two digests),
and the other checkout,
offline,
then cannot install.
A third way follows from it:
keep a copy that any known lock vouches for.
A round returns that as a proposal and changes only the sentence printed.

The round also split two files over the line budget and named the new ones
`the_completion_marker.rs` and `what_a_recovery_answers.rs`;
the names are open to renaming.

## Picks rounds made on 2026-11-27, veto open (recorded 2026-11-27T06:40:00-05:00)

Each is merged.
None needed an answer to proceed,
and each can be undone by one round.

- `process/`'s file list puts `open_file_of_a_descriptor_in_a_child` before `open_file_of_a_descriptor`
  (`356c39c4d7`),
  because `opened_file_of` calls into the child for an overlay mount.
  The target table lists them the other way;
  taking the overlay branch out of `opened_file_of` would put them back,
  and that is a call change.
- `lock/`'s loop of `index_lock_evidence` and `index_lock_evidence_words` closed by deleting a re-export and re-pointing callers,
  not by moving the evidence types into the file of sentences as the plan said
  (`356c39c4d7`).
  The other form renames that file for the types and the sentences together and touches many callers.
- In `push/` (`d356c58a03`):
  `push_once`,
  the plain-thread face nothing in the release calls,
  moved into `auto_push_awaiting.rs`,
  where the alternative moves that file's seventeen borrowed items into a file of their own;
  `STOP_GRACE` moved into `push_deadline_and_patience_settings.rs`;
  and the batch query for a push's range sits in `manual_push_update_and_query_limits.rs`.
- The control for issue 6564 (`a1119838ca`) both asserts either order of the two printed lines
  and holds the answer away so that the place line is said at all.
  With the assertion alone the control could pass with no place line.
- The `source_guard` ban on a control that writes a fixture Git inside the run
  still reads one test file (`invocation/git_version_cache_tests.rs`).
  Issue 6540's round found two more files that do it and held neither to the ban,
  as a change for the owner to approve.
- The window of the control for issue 6546 is still a wall clock,
  sized over the whole run (`eeeee1f44e`).
  A bound the load does not move needs a reader of one child's own process tree,
  which the binary test target does not include.
- The linter's alpha.18 refuses a `uses` line with spacing before its arrow and stops the whole folder rule,
  where spacing after the arrow is only a dead entry.
  The fix round for the alpha.18 pass was told to make the two sides one rule,
  and to prefer a finding on the line over a refusal of the list unless the reader cannot recover.
- A drain's batches are sized by a host its caller states,
  where the drain used to read the machine itself (issue 6567,
  `e22e629637`):
  `drain_pending_awaiting`,
  `drain_the_slow_work_awaiting` and the planned drains take `&HostResources`.
  Nothing test-only entered a signature;
  the callers read the machine once.
  The round measured why the two controls saw one item of two:
  `HostResources::read` takes the control group's free bytes,
  which count the page cache a build wrote inside a memory-capped scope,
  so a scope whose cache fills its cap reads no headroom and every batch falls to one item.
  That is the shipped conservative behaviour for a service unit too,
  and whether the width should count reclaimable cache is a design question the round left open
  (`doc/troubleshooting/cgroup-free-memory-counts-the-page-cache-a-build-wrote.md`).
- A fixture's removal now waits the package's five seconds for what a landing starts beside its loop,
  and names what is left when it fails (issue 6568,
  `291fe5309f`).
  The writer was the thread that draws a shadow ahead,
  five milliseconds after the landing returned.
  Open:
  the two index-lock markers a landing makes for the next batches stay in a control's fixture
  until a service's shutdown or the next landing's take,
  and ending them needs a way that is not a test-only argument of `land_group`;
  and the shadow pool's fill was measured at 5.6 s in a release note,
  longer than the five seconds the removal waits.
- Three place controls of `worktree_copying` (stage,
  journal,
  install) own their process now (issue 6477,
  `d3ea7eaeea`) and can still pass when the bound is not one place,
  because nothing inside them asserts the width.
  Adding the assertion issue 6534's control has is one line each.
- The control for issue 6573 reads every release file of `repo_read/` and `invocation/` for the bounded call,
  where it read one named file (`53d7f739f3`).
  The round's search found no other control made stale by the splits,
  and 58 places where a required literal is absent from the file a pin reads although every such module passes,
  so it built no guard over source pins and left whether one is worth building.
- The control for issue 6572 waits for the record its landing writes
  (`41d82a677e`).
  The release code did what `SPEC.md` says:
  a command may report a landed commit from an open batch before the finished record exists.
  Its new failure message for the bound itself was written and not reached by a run.

## Open: point the session's `git` at the wrapper's `0.6.1-alpha.47`? Recorded 2026-11-26T08:55-05:00

The session's `git` (`.cache/monochromatic-git-policy-cli-bin/git`) is the wrapper built from `ee0cf2268b` on 2026-11-22,
because the AppArmor rules in `/etc/apparmor.d/local/git` and `/etc/apparmor.d/local/gh` name that exact path
and the records say "a rebuilt wrapper requires a fresh review"
(`doc/handover/full-system-policy-handover.md`;
you asked on 2026-11-25 to "Repoint the git link too" back to it after a toolchain install moved it).
That build predates the fix that stops removed worktrees leaving their commit service journals,
so every `git worktree remove` warns and keeps one.
`0.6.1-alpha.47` was released from `68af165851` after a whole gate on `main` at `196b8d0c87` failed nothing
(11991 of 11991 passed);
its x86_64 archive is unpacked at `.cache/wrapper-release-0.6.1-alpha.47/`
with sha256 `aac27ad2eebceda5d03ed0f080b4951cd7271555725700ab851b47f99ace24f7`,
the digest GitHub lists for the release asset,
and its sweep,
run by absolute path,
removed the 201 journals the old build had kept.
Its Sonnet release passes are running.

### Option B: point the link at alpha.47 by its exact path

Add alpha.47's path to `local/git` (`rix`) and `local/gh` (`px -> git`) beside the old lines,
reload,
prove `gh` reaching `git` through a scratch link first,
then repoint the link and keep the old build for a rollback.
Pros:
the session runs what the gate passed,
and removals stop leaving journals.
Cons:
each later release needs the same rule change and review.

### Option A: keep the reviewed build

Pros:
no rule change.
Cons:
the session runs code three days behind `main`,
and alpha.47's sweep must be run by hand after each batch.

### Option C: bump the wrapper's lock row and let `toolchain:install` own the link

Pros:
one mechanism for every tool.
Cons:
the link then points at `.cache/toolchain-launchers/monochromatic-git-policy-cli`,
which the `gh` rule does not name,
the failure recorded in
`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md` ("The wrapper link a toolchain install repoints"),
and every later install moves it again.

Ranking:
B > A > C.
B over A because the session then runs the gated code and stops leaving a journal per removed worktree,
for one reviewed rule change per release;
A over C because C ties the grant to a launcher any toolchain install rewrites,
which is the recorded `gh` failure.

## Open: should local lists of names become slices too? Recorded 2026-11-26T06:41-05:00

Debt ledger item 19 (merged as `df7e6467c5`) turned every constant list of names with a hand-counted length into a slice,
and a source guard now refuses a new one.
It left 297 local `let NAME: [&str; N] = [...]` lists in 184 files,
almost all in test files,
because the item measured constants and no one has said whether locals are in it.
Converting them is the same mechanical change and the guard can be widened to refuse them;
the cost is a large diff across test files the crate split is about to move.
If yes,
the natural time is after the split's moves,
so it is written once on the files' final paths.

## Open: the crate split's points after the rescan of `main`, recorded 2026-11-26T05:05-05:00

The rescan after the reformat (merged as `6329f89af0`) left these for the owner,
each with its options,
pros,
cons and ranking in `doc/planning/cli-git-source-layout-target-tree.md`,
"Open points for the owner".
The split's rounds 1 to 6 of that document's "Next" section need none of them,
so those run meanwhile;
the round each point holds is named beside it.

- The test helpers,
  where the libraries live,
  and plural folder names:
  the document records both the answer "Not yet."
  of 2026-11-07 and an answer of 2026-11-15
  (a separate test-folders library,
  `package/<job>/core-rust`,
  plural names kept with an exemption),
  and takes no side.
  Which of the two stands?
  Holds rounds 10 and 11.
- Which Git-free jobs become libraries (the library count):
  unanswered;
  its readings D and E stand.
  Holds round 11.
- The 32 ZFS folder names,
  and whether the ZFS route's folders form one group.
  Holds round 7.
- The wording of the group rule,
  which the 17 group summaries need.
  Holds round 9.
- Whether `repairs_the_service_runs_for_a_new_worktree` stays a program folder.
  Holds round 7.
- Which file owns the clone-and-settle copy path once `copy_file_exclusive` leaves `worktree_copying/worktree_copy_stage_leaf_copying.rs`:
  the plan sends it to `saving_private_files` (`foundation/private_storage.rs` today),
  but its body reads `worktree_copying`'s file stamps and `subvolume`'s clone,
  and `private_storage` may use neither without new allow-list lines,
  which the list may not gain.
  Split round 1 (2026-11-26T09:10-05:00) stopped that one move and recorded the point under step 2
  of `doc/planning/cli-git-source-layout-target-tree.md`.
  Holds the rest of step 2.

## Open: decisions a read-only audit of all recorded work found, recorded 2026-11-08T16:05:49-05:00

None of these has been asked yet,
and none is holding up running work.
Each record is cited so the context is one read away.

- Pi agents started through the driver no longer inherit a credential-named variable (`9ae27d221e`).
  Two choices follow from it.
  Should mise get its own GitHub token home (`github.credential_command` or `mise token github`),
  since a subagent's `mise` now runs anonymously at 60 requests an hour instead of 4,875?
  Should `SSH_AUTH_SOCK` be allowed through for `ssh m1` probes?
  Nothing has proved it necessary.
  (Interactive Pi sessions keep their environment,
  answered 2026-11-10T08:23:02-05:00.)
- Do the `container-`,
  `windows-` and `test-` record prefixes belong in `cargo-shadow lean`'s list?
  And what should happen to the eighteen logs redirected by hand,
  and the one renamed 17.9 MB record,
  in the linter's verification folder?
- Fuzzing the wrapper:
  may the artifact reference stand without a landed evidence file?
  (Whether the checked-in seeds get a cheap native control in normal tests was answered
  "Leave them to campaigns",
  2026-11-10T08:24:33-05:00.)
- The unmerged copy-in-service branch:
  the audit in `doc/planning/cli-git-copy-route-findings-on-main.md` ranks B over C over A.
  The coordinator picked option B,
  with the veto open:
  port three narrow fixes into `main` and leave the branch as it is,
  neither merged nor deleted.
  The reason is that a kill during a worktree add on the copy route silently loses that worktree's ignored files,
  and recovery can fill a registration the command did not create.
  Two agents are building it.
  A new `git worktree add` from a worktree still being filled now gets no ignored-state copy and one line saying so.
  Two gaps stay open:
  a copy still staging records no destination,
  so it is not detected until the record written before Git is on `main`;
  and the snapshot and clone routes can still carry a half-filled set from a busy source.
- `package/hall-monitor/daemon` fails its own `lint:oxlint` on `test-import(require-eventual-artifact)`,
  two findings before and four now that it has the tests issue 1297 asked for.
  The rule needs an import from a built artifact,
  and that package has none.
  Should it gain a `dist` build,
  or an exemption?
- `mpm-shadow` launcher records named a folder carrying the version,
  so any newer install left them dangling (299 of 1051 records at 2026-11-08T18:17:00-05:00).
  The fix on `fix/mpm-shadow-launcher-records-and-lock-warning` records a path that names no version.
  The records already dangling heal on the launcher's next run,
  which is the coordinator's pick with the veto open;
  the alternative is an `mpm install` sweep over every worktree after the release.
  The newest-installed launcher pick has no `doc/decision/` record yet.
- The umbrella list (`225a3b6318`) has three open points left:
  which of its disagreeing maintenance mutation records is right,
  whether to relink the 16,177 dangling links it counted in 172 of 239 readable worktrees,
  and whether its release-artifact gate also meant a check of the archive's own contents.
  Its Windows and release-gate entries are split in
  `doc/handover/cli-git-rust-implementation.md`,
  where the gate's latency half is answered:
  `native:perf`,
  run against the installed binary,
  measures what mpm installs from the release tarball,
  its limits are its own task description,
  and the release records of `doc/planning/one-release-each-batch-2026-11.md` hold its figures.
  A search of `doc/` and `.github/` on 2026-11-08 for `release-artifact` and `release performance`
  found only items forwarding to that list.
- The questions that end `doc/planning/cli-git-shipped-limits-audit.md`,
  which ask what each shipped limit should cite.
- Does "never discard user data" cover the audit history?
  The timing logs and the push trigger log keep one older generation,
  so a rotation drops the one before it.
- Should `package-paused/dev-script/inference-canary`,
  `package-paused/webapp-edu/paper2vn` and the wrapper's own test runners get the container limits?
- Should the mount unit's stop block guard its `luksClose`?
  A stop with no mapper now fails where the old stop hid the failure
  (`doc/handover/repository-move-dangling-link-items.md`).

Checked and not asked,
because you have already answered them:
deleting the merged branches with nondescript names (done with your approval),
and bumping and publishing ("I'm authorizing you to bump and publish any updates this session.").
The two library layout questions stay at your "Not yet.".

From lines 37014 to 37027 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

Its nine questions for the owner,
not yet asked:
under `trustLockfile = true` with base `HEAD` a committed merge's lockfile is already base,
so cases 1 and 2 do not fire locally (compare with upstream,
check every entry locally,
or keep);
release age versus trust under `trustLockfile`;
CI still warns and runs `--locked` on a refusal;
the window between cargo writing the live lockfile and the checks;
the restore read-then-rename gap;
strict cutoff with exemptions;
`--breaking` sync failure;
a killed `--breaking` run's manifests;
whether the trust line must name the publisher account.

## Coordinator picks made 2026-11-07 with your veto open (not asked yet)

Each is built and in main;
saying nothing keeps it.
- How the 860 move copies each subvolume:
  every first-pass copy is sent against one empty read-only snapshot,
  with every earlier copy as a clone source,
  and only the final pass sends a subvolume as an increment on its own first-pass snapshot.
  The plan it replaced sent a worktree as an increment on a snapshot of `main`,
  which the kernel's send code can pair with the wrong file
  (it treats two inodes as one when their number and creating transaction match,
  `fs/btrfs/send.c` `changed_inode`);
  on the throwaway fixture that plan failed,
  and the new one copied every file exactly and kept sharing.
  Record:
  `doc/troubleshooting/btrfs-send-clone-sources-need-a-parent.md`
  and `doc/planning/repository-on-the-860-drive.md`.
- `monochromatic-lint --fix` now applies only whitespace fixes,
  and content-changing fixes need `--fix-content`
  (0.5.2-alpha.5);
  the repository's `format:monochromatic-lint` task therefore changes no word.
- When the fixer breaks a line inside a blockquote,
  the new line carries no `>`
  (a lazy continuation that renders the same under CommonMark 0.31.2,
  section 5.1,
  rule 2),
  so the fixer never adds a character to your text.
- After the repository move,
  the wrapper checks on every Git command that the repository link still leads to the 860
  (W2,
  merged `b842b34fff`),
  so a dangling link gives a plain message instead of creating things in the wrong place.
  Measured cost:
  about 1.0 ms per forwarded command (median,
  `status`;
  0.8 ms for `rev-parse`),
  beyond the same-binary spread;
  nothing before the move.
  Cheaper forms (skip the check for read-only commands,
  or trust a cached answer across commands) would let a dangling link go unnoticed,
  so the full check stays.
- Tokio step T10
  (the commit service's connections,
  claim renewal and syncs become tasks;
  gate 5313 of 5314,
  not merged yet)
  raised one idle service from 4 threads to 27,
  because the runtime starts one worker per CPU.
  Many services run at once,
  so the coordinator picked a cap on workers,
  measured at 2,
  4 and 8,
  taking the smallest within the measurement spread and never below 2
  (your ruling "Full on tokio.
  Multi-thread.").
- T10 also made the service-memory fault test find no limit at which the service starts but a commit cannot land
  (the service now reserves room for a commit before it starts).
  The agent recommended accepting that,
  which would leave the test testing nothing;
  the coordinator picked instead lowering the running service's address-space limit after it starts,
  so the test still reaches exhaustion in the middle of a landing and checks that the repository recovers.
- The claim-renewal stop still waits without a limit
  (as before T10),
  against your rule that nothing waits forever;
  the coordinator picked a measured ceiling,
  with the claim expiring by its lease if that ceiling passes.
- An equivalent mutant that no test can kill
  (`workspace_count -> 0` in `package/linter/monochromatic-lint/src/rust_file_engine_syntax.rs`,
  a function that already returns 0)
  stays recorded as equivalent,
  so the linter's `--syntax-only` mutation campaign reports one miss every run
  until `mutation:coverage` learns to skip recorded equivalent mutants
  (record:
  `doc/handover/monochromatic-lint-layout-and-workflow-mutants.md`).
- The wrapper's own `markdown/autofix` commit policy keeps applying its one content fix
  (the LFS image link rewrite),
  since that rewrite is the policy's whole purpose.

## Open: lean build folders, after the first run (not asked yet)

Built and run 2026-11-07 to 2026-11-08 (cargo-shadow 0.1.5-alpha.7;
details and per-removal proof in `doc/handover/cargo-shadow-lean-build-folders.md`).
The first run freed 128.1 GiB on the 860,
retired 12 of 17 dated nightlies (52.95 GB to 22.47 GB)
and removed 251 Podman images and 27 volumes;
the daily timer runs at 04:30 and never at login.
- Question:
  should images held only by exited containers be removed too?
  Today a container in any state keeps its image.
- Coordinator picks,
  veto open:
  - an old build tree of a live worktree
    (the folder Cargo used before the toolchain-keyed folders existed)
    is retired after the 7-day idle period once a keyed folder exists for the same workspace;
    without this nothing ever removes those trees,
    and most of the 6,884 trees the first run kept are these;
  - the size caps (E) read the same fingerprint fields as the sweep (B);
    your amendment said "for that sweep only",
    and the agent extended the reader to the caps because they evict by the same units;
  - grace 7 days,
    unidentified trees 21 days,
    run records 14 days and 2 GiB per folder,
    caps 12 GiB in a checkout and 24 GiB in the shared build folder,
    each measured (`doc/planning/lean-build-dirs.md`).
- The timer's pass order and Podman's three refusals are fixed in the sources (merge `a5b31ab7de`);
  the facts are in `doc/handover/cargo-shadow-lean-build-folders.md`,
  and they reach the timer with the next `cargo-shadow` release.

## Open: where a blank at the end of a row goes in the IDE (not asked yet; the owner asked to focus on the Git wrapper)

The text-layout library the IDE uses (Parley) moved to 0.12 in the daily update,
and it now follows the CSS Text 4 rule for blanks at a line end:
an overflowing blank "hangs" past the edge instead of staying on the row and pushing what follows down.
Two IDE tests describe the old behavior and now fail
(`ide::wrap_keys a_hanging_blank_at_the_end_of_the_text_leaves_no_empty_row`,
`ide::wrap_keys the_fixtures_wrap_as_described`);
their fixture text says "an overflowing blank stays on the upper row,
so a tab right after it starts the next row".
Measured at width 240,
scale 1:
a continuation row now starts with a tab (rows `0..28`,
`28..38`),
and no fixture produced an empty tail.
Source:
`parley-0.12.0/src/layout/whitespace.rs` (header doc),
merge `d2598b2842`.
The choice:
keep the IDE's old rule (our code moves the blank back)
or adopt the CSS rule (the tests change).

## Choices the session made itself (non-destructive; say "undo" next to any of them)

- 2026-11-10T04:28:34-05:00,
  issue 613,
  the fuzz coverage harness copied across the sidecars:
  one shared package that keeps both coverage generations,
  the issue's own first-ranked option,
  so no sidecar's baseline changes meaning.
  Say "undo" to take line coverage only and refreeze the baselines.
- 2026-11-10T04:28:34-05:00,
  issue 1355,
  the IDE inspection runs that need `rust-analyzer`:
  each run gets its toolchain for that one command,
  so the host's global `mise` configuration stays as it is.
  Say "undo" to set a global Rust version instead.
- 2026-11-10T04:28:34-05:00,
  issue 1372 of `package/kwin/key-helper`:
  discovery tries the remaining sockets when the newest is stale.
  Say "undo" to keep giving up at the first stale socket.
- 2026-11-10T04:28:34-05:00,
  issue 1369 of `package/kwin/key-helper`:
  region markers go on the whole package and not only on changed files.
  Say "undo" to limit them to changed files.
- 2026-11-10T04:28:34-05:00,
  issue 1673:
  the shared text-scan package also gains the run-scan helper the issue asks for,
  replacing the remaining hand-written scan loops.
  Say "undo" to keep those loops.
- 2026-11-09T19:25:35-05:00,
  issue 1555 of `package/ssg/aquati.cat`:
  a post's dates compare calendar days in the git author's own offset,
  carried through `ResolvedDates` and the build manifest,
  so the answer does not depend on the machine that builds.
  Say "undo" to compare in the build machine's zone instead.
- 2026-11-09T19:25:35-05:00,
  issue 1557 of `package/ssg/aquati.cat`:
  `@lezer/json` joins the catalog and that package so JSON fences keep their highlighting.
  Say "undo" to drop the alias and leave JSON fences unstyled.
- 2026-11-09T19:25:35-05:00,
  issue 1541 of `package/ssg/aquati.cat`:
  an agent writes the alert labels and the brand-image alt in Catalan and Chinese,
  and `lang` goes into the MDX component scope,
  which makes the cached post HTML depend on language.
  Say "undo" to keep English labels on every language.
- 2026-11-09T19:25:35-05:00,
  issue 1549 of `package/ssg/aquati.cat`:
  the dark-theme dropdown shadow is `rgb(0 0 0 / 45%)`,
  and the light theme keeps its earlier alpha.
  Say "undo" to pick another value.
- 2026-11-09T19:25:35-05:00,
  issue 1400 of `package/logger/core`:
  a record buffer holds its page lifecycle listeners only while records wait,
  and no public teardown method was added,
  because nothing calls one today.
  `DECISIONS.md` in that package records the alternative.
  Say "undo" to add `Sink.dispose` and a `Logger` method.
- 2026-11-09T19:25:35-05:00,
  issues 1500 and 1510 of `package/pi-shared/model-review`:
  `assertModelUsesApi` is gone,
  because a table keyed on `model.api` cannot mismatch,
  and the Bedrock and Google dispatch branches get tests through a fake table,
  so `@aws-sdk/client-bedrock-runtime` and `@google/genai` do not become dependencies for tests alone.
  Say "undo" to restore the assertion or add the SDKs.
- 2026-11-09T16:22:51-05:00,
  the merge's second pass becomes awaitable as one body:
  `GitRunner::run` hands back a boxed future,
  and the forwarded `git merge` path keeps one synchronous face that enters the runtime once.
  The owner's "Full on tokio.
  Multi-thread."
  settles that it goes awaited,
  and the alternative is two bodies that must not diverge,
  in the code a person's conflict resolution depends on.
  Both ways are recorded in `doc/planning/git-policy-cli-full-tokio.md`.
  Say "undo" to have awaited twins instead.
  Carried out on the branch `refactor/git-policy-cli-merge-second-pass-awaited`
  at `403254deeb`;
  the awaited push chain stops at the branch move,
  because the recovery of an interrupted merge still reads Git synchronously,
  as that document's step record measures.
- 2026-11-09T16:22:51-05:00,
  all 22 crates that declare a build profile are now covered by `file-enforcer` presets,
  with a new rule for the shape the shadow crates share (`cargo-profile-shadow`),
  so the "Enforced by file-enforcer" comments are true (issue 1641,
  landed as `f4c972b66f`).
  This was the option this file recommended.
  Say "undo" to delete those claims and leave the crates free to drift.
- 2026-11-09T05:12:19-05:00,
  `monochromatic-git-policy-cli fix` refuses a worktree whose Git directory is on another file system,
  before it writes anything,
  because its crash-safe swap needs both names on one file system (`package/git-policy/cli/SPEC.md`,
  on the branch `fix/git-policy-cli-commit-checks-keep-newer-writes-and-modes`).
  This session reads that branch's report as saying the refusal has not been exercised by a test.
  Say "undo" to have `fix` fall back to a copy in that case,
  which loses the guarantee that a kill cannot leave a half-written file.
- 2026-11-09T05:12:19-05:00,
  the publish-list reader in `dependent_version` accepts only the line forms
  `package/config/pnpr/src/generated-config-render.ts` writes and refuses every other line by name and number.
  No YAML crate was added.
  The agent found one viable crate,
  `yaml_serde` 0.10.7,
  and rejected it because a general parser accepts documents the generator never writes,
  which is the class of silent miss three reviews kept finding.
  Say "undo" to have the crate vetted instead.
- 2026-11-09T04:26:58-05:00,
  a shell alias (`!`) stays unguarded.
  Git runs it with its own folder first on `PATH`,
  so the `git` inside it is real Git and not the wrapper
  (the agent's control prints `/usr/libexec/git-core/git` both ways).
  Refusing every `!` alias would break ordinary ones such as a pretty `log`.
- 2026-11-09T04:26:58-05:00,
  the audit line for `git worktree remove` keeps its count of changed tracked files.
  It costs 0.09 s against 0.73 s for the removal on a full checkout of this repository.
  Say "undo" to drop the count and keep the line to two file reads.
- 2026-11-09T04:26:58-05:00,
  the in-process ref listing lands although it is slower here than starting Git,
  by a median of about 9 ms on this repository,
  because its callers are not on the commit path.
  A follow-up makes it cheaper or leaves large repositories to Git.
- 2026-11-09T03:14:38-05:00,
  `mpm-shadow` now refuses a `cargo-git` plan entry that has no `packageId`,
  where it used to take the plan key as the crate name (issue 2626).
  The cross-vendor review points out that an entry whose key equals its crate name worked before and is refused now.
  No entry in `toolchain.plan.jsonc` or `toolchain.lock.json` uses that route,
  so nothing changes today.
- 2026-11-09T02:08:59-05:00,
  `search-shadow link` when something else already sits at a tool's name:
  it leaves the entry in place,
  names it on stderr and exits 0,
  because the rest of the run succeeded.
  It takes over an entry only when the link resolves to this program,
  resolves to a shadow of any version,
  or is spelled `.../search-shadow`,
  the last so an upgrade can replace a link a version prune left dangling.
  A dangling link named `find` that points elsewhere is left alone.
  Say "undo" to make a conflict exit non-zero or refuse the whole run.
- 2026-11-09T02:08:59-05:00,
  the audit `doc/planning/cli-git-tests-that-pass-without-their-race.md` asked whether its weak controls should get
  fixed now.
  They are queued for a fix agent,
  in test code only,
  under your "Fix flaky tests properly."
  Say "undo" to leave it as an audit.
- 2026-11-08T23:47:13-05:00,
  conclusion record on repositories using the files backend:
  a commit no longer writes an empty `conclusion/store-held.json` (main `c13851b2fa`),
  and the new reader accepts its absence.
  A cross-vendor review found one consequence:
  if a commit is killed before its cleanup finishes and a wrapper from before that change recovers it,
  that older wrapper's cleanup fails instead of completing.
  The review established no data loss,
  and the case needs an older wrapper recovering a newer one's transaction.
  Say "undo" to keep writing the empty record.
- 2026-11-08T22:14:46-05:00,
  secret scanner:
  when `forbidden-strings --all` cannot read a repository's index,
  it prints that tracked files hidden by `.gitignore` were not covered,
  and it keeps its exit status,
  so a scan that finds nothing in the visible files still exits 0.
  The reason is your rule "Commits should never fail."
  The cost is that the commit gate and CI pass with less coverage while saying so.
  Say "undo" to make that case a failure.
- 2026-11-08T22:14:46-05:00,
  TOML library:
  a float wrapper holding a numeric string (`{ tomlKind: 'float', value: '1.5' }`) no longer turns into a number;
  it is refused.
  Only one unit test relied on the old behaviour.
  A plain number or bigint with no wrapper is still written unchecked,
  so a very large one can come out as `1e+21` or outside TOML's range;
  that is unchanged.
- 2026-11-05T03:58:47Z,
  program rename:
  this repository's settings file keeps its old name `cli-git.config.jsonc`
  until the release step,
  when what this repository installs switches to the new program.
  Reason:
  the installed wrapper is the old program and reads only the old name,
  so renaming the file earlier would run every commit on `main` with no policies until then;
  the new program reads the old name too,
  with a one-line notice.
  Your "rename them."
  still holds;
  only the timing of this one file moves.
  Done 2026-11-07T08:54:45-05:00 (`c0b4812639`,
  merged in `71498b658f`) once 0.6.0 was installed.
- 2026-11-05T03:58:47Z,
  program rename:
  the link folder `.cache/cli-git-bin` becomes
  `.cache/monochromatic-git-policy-cli-bin` at the release step
  (you said to rename it and gave no name;
  this follows the program's name).
  Done 2026-11-07T09:00:24-05:00 (lock `f44c638269`,
  merged in `71498b658f`).
- 2026-11-05T03:58:47Z,
  program rename:
  before the renamed program is released,
  it must finish any work an old
  background helper left half done (an agent is checking whether that can happen and fixing it if so),
  following "never discard user data".

- 2026-11-05,
  target layout:
  the proposal's technical points are taken as drafted:
  the failure codes stay with the commit service's messages;
  the crash points for tests and the durable index-lock code get their own crates;
  the source-text guard tests move to the program crate;
  sidecar tests move with their code and no new tests are written for the move;
  the two extra call changes (the copy error type,
  the hint functions moving with their users);
  the `include` list goes at the split;
  pins are kept in step by the catalog sync;
  names that fail the "monkey" test are renamed while grouping;
  `commit-command`,
  `doctor-report`,
  `merge-in-isolation`,
  `typed-push-planning` fold into the program,
  and `service-after-commit-work` into the commit service program,
  because no outside program would use them alone;
  `new-worktrees-quickly` is split first after the lower half.
- 2026-11-05:
  the layout rule reads a folder's file-list marker even when it wraps over two lines,
  because the repository's line-break rule demands a break after its comma (merged to `main`;
  854 tests,
  122 in the container;
  the "no file list" help now shows the wrapped form).
  A blank `//!` line or a plain `//` comment inside the marker still means no marker.

- 2026-11-05:
  index files in the new layout lint rules (`mod.rs`,
  and `lib.rs` used as an index) may hold only
  the top doc,
  `mod` lines and re-exports;
  anything else is a finding,
  and a name reached through a re-export counts as a use of the file that defines it.
  Reason:
  your principle "Who depends on who must be immediately visible from looking at the file structure";
  code inside `mod.rs` hides who uses whom.
  Built on branch `feat/monochromatic-lint-index-files-hold-only-declarations` and merged to `main`
  (840 tests pass,
  108 layout tests in the container);
  rules stay off by default.
  On the wrapper it finds no index holding code,
  and no loop was hidden behind a re-export.
  A `main.rs` that is its folder's index may hold `fn main` and its `use` lines as well;
  `extern crate` and an inline `mod` with a body count as code.
  The same check now also covers a `main.rs` beside a `lib.rs` and single-file `src/bin/*.rs` programs
  (merged to `main`;
  842 tests,
  110 in the container).
  A repo-wide survey with the file rule on finds 74 entry-point findings in 15 program files outside the wrapper,
  and 107 index-file findings in 11 files;
  none in the wrapper.
- 2026-11-05:
  after the four lint branches land,
  one more pass removes most closure-lint suppressions
  by giving the called functions a guard or trait form,
  keeping only those that `std::thread::scope` and `catch_unwind` cannot do without,
  and merges the three suppression notes into one note beside the wrapper's code.
  Reason:
  the repository's lint rules say to restructure rather than suppress.
- 2026-11-05:
  the one test function inside `testing/no_threads_child_test_support.rs`
  moved into a `*_tests.rs` sidecar,
  which clears the last `rust/tests-in-sidecars` finding in the wrapper
  (`5983d1603d` on the lint integration branch;
  the probe and a test that relies on it pass).
- 2026-11-05:
  our seven tools will install from their GitHub releases (the `archive` route `mpm-shadow` uses)
  and stop publishing to crates.io after their one release each.
  Reason:
  it is your first pick ("a workspace:
  protocol or something"),
  and the research found it the only route that works both for crates using each other and for installing tools.
  Not started:
  it waits until the layout and lint work land,
  as you asked.
- 2026-11-05:
  the toolchain lock keeps following the newest GitHub release of each tool,
  as `mpm-shadow`'s row does,
  rather than the version written in the checkout;
  that needs no new code.
- 2026-11-05:
  test files stay inside the crates we still publish.
  Reason (`doc/research/tests-inside-published-rust-crates.md`):
  Debian,
  Fedora,
  Gentoo,
  NixOS and the Rust
  project's crater runs all run `cargo test` from a published crate,
  and leaving our sidecar test files out makes that fail with "file not found for module" (E0583)
  unless the sources change too.
  Once the tools stop publishing,
  what is left to publish is small
  (about 153 KiB of test code in `forbidden-strings` and 165 KiB in `forbidden-regex`,
  before compression).

### A landing waiting for the flush of a marker made ahead (coordinator's pick, veto open)

Agent `commit-waiting-path-with-dozens-of-agents` measured landings waiting 9 to 13 seconds
for the index lock's marker made ahead to finish its flush (`lock_stage_kind=pre`),
and asked whether that marker may be published without its flush.
Under the owner's "Please make any non-destructive options (when they are the ones I would likely have picked)
yourself from now on for this session.",
"never discard user data" and "commits vs native git should not take 5s longer",
the coordinator picked neither side of that trade:
the flush stays,
it starts as early as the code allows,
and a landing that finds the marker still flushing takes the lock the ordinary way and never waits for that flush.
The same agent built it (`c72e9f3174`):
the marker of the next batch is started as soon as a landing has its lock,
and a landing that finds the flush still running stages its own,
waiting for nothing,
with the marker left to finish and removed once it is done.
Measured:
`lock_wait_ms` 9,369 and 12,867 with `lock_stage_kind=pre` before,
0 across 46 landings after,
and 8,086 ms of a wrapped commit at the median before against 804 with the host quieter.
Say "undo" to publish without the flush instead.

### Two tokio choices made by an agent (coordinator's pick, veto open)

Agent `tokio-next-step-on-the-runtime`,
building the owner's "Full on tokio.
Multi-thread.",
made two choices on branch `refactor/git-policy-cli-tokio-next-step-on-the-runtime`
and recorded both in `doc/planning/git-policy-cli-full-tokio.md`:

- At the service loop's three state steps,
  a job the system refuses a thread for no longer runs in turn on the awaiting task;
  the loop keeps its state and lets the pool or the next wake write it,
  so no storage call ever runs on a runtime worker.
- The stronger fix for a job that starts after its caller gave up
  (the job hands its value back through a store the caller awaits)
  was not built,
  because it changes the helper every command's blocking work goes through;
  the plan records it as the alternative.

Under the owner's "Please make any non-destructive options (when they are the ones I would likely have picked)
yourself from now on for this session.",
the coordinator keeps both as built,
and the next tokio agent builds on them.
Say "undo" for either one.



### A commit whose command died before its checks ran now stays local, changed 2026-11-09T04:26:58-05:00

A landing moves the branch first and runs the post-commit checks after,
holding the push until those checks approve it.
If the command is killed in that gap,
no verdict exists,
and the commit is on the branch with its push held.

This entry used to say the session would push such a commit,
and asked you to overrule that.
The session has reversed it,
for three reasons.
Cross-vendor reviews keep finding defects in the code that decides a command is gone and releases its push,
each in something added to fix the last
(`${HOME}/temp/agent/reviews/landing-route-round-6-sol.txt`).
The worry behind pushing was a commit held for ever,
and reading the code says that does not happen:
a queued push is `git push <remote> <branch>:<target>`
(`package/git-policy/cli/src/native/push/auto_push.rs:230-245`),
so the next commit's push on that branch carries the held commit.
The branch's agent reproduced that with real Git.
And keeping a commit local can be undone,
while a push of a commit whose secret check never ran cannot.

So the branch `fix/git-policy-cli-landing-recovery-keeps-the-owed-index-update` is being changed:
nothing releases a held push by judging a process,
and the next command in that worktree says in one line that the commit is held and why.
Write "push it" here to overrule this.

The same fact raises a question that does need you,
`What should the next push do with a commit its checks blocked?`.


### Should `add-explicit` judge a pathspec by its spelling or by what it selects?

The session picked the audit's recommendation,
extending the fixed list,
because it starts no process and is non-destructive.
A fix agent is queued for it.
Write "ask Git" here to change that.

The policy exists so `git add` names the paths it stages
(`package/git-policy/cli/src/native/command/rule_add_explicit.rs:1-20`).
It reads the arguments alone and compares each positional pathspec with four
byte strings,
`.`,
`./`,
`*` and `:/`
(`package/git-policy/cli/src/native/command/command_add.rs:113`).
Five spellings that Git resolves to the whole tree pass that list and staged
every file in a reproduction:
`:/*`,
`:/**`,
`:(top)`,
`**` and
`:(glob)**`
(`doc/planning/cli-git-policy-and-candidate-route-findings-on-main.md`,
finding 2).
`:(top)` is the one a person writes expecting a scope rather than a sweep.
A sixth,
`:(top)/*`,
fails inside Git 2.56.0 (`fatal: oops in prep_exclude`) and is reported as
`content-unavailable`,
which is Git's own failure and not this choice.

Recommended:
extend the list with those five and name them in the rejection text,
because the decision stays argument-only and starts no process before Git
does,
and the accidents this policy is about are the common spellings.
The alternative asks Git what the pathspec selects,
which catches every form including ones added to Git later,
at the cost of one Git start per `git add`
and a decision that no longer follows from the arguments.


### Picks made for the owner, veto open

These fall under the owner's "Please make any non-destructive options (when they are the ones I would likely have
picked) yourself from now on for this session."

- Issue 1678 is closed:
  the logger half is fixed on main,
  and the remaining matches of its acceptance regex are `console.*` terminal output,
  which rule `TLG` permits.
- Issue 1396:
  a bare `-0` in `@monochromatic-dev/toml-edit-core` emits `-0.0`,
  so the sign survives a reparse.
- `catalog-tighten`'s `test:matrix` adopts the passing arm of
  `doc/troubleshooting/catalog-tighten-matrix-cannot-start.md`:
  mount the resolved store read-only and take pnpm's
  first version line.
- The first trash hand-over in a repository whose service is not running keeps waiting for the service to start,
  bounded by the commit path's start wait.
  It measured 12.1 s,
  once per repository.
- Whether `node_modules/.monochromatic` can be a subvolume is measured on a throwaway checkout before anything changes:
  it would remove roughly 165,000 of the 269,000 deletions a snapshot's trash costs,
  and nothing yet shows that the tools leave a subvolume in place.
- `MD2` in `AGENTS.md` was reworded by an agent to say the marker width in words,
  with the same meaning.
- `package/test-fixture/data-sequence` keeps a `.` entry serving TypeScript source,
  following issue 1579's acceptance and the `hook-type` precedent.

## Open: two `AGENTS.md` rules proposed after agent failures, recorded 2026-11-19T22:53:13-05:00

Both wait for the owner's word,
because `AGENTS.md` changes are the owner's to accept;
until then every brief the coordinator writes carries the same instruction.
Each code was checked against `AGENTS.md`,
the skills and both forbidden-strings appendixes.

### Stop only what you started

What happened:
at 22:34:24 a ZFS step 4 fix round's agent listed the running containers,
took two that ran `cargo nextest` for its own,
and removed both by ID (`podman rm --force --time 0 7a5c4dce9245 5982b11d2ac9`).
One was another session's whole container gate,
stopped after 8446 of 9199 tests;
the other belonged to a third session.
Rule `NLD` tells an agent to stop every process and unit it left,
and says nothing about how it knows one is its own.
The proposed rule,
under the 50-word and 200-character limits:

```text
SYO:
 Stop or remove only processes, units and containers you started,
 known by a name or ID you recorded at start;
 never one because `ps` or `podman ps` lists it.
```

### Commit dates are the clock's

What happened:
two design writers set `GIT_COMMITTER_DATE` and `GIT_AUTHOR_DATE` on fifteen commits,
one of them to a time not yet reached,
so a revision label could equal the last commit's `%cI` as their briefs asked;
a label written into a commit cannot name that commit's own time.
Rule `WHN` asks for a second-precise timestamp and names `git log --format=%cI` for a commit,
which reads as permission to make the commit match.
The proposed rule:

```text
KDT:
 Never set `GIT_AUTHOR_DATE`, `GIT_COMMITTER_DATE` or `--date`;
 a label naming a commit's time names one already made,
 never the commit that writes the label.
```

## Open: link the already installed tools into the main checkout? (recorded 2026-11-19, about 23:20 -05:00)

The plan's step 1 records that nothing is installed on this host until the mpm-shadow change you put first
("A:
mpm-shadow change first (Recommended)").
Tonight a ZFS step 4 agent ran `mise run toolchain:install` in its worktree,
and it reported `0 installed, 66 unchanged, 0 failed`:
the locked tools are already in `~/.local/share/monochromatic-dev/toolchain`,
`cargo-shadow` and `cargo-nextest` among them,
and the run only made that worktree's `.cache/toolchain-bin` link folder.
The main checkout has no such folder,
so tasks that start with `cargo-shadow` (the wrapper's `native:test:container`,
`test:git-image`) exit 127 there
and every gate runs their second step by hand.
Running the same task in the main checkout would install nothing and make the link folder,
putting those tools on the checkout's `PATH` for every session.

- Link them now (recommended):
  the documented task routes work again and no tool is installed;
  the cost is that every session in the main checkout starts finding these tools at once.
- Wait for the mpm-shadow change:
  nothing on `PATH` changes;
  gates keep running the second step by hand.

Ranking:
link now > wait,
because the record's reason for waiting was an install that converges the whole toolchain,
and the measurement shows nothing left to install.

## Open: two choices in `doctor`'s managed-lock line, for when plan step 18 is wired (recorded 2026-11-25T20:35-05:00)

Plan step 18's managed-lock line in `doctor` is built and unwired
(branch `feat/git-policy-cli-zfs-step-18-doctor-managed-lock-lines`,
`src/native/health/doctor_zfs_route/managed_lock_checks.rs`),
and it prints the sentences step 13 already wrote (`worktree_clone_managed_lock/lock_state_words.rs`).
The round that built it kept two of the design's mappings rather than deciding them in `doctor`,
and they wait here until step 18 is wired,
since no command prints the line before then.

### A lock whose own read was refused

It is folded into the state "locked by words its add did not write",
with the refusal added in parentheses,
as the managed lock design maps it.
The alternative is a sentence of its own,
since a lock nobody could read is not shown to hold someone else's words.

### A lock cut short, and no lock at all

Both are findings,
because the lock the add writes is not standing,
though the next holder of the slot's claim repairs both without a person.
The alternative is to print both as notes;
in the code that is one match arm.

## Answered: approve ZFS step 4's slot identity store format? (recorded 2026-11-20, about 02:00 -05:00)

Answered 2026-11-21T16:24:40-05:00 in the question tool:
"B:
folder,
one file per slot".
Recorded in `doc/decision/cli-git-zfs-slot-identity-store-format.md`;
a design for B goes to review before it is built.
The question as it was put stays here for the record.

What it is:
step 4 records,
for each worktree slot,
the identities the attach checks (device,
inode,
owner,
permissions,
access
lists of the folders),
in one root-controlled file the wrapper appends to,
`git-worktree-clone-slot-identities`,
as
`package/git-policy/cli/SPEC.md` describes under "The recorded identities store".
Step 8 will build on this format,
and no decision record approves it yet;
step 4's second integration pass asked that
you do.

- (a) One file,
  one line per slot,
  appended under a lock only root can open,
  waited for with a bounded deadline
  (as built;
  recommended).
  One read sees the parent and every slot in one state,
  which step 8's checks rely on.
- (b) A folder with one file per slot,
  each made with `O_EXCL`,
  no lock.
  Lock-free,
  which the repository's concurrency
  rule prefers;
  it loses the single-read state and rewrites the format,
  every refusal that names a line and about a
  thousand lines of controls.

Ranking a > b:
a keeps the one-read consistency step 8 needs and its lock covers one appended line,
held only by root;
b buys lock-freedom at the cost of that consistency and a rewrite.

## Answered: should ZFS step 6 refuse a BTF entry wider than `btf-rs` reads? (recorded 2026-11-20T23:40-05:00)

Answered 2026-11-21 in the question tool:
"A:
accept,
revisit later (Recommended)".
Recorded in `doc/decision/cli-git-zfs-runtime-layout-checker.md`,
section "Widened entry fields,
answered 2026-11-21".

What it is:
step 6's start-time check reads the loaded ZFS module's type data through `btf-rs` 2.0.1,
adopted on
2026-11-20T20:18:04-05:00.
That release reads an entry's kind in 5 bits and a record's member count in 16,
so a kernel that widened either field
would be read as the narrower one;
the step 6 integration pass measured a kind-33 entry read as compatible,
where the removed stand-in refused it
(`~/temp/agent/integration-pass-zfs-step-6-with-the-btf-rs-reader-2026-11-20/keep/review.md`,
M1).
No kernel on record uses a wider field,
and the adoption record lists the lag as a revisit trigger.
It does not hold step 6's merge:
a round is making the docs say what the code does.

- (a) Accept the lag as recorded and revisit when a kernel or a `btf-rs` release widens the fields (recommended):
  no new
  code,
  and the gate's other checks stay.
- (b) Add a reader-neutral check to the pre-scan that refuses any entry whose kind or member count exceeds what the reader
  reads:
  keeps the old refusal,
  but it is a second small walker of the entry stream beside the adopted reader.
- (c) Wait for a `btf-rs` release that reads the wider masks before merging step 6.

Ranking a > b > c:
a matches the adoption record and costs nothing while no kernel uses a wider field;
b restores a
refusal at the price of parsing the stream twice,
near what the record declined as an in-house reader;
c holds step 6 on
another project's release.

Also for information,
not a choice:
the record's rule that the request range's two endpoints are compared means a ZFS release that only appends a request,
moving `ZFS_IOC_LAST` with no layout change,
turns the raw route off until the expected range is updated;
it fails safe,
and neither the record nor the design examined that case.

## Answered: ZFS step 3 makes a worktree add from a dirty checkout slower; what then? (recorded 2026-11-20, about 00:30 -05:00)

Answered 2026-11-20 on the third asking,
"A:
merge with cloning on",
recorded as
`doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md`,
which also holds the partial-clone proposal that ends the
waiver and the miss behind it.
The same round chose "Tighten NQH" for `AGENTS.md`,
committed the same day.
This section moves out once step 3 merges.

First asking:
through the question tool on 2026-11-20,
the owner replied "What?
Why are we running checks?".
The coordinator answered that in the session and asked again,
adding a question about cloning only the unchanged files.
The owner's replies,
2026-11-20:
to what step 3 merges with now,
"Then this was an even worse decision made and implies multiple agents missed the rule
about the machine never being quiet."
(read as:
a fast path that needs a clean source was designed and measured as the
main case on a machine whose main checkout other sessions keep dirty;
no option was picked,
so the merge choice is
asked again);
to "Should a writer design and measure a different clone path?"
(clone every tracked file the proof shows unchanged,
let Git write only the changed ones),
"Yes,
design and measure it (Recommended)".
That proposal goes to `doc/planning/` with measurements and comes back to the owner before any code changes.

### Why the clone path checks the source

Git's `worktree add` writes each file from the commit.
The clone path instead clones the files sitting in the source checkout (`FICLONE` of every tracked file) and installs
the source's own index with the clones' stat data (`worktree_clone.rs`,
`install_index_in`),
because a reflink needs a
file on disk with the same bytes and the commit's bytes live compressed in Git's object store.
So the clone path gives the right worktree only when the source's files are exactly the commit's.
Before it clones it proves that (`SPEC.md`,
"Rung 2 needs a clean source"):
no merge,
rebase or other operation in
progress,
the index equal to `HEAD`'s tree,
every tracked file matching its index entry,
and no untracked file
(the refusal kinds in `src/native/worktree/worktree_clean_stat_verdicts.rs`).
Without the proof an uncommitted edit would be cloned into the new worktree,
and the copied index would make its
`git status` call the file clean.
On `main` the wrapper does not know ZFS can clone,
so on this host it goes straight to Git's checkout and never runs
the proof;
step 3 teaches it ZFS,
so the proof now runs,
and on a dirty source it finds the change and falls back to
Git's checkout after spending the time.
The steps measured on that path:
`git ls-files` 79 to 133 ms against the control's 45 to 50,
`git check-ignore` 36 to
47 ms,
`git status` 36 to 57 ms,
and an in-process stat walk of about 34 ms
(`~/temp/agent/zfs-step-3-integration-pass-findings-fixed-2026-11-19/keep/report-from-summary.md`).

What it is:
step 3 teaches the wrapper to see a ZFS dataset as able to share data blocks,
so `git worktree add` from a clean source clones the files instead of checking them out.
Your merge bar says a branch may not make anything `main` does well worse.

What was measured on this machine (24,313 tracked files,
the branch against `main` built the same way,
alternating):

- Clean source:
  the add now takes 0.51 to 1.57 times `main`'s time,
  median 0.66,
  since the round cloned the files in parallel slices
  (before that round it was about 1.9 times,
  which the integration pass rightly refused).
  A source written in the last few seconds can add at most about 0.7 s once,
  because OpenZFS waits for its write-out before cloning freshly written blocks.
- Dirty source (uncommitted changes,
  so no clone happens):
  about 1.2 times `main`'s time,
  about +0.2 s.
  `main` skipped the clone rung's checks on ZFS at once;
  the branch now runs them before it falls back to Git's
  checkout.
  The round tried two ways to make those checks cheaper and neither reached parity.

The main checkout here usually carries uncommitted files (it holds four now),
so adds from it take the dirty path.

What the clone saves on disk,
measured by step 3's guest arm on 2026-11-20 (an encrypted test pool,
the crate's own
`src` as the tree,
2,456 files,
34.7 MB,
Git's add and the wrapper's add alternated over two rounds,
the pool's
allocation read after a sync):
a clean add saved 22,979,072 bytes,
about two thirds of the tree.

Options:

- C.
  Do not take the clone rung on ZFS until the dirty-source checks cost no more than `main`'s,
  then turn it on
  (recommended).
  Both paths behave exactly as `main` does today;
  the recognition,
  its tests and the guest arm stay.
  It
  postpones a part of the accepted design (the clone rung on ZFS),
  which is why it is yours to choose.
- A.
  Keep the clone rung on ZFS and waive the merge bar for this +0.2 s on a dirty-source add,
  recorded in
  `doc/decision/`.
  Clean adds get faster now and save about two thirds of the tree's disk;
  most adds here,
  from a dirty
  checkout,
  get slower by about 1 per cent of a real add (which also copies the ignored files) and save nothing.
- B.
  Run the tracked-file check first and the untracked scan only if it passes:
  the dirty path about 50 ms cheaper,
  the clean path about 34 ms dearer on every host whose snapshot rung is refused;
  still short of parity,
  so the bar is
  still missed.
- D.
  Choose the clone rung by a measured size threshold:
  departs from the ladder's design and rests on a guess.

Ranking:
C > A > B > D.
C over A because the dirty path is the common one here,
so A's cost lands on most adds and
its saving on few,
and C needs no waiver;
A over B because B still misses the bar and moves cost onto other hosts;
B over
D because D leaves the design for a guessed threshold.
The round's report,
with every measurement:
`~/temp/agent/zfs-step-3-integration-pass-findings-fixed-2026-11-19/keep/report-from-summary.md`.

## Moved from the old handover

From lines 46556 to 46556 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The margin above the longest quiet gap for each quiet window (picks 4 and 9 of the limits audit):
  not yet framed with its measurements.

From lines 2755 to 2822 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- Added 2026-11-15T22:34:53-05:00,
  from the run on the automatic push of a branch never pushed
  (tip `c11272e4b6`,
  six controls and no product change:
  a commit on such a branch is pushed to `origin` with its upstream set,
  as `SPEC.md` lines 9204 and 9205 say;
  what pass 53 saw were two `git merge` commands,
  which the trigger declines for a branch with no upstream),
  not blocking work:
  - Should a merge on a branch never pushed push it and set its upstream,
    as a commit there does?
    Today the trigger declines it (`upstream-not-single`) and `SPEC.md` near line 9048 and `README.md` line 556 say so.
  - A declined push says nothing to the person;
    only the trigger's record file explains it.
    Is one line of notice wanted,
    or a README line naming the record file?
  - Three settings the author measured where the rule can be read either way,
    left as built:
    `push.default=nothing` with no upstream and an `origin` (the wrapper pushes;
    plain Git refuses);
    `remote.pushDefault` naming another remote with no `origin` (skipped in silence);
    `branch.<name>.remote` naming another remote with no `.merge` (pushed to `origin`;
    plain Git refuses).
- Added 2026-11-15T02:12:39-05:00,
  from two authors' reports of that hour,
  not blocking work:
  - The review brief's template
    (branch tip `1bcee2933e`,
    in review):
    a review that states "4 of 9 listed files read in full" is counted as covering its change,
    since its statement does not contradict the brief.
    Should a review count only when it states that it read every listed file?
- Added 2026-11-14T22:38:58-05:00,
  from the report of the run on a refused service start
  (`~/temp/agent/controls-of-a-service-start-refused-agree-with-wrapper-2026-11-14/keep/`,
  branch tip `2df2853ed6`,
  in review),
  not blocking work:
  - Before it ends,
    the wrapper now waits up to 250 ms for its log thread to write what was handed to it
    (`WAIT_BEFORE_AN_EXIT`,
    with its measurement beside it:
    a median of 3,160 µs against 3,052 µs on an exit,
    inside the spread).
    Is 250 ms the bound wanted?
- Added 2026-11-14T11:41-05:00,
  each from an author's report of that morning and none blocking work:
  - The commit service started by the wrapper itself (tip `77f5d525e3`):
    does a scope's service that answers after `START_WAIT` get a grace period,
    and is the claim file,
    which is now transient,
    given another name?
  - mpm-shadow's cleanup (tip `3a96da6dbf`):
    are one year for the work limit and one hundred years for the age floor the right largest values?
  - The queue of runs filled from the package,
    its third answer (tip `a634fbf060`,
    added 2026-11-14T17:41-05:00):
    does a put-back finish an interrupted set-aside itself?
  - The commit service that needs no service manager (tip `b896c60d97`,
    added 2026-11-14T15:02-05:00):
    a service that is alive but stopped (a debugger,
    a starved acceptor) reads as recovering,
    and the command ends after up to two minutes;
    is a shorter bound wanted with the same words?
  - mpm-shadow's install removes no version,
    the fresh run's second answer
    (tip `937485686f`,
    added 2026-11-14T19:22-05:00):
    is keeping a version when all 32 claim names are taken the trade wanted?
  - mpm-shadow's cleanup of old versions,
    its fourth answer (tip `b416f1e2b0`,
    added 2026-11-14T16:23-05:00):
    on Windows,
    where no inode is read,
    is the write time alone enough to say an entry is the one judged?
  - mpm-shadow's cleanup of old versions (tip `542b881860`,
    added 2026-11-14T14:48-05:00):
    a `current` link can dangle until the next sync repairs it;
    no store-wide lock is adopted to close that window,
    by the project's rule on concurrency,
    open to veto.
  - The queue of runs filled from the driver package (tip `9a778654e9`,
    added 2026-11-14T13:15-05:00):
    is half an hour the right bound for the notice that the filling has stalled;
    is an entry whose batch gave up waiting for a place at its two-hour deadline put back
    (a fresh run now covers a launcher that fails or gives up)?

From lines 3597 to 3601 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- Whether the hand-written writes to standard error and the commit's reports (about sixty files,
  its row six) go through the door next.
  **Kept for the owner**:
  that would make an error's own text and a command's last words lines that can be dropped after 250 ms,
  which is a different trade from a `debug` line,
  and no record settles it.

From lines 8699 to 8712 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The stop of a stopped push**,
  its author's second run,
  ended 07:12:27 at `2da23e4478`:
  a stop with no worker still reaps its child and judges the locks it left;
  a stop answers inside its bound while the blocking pool cannot sweep,
  and the sweep is recorded as owed with the repositories named.
  Every wait of a stop is listed with its bound in the module's doc.
  Both controls fail with the fix out
  (a zombie;
  300.6 ms against a bound of 100 ms).
  Two new constants,
  50 ms for the reap after the kill and 200 ms for the sweep after the stop,
  are marked by the author as questions for the owner;
  kept.

From lines 11473 to 11487 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The push of a branch that is not checked out**,
  its author's seventh run,
  ended 02:42:04 at `cae78e1d7e`.
  The four findings answered:
  a person's file at a path the repair would write is left alone;
  a failed worktree listing is not read as no worktree;
  the move writes a durable record before `update-ref`,
  swept at every command;
  an index that agrees while the worktree holds the merge is reported as holding.
  Seven controls,
  each quoted failing with its fix out.
  Cost:
  +13.1 ms paired at the median over the rounds without a stall,
  the price of the one durable write,
  kept as a question for the owner.

From lines 31101 to 31108 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

Open,
for the owner:

- A merge whose answer passes the ceiling is admitted whatever its part count,
  and its commit message and journal entry grow with that count
  (35 MB for 103,000 parts).
  Whether the part list keeps a cap,
  and whether a merge that deterministically exceeds the 300-second step
  ceiling should be retried,
  are the owner's decisions;
  today the service retries it.

From lines 34613 to 34616 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  The third finding,
  a `packageId` requirement on `cargo-git` rows,
  stays as it is:
  no entry uses that route and it goes to the owner as a choice.

## Open: should the package.json repository rule also cover paused and deprecated packages? (recorded 2026-11-26T00:32-05:00)

Issue 1124 asked that no generated `package.json` carry the pre-transfer owner `Aquaticat` again.
The `json-keys` rules in `file-enforcer.config.jsonc` own `repository` and `homepage` in every `package/*/*/package.json`,
and `def79174b0` left `package-paused/` and `package-deprecated/` out on purpose:
"a pause means no work",
and the repository already excludes the deprecated folder from every repo-wide pass.
Eight manifests there still carry the pre-transfer URL
(`git grep --files-with-matches 'github.com/Aquaticat/Monochromatic' -- '*package.json'`,
measured 2026-11-26):
`package-deprecated/mcp/nvim`,
and under `package-paused/`:
`dev-script/inference-canary-viewer`,
`dev-script/inference-canary`,
`module/es`,
`pi-plugin/morph-compact`,
`stylesheet/monochromatic`,
`web-demo/glass`,
`webapp-edu/paper2vn`.
So the issue's own acceptance search does not come back empty.
The alpha.2 release pass raised it as a scope question (Low 7).

- Leave them out (recommended):
  a paused or deprecated package gets no work until it is resumed,
  and the rule takes it in on the day it moves back under `package/`;
  the cost is that the eight old URLs stay until then,
  and the acceptance search names them.
- Widen the rule to both folders:
  the old URL leaves every manifest now;
  the cost is that a paused package changes while no work is meant to touch it,
  and the deprecated folder joins a pass it is kept out of everywhere else.

Ranking:
leave them out > widen,
because the folders exist to hold work that is not being done,
and the rule already reaches a package the moment it comes back.

## Veto open: a name of another boot that carries no start time is ended (recorded 2026-11-27T15:23:31-05:00)

The Claude Sonnet read of `mpm-shadow` `0.2.7-alpha.15`
(returned 2026-11-27T14:35:51-05:00)
found that `Owner::liveness` asked whether a hidden name's start time field is zero before it asked which boot the name belongs to.
A run that cannot read its own `/proc/<pid>/stat` record writes that zero,
so a name it wrote under an earlier boot was read as "the system did not say" and never reclaimed,
although no process survives a reboot,
and the README says a name from another boot names an installer that reboot certainly ended.
A round answering that read gave the boot the first word
(merged as `2c6513069b`),
so the code and that sentence now agree.
The round wrote this entry on its branch;
the coordinating session re-broke its lines and named the read on 2026-11-27,
changing nothing it says.

- The boot decides first (taken):
  a name of another boot is ended whatever its start time says,
  so the copy under it is published or removed instead of staying for ever.
  The cost is that a name of a boot this reader is not in is now removed on the strength of the boot alone,
  where before a zero start time held every such name back.
  If `/proc/sys/kernel/random/boot_id` ever answers two values for two live processes of one machine
  (a container that bind-mounts another over it,
  or a kernel that namespaces the file),
  a live installer's directory could be removed.
  The same risk already covers every name with a start time other than zero,
  so this widens a hazard that exists rather than making one.
- The zero decides first (the code before that round):
  such a name is never judged,
  and the README sentence needs the qualification that a name which carries no start time is not placed in any boot.
  The cost is a copy that stays for ever:
  nothing publishes it,
  nothing removes it,
  and `prune --toolchain` prints it for a person to delete by hand.

Ranking,
the coordinating session's on reading the round's entry:
the boot first > the zero first,
because a reboot ends every process whatever a name's start time field holds,
and the hazard the boot's word carries already covers every other name.

Neither way changes the picks recorded at `2026-11-27T13:23:53-05:00`:
a name of another boot never held the copy a kill moved aside,
because it does not carry this boot's identifier,
and it does not become a vouched finished copy either.
What changes is only which `.staging` and `.replaced` directories of another boot with a zero start time a recovery or `prune --toolchain` removes.

## Veto open: what `mpm-shadow` tells a person about a finished copy it cannot judge (recorded 2026-11-27T15:56:07-05:00)

A round answering the same read changed messages and the README and no rule
(merged as `7a216deafc`).
The coordinating session read its report whole and wrote this entry.

An install whose download failed,
while a finished copy of that version sits under a hidden name it cannot judge,
used to print the download's failure alone.
It now goes on:

```text
a finished copy of mpm-shadow 0.2.1 sits under the hidden name <toolchain root>/mpm-shadow/.0.2.1.staging.42.100, whose installer this build cannot judge, so no install of this build publishes it; the version comes back from a download with the network there, or by moving that copy to <toolchain root>/mpm-shadow/0.2.1 by hand, which checks none of its bytes against the lock
```

- Name the move by hand and what it gives up (taken):
  a machine with no network has a way to get the version back,
  and the sentence says the bytes are then unchecked.
  The cost is that the tool itself tells a person how to run bytes the lock did not verify.
- Name the copy and stop there:
  the tool never suggests running unverified bytes.
  The cost is that a person with no network is told a finished copy exists and not what can be done with it.

Ranking,
the coordinating session's:
name the move and what it gives up > name the copy alone,
because the person is the only one left who can judge that copy,
and a sentence that hides the one way out leaves a machine with no network without the version.

## Open: two `mpm-shadow` proposals nobody built, for a finished copy no rule can use today (recorded 2026-11-27T15:56:07-05:00)

Both come from the round merged as `7a216deafc`.
Neither is built,
and each changes a rule an earlier round picked with the owner's veto open,
so nothing starts on either without an answer.

### Publish a finished copy an earlier build left, instead of only naming it

The lock records the archive's SHA-256 and the copy on disk is the unpacked tree,
so a run with no network cannot compare the copy's bytes with the lock's row.

- A tree digest in the marker and in the lock:
  every install writes a digest over the unpacked tree into the marker,
  and `update` writes it into the lock,
  so a copy is judged byte for byte with no network and no other build's word.
  The cost is a change of the lock's format and of the marker's,
  and the markers earlier builds wrote carry neither.
- Trust the earlier build's marker:
  one line of code.
  The cost is that bytes edited since that build's own check are published unchecked.
- Publish only after a person confirms:
  the cost,
  in the round's words,
  is that it "defeats the offline case it is for".
- Leave it (today):
  the copy is named and never published,
  and the person moves it by hand or finds a network.

Ranking,
the round's for the first three and the coordinating session's for the last:
the tree digest > leave it > trust the earlier marker > publish after a confirmation,
because only the digest lets a run with no network judge the bytes;
leaving it runs no unchecked byte,
which trusting the marker does;
and trusting the marker at least serves the machine with no network,
which the round says a confirmation does not.

### Keep a finished copy another known lock vouches for

Today a removal and a publish consult only the running tree's lock row,
and `prune` reads every known lock's versions and never its digests,
so one checkout's install removes a finished copy another checkout's lock vouches for.

- Keep it hidden when any known lock names the marker's digest:
  the other checkout's copy survives,
  which is the only copy a machine with no network has of it.
  The cost is a read of every known lock's artifact rows inside `install`,
  and bytes that no tree's own install may ever run staying on disk with no line printed for them.
  Two trees that pin one version with two archives still do not recover each other's copy.
- Keep the rule and say what the removal took (what the round did):
  a person who needs that copy knows it is gone.
  The cost is that the copy is gone.

Ranking,
the coordinating session's:
keep it hidden > say what the removal took,
because a sentence does not bring a finished copy back,
and the read of the other locks is one `prune` already makes for their versions.

## Open: two words in `mpm-shadow`'s install report that say more than the run read (recorded 2026-11-27T16:19:06-05:00)

The round that made the report's sentences about a leftover say only what the run read
(merged as `a49bf155cd`)
left these as they were and returned them,
because each trades exactness against something a person learns from the word.
Neither holds a release.

### "another install published that version meanwhile"

Printed when this run drops a copy because the version's name already holds a finished one.
What the run read is that a finished copy stood at the name which this run did not put there.

- Say what was read:
  "because a finished copy of that version appeared at its name meanwhile".
  The sentence then claims nothing about who did it;
  the cost is that a person is no longer told the usual writer of that name,
  another install.
- As now:
  the sentence names the one writer the scheme has;
  the cost is that a copy a person put there by hand is called an install's.

Ranking,
the coordinating session's:
say what was read > as now,
because the release read's finding was that these sentences name causes the tool does not know,
and the exact words lose only a guess.

### "the older version that was installed before this run"

Printed when the newest-only rule removes an older version directory this run did not put there.
The rule removes unfinished directories too,
so "installed" is one step past what the run read.

- Say which it was:
  the run reads whether that directory held a finished copy,
  says "installed" when it did,
  and "an unfinished copy that was there before this run" when it did not.
  The cost is one more read of a directory that is about to be removed.
- Say "that was there before this run" for both:
  exact,
  at the cost of the word that tells a person the older version had been working.
- As now.

Ranking,
the coordinating session's:
say which it was > "that was there" > as now,
because the first is exact and keeps the word where it is true,
and the second is exact where today's sentence is not.

## Veto open: three picks of the wrapper round on a process's name in two sentences nothing calls yet (recorded 2026-11-27T16:36:52-05:00)

The round answering the behaviour read of `0.6.1-alpha.50` made its picks and returned each with its other way.
The coordinating session read its report whole and wrote this entry;
the round merged as `e772d8ad66`.
None holds a release.

### A process's recorded name is kept as the kernel gave it, and the sentence escapes it

`CommandWord::of` used to escape control characters itself,
by a rule weaker than the one every other sentence uses,
and the two refusal sentences printed its text raw inside parentheses.

- One escape and at the sentence (taken):
  the record keeps the kernel's bytes,
  cut to the kernel's own length,
  and each sentence shows the name through the one function every other sentence uses.
  The cost is a changed behaviour of a name another crate can reach:
  a caller that relied on the record's text being escaped now gets the bytes.
- Escape in the record as well:
  the record's text stays safe to print as it is.
  The cost is a name escaped twice once a sentence shows it,
  so a line feed reads as an escaped backslash and letters,
  or a record with two texts and two escapers again.

Ranking,
the round's and the coordinating session's:
one escape at the sentence > an escape in the record as well,
because the repository's rule is to encode at the final interpolation,
and two escapers were the finding.
The crate is not on crates.io at this version,
so the changed behaviour is a design change and breaks no published contract.

### A record written as JSON leaves a bidirectional control and U+2028 as the name's bytes

`SPEC.md` said every line that names a process escapes everything outside printable ASCII.
A JSON record is escaped by JSON's own rule,
which leaves those characters as bytes.

- Say so in `SPEC.md` and leave the record (taken):
  the record is data a program reads,
  and its bytes compare equal with the kernel's.
  The cost is that a person who prints such a record in a terminal sees those characters act.
- Escape them in the record too:
  the record is safe to print.
  The cost is a change of the bytes of every record that holds a name outside ASCII,
  and of every reader that compares them.

Ranking,
the round's and the coordinating session's:
leave the record > escape in the record,
because the reader of a record is a program,
and the line rule already covers every sentence a person is shown.

### The sentence about an unusable identity file is remembered for the process and never reset

It is said once for a command because a process is one command.
The round showed from the source,
and from one run of a serving process,
that the commit service and the clone mount handler's modes never read the identity file:
`main.rs` chooses each before the wrapper's entry runs.
That run served its socket and no commit went through it,
because a fixture has no service manager.

- Leave it (taken):
  the memory cannot outlive a command today.
  The cost is that a later change which makes a serving process read the file would say the sentence once for ever,
  and no test can show that yet.
- Reset it per request now:
  safe against that later change.
  The cost is code nothing can exercise.

Ranking,
the round's and the coordinating session's:
leave it > reset it now,
because a reset no run reaches is held by no control.

## Veto open: how long `cargo-shadow` asks its route question again, and the release that waits on it (recorded 2026-11-27T17:05:55-05:00)

The round of issue 6623 merged as `e595486ec8`.
The coordinating session read its report whole and wrote this entry.

### The asking again is sized to a measured reexec

- Five asks,
  200 ms apart,
  spanning 0.800 s (taken):
  it covers the longest reexec of a user manager across which asks were measured to fail,
  0.588 s,
  and a control that reexecutes a container's own manager passes with it.
  The cost is paid where no user manager can be reached at all:
  every call makes five asks and takes 883.1 ms at the median,
  against 43.2 ms and one ask for `0.1.5-alpha.15`.
- Three asks,
  50 ms apart,
  as before that round:
  the cost where no manager answers is about a tenth of a second.
  The same control failed with it in a container,
  142 ms into a real reexec.

Ranking,
the round's and the coordinating session's:
five asks 200 ms apart > three asks 50 ms apart,
because only the first rides through what was measured.
Its cost has a round of its own,
issue 6629:
a call with no manager to reach asks once.

### `0.1.5-alpha.16` waits for issue 6629's round

- Wait (taken by the coordinating session):
  the release and its lock would put the five asks into every call where no manager answers,
  and the repository's workflows install the locked binary.
  The cost is that this machine keeps `0.1.5-alpha.15` for as long as the round takes,
  in which one failed ask still reads as a host with no delegated control group.
- Release now and again after the round:
  this machine gets the asking again at once.
  The cost is that wait on every such call until the next release.

Ranking,
the coordinating session's:
wait > release now,
because the wait is one round long and the cost of releasing lands on every call of a kind.

That wait is over:
the round merged as `aa7b4eb927` at 2026-11-27T17:49:46-05:00,
a call with no user manager to reach makes one ask again,
and the release started after it.
One thing the round left:
a host whose manager died and left its socket file or its directory behind is still asked five times.

- Leave it (taken):
  the cost is the five asks on such a host and nothing anywhere else.
- Tell a dead socket from a manager that is reexecuting:
  such a host would be asked once.
  The cost is a rule nobody has found:
  in the round's sampling a reexec answered `Connection refused` in 393 of 399 refused asks,
  which is what a socket file with nothing listening answers too,
  and a wrong guess there refuses a call across a reexec.

Ranking,
the round's and the coordinating session's:
leave it > tell them apart,
because the second risks the failure the asking again exists to prevent.
Issue 6629 stays open for it.

### What stays unexplained

The ten failures of 2026-11-27 came after the reexec had ended,
and no measurement reproduced them.
Issue 6623 was closed by its commit's `Closes` line at the merge;
a comment on that commit says `Refs` is the accurate word,
and issue 6628 holds the event.

## Veto open: four picks of the linter round on `file-uses` lines and characters a reader cannot see (recorded 2026-11-27T17:05:55-05:00)

The round for the read of `0.5.2-alpha.21` merged as `83d10dd296`.
The coordinating session read its report whole and wrote this entry.
None holds `0.5.2-alpha.22`.

### A line whose words name a folder is refused by naming that folder, for every spelling of its gaps

- For every spelling (taken):
  one folder gets one sentence,
  which is what the read asked.
  The cost is that a doubled gap,
  which re-spacing would mend,
  is answered with a sentence about the folder's name.
- The word-count sentence for the gaps a person can re-space,
  and the folder's name only where no re-spacing gives a readable line:
  the smaller edit is named.
  The cost is two sentences for one folder in a list aligned by hand.

Ranking,
the coordinating session's,
since the round gave costs and no order:
for every spelling > split by gap,
because the read's finding was that one folder drew different sentences.

### A doubled gap beside a name is read as the name without the gap's whitespace

A name that begins with whitespace would be read as the name without it.
The round's reason for leaving it:
no tree the layout rules accept can hold such a name,
so the choice changes a message and no verdict.
No other way was built.

### `AllowEntry` gains a public field, `column`

Code outside this repository that builds an `AllowEntry` must pass it.
Nothing in the repository does,
and the crate is a pre-release,
so this is a design change.

### The set of characters is tied to Unicode 17.0.0

The ranges beyond the general categories are written beside the data file they come from,
and a control fails when the linked tables move to another release.
The other way was a crate that answers the property directly:
one call in place of the ranges,
at the cost of that crate's own Unicode release beside the one the categories come from.

## Veto open: the picks of two more wrapper rounds for the `0.6.1-alpha.50` reads (recorded 2026-11-27T17:05:55-05:00)

The coordinating session read both reports whole and wrote this entry.
None holds a release.

### No resolved merge answer holds a copy of a line a side deleted

The round merged as `85be73acf1`.

- The copies bound for every shape of region (taken):
  no answer the word pass resolves holds more copies of a line the base held than the two sides leave.
  The cost,
  on the read's 2,300 seed cases:
  21 answers that resolved are conflicts now,
  each of which broke the bound,
  and a refused file keeps both sides of every region the sides disagree about.
- Leave the code and state the exception in `README.md`:
  21 more answers resolve.
  The cost is text a side removed landing with no marker.
- The bound read for each region alone,
  not taken:
  it refuses right answers,
  where one region holds more copies than its own two sides leave and another holds fewer.

Ranking,
the round's and the coordinating session's:
every shape > the stated exception > each region alone,
because the exception is the loss issue 6529 exists to prevent,
and the per-region reading refuses answers that are right.
The seeds are synthetic and repetitive,
so neither figure is a rate for real files.
Where both sides wrote more copies of one line than the base held,
nothing judges the answer;
the round left that as its brief said.

### Ten names and one alias leave the crate's published surface

The round merged as `35fdc1797c`.
The widened control saw ten names `0.6.1-alpha.50` published that nothing outside their module reads:
seven associated constants,
and three fields of one struct.

- Narrow them (taken):
  each takes the reach its readers need,
  and an alias with no reader at all is deleted.
  The cost is that a consumer outside this repository that names one stops compiling;
  the round found none in the binary,
  the fuzz sidecar,
  the end-to-end folder,
  the integration targets
  or any branch on origin.
- Give each a line in the list:
  the surface stays as `0.6.1-alpha.50` published it.
  The cost is ten published names nobody chose to publish.

Ranking,
the round's and the coordinating session's:
narrow > list,
because the crate is not on crates.io,
so a narrower surface is a design change and breaks no published contract.

## Veto open: two things `mpm-shadow` `0.2.7-alpha.16` does that no record chose (recorded 2026-11-27T17:12:22-05:00)

The second pass over `0.2.7-alpha.16` found both,
with both released binaries on fixtures.
The coordinating session read it whole and wrote this entry.
A round held each with a control and wrote it into the README,
merged as `d46572181c`;
neither behaviour changes before the owner says so.

### A recovery that puts an unfinished copy back also points `current` at the newest finished version

The state:
an older version finished,
`current` absent,
and a newer version whose copy a kill left unfinished and no lock vouches for.

- As `0.2.7-alpha.16` does (kept for now):
  `current` names the older version and the launcher starts it,
  while `list` says drift and `install` ended 1.
  A person's tool works.
  The cost is that it is silently the older one.
- As `0.2.7-alpha.15` did:
  `current` stays absent,
  and the launcher fails and says to run the install.
  The cost is a tool that does not start although a finished version is on disk.

Ranking,
the coordinating session's:
as alpha.16 does > as alpha.15 did,
because `list` and the install's own exit status already say the newer version is missing,
and a start that fails helps nobody who has a working older copy.

### A move of `current` that fails once ends the run 1, and the older version stays

One failed rename of the link:
nothing is removed,
the run reports the failure and ends 1,
a later step of the same run moves the link,
and the older version stays until `prune --toolchain`.
The README says a run ends 1 only when the locked version is not at its name afterwards,
which is not so here.

- The run makes up for it when its later step moves the link:
  the removal runs then and the first failure is a warning.
  The cost is a second place that removes older versions.
- The behaviour stays and the README says what happens:
  no code changes.
  The cost is a status 1 for a run that left everything in place,
  and a retired version on disk until a `prune`.

The round kept the behaviour and wrote it into the README (merged as `d46572181c`),
with a control that holds it.
Its argument is from the code:
the later step moves the link of whichever toolchain root wins,
which can be the other root,
so a removal tied to that step could remove under this root's link that has not moved.
A make-up would need a second move of this same link,
would still end 1 if that failed too,
and would have to leave a plain install of an already-installed tool alone.

Ranking,
the round's and the coordinating session's:
the behaviour stays > the run makes up for it,
because the removal must never run under a link that still names the version going away,
and the cost of staying is one status 1 and one `prune`.

## Veto open: two picks of the wrapper round on the list guards (recorded 2026-11-27T17:12:22-05:00)

The coordinating session read the report whole and wrote this entry.
None holds a release.

### An excuse names the line of the one use it covers

- The line (taken):
  a use removed and another of the same text added to the same file fails,
  which a count let through.
  The cost is that a line `rustfmt` reflows makes its entry stale,
  and the entry and the reflow become one commit's work.
- A count of uses per file,
  as before:
  a reflow costs nothing.
  The cost is the swap the read found.

Ranking,
the round's and the coordinating session's:
the line > a count,
because the swap is the finding,
and one guard of the crate already named its lines.

### `Command::new(r` stays a banned text

It also matches a start through a variable whose name begins with that letter.
No file of the commit path holds one today,
so the round left the ban;
a rule that reads a literal argument would be narrower and is not built.

## Veto open: three picks inside the router the owner chose for the writer's list (recorded 2026-11-27T17:30:22-05:00)

The owner chose that the router pass over a member too small for a request,
and that a pool advertise its head member's window.
The round that built it merged as `45b9aaeb94`,
and the built copy Pi loads was rebuilt at 2026-11-27T17:28:33-05:00,
so it is live.
The coordinating session read its report whole and wrote this entry.

### A member is too small when the request's input alone exceeds its window

- The input alone (taken):
  Pi cuts the answer's limit to what the window leaves,
  so a request whose input fits is served.
  The cost is a short answer near the edge:
  the round's case leaves 10,875 tokens of answer for an input of 509,317 in a window of 524,288.
- The input plus the member's output limit:
  no answer is cut short.
  The cost is that a request Pi can serve is passed over.

Ranking,
the round's and the coordinating session's:
the input alone > the input plus the output limit,
because passing a member over refuses a request it can answer.

### No margin beside the estimate

- None (taken):
  the estimate is Pi's own,
  and over 48,805 requests in 200 sessions it stood within 0.1% to 0.2% of the count the member reported,
  at the 5th and the 95th percentile.
  A request the estimate misjudges ends in the member's own rejection,
  after which Pi compacts and retries the turn once,
  as the owner's chosen option said.
- A margin of a few percent:
  fewer such rejections.
  The cost is a member passed over for a request it can hold;
  0.6% of the requests in untouched sessions stood 5% or more from their member's count.

Ranking,
the round's and the coordinating session's:
none > a margin,
because the tail is rare and already has a recovery,
and a margin costs every request near the edge.

### The head is the first member the session's registry resolves

Where the first member of the list is not registered,
the pool advertises the next member's limits,
which is 524,288 and 65,536 where Qwen's member is missing.
The other way would keep the list's first member's limits whether or not it is registered,
and advertise a window no member of the session has.
No ranking is needed:
the second is wrong for such a session.

## Veto open: the Rust toolchain row no longer declares `rustup`, and what its two format programs need (recorded 2026-11-27T17:30:22-05:00)

The round of issue 6626 merged as `7aa3e5e61f`.
The coordinating session read its report whole and wrote this entry.

### The row stops declaring `rustup`

- Stop declaring it (taken):
  `rustup` is the manager,
  no toolchain holds it,
  and nothing in the repository's tasks,
  scripts or workflows reaches it through the exposure folder.
  An install now names no unresolved bin on this machine.
- Teach the route to link the manager's own program:
  the folder would hold a `rustup`.
  The cost is a link into a program this tool neither installs nor updates,
  whose place differs by machine,
  and which would shadow the manager for every task.

Ranking,
the round's and the coordinating session's:
stop declaring it > link the manager,
because nothing needs the link.

### The row declares `cargo-fmt` and `rustfmt`, and no component it declares installs them

Both resolve on this machine only because the `rustfmt` component was added by hand on 2026-11-25.
A machine that follows the plan has neither,
and a task that runs `rustfmt` finds none.
This is issue 6630.

- Declare the `rustfmt` component in the row,
  which is the coordinating session's pick and what a round builds:
  the row's bins resolve on any machine.
  The cost is one more component download where it is missing,
  and nothing here.
- Stop declaring the two bins:
  no download.
  The cost is that the tasks that run `rustfmt` or `cargo fmt` depend on some other copy on `PATH`,
  which this machine does not have.

Ranking,
the round's and the coordinating session's:
declare the component > drop the bins,
because the repository's own tasks run both programs.

## Veto open: two things the round on the `final-newline` hint left as they are (recorded 2026-11-27T17:48:05-05:00)

The round of issue 6627 merged as `0f69dbfe93`.
The coordinating session read its report whole and wrote this entry.
None holds a release.

### `finding_hint` takes a fourth positional parameter

The repository's rule asks a function of two or more parameters to take one object.

- A fourth positional parameter (taken):
  it follows the neighbouring functions of the crate,
  and publishes three names.
- One object:
  the rule is kept.
  The cost is five more published names,
  by the round's count.

Ranking,
the round's:
the positional parameter > the object,
because the crate's own functions around it are positional.
The coordinating session adds no ranking:
the rule is written for TypeScript,
and this is Rust.

### A landed commit and a manual push still say to edit the file and stage it again

That is a true next step and not the only one:
the `fix` command corrects the worktree copy,
and the person still commits it.
The issue asked that the sentence stay only where nothing corrects the bytes,
which is what the round did;
saying more there was not built.

## Veto open: three things the round on the driver's watch cases chose (recorded 2026-11-27T18:11:08-05:00)

The round of issue 6631 merged as `ae937863bc`.
The coordinating session read its report whole and wrote this entry.
None holds a release.

### How long a case waits for its event

A case of the driver's suite waited three polls,
900 ms,
for an event that takes two looks,
and one whole run of the suite left it 177 ms of room.

- A bound from the looks a case needs (taken):
  each look is counted with the longest wait a look can be handed,
  so no loaded host decides these cases.
  The cost:
  a watch that reaches its event only after seconds still passes,
  and one that never reaches it is reported after as long as 181.8 s,
  where it took about a second.
- A clock each case owns,
  as `wait-for-an-answer.unit.test.ts` has:
  the case ends at its event with no figure in milliseconds at all,
  and a slow watch is caught.
  The cost is a stand-in clock and its driver in twenty-one cases,
  and the record's silence window reads that clock too.

Ranking,
the coordinating session's:
the owned clock > the bound from looks,
because it catches a slow watch and the bound does not.
The round built the bound because it is the smaller change to a file it only had to make hold.

### Two cases that assert the wall clock keep their poll counts

One of them needs its first look before its clock,
so a timer that fires about 285 ms late still fails it.

- Leave them (taken):
  no failure of either is on record.
- Give those two the owned clock:
  the exposure goes.
  The cost is a round on two cases.

Ranking,
the coordinating session's:
the owned clock > leave them,
for the same reason,
and the two questions are one round if the first is taken.

### Where the cause is written down

The round wrote it into `doc/troubleshooting/testing.md`,
as the next cause of a bug whose other causes are written there.
`AGENTS.md` asks that a fact about one package go in that package's docs.

- Leave it beside the bug's other causes (taken).
- Move the whole bug,
  with every cause,
  into the driver's own docs:
  the rule is kept and the causes stay together.
- Move this cause alone:
  the rule is kept for it and the bug is split over two files.

Ranking,
the coordinating session's:
move the whole bug > leave it > move this cause alone,
because the first keeps the rule and the record whole,
and the last trades one for the other.

## Veto open: whether a command the linter prints may spell a name it must hold raw (recorded 2026-11-27T18:17:40-05:00)

The Claude Sonnet read of the linter's `0.5.2-alpha.22` found that the loop message,
the missing-link message and the command that message gives
(`mkdir --parents <folder>/uses && ln --symbolic ...`)
print a folder's name raw when it holds a character that prints as nothing,
a right-to-left override among them.
The list reader's own messages spell such a character since alpha.22.
No round builds any of this yet;
the read's other findings went to a round.

- Sentences spell the name,
  and a command keeps it raw and says in words that it holds such a character (the coordinating session's pick):
  the command works when copied.
  The cost:
  the raw character still reaches the terminal inside the command's text.
- Spell the name everywhere,
  commands too:
  nothing raw reaches a terminal.
  The cost:
  the copied command names a folder that is not there,
  so the message has to say how to type the real name.
- Leave the messages as they are:
  no work.
  The cost:
  the module's own comment says every message spells such a character,
  which is then not so.

Ranking,
the coordinating session's:
sentences spell and commands stay raw > spell everywhere > leave them,
because the first keeps the command usable and the second gives a command that fails,
and because the second still tells a person what the name holds where the third prints it unseen.

The Braille blank,
U+2800,
prints as an empty cell and is printed raw by both releases:
Unicode does not count it among the characters that print as nothing.
The pick is to leave it;
the other way is to add it to the set by hand,
which ties the set to a list of the linter's own beside Unicode's.

## Veto open: what four rounds of 2026-11-27 chose or left (recorded 2026-11-27T18:42:35-05:00)

The coordinating session read each round's report whole and wrote this entry.
None holds a release.

### The wrapper: a slot command behind a shell alias that could not be placed is refused

When Git cannot be asked where a `!` alias's shell runs,
a removal,
a move or a prune of a worktree behind that alias was forwarded with no refusal.
The round on the alias reading (merged as `461d441ef7`) refuses it,
with words saying what could not be read.

- Refuse it (taken):
  a bound slot keeps its protection.
  The cost:
  such a command is refused whenever one start of Git fails,
  also in a repository with no record at all,
  and the person types it again without the alias.
- Forward it and print a line:
  nothing is refused that Git would have run.
  The cost:
  a removal of a bound slot deletes the registration and the pointer under the clone.

Ranking,
the round's and the coordinating session's:
refuse > forward,
because the owner's answers of 2026-11-26 refuse every slot command the wrapper cannot prove safe.

### The wrapper: the guard in that same failure state

When that ask fails,
the guard judges the caller's own worktree.
For an alias whose own words name no other place that is the right worktree.
For one whose words carry `-C`,
`--git-dir` or `--work-tree` it is the wrong one.

- Leave it (taken):
  the guard refuses only what it can prove.
  The cost:
  in that failure state a guarded command aimed at another worktree is judged for the caller's.
- Refuse a guarded command behind such an alias when the ask fails:
  the hole closes.
  The cost:
  a false refusal of every such command whenever one start of Git fails,
  and no record says the guard may refuse what it cannot place.

Ranking,
the coordinating session's:
refuse > leave it,
because a command judged for the wrong worktree is the failure this release set out to end,
and a refusal can be typed again.
The round built neither way of its own accord and left it as it was.

### The wrapper: one more line in the folder rule's allow list, and two more files over the line budget

The plan says the allow list may only shrink
and that a split inside the `invocation/` folder waits for the crate's move.
That round made one new file there to keep a file under the budget,
with one `group-file` line for it,
and left two other files over the budget,
at 313 and 305 code lines.

- As the round did (taken):
  one file stays under the budget at the cost of one line the move deletes.
- Take the line out and leave that file over the budget too,
  with the folder's other nine:
  the list does not grow.
  The cost:
  one more file over the budget until the move.

Ranking,
the coordinating session's:
take the line out > as the round did,
because the plan's rule is that the list only shrinks and the round broke it for one file while keeping it for two.
Nothing is built either way until the owner says.

### The scanner of forbidden strings: what a link given by name means

The workflow no longer hands the scanner a link whose target is a folder
(issue 6632,
merged as `fc1c84b58e`),
and a round now makes it hand over each such link's own name and target text through an option the released scanner has
(issue 6633).
Inside the scanner a link given by name is still followed:
a link to a file is read through,
and a link to a folder is a read error.
Each way below needs a release of the scanner,
which is not an alpha and is asked first.

- Leave it (taken):
  no release.
  The cost:
  any other caller that lists a folder link gets one read error for it.
- Leave links out of named arguments as the walk does:
  no read error.
  The cost:
  a link to a file is no longer read through.
- Scan a link's own name and target text and never follow it:
  the most is inspected.
  The cost:
  a new output shape in a security gate,
  and a link to a file is no longer read through.

Ranking,
the coordinating session's:
scan name and target text > leave it > leave links out,
because the first inspects what Git stores and follows nothing outside the repository,
and the last inspects least.

### The linter: when its read, fix and release loop ends

Twelve alphas of the linter were released on 2026-11-26 and 2026-11-27,
by the count of its release commits,
each read by a second pass,
each read answered by a fix round and the next alpha.
The read of `0.5.2-alpha.22` found no High and no Medium finding,
and nine Low ones on folder names that hold tabs,
arrows or characters that print as nothing.
A fix round for seven of them runs.

- Stop reading each alpha once a read finds no High and no Medium
  (the coordinating session's pick for what follows `0.5.2-alpha.23`):
  the alpha is still released and locked,
  and the next read waits for a change that is more than an answer to a read.
- Go on as now:
  every alpha is read.
  The cost:
  each read finds a handful of Low points on names no repository here holds,
  and each fix adds reading logic the next read probes.

Ranking,
the coordinating session's:
stop at a read with no High and no Medium > go on,
because the findings no longer change what a person using the linter here meets.
Until the owner says,
`0.5.2-alpha.23` is read as the others were.

### `cargo-shadow`: how much room its module list has

`src/lib.rs` holds 201 code lines against the budget of 300,
after 52 test files were declared by the modules they test (merged as `60ecd08eef`).
Thirty test files that belong to no one module stay declared there.

- Leave them (taken):
  99 lines of room.
- Group those thirty under one parent module:
  about 149 code lines.
  The cost:
  26 more test paths change,
  for a grouping the suites do not have.

Ranking,
the round's:
leave them > group them,
because the room is enough for 99 more modules.

## Veto open: what the rounds read on the evening of 2026-11-27 chose or left (recorded 2026-11-27T19:13:58-05:00)

Each entry is a choice the coordinating session made,
or one a round's report left,
with the way not taken.
None holds work:
each stands unless the owner says otherwise.

### `mpm-shadow`: where the Rust row's programs come from

The Claude Sonnet second pass of `0.2.7-alpha.17` found that the folder of links (`.cache/toolchain-bin`)
takes each program of the Rust row from the first toolchain in name order that holds it,
not from the pinned one
(issue 6634).
This machine's folder points at the pin today,
because the pin's name sorts first;
it stops doing so when the pin moves to a later date while the older toolchain stays.
The fix round was given these picks as its own to show wrong:

- the row's programs come from the toolchain the row names and from no other;
- the rebuild removes a symbolic link the folder holds for a name no row declares,
  never a regular file,
  and the install's report says which it removed;
- `list` reports a declared component or target the toolchain lacks as drift,
  since `install` would act on it.

The way not taken for the second pick is today's:
the rebuild only ever adds links,
so nothing in the folder is deleted,
at the cost of a link that dangles or serves a program no row asks for.
The round lists what the rebuild would remove from a folder like this machine's,
and that list is read before `0.2.7-alpha.18` is installed here.

### `mpm-shadow`: what a warning and a `prune` line may say

The same pass found a warning that promises a publish the next run does not make,
and two sentences that call a version whose program is cut short the holder of "a second copy".
The picks given to that round:
each warning says what the next run does for the kind of failure that reached it;
the rule for a finished version
(a marker and a program found,
with no size or digest)
stays,
as the round before priced it,
and the two sentences end at what the run read.
The way not taken:
check a program's size or digest before calling a version finished,
which costs every install a read of every program.

### The store fixture: what the round of issue 6410 left

- The branch's history holds merge commits with no `Refs` line,
  and commits that repeat one tree,
  made by the pusher before a message was amended.
  Pick:
  leave them,
  since removing them rewrites pushed history.
- The default deadline of 2,000 ms.
  It failed four calls falsely under a small cap,
  and a recorded disk stall on this machine is 2,564 ms.
  Pick:
  size it from a measurement under the caps its users run in,
  which a round is doing now;
  the way not taken is to keep 2,000 ms and call the small cap the fault.
- The tool runs in a process group of its own,
  so that a helper it leaves behind stops with it at the deadline.
  The cost:
  an interrupt typed at a terminal no longer reaches the tool,
  which then runs to the deadline.
  Pick:
  keep the group.
- A helper that moves itself out of the group,
  and the moment between the leader's exit and the signal in which its number is free to be reused:
  nothing reaches either.
  Left as written in the package's README.
- A call that succeeds does not stop a helper the tool left running.
  Pick:
  leave it,
  since stopping there widens what a passing call signals.

### The scanner's workflow: what the round of issue 6633 left

- A link to a file is read through,
  and its stored target text is not scanned;
  a link to a folder now has its text scanned.
  Ranking:
  scan it > leave it,
  because a forbidden string can sit in a link's target text whatever the link points at,
  and the cost is a second finding under another label when the file behind it matches too.
  No round runs on it.
- The whole-tree arm (`--all`) inspects neither a link's name nor its target text.
  Ranking:
  list the index's links there too > leave it,
  because the whole-tree arm is the one that runs when no baseline exists;
  the cost is every link inspected on each such run.
  No round runs on it.
- A name that holds a line break fails the gate whether or not it holds a forbidden string,
  because the released scanner counts its `unsupported pathname line break` answer.
  Left:
  it is the scanner's behaviour,
  and changing it needs a release that is not an alpha.

### The linter: what the alpha.22 fix round chose

- Where one text is the line for two folders,
  the raw reading wins,
  so the line names the folder whose own name begins with the character it wrote.
  Not taken:
  the joined reading first,
  which names a folder the line was not written for.
- The second reading takes the whole run of whitespace before the file,
  then the leading run one character at a time.
  Not taken:
  trimming both runs whole,
  which loses a name's own leading whitespace.
- A dead line that holds an unseen character is told to be deleted,
  and the sentence still names the character and its place.
  Not taken:
  a bare "Delete this line",
  which leaves the README's promise that every message names it false.
- A key line whose edited text would still not read as a list line is told to be deleted.
  Not taken:
  the advice to take the character out,
  which the next run contradicts.
- A line of more than five words whose first folder word is absolute or holds a backslash
  is refused for that cause before its word count.
- A six-word line refused for a spaced folder is not told that it would read as a `file-uses` line.
- The help of a dead `uses` line spells the folder's name in the sentence that says what the name holds,
  so the name appears twice.

### The driver: a round's edits landed in the main checkout

Rounds of 2026-11-27 wrote edits into the main checkout
(issue 6636;
the handover's Next list has what was done).
The coordinating session put three files of the main checkout back at what `main` holds,
after saving their differences as a patch,
with the wrapper's `--no-enforce-worktree`.
That is a restore in the main checkout made without asking:
the files were this session's own round's,
by their content,
and an edit left there blocks merges.
Pick for the cure:
the driver refuses an edit or write call on a main-checkout path > the brief's wording alone,
because the wording failed twice in one day.
A round for it starts after the wrapper's release is dispatched.

## Veto open: what two more results of 2026-11-27 left (recorded 2026-11-27T19:25:26-05:00)

### `cargo-shadow`: where the asking again is spent

The Claude Sonnet second pass of `0.1.5-alpha.16` found no High and no Medium point.
Its Low points say the five asks of the route question,
about 0.9 s where one ask took about 0.04 s,
are spent in more states than the records name:
on hosts with no manager where a socket file,
a `systemd` directory or a bus was left,
and on a host whose manager is reachable and refuses at once
(`app.slice` masked:
941 ms against 68 ms).
The picks given to the fix round as its own to show wrong:

- the read of the host counts only the transports the ask can take,
  shown from systemd's source at the versions met here;
- an ask the manager answered with a refusal of its own ends the asking,
  and a failure to reach the manager is asked again,
  unless a real reexec shows such an answer inside its window,
  in which case the README names the cost;
- the README and the package document name every state in which the span is spent.

The way not taken:
accept the wait everywhere and only correct the sentences,
which costs each such host about 0.9 s on every call that asks.

### `mpm-shadow`: what the sentences round chose beyond its brief

- The warning states the kept copy's fate only where the publish itself failed.
  For a failed claim and a failed put-back it says nothing of the copy,
  because no control can build a state in which the install that follows succeeds there.
  Not taken:
  say it in those arms too,
  on an assertion no test holds.
- Where the lock was already satisfied,
  and where the install that followed failed too,
  the warning claims nothing about a copy.
- The `prune` line for a second copy keeps its clause on what deleting the directory loses.
- Two README sentences outside the round's section made the same promise and were corrected with it.

## Veto open: what the round on `mpm-shadow`'s Rust row chose (recorded 2026-11-27T19:34:10-05:00)

The round is merged and goes out in `0.2.7-alpha.18`.
Its picks were the coordinating session's,
in the entry recorded 2026-11-27T19:13:58-05:00;
these are what it chose beyond them.

- The row's toolchain is found by its directory's name,
  the channel or the channel followed by one host triple.
  Not taken:
  ask `rustup which` for each declared program,
  which is exact for every case and costs a process per program on each rebuild.
  What the name rule gives up:
  a channel installed for more than one host triple,
  where the first name in order answers.
- A toolchain whose name carries a date never answers for a bare channel,
  so `nightly` does not take a dated toolchain's programs.
- The rebuild removes a link a person made by hand too,
  when its name is one no row declares,
  and names it in the report;
  a regular file stays.
  This machine's folder holds no such name today.
- The name the running `mpm-shadow` is linked under stays even where no row declares it.
- `list` asks `rustup` three questions for a Rust row where it asked one,
  to say that a declared component or target is missing.
  Open:
  whether that answer is worth the two extra processes on every `list`.
- Open:
  whether the README should say that a plan may declare `rustup` on purpose,
  for a machine whose cargo bin directory holds rustup's own programs.

## Veto open: what four more rounds of 2026-11-27 left (recorded 2026-11-27T19:45:16-05:00)

### The test tasks: `depends` against the root rule

The root `mise.toml` says of tasks:
"never use depends or post depends,
use run only because it's the easiest to understand."
The round of issue 6635 used `depends = ["build"]` on the driver's `test:unit`,
as other packages do for the same need,
and as the root's own `lint:monochromatic-lint` does with its reason beside it:
a `run` list hands the words after `--` to every entry,
and a test task takes test file names there.
Ranking:
say in the root sentence when `depends` is right > leave the sentence > forbid `depends` everywhere,
because the first makes the rule match what works,
the second leaves a rule that 90 package files contradict,
and the third breaks `test:unit -- <file>` wherever a build must come first.

### The test tasks: packages whose tests read a built copy the task does not build

96 more packages are in the state issue 6635 cured for the driver
(issue 6637 lists them),
and the root aggregate `test:unit` builds nothing.
Ranking:
give each package's `test:unit` the same line > make the root aggregate build > leave,
because the fault is quiet wherever the stale copy still exports every name a test asks for,
and a per-package line keeps a package's own task right when it is run alone.
The cost is one build per run of each such task.
No round runs on it:
it changes packages this work does not own.

### The store fixture's deadline

- The value.
  15,000 ms is 3.26 times the slowest call measured under the small cap,
  and equals the reader's own deadline for the same tool.
  Not taken:
  about 5,000 ms,
  which still clears that call and makes a true hang cost a third as much.
- The equality with the reader's `SOPS_TIMEOUT_MS` is a comment and not an import,
  so the two can drift.
- The test runner's worker count ignores the control group's `cpu.max`.
  Ranking:
  leave it > cap the workers at the quota,
  because the suite now passes at the small cap in about 150 s,
  and capping would make that several times longer without making any call faster.

### The wrapper: a slot command at a directory that is no repository

A removal,
a move or a prune typed with `-C` at a directory that resolves to no repository reaches Git unchanged,
and Git answers in its own words.
Left as it is.
Not taken:
refuse every slot command whose directory Git cannot resolve,
which would also refuse `git worktree prune` typed where there is no repository at all.

## Veto open: where the shared guest steps live (recorded 2026-11-27T20:02:01-05:00)

Issue 6612's round kept the one set of guest-step helpers in the wrapper package's `bin/`,
not beside the drivers in `e2e/`,
because the package's lint refuses a unit test under `e2e/` that imports sibling source,
and a helper under `e2e/` may carry a unit test only when its name ends `-fixture`.
Not taken:
move the steps to `e2e/` and change the lint or the names around that rule.
The round also left one case to a unit test alone:
a run that keeps its guest (`--keep-guest`) was run in no guest.

## Proposed: an `AGENTS.md` rule on the holds a session carries (recorded 2026-11-27T20:47:01-05:00)

The owner's "parallelize better" found three holds the coordinating session kept with no failure on record:
`main` held still through every release,
one command at a time,
and the ZFS route's next step sequenced behind rounds that had merged.
`AGENTS.md` already asks for the rest:
"start every ready tracked item unasked",
"run independent jobs in parallel",
and that a precaution which makes work wait "names the recorded failure it prevents".
What it does not say is when a hold is read again.
Proposed,
its code to be chosen and checked when accepted:
"A hold a session carries in its notes or summary is read against its recorded failure each time it makes work wait;
a hold with none is dropped that turn."
Not applied:
`AGENTS.md` is the owner's.
The handover's "How work is run side by side" holds what the session does now.

## Veto open: what was done or picked on 2026-11-27 after 20:00 (recorded 2026-11-27T20:47:01-05:00)

Each is the coordinating session's own,
done or briefed without asking,
and each can be undone.

- The lock commit of `cargo-shadow` `0.1.5-alpha.17` (`7f21ae4ef9`) reached origin's `main` by fast-forward with no merge commit:
  the lock branch had been made from `origin/main` with tracking,
  and origin's `main` moved in the second the commit service's push of that branch ended.
  The commit holds only the lock row's version,
  address and checksum lines,
  which a check in the same command had required before it was made.
  Lock branches are made with `--no-track` from now on.
- Issue 6641,
  the wrapper's two High points of the alpha.51 read.
  The brief's reading of `SPEC.md`'s "the worktree a command changes is the one measured and the one refused":
  where a command behind a `!git` alias writes files in one place and an index and `HEAD` in another,
  both are measured and the command is refused when either would be refused.
  Not taken:
  judging the files alone (alpha.51) or the repository alone (alpha.50 for these rows).
- The wrapper's `final-newline` hint:
  at an add refused at `error` it leads with `fix -- <path>` and staging again,
  and at a manual push it says to edit and commit again.
- The crate's one doc test becomes a unit test the gate runs,
  with its fence written as the crate's other 15,635 are.
  Not taken:
  a doc-test task in the gate.
- Dismissed from the alpha.51 reads:
  the alias spellings `SPEC.md` names as the guard's boundary,
  and the line numbers the internal-error notice prints.
- The linter:
  U+2800 is spelled as other characters a reader cannot tell from a space are,
  or listed in the README as left raw,
  whichever the code's own table makes right;
  and a sentence no longer tells a person to untangle a loop in a folder that holds no module file.
  Whether the rule `markdown/semantic-line-breaks` reports a break at no boundary is proposed by the round of issue 6642 and not built.
- `rust/avoidable-closure` runs on no host run on this machine,
  because its Dylint driver is absent and the linter downloads none.
  Who puts the driver on a machine,
  and whether a skipped rule fails a check,
  is undecided:
  issue 6643,
  whose round writes the proposal and builds nothing.
- `mpm-shadow`,
  from the alpha.18 read:
  a temporary link whose installer is alive stays and one whose installer is gone is removed and named;
  a link named `mpm-shadow` that resolves into an installed copy stays whatever binary runs;
  a Rust row reads the directory named the channel or the channel followed by this host's own triple;
  and `list` gets a bound on its wait only with a measurement beside it.
- `cargo-shadow`'s alpha.16 round chose:
  a confirming ask after a refused start,
  in place of reading the failure's words;
  asking where the slice is before starting it;
  counting the transports of both systemd versions met here;
  and a file of its own for the one ask,
  because its comments took the discovery file over the line budget.
- Issue 6636's round left three points:
  whether the driver checks before the first prompt that the guard is armed,
  as it does for the masking,
  which needs a new end reason;
  whether a worktree outside `.claude/worktrees` stays allowed to change itself;
  and whether a guard for shell commands is worth building on the shell analyzer,
  which would leave a command run after a change of directory open.
- Issue 6639's round left four:
  where a unit-tested reader of a guest build's log lives
  (it ranks a shared module under `bin/` first);
  whether the shared wait's refusal carries the log's tail itself;
  that the `step` shape of the name check runs the work whatever the check answers,
  measured;
  and that a launcher which refuses for the wrong guest leaves the host polling until the finish bound,
  43 minutes for this build.

## Veto open: what was picked or left on 2026-11-27 after 21:00 (recorded 2026-11-27T21:27:22-05:00)

Each is the coordinating session's own or a round's,
done or briefed without asking,
and each can be undone.

- The fourteen ZFS route rounds were briefed with these picks,
  each labelled in its brief as the coordinating session's own:
  `--force` stays refused in every spelling when the removal is switched on,
  and no printed sentence offers it;
  the private propagation step,
  the body of `shutdown-unmount` and the lost-event merge device are built with their guest arms and get no call from a root mode,
  because those calls are plan step 19's wiring;
  the lost-event check before a command is measured against the same wrapper without it,
  where the design claims it costs next to nothing;
  the two forced-arm rounds build the stop of a locked holder's unit,
  and the ending of a taskless holder,
  only as far as their measurement supports,
  and say plainly where a refusal the owner kept has changed and on what evidence;
  and new code goes into child folders,
  where the layout rules print no finding.
  Not taken for the three step 19 pieces:
  calling each from its root mode now,
  which the built-ahead decision reserves for the step's own wiring.
- The round on the wrapper's `final-newline` hint left five points.
  A landed commit shares the manual push's sentence.
  The direct check's sentence is unchanged,
  on the round's own argument.
  `fix-summary` prints no hint,
  so nothing tells a person to stage again after `fix`.
  `CandidateRequest::Staged` is built by no release code.
  And the enum `CommitCorrection` kept its name while four of its variants were renamed,
  which changes the library's published names;
  the README and `SPEC.md` passages were not rendered.
- The round on the two failures left two.
  Every fact failure prints the program's name twice
  (`monochromatic-git-policy-cli: monochromatic-git-policy-cli could not start ...`),
  older than this release and the same wherever a fact cannot be read.
  And `invocation/entry_forward_action_planning.rs` holds 316 code lines,
  three of them the round's,
  over the budget before it.
- The `mpm-shadow` round on a failed install's message chose:
  one sentence for all six failures,
  naming both the publish and the removal,
  where two tailored sentences would each be true of one state;
  nothing about a later install where a put-back of the copy that was moved aside fails,
  because that copy is no finished one;
  and the case of a newer repository tool,
  which leaves the kept copy hidden,
  inside the sentence.
- The round of issue 6638 chose the version string alone as what tells a built module from another,
  so a rebuild of the same source reads as already there;
  a plain `node` check in place of a harness unit test,
  because the package has no `package.json`;
  and left the template's own prepare,
  `prepareFromTemplate`,
  loading with `modprobe --quiet zfs 2>/dev/null || true`.
- The linter's round on the alpha.23 read chose:
  the bound on a gap's names from the package folder's longest name,
  which changes no answer;
  U+2800 left as written,
  with the reason in the README;
  a `uses` line in the file rule's list still read and reported as tolerating nothing,
  where refusing it would refuse lists written before;
  and the remedy for a spaced folder chosen by whether the folder holds a `.rs` file.
- The proposal of issue 6642's round,
  `doc/planning/monochromatic-lint-a-line-break-at-no-semantic-boundary-is-or-is-not-a-finding.md`,
  asks three things:
  whether a break at no semantic boundary becomes a finding at all;
  whether its removal fix ships in the same release;
  and whether the bound a join uses is the rule's own or the repository's 120 characters.
  It ranks first the reporting of only those breaks a removal fix can join.
  The round itself chose that a fragment is one or two words,
  that a join happens only where this pass's own break cut it,
  and that no line the author wrote is rewritten.
- The second pass of `cargo-shadow` `0.1.5-alpha.17` leaves two choices its fix rounds will meet:
  whether a path the confirming ask reports after a failed start is the route,
  and whether a stop job queued on `app.slice` is a state the discovery waits out,
  as alpha.16's start did,
  or one it names.

## Veto open: what the rounds read on the night of 2026-11-27 chose or left (recorded 2026-11-27T21:42:08-05:00)

Each is a round's own or the coordinating session's,
done or briefed without asking,
and each can be undone.

- The linter's closure rule,
  issue 6643:
  the proposal `doc/planning/monochromatic-lint-who-provides-the-dylint-driver-and-what-a-skipped-rule-returns.md` asks who provides the Dylint driver and what a skipped rule returns.
  It ranks the toolchain install building the driver first,
  then a published driver installed as a tool,
  then the linter's task warming it,
  then a gate that owns its driver,
  then a skip that stops being a pass,
  then pre-filled caches,
  and last folding the rule back into the linter,
  which a decision of 2026-10-18 rejected.
  Its first choice waits on the pause of `toolchain:install` (issue 6316).
  It says a code of its own for a skip is worth taking with any of them,
  and that an exit status of its own for a skip contradicts the answer of 2026-11-07 that a skip is a warning.
  Nothing is built.
- The wrapper's round on the surface control chose:
  a `mod` statement at a file's first column that the reader cannot place stops every scan,
  so a later `pub(in ...) mod` line has to be written `pub(crate)`,
  and fixture text holding such a line is written indented;
  a carried type's members are read from the facade,
  which added fifteen lines to the list that no round decided one by one,
  each a name alpha.51 already publishes;
  the commit path's files may not call `libc::syscall` at all,
  where a sentence about the one constant was the other way;
  and a name behind a feature gate stays in the list as published.
  Left unread and said so in each guard:
  a write to descriptor 2 by its number,
  a start whose program is a value,
  and a call a macro of another file expands to.
  One of its commit messages says eighteen findings where twenty-nine is right,
  corrected by a commit comment.
- The `mpm-shadow` round on the Rust row asks whether `list` gets a deadline for its `rustup` questions.
  None ships.
  It ranks none first,
  because it cannot report a part unreadable that is fine;
  then a bound of 60 s,
  which only a hang would reach and which needs the owner's decision under the rule on shipped limits;
  and last a multiple of the measured cost,
  which this machine's own slow runs would trip.
  It added `Context::the_toolchains_bin_dir` and `Context::find_bin_of_in` and a sixth parameter to `layout::bin_search_dirs`,
  and one of its commit subjects is wrong and has a corrective commit comment.
- The `mpm-shadow` round on the exposure folder's removal asks three things:
  whether a temporary link whose install cannot be judged is named in the install's report or stays silent,
  as it does;
  whether the temporary name's growth,
  from about eleven bytes to about seventy,
  is acceptable,
  since a program name past roughly 180 bytes would now fail at the link write;
  and whether the rule for the earlier name shape,
  which keeps a link while any process holds its number,
  should expire.
  It changed `rebuild_exposure` to answer an outcome that carries both the report and the error.
- The scanner's workflow,
  issue 6640:
  a link to a file has its name scanned twice,
  so a forbidden name there prints two findings;
  the two later steps run after a failure and not after a cancellation;
  and `git cat-file --batch` for the index listing was not taken,
  on a measured 2.3 s to 2.7 s for this repository's links.
  Scanning both a link's content and its stored target text was the coordinating session's ranking in the issue.
- The two `cargo-shadow` rounds on the alpha.17 read were briefed with these picks:
  a call alpha.16 carried is carried again;
  a path the confirming ask reports after a failed start is the route;
  a stop job queued on `app.slice` does not fail the call,
  and a wait for it gets a bound that cites the round's measurement;
  the ordinary call keeps one `systemctl` program;
  and whether a scope's creation that meets a reexec is tried again is set out for the owner and not built.
- The round of issue 6644 was briefed with this pick:
  the template guest's prepare says which module it loaded and refuses in words when it is not the one the template was built with,
  through the shared step or a smaller one of its own,
  which the round decides from what the template holds.
- The round on the ZFS records' status lines was briefed with these picks:
  a sentence that states a decision keeps its words and gains a dated note in place;
  a sentence that only states status is corrected;
  and the step 22 record's sentence that no value is set until the owner accepts one is not touched.
- The four rounds that cut the handover's landed history were briefed with these picks:
  a count from a check of a branch that has merged is not a durable fact;
  a fact with no home goes into an existing doc on its subject,
  and into a new file only where the sorter's ledger shows none fits;
  and words the handover gives as the owner's stay unless the round has read them in the other record the ledger names.
  The coordinating session read none of the four ledgers whole.

## Veto open: what four more results of the night of 2026-11-27 left (recorded 2026-11-27T22:15:09-05:00)

Each is a round's own or the coordinating session's,
done without asking,
and each can be undone.

- Issue 6641,
  a command behind a `!git` alias.
  The issue's text,
  written by the coordinating session,
  read `SPEC.md`'s "the worktree a command changes is the one measured and the one refused" as:
  both places are measured,
  and the command is refused when either would be.
  The round did otherwise and said so:
  it measures the one state the nested Git is in,
  the files of the place its words reach with the index and `HEAD` of the Git directory its shell inherited,
  and refuses when that state would be refused.
  For:
  the refusal describes what Git will do,
  one facts query answers it,
  and none of the dangerous rows runs.
  Against:
  three commands that change nothing in real Git are refused,
  `g -C <another worktree> checkout -- <path>` and the `restore` and `checkout <tree> -- <path>` rows of that shape,
  typed in a dirty linked worktree,
  because the state is dirty and the bytes happen to be the same on both sides.
  Not taken:
  asking the facts twice and refusing if either refuses,
  which refuses those three and more;
  and judging only the files' place for the forms that write no index,
  which needs a table of what each guarded form writes
  (`clean --force` writes files only and still chose them from the inherited index).
  Its refusal keeps the words "this worktree has uncommitted changes to tracked files",
  true of that state,
  without saying that the files and the index come from two places;
  a sentence that names both is a new message and was left for a decision.
- The records-only round on the ZFS route's status lines left four things as they are.
  `SPEC.md` says the settle that finishes a failed add with Git's own checkout "is not built",
  where the settling command's module now exists and no command reaches it:
  name the module and its cut,
  or leave the sentence.
  The plan's section "What can start now" is the scheduling text of 2026-11-19,
  passed by sixteen merged steps:
  rewrite it,
  or keep it as that day's record.
  The lost-event design's Status says nothing of it has been built or run,
  which the round did not verify either way,
  and two rounds are building it now.
  And the step 23 record left the form of the gate arm's assertion to the owner,
  release-independent or a per-guest expectation;
  `main` holds the release-independent form since `bd180d7de9`,
  because a round built it,
  and no record says the owner accepted it.
- `mpm-shadow`'s join of the exposure removal with the Rust row's change:
  no control drives a postinstall link or command on a `rustup-toolchain` row,
  so the two call sites the join took over rest on the compiler.
  The gap is older than both rounds,
  and it goes to the round on the read of `0.2.7-alpha.19`.
- The driver refused one start while memory was short,
  because its guard waits 10 s for the wrapper's `--help`
  (issue 6646).
  The round was started again from a list of its own.
  Nothing was changed in the driver.
- The wrapper round that repairs its doc comments was briefed with these picks:
  the repair is made by the fixed linter on each file as it stood before the sweep,
  and replaces what `main` holds only for a doc paragraph whose words have not changed since;
  nothing but doc-comment lines changes,
  and no word of any;
  and the doc fence in `policy/policy_forbidden_strings.rs` is closed where its block ended.

## For the owner, and veto open: what the night of 2026-11-27 left after 22:15 (recorded 2026-11-27T23:01:36-05:00)

### A question that waits: what bounds the guests and the builds that run at once?

The machine ran out of memory three times on the night of 2026-11-27,
the second and third time for about twenty minutes each
(`doc/handover/cli-git-rust-implementation.md`,
"Memory";
`doc/troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md`).
Fourteen rounds on the ZFS route had been started within five minutes of each other,
on the direction to parallelize better and with no start waiting on memory,
as the owner said on 2026-11-26.
An earlier text of this entry named the builds as the load.
A reading of 2026-11-27T23:20:27-05:00 corrects it:
the unit that holds the guests held 29.9 GB of memory and 50.7 GB of swap with ten guests alive,
each at the 8,192 MiB the owner kept until the crate split,
and all the rounds' builds together held 15.8 GB and 1.4 GB.
That reading counts what was left:
it was taken seconds after twelve builds had been killed,
with three compilers alive.
Over its life the builds' slice has peaked at 55,702,958,080 bytes of memory and 56,149,524,480 of swap,
and the guests' unit,
since it started at 20:03:51,
at 33,109,938,176 and 52,823,150,592
(`MemoryPeak` and `MemorySwapPeak`,
read at 23:25:24),
so neither is shown to be the larger at the moment the memory ran out.
What both readings of the night show is that the guests' unit holds about 50 GB of the 64.7 GB of swap the whole time,
whether a guest is in use or not,
and a build holds its peak for minutes.
The builds are what the machine kills when the swap is full.
What it cost,
measured:
23 build scopes ended `oom-kill` between 22:34 and 23:20,
the wrapper gate lost its build after 41 minutes and ran no test,
one round's start was refused,
and for a third to a half of the worst fourteen minutes no task ran.
No round and no guest was lost.
By the owner's criteria of 2026-11-13 the stall fails no level of agents at once;
those criteria say nothing of a killed build.
The options:

- A bound on guests alive at once,
  found the way the disk's ceiling is found
  (`doc/handover/agents-at-once-on-this-machine.md`),
  with ten as a level that failed.
  For:
  it aims at the largest holder measured,
  and rounds that need no guest stay bounded by 64 alone.
  Against:
  it is a rule on starts,
  rounds that need a guest queue,
  and the search costs runs.
- Smaller guests for rounds that build nothing in the guest.
  For:
  ten guests would hold half as much at 4 GiB.
  Against:
  the owner shelved the guest memory proposal until the crate split on 2026-11-26,
  because its figures measure guests building today's one crate,
  so this reopens a decision.
- A bound on rounds that compile the wrapper at once.
  For:
  it removes the builds the machine kills.
  Against:
  rounds wait,
  and the guests' share of the swap stays as it is.
- No bound,
  and a killed build made cheap:
  every brief says that a kill by the machine is run again and says nothing about the change.
  For:
  no rule on starts.
  Against:
  each kill throws a compile away,
  and tonight the kills came faster than the builds finished.
- The crate split,
  which waits for the owner's answer on its fourth round:
  it lowers the compiler's peak and lets the guest question be measured again.
  It goes with any of the others and replaces none tonight.

Ranking:
the bound on guests over smaller guests,
because it needs no shelved decision reopened;
smaller guests over the bound on builds,
because a guest holds its memory for its round's whole life and a build for minutes,
though at their peaks the two hold the same range and both bounds can be taken;
the bound on builds over no bound,
because it removes work that the other only repeats;
the split beside whichever is taken.
Until this is answered the coordinating session starts no second batch of that size:
it starts the next round that makes a guest or compiles the wrapper as a running one ends.
That is its own measure,
with the veto open.

### Veto open

Each is the coordinating session's or a round's own,
done without asking,
and each can be undone.

- The wrapper gate's rerun waits on a memory reading:
  `MemAvailable` plus `SwapFree` at 40,000,000 kB on three readings a minute apart,
  for six hours at most.
  The owner's rule is that no start waits on memory;
  this is one gate,
  after its build was killed once,
  and nothing else waits on it.
- Every live round got one note at 2026-11-27T22:54:05-05:00:
  what happened,
  that a build the machine killed is run again,
  and a suggested single reading of `/proc/meminfo` before a build that compiles the wrapper.
  No round was stopped or frozen.
- The linter's two proposals are not put to the owner yet.
  The gate read of each came back with High findings that break the ranking its question would carry
  ("What the gate reviews of the linter's two proposals found" in the handover),
  so each is revised by a round and read again first.
  Two questions inside them are the owner's and will be asked then:
  which kinds of line break `AGENTS.md` rule MD1 permits,
  and whether the nightly toolchain is pinned.
- Part A of the handover's history wrote the practice by which a release is dispatched from a tree newer than its gate into
  `doc/decision/wrapper-merges-and-publishes-do-not-wait-for-reviews-or-mutation-runs.md`,
  labelled as the coordinating session's practice.
  It is that and no more:
  nothing in a dispatch reads the container gate.
  The coordinating session held part A's merge for that practice for half an hour and then dropped the hold,
  which no recorded failure backed;
  part B had waited eighteen minutes for `mpm-shadow`'s release commit,
  for the reason the release workflow itself gives.
- Parts A and C were joined in one tree before their merge.
  The one conflict,
  in the linter's README,
  kept both sides:
  the handover as it stood before the cuts shows the two sets of figures are different measurements.
  No wrapper test ran on the three files of the wrapper's that the parts changed;
  the next gate is their first run.
- What the three merged parts chose against their ledgers:
  part A let nine facts go as counts of a merged branch's check,
  wrote a fact into a module doc comment and into the comment header of the list of published names,
  and left the paragraph on issue 6638's round alone;
  part B kept two list openers whose items are now only open points;
  part C wrote no gate total and no lint count into a planning record,
  kept the working rule for a built wrapper executable in the handover alone,
  and gave the `~/.viminfo` incident a troubleshooting file of its own.
- Part C asks two things it left:
  whether the release binary's growth of about 112 KiB between `0.6.1-alpha.48` and `0.6.1-alpha.49`,
  which has a home and no cause,
  gets a round;
  and whether an agent's `vim` run rewriting `~/.viminfo` becomes a rule in `AGENTS.md` or stays a record briefs point at.
- Issue 6644's round chose a step of its own for the template guest's prepare and not a parameter of the built module's step;
  reads before the load;
  a pin that covers both module files where each records a version;
  a step that removes nothing;
  and one reading of the loaded version in place of the recipe's own two.
  One state is held on the host alone:
  a loaded module that is not the file's,
  because the template's kernel refuses a repacked module's signature.
- Two issues were filed from what the night showed:
  6647,
  the logger's file sink verification reporting an internal error after 5000 ms on a loaded host,
  and 6648,
  a slow `git worktree remove` that removes the worktree and then fails.
- Part D of the handover's history,
  merged as `00740b0320`,
  cut 73 of its ledger's 91 entries and kept fifteen open ones.
  It read two duties as discharged,
  "the wrapper's next release pass reads it as a change",
  because the passes of `0.6.1-alpha.48` were the next ones,
  where no tracked file says a pass read each item;
  it let five verification figures go as counts of a merged branch's check;
  it could not name the private helper that removes directories among the seven a split made public,
  and kept the pass's own words;
  and it added a two-line comment to `package/git-policy/cli/bin/native-check-commit-line.ts`,
  a file of code.
  Its conflict with `main`,
  in the linter's README,
  kept both sides.
- The `cargo-shadow` round on the Low points of the read of `0.1.5-alpha.17` leaves the owner one choice and built neither way:
  a manager reexec that begins after the route is discovered fails the whole call,
  and a scope's creation that meets one is either tried again or left.
  Tried again:
  a call that overlaps a reexec finishes,
  which is what a person upgrading packages needs;
  but `systemd-run --user --scope` gives no idempotence,
  so a retry can make a second scope beside one that was made and whose reply was lost,
  and each attempt spends another discovery bound.
  Left:
  no new branch and a loud failure with the manager's own words;
  but a build that overlaps a reexec fails whole,
  and the read measured 11 of 24 such calls exiting 1.
  The round also corrected four line ranges the brief did not list,
  wrote records for its two controls,
  and measures the pause from a stand-in's own start and end lines,
  which showed the planted edit in one gap of four on the loaded host.
