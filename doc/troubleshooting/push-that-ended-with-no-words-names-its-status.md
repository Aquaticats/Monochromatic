# A push that ended without writing a word left the retry line's cause empty

One gate of `monochromatic-git-policy-cli` 0.6.1-alpha.44 on a Linux host wrote a retry line with nothing after
`cause:`,
and the two-worktree push control whose push it was failed with the origin still naming the earlier commit.

## Symptom

The service journal of the gate run of `2026-11-21T15:09:44-05:00` held these two lines together,
for the second of the two branches the control pushes:

```text
pushed branch="refs/heads/other" oid=9fe62eac61482f54ebbb5b8e93bec4294463b49e items=1 outcome=retrying push_ms=173
push will be tried again branch="refs/heads/other" oid=9fe62eac... next_try=2 in_seconds=10 cause:
```

and the control
`lfs_hang::two_worktrees_committing_at_once_with_a_slow_upload_both_finish_without_waiting_for_each_other`
panicked at `package/git-policy/cli/src/native/binary/binary_lfs_hang_tests.rs:102`:

```text
assertion `left == right` failed: other: the new commit, not the earlier one, reached the origin
  left: "5ca9353a41684c9c58ccb8f0064df68f408af281"
 right: "9fe62eac61482f54ebbb5b8e93bec4294463b49e"
```

The command that made the commit exited `0`,
and the commit is in the worktree:
the origin simply never received it,
because the push ended `retrying`,
and a retried push is settled `failed` until a later try lands it
(`service_work_settling_a_verdict.rs`,
the `Verdict::Retry` arm).

The same line carries a cause whenever the push printed one,
for example `cause: the network is down`,
so the empty field is a report about the push and not about the journal.

## Root cause

The cause of a retry line is the first line of the push's own output that gives a reason,
read by `cause_line`
(`package/git-policy-cli/src/native/push/push_retry_cause_line.rs`):

```rust
for line in text.lines() {
    let trimmed: &str = line.trim();
    if trimmed.is_empty() {
        continue;
    }
    if trimmed.starts_with(REMOTE_HEADER) {
        remote = Some(trimmed);
        continue;
    }
    return Some(trimmed);
}
return remote;
```

The line was written with `unwrap_or_default()`,
so a text with no line at all printed nothing after `cause:`
(`planned_retry` in `service_work_handlers_replanning_and_acceptance.rs`;
the same pattern stood at two more sites in `service_work_drains_awaiting.rs`).

The text is the verdict the push's handler answered,
and every arm of `GitHandlers::push_awaiting`
(`package/git-policy-cli/src/native/landing/service_work.rs`) puts words in it except one:

- `Verdict::Skipped` names whose list it searched,
or that the branch is gone.
- `Verdict::Delivered` carries the server's own lines.
- `Verdict::Failed` names the refusal,
its plain words and Git's output.
- `Verdict::Retry` from `AutoPushOutcome::HeldBack` is a line built by `held_back_line`,
  `undecided_line` or `unanswered_line`,
  each of which names the commit that holds the branch back.
- `Verdict::Retry` from a failed attempt is `retried_failure_record(result.shown)`,
  where `shown` is the attempt's own output.

`shown` is built in `reported`
(`package/git-policy-cli/src/native/push/auto_push.rs`):

```rust
let failed: bool = attempt.exit_code != 0 && !delivered;
let said: String = filter_push_output(&attempt.output, attempt.exit_code);
let mut shown: String = match (held_back.clone(), said.is_empty()) {
    (None, _) => said,
    ...
};
```

So a push that exited nonzero and wrote nothing reached `final_failure("")` and `unrecognised_rejection("")`,
which both answer nothing,
and the verdict was `Verdict::Retry(String::new())`.
The attempt's status and the signal beside it were dropped on the way:
`PushAttempt` keeps them,
and nothing read them for a push that printed no line.

The one other arm that can answer an empty `shown` is `held_back_result` with `WhereThePushStops::Nothing`,
and it is unreachable:
`source_of` answers `Some(sent_up_to)` for that stop,
so `push_once` never calls it (`auto_push.rs`,
`source_of`).

Which ending the attempt had is not recoverable from that journal.
An attempt that exits nonzero without writing a line is one a signal ended before it wrote,
or one whose program exited silently,
and the report threw away the field that would tell them apart.
`push_ms=173` says only that the attempt was short.

### What this branch did not do

The round's own paths do not run in that control and touch no attempt:

- `ClientMessage::ForgetPrograms` removes entries from the service's memory list
  (`service_push_programs_held_in_memory.rs`,
`forget`).
- The forgets in `PushRetries::plan` and `PushRetries::forget`,
  in `SkipNotice::settled`,
  and at every ending of a due retry,
  remove the same entries.
- The drain's endings answer verdicts,
and none of them builds one.

`package/git-policy-cli/src/native/push/auto_push.rs` and
`package/git-policy-cli/src/native/push/push_retry_cause_line.rs` hold no line this branch touched,
and `main` carries the same two files as blobs `6d12fbc06d` and `846e219059` at `1a53a03178`,
the `main` this branch merged.
So `main` has the empty cause,
and the silent attempt it comes from.

### The two incidents this control already had

Neither is this one:

- `pdeathsig-fires-when-the-parent-thread-ends.md` records the control failing after 60 seconds with
  "the service never settled the after-commit work":
nothing settled,
and no push had failed.
- `service-journal-listing-removed-another-instances-temporary-file.md` records
  "The commit service could not finish this commit,
so it was not made":
the landing failed,
not the push.

This failure has a landed commit,
a push that ran 173 ms,
and a retry with no words.

### Found again, with the cause named

The same control failed again in the gate of this branch at `2026-11-21T15:58:25-05:00`,
whose tree carries the report below,
and the line that had said nothing now said what ended the push
(the `pushed branch=` entry carried it after `outcome=retrying push_ms=150`):

```text
monochromatic-git-policy-cli: the push was ended by signal 9 (status 137) and printed nothing about why,
so what the remote did is not known; the commit is saved in this repository.
```

The attempt was a `git push` a signal killed with nothing written,
which is the second ending this document's root cause named.
What killed it is not in the record,
and the candidates the sources rule out are these:

- The wrapper's own stop of a push appends its reason,
  so a stop is never silent (`bounded_push.rs`,
`stop_cause_and_words.rs`).
- The fixture's teardown kills a service and the processes of its group with `SIGKILL`
  (`kill_with_everything_it_started`),
  and the service logged the retry 60 ms before it was told to stop,
  so the push was not in a group that teardown ended then.
- A failed upload is explained with words before it becomes an attempt
  (`lfs_upload_explanations.rs`,
`explained`),
  so the child that died was the push and not `git-lfs`.

What is left is a `SIGKILL` from outside those paths,
and a container at its memory bound is the shape that fits
(issue 6335 is filed for the killer,
naming the seam below):
this failure happened in two full gates and never in twelve runs of the same control alone
(`keep/closure2-lfs-hang-alone-in-the-gate-image.log` in the round's folder).

## Verification

The condition is forced by a seam in a scratch copy of the package,
never committed:
`run_bounded_push_awaiting_naming` ends a run whose arguments carry the value of `CLI_GIT_RETRY_PROBE_KILL_PUSH`
with `SIGKILL` right after its start,
and cuts its output file to nothing before it is read,
and the control's own fixture gives the service that variable for the `other` branch only.
Every run of that arm then meets a push attempt that ends nonzero having written nothing,
which is the input the defect is made of.

The run is one nextest filter in the gate image,
once per arm,
with the seam's two files mounted read-only over the image's copies,
and,
for the arm that speaks for `main`,
the four files that build the report and the line as `1a53a03178` has them
(`auto_push.rs`,
`push_retry_cause_line.rs`,
`service_work_handlers_replanning_and_acceptance.rs` and
`service_work_drains_awaiting.rs`).

- This branch,
`refs/heads/other` ended by the seam:
`outcome=retrying push_ms=40: monochromatic-git-policy-cli: the push was ended by signal 9 (status 137) ...`,
the retry line's `cause:` names it,
and the control fails with
`assertion left == right failed: other: the new commit, not the earlier one, reached the origin`
(`1 test run: 0 passed, 1 failed`).
- `main`'s report,
the same seam:
`outcome=retrying push_ms=40`,
`push will be tried again ... next_try=2 in_seconds=10 cause:` with nothing after it,
and the same assertion failure
(`1 test run: 0 passed, 1 failed`).

So `main` has the empty cause,
and the branch's fix is what turns the same failure into a line that names it.
The controls of both halves were run the same way in the same image:

- With the image's own files:
`2 tests run: 2 passed`.
- With `main`'s four files mounted over the image's copies:
`2 tests run: 0 passed, 2 failed`,
one on the record's words and one on the journal line's cause.

The image is `localhost/git-policy-native-test:gpc-retry-20261121t1620`,
built by the gate of this branch,
whose suite was `9527 tests run: 9525 passed (6 slow), 2 failed, 35 skipped`,
the two failures being the cause-H pair this host's note records.
Its log is `keep/gate-2.log` in the round's folder,
and each arm's log is beside it.

### The container's memory bound, measured

While those arms were being run,
a `rustc` compiling the crate in a container bound to 2 GiB was ended with `SIGKILL` and nothing written
(`keep/red-main-code.log`);
the same command with 6 GiB compiled.
That is the same ending the push met,
so a container at its bound is the plausible killer of a process nothing else in the tree signals.

## What does not work

- Reading the gate's journal for the attempt's status:
  `reported` kept no status for an attempt without output,
  so the field is gone before any line names it.
- Reproducing by repetition:
  the control passed in twelve runs of the gate image and in most gates,
  and the failure needs a push that dies without a word.
- Supposing a held-back push:
  `held_back_line`,
`undecided_line` and `unanswered_line` always name a commit,
  and `moved_past_the_pin` names the branch.

## Upstream filing decision

Nothing to file.
The empty field is this repository's own report,
the attempt's status and signal already reach `PushAttempt`,
and Git printing nothing when it is killed is not a behaviour upstream can change.

## Limits

The fix names what is known,
not what happened:
a push that ends silent is reported with its status and with its signal,
and the next occurrence can be told from a killed push and a refused one.
Which signal ended this one is not recoverable,
because no record of that run kept it.
