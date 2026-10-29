# Imports nothing uses: a question about the lint policy, not about one package

Written 2026-08-15 while acting on a review of `@monochromatic-dev/module-translation-repair`.
It blocks nothing.
It is here because the finding that started it was not a defect in that package so much as a gap
that lets the same defect happen anywhere, quietly, and the gap is a decision somebody already
took deliberately.

## What was found

An earlier extraction moved four groups of code out of `repair-translation.ts` and left their
imports behind.
Six names remained in the file: `buildLaneSliceTexts`, `buildChunkCriticRecords`,
`ChunkCriticRecord`, `buildIssueRecords`, `RepairIssueRecord`, `repairReplacements`.
Nothing in the file mentioned any of them except the import statement itself.
Neither `mise run //package/module/translation-repair:lint` (moved on 2026-10-18 to
`//package/translation-repair/<package>:lint`) nor
`mise run //package/module/translation-repair:lint:types` (moved on 2026-10-18 to
`//package/translation-repair/<package>:lint:types`) said a word, on a run that reported
zero warnings and zero errors.

That is not a bug in either tool.
`package/config/oxlint/src/rule/restriction.ts:289` turns `eslint/no-unused-vars` off, with a
rationale directly above it: the editor reports them, unused names are often deliberate during
work in progress, and the bundler removes them anyway.
Every one of those reasons holds for a local variable.
None of them holds as well for an import, which is a statement about what a module DEPENDS on:
a reader deciding whether a file may be moved, split or deleted reads its imports first, and a
dead one says the file is coupled to something it has not touched in months.

## What it costs today, measured

Measured with a throwaway script in this session's scratch directory, which is why the method is
written out here rather than linked: the directory does not survive, and the number has to be
reproducible without it.
The script walks every `.ts` and `.mts` file under `package/`, skipping `node_modules`, `dist`,
`bundle` and `target`.
For each file it collects the names in `import { ... }` and default-import statements, then
counts word-boundary occurrences of each name in that file; exactly one occurrence means the
import line and nothing else.

    files scanned                                          3459
    imports used nowhere but their own import line            93

By package, the ones with more than two:

    33  package/oxlint-plugin/prefer-readonly-parameter-type/src
    19  package/module/translation-repair/src (moved on 2026-10-18 to `package/translation-repair/*/src`)
     5  package/git-policy/cli/src
     4  package/figma/to-penpot/src
     4  package/pi-plugin/advisor/src
     4  package/pi-plugin/auto-mode/src
     3  package/wg/cli-quicker/src
     3  package/dev-script/vm-builder/src

THE MEASUREMENT UNDER-COUNTS, deliberately, and the direction matters.
A name mentioned in a comment or inside a string reads as used, so the real number is at least
93.
It never over-counts in the other direction: a name that appears once cannot be in use.
The 19 in `module/translation-repair` (split on 2026-10-18 into the packages under `package/translation-repair`) were
removed the same night, and the six that started this
were removed before them; both are verified by a full build, test, lint and type-check run.
The other 74 are untouched, because they belong to packages this session was not working in.

## The question

Do you want the lint rule to enforce this, and if so, how narrowly?

### Option A: leave the rule off, keep the census as a task

The census script becomes a repo task somebody can run, and the number is checked when it is
worth checking rather than on every lint run.

-   Pro: costs nothing to adopt, changes no existing configuration, and keeps every reason the
    original decision named intact, including deliberate work-in-progress names.
-   Pro: the heuristic is honest about being a heuristic, and a periodic reading is exactly what
    a heuristic is good for.
-   Con: nothing stops the next extraction leaving imports behind, and nobody runs a task they
    have to remember.
-   Con: 74 names stay dead until somebody chooses to look.

### Option B: turn the rule on for imports only, if oxlint can express that

`eslint/no-unused-vars` takes options; a configuration reporting imports while ignoring local
variables, parameters and caught bindings would enforce exactly the half that the original
rationale does not cover.
Whether oxlint's implementation supports an import-only shape has NOT been verified: the rule's
ESLint options (`vars`, `args`, `caughtErrors`) do not name imports separately, so this may not
be expressible without a plugin rule of our own.

-   Pro: catches the defect at the moment it is introduced, in the tool that already runs.
-   Pro: leaves every deliberate unused local alone, so the original decision survives intact.
-   Con: may not be expressible in the rule's options at all, in which case this becomes a
    plugin to write and maintain.
-   Con: 74 existing names have to be cleared first, across packages with their own owners.

### Option C: turn the rule fully on and clear everything

-   Pro: one rule, no exceptions, nothing to explain.
-   Con: contradicts a decision taken deliberately and documented in place, and the reasons
    given for it are still true of local variables.
-   Con: the cleanup is repo-wide and touches packages nobody is working in, which is where
    a mechanical edit is least likely to be noticed if it goes wrong.

### Ranking: A > B > C

A beats B because B's central mechanism is unverified: the option shape may not exist, and the
fallback is writing and maintaining a plugin rule for a defect that has cost this repo one
cleanup so far.
A is reversible in an afternoon if the number grows.

B beats C because the original decision's reasons are sound for local variables and silent about
imports, and B keeps the part that is sound.
C would force the deletion of names deliberately kept during work in progress, which is the exact
friction the rule was turned off to avoid.

## If you want none of this

Say so and the census stays a scratch script.
The 19 names in `module/translation-repair` (split on 2026-10-18 into the packages under `package/translation-repair`)
are already gone either way; nothing else in the repo
changed.
