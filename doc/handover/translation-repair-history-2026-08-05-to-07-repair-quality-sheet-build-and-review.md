# Translation repair history 2026-08-05 to 2026-08-07: the repair-quality sheet, built and reviewed

How repair quality was made measurable on its own grading sheet:
what was wrong with detection-only grading,
the region and record types,
the two-sheet decision,
the denominators it enables,
the cache invalidation,
the deferred defects,
and the second review of the implemented code.

Part of [Translation repair history: 2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Session 2026-08-06: repair quality made measurable, on its own sheet

TASK #47 COMPLETE.
Commits `a10dc94ab` (provenance and sheets),
`bac30e20d` (tests),
`cc9f6ad58` (sheet contradiction fix).

### What was wrong

The grading sheet asked exactly one question per sampled item,
whether the accepted issue is a real defect,
and nothing anywhere recorded what the pipeline actually WROTE.
A correct detection carrying a poor repair therefore scored as an unqualified
success.
Round two shows the gap directly:
four of the thirty-seven true positives came back with the grader asking
whether there was a better way,
and all four counted as successes.

The checker stage does not substitute for that measurement.
Measured across the thirty-one settled artifacts:
2257 accepted issues,
2215 with `resolved: true`,
so checkers confirm 98.1% of repairs.
A verdict that near-unanimous separates almost nothing.

### What was built

`repair-region.ts`:
`RepairRegion` (`envelopeId`,
`issueIds`,
`before`,
`editorAfter`) and
`collectRepairRegions`.
REGION-shaped,
not issue-shaped,
and that is load-bearing.
`deriveEditableEnvelopes` merges overlapping AND touching intervals
(`interval.start <= last.end`),
so one replacement can serve several accepted issues and fix only some of them.
Copying a replacement onto each issue as "the repair for this issue" would erase
that;
the served issue ids travel with the region and the sheet discloses siblings.

`repair-record.ts`:
`RepairIssueRecord` moved here out of `repair-translation.ts`,
plus `RepairDisposition`
(`shipped`,
`not-selected`,
`withdrawn`,
`no-region`) and `buildIssueRecords`.
Shipping status is decided HERE,
not in `repairChunk`,
for two independent
reasons.
A document blocked for non-translation returns its input and withdraws every
slice repair at once,
which no slice can know.
And `runRefinePhase` sets `changed: true` on a refinement-only rewrite,
so after that phase `changed` no longer answers whether an accuracy repair was
selected;
`ChunkRepairOutcome.accuracyPatchSelected` is the frozen accuracy-stage fact.
One builder serves both driver exits so they cannot drift,
which is how the blocked exit came to report `resolved: false` correctly while
carrying no repair provenance at all.

`ChunkRepairOutcome` also gained `refined`,
set by `refine-phase.ts` when a refinement is kept.
`RepairIssueRecord.finalSliceText` is carried ONLY when `refined` is set,
which is exactly where the recorded replacement stopped being the returned
wording.
Always carrying it would multiply a large document's slice text by its
accepted-issue count
(Dethelly has 260 accepted issues) for no added fact.

`artifact-guard.ts` holds the shape checks both artifact readers share;
`artifact-repair-read.ts` reads provenance back out with ONE tolerance,
a named `{ kind: 'unrecorded' }` for artifacts predating repair recording.
Absence and emptiness stay distinct all the way to the sheet:
no disposition means repair quality is unknowable for that item,
while `no-region` is a real measurement belonging in the coverage denominator.

`chunk-measure.ts` holds the selection measurements,
extracted because `repair-chunk.ts` hit the 300-line budget.
Behavior unchanged.

### The two-sheet decision

Repair grading is a SEPARATE sheet
(`repair-sheet.ts`,
`formatRepairSheet`,
path stem `repair-sheet-<seed>`),
not a second box on the detection sheet,
and that is a measurement decision rather than a layout one.
Showing a grader the correction makes an alleged defect look more salient,
which moves the answer to "is this a real defect".
Round two's precision was measured by a sheet showing no repair,
so folding repair text into round three's sheet would compare two rounds through
two different instruments and credit the change of instrument to the pipeline.
The detection sheet is byte-for-byte unchanged.
A unit test asserts the detection sheet contains no repair text;
nothing else would catch a leak.

The repair sheet also never prints the checker verdict,
since 98.1% agreement would anchor the human toward agreement on precisely the
population they are auditing.
Its header orders the detection sheet graded first,
and items whose repair did not reach the reader carry no grade box but do carry
a plain-language reason,
so the coverage denominator stays visible instead of turning into a gap.

### Denominators this enables

Detection precision:
human-confirmed real defects over all sampled accepted issues.
Unchanged from earlier rounds.

Targeted repair coverage:
real defects with a shipped targeted repair over all real defects.

Conditional repair effectiveness:
human-confirmed fixes over real defects with a shipped targeted repair.

End-to-end repair yield:
human-confirmed fixes over all real defects.

`Y` on the repair sheet means the returned wording fully resolves the defect and
introduces no new error nearby.
The number is repair EFFECTIVENESS,
not broad repair quality:
a `Y` is still compatible with a better phrasing existing.

### Verified at the user boundary

Exercised `parseSettledArtifact` -> `extractGradingCandidate` ->
`drawStratifiedSample` -> both formatters over the REAL thirty-one artifacts
(read-only;
sheets written to `${HOME}/temp/agent`,
never the runs dir).
All 2257 accepted issues parse,
all 2257 read as `unrecorded`,
the detection sheet renders 50 grade boxes exactly as before,
and the repair sheet renders 50 `NOT GRADABLE` items with zero grade boxes.
A synthesized recorded artifact exercised the other path and surfaced a real
defect,
fixed in `cc9f6ad58`:
an item could say "grade the FINAL wording" and,
one line later,
"not graded",
because a repair can lose its slice selection and still have its paragraph
rewritten by the naturalness lane.

### Cache invalidation

`repair-translation.ts` gained `SLICE_CACHE_VERSION = 2`,
mixed into every slice-cache key.
A resumed pre-change outcome would splice repair-less slices into a run and
contribute ungradable items to a precision sheet with nothing looking wrong.
`isChunkRepairOutcome` in `slice-cache-store.ts` also now requires
`repairRegions` and `accuracyPatchSelected`,
but the key is the primary mechanism:
the structural guard cannot detect an existing field CHANGING MEANING,
only one going missing.
Measured before bumping:
the single in-flight cache directory (`TianqiChen666`) is EMPTY,
so no partial work was discarded.

### Operational fact for round three

`corpus-pass.ts` treats any existing artifact file name as settled and skips
that entry,
so repair provenance will NOT appear for the entries already on disk however
many times the pass reruns.
Round three needs a fresh artifacts directory.
Filed as task 55,
including the question of archiving rather than deleting the
round-two artifacts,
which remain the calibration set for task 48.

### Deferred defects found while doing this

Filed rather than fixed,
because each changes what the pipeline decides:

- Task 52.
  `runCheckerStage` is asked about EVERY accepted issue,
  including
  unenveloped ones and ones whose envelope received no surviving operation,
  and
  `resolvedHighSeverity` / `resolvedTotal` are computed over that whole set.
  So
  a patch touching issue A can beat unchanged on credit for issue B that no
  operation touched.
  The new provenance exposes this as records with
  `resolved: true` and `repairDisposition: 'no-region'`.
- Task 53.
  `regressionCount` can only count EXISTING accepted issues the
  checkers marked regressed,
  so a wholly new defect the patch introduces has
  nowhere to be counted,
  despite the field being documented as "new defects".
  `changedCharCount` sums `Math.max(baseText.length, newText.length)`,
  which is
  touched-region size rather than differing characters.
- Task 54.
  Emptying `resolvedIssueIds` when unchanged wins is semantically
  correct,
  but the checkers' opinion of the rejected candidate is lost.
  The
  rejected repair itself is now recorded;
  only the verdict on it is not.

### Tooling note

`pi`'s file attachment is a POSITIONAL argument form,
`pi [options] [@files...] [messages...]`,
not `@path` written inside the message text.
Writing the paths into a prompt file and passing that file got back
"the prompt contains file paths,
not their contents".
Passing each source as its own `@path` argument and the question as the message
works,
and sidesteps the `Argument list too long` failure that killed an earlier
163 kB inline prompt.

### Second review, and what it caught (2026-08-06)

A second sol pass over the IMPLEMENTED code found five real defects,
all fixed in `21134e8bb`.
Recording them because each is a class of mistake,
not a one-off.

REFINEMENT-ONLY SHIPMENT WAS HIDDEN.
`runRefinePhase` runs whatever the accuracy selection decided,
so a slice whose targeted repair lost can still be rewritten and reach the
reader.
The sheet said only "nothing reached the reader",
which is true of the repair and false of the text.
The disposition itself is correct and was left alone;
it describes the TARGETED repair's fate,
and its TSDoc now says so explicitly.
The returned slice is shown either way now;
only the grade box depends on a targeted repair having shipped.
An earlier fix of mine (`cc9f6ad58`) had made this worse:
it removed the refinement caveat from ungradable items to resolve a
contradiction,
which resolved the contradiction by deleting the true half.

REPLACED TEXT CROSSED INTO MARKDOWN GRAMMAR RAW.
A replacement is arbitrary corpus-derived model output.
Interpolated into a bullet list it can contain a line starting `###`,
a literal `- repair grade: [ ]`,
or a backtick run,
and the first invents a heading,
the second puts a grade box on the sheet that nobody wrote,
and the third breaks the block.
Curly quotation marks are not Markdown syntax and prevent none of it.
This is AGENTS.md SYB,
and it was missed in review-one because the question
asked was about measurement bias rather than about encoding.
Both sides are fenced now via `markdown-fence.ts`,
which chooses the fence against the content the way
`candidate-select-wire.ts` already chooses its prompt fence.
The test asserts grade boxes OUTSIDE fenced blocks,
since fencing does not delete the injected characters,
it stops them being read as sheet.

A RECORD CARRYING REGIONS BUT NO DISPOSITION READ AS A LEGACY RECORD.
Keying the legacy judgement on one field made a half-written repair the one
malformed shape a strict parser would silently accept.
Absence is now judged over every repair field.

A FINAL DRAW OVER PRE-RECORDING ARTIFACTS RENDERED FIFTY UNGRADABLE ITEMS AND
REPORTED A NUMBER ANYWAY.
`--final` now refuses when any sampled item carries no recorded repair.
Reachable simply by drawing against a directory still holding an earlier
round's artifacts,
which `corpus-pass` never overwrites.

SHEET WRITES WERE NOT TRANSACTIONAL.
The detection sheet was written before the repair path was resolved,
so a refused repair path left a protected detection sheet with no companion.
Both paths resolve before either write now.

Also from that review:
the preliminary banner now reaches both sheets rather than only the detection
one;
the slice-cache guard requires `refined`;
the zh original is carried onto the repair sheet,
since that is what "does it fix it" is answered against;
and a deletion says it is a deletion instead of rendering `after: ""`,
which read as a rendering fault.

TEST FIXTURE THAT PROVED NOTHING.
`repair-record.unit.test.ts` used the refined wording as `repairedText` even
when `refined` was false,
so the conditional-`finalSliceText` test passed while modelling a state its own
documentation says cannot occur.
The fixture now returns patched text carrying the replacement verbatim for a
shipped unrefined slice,
and a test asserts that containment directly.

STILL OPEN FROM THAT REVIEW,
deliberately:
the slice-cache key covers version,
index,
and both texts but NOT the model
roster,
adjudication config,
editor addendum,
or identity context,
so a
cross-run cache could return an outcome produced under different inputs.
Not a live hazard for round three because the round-two slice cache was
archived with its artifacts and the version bumped,
but it is a real gap.
