# The third rendering: what settled-artifact consumers read, and the reader for what would ship

Why the consumers of settled artifacts find no records on a current artifact,
and what a naive port would cost;
the audit error sized,
the consumer inventory corrected,
and the reader that derives what a document would carry;
both consumers made to ask both questions,
the zero-count refusal found already present,
and the counters renamed.

Part of [The third rendering](the-third-rendering.md),
which lists every file of this record.

## What the settled-artifact consumers actually read

`#166` asked whether a graded sheet drawn from an accepted repair-lane issue
 describes text a later stage replaced,
which would make the precision number describe a draft rather than a publication.
The answer is no,
for a reason nobody had checked:
those consumers cannot read a current artifact at all.

### The schema split as measured

Every artifact the current pipeline has written carries `artifactSchemaVersion: 2`.
Across every run directory under the agent scratch root,
every settled artifact is version 2 and none carries a top-level `issues` array;
the count was 24 when measured and 25 once `Zha_Ke` settled.

`artifact-probe-read.ts` reads `artifact.issues`,
and filters each record on `record.repairDisposition === SHIPPED_DISPOSITION`.
`artifact-repair-read.ts` and `repair-sheet.ts` key on the same field,
which `repair-record.ts` defines as the string `shipped`.
THE PATH MOVED,
and an earlier revision of this section said the fields were gone.
They are not.
Version 2 carries both at `lanes.repair.result.issues[]`,
measured across 28 version 2 artifacts:
0 carry a top-level `issues` array,
27 carry the nested one,
holding 871 records whose `repairDisposition` reads `shipped` 432 times,
`no-region` 427 and `not-selected` 12.

So the readers find nothing because they look at `artifact.issues`,
which version 2 does not have,
and not because the data stopped being written.
That makes the port a re-pointing rather than a reconstruction,
which is much cheaper than this section first recorded.

Version 1 artifacts do carry both fields,
which is the positive control this claim needs:
an uncapped sweep of every artifact directory under the agent scratch root found
 164 distinct version 1 artifacts carrying `issues` arrays,
the largest with 405 records.
The raw file count is 492,
because three identical worktree-copy payloads hold the same run.
The reader was never broken.
The artifact moved out from under it.

### Why the word was accurate when it was written

`repairDisposition: 'shipped'` is documented as the disposition of a repair
 that the returned document carries,
and `repair-record.ts:351` is its only writer.
In the version 1 pipeline that claim was exactly true,
because the repair lane's returned document was the publication:
no stage ran after it.

Two stages run after it now.
The lane contest chooses between the repair lane,
 the translate lane and the incumbent,
and the consolidation can replace whatever the contest settled on.
A field written before either runs cannot describe what either decided.

### What the naive port would cost

The version 2 analogue of that field is `lanes.repair.delivery[].delivery.kind`,
and mapping one onto the other is the obvious port.
It would be wrong at every row this repository can measure.

On the artifacts carrying both a lane contest and a consolidation,
6 rows claim `replacement-shipped`,
and the text of 0 of them reached the page.
The contest alone had already chosen another lane at 5 of the 6,
and the consolidation then overrode 4 rows including the one the contest had kept.
That population is 2 artifacts,
so it establishes the mechanism and its direction rather than a rate.
The corroborating figure is the consolidation override rate recorded for the band pair,
8 of 10 decided slices in run 8 and 7 of 8 in run 9.

### The failure mode this leaves today

A reader that finds no records does not mislead;
it returns nothing.
The danger is that nothing is indistinguishable from a clean result:
an introduced-defect probe reporting no damage over zero shipped records
 reads exactly like one reporting no damage over every shipped record.

`artifact-probe-read.ts` anticipated this and already carries the discriminator,
`shippedRecords`,
documented so a run whose probe never fired is distinguishable from one that had
 nothing to ship.
`probe-telemetry-report.ts:120` prints it and `score-probe.ts:180` sums it,
so the zero is reported rather than hidden.
What no reader does is treat that zero as a refusal,
which is what the port must add.

### Decided that every consumer names the question it asks

Two different questions are worth answering,
and the split is what `#166` exists to fix.

The lane-scoped question is whether a lane's own output carried a given repair.
It stays.
Roster decisions rest on lane attribution,
and a consumer that could only see the page could no longer tell which voice
 earned a result.

The shipped-scoped question is whether the text reached the page.
It is added beside the lane-scoped one rather than replacing it.

Neither is a default.
Every consumer names the question it asks in its own identifier rather than in a
 comment,
because the defect being repaired here is a name that asserted more than the
 value meant.

### Decided that the shipped text is derived once

A single reader answers the shipped-scoped question for one slice,
and every consumer calls it rather than walking the artifact itself.

It cannot simply read `shipped.text`.
Measured on the settled artifacts,
`shipped` carries a `text` key only where the terminal is `consolidated`.
Where the terminal is `slate-kept-standing` or `no-standing-text`,
`shipped` carries a bare `kind` of `unchanged` and no text at all.

So the reader walks the deciders in the order they ran:
a `consolidated` terminal yields `shipped.text`;
any other terminal yields the standing text,
which is whatever the lane contest settled on;
an artifact with no consolidation yields the contest result directly;
and an entry with no contest yields the incumbent.

### Still owed

The port itself is queued behind the source freeze,
with `#167` and `#168`.
It carries three parts:
the shipped-text reader,
the renaming of every consumer that currently implies a page-level claim,
and the zero-denominator refusal.

`#66` recorded that the probe named neither damage that shipped.
That answer was reached on version 1 artifacts,
where the reader could see records,
so it is not void.
It is also not evidence about anything the two-lane pipeline has produced,
and it should not be cited as if it were.

## Sizing the audit error

`#166` item 1 called `rendering-audit-settled-input.ts` wrong as it stands
 without sizing it.
Sized now,
against `laneSelection.slices[].verdict` on every artifact whose contest settled.

The audit derives every subject from `lanes.translate.delivery` alone.
Of 6 contest slices across 2 artifacts,
5 are subjects whose translate-lane text is not what shipped:
1 where the repair lane won the contest,
1 that settled neither,
and 3 where the translate lane won and the consolidation then replaced it.
`Acheron` is wrong at 3 of 4 slices and `Weideriche_` at 2 of 2.

The audit therefore describes shipped text at 1 of 6 slices.
The population is 2 artifacts,
so this is mechanism and direction rather than a rate,
and it is the whole population that exists.

### Two fields named verdict

Sizing that error produced a wrong number first,
which is worth recording because the mechanism will catch the next reader too.

A settled artifact carries two fields spelled `verdict` at sibling paths.
`comparison[].verdict` is `SliceLaneVerdict` from `lane-comparison.ts:39`,
a string union naming which lanes changed the slice.
`laneSelection.slices[].verdict` is `ArtifactContestVerdictV2` from
 `corpus-run/artifact-v2-contest.ts:29`,
an object discriminated on `kind` naming who won.
Only the contest field answers a question about shipping.

Reading `row.verdict.kind` against the string form yields `undefined` on every
 row,
which is not an error and not an obviously wrong answer.
The first probe written to size the audit error did exactly that,
and reported 142 of 142 slices wrong.
The correct read gives 5 of 6.

It was caught by a positive control rather than by review:
the probe printed the distribution of the field it was reading,
and every one of 142 rows fell in a single bucket.
That is now the standing rule for artifact probes here.
A probe prints the distribution of what it reads before its headline number is
 trusted,
because two probes in one session returned a false total from a wrong field path
 and neither threw.

Filed as `#170`,
queued behind the source freeze.

## Correcting the consumer inventory

Two claims recorded against `#166` are wrong,
and both were mine.

### The six files exist

The correction filed on `#166` said `draw-entry-load.ts`,
`score-probe.ts`,
`probe-telemetry-report.ts`,
`probe-relabel-*.ts`,
`score-attribution.ts` and `score-crosscheck.ts` do not exist.
All of them do,
in `package/module/translation-repair/src/corpus-run/` (moved on 2026-10-18 to
`package/translation-repair/*/src/corpus-run/`).

The listing that produced the claim ran without recursion in the top-level
 `src`,
so it saw only the files beside `index.ts`.
The same mistake produced the empty grep it was meant to correct:
the first grep named the right files and searched the wrong directory,
and the correction then declared them absent for the same reason.
A file list is a search result,
and it fails the way `QRY` describes.

### The discriminator is read

That section also said nothing reads `shippedRecords`.
`probe-telemetry-report.ts:120` prints it in the `PROBE` line beside the entry
 count,
and `score-probe.ts:180` sums it across entries.
A run whose probe saw no shipped records already reports a zero rather than
 hiding one.
What no reader does is treat that zero as a refusal.

### What survives once the real files are read

The family reaches artifacts through `artifact-probe-read.ts`,
which reads `artifact.issues` and filters on `repairDisposition`.
`probe-relabel.ts` and `probe-relabel-artifact.ts` read `.issues` directly.
No version 2 artifact carries either field,
so the conclusion stands:
on a current artifact this family sees no records.

None of the ten files mentions `consolidat`,
`laneSelection` or `contest`,
so none of them accounts for either decider.

### The port has a pattern to follow rather than invent

`score-probe.ts:147` already carries the idea the port needs.
It collects `refinedIssueIds` for issues whose slice the naturalness lane
 rewrote after the probe ran,
commenting that the probe's verdict is about wording that did not ship.

That is the right concept applied one stage too early.
It watches a rewrite inside the repair lane and stops at the lane boundary,
while the contest and the consolidation can each replace the same text
 afterwards.
The shipped-scoped reader extends that set rather than introducing a new one.

## What would ship is derived, because nothing publishes it

The reader `wouldShipTextFor` lands in
 `package/module/translation-repair/src/corpus-run/would-ship-text.ts` (moved on 2026-10-18 to
 `package/translation-repair/page-check/src/corpus-run/would-ship-text.ts`).
It answers,
 for one slice,
 what a document assembled today would carry there.

### The name is what it can prove

Not `shippedTextFor`.
Nothing assembles a document out of these decisions at all,
so nothing this reader names has been published.
The founding defect of this family was `repairDisposition: 'shipped'`,
a name that went on asserting a publication after the stage performing it
 stopped being last,
and a reader called `shippedTextFor` over a pipeline that publishes nothing
 would repeat that on its first day.
The pipeline-scale version of that gap is recorded as its own task,
 because whether the pipeline should publish is the owner's decision.

### No reading may carry an empty string

`standingTextFor` returns the empty string at a declined contest,
 deliberately,
 because it answers what a consolidation slate must beat.
`ArtifactConsolidateShippedV2` already documents the consequence:
 a consumer reading a bare per-slice `text` and writing it into a document
 would delete every declined slice outright.

So a reading carries either wording plus the decider that settled it,
or a named silence,
and never a `text` key holding nothing.

This corrects one line of the decision recorded earlier in this document.
That line said any terminal other than `consolidated` yields the standing text.
Taken literally it returns the empty string at `no-standing-text`.
The archive is the floor underneath both declines instead:
 this pipeline repairs an English translation that already exists,
 so a slice no later stage displaced still carries what the archive published.
Where the archive holds nothing either,
 the reading names which stage left the slice silent.

### The order the deciders ran in, read backwards

A `consolidated` terminal yields its text.
Otherwise the contest's winning lane.
Otherwise,
 where the contest never saw the slice,
 the wording both lanes agreed on,
 since a slice is eligible exactly where the lane texts differ.
Otherwise the archive.

Terminals are tested by shape rather than enumerated by name,
so the eight keeping terminals need no list and a terminal added later cannot
 silently begin yielding text.
That includes the retired `slate-kept-standing` spelling.

### The control was registered before the instrument existed

Task `#166` recorded these numbers on 2026-08-22,
 measured by hand,
 before this reader was written.
Running the reader over every settled artifact reproduces them exactly:

-   `Acheron`,
    4 contested slices,
     the translate lane's text is what would ship at 1,
     so the audit is wrong at 3 of 4.
-   `Weideriche_`,
    2 contested slices,
     the translate lane's text is what would ship at 0,
     so the audit is wrong at 2 of 2.
-   Together,
    1 of 6.

Across the whole population,
 47 version 2 artifacts parsed and 0 refused,
 14 of them carrying a settled contest across 6 entries.

The first roll-up of this measurement reported counts above the registered ones,
 because it tallied per entry while the registered numbers are per artifact.
An entry carries one artifact per pass,
 and the passes are not one:
 the two entries the control names carry two artifacts each,
 while `Zha_Ke` carries four across two run roots.

The copies are not always the same artifact twice.
The two `vub-run1` copies of `Zha_Ke` disagree at two of its four contested slices,
 where the contest reads `settled-neither` in the earlier and `lane-won` in the later.
That is not a cache the second pass re-bought.
`pass-entry.ts` discards a settled entry's whole slice cache after the artifact write,
 bounding the directory to documents still in flight,
 so a pass that settles every entry leaves nothing for the next one to resume.
The second pass was cold rather than resumed,
 the repair lane bought new wording at three of those four slices,
 and the contest key covers that wording,
 so re-asking was the correct answer to a question whose inputs had moved.

### One decomposition differs, and the difference is the point

The earlier hand count described the five mismatched slices as one where the
 repair lane won,
 one that settled neither,
 and three where the translate lane won and the consolidation replaced it.
The reader attributes four to the consolidation and one to the archive.
Both totals are five.
The disagreement is that one slice the repair lane won was then replaced by the
 consolidation,
 so the contest verdict and what would ship name different stages.
Asking who won and asking what would ship are different questions,
 which is the whole reason this reader exists.

## The consumers ask both questions now, and measuring stopped one wrong port

Task `#166` item 2 told me to route the settled rendering audit and the damage
draw through `wouldShipTextPerSlice`.
Measuring both consumers first refuted the instruction,
and the reason is the same in each case:
33 of the 47 settled artifacts carry `laneSelection: pending-human-decision` and
no consolidation at all,
so a would-ship reading on them falls all the way back to the archive.

For the audit,
that would have put the archive's own English under audit at 227 of 271
subjects,
which is exactly the case `auditsArchiveText` exists to separate out,
since the instrument was built for output that has no BEFORE text.
For the damage draw,
the failure has the opposite shape and is worse:
the population does not get polluted,
it vanishes.
A would-ship-scoped builder keeps 30 of 378 regions and deletes every region
from those 33 artifacts,
because where the archive stands nothing replaced an incumbent and no region
exists at all.

The archive standing on an undecided artifact is not a decision.
It is the absence of one,
and it is pending `#175` with the owner.
`ArtifactLaneSelectionV2` says as much in its own docblock:
`pending-human-decision` means the contest has not run over this entry.

So both consumers keep their lane question and gain an annotation beside it,
which is what this task's decision 1 said in the first place.
`SettledPageRelation` in
`package/module/translation-repair/src/corpus-run/rendering-audit-settled-relation.ts` (moved on 2026-10-18 to
`package/translation-repair/probe/src/corpus-run/rendering-audit-settled-relation.ts`)
carries four recorded kinds plus `unrecorded`:
`undecided`,
`survives`,
`displaced` with the decider that displaced it,
and `nothing-would-ship` with the reason nothing stands.

The decider rides by reference rather than being spelled into member names like
`displaced-by-consolidation`.
Spelling them would duplicate `WouldShipDecider`'s list,
so a decider added later would silently produce no new relation,
which is the drift class `#170` measured twice in this same family.

### The branch order is the whole design, and the first guard for it was vacuous

Undecided is answered before any text is compared.
Reversing that reports a displacement on artifacts where no stage displaced
anything.

The first attempt to show that guard failing showed nothing,
because the fixture's texts differed and the branch below also answers
undecided.
Only a fixture where the archive wording matches the lane wording separates the
two orders,
and there the broken order answers `survives`:
it asserts a document would carry that wording on an entry no stage has decided.
That case is now pinned,
and it is the only one that fails under the break.

### Numbers, pre-registered before the field existed and reproduced through the built artifact

Audit subjects,
all 271:
undecided 227,
displaced by the consolidation 20,
survives 10,
displaced by the contest 10,
displaced by the archive 4.
On the 14 decided artifacts that is 34 displaced of 44 subjects,
which supersedes the earlier "5 of 6" sizing taken from a hand count of
`Acheron` and `Weideriche_`.

Damage regions,
all 378,
read through `collectShippedRegionsV2` rather than a reimplementation of it:
undecided 306,
displaced by the consolidation 38,
displaced by the contest 16,
survives 10,
displaced by the archive 8.

One number must not be quoted alone.
368 of 378 regions disagree with the would-ship text,
but a region exists only where a lane replaced the incumbent,
and on an undecided artifact the would-ship text is the incumbent,
so all 306 of those differ by construction.
The bounded figure is 62 of 72 on the decided artifacts.

### What the report prints, and what the tally counts

No printed line claimed page scope before this,
so nothing had to be corrected there.
`THE TWO HALVES, READ APART` prints `ARCHIVE text` and `FRESH text`,
both lane-scoped and both still accurate.
The annotation goes in as a new line instead.
It counts claims beside subjects,
because a displaced subject nobody claimed anything about cost the run a call
and nothing else,
while a displaced subject carrying claims means the instrument reported defects
in wording no reader of a document would meet.

`settled-tally.ts` gains `pageChanged` and `pageSilent` beside `repairChanged`
and `translateChanged`.
Zero is the honest answer on an undecided entry,
and it is meant to be read beside `selection=pending-human-decision` on the same
line:
two lanes proposed changes and,
as things stand,
a document would carry none of them.

### Two defects found on the way, both of this task's own class

The tally fixture omitted `consolidation`,
which the contract requires and states an absence for,
and an `as unknown as` cast hid the omission.
The first reader to reach for the field got `undefined`.

The annotation also pushed `rendering-audit-settled-input.ts` one line over its
budget.
Nothing was disabled and nothing was reformatted to fit:
`rendering-audit-settled-subject.ts` now holds the subject types,
the archive delivery constant,
`identityOf` and `subjectsOf`,
while the input module keeps the archive walk,
the provenance check and the reading.

## The last owed item asked for a refusal that was already there

Item 4 of `#166` prescribed a refusal:
treat a whole-run `shippedRecords` of zero as a stop rather than a printed zero.
Measuring before building refuted it,
and the refutation has three separate legs.

### The pointed-at-nothing refusal exists and fires

`EmptyPoolError` is raised from `selectEligible` in
`package/module/translation-repair/src/corpus-run/artifact-eligible.ts` (moved on 2026-10-18 to
`package/translation-repair/artifact-pool/src/corpus-run/artifact-eligible.ts`),
reached through `resolvePool`,
which all four pool consumers call:
`score-probe.ts`,
`draw-sample.ts`,
`damage-sample.ts` and
`attribution-read.ts`.
Its message already states the rationale item 4 was going to add:
a rate over zero entries is
"a denominator quietly shrunk,
here all the way to nothing,
while the number above it still renders".

Proved at the user boundary rather than inferred.
Pointing the built `score-probe.mjs` at a throwaway run holding an empty artifacts directory
printed the `SOURCE` line naming the directory and then raised,
exiting 1:

```text
EmptyPoolError: No entry has settled yet, so there is nothing to pool.
```

### A surviving zero is a legitimate run, not a misread

Item 1 of this task closed every path by which a misread reaches the printer.
`readArtifactProbe` reads the records through `parseSettledArtifactV2`,
so an absent list,
an absent field,
and a disposition the pipeline never writes each raise
rather than shrinking the count.
The reader's own docblock records why:
both lists "used to be read off the artifact root,
where version 2 writes neither,
and both answered their absence with an empty list".

What that leaves is a run whose repair lane genuinely accepted nothing.
The population holds one artifact with repair status `unchanged`,
so a single-entry run over it is a healthy run with a zero denominator.
A refusal there would reject it.

### Nothing divides by the count, which changes what the note must say

The working justification had been that every rate below the count loses its denominator.
Reading `reportProbeTelemetry` refuted that:
every figure it prints is a raw count,
and there is no rate in the report at all.
The real hazard is narrower and worse to state loosely.
With no repair-lane record,
`repairUnprobedRecords`,
`regions` and every `*Introduced` and `CLAIMS` figure are zero by construction,
so the reader meets seven zeros and reads `majorityIntroduced=0`
as the probe having cleared the run.

### So it is a note, and the file already had the idiom

`probe-telemetry-report.ts` met this hazard once before,
for `rewrittenSlices=0`,
and answered it with a note rather than a refusal
because a zero there is ordinary on artifacts written before the lane was audited.
The new note follows it,
and points at `editorOffered` on the `ROSTER` line,
which is the figure that separates the two readings:
above zero the editor was asked and nothing it produced shipped,
at zero the critics filed nothing to repair.

Verified at the user boundary through the built CLI,
on a throwaway run under `~/temp/agent` holding one real artifact
whose only shipped record was moved to `not-selected`:

```text
PROBE entries=1 repairShippedRecords=0 repairUnprobedRecords=0 regions=0 majorityIntroduced=0 ...
NOTE repairShippedRecords=0 means no artifact read here carries a repair-lane record ...
ROSTER editorOffered=1 editorDegraded=0 editorSilent=0 ...
```

`editorOffered=1` beside the zero is the discriminating case working:
the editor was asked at one slice and nothing it produced shipped.

The note is covered by three cases in
`probe-telemetry-report.unit.test.ts`,
which is the file's first test.
Per GFP the note was removed,
rebuilt and rerun:
the two presence cases fail and the silence case correctly keeps passing.

## Two counters were named for a page they never reached

The remaining half of item 3,
renaming consumers that imply a page-level claim,
had no recorded disposition.
`shippedRecords` and `unprobedRecords` are that class,
and the rename landed in `693b0c58a`.

Both count repair-lane issue records whose `repairDisposition` is `shipped`,
which says that lane applied a replacement to the document it built.
The contest and the consolidation both sit downstream of it.
Under the bare word,
`PROBE shippedRecords=271` reads as a count of wording a page carries,
and this task's own measurement says the contest had not run on 33 of 47 artifacts,
so what a page would carry there is the archive's wording rather than either lane's.
They are now `repairShippedRecords` and `repairUnprobedRecords`.

Adding one export to `corpus-barrel.ts` pushed it one line over its budget.
Per MXL nothing was disabled:
`generation-barrel.ts` now holds the pooling and generation-identity exports,
split at the seam the corpus barrel's own header names,
by audience.

## Asking what a document would carry, before writing the artifact that claims it

`settledTallyLine` gained a call to `wouldShipTextFor` per comparison row,
which made `UnansweredContestSliceError` reachable on the production pass path.
It was computed AFTER `writeFileAtomic`,
so a document whose lanes differ at a slice the contest names nowhere
would have its complete artifact written
and then be reported `status=ERROR` by the entry catch below.
Every later reader would find a settled file the pass reported as failed.

Asking first makes the refusal truthful,
and it is the correct refusal rather than the convenient one:
a contest that cannot account for a slice it was obliged to decide has not settled the document.
The stage caches still hold every answer,
so a re-run reproduces the contradiction rather than losing it.
Landed in `462ecd70d`.
