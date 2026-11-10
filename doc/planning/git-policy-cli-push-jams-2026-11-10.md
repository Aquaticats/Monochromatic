# The pushes of `main` that reached the remote late on 2026-11-10, and the one cause the record could not name

Date measured:
 `2026-11-10T18:00:00-05:00`.
Branch of this work:
 `fix/git-policy-cli-pushes-of-main-that-stalled-today`.
The owner's words this serves:
 "Prioritize fixing the jams and release jam fixes asap."
 and
 "Agents should never wait on each other on commits or merges."

## What was measured,
 and how

Every push of `refs/heads/main` from the main checkout
 that did not reach the remote within a minute of the commit's landing.

Three sources,
 all read only:

- `journalctl --user` of every unit named `monochromatic-git-policy-cli@<instance>.service`.
  The instance of the main checkout is
  `f30a114aaabc977cf77309db0f77f1ee`,
  the hash of `/var/mnt/encrypted/repository/Monochromatic/.git`
  (`package/git-policy/cli/src/native/service/service_instance.rs`).
  Its five process ids of the day
  (`207708`,
  `2376273`,
  `2916888`,
  `3462772`,
  `3993568`)
  were matched to the instance by the `started ... instance=` line each prints.
  The instance writes to the journal without a unit field,
  so the process id is the only handle on it,
  and a filter on the branch name alone mixes in every throwaway repository
  an agent's test run made that day.
- The push items under
  `<git-dir>/monochromatic-git-policy-cli/work/`
  of the main checkout.
  Each item records the moment the commit landed (`created`),
  so a landing is a stored fact and not the moment the service answered the commit,
  which is later and would hide the wait.
- `git ls-remote --heads origin refs/heads/main`,
  run at the end of the measurement.
  It printed `77bc0860fc`,
  which confirms that the commits the journal said had landed were on the remote.

The wait of one commit is the end of the first push of that branch that started after
 its push item was created and reported `outcome=done`
 (or `the push of "refs/heads/main" landed on try N`).
A push that reported `outcome=retrying` or `outcome=failed` did not reach the remote,
 so it does not count as the end of a wait.

Of the day:

- 139 commits landed on `refs/heads/main`.
- 126 pushes of `refs/heads/main` were logged for that checkout.
- 24 of the landings reached the remote more than a minute after landing.
- one more was still waiting when the measurement stopped.

## The pushes that were late

Each line is the landing's own time,
 the commit,
 the wait,
 and the moment it reached the remote.

### Waits the journal explains

- `2026-11-10T02:24:50-05:00`
  `1535eab9bf`,
  169 s,
  on the remote by `02:27:39`.
  The service refused the push at `02:24:59`:

  ```text
  02:24:59 pushed branch="refs/heads/main" oid=1535eab9bf... items=1 outcome=failed push_ms=9531:
  monochromatic-git-policy-cli: the server refused this push: a hook on the server declined it ...
  remote: error: GH006: Protected branch update failed for refs/heads/main.
  remote: error:
  ```

  A plain `git push origin main` of the same commit then landed it
  (`doc/troubleshooting/github-protected-branch-refusal-that-cleared.md`).
- `2026-11-10T10:36:47-05:00`
  `62994776ec`,
  1 224 s,
  on the remote by `10:57:12`,
  with the next commit.
  The service refused the push at `10:37:02` with the same code:

  ```text
  10:37:02 pushed branch="refs/heads/main" oid=62994776ec... items=1 outcome=failed push_ms=14215:
  ... The commit service stops pushing this branch until your next commit. A push by hand may succeed ...
  remote: error: GH006: Protected branch update failed for refs/heads/main.
  remote: error:
  ! [remote rejected]       main -> main (protected branch hook declined)
  ```

- `2026-11-10T16:10:22-05:00`
  `b5d714959c`,
  485 s,
  on the remote by `16:20:56`,
  with the next commit.
  `2026-11-10T16:14:43-05:00`
  `e5e689c8a5`,
  224 s,
  on the remote by `16:20:56` too.
  `2026-11-10T16:17:45-05:00`
  `de3fdeb838` waited 42 s and landed on try 2,
  which is under the minute.
  It is named here because it is the last of the same run.
  The cause was the push URL of the shared remote settings:

  ```text
  16:10:40 pushed branch="refs/heads/main" oid=b5d714959c... items=1 outcome=retrying push_ms=17518:
  fatal: 'DISABLED' does not appear to be a git repository
  fatal: Could not read from remote repository.
  16:10:40 push will be tried again branch="refs/heads/main" oid=b5d714959c... next_try=2 in_seconds=10
  cause: fatal: 'DISABLED' does not appear to be a git repository
  ```

### Waits the journal does not explain

Twenty landings waited with no line about a push in between,
 only the commit answers of the landings that kept arriving.
They fall into five intervals.
In each of them the service also made no maintenance and no review.

- `2026-11-10T11:34:24-05:00`
  `a2dd6a2470`:
  88 s,
  on the remote by `11:35:52`.
  `2026-11-10T12:01:29-05:00`
  `b22dff99f6`:
  317 s,
  by `12:06:46`.
  `2026-11-10T12:16:10-05:00`
  `cd37452fac`:
  229 s,
  by `12:19:59`.
  The round that landed `cd37452fac` finished its review of `b22dff99f6`
  at `12:19:53`,
  so the push waited for the earlier landing's round to end.
- `2026-11-10T12:50:25-05:00`
  `7801a67ad0`:
  4 946 s,
  by `14:12:50`,
  in one push that carried seven records (`items=7`).
  Seven more landings waited in the same interval:
  `13:01:32` `1dc9c27801` (4 279 s),
  `13:01:36` `22443fa4ea` (4 274 s),
  `13:02:19` `6fa8299806` (4 231 s),
  `13:05:55` `5e3a6025ba` (4 015 s),
  `13:06:15` `a8cf445cd5` (3 995 s),
  `13:11:20` `6ad29d302d` (3 691 s),
  `13:59:07` `6c039edb19` (823 s).
  No `pushed`,
  `automatic maintenance ran`
  or
  `the review of ... is done` line appears between `12:35:56` and `14:12:50`.
  `doc/troubleshooting/git-policy-cli-service-answers-a-commit-late-while-a-round-is-busy.md`
  records the same interval and the same reading:
  one round holds the pushes.
- `2026-11-10T14:25:14-05:00`
  `b4ed7deb73`:
  3 748 s,
  by `15:27:42` (`items=3`).
  `14:53:48` `807f3ea26d`:
  2 033 s.
  `15:13:53` `483bbc6587`:
  829 s.
- `2026-11-10T15:33:10-05:00`
  `cd04c67508`:
  2 106 s,
  by `16:08:16` (`items=4`).
  `15:43:45` `fdf651a9be`:
  1 471 s.
  `16:01:22` `0150fc0f29`:
  414 s.
  `16:06:46` `953b68b0fc`:
  90 s.
- `2026-11-10T17:29:14-05:00`
  `cefec4667f`:
  932 s,
  by `17:44:46`.
  `17:42:03` `77bc0860fc`:
  163 s,
  by the same push,
  which carried two records (`items=2`).
  The push item of `cefec4667f` was created at `17:29:14.247`
  and moved to `done` at `17:44:42.343`,
  so it existed the whole time and was pushed only once the next commit had landed.
  One command ran in the main checkout between the two.
  It was a `merge` at `17:37:49` that declined with `tip-unchanged`,
  and it did not release the record.
  This is the behaviour the branch `fix/git-policy-cli-held-push-released-by-the-retry` names:
  a held push waits for the command that landed it,
  and a record whose command died is released by the next command of the worktree.
  The journal cannot prove the record was held rather than merely pending,
  because a held record leaves no line of its own in the release this checkout runs
  (`0.6.1-alpha.8`).
- Still waiting when the measurement stopped:
  `2026-11-10T17:46:00-05:00`
  `630cba63d4`,
  whose push item was in `pending` at `17:47:50`
  while the remote was at `77bc0860fc`.

### One wait the day's own record could not state

Twenty retry lines of the day's real worktrees,
 including the one for `a46a2230ce` at `2026-11-10T01:42:43-05:00`,
 named the remote as the cause and nothing else:

```text
01:42:43 push will be tried again branch="refs/heads/main" oid=a46a2230ce... next_try=2 in_seconds=10
cause: To https://github.com/Aquaticats/Monochromatic
01:42:43 the server rejected the push for a reason this release does not know, so it is tried again:
! [remote rejected]       main -> main (missing necessary objects)
```

Git writes the remote it pushed to above every line it writes about the references
 (`transport.c`,
 `print_push_status`,
 Git 2.56.0).
So the journal's cause line named where the push went and not why it failed.
That is fixed here.
`cause_line` in
 `package/git-policy/cli/src/native/push/push_retry.rs`
 returns the first of Git's own lines that is not that remote line,
 and both records that name a cause use it
 (`landing/service_work_handlers_replanning_and_acceptance.rs`,
 `landing/service_work_drains_awaiting.rs`).
The control is
 `the_cause_a_retry_names_is_a_reason_and_not_the_remote`
 in
 `package/git-policy/cli/src/native/push/push_retry_tests.rs`.
It failed before the change with
 `left: Some("To https://github.com/example/repository")`.

## Which causes are fixed,
 and where

- A server's rejection that this release does not know is retried,
 and the day shows the retry working.
 `remote: fatal error in commit_refs_rules` at `00:08:03` landed on try 2 at `00:08:18`.
 `! [remote rejected] main -> main (missing necessary objects)` at `01:42:43`
  landed on try 2 at `01:42:57`.
 The words of such a retry now name the reason instead of the remote (`cause_line`).
- The push URL of the shared remote settings:
 fixed on `feat/git-policy-cli-shared-remote-settings-refused-in-worktrees`,
 which refuses the write in a linked worktree.
Not on `main`.
`main` still meets a `DISABLED` URL and retries it as if the fault were the network's.
- A held push whose command died:
 fixed on `fix/git-policy-cli-held-push-released-by-the-retry`,
 which writes the command and its verdict into the record and releases it from the next command.
- The pushes of a repository that no longer exists:
 fixed on the same branch,
 whose `unreachable_git_dir_proven_gone.rs` ends the plans of a deleted repository.
The day's evidence for it:
 a service retried two throwaway repositories under
 `${HOME}/temp/agent/lfs-login/proof-scratch-*` at try 60 and climbing,
 every 600 s,
 from `16:24` to `17:41`.
- A round that holds the pushes of its own landing:
 not fixed on `main` and not on either branch.
`doc/troubleshooting/git-policy-cli-service-answers-a-commit-late-while-a-round-is-busy.md`
 records it and names the limits it needs.
It belongs to the waiting path of a commit and not to this branch.
It is the cause of eighteen of the twenty four late pushes.
- A server hook's decline that clears on a retry:
 the behaviour is the owner's decision and is unchanged.
The evidence for asking about it is in the next section.

## A decline that clears on a retry and the options for the owner

Known:

- `2026-11-09T23:30:11-05:00`
  `main` was refused with
  `GH006: Protected branch update failed for refs/heads/main.`,
  the reason after the code empty.
  A plain `git push origin main` of the same commit then succeeded.
- `2026-11-10T02:24:59-05:00`
  the same code and the empty reason on `1535eab9bf`.
  Every one of its 14 unpushed commits was already signed
  (`git log --format=%G?` printed `G`),
  and a push by hand landed the same commit.
- `2026-11-10T10:37:02-05:00`
  `GH006` again,
  this time with the reason the wrapper reads as a hook's decline
  (`protected branch hook declined`).
  The commit reached the remote only with the next commit,
   1 224 s after it landed.
- The wrapper cannot tell the two apart from the output.
  The reason a hosting service prints after the code is its own text,
  and GitHub is closed source,
  so no call chain can name what it refused.

The rule in force,
 quoted in
 `package/git-policy/cli/src/native/push/push_retry.rs`:

> Offline is retried,
> but auth and server hook declines stop until your next commit.

Three options.
This record changes none of them.

1. Keep the rule as it is,
  and only keep the words that say a push by hand may succeed.
   Pros:
   a refusal never spins the service,
   a policy or permission refusal is not repeated into a rate limit,
   and the person is told which refusal it was.
   Cons:
   today two commits waited 169 s and 1 224 s for the next commit,
   and the one at `02:24:50` needed a person to push it.
2. Retry a decline a bounded number of times and then stop,
  for example two more tries 30 s and 120 s apart.
   Pros:
   a refusal that clears lands by itself inside about three minutes,
   which is the observed shape of both `2026-11-09T23:30:11-05:00`
   and `2026-11-10T02:24:59-05:00`,
   and the person is still told when the last try is refused.
   Cons:
   the owner's decision changes,
   a branch refused for a policy reason is refused three times per commit instead of once,
   and the words of a final failure would have to say which try refused it.
3. Keep the rule and let any command of the worktree wake the stopped push,
  not only a commit that moves the branch.
   Pros:
   no decision changes,
   and a person who reads the record and runs `git status` or `git log` clears the stop.
   Cons:
   it still needs a person,
   and it makes an ordinary read command carry a push.

Ranking:
 2 > 1 > 3,
 because 2 is the only option that lands a refusal which clears without a person,
 1 is what runs now and costs the two waits named here,
 and 3 keeps the cost of 1 while adding work to commands that change nothing.

## How to read this again

```sh
# journal-path: the service instance of the main checkout, by process id
journalctl --user --since '2026-11-10T00:00:00-05:00' --output=json \
  | jq --raw-output 'select(._PID == "207708" or ._PID == "3993568") | .MESSAGE' \
  | grep 'refs/heads/main'

# worktree-path: the moment each commit of main landed and when its push item finished
ls --time-style=full-iso --long \
  .git/monochromatic-git-policy-cli/work/done/push-*-f921bd05e68b0374
```

The suffix of a push item's name (`f921bd05e68b0374`) is the hash of the push destination,
 so a name from an earlier day may carry another suffix.
