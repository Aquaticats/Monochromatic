# cli-git object reader: a named replacement base is a recorded limit

## Status

Answered in the question tool on 2026-11-23,
recorded 2026-11-23T17:02:00-05:00.
Nothing of it is built.
It settles finding 1 of the object reader branch's fourth integration recheck,
on `fix/git-policy-cli-object-reader-declined-branches-controls-and-lows` at `daa0836b75`.

## What was asked

The fourth Sonnet recheck found the branch fit to merge after one fix,
and the fix was the project's to choose.
The case:
someone sets `GIT_REPLACE_REF_BASE` to a ref base (for example `refs/probe/`)
that holds a replacement of the commit being checked out,
and runs `git worktree add`.
Plain Git checks out the replacement's files.
The wrapper,
on `main` and on this branch alike,
snapshots the original commit's files,
so Git then reports the new worktree dirty.
A later `git worktree remove` is refused by plain Git but goes through on the wrapper.
A test showed no uncommitted work lost.
The branch's note wrongly says a named base "cannot change the answer".

The question ranked the options A over B over C:
A before B because A follows how the project settled the two earlier cases of this kind
(the commit refusal and issue 6347:
record the limit,
do not judge whether the setting is in effect)
and costs a paragraph;
B before C because B keeps the harmless setting (a named base with nothing under it) on `main`'s fast route,
which C breaks.

## What was chosen

The project chose "A:
record the limit (Recommended)":
"Correct the false sentence,
document this case as a known limit next to issue 6347,
and file it as an issue.
Pros:
no code change,
merge bar stays met,
same as your two earlier calls.
Cons:
in this rare setup the worktree's files still don't match what Git expects,
and removal goes through where plain Git would refuse."

Not chosen:
"B:
decline when base holds a ref",
in which the wrapper hands the add to Git whenever the named base actually holds a ref;
and "C:
decline for any named base",
which hands the add to Git whenever the variable is set at all.

## What follows

The coordinator's reading,
not the project's words.
The branch's note `doc/object-database-and-a-reftable-ref-store.md` corrects the sentence
(the reasoning that names are their bytes holds for the two store variables only),
the note and the package README record the case as a limit beside issue 6347's,
with the case the recheck measured,
and a new issue holds it.
The add and the removal keep `main`'s behaviour.
The recheck's two lows ride with the same fix:
a control for the ladder target's open,
and the note's leftover words.
The recheck's review is
`~/temp/agent/integration-recheck-object-reader-the-commit-refusal-is-out-2026-11-22/keep/review-fourth-recheck.md`.

## Moved from the old handover

From lines 273 to 275 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The object reader branch's commit refusal comes out
  ("B:
  take it out (Recommended)").
