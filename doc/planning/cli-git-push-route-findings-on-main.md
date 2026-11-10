# Push-route findings on `main`

Status:
audit,
2026-11-09.
Audited revision:
`main` at `554f973343`,
in the worktree
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-push-route-findings-on-main`.
No production code was changed and no test was committed.
The six throwaway reproductions ran from a scratch test sidecar that was deleted before the commit;
each is quoted in "Reproductions" with the exact code it called.
Scope:
the publishing route only.
An explicit `git push`,
the push record a landing or a merge leaves,
the push queue,
the retry plan,
the reconcile of a moved remote branch,
and the Git LFS step in front of a push,
read only as far as it can block or lose a push.
The code is
`package/git-policy/cli/src/native/push/`
(75 files,
11,905 lines without its 34 test sidecars)
and about 3,200 lines of the modules the route depends on
(`src/native/landing/service_work*.rs`,
`src/native/landing/work_item*.rs`,
`src/native/lock/push_lock_clearing.rs`,
`src/native/worktree/worktree_removal_rescue.rs`,
`src/native/push/stacked_merge_push.rs`).
Linux only,
as the owner's pause covers Windows and macOS.

Purpose:
the handover's "Open review findings" records nine open findings against the copy route
and nothing about the push route,
and the copy audit (`doc/planning/cli-git-copy-route-findings-on-main.md`)
found defects that no test had caught,
mostly what a process killed between two steps leaves behind.
This page does the same for publishing:
it walks every state change,
says what a kill just before and just after it leaves on disk,
lists every wait and every lock,
looks for races,
and reproduces what a seam can reproduce.
Nothing here is already recorded in the handover's open findings.

## What the route is

Two entrances,
one queue.

After a landing,
the client writes its work records and then wakes the service
(`src/native/landing/service_handoff.rs:76-118`):
`push-<oid>` and `maintenance-<oid>`,
and a `review-<oid>` where the repository keeps a shadow log,
under `<git_dir>/monochromatic-git-policy-cli/work/pending/`.
The records are durable before the socket is touched,
so a service that never wakes still finds them.

After a `merge`,
`pull`,
`cherry-pick`,
`revert` or `am` that moved the branch ahead of its upstream,
the wrapper writes only the `push-<oid>` record and wakes the service
(`src/native/landing/service_handoff.rs:137-150`,
reached from `src/native/push/auto_push_trigger.rs:389-424`).

An explicit `git push` that Git would accept as an ordinary branch update,
or one in a repository with LFS keys,
runs as a child of the wrapper with the caller's streams,
and on failure the wrapper re-derives the updates and may merge and publish
(`src/native/push/push_stacking.rs:252-320`).

The service is one instance per worktree.
A round
(`src/native/landing/service_work_run_round.rs:98-124`)
takes the pushes that are due again out of its retry plan,
drains every named repository side by side,
claims the pending records by atomic rename,
coalesces the records of one branch into one push,
runs one push per branch,
settles each record,
writes a record for the person when a push failed,
and applies the workers' retry plans on its own thread.

The files the route creates or removes:

- `<git_dir>/monochromatic-git-policy-cli/work/{pending,recovering,claimed,done,failed}/push-<oid>`
  (`src/native/landing/work_item.rs:88-90`).
- `<git_dir>/monochromatic-git-policy-cli/notices/push+<branch key>+<oid>.txt`
  (`src/native/push/push_failure_notice.rs:135-140`).
- `%S/monochromatic-git-policy-cli/service/<instance>/push-retries/<key>~<attempt>~<hash>.plan`
  (`src/native/push/push_retry.rs:53-56`,
  `src/native/push/push_retry_store.rs:71-80`).
- `<common>/monochromatic-git-policy-cli/orphaned-notices/`
  and
  `<common>/monochromatic-git-policy-cli/removal-snapshots/`
  (`src/native/worktree/worktree_removal_rescue.rs:38-90`).
- `<git_dir>/monochromatic-git-policy-cli-merges/<id>.json`,
  the intent that guards the fast-forward after a published reconcile merge
  (`src/native/push/auto_push_reconcile_branch_advance.rs:218-240`).
- the pushed remote ref,
  `refs/remotes/<remote>/<branch>`,
  `refs/cli-git/auto-push/fetch-<pid>`
  (`src/native/push/auto_push_reconcile_basics_and_limits.rs:150-176`),
  and whatever `*.lock` files Git takes on the way.

## Findings ranked by what they risk losing

The first two findings each leave a branch of a landed commit unpublished,
with no record that says so,
until the person's next commit on that branch carries it along.
The first needs no kill at all;
the second needs a kill in a window of a few file writes.
In the first the commit's objects are already on the remote when another branch pushed them,
so what is missing is the branch;
where nothing pushed them,
neither is there.
The third and fourth are waits that are believed to be bounded and are not;
the fifth moves a branch the reconcile never checked.
The sixth is a record that is believed to be shown and is deleted first.
The rest leave a merge unqueued when no route exists,
retry a cause that cannot clear,
hide a stale local view,
abandon a push silently after the service gives up,
leak a ref,
or spin a task.

### 1. A push record is keyed by the commit alone, so the second branch that reaches a commit is never pushed

`src/native/landing/work_item.rs:88-90`:

```rust
//! Quoted from src/native/landing/work_item.rs:86-90.
/// The id and file name:
/// the kind and the landed object,
/// so the same landing always yields the same id.
pub fn id(&self) -> String {
    return format!("{}-{}", self.kind.name(), self.oid);
}
```

The id holds the kind and the commit and nothing else:
not the branch,
not the repository.
`enqueue_all` treats an id that already exists in `pending`,
`claimed`,
`recovering`,
`done` or `failed` as already known
(`src/native/landing/work_item.rs:262-283`),
and answers `Enqueued::Known` without writing anything.

The sequence:

1.  A commit lands on `refs/heads/topic` and its push record `push-<C>` runs and settles `done`.
2.  In the same worktree the person switches to `refs/heads/release`,
    whose upstream is `origin/release`,
    and runs `git merge --ff-only topic`
    or `git pull` with a fast-forward.
    The branch tip becomes the same commit `C`.
3.  `auto_push_trigger::hand_over_if_ahead` tests the branch and finds it ahead of its upstream
    (`src/native/push/auto_push_trigger.rs:389-424`),
    builds the record for `refs/heads/release` at `C`,
    and `hand_over_push` enqueues it
    (`src/native/landing/service_handoff.rs:137-150`).
4.  `enqueue` finds `done/push-<C>` and answers `Known`,
    then the wrapper still wakes the service.
5.  The service finds nothing pending for that branch and pushes nothing.

The wrong result:
`refs/heads/release` and `origin/release` stay apart with no failure record anywhere,
because nothing failed.
The person is told nothing,
and the branch is published only when a later commit on it is pushed,
by a later push record with another commit,
or by hand,
and not at all if the worktree goes away first.
The record that does exist names `refs/heads/topic`,
so what the queue carries belongs to a different branch than the one that moved.
The commit's own objects reached the remote when `topic` was pushed;
what is missing is the branch,
not the backup of the work.

The queue collision is reproduced;
the trigger that hands the second branch over is read from the code,
because the reproduction stops at the queue.

Reproduced,
in "Reproductions" A.

Suggested fix,
a suggestion only:
make the id `push-<oid>-<hash of the branch ref>`,
or add a second,
cheap check that the existing record's branch is the one being handed over,
and enqueue a second record when it is not.
A record already holds the branch it was made for
(`ref` in the JSON `WorkItem::encode` writes,
`src/native/landing/work_item.rs:93-112`),
so a key over it needs no format change.

### 2. The retry the notice promises is written after the notice

The notice the person reads says:

```text
monochromatic-git-policy-cli: the commit service tries this push again by itself in 10 seconds (try 2),
and keeps trying with longer pauses until the push lands or a conflict needs you.
```

(`src/native/landing/service_work_handlers_replanning_and_acceptance.rs:315-345`).

The order of the writes,
and of the settlement between them,
is:

1.  `planned_retry` appends a `RetryChange` to the drain's own `Replanning` list,
    which lives in that worker's memory,
    and returns the words
    (`src/native/landing/service_work_handlers_replanning_and_acceptance.rs:315-345`,
    called at `src/native/landing/service_work_drain_pending_claims.rs:134-138`).
2.  `record_push_failure` writes the notice,
    with those words,
    into the notice directory
    (`src/native/landing/service_work_drain_pending_claims.rs:139-143`).
3.  The record is settled to `failed`
    (`src/native/landing/service_work_drain_pending_claims.rs:144`).
4.  Only when the worker returns does the service loop write the plan to disk,
    in `PushRetries::apply`
    (`src/native/landing/service_work_run_round.rs:114`,
    `src/native/push/push_retry.rs:432-450`).

A process killed between step 3 and step 4 leaves the notice and no plan.
The window starts at step 3,
not at step 2:
a kill after the notice and before the settlement leaves the record claimed,
and its lease brings it back for a new push with one more try
(`src/native/landing/work_item_recovery.rs:203-283`),
so the promise of the notice is kept by recovery.
After the settlement the record is terminal,
and only step 4 would carry it.
The window is wider than one worker's return:
`apply` runs after `drain_directories` has answered for every named repository
(`src/native/landing/service_work_run_round.rs:104-114`),
so another repository's slow push lengthens the exposure.
A restarted service loads no plan for that branch
(`src/native/push/push_retry.rs:401-411` reads the directory),
and the record sits in `failed`,
which nothing revisits.

The plan file is not lost in the same way without a kill:
`PushRetries::plan` adds the branch to the plan in memory before it saves
(`src/native/push/push_retry.rs:432-441`),
so a failed save costs the retry only at the next restart,
which then finds no plan for it.
A full disk or an unwritable journal therefore makes the notice untrue later,
not at once.

The wrong result:
the person is told the service keeps trying,
while the only durable statement of a push that has not been backed up says the opposite,
and the persistence that record depends on was never attempted before the record was written.
The failure is visible once (the next command shows the notice and removes it)
and then only in the journal,
where nothing points at the missing plan.

Reproduced in part,
in "Reproductions" B,
with the positive control that shows a plan written by the loop survives a restart:
the reproduction drops a worker's changes before they are applied,
and the settlement to `failed` that ends recovery is read from the code.

Suggested fix,
a suggestion only:
write the plan first (the service loop could apply a `RetryChange` synchronously for the one branch before the notice),
or word the notice from what is on disk after `apply`,
or settle the record back to `pending` when the plan could not be written,
so the next drain tries it again instead of leaving it in `failed`.

### 3. The reconcile's own Git children have no bound at all

`src/native/push/auto_push_reconcile_merge_commit.rs:31-100`.
`PushRunner::run` starts Git with `stdin` piped,
takes its output with `child.wait_with_output()`
(`:67`),
and has no window,
no meter,
no stop arm and no ceiling.
It is the runner for `git merge-tree --write-tree`
(the tree of the reconcile merge,
`:119-145`),
for every command of the word-level second pass
(`src/native/push/auto_push_reconcile_merge_commit.rs:150-200`),
and it also joins its input thread with no bound.

Every other Git child the service starts for a push is bounded:
the push and the fetch by the progress meter and the stall window
(`src/native/push/bounded_push.rs:408-445`),
the queries by the same meter with `without_wall_fallback`
(`src/native/push/bounded_query.rs:156-215`),
the commit's steps by `child_watch`
(`src/native/push/child_watch.rs`).
The children of a command the person typed are not watched,
by design:
the person is at the terminal and can end them,
and that includes the reconcile the explicit push runs in process
(see the wait list).
This one is not.

The sequence:

1.  A push is refused as stale and the reconcile fetches the remote tip and computes the merge.
2.  `merge-tree` or a second-pass command enters an uninterruptible or simply endless state:
    a stalled network filesystem the object store sits on,
    a Git that waits on a lock inside an object alternates file,
    or any of the crashes and hangs `AGENTS.md` rule `NHP` calls normal here.
3.  `wait_with_output()` never returns.
    The push worker never returns,
    so the drain never settles the record,
    and the claim renewal thread keeps stamping the claim
    (`src/native/landing/service_claim_renewal_running.rs:235-264`,
    a task on the multi-threaded runtime started at
    `src/native/landing/service_work_drain_pending_claims.rs:33-40`).
4.  Because the claim never expires,
    `requeue_expired_reporting` never takes the item back
    (`src/native/landing/work_item_recovery.rs:203-283`),
    and the service's stop arm never sees this child,
    because nothing on this path registers with `begin_unfinished_work`,
    which the push and the queries do
    (`src/native/push/bounded_push.rs:416`,
    `src/native/push/bounded_query.rs:164`).

The wrong result:
every later push of that repository waits behind the stuck worker,
forever,
and the person's commits stay local with no notice and no retry while the service appears healthy.
This is the one finding that turns a stuck child into a permanently stalled queue,
and it is the direct opposite of the owner's
"nothing should ever wait forever or indefinitely"
(`package/git-policy/cli/SPEC.md:2734`).

Reproduced in part,
in "Reproductions" E:
a fake Git that sleeps four seconds is waited for the whole four seconds,
and the runner takes no window to be stopped at.

Suggested fix,
a suggestion only:
run these children through the same bounded wait as `bounded_query`
(the meter,
the stop arm and `STOP_GRACE`),
and count them with `begin_unfinished_work` so a service stop ends them.

### 4. Every push query is unbounded where the kernel does not count a process's bytes

`Clock::check` decides a run is stuck from three signs read through `/proc`
(`src/native/push/push_progress.rs:534-556`),
and where `/proc/<pid>/io` cannot be read it has no signs at all:

```rust
//! Quoted from src/native/push/push_progress.rs:541-545.
let Some(meter) = self.meter.as_mut() else {
    // Nothing is measured here, so the stall window is the whole deadline, unless the run must never end by wall time.
    return (self.wall_fallback && elapsed >= stall).then_some(StopCause::Deadline);
};
```

The push itself is not the problem:
`wait_until_done_noting` keeps the fallback
(`src/native/push/bounded_push.rs:230-240`),
so where nothing is measured its stall window becomes a fixed deadline,
30 seconds by default,
exactly as `push_deadline_and_patience_settings.rs:29-49` documents.
The queries are:
`run_bounded_query` waits through `wait_until_done`,
which clears the fallback
(`src/native/push/bounded_push.rs:204-214`,
`:212`),
so on such a system no sample ever returns a cause,
and no other bound is passed to it.
The stalled push and the connected wait
(`src/native/push/push_progress.rs:541-570`,
`CONNECTED_PUSH_QUIET_WINDOW`,
150 seconds)
are both behind the same early return,
so neither applies either.

`package/git-policy/cli/SPEC.md:2358-2372` says this case is handled:
"where `/proc/<pid>/io` cannot be read nothing is measured,
so no quiet window can end a run,
and only the ceilings do".
The ceilings it names are the ceilings of steps that can be run twice,
a repair tool,
the Markdown check and the review call.
None of them is a push or a push query,
so the sentence describes a ceiling this route does not have.

The wrong result:
on a host whose `/proc` hides task I/O accounting
(`hidepid`,
a kernel built without `CONFIG_TASK_IO_ACCOUNTING`,
a hardened container),
a `git config --list`,
`rev-parse`,
`merge-base` or `for-each-ref` that hangs
(a file the kernel never answers for,
a network filesystem the repository or the scratch lives on)
holds the push worker,
and with it the drain of that repository,
for good:
the record is never settled,
its claim is renewed forever,
and every later push of that repository waits behind it.
The query is the step the decision left without any ceiling,
because the decision assumed the meter would end it;
the reconcile's own children (finding 3) are the other.

Reproduced with the seam that stands for such a host,
in "Reproductions" D,
with the positive control in the same test.
Not reproduced on this host,
whose `/proc/self/io` reads.

Suggested fix,
a suggestion only:
keep the wall-clock fallback on for the push's queries,
as the push itself does
(a query is read-only,
so stopping it is safe,
and the caller reports the query as stopped and retries the push).

### 5. The post-reconcile fast-forward moves whichever branch is checked out when it runs

`src/native/push/auto_push_reconcile_branch_advance.rs:148-215`.
`advance_local_branch` checks that the worktree holds the branch that was pushed,
at the tip that was pushed,
and then runs `git merge --ff-only <merge>`,
which moves the branch `HEAD` names *at that moment*:

```rust
//! Quoted from src/native/push/auto_push_reconcile_branch_advance.rs:72.
let moved_arguments = argv(&["merge", "--ff-only", merge_oid]);
```

No operand names the branch,
and nothing between the check and the run holds `HEAD` still.
An ordinary `git switch` takes `index.lock`,
but only for itself:
the switch may finish between the check and the merge,
and the merge then runs on the new branch.

The sequence:

1.  Branches `A` and `B` both point at `C`;
    the worktree has `A` checked out,
    and `A` is the branch the service pushed.
2.  The push is refused as stale,
    the reconcile publishes the merge `M` of `C` and the remote tip,
    and calls `advance_local_branch(git, refs/heads/A, C, M)`.
3.  The branch check and the tip check pass for `A`,
    and so do the status read and the path overlap test when the worktree is dirty.
4.  A person's `git switch B` runs and finishes:
    `HEAD` now names `refs/heads/B`,
    which also holds `C`.
5.  `git merge --ff-only M` runs and fast-forwards `B` to `M`,
    moving `B` and the index and worktree to `M`.

`settle_failed_merge` is not reached,
because the merge exited 0,
so the intent record that names `A` is removed
(`src/native/push/auto_push_reconcile_branch_advance.rs:71-145`),
and the report says:

```text
A and the worktree were fast-forwarded to it.
```

The wrong result:
a branch this command never checked is moved and its worktree now holds another branch's commit,
`A` stays behind the remote,
and the person is told `A` was fast-forwarded.
The merged content is not lost:
`M` contains `C` and the remote tip,
so `B` is a fast-forward of itself plus the remote's commits,
but `B` was published nowhere by this run and its next push is a merge the person did not ask for.
This is the route's one way to write to the wrong place.

Read from the code only.
A deterministic reproduction needs a seam between the check and the merge,
and there is none:
the check and the run are two statements of one function.
The wrong branch is Git's own behaviour for a branchless `merge --ff-only`,
which the function's checks are supposed to fence but cannot.

Suggested fix,
a suggestion only:
make the move name its branch (`git update-ref` on `refs/heads/A` with the old value as the guard)
and move the index and worktree with a checkout that re-checks `HEAD` under the same hold,
or re-read `symbolic-ref HEAD` immediately after the merge and report the mismatch as a refusal to advance.

### 6. The failure notice is deleted before it is shown

`src/native/push/push_failure_notice.rs:612-645`:

```rust
//! Quoted from src/native/push/push_failure_notice.rs:627-635.
// The record is removed before its text is used, so two commands that start together cannot both show it.
if std::fs::remove_file(path.as_path()).is_err() {
    continue;
}
texts.push(match parsed {
    Some((branch, commit, RecordKind::Failed(reason))) => worded(branch, abbreviate.name_of(commit).as_str(), reason),
```

The commit's name is asked of Git after the file is gone,
and the text is returned to a caller that writes it to standard error
(`src/native/invocation/wrapped_command_notices_snapshots_and_manual_push.rs:105-160`).

The sequence:

1.  The service failed a push and left the notice.
2.  The person's next command reads it,
    removes it,
    and asks Git for the short commit name.
3.  The command is killed there
    (`AGENTS.md` rule `NHP`:
    a process killed at any instruction,
    a closed terminal),
    or its standard error write fails,
    or `name_of` runs Git and the command dies while that child runs.

The wrong result:
the only durable statement that a commit exists only on this computer is gone,
and the person was never shown it.
A journald line remains,
so the fact is not lost from the machine,
but nothing in the repository points at it any more,
and no later command can show it.

Reproduced,
in "Reproductions" C:
a `CommitNamer` that looks for its own record finds it already gone.

Suggested fix,
a suggestion only:
render the text first,
then remove the record,
and let two commands show the same notice rather than lose it;
or rename it aside and remove it after the write.

### 7. A merge that lands commits leaves nothing durable when no route to the service can be built

`src/native/push/auto_push_trigger.rs:389-424`.
`hand_over_if_ahead` records the decision,
prints Git's failure,
and returns:

```rust
//! Quoted from src/native/push/auto_push_trigger.rs:398-403.
let Some(Route { address, provision }) = route_from_environment(...) else {
    record_ending(watch, command, Outcome::Declined, "no-route", "no way to reach the commit service");
    let _written = writeln!(stderr, "{}", render(&ClientFailure::NoActivation(platform_without_manager())));
    return;
};
```

The trigger returns before anything is written only when `route_from_environment` answers nothing
(`src/native/landing/service_route.rs:122-151`):
no `XDG_RUNTIME_DIR`,
or a Git directory that cannot be resolved,
or neither a session bus nor a socket file,
or no executable of the wrapper's own and no socket file.
Once a route exists,
`hand_over_push` writes the record and then wakes
(`src/native/landing/service_handoff.rs:137-150`),
so a service that is not running,
or a socket that refuses the message,
costs nothing:
the record stays pending and the next wake or the next service start finds it.
The commit path does the same and needs no route at all.
The merge path needs none either:
the work directory is a directory of the worktree's own Git directory.

The wrong result:
a merge,
pull,
cherry-pick,
revert or am that adds commits in a context with no route
(a non-systemd session,
a container,
a different `XDG_RUNTIME_DIR`,
a first command whose bus and socket are both absent)
leaves those commits local,
with one line on standard error,
a `push_trigger_log` entry,
and no retry.
The person's next commit carries them,
so this ranks below the first two,
but a handover that failed is the one case where the design's own rule
("work already handed over stays on disk and the next run finishes it",
`SPEC.md:2127-2135`) does not apply,
because nothing was written.

Read from the code only.
A live end-to-end reproduction needs an environment with no route and a merge,
which this audit did not build.

Suggested fix,
a suggestion only:
enqueue the record before the route is read,
and let the wake be the best effort it already is.

### 8. A push the LFS step refuses is retried forever

`src/native/lfs/lfs_bounded_push.rs:98-145` returns a failed attempt,
before any push,
when the repository has LFS keys and the upload cannot start or fails:

- no `git-lfs` on the service's `PATH`
  (`src/native/lfs/lfs_push_precheck.rs:45-70`,
  the message at `src/native/lfs/lfs_pre_push.rs:279-290`);
- a plan that cannot be worked out
  (`src/native/lfs/lfs_bounded_push.rs:101-125`);
- a `git-lfs push` that exits nonzero
  (`src/native/lfs/lfs_upload.rs:318-370`).

The first two of those are local and cannot clear by themselves:
a program missing from the service's `PATH`,
and a configuration the plan refuses.
The third is an upload whose cause may be as transient as the network,
and it is classified by its own words,
which is the behaviour the owner chose for a cause that can pass.

For the local two,
the service classifies the output in `final_failure`
(`src/native/push/push_retry.rs:162-200`):
it looks for the conflict phrase,
`[remote rejected]` with `hook declined`,
and five login phrases.
The missing-program message carries none of them
(`src/native/lfs/lfs_pre_push.rs:279-290`),
so the verdict is `Retry`,
the plan doubles up to `LONGEST_PAUSE`
(600 seconds,
`src/native/push/push_retry.rs:58-62`),
and the service tries again,
every ten minutes,
for as long as the program stays missing.
The owner's rule is that a retry is "at most the last resort for a cause outside the system"
and that a cause that cannot clear "is named and why it cannot be designed out is said"
(`SPEC.md:2121-2135`),
and `SPEC.md:2283-2290` lists a login and a server hook as the two local causes that stop.

The wrong result:
for a missing `git-lfs`,
the retry plan never empties,
the notice is rewritten every ten minutes,
and the branch is never pushed while the notice keeps saying the service keeps trying,
which is true and useless.
Installing the program clears it at the next try,
so the cost is an unending retry of a cause the person can fix,
and a state that never reaches "the person must act".

Read from the code only:
the LFS step is paused by the owner,
and this audit did not run an LFS repository.

Suggested fix,
a suggestion only:
a `FinalFailure` class for a missing `git-lfs` and for a plan this release refuses,
with the words that name the program and the ways forward the message already gives;
the first failure then stops the plan until the next commit,
as a login does.
A failed upload keeps the classification it has.

### 9. A tracking-ref lock refusal is dropped from the output after 1.5 seconds

`src/native/push/auto_push.rs:270-283` repeats the push while Git says
`cannot lock ref`,
three times,
with pauses of 250,
500 and 750 milliseconds,
and then stops looking.
The push itself has usually delivered by then,
so the attempt exits 0 with the refusal in its output.
`push_once` then filters the output for a zero exit
(`src/native/push/auto_push.rs:116-131`),
which keeps only `remote: ` lines and the wrapper's own reconcile lines,
so the refusal never reaches the person or the failure record,
and the verdict is `Done`.

The service treats `Done` as a whole push
(`src/native/landing/service_work_drain_pending_claims.rs:122-132`):
it clears every earlier failure notice of that branch with a cutoff of the moment the push began
(`src/native/push/push_failure_notice.rs:469-500`).

The wrong result:
`refs/remotes/<remote>/<branch>` stays where it was,
`git status` in that worktree reports the branch ahead of its upstream by commits the remote already has,
nothing says why,
and the notices that would have explained an earlier failure are deleted on the strength of a push whose
own bookkeeping Git refused.
The refusal is only reachable when another process holds that reference's lock,
which in practice is another instance of this service pushing the same branch
(the comment at `src/native/push/auto_push.rs:272-276` says so)
or a person's own `git fetch`/`git push`.
The three pauses total 1.5 seconds of sleeping,
and the exposure is those sleeps plus the run time of the four pushes;
the refusal survives when the holder still has the lock at the last attempt's
tracking-ref update.
The 7 second margin this repository uses for a lock it will not touch
(`src/native/lock/stale_ref_lock_judge.rs:64-90`) is a different question,
and it is not evidence that the holder outlasts four pushes.

Read from the code only.
It is reproducible with a fixture that holds `refs/remotes/<remote>/<branch>.lock`
for longer than the four attempts take and then releases it,
which this audit did not build.

Suggested fix,
a suggestion only:
report the refusal as its own line when it survives the retries,
and do not clear the branch's notices on a push whose tracking ref Git refused.

### 10. The notice of a push the service gave up on is written after the state that ends the obligation

`src/native/landing/service_work.rs:153`:

```rust
//! Quoted from src/native/landing/service_work.rs:153.
tell_about_given_up(git_dir, requeue_expired_reporting(work_dir.as_path(), now, lease, MAX_TRIES).given_up.as_slice());
```

The three deaths of a holder that end a push are one call and one notice:
`requeue_expired_reporting` moves the record to `failed` and returns it,
and `tell_about_given_up` writes the notice that names the branch
(`src/native/landing/service_work_handlers_replanning_and_acceptance.rs:226-240`).
A kill between the two leaves the record in `failed`,
which nothing scans for:
recovery looks at `claimed` and `recovering` only
(`src/native/landing/work_item_recovery.rs:203-233`),
so the next drain's `requeue_expired_reporting` returns nothing to tell,
and no later command ever learns that the branch was not pushed.

The wrong result:
a branch abandoned after three deaths is abandoned silently,
and the rule that every failure is told
(`SPEC.md:2113-2135`) fails exactly where it is the only remaining word.
The same three deaths give a notice when the service is not killed in that one instant,
so this is a narrow window rather than the common case.

Read from the code only.
A reproduction needs a kill between two adjacent statements of one function,
which no seam reaches.

Suggested fix,
a suggestion only:
write the notice first and move the record to `failed` after it,
and let a repeated notice replace itself,
which `record_push_skip` already does
by removing the file of the same branch and commit before it writes.

### 11. The reconcile's private fetch ref leaks on a kill

`src/native/push/auto_push_reconcile_basics_and_limits.rs:145-176` fetches the remote tip into
`refs/cli-git/auto-push/fetch-<pid>`,
reads it,
and removes it:

```rust
//! Quoted from src/native/push/auto_push_reconcile_basics_and_limits.rs:175.
let _deleted = run_query(git, &argv(&["update-ref", "-d", &private_ref]));
```

The comment says a failed deletion leaves one small ref "that the next run overwrites".
It does only while the same service instance lives:
the name carries the process id,
and a resident service keeps one,
so a later reconcile in the same process writes the same name again and then deletes it.
After a restart the process id is another,
and the ref of the instance that died is overwritten by nothing.

The wrong result:
a kill of the service between the fetch and the deletion leaves a ref under `refs/cli-git/` for good,
and a release,
a crash or a unit restart does exactly that.
Nothing lists or sweeps that namespace,
the ref pins the objects it names against `gc`,
and a repository-wide push that is not restricted to `refs/heads/*` would carry it.
No user data is lost.

Read from the code only.

Suggested fix,
a suggestion only:
sweep `refs/cli-git/auto-push/` at the start of every reconcile and delete the refs whose fetch completed,
so a leaked one is taken by the next run;
naming it without the process id would need a rule for two worktrees of one repository
reconciling at once,
which is why the name carries the process id at all.

### 12. A detached sweep task waits with no ceiling

`src/native/push/stop_of_a_push_and_sweep_of_its_locks.rs:130-140`:

```rust
//! Quoted from src/native/push/stop_of_a_push_and_sweep_of_its_locks.rs:135-138.
// Nothing waits for this task, so it waits for the rest of the tree as long as that runs.
while tree.iter().any(still_running) {
    tokio::time::sleep(POLL_PAUSE).await;
}
```

`POLL_PAUSE` is 10 milliseconds
(`src/native/push/stop_of_a_push_and_sweep_of_its_locks.rs:49`).
Every other wait of a stop is bounded by `STOP_CEILING`
(`src/native/process/stop_waits_while_a_member_cannot_be_interrupted.rs:120-125`),
and this one is not.
The loop is entered only after the stopped push itself has been collected
(the `owned.wait().await` above it),
which is also what keeps a leader that never exits from being polled:
that case waits on the pidfd instead.
What is left is a descendant.

The wrong result:
a stopped push whose descendant never ends
(a process wedged in a call on a dead network filesystem,
or one that left the process group the kill reached)
leaves a task that wakes every 10 milliseconds for the life of the service,
doing nothing.
No push waits on it,
so this ranks last.

Read from the code only.

Suggested fix,
a suggestion only:
bound it by the same ceiling,
or drop the task and let the stale-lock judge handle what is left.

## Every state change, and what a kill leaves

For each step,
the state on disk,
and what notices it later.

1.  Work record written,
    before the wake.
    A kill before it leaves nothing,
    and nothing later revisits a commit the person landed:
    the person's next command does not announce a backup,
    and no push record exists.
    A kill after it leaves `pending/push-<oid>`,
    which the next wake or the next command's wake drains,
    and
    `oldest_pending_age`
    (`src/native/landing/work_item.rs:666-670`)
    is what the backlog notice is built from.
    The commit path writes the record before the socket,
    so this window is outside the route's control.
    The record is published whole or absent by a staged write and an exclusive link,
    but without a sync,
    so a power loss can leave an empty record where a kill cannot;
    the next drain cannot decode it and moves it to `failed` with a journal line and no notice
    (`src/native/foundation/private_storage.rs:418-434`,
    `src/native/landing/work_item.rs:504-545`).
    That discard is what the writer's own doc comment declares,
    and its cost is the same silent,
    unpublished branch.
2.  Wake message sent.
    Not durable by design;
    the records are what matter.
3.  `pending` to `recovering`
    (`src/native/landing/work_item.rs:504-545`).
    A kill between the two renames leaves the record in `recovering` under a claim name,
    which the next drain finds by lease expiry and requeues with one more try
    (`src/native/landing/work_item_recovery.rs:203-283`).
    Sound.
4.  Stamped and renamed into `claimed`.
    A kill before the stamp puts the record back into `pending` with its own age
    (`src/native/landing/work_item.rs:504-545`).
    Sound.
5.  Claim renewed every 40 seconds.
    A kill stops the renewals;
    after the 120 second lease another instance runs the item with one more try.
    Three such deaths move it to `failed` and the next drain leaves a skip notice that names the branch
    (`src/native/landing/service_work_handlers_replanning_and_acceptance.rs:226-240`,
    `src/native/landing/work_item_recovery.rs:264-270`).
    Sound,
    and it is the answer to "nothing waits forever" for a killed holder.
6.  The push itself.
    Git owns the remote ref and the local tracking ref;
    a kill leaves whatever Git wrote before it died.
    Git's own lock files may survive
    (`src/native/lock/stopped_git_locks.rs:1-60`),
    and the next push or fetch judges them before it starts
    (`src/native/lock/push_lock_clearing.rs:50-70`).
    A delivered push is idempotent:
    the same push again finds everything up to date.
7.  Reconcile fetch.
    The private ref of finding 11.
8.  Merge commit built from objects.
    Loose objects only;
    a kill here leaves unreachable objects that `gc` may prune.
    No worktree,
    index or ref is touched
    (`src/native/push/auto_push_reconcile_merge_commit.rs:105-230`).
9.  Merge pushed.
    A kill after this and before the branch advance leaves the local branch behind the remote;
    nothing advances it later,
    and the next push of that branch is refused as stale,
    after which the reconcile reports "already contains ... nothing to push" and exits 0
    (`src/native/push/auto_push_reconcile.rs:295-310`).
    Self-healing on the next commit,
    which lands on the new tip and pushes the merge of the two.
    The published merge is not lost.
10. Branch and worktree fast-forwarded through `git merge --ff-only`,
    guarded by an intent record written first
    (`src/native/push/auto_push_reconcile_branch_advance.rs:71-145`).
    A kill between the branch write and the checkout is settled by the next command
    (`src/native/push/auto_push_reconcile_branch_advance.rs:148-215`,
    `src/native/merge/merge_isolation_recovery.rs:213-272`),
    which is sound;
    which branch the move lands on is finding 5.
11. Notice written,
    then the record settled,
    then the plan written.
    Finding 2 lives here;
    the give-up notice of finding 10 is the reverse order.
12. Notice shown:
    removed,
    then named,
    then written.
    Finding 6.
13. Retry plan saved,
    then the attempt run.
    `take_due` removes the plan from memory only
    (`src/native/push/push_retry.rs:464-480`),
    so a kill while a due retry runs keeps the file and a restart finds it.
    Reproduced as the control of "Reproductions" B.
14. Record settled `done` after a successful push,
    with the plan forgotten only when the round applies the workers' changes.
    A kill between the two leaves the record `done` and the old plan file on disk,
    so a restart pushes the branch again from the plan,
    which is idempotent,
    and then forgets it.
15. Worktree removed while records are unfinished or notices unshown.
    The wrapper copies both into a snapshot before Git runs,
    and a later command turns the copy into notices or discards it once the removal is proven
    (`src/native/worktree/worktree_removal_rescue.rs:1-90`,
    `src/native/invocation/wrapped_command_notices_snapshots_and_manual_push.rs:190-215`).
    Sound,
    and the `gitdir` file written last is what proves a snapshot whole.
16. Explicit push that lost a race,
    merged and published in process.
    The merge is pushed with the marker in the planned environment only
    (`src/native/push/stacked_merge_push.rs:48-120`),
    so the wrapper never starts itself
    (the owner's "We should never invoke ourselves recursively").
    A kill after the merge is published and before the branch advance leaves the same state as step 9.

## Every wait and its ceiling

The push route's waits,
with the bound each has.
The owner's rule is
"nothing should ever wait forever or indefinitely"
(`SPEC.md:2734`),
which `SPEC.md:2737-2740` says is newer than and overrides
"Wait while connected (Recommended)".

- The service's client wake and answer:
  none of its own;
  the wrapper only sends a datagram or short message
  (`src/native/service/service_client.rs:412-430`),
  bounded by `START_CEILING` and `ANSWER_CEILING` where it waits at all.
- The claim lease:
  120 seconds,
  renewed every 40 (`src/native/landing/service_work.rs:83-105`).
  A bound,
  not a wait.
- A push child,
  no connection:
  the stall window,
  `CLI_GIT_PUSH_DEADLINE_SECONDS`,
  30 seconds by default
  (`src/native/push/push_deadline_and_patience_settings.rs:29-60`).
- A push child with a connected,
  silent remote:
  `CONNECTED_PUSH_QUIET_WINDOW`,
  150 seconds
  (`src/native/service/wait_ceilings.rs:346-357`,
  used at `src/native/push/push_progress.rs:556-566`).
  Note the stale prose:
  the module doc of `push_progress.rs` and `SPEC.md:3665-3700`
  still say a connected push has no ceiling,
  which the code and `SPEC.md:2734` contradict.
- A push child that keeps moving bytes or processor time:
  **no ceiling**.
  By decision ("Stop only on no progress"),
  and the accepted cost is that a spinning Git is waited for.
- The push's read-only queries:
  the same stall window,
  and **no ceiling where the meter reads nothing** (finding 4).
- The reconcile's `merge-tree`,
  second pass and input thread:
  **no bound at all** (finding 3).
- The reconcile fetch:
  the push's stall window and the connected window.
- The retry pauses:
  10 seconds doubling to 600
  (`src/native/push/push_retry.rs:58-62`),
  and the plan is retried without end while the failure is retryable (finding 8 for one class that should not be).
- The explicit push's child:
  none;
  it is the person's command with the person's terminal,
  and the wrapper waits for it as Git does.
- The explicit push's in-process merge push:
  **none**.
  It is planned in process but run as a forwarded child of the wrapper
  (`src/native/push/stacked_merge_push.rs:101-120`,
  `src/native/worktree_copying/worktree_copy_forward.rs:171-184`),
  so it has the person's terminal and the same absence of a window as the push it follows.
- The stop of a stopped push:
  `STOP_GRACE` 5 seconds,
  extended to `STOP_CEILING` while a member of the tree is in uninterruptible sleep,
  then the forced end,
  then the detached sweep of finding 12 (**no ceiling**).
- The stale-lock judgement before every push and fetch:
  no wait at all;
  it judges and returns
  (`src/native/lock/push_lock_clearing.rs:50-70`).
- The LFS upload:
  the push's windows,
  per remote and per run.
- A missing Git to start:
  `START_RETRY_BUDGET`,
  5 seconds
  (`src/native/push/start_retry.rs:33`).

## Every lock, and why it exists

The push route takes **no lock of its own**:
no `flock`,
no `Mutex` around a push,
no lock file.
That is the owner's "Use as few locks as possible" carried to zero here,
and it rests on four properties:

- one coalesced push per branch per service instance,
  because the drain groups the pending records by branch and runs one push for the group
  (`src/native/landing/service_work_drain_pending_claims.rs:76-95`);
- the retry plan is owned by the service loop and changed only by it
  (`src/native/push/push_retry.rs:340-400`),
  so no worker shares it;
- work records move by atomic rename and settle under a name that carries their generation
  (`src/native/landing/work_item.rs:447-620`);
- the only plan file is written by one instance,
  "one instance is the only writer of its plan,
  so no lock is taken"
  (`src/native/push/push_retry_store.rs:1-40`).

Two of those are assumptions rather than proofs,
and both are safe only as claims about systemd:
an instance per worktree is a unit,
and no second instance of the same worktree runs.
The code elsewhere does not assume that:
claims are recognised as abandoned by lease,
and `drain_common_records` says several linked instances may drain one directory at once
(`src/native/landing/service_work.rs:185-205`).
Two instances of one worktree that both write the plan directory would delete each other's
files,
because `remove_files_of` removes every file of the branch key except the one just written
(`src/native/push/push_retry_store.rs:145-175`).
No lock can be removed here,
because none exists,
and none needs to be added.

The locks that are taken are Git's,
in the repository:

- the remote-tracking ref lock (`refs/remotes/<remote>/<branch>.lock`)
  that a push's tracking update takes,
  and the same lock a fetch takes;
- the packed-refs lock and the per-ref locks a fetch takes for the private tip ref
  (`src/native/push/auto_push_reconcile_basics_and_limits.rs:145-160`);
- `config.lock` for a push that writes an upstream with `--set-upstream`
  (`src/native/push/auto_push.rs:230-244`);
- `index.lock` for the post-reconcile `git merge --ff-only`
  (`src/native/push/auto_push_reconcile_branch_advance.rs:71-145`).

Two rules govern them.
The wrapper never waits for one of them without a bound in the push route:
the merge's refusal is read and reported instead of waited for,
and a push's own refusal is retried a fixed number of times (finding 9).
And the wrapper removes a reference lock only on a proof about its holder
(`src/native/lock/stale_ref_lock_judge.rs:64-90`,
`:130-200`:
a pid file naming a process that is gone,
a number reused by something that is not Git,
an orphaned `packed-refs.new`,
or a member of a tree this wrapper stopped and saw end),
never on age alone.

## Races between two commands, and between two worktrees

- Two commands in one worktree.
  The push path takes no lock,
  so a person's `git commit`,
  `git status` or `git fetch` runs beside the service's push.
  The one shared file the push writes through Git is the tracking ref,
  and the one shared file `advance_local_branch` writes is the index and worktree under `git merge --ff-only`.
  The fast-forward is guarded by a status read,
  an overlap test against the paths the merge changes,
  and a check that the worktree holds the pushed branch at the pushed tip
  (which finding 5 shows is not bound to the move),
  and `--ff-only` itself,
  which refuses rather than overwrite
  (`src/native/push/auto_push_reconcile_branch_advance.rs:148-215`).
  A refusal is reported and not retried.
  Sound.
- Two worktrees of one repository.
  They share references,
  `objects`,
  `packed-refs` and the remote-tracking refs,
  and they push independently:
  there is no per-branch coordination across instances,
  which is what the comment at `src/native/push/auto_push.rs:272-276` describes and finding 9 is about.
  Both may reconcile around the same remote tip.
  Each builds its own merge from objects and pushes it;
  the loser is refused as stale and its loop fetches again and merges onto the newer tip,
  up to `MAX_RECONCILE_ATTEMPTS` (10,
  `src/native/push/auto_push_reconcile_basics_and_limits.rs:49`).
  Nothing is forced and nothing is lost.
  Two merge commits may be published instead of one,
  and a local branch that another worktree moved is left behind with a reason rather than moved.
- Two worktrees and the notice directory.
  Notices live in each worktree's own Git directory,
  so a notice is shown in the worktree whose service made it
  (`src/native/push/push_failure_notice.rs:530-560`).
  Orphaned notices of removed worktrees are shown to whichever command asks first,
  which is intended.
- Two services and the same branch's tracking ref lock.
  Finding 9.
- A person's `git fetch` and the reconcile's fetch.
  The reconcile fetches into its own private ref and never reads `FETCH_HEAD`
  (`src/native/push/auto_push_reconcile_basics_and_limits.rs:145-176`),
  so a concurrent fetch cannot change the tip it reads.
  Sound.
- A person's `git worktree remove` and the service's push of that worktree.
  The records are copied into a snapshot before Git runs,
  and the service's claim on a record is looked for twice,
  in `pending` and in `claimed`,
  because it moves between them
  (`src/native/worktree/worktree_removal_rescue.rs:55-90`,
  `:160-260`).
  A push that was mid-flight when the worktree went produces a "lost" notice that says the commit
  may exist only on this computer,
  which is the honest statement.
  Sound.

## Reproductions

All six ran against the audited revision's own code,
from a scratch sidecar test module that was deleted before the commit.
The command,
run from `package/git-policy/cli`,
was the one the package task runs,
with the test filter added:

```text
systemd-run --user --scope --property=CPUQuota=400% --property=MemoryMax=12G \
  --property=TasksMax=1024 --property=IOWeight=10 \
  -- mise exec -- cargo nextest run --locked --offline --lib --no-capture -- push_route_audit_scratch
```

Result:
`Summary [4.109s] 6 tests run: 6 passed, 4821 skipped`.
Fixtures were the package's own disposable ones under `target/test-fixtures`,
which the test harness removes.
A,
B,
C,
D and F use a seam.
E measures a wall-clock wait,
and the paragraph on E says what that does and does not show.

### A. One commit, two branches

```text
second branch enqueue answer: Ok(Known)
id of the record the first branch made: push-0123456789abcdef0123456789abcdef01234567
id of the record the second branch asked for: push-0123456789abcdef0123456789abcdef01234567
records the service can claim for the second branch: 0
```

Code:
the first record is enqueued,
claimed and settled `done`;
the second record for another branch at the same commit is then enqueued.
It answers `Known` and nothing is left to claim.
Finding 1.

### B. A retry announced before it is kept

```text
monochromatic-git-policy-cli-service: push will be tried again branch="refs/heads/main"
oid=0123...67 next_try=2 in_seconds=10 cause: the network is down
first line of the record the person will read: monochromatic-git-policy-cli: the commit service tries
this push again by itself in 10 seconds (try 2), and keeps trying with longer pauses until the push
lands or a conflict needs you.
plans the worker asked for: 1
plans a restarted service loads: 0
plans on disk once the loop applied them: 1
wait the restarted service takes from them: Some(9.99991233s)
```

Code:
`planned_retry` then `record_push_failure`,
then the `Replanning` is dropped as a killed worker's memory is;
`load` finds no plan and a restored plan has no wait for the branch.
The last two lines are the positive control:
the same changes handed to the loop leave one plan on disk and a 10 second wait.
Finding 2.

### C. A notice removed before its text is made

```text
record on disk before the person's command: true
record on disk while its commit is being named: Some(false)
notices returned for the terminal: 1
record on disk after the call: false
```

Code:
`take_push_failures_for` with a `CommitNamer` that looks for its own record while it is asked for the
commit's name.
The third line is the control that the notice reached the caller,
so the removal is not a failure to find it.
Finding 6.

### D. A wait with nothing measured

```text
a clock that counts nothing, with the wall-clock fallback: Some(true)
the same clock without the fallback, which is what the push's waits use: false
```

Code:
two clocks with
`Patience::Progress { stall: Duration::ZERO }`,
both `unmeasured()`,
the second `without_wall_fallback()`.
The first ends the wait,
so the zero window is live in the test;
the second never does.
This is the state `wait_until_done` puts every push query in on a host whose `/proc/<pid>/io` cannot be read.
Finding 4.

### E. The reconcile's runner and a child that does not end

```text
the reconcile's runner returned after 4002 ms with code Some(0)
the stall window a push's own child is stopped at is 30 s, and this runner was given no window at all
```

Code:
`PushRunner::run` with a `real_git` that sleeps four seconds.
The runner waited the child's whole life and has no argument through which a window could end it.
What it shows is that the wait belongs to the child alone:
no window of any length is consulted.
What it does not show is a wrong result,
because a bounded runner with a window longer than four seconds would also have waited;
finding 3 is read from the code,
and this is the seam that shows where its wait comes from.

### F. The control for the plan's durability

```text
plans a restarted service loads: 1
```

Code:
`PushRetries::apply` with one `RetryChange::Plan`,
then `load` of the plan directory.
This is what a kill cannot take away once the loop has applied a plan,
and it is why finding 2 is about the order and not about the format.

## What I checked and found sound

- **The record's publish and claim.**
  Whole or absent:
  the bytes go to a staging file beside the target and are published by an
  exclusive link,
  so a kill leaves one or the other and never half a record
  (`src/native/foundation/private_storage.rs:377-395`,
  `src/native/landing/work_item.rs:256-283`),
  and the idempotency key makes two racing producers agree on one winner,
  except for the branch hole of finding 1.
- **The claim's lease and renewal.**
  A killed holder is noticed by its lease,
  never by a process id,
  and both sides change only names they own,
  so a renewal can never touch another instance's claim
  (`src/native/landing/work_item.rs:591-650`,
  `src/native/landing/service_claim_renewal.rs:1-60`).
- **Three deaths of a holder end with a notice that names the branch**,
  except for the one-instant window of finding 10.
  `MAX_TRIES` (3) with `tell_about_given_up`
  (`src/native/landing/service_work_handlers_replanning_and_acceptance.rs:226-240`).
- **A due retry is not lost by a kill.**
  `take_due` removes it from memory only;
  the file stays until the loop forgets it.
- **The plan file's replacement.**
  The new file is published whole before the old ones are removed,
  and the newest file of a branch wins on load,
  so a kill leaves one plan,
  never none
  (`src/native/push/push_retry_store.rs:122-145`).
- **The failure notice's name and content.**
  It is keyed by branch and commit,
  a repeat replaces it rather than piling up,
  only credentials are masked,
  and control characters are dropped
  (`src/native/push/push_failure_notice.rs:160-300`).
- **The notice is not shown to a machine reader**,
  and not taken by one either,
  so a script cannot consume a person's notice
  (`src/native/push/push_failure_notice_machine_output.rs:1-66`).
- **Every push child runs with the person's hooks off**,
  and every service push carries `core.lockfilePid=true` and the never-prompt pair
  (`src/native/invocation/child_environment.rs:200-300`,
  `:449-470`),
  so a push cannot wait on a prompt or on a hook nobody sees.
- **The stale reference lock judgement.**
  Four proofs and no age rule,
  a conditional removal,
  a local filesystem list so a lock written on another host is never judged here,
  and a 7 second margin before any judgement
  (`src/native/lock/stale_ref_lock_judge.rs:38-200`).
  This is the one place the route deletes a file it did not create,
  and it deletes only on evidence about the holder.
- **The stop of a stopped push.**
  The tree is read before the signal,
  the locks are claimed while their pid files still name the holders,
  a member in uninterruptible sleep gets `STOP_CEILING` instead of the grace,
  and the sweep runs only once the tree is gone
  (`src/native/push/stop_of_a_push_and_sweep_of_its_locks.rs:168-250`).
  Every wait in it is bounded except the detached one of finding 12.
- **The reconcile never touches the worktree or the index while it merges.**
  `merge-tree --write-tree` and `commit-tree` read and write objects only,
  and the second pass works on trees.
- **The post-reconcile fast-forward loses nothing and is settled after a kill.**
  The worktree must hold the branch at the pushed tip,
  uncommitted changes must not overlap the paths the merge writes,
  `--ff-only` refuses rather than overwrites,
  and the intent record makes recovery finish or discard the move
  (`src/native/push/auto_push_reconcile_branch_advance.rs:148-215`).
  It does not bind the move to the branch it checked,
  which is finding 5.
- **The explicit push never starts the wrapper again.**
  The merge push is planned in process with `CLI_GIT_STACKED_PUSH` in the planned environment only,
  and the marker stops a second stacking
  (`src/native/push/stacked_merge_push.rs:48-120`,
  `src/native/invocation/entry_push_stacking_and_lfs_upload.rs:150-170`).
- **The explicit push's policies see what is published.**
  The merge is pushed through `plan_invocation`,
  so the manual-push policies read the integrated result
  (`src/native/push/stacked_merge_push.rs:148-162`).
- **The worktree removal rescue.**
  Unfinished push records and unshown notices are copied before Git deletes the Git directory,
  the `gitdir` file is written last so a snapshot is whole or not taken,
  and a snapshot is settled only when the removal is proven
  (`src/native/worktree/worktree_removal_rescue.rs:1-90`,
  `:343-520`).
- **The manual push's scope is re-derived after a rejection.**
  The dry-run report and the `ls-remote` confirmation must agree,
  and a mismatch stops the probe instead of publishing something unscanned
  (`src/native/push/manual_push_probe.rs:195-260`).
- **A push that is delivered but whose remote moved.**
  The merged result is published without a force,
  the local branch follows only where nothing can be lost,
  and the boundary is `MAX_RECONCILE_ATTEMPTS` with a report that says the remote kept moving
  (`src/native/push/auto_push_reconcile.rs:224-400`).

Two smaller things worth knowing,
neither a loss:

- A fast-forward that Git refuses because `HEAD` is not the pushed tip is reported and not retried
  (`src/native/push/auto_push_reconcile_branch_advance.rs:150-170`),
  and the "already contains ... nothing to push" path
  (`src/native/push/auto_push_reconcile.rs:295-310`)
  reports success without advancing the local branch.
  Both self-heal on the next commit.
- `SPEC.md:3665-3700` and the module doc of `src/native/push/push_progress.rs` still describe
  "no ceiling" for a connected push,
  which `src/native/push/push_progress.rs:556-566` and `SPEC.md:2734` replaced with
  `CONNECTED_PUSH_QUIET_WINDOW`.
  The code is right and the prose is stale;
  a reader of the prose would believe a connected push is unbounded,
  which neither finding 3 nor finding 4 says.

## What I could not determine

- Whether a host whose `/proc/<pid>/io` cannot be read is a supported deployment.
  If it is not,
  finding 4 is a documentation defect in `SPEC.md` rather than a behaviour one;
  the sentence in `SPEC.md:2358-2372` is what led me to expect a ceiling.
- Whether two live pushes racing one reference lose finding 9's retry in practice.
  The four attempts and their 1.5 seconds of sleeping bound nothing about how long a holder keeps
  the lock,
  so a fixture can answer it and I did not build one.
- Whether the fast-forward of finding 5 is reachable under live concurrency.
  The window is the two statements between the `HEAD` check and Git's own merge,
  and I read it rather than raced it.
- What a kill during the settlement of a refused fast-forward leaves
  (`src/native/merge/merge_isolation_recovery.rs:213-272`).
  I read the state machine and did not exercise it;
  it is the merge route's,
  not this one's.
- Whether `git push --set-upstream` that cannot write `config.lock` exits nonzero.
  If it does,
  the service retries a push that already delivered;
  I did not run it,
  and the rule is to read the deciding source before saying.
- Whether the LFS step is reachable in this repository at all.
  The owner paused it,
  and I read it only as far as the refusal that ends an attempt before the push.
- The installed wrapper's behaviour:
  every claim here is about `main`'s code.
  No release was run,
  and the owner's pause covers Windows and macOS.

## What to inspect and how to respond

Inspect,
in this order:

1.  findings 1 and 2,
    against
    `src/native/landing/work_item.rs`,
    `src/native/landing/service_work_drain_pending_claims.rs`
    and
    `src/native/push/push_retry.rs`;
2.  findings 3 and 4,
    against
    `src/native/push/auto_push_reconcile_merge_commit.rs`
    and
    `src/native/push/push_progress.rs`;
3.  finding 5,
    against
    `src/native/push/auto_push_reconcile_branch_advance.rs`,
    because it is the one finding that writes to a branch this command never checked;
4.  finding 10,
    against
    `src/native/landing/service_work.rs`,
    because it is the only place a failure goes untold;
5.  the reproductions of "Reproductions",
    A to F;
6.  the waits and locks lists,
    for anything a ceiling should have covered and does not.

No production code was changed,
so nothing else is pending.
The three things a decision would unlock are
the record's key (finding 1,
a change to a durable name),
the move that names its branch (finding 5,
which changes what the reconcile reports as done),
and the bound for the reconcile's children (finding 3).
Records already on disk are not renamed by any of them,
so an old record keeps its name and is read as it is.
