# A slow write of a side reference failed a commit the service had made

On 2026-11-11 the release gate of `0.6.1-alpha.11`
read one commit as a failed commit when it had been made:
`service_reference_lock::a_held_commit_real_git_built_is_kept_on_the_side_reference_and_survives_a_prune`
(`package/git-policy/cli/src/native/binary/binary_service_reference_lock_tests.rs`)
FAILED after 177.143 s
with "The commit service could not finish this commit,
so it was not made",
and the same test passes alone in 13 to 18 s on the development host.

The gate log is
`~/temp/agent/gate/gpc-gate-alpha-11-on-82829a1696-20261111T070128-0500.log`,
which ran the suite in a container beside other agents' builds.

## What the service said

The service's own line for the request,
in that log,
with the fields this case does not need elided:

```text
answered request="1f239541-…" branch="refs/heads/main" outcome=failed:unavailable batch=1794400109672163 size=1 position=1 …
reads_ms=173638 … landing_ms=173638 … attempts=1
quiet_steps="git config=374.919;git rev-parse=1586.299;git update-ref=969.802"
steps="git config@114+507;git update-ref@321+127104;git rev-parse@622+38;git rev-parse@127437+46126"
```

and the wrapper's notices for the step it was waiting for,
wrapped here as the terminal wrapped them:

```text
monochromatic-git-policy-cli: still waiting for git -c core.logAllRefUpdates=false update-ref --no-deref
refs/worktree/cli-git-service/1794400109672163 f955c9b0…, a quick query step (process 203667) after 125 seconds;
it is a step that may not be run twice, so it is waited for as long as it lives (CLI_GIT_CHILD_WAIT_SECONDS changes this).
```

The answer and the notice each say something the wrapper does not do,
and the sections below say why.

## What it was waiting for

`objects_of_batch` keeps the commit a batch lands on
`refs/worktree/cli-git-service/<batch>`
before the branch write moves a reference to it,
so a commit real Git built,
which no reference names yet,
is not what `git gc --prune=now` removes while the batch is held
(`package/git-policy/cli/src/native/landing/service_land_objects_of_batch.rs`,
`package/git-policy/cli/src/native/landing/held_commit_settle.rs`).
For a commit that real Git built,
`--verbose` in that control,
this write is the step that kept the commit.

The write is not in the allow-list of steps the wrapper may end,
and it is one of the wrapper's own side references,
so `step_bound` gives it `StepBound::LeaveRunning`:
it is never ended,
and the wait for it ends at
`PLUMBING_STEP_CEILING` (120 s) plus `SHORT_EXTRA_WAIT` (7 s)
(`package/git-policy/cli/src/native/transaction/transaction_git_step_bounds.rs`,
`package/git-policy/cli/src/native/service/wait_ceilings.rs`).
127.104 s in `steps` is exactly that sum.

## What ended the wait, and what the service then read

The wait ended at its bound while the write was still working:
the longest quiet gap of that step was 969.802 ms,
so the child was showing signs of work all along,
under the load several agents' builds put on that container.

That ending is `NotKnown`,
whose own sentence says the result is not known and the step was left running
(`package/git-policy/cli/src/native/push/child_watch_step_bounds_and_errors.rs`).
`run_words` turned it into `FailureCode::Unavailable`,
because it knows two endings of a step,
the one at a ceiling and the one a stop makes,
and every other error is unavailability
(`package/git-policy/cli/src/native/landing/service_land_git_steps_and_import_stream.rs`).
The landing then answered the person
"The commit service could not finish this commit,
so it was not made",
with exit status 1,
for a commit that real Git had built and whose objects were in the store.

The 46.1 s `git rev-parse` after it is the failed landing's own cleanup:
`end_scratch_ref_record` reaches `clear_after_a_failed_landing`,
which reads the side reference before it keeps the commit under another name
(`package/git-policy/cli/src/native/landing/service_scratch_ref_of_a_dropped_batch.rs`).
So 177 s is that read plus the 127 s write,
and the answer that was sent was wrong about the commit,
which is the owner's rule "Commits should never fail".

## What changed

One function writes every one of the wrapper's side references
(`write_until_the_reference_names` in
`service_land_git_steps_and_import_stream.rs`):

- Git's own answer is read as it is;
  a write Git refused is not written a second time.
- An ending without an answer is followed by reading the reference.
  A reference that names the commit is the answer,
  whether the write that was left running wrote it or the retry did.
- A reference that does not name it is written once more,
  which is safe because the write names one value for one name,
  and the reference is read again.

The notice of a step the wrapper never ends now says what happens instead of
"waited for as long as it lives":
the command stops waiting after its bound and leaves the step running,
so whether the step finished is not known then.
The bound travels from the step's `StepBound` through the watch
(`package/git-policy/cli/src/native/push/child_watch.rs`,
`stops_waiting_after`) to `child_wait_line`
(`package/git-policy/cli/src/native/push/wait_progress.rs`).

## Controls

`package/git-policy/cli/src/native/landing/service_land_side_reference_write_tests.rs`
drives real Git with a program that lets the first side-reference write outlast
its wait,
under the shortened ceilings the unit controls set:

- `a_side_ref_write_whose_wait_ran_out_is_answered_by_the_reference` failed before
  the fix with `Some("Git could not be run (Unavailable)")`
  and passes with no words and the reference naming the commit.
- `a_held_commit_git_built_is_kept_when_its_side_ref_write_outlasts_the_wait_for_it`
  failed before the fix with `Unavailable` for a landing that must answer `Held`,
  and passes with the held answer,
  the side reference naming the commit,
  and `git gc --prune=now` keeping it.

`a_notice_of_a_step_left_running_names_the_bound_the_wait_ends_at`
(`package/git-policy/cli/src/native/push/wait_progress_tests.rs`)
passes with the wording above,
and fails with the old sentence when that branch of `child_wait_line` is removed,
which is the line the gate log carries.

## What this does not settle

- A write the wait left running is not ended,
  so it can finish after the branch write removed the side reference of a landing
  that no longer needs it.
  The reference it recreates names the commit its branch already has,
  and the record a restart would sweep it by goes when the landing ends,
  so it costs space once that commit leaves the branch,
  never the person's data.
- After the second write and the second read a repository that will not write
  its references still answers `Unavailable`,
  and the person's index and worktree are as they were,
  so the commit is then not made anywhere they can reach.
- The count of two writes is unmeasured:
  it is the smallest count that lets a wait which ended be finished by a read and
  one write after it.
