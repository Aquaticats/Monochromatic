# cli-git worktree removal: the "recorded and not woken" warning names the way out

## Status

Answered in the question tool on 2026-11-23,
recorded 2026-11-23T04:24:51-05:00.
Nothing of it is built.
It settles issue 6354,
and its build follows the merge of issue 6345's branch,
whose round edits the same documents.

## What was asked

After `git merge` lands a branch,
the wrapper removes that branch's worktree in the background.
If waking the background service fails,
the merge prints a warning ("recorded and not woken") and the worktree stays.
The second integration pass of issue 6345's branch ran thirteen ordinary commands against such a record
(`git status`,
`git log`,
`git add`,
`git worktree list` and others):
only `git commit` woke the service and finished the removal.
Neither the warning nor the merge's own line says so,
and the documentation says "every commit and every add".
Nothing is lost meanwhile,
and `git worktree remove <path>` works at once.

## What was chosen

The project chose "A:
say it in the warning (Recommended)":
"Keep the behaviour;
the warning says that the next `git commit` finishes the removal and that `git worktree remove <path>` does it now;
correct "every add" in the docs.
Pros:
one clause,
no behaviour change.
Cons:
someone who misses the warning gets no second signal."

The warning as it was shown with that choice:

```text
[warn] the removal of .claude/worktrees/x is recorded and
not woken (the service could not be reached). Your next
`git commit` here finishes it; `git worktree remove
.claude/worktrees/x` removes it now.
```

Not chosen:
"B:
document it only",
which left the warning as it is;
and "C:
read commands also wake it",
which had commands a person reads wake the service when they find an old pending removal.

## What follows

The coordinator's reading,
not the project's words.
The change is the warning's clause,
the correction of "every commit and every add" in the wake function's doc and in the troubleshooting document,
and a control that the warning carries both commands.
What wakes the service does not change.
