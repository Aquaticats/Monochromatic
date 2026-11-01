# Preventing mistakes: moves, splits and rewrite scripts

Rules for moving or rewriting many files at once:
the dry run of the move,
the slices handed to parallel agents,
the scans and controls that must read the same set after the move as before,
and the estimates and measurements that decide what the move costs.
The first record is the split of `package/module/translation-repair` into the packages under
`package/translation-repair`,
of 2026-10-18 (UTC).

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.

## A move's dry run

What happened:
the dry runs of the split built the packages and ran the suite but never linted,
and the move commit landed oxlint findings in 30 of 36 packages (ledger M239).
One of the packages was named `test-fixture`,
which a repository-wide ignore pattern hid from the lint,
so its lint task found no file to lint and reported nothing (ledger B655).

The rule:
the dry run of a move or of a rewrite script runs the same checks as the gate:
the build,
the lint,
the types,
the suite and the scans.
A new package's first lint names a nonzero count of files linted;
"No files found to lint" is a failure and not a pass.

What enforces it:
habit.
The lint of a package with no file to lint exits cleanly,
so nothing but the count read from its output shows the fault.

## Slices of one move

What happened:
the briefs for two parallel slices did not say who owns the shared fixtures `runBuiltCommand` and `runProbeOver`,
and both agents changed them in incompatible shapes,
so the merge conflicted and needed an agent of its own (ledger M243).
One slice never ran the scans after its changes,
and six findings reached the merge agent's checks (ledger M248).

The rule:
a brief that splits work across agents names the owner of every shared helper the slices may touch.
Every slice brief lists the scans in its checks,
by the name of the task that runs them,
and says which scans the task does not include.
Work that is merged from such briefs is also compared with each round's own change
(see "Work merged from parallel briefs" in
[Preventing mistakes: shell commands, edits, commits and agents](mistake-prevention-shell-edits-commits-and-agents.md)).

What enforces it:
habit.
The scan tests of `package/translation-repair/source-scan` fail on the findings of M248 when the merge agent runs them,
and not before.

## Scans and controls across a move

What happened:
the TSDoc example scan listed its own package's directory,
so after the move it read only the scan tests and passed without reading any source (ledger B651).
The control that compared the census and the audit before and after the move read git's renames end to end,
and two fixtures that moved as exact renames and were edited afterwards read as missing (ledger M251).

The rule:
a scan's read set is compared before and after a move,
by the count and the list of the files it read.
A scan whose input set shrinks to nothing passes,
so a count of zero is a failure.
A control that rests on renames is read at the commit that moved the files,
and the renames inside the family are chained from there to the head.

What enforces it:
the controls of the split,
recorded in `doc/planning/translation-repair-package-split.md`
and in the commit `389f03897c`.
No scan checks its own read set.

## Estimating and measuring a cost

What happened:
the plan of the split predicted 3 to 5 seconds of lint per package from small slices of the old package,
and the family linted one package at a time in 333 to 352 seconds against 25.5 seconds for the old package
(ledger M252).
The first probe of the raised cache bounds edited the plugin's `src` while the root config loads the built bundle,
so its edit never ran and the measure read the old bounds (ledger M253).
A comparison of closures serialized maps with `JSON.stringify`,
and key order read as 708 differing closures (ledger M254).
The first baseline run of the whole-repository lint comparison linted with the hardened plugin:
the saved baseline bundle kept its old file times,
and the lint task rebuilds `package/config/oxlint/dist` when a plugin source is newer than the oldest output (ledger M262).

The rule:
a cost that scales with the number of packages is measured on the whole family before it is promised,
and never estimated from the cost of a slice.
Before a measure of an edit,
the artifact the measure loads is searched for the edited value.
Maps are compared as sorted entries,
and one example is read before a count of differences is trusted.
A bundle that stands in for another is swapped in with fresh file times,
and the live bundle is proved after every package and not only at the start of a phase.

What enforces it:
habit.

## Scripted merges of test blocks

What happened:
two top-level describe blocks in each of three test files were merged by a script that rewrote text,
and the three files broke (ledger M290).
The merge also left trailing whitespace on blank lines inside object literals,
which `oxlint --fix` does not strip and the source scan for trailing whitespace lists.

The rule:
two describe blocks are merged by hand or with a tool that reads the syntax tree,
never by text substitution.
After any scripted merge of files the `test:unit` of the source-scan package runs,
because it reads what the formatter leaves.

What enforces it:
`trailing-whitespace.unit.test.ts` finds the whitespace;
the merge itself is a habit.

## Files near the line limit

What happened:
the review fixes of the every-provider-dry stop added blocks to files near the `max-lines` limit
(300 lines,
comments and blank lines skipped),
and three extractions followed the code:
`stage-voice-lost.ts`,
`image-reading-contain.ts` and `run-seats-refusal.ts` (ledger M291).

The rule:
before a block goes into a file near the limit,
the file's lines are counted the way the rule counts them,
and the extraction is planned and made first,
as its own commit,
named for what moves.

What enforces it:
the `max-lines` rule finds the overflow after the code is written;
the planning is a habit.
