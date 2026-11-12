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

The log is not the first file a bisect leaves.
`git bisect start` writes `BISECT_START`,
then `BISECT_NAMES`,
and only then opens `BISECT_LOG`
(measured 2026-11-11 with Git `2.56.0` on a fixture under `strace`:
the process was killed at its first `openat` of the log,
and the worktree's own Git directory then held `BISECT_START` and `BISECT_NAMES` and no log,
with `HEAD` on `refs/heads/feature` and an empty `git status --porcelain`).
A bisect interrupted that way is the same state to every check that reads a worktree:
clean,
on its branch,
and holding no log.

## What the measure shows, and the boundary of the claim

Measured 2026-11-11 with Git `2.56.0` on a fixture repository under
`/var/mnt/encrypted/temp` whose worktree was not a btrfs subvolume.
Only this was measured about `git worktree remove`:

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
the `IN_PROGRESS` list,
which names `BISECT_START` and `BISECT_LOG`),
but that check only decides whether the wrapper destroys the subvolume itself:
a worktree it cannot prove clean is handed to Git,
and Git removes it as measured above.
So neither removal route refused,
and the log went with the worktree.

## What the wrapper does

The landed sweep,
the one removal nobody asked for,
keeps a worktree whose own Git directory holds any file whose name starts with `BISECT_`,
beside the marks of the other unfinished operations
(`package/git-policy/cli/src/native/worktree/worktree_landed_removal.rs`,
`BISECT_MARK_PREFIX`).
Every file a bisect keeps in that directory is named under that prefix,
and `git bisect reset` removes every one of them
(a fixture had none left after the reset),
so a kill between any two writes of a bisect keeps the worktree it happened in.
The line the person reads names the file that stands there:

```text
monochromatic-git-policy-cli: left the worktree <feature> in place, though its branch refs/heads/feature is now on main:
a bisect is unfinished there, and its BISECT_NAMES is still in the worktree.
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

### Reading `BISECT_LOG` alone as the mark of an unfinished bisect

The log is the last file written,
so a bisect killed before that open leaves `BISECT_START` and `BISECT_NAMES` and no log.
A guard that asks only for the log sees a clean checkout on its branch and takes the worktree,
and the person loses which branch the bisect came from and which commits it covered.
The wrapper's landed sweep did exactly that until 2026-11-11:
a fixture with those two files and no log lost the worktree and both files to a wrapped `git merge`
whose branch landed.

### Reading `BISECT_LOG` as the person's own note

It is the file Git writes and reads,
so it is also what `git bisect reset` consumes.
Copying it aside keeps the record,
but removing the worktree with it in place is the loss this record is about.
