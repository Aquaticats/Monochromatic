# Git's `worktree remove` deletes a worktree with an unfinished bisect, and its `BISECT_LOG`

## Symptom

A linked worktree has an unfinished bisect and is otherwise untouched.
Two commands show that:

```console
$ git -C <feature> bisect start
$ git -C <feature> symbolic-ref --quiet HEAD
refs/heads/feature
```

`BISECT_LOG` exists in the worktree's own Git directory,
`HEAD` still names the branch,
and no file of the checkout changed.
Removing that worktree then deletes it,
its registration and the log,
with exit `0` and nothing on standard error or standard output:

```console
$ git -C <main> worktree remove <feature>
exit=0
$ test -e <feature> ; echo $?
1
$ test -e <common>/worktrees/feature/BISECT_LOG ; echo $?
1
```

`--force` is not needed and changes nothing:
the unforced form above is the one that deleted the log.

## What the measure shows, and the boundary of the claim

Measured 2026-11-11 with Git `2.56.0` on a fixture repository under
`/var/mnt/encrypted/temp` whose worktree was not a btrfs subvolume.
Only this was measured:

- `git worktree remove` without `--force` removed a worktree whose only unusual state was `BISECT_LOG`;
- it printed no diagnostic and exited `0`,
  so nothing told the caller that an operation was unfinished;
- `git bisect start` left `HEAD` on `refs/heads/feature`,
  so the worktree was not detached.

No Git source was read on this host,
so this record does not claim which Git function decides it.
The measured boundary is what matters to a caller:
the removal judges a worktree it can see as clean,
and an unfinished bisect keeps every file and every reference as it was,
so the clean-looking state and the unfinished operation are the same state.

The wrapper's own clean check does treat a bisect as an unfinished operation
(`package/git-policy/cli/src/native/worktree/worktree_clean.rs`,
the `IN_PROGRESS` list),
but that check only decides whether the wrapper destroys the subvolume itself:
a worktree it cannot prove clean is handed to Git,
and Git removes it as measured above.
So neither removal route refused,
and the log went with the worktree.

## What the wrapper does

The landed sweep,
the one removal nobody asked for,
keeps a worktree whose own Git directory holds any mark of an unfinished operation,
and `BISECT_LOG` is in that set
(`package/git-policy/cli/src/native/worktree/worktree_landed_removal.rs`,
`FURTHER_UNFINISHED_MARKERS`).
The line the person reads says which mark stands there:

```text
monochromatic-git-policy-cli: left the worktree <feature> in place, though its branch refs/heads/feature is now on main:
a bisect is unfinished there, and its BISECT_LOG is still in the worktree.
Remove it with `git worktree remove <feature>` when nothing there is yours.
```

A person's own `git worktree remove` is still Git's own command and still deletes the log:
the wrapper forwards it,
and Git's refusal covers files,
not operation marks.
Finish or abandon the bisect first,
which consumes that log:

```console
git -C <feature> bisect reset
```

## What does not work

### Relying on `git worktree remove` to refuse an unfinished bisect

Measured above:
the unforced removal exits `0` and takes the log,
so a caller that only forwards the removal keeps no record of where the bisect stopped.
Git's own clean-worktree refusal answers for modified,
staged and untracked files,
which a bisect leaves alone.

### Relying on a detached `HEAD` to show the bisect

`git bisect start` alone leaves `HEAD` on the branch.
Only a `git bisect good|bad` moves it,
so a check that looks for a detached `HEAD` sees an ordinary checkout.

### Reading `BISECT_LOG` as the person's own note

It is the file Git writes and reads,
so it is also what `git bisect reset` consumes.
Copying it aside keeps the record,
but removing the worktree with it in place is the loss this record is about.
