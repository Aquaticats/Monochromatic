# Commit and landing route findings on `main`

Status:
audit,
2026-11-12.
Findings 1 to 4 are fixed on branch `fix/git-policy-cli-commit-and-landing-route-findings`
(`package/git-policy/cli/SPEC.md`,
"What a landed batch leaves behind";
the copy of a conclusion's state in the journal;
the push record the service leaves before it answers).
Finding 5 is open:
it is the owner's question,
and no kept ref is removed by this branch.
Audited revision:
`main` at `554f973343`,
which is `origin/main`.
Worktree:
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-commit-landing-route-findings-on-main`.
No production code was changed and no test was committed:
the four controls in "Reproductions" were added as one sidecar test file and one `mod` line,
run,
quoted,
and removed;
`git status` in this worktree is clean apart from this page.
Paths below are relative to `package/git-policy/cli/src/native/` unless they start with `doc/`.
Linux only;
the owner's pause covers Windows and macOS.

Purpose:
the same audit the copy route got
(`doc/planning/cli-git-copy-route-findings-on-main.md`),
for the other route:
one `git commit` from the wrapper,
through the resident service,
to a landed commit,
including preparation,
the transaction,
the branch move,
the journal and its recovery,
and a service restart while work is queued.
Each finding is a sequence of steps and the wrong result it leaves.
Anything I could not state that way is left out,
so this page is short on concerns and long on sequences.

## The route, step by step

The wrapper,
in the command's own process:

1.  Chooses the service route from the worktree's own Git directory (`landing/service_route.rs:122`,
    `choose_route`).
2.  Captures the invocation and builds a private index inside the command's scratch directory,
    then stamps the capture (`landing/service_prepare.rs:223`).
3.  Runs the pre-forward policies over the private index.
4.  Hands the scratch directory to the service over a second connection,
    whose close tells the service to remove it (`landing/service_commit.rs:222`,
    `watch_scratch`).
5.  Provisions the unit files once,
    if it has to (`landing/service_commit.rs:316`).
6.  Sends the request with a random identity,
    and resends the same identity after a broken connection
    (`service/service_client.rs:177`,
    `submit_loop`).
7.  On `Landed`:
    installs the corrections a policy made into the worktree and removes the saved message file
    (`landing/service_commit.rs:332`,
    `report_landing`).
8.  Prints the events,
    runs the post-commit gate (`commit/commit_command.rs:256`),
    then writes the push and upkeep work records and wakes the service (`commit/commit_command.rs:263`,
    `landing/service_handoff.rs:117`).

The service,
on its one landing thread:

9.  Settles the open batches an earlier instance left,
    claims a batch number (`service/service_server_land_group.rs`),
    then lands.
10. Reads the branch,
    checks the worktree still has the branch checked out,
    compares each request with the branch (`landing/service_land.rs:258`,
    `:290`).
11. Writes the batch's objects with one `fast-import`,
    which moves `refs/worktree/cli-git-service/<seq>`,
    and leaves a `marks-<seq>` file in the journal (`landing/service_land_objects_of_batch.rs:108`).
12. Writes the intent durably (`landing/service_land.rs:350`,
    `service/service_journal.rs:209`).
13. Takes Git's own `index.lock` (`landing/service_land.rs:362`),
    then records the witness of the live index
    (`landing/service_land.rs:381`).
14. Moves the branch by compare-and-swap (`landing/service_land.rs:406`).
15. Removes the conclusion state,
    builds the new index beside the live one,
    renames it over the live index,
    records `<seq>.indexed`,
    and gives the lock up
    (`landing/service_land_finish.rs:150`,
    `landing/service_land_follow.rs:299`,
    `landing/service_index_hold.rs:241`).
16. Renames the intent to `<seq>.done` and removes the start and indexed records
    (`landing/service_land.rs:435`,
    `service/service_journal.rs:359`).
17. Answers each client (`service/service_server_land_group.rs:246` onward).

## What a process killed at each step leaves behind

Kill points 1 to 6 are in the wrapper,
9 to 17 in the service.

1.  Before the private index exists:
    nothing.
    The scratch directory is written under the wrapper's own scratch root and swept by age
    by a later command,
    so a kill here leaves a directory the sweep takes,
    not a record.
2.  Private index built,
    before the service sees it:
    a scratch directory with no durable record,
    removed when the wrapper's watch connection closes or by the sweep.
    The worktree itself is untouched,
    so nothing is lost.
3.  Sent,
    before the service reads it:
    nothing on disk;
    the client resends the same identity.
4.  Landed,
    before `report_landing` runs (step 7):
    the commit is on the branch,
    the worktree keeps the uncorrected bytes the policy corrected,
    and the saved-message file of a rebuilt commit stays.
    `git status` shows the corrected paths as modified,
    which is visible but is not said.
5.  After the post-commit gate,
    before the hand-off (step 8):
    the commit is on the branch and no push,
    upkeep or review record exists.
    Finding 3.
6.  After the hand-off:
    everything the command owned is written;
    the push record is the service's.
7.  Objects written,
    before the intent (step 12):
    `refs/worktree/cli-git-service/<seq>`,
    a `<seq>.scratch` record,
    a `marks-<seq>` file in the journal,
    and the busy decision file.
    Recovery keeps the commit the reference holds on `refs/cli-git/kept/<seq>` and removes the reference
    (`landing/service_scratch_ref_marker_of_a_batch.rs`,
    `clear_recorded_scratch_refs`),
    so no commit is lost and the client lands a fresh one;
    the kept ref is never removed.
    Finding 5.
8.  Intent written,
    before the branch moves (steps 13,
    14):
    recovery reads the branch,
    finds it unmoved,
    decides `Drop`,
    keeps the commit on the kept ref,
    removes the lock and the records
    (`landing/service_land_finish.rs:363`,
    `:587`).
    The commit is not on the branch and the client lands again.
9.  Branch moved,
    before the index is installed:
    the state recovery exists for.
    `settle` (`landing/service_land_finish.rs:229`) replays the index update only while the live index file is
    still the file the landing saw (`landing/service_land_start.rs:15`,
    `gate_of` at
    `landing/service_land_finish.rs:204`).
    Index installed but `indexed` not yet recorded:
    the witness differs,
    so the replay is skipped,
    which is correct because the index is already right;
    the journal says the paths were not updated,
    which is then untrue.
10. Index installed and recorded,
    before `<seq>.done`:
    `indexed_state` answers `Installed`,
    the batch is finished without touching the index,
    and it is marked done.
11. Done,
    before the answer:
    the client resends,
    and the new instance answers `Landed{already}` from
    `landed_commits` (`service/service_journal.rs:578`),
    which reads every `<seq>.done`.

## Waits and their ceilings

Every value is in `service/wait_ceilings.rs` with its measurement or owner decision beside it.

- The wrapper's wait for an answer:
  `ANSWER_CEILING`,
  20 minutes (`:34`),
  plus `SHORT_EXTRA_WAIT`,
  7 seconds,
  for a request the service says is inside its branch write (`:553`).
  At the ceiling the request is withdrawn on a second connection and the withdrawal is waited for
  `WITHDRAW_CEILING`,
  10 minutes (`:345`,
  `service/service_client.rs:315`).
  A service that has not started is waited for `START_CEILING`,
  60 seconds (`:60`),
  and a connection that breaks is retried `RECONNECTS` times
  (`service/service_client_settings_and_watches.rs`).
- The landing's wait for another program's `index.lock`:
  `LANDING_LOCK_CEILING` (`:198`),
  waited on an event watch with the holder's line repeated every `NOTICE_EVERY`,
  10 seconds
  (`landing/service_index_lock.rs:74`,
  `landing/service_index_attempts.rs:175`).
  A dead owner ends it at once;
  a holder that proves nothing is waited for like a live one.
- The index update after the branch moved:
  `RECONCILE_LOCK_CEILING` (`:327`)
  plus five tries 100 milliseconds apart (`landing/service_land_follow.rs:62`).
- Each Git step:
  `PLUMBING_STEP_CEILING`,
  2 minutes (`:378`),
  or `OBJECT_STEP_CEILING`,
  10 minutes (`:395`).
- The look at other processes for an earlier release's helper:
  `PROCESS_LOOK_CEILING`,
  10 seconds (`:74`).
- Work items:
  a claim's lease is 120 seconds,
  renewed three times per lease
  (`landing/service_work.rs:83`,
  `landing/service_claim_renewal.rs:56`),
  so a killed push is retried within about 120 seconds.
- Housekeeping ages:
  `TEMPORARY_GRACE`,
  ten minutes,
  for a journal file a killed write left
  (`service/service_journal.rs:67`);
  `STRAY_TEMPORARY_AGE`,
  a day,
  for a recovery's temporary
  (`landing/work_item_recovery.rs:39`);
  `IDLE_EXIT`,
  three hours (`service/service_server.rs`),
  and `OWN_WORKTREE_CHECK`,
  60 seconds.

One wait has no ceiling,
by owner decision:
the branch write,
`git update-ref --stdin`,
is `StepBound::Unbounded` on the landing thread
(`transaction/transaction_git_step_bounds.rs:97`),
recorded from the owner's 2026-11-04 answer "Never cut a branch write (Recommended)"
(`service/service_busy.rs:12`).
The same option's stated cost is that a write that never returns keeps the service busy and
later commits end at their own ceilings (`service/service_busy.rs:16`).
The wrapper's rule "nothing should ever wait forever or indefinitely" therefore holds for the person
and not for the service:
a stuck write holds that worktree's queue,
and each queued commit ends after 20 minutes plus its withdrawal ceiling with an error naming the write.
A request already inside a landing is never withdrawn,
only reported as not known.

No other wait in this route is unbounded:
every sleep is between two attempts that have a ceiling,
and every wait either takes the lock,
fails on a dead owner,
or ends at its ceiling.

## Locks and why each exists

- Git's own `index.lock`:
  Git's protocol,
  and the only writer of the worktree's index.
  The landing takes it after the objects and the intent exist and holds it for the branch move and the
  index build only (`package/git-policy/cli/SPEC.md`,
  "Commit protocol").
- The marker inside that lock,
  and the claim chain beside it (`landing/service_index_marker.rs`,
  `landing/service_index_claim.rs`):
  no lock primitive,
  exclusive creation and one rename.
  It keeps two reclaimers of one stale lock apart without any system call
  that unlinks a name only if it names a given file,
  and it is listed in the SPEC's kept-inventory
  ("One reclaimer per stale lock").
- An exclusive `flock` on the private copy's own directory (`landing/service_land_copy.rs:79`):
  it is how a later sweep proves a copy belongs to nobody,
  without asking about a process number,
  boot or PID namespace.
  Removing it would need the process identity this design deliberately avoids.
- A mutex over the pending bytes of one journald line (`service/journald_native.rs:403`):
  the rest of an unfinished line and the write that finishes it are one critical section of one
  non-blocking write.
  Nothing waits for the journal.
- Two process-wide timing ledgers,
  `transaction/transaction_step_durations.rs:80` and
  `commit/commit_index_builds.rs:72`:
  a mutex around a vector that one thread drains.
  These are the only locks I would call removable,
  and only with a proof that the recording thread is
  the draining thread;
  the landing's blocking jobs do not prove it today.
- The landing's sync queue,
  the busy record and the claim renewal use no lock:
  exclusive creation,
  a `RENAME_NOREPLACE` publish (`service/service_busy.rs:37`),
  a rename,
  and per-claim
  atomics with a channel.

The wrapper takes no lock of its own for a commit,
and the hook lock of the earlier design is retired
(`config/config_schema.rs:199`).

## Races between two concurrent commands

- Two commits in one worktree:
  one service instance,
  one landing thread,
  arrival order within a branch.
  Nothing is shared mutably between the queue and the landing.
- Two worktrees of one repository:
  one service each,
  separate journals under
  `$XDG_STATE_HOME/monochromatic-git-policy-cli/service/<instance>`
  (`service/service_units/monochromatic-git-policy-cli@.service`),
  separate indices,
  and per-worktree refs for the scratch reference.
  What they share is the object store (Git's own concurrency-safe paths) and the common kept ref,
  below.
- A commit and a forwarded index writer (`git add`,
  `git reset`):
  the writer waits for the service's hold
  on the index on an event watch with its own ceiling,
  and the landing waits for a foreign holder the same way.
- A commit and a program that bypasses the wrapper:
  both take `index.lock`,
  and the branch move is a
  compare-and-swap against the tip the landing read,
  so a bypassing commit makes the batch rebuild instead of
  overwriting it.
- A commit and `git worktree add` or a branch switch in the same worktree:
  `still_checked_out` is asked
  before the lock and again under it (`landing/service_land.rs:253`,
  `:365`),
  and the landing answers
  `BranchMoved` for the whole batch with nothing changed.
- This release's service and an earlier release's helper for the same worktree,
  which the design admits can
  overlap:
  both write the same index,
  and both go through `index.lock`,
  so their branch move and index install
  cannot interleave.
  This release never reclaims an earlier release's lock
  (`landing/service_index_reclaim.rs:183`),
  so the older lock ends the wait at the ceiling with a notice
  naming the file to remove.
- Two services numbering batches in one journal:
  the number is claimed by exclusive creation and a number with
  an intent,
  a done record or a scratch record is skipped (`service/service_journal.rs:173`).
- The common kept ref `refs/cli-git/kept/<number>` is not per worktree,
  so two worktrees can collide on it;
  the second drop leaves the existing ref and says so in the log (`landing/service_land_finish.rs:363`).
  A collision needs two service starts in the same microsecond,
  and its cost is a commit kept nowhere.

## Reproductions

Controls:
one sidecar test file added beside `landing/service_land.rs` with the module line,
run with
`mise exec -- cargo nextest run --locked --offline --lib -- audit_tests --nocapture`,
then both changes removed.
They used the existing fixtures (`landing/service_land_tests.rs`,
`world` and
`request`) and one seam,
`crate::service_fault::hooks::fail_at("build-index")`,
which is the seam
`land_with_failed_index` already uses to leave a landed batch open.

### An unreadable open intent is deleted, and the index it alone owed is never updated

```text
the commit landed: "message of one"
the index shows: "D  one.txt\n?? one.txt"
recovery reported 0 batches
the intent is still there: false
the index changed: false
the index shows after recovery: "D  one.txt\n?? one.txt"
```

The batch landed with its index update owed,
the intent file was then replaced by text this build cannot
parse,
and recovery removed it without a report.
The index is exactly as it was,
so the file the landed
commit holds is a staged deletion.

### A batch whose index file was written meanwhile is finished, and the index keeps the staged deletion

```text
monochromatic-git-policy-cli-service: batch 9: the index entries of its paths were not updated and are left as
they are, because the index file is not the one the landing saw, so another program may have staged into it;
`git status` shows the commit's paths against the index until they are staged again
recovery decided [(Finish, Finished)]
the index changed: false
the intent is still open: false
git status says: "D  one.txt\nA  other.txt\n?? one.txt"
```

The one notice is a service log line.
The batch is finished,
and the index still shows `one.txt` as a staged
deletion of a file the landed commit added.

### A dropped batch keeps its commit on a ref nothing reads again

```text
monochromatic-git-policy-cli-service: batch 11 is not on its branch, so its commit
ec30320e3bf7cf7afa81d69ba1d771403cab718b is kept at refs/cli-git/kept/11
recovery decided [(Drop, Dropped)]
the refs that were left: "refs/cli-git/kept/11 ec30320e3bf7cf7afa81d69ba1d771403cab718b"
the intent is still open: false
```

### The conclusion cleanup reports success when the client's directory is gone

```text
the cleanup answered true
MERGE_HEAD is still there: true
MERGE_MSG is still there: true
```

Called with a `Cleanup` whose scratch directory does not exist,
`remove_conclusion_state` answers success and
removes nothing.

## Findings ranked by what they risk losing

### 1. An unreadable open intent is removed without a word, and the landed commit's index update is lost

`landing/service_land_finish.rs:567`:
an intent file that does not parse is removed by the landing worktree's own recovery,
with no report,
no log line and no retry
(`recover_selected`,
the `else` of `list_intents` at `service/service_journal.rs:485`).
Everything that finishes a batch is named by that file:
the branch values,
the paths,
the index,
the cleanup,
and the request identities.

Sequence:

1.  A commit lands:
    the branch moves,
    and the index update is owed because a step after the move did not
    finish (the batch stays open,
    as reproduction 1 and 2 show).
2.  The intent is written by one release and read by another,
    or is damaged,
    so `parse_intent`
    (`service/service_journal_intent_text_and_parse.rs:203`) answers nothing.
    Downgrading the wrapper,
    or a write whose tail is lost on an unusual filesystem,
    both do it.
3.  The next landing in that worktree runs `settle_open_batches`
    (`service/service_server_land_group.rs:35`) before it lands anything.
    The unreadable file is deleted,
    no batch is selected,
    and nothing is finished.
4.  The landed commit's paths stay as the index had them:
    a file the commit added is a staged deletion,
    and a file the commit changed is staged as the old content.

Wrong result:
the person's next plain `git commit` records the deletion of a file the previous commit added,
because the index says so and the wrapper believes the index.
Reproduced (first block above),
with the exact `git status` line that shows the deletion staged.
The same loss hides the cleanup and the request identities,
so a resend of that request lands a second commit.

The owner rule this touches is the design's own recovery contract,
which the decision record states as
"recovery handles every prepared,
reserved,
or landing transaction"
(`doc/decision/cli-git-concurrent-commits.md`,
"Recovery and compatibility").
A file that is deleted rather than reported is not handled.

Suggested fix (a suggestion,
not a decided change):
never delete an open record that cannot be read.
Leave it,
report it once per service start with its path and batch number,
and let
`doctor` count it,
so a person can delete it or open it;
and write a format marker into the intent so a later
release refuses rather than misreads one.

### 2. A recovery that finds the index written meanwhile finishes the batch and keeps the staged deletion

`landing/service_land_finish.rs:281` (the `Gate::Leave` arm):
when the live index is not the file the landing saw,
the index is left as it is and the batch is marked done.
The reasoning is sound for the case it was written for,
which is to avoid resetting what another program staged.
What it does not do is tell the person,
in the command they are running,
that the index now contradicts the
commit that landed.

Sequence:

1.  A commit lands and the index update is owed (a step after the branch move failed,
    or the service was killed
    between the branch move and the index install).
2.  Before any landing runs again,
    some program writes the index:
    a person's `git add`,
    an editor's Git integration,
    or a `git status` that refreshed its stat data.
3.  The next landing in that worktree settles the open batch,
    finds the witness different,
    writes one service log
    line,
    and marks the batch done.
4.  The index still describes the state before the commit,
    so the commit's added paths are staged deletions and
    its changed paths are staged as their old content.

Wrong result:
the next plain `git commit` in that worktree records the revert of the commit that just landed,
and no command the person ran said the index disagreed with the branch.
Reproduced (second block above).

The alternative was rejected for a good reason,
stated in the code:
a reset would discard staging that is not
the landing's.
What is missing is the telling,
and a durable record.

Suggested fix (a suggestion):
finish the batch,
but leave a record the wrapper reads,
such as `<seq>.index-left` next to the journal,
and print one line on the next commit in that worktree
naming the paths whose index entries the landed batch did not update and the command to re-stage them.
Marking the batch done without the record is what makes the state permanent and silent.

### 3. A commit whose hand-off never ran is never pushed, and nothing says so

`commit/commit_command.rs:263` writes the push record after the post-commit gate:
`landing/service_handoff.rs:117` (`enqueue_all`) then `:118` (`wake`).
Between `landed_oid` at `:222` and that write the commit is already on the branch.

Sequence:

1.  The service lands the commit and answers `Landed`.
2.  The wrapper is killed,
    or its terminal is closed,
    or its standard error pipe breaks,
    any of which the
    repository's rules count as normal on this machine (`AGENTS.md`,
    rule `NHP`).
3.  No push record is written and no wake is sent,
    so the commit stays local.

Wrong result:
the branch has a commit the remote does not,
and no command says so.
Nothing re-derives a push for a commit that is already landed:
the only producers of a push record are this hand-off and `hand_over_if_ahead`
(`push/auto_push_trigger.rs:419`),
which runs after a forwarded command that adds commits
(`invocation/entry_run_forwarded_actions.rs:69`),
such as merge,
pull,
cherry-pick or revert.
A later commit's own push record carries the earlier commit,
because it pushes the branch tip;
so the loss becomes permanent only when that commit is the last thing done from that worktree,
which is the common end of a session.

The decision record states the promise this breaks:
"Pushes are single-flight per branch.
A landed commit joins an in-flight push covering its OID or pushes the
branch tip itself,
and exits only once the remote contains its OID"
(`doc/decision/cli-git-concurrent-commits.md`,
"Auto-push").

Read from the code only:
no seam pauses a wrapper between the answer and the hand-off
(the wrapper's phase markers stop before the send,
`transaction/transaction_phase.rs:55`),
and a race with a kill is what this finding is about.

Suggested fix (a suggestion):
make the record the wrapper writes before it sends,
not after the answer.
The push record needs the landed commit,
which is not known before the landing;
so instead write one pending "push after landing" record that names the request identity and the branch,
and let the service resolve it against the journal when the batch ends.
Then a wrapper that dies anywhere after
the request cannot lose the push,
and the record is idempotent by identity like a work item.

### 4. A conclusion's cleanup is dropped when the client's scratch directory is gone

`landing/service_land_follow.rs:350`:
`remove_conclusion_state` answers success when the scratch directory named by the cleanup does not exist,
because the comparison it needs is impossible.
The directory belongs to the command's process,
and the service removes it when that process's watch connection closes
(`service/service_server.rs:470`,
`remove_tree`),
or the age sweep does.

Sequence:

1.  A commit concludes a merge,
    a cherry-pick or a revert.
    The request names a cleanup
    (`landing/service_prepare.rs:244`).
2.  The branch moves.
    The cleanup fails once:
    `reproduce_conclusion_cleanup` errors on one state file,
    or `carry_stash` fails,
    or the comparison errors (`landing/service_land_follow.rs:357`,
    `:360`).
    The batch stays open and the client is told the commit landed.
3.  The wrapper exits,
    and the service removes the scratch directory.
4.  The next landing settles the batch.
    `cleanup_done` (`landing/service_land_finish.rs:65`) finds no scratch,
    answers true,
    and the index update finishes.

Wrong result:
the worktree keeps `MERGE_HEAD`,
`MERGE_MSG`,
`AUTO_MERGE`,
`MERGE_RR` and `sequencer/`
after the commit that concluded the merge,
so the next `git commit` in that worktree concludes the merge a
second time,
adding the merged commit as a parent it already has,
and `git status` says a merge is in progress.
Reproduced at the mechanism (fourth block above):
success,
and the files stay.

Suggested fix (a suggestion):
make the directory the cleanup compares against survive the command.
The comparison needs the copied state files,
not the client's private index;
the service could copy the conclusion state into the journal when it writes the intent,
recorded in the same durable write,
and compare against that.

### 5. A batch that never lands parks its commit on a ref nothing removes or reports

`landing/service_land_finish.rs:363` (`keep_what_the_branch_does_not_show`) and
`landing/service_land_git_steps_and_import_stream.rs:42` (`keep_on_a_side_ref`):
a batch that will never land keeps its commit on `refs/cli-git/kept/<seq>` before the intent goes.
`rg 'cli-git/kept'` over `src/` finds the name only in the writer,
in the scratch-ref rules,
and in `is_side_ref_name`,
which accepts it as one of the wrapper's own side references
(`transaction/transaction_git_step_rerun_and_end_rules.rs:516`).
Nothing lists,
reports or removes them.

Sequence:

1.  A landing is killed after its objects exist and before its branch move (reproduction 3 shows the shape with
    the branch moved back instead,
    which recovery decides identically).
2.  Recovery decides `Drop`,
    writes the kept ref,
    and removes the intent,
    the index record and the scratch ref.
3.  The client resends,
    prepares again,
    and lands its commit normally.

Wrong result:
the repository keeps one reference per dropped batch forever,
because a reference keeps its
commits alive against `git gc`;
the line that says where the commit is goes to the service journal,
not to any
command's terminal;
and the kept commit is a duplicate of one that did land.
On a worktree with many killed
commands the ref namespace and the object store grow without bound,
and a `git push --mirror` from this build would carry them
(the owner's later pick to leave wrapper refs out of a mirror push is recorded in
`doc/handover/cli-git-rust-implementation.md`,
"Never fail,
and what is outside our control",
and is not in `main`).

Suggested fix (a suggestion):
make the kept ref a decision,
not an always.
Keep it only when the commit is not
reachable from the branch the client will land on,
and have `doctor` list kept refs older than a bound with the
command that removes them.

## What I checked and found sound

- The journal's one durable write (`write_intent`,
  file flush then rename then directory flush,
  `service/service_journal.rs:209`) is written before the branch moves,
  so after any kill the branch decides
  whether the batch landed.
  Recovery reads the branch,
  not the record,
  to decide
  (`service/service_journal.rs:94`,
  `recovery_for`),
  and treats "no answer from Git" as "keep the batch open"
  (`landing/service_land_finish.rs:499`,
  `current_value`).
- A batch killed after the branch move is finished without landing twice:
  `mark_indexed` is written while the lock is still held (`landing/service_index_hold.rs:288`),
  so a restart
  never replays an installation over what was staged after the lock was given up.
- The index install is one rename of a file built beside the live index,
  with the old index readable
  throughout,
  and the copy being installed is flushed first unless the in-process update already did
  (`landing/service_index_hold.rs:283`).
- The lock's release order is right:
  rename,
  directory flush,
  record,
  remove the lock.
  A kill anywhere in it leaves a lock that names a dead process,
  which the next attempt reclaims,
  and never an
  installed index that is unlocked and replayable.
- Reclaiming a stale lock cannot remove a live one:
  the reclaimer opens the file,
  judges the marker from that
  open file,
  takes its place in the claim chain,
  then checks that the name still names that file before
  unlinking (`landing/service_index_reclaim.rs:117`).
  The claim chain passes over a dead claimer and never waits
  for one.
  A live holder is never removed;
  a foreign or earlier-release file is named,
  not removed.
- Batch numbers cannot be reused while any record of them survives:
  a number is taken by exclusive creation and
  skipped when an intent,
  a done record or a scratch record exists (`service/service_journal.rs:135`,
  `:173`).
- Two worktrees of one repository cannot move the same branch:
  Git allows one checked-out branch per worktree,
  and the landing checks it twice,
  before and under the lock.
- The service's memory of landed requests is bounded at 1024 finished batches and pruned at start
  (`service/service_journal.rs:557`),
  and a withdrawal is answered on the landing thread,
  so a request is either
  landed before its withdrawal or dropped and never landed (`service/service_server.rs:377` to `:395`).
- The client never lands twice for one identity:
  the same identity is resent,
  and the service answers a repeat
  from `landed`,
  from the journal's done records,
  or as a repeat inside one batch
  (`service/service_server_land_group.rs:230`).
- A stuck branch write cannot lose the commit:
  the write is not ended,
  the scratch ref holds the objects until
  the write returns,
  and the clients behind it are told what it is stuck in and end at their own ceilings.
- The work queue needs no lock:
  a claim is a rename under a name that carries a generation and a clock reading,
  a renewal is a rename of that exact name,
  and a settlement is a rename of that exact name,
  so a takeover and
  a settlement cannot both happen (`landing/work_item.rs:597`,
  `landing/service_claim_renewal.rs:407`).
- Two writers in one worktree of the same branch are impossible by construction:
  one systemd unit per worktree,
  one main process per unit,
  and the wrapper is never started by itself
  (`package/git-policy/cli/SPEC.md`,
  "One marker-writing taker at a time").
- Preparation runs with no lock,
  so two commits prepare in parallel,
  and the only shared file,
  the real index,
  is copied before the landing touches it;
  the landing installs a whole file by rename,
  so a copy is never torn.

## What I could not determine

- Whether an intent file in the current format can in practice be left half written.
  `write_intent` flushes the file before the rename,
  so only a filesystem that loses a renamed file's tail,
  or a release that changes the format,
  produces the state finding 1 needs.
  The reproduction assumed it rather
  than produced it.
- Whether a `git status` refreshes the index often enough to be the trigger of finding 2 on a worktree with no
  stat work to do.
  I proved the gate leaves the index as it is when the index differs,
  and used `git add` in the
  reproduction;
  which ordinary commands write the index in a given worktree was not measured.
- Whether the kept refs of finding 5 are ever removed by a command outside this package's sources;
  I searched
  this crate only,
  and `doctor` does not mention them.
- Whether the earlier release's helper and this release's service can both land for one worktree in a shipped
  state,
  and what their two journals would then say.
  The code admits the overlap and both serialize on
  `index.lock`;
  no test covers the pair.
- How long a worktree's queue is stalled by a stuck branch write in practice;
  the timing logs contain no such
  case (`service/wait_ceilings.rs:94` records the four commits that met a lock that never went,
  not a write).
- The behavior of these paths on a repository on a network filesystem where `renameat2` and hard links are
  refused:
  `service/service_busy.rs` documents a checked rename for that case and I did not exercise it.
