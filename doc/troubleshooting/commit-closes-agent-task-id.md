# Commit messages that close the wrong issue

Ten commits on `translation-repair-rebased` carry a `Closes #N` trailer where `N` is an agent task-list ID,
not a `Aquaticat/Monochromatic` issue number.
The two numbering spaces overlap in the range these commits used,
so every one of those trailers names a real repository issue about unrelated work.

## What happens, and when

GitHub acts on a closing keyword when the commit carrying it reaches the repository's default branch.
These commits sit on `translation-repair-rebased`,
so nothing has closed yet:
checked 2026-08-21, eight of the ten named issues were still open,
and the two that were closed had closed for their own reasons.

On merge to `main`, GitHub would close all eight.

## The affected commits

Each line is the commit, the trailer it carries, and the repository issue that trailer actually names.

-   `655b4bb3d` `Closes #53`, against `build: assign distinct DEFAULT_PORT values across webapps`.
-   `8f02e72bf` `close #72`, against `refactor: consolidate webapp-forge/server jsx-runtime.ts with ssg-test`.
-   `a6b0c6b83` `Closes #112`, against `build: investigate agent-initiated compaction approach`.
-   `41fca1a72` `Closes #116`, against `build: track upstream dprint fix for exec plugin extension-fallback`.
-   `e0a655186` `Closes #117`, against `fix(paper2vn): document or fix [hidden] attribute specificity collision`.
-   `67228e9ab` `Closes #126`, against `perf: declare sideEffects: false on pure module packages`.
-   `446f9b510` `Closes #152`, against `build(module/es): document JSONC parser rationale`.
-   `0cf2f759b` `Closes #153`, against `fix(exa-search): use replicating element logic`.
-   `5de9d9085` `Closes #155`, against `fix(pi/auto-mode): add missing TSDoc per TODO in context.ts`.
-   `d428bf5db` `Closes #159`, against `build(automation): set up release versioning and publishing automation`.

One commit on the branch gets it right:
`b3e3b01c5` carries `Closes #426`,
and repository issue 426 is
`Artifact draws silently mix pipeline generations: every reader ignores the recorded tip`,
which is the work that commit does.

## What was done about it

The messages are not amended,
per the repository rule against amending an inaccurate commit message.
Each of the ten commits carries a corrective commit comment naming the collision,
which also lands a cross-reference in the wrongly named issue's timeline.

## What is still owed at merge time

Whoever merges `translation-repair-rebased` into `main` has to handle these trailers,
by dropping the keywords during the merge,
or by reopening the eight issues afterward.
Nothing in the repository enforces this,
so this document is the record.

## How to not repeat it

`AGENTS.md` rule `TID` states the invariant:
a `#N` in a commit message is a repository issue,
and an agent task-list ID never goes into one.
The measurement that produced this list is reproducible:
read every commit on the branch that is not on `main`,
match closing keywords against `#(\d+)`,
and check each number with `gh issue view`.
