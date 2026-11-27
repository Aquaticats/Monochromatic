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
