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
with the two files that make the difference mounted read-only over the image's copies:

- This branch:
no file mounted,
so the image's own fixed report writes the line.
- `main`:
`auto_push.rs` and `push_retry_cause_line.rs` as `1a53a03178` has them.

The control's assertion fails in both arms,
because the push really did fail.
Only the words it leaves differ.

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
