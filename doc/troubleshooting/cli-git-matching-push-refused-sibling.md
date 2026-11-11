# A `push.default=matching` push was reported as failed, and retried every ten seconds, for a branch it had published

## Symptom

A commit under `push.default=matching` lands,
and its automatic push carries every local branch the remote has.
One sibling branch cannot fast-forward,
so Git refuses that ref and exits non-zero while the recorded branch is delivered:

```text
To /path/to/remote.git
   c3b4788..a325533  main -> main
 ! [rejected]        topic -> topic (fetch first)
error: failed to push some refs to '/path/to/remote.git'
```

The next command in that worktree printed:

```text
The backup push of branch main for commit a325533 did not finish, so the remote may or may not have taken that commit. The commit is saved in this repository and nothing was lost, and the commit service keeps the push in its retry plan, so it tries again on its own.
monochromatic-git-policy-cli and Git said:
    monochromatic-git-policy-cli: the commit service tries this push again by itself in 10 seconds (try 2), and keeps trying with longer pauses until the push lands or a conflict needs you.
```

and the service journal said:

```text
pushed branch="refs/heads/main" oid=a325533 items=1 outcome=retrying push_ms=271
push will be tried again branch="refs/heads/main" oid=a325533 next_try=2 in_seconds=10
```

Three things are wrong in that report.
The remote had taken the commit (`main -> main` above).
The retry could not clear,
because the sibling stayed refused and every try failed the same way,
so the plan went on at ten seconds,
twenty,
forty,
up to its longest pause.
And nothing told the person which branch Git had refused.

## Cause

`push.default=matching` runs the plain `git push <remote>` with no refspec,
so one push carries every matching branch and Git's exit status is the worst fate among them
(`package/git-policy/cli/src/native/push/auto_push.rs`,
`push_arguments` and the `Sent` type).
`push_once` read that status as the recorded branch's own failure,
so `reported` answered `Failed`,
`GitHandlers::push_awaiting`
(`package/git-policy/cli/src/native/landing/service_work.rs`)
planned a retry,
and `push_retry::FIRST_PAUSE`
(`package/git-policy/cli/src/native/push/push_retry.rs`,
ten seconds)
was the pause the person saw.

## Fix

The recorded branch's fate is asked of the remote,
never read from the aggregated status:
`package/git-policy/cli/src/native/push/matching_push_delivered_the_recorded_branch.rs`
runs one `git ls-remote --refs <remote> refs/heads/<name>` after a matching push that failed,
and a push whose answer is the commit it sent is reported as published,
so no retry is planned.
The service journal now names each refused ref from Git's own line for it.

A remote that cannot be asked,
or that names another object,
proves nothing and leaves the push the failure it was.

## Why the remote-tracking ref is not the proof

It was tried first and it is wrong in both directions.
`refs/remotes/<remote>/<name>` is a local copy that lags the remote,
so it still names a commit whose branch the remote deleted
(`a_matching_push_with_no_branch_in_common_fails_with_gits_words`),
and a push to a remote that cannot be reached at all reads as delivered
(`a_matching_push_to_an_unreachable_remote_fails_loudly`).
Both controls caught it.

## The ten seconds was not a fixed grace

The pause above is the retry plan's first pause,
reached only because the push was judged failed.
A successful matching push has no ten-second step in it:

- The only ten-second values on the push path are
  `push_retry::FIRST_PAUSE` (outside the push's own duration),
  `push::child_class::OBJECT_NOTICE_GRACE` and
  `push::wait_progress::DEFAULT_NOTICE_INTERVAL`,
  which time a notice and nothing else.
  `push_deadline_and_patience_settings::DEFAULT_PUSH_DEADLINE` is thirty seconds.
- Fifteen interleaved rounds of a disposable local remote,
  comparing `push.default=matching`,
  `current` and an upstream push with one binary,
  gave `push_ms` of 233 to 1642 ms under `matching`,
  every one of them `outcome=done` on the first attempt,
  and publish latencies of 184 to 1607 ms;
  the host's own Git pushing the same kind of commit at the same moment took 120 to 4103 ms.
- Twenty paired typed pushes gave a wrapper median of 451 ms against the host's Git at 326 ms.
- The service journal of this host over seven days holds 8969 `pushed branch=` lines.
  Pushes of nine to eleven seconds are 368 of them and of eleven seconds or more 370,
  and the values between 9900 and 10199 ms are 73,
  in line with the 106 between 9500 and 9899 and the 52 between 10200 and 10499.
  A fixed ten-second wait would put a spike there;
  the tail is the host's load.

Measured 2026-11-11 on a host running a load average far above its eight threads,
so the spread of every arm is reported rather than one figure per arm.

## Limits

The question goes to the remote's fetch URL,
so a `remote.<name>.pushurl` that differs can leave the answer unproven,
which errs toward the retry and never toward a claim.
A remote that answers with an error,
or that names another object,
also proves nothing.
