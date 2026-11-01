# Audit ledger: work merged from parallel briefs

What went wrong when two pieces of work written apart met in the same files.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## M144: two briefs touched one entry file, and both agents built the same block

Status:
happened on 2026-10-10 (UTC),
by the lead's notes for the follow-up round of B391 to B400;
caught at the merge,
before `477bbd6601509ac08d867e858af680a59c4f91eb`,
which holds one copy.
The follow-up round's briefs named `RUN_OUTSIDE_READS` as an item of the pipeline-wide reads (B398 to B400)
and `corpus-pass.ts` as a file of the corpus pin's change (B394),
and neither said which agent owned the statements of the entry files both would touch.
Both agents removed the constant and built an `outsideReadsFrom` block in its place,
in `corpus-pass.ts` and in `rendering-audit-settled.ts`.
The lead found the duplicate block in `rendering-audit-settled.ts` among eleven conflicting hunks at the merge
and kept one;
the removal stands in B394's fix commit,
and the fix commit of B398 to B400 keeps a message line claiming it.
At the same merge,
a case B394's agent wrote lacked two values B398's agent made required,
and the lead added them in B398's fix commit.

Prevention:
when two briefs touch one entry file,
the briefs name which agent owns each statement of it,
and the lead merges each patch's red commit before its fix.
`mistake-prevention.md`,
"Copies of shared code".

## M153: features built apart broke each other's fixtures when merged onto one head

Status:
happened on 2026-10-12 (UTC);
both caught by a gate or a named run before any pass,
and fixed in `c5d6f1d2d6` and `24a4377fed`.
The tense work (B419,
B433) and the consistency pass (B431) were built in separate worktrees from older heads,
and both changed what artifact generation 15 holds.
Each merged with no textual conflict in code,
yet five of the tense work's replay cases built artifacts by hand:
two wrote generation 15 without the `consistency` section its reader requires,
and three restamped a generation 15 artifact as 14 with the section still in it,
which the older reader refuses.
The lead's gate g76 caught the first three,
and the named run after the next merge caught the other two.

Prevention:
when two pieces of work both change what one record generation holds,
the second merge runs every test that writes that generation by hand before its commit,
and a fixture that restamps a generation does it through one helper
(`generation-restamp.test-fixture.ts`) that knows what each generation lacks.
`mistake-prevention.md`,
"Files a later run reads back".

## M156: two agents were briefed onto the same production modules at once

Status:
happened on 2026-10-12 (UTC);
caught by the lead's named run after both merged,
before any pass.
The lead launched the consistency review (B443 to B446) and the consistency own tests (B437 to B440) together
from one base,
keeping them apart by test file only,
while both briefs let each change the production modules under test.
Both fixed the reader's slot-offset and place-count rules,
each its own way,
the merge held the place-count rule twice,
and four of the own-test files pinned rules or messages the review had changed.
The lead kept one statement of each rule on merge
and gave the reconciling of the cases to a follow-up.

Prevention:
work that may change the same production files runs one after the other,
or one brief owns those files and the other only reads them;
separating test files is not separating the work.
`mistake-prevention.md`,
"Work merged from parallel briefs".

## B366: the runner observations change met main's rewrite of its files when it was ported

Red in `bb2a8b9ad1e164974e54d2750432a25b267ef0de`,
fixed in `cfac5b981`.

Found on 2026-10-10 (UTC) by the agent porting the runner observations change onto `4cca8c301`,
from its base `f3c2bae13`.
Of the change's 77 modified files,
15 had changed on main,
and three conflicted.
`corpus-run/bench-sample.ts` conflicted because main had replaced `sliceOne`,
which held the change's only hunk there,
with `textOf`,
`pageOf` and `stepOf` under `allInInputOrder`;
main's file was kept,
and the change's rule moved into `textOf` (B367).
`corpus-git-context.ts` conflicted on two TSDoc paragraphs added at one place,
main's on `LC_ALL` and the change's on `GIT_CEILING_DIRECTORIES`,
and both were kept;
`corpus-run/bench-sample-draw.unit.test.ts` conflicted where both sides appended cases,
and the clean part of its merge doubled an import.
Four more broke on main's code with no conflict marker:
main's case of `corpus-git-context` called the builder without `cloneDir`,
which the type check found;
main's `ordered-page-reads.test-fixture.ts` built a `CorpusReadError` without the `commit` the change requires;
two of main's `rendering-audit-settled-input` cases named a lacked commit while expecting `missing-object`;
and main's bench draw cases read one remedy where the change gives two.
On the port build,
by the agent's run,
61 of 62 named test files passed,
and the one holding the change's own case on a lost object failed as expected,
since that case needs the rethrow of B367.
The red commit fails 51 of the 65 named test files
against `4cca8c301`'s production code plus the fix of `fba9c5b60`,
four of them without running a case,
since they import exports the fix adds
(`code-points`,
`corpus-git-context`,
`editor-calibrate-standing` and the new `editor-standing-absence`);
the other 14 pass either way,
and the red commit leaves the type check failing on those exports.

The fix:
the fixture takes `commit` from its case,
the two settled-input cases expect `missing-commit`,
and `CLONE_ADVICE` is split into `ABSENT_PAGE_ADVICE` and `OTHER_FAILURE_ADVICE`,
with the findings of the review of the change fixed beside the port (B367 to B377).
`corpus-source.ts` and `editor-standing-read.ts` went over the line cap,
so the commit probe moved to `corpus-commit-probe.ts`
and `noJudgedRoundLine` to `corpus-run/editor-standing-absence.ts`,
each with a test file of its own;
two barrel lines export `gitSearchCeiling` and `noJudgedRoundLine` for tests.
Run on the merged tree before the fix commit:
lint clean,
49 scans passing,
and the suite at 2,030 passing with no failure.

Open to the owner's veto:
The DONE line gains `declined=`,
the `--only` parenthetical and one more English read per finished entry;
the report names no alternative.
`countSettled` and `judgedAuthors` are removed;
the report names no alternative.
The two splits at the line cap add the modules `corpus-commit-probe.ts` and `corpus-run/editor-standing-absence.ts`;
the report names no alternative.

Recurrence:
`mistake-prevention.md`,
"Lint and edits".

## M174: the first merged tree of the fourth review's fixes held three slips

Status:
found on 2026-10-13 by gate g88 on the first merged tree;
each was mended before the commit.

- A case pinned "unknown generation" at the number 17,
  which generation 17 made known.
  It now reads the known list,
  as `artifact-dispatch.unit.test.ts` already did.
- Two test files carried one helper.
  It now lives in `runs-of-pieces.test-fixture.ts`.
- Two predicates tested membership in a string,
  so they needed a blankness guard.
  They now test membership in sets.

Prevention:
a case about an unknown member reads the known list and names no number;
a helper two test files need lives in one fixture;
membership is tested on a set and never on a string.
The first merged tree runs the whole gate before its results are relied on.
`mistake-prevention.md`,
"Work merged from parallel briefs".
