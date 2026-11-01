# The third rendering: the settlement terminals split and the artifact keys renamed

Why the keep-standing terminal could not be counted,
what it merged,
and the queue order for the schema change that was later superseded;
the pair index rename and the verdict key that meant two things;
the three-state terminal split with its legacy reader,
and the archive read verified at the real boundary.

Part of [The third rendering](the-third-rendering.md),
which lists every file of this record.

## The settlement that cannot be counted, and the two opposite verdicts it hides

`#165` asked for a count before deciding whether the consolidation should buy a
 second judging the way the translate lane does.
Taking that count found something that outranks it.

`consolidate-settle.ts` assigns `terminal: 'slate-kept-standing'` on a single
 condition,
`decided.origin !== 'fresh'`.
That condition is true in two opposite cases.
It is true when the panel could not agree,
or rejected every candidate,
and the slice keeps its standing text because nobody chose anything.
It is also true when the panel deliberately read the standing text as the best
 rendering on the slate and picked it.
One says the judges failed.
The other says the judges succeeded and the archive won.
The record calls both the same thing.

Where the distinguishing fact lives,
measured rather than assumed:

-   NOT in the artifact.
    A consolidation slice carries exactly these fields,
    read off `Acheron.json` from the run in flight:
    `chunkIndex`,
    `terminal`,
    `shipped`,
    `rewrapped`,
    `demoted`,
    `verdicts`,
    `gate`.
    There is no `decided` field,
    so `decision` and `origin` never reach it.
-   In the cache record,
    which carries `decided` with `origin` and `decision` among its keys.
    That is where it cannot be read for this question,
    because `consolidationWorthResuming` deliberately refuses to persist a
    `slate-kept-standing` settlement carrying one of the two retried declines.
    The mitigation and the measurement want opposite things from one record.
-   In the log,
    but only as free text.
    The decline path logs the judge's own reason and `keeping the incumbent`.
    The enum name is computed one line earlier in `translate-judge.ts`,
    and it is used only in the `TranslateAbsenceError` thrown when the incumbent
    is absent,
    so on the keep path it is never written anywhere that outlives the run.

So the count `#165` asked for is not available from either durable output.
The first action is to make the decision observable,
not to count.

### What the log does show, so far

Run 1 had settled one entry and was working a second when this was read.
Across both,
`translate stage:` appears 14 times:
5 under `consolidateDocument` and 9 under `translateDocument`,
with none unattributed,
which is the check that the filter did not silently drop lines.

Of the 5 consolidation judgings,
4 were fresh wins,
and 1 declined for a winner short of the minimum vote weight.
Acheron contributed 3 judgings and the entry in flight contributed 2.
This is far too small to answer whether a retry is cheap.
It is recorded only so the finished run can be compared against it.

### Options, ranked

-   **Record `decision` and `origin` on the artifact's consolidation slice.**
    Durable,
    measurable across every future run without a log,
    and it separates the two verdicts for every consumer rather than only for
    this count.
    Against it:
    an artifact schema change,
    so `parseSettledArtifactV2` and its refusal tests move with it.
-   **Split the terminal into two names,
    one for a deliberate keep and one for a decline.**
    Makes the distinction impossible to miss,
    adds no field,
    and reads correctly at a glance.
    Against it:
    every consumer switching on the terminal moves with it,
    and it still does not say WHICH decline it was.
-   **Log the enum on the keep path.**
    One line,
    and no schema change.
    Against it:
    it leaves the fact in a log no consumer reads and no run keeps,
    and does nothing about the conflation itself.

Ranking:
the artifact field,
then the terminal split,
then the log line.
The field beats the split because it leaves the terminal vocabulary alone while
 adding the fact that is actually missing,
and because it carries which decline it was rather than only that there was one.
The split beats the log line because it puts the distinction in the durable
 record where consumers read it,
while a log file is deleted with the run.

This is not only a measurement gap.
A consumer deciding what to show a reader cannot currently tell a passage the
 judges endorsed from one they could not rule on,
which puts it in `#166`'s scope as well as `#165`'s.

## What `slate-kept-standing` actually merges

`#165` said the terminal merges a decline with a deliberate keep.
Read at the branch that assigns it,
it merges three states,
and the sole-candidate case is the one that most distorts a count.

### The condition and what reaches it

`consolidate-settle.ts:379` assigns the terminal on one test,
`decided.origin !== 'fresh'`,
under a comment reading that the judges choosing the incumbent ends it.
That comment describes one of the three cases the condition admits.

Judges heard a slate and chose the archive.
`decision` is `judged` and `origin` is `incumbent`.
This is a real endorsement of the standing wording,
and the only case the comment describes.

The slate carried nothing but the incumbent.
`decision` is `sole-candidate`,
`judgesAvailable` is 0 and `ballots` is 0.
No judge spoke,
so nothing was endorsed.
The capture holds this case live on `Zha_Ke`.

Judges heard a slate and refused it.
`decision` is `declined-indecision` or `declined-rejection`,
which `translate-judge.ts:439` assigns when the panel could not agree or rejected
 what it saw.
`Acheron` slice 0 took this path,
its winner short of the minimum vote weight.

### Why the count cannot be taken

A tally of `slate-kept-standing` adds an endorsement of the archive to a slice
 where no judge spoke and a slice where the judges refused to speak.
Those answer opposite questions about roster quality.
A roster that endorses the archive is working;
a roster that cannot agree is not;
a slate with one candidate measures the producing stage rather than the judges.

`translate-judge.ts:442` already carries the reasoning,
in a comment saying a decline is not a keep where there is nothing to keep.
The judge separates them.
The settlement collapses them one call later.

### The ranking changes

`#165` previously ranked an artifact field over a terminal split.
Reading the surrounding code reverses that.

`consolidate-settle.ts:424` already splits the gate side of the same question
 into three named terminals,
`consolidated`,
`wrap-erased-difference` and `gate-kept-standing`,
under a comment saying they are kept apart because they answer different
 questions about the roster.
Splitting the slate side the same way is the design this file already states,
not a new mechanism.
An artifact field would record the discriminator beside a terminal that still
 reports one state for three.

The split is three terminals in place of one,
distinguished by the `decision` the judge already produced.
`slate-kept-standing` stays for the endorsement,
so the name keeps meaning what it says.
Nothing forbids the new spellings:
`forbidden-strings.append.local.txt` carries no entry for `slate`,
`declined` or `sole`.

Cost counted rather than guessed:
`slate-kept-standing` occurs in 7 files,
4 under `src` and 3 tests,
and the artifact schema and its version 2 reader move with it.

Queued behind the source freeze.

## The order the frozen queue lands in

Five tasks are queued behind the source freeze,
and three of them change the same artifact contract.
Landing them one at a time would migrate the schema three times,
and would write `#166`'s shipped-text reader against a terminal vocabulary
 `#165` is about to change.

The order,
decided rather than left to whichever is picked up first.

One coordinated schema change lands first,
bundling `#165` and `#170` and any field rename `#166`'s port needs.
`#165` splits `slate-kept-standing` into three terminals.
`#170` renames `comparison[].verdict`,
which collides with the contest field of the same name.
Both touch the version 2 reader and its guards,
so they share one migration and one set of absence rules for older artifacts.

`#166`'s port lands second,
against the vocabulary that change leaves behind.
It carries the shipped-text-per-slice reader,
the renaming of every consumer implying a page-level claim,
and the zero-denominator refusal.
Writing it first would bind it to terminals that no longer exist.

`#167` and `#168` are independent of all of that and land in any order.
`#167` bounds the semantic wrap away from line-structured slices.
`#168` lowers the transport ceiling to 7 MiB.
Neither reads the artifact contract.

Nothing here starts before run 2 finishes,
which is `#138`'s deliverable and the only thing that lifts the freeze.

## The `pairIndex` rename is not the comparability change it was deferred as

`#98` left `AlignmentFinding.pairIndex` documented rather than renamed,
and `chunk-document.ts:89` records the reason:
the scorecard renders it into a finding string that 56 settled artifacts share,
so renaming it would be a comparability change.

Both halves of that are wrong,
and the real hazard is a third thing neither names.

### No scorecard renders it

The render is `document-preparation.ts:206`,
into `prepared.alignmentFindings`,
which `document-preparation.ts:109` declares as `readonly string[]`.
Neither `scorecard.ts` nor `repair-scorecard.ts` mentions `pairIndex`.

### Renaming the field cannot change any string

The template writes the literal word `pair` and interpolates the value:

```ts
// package/module/translation-repair/src/document-preparation.ts
return `alignment ${finding.kind} (pair ${String(finding.pairIndex,)}: ${finding.detail})`;
```

Moved on 2026-10-18 from `package/module/translation-repair/src/document-preparation.ts`
to `package/translation-repair/adjudication/src/document-preparation.ts`.

`alignment structure-mismatch (pair 0: ...)` is byte-identical whatever the
 TypeScript field is called.
The field name never reaches an artifact.
So the rename is a rename,
and it is free.

### What is actually a comparability change is the word `pair`

Two producers write that word over two different numbering spaces.

`chunk-document.ts:385` sets `pairIndex` to `step.sourceIndex` or
 `step.targetIndex` for a refusal,
which is an index on the refused chunk's own side.
`chunk-document.ts:290` sets it to zero for a whole-document observation.
Both render as `structure-mismatch (pair N: ...)`.

`document-preparation.ts:300` renders `target-unclaimed (pair N: ...)` from the
 loop at `document-preparation.ts:242`,
where N is a real pair index.

So a reader meeting `(pair 0: ...)` cannot tell which numbering space the zero
 lives in,
and the two families are told apart only by the kind word beside it.
Changing that word is what the settled artifacts would notice.

### How many artifacts carry one

5 of 56 version 1 artifacts in the settled payload carry an alignment string,
which reproduces the count `#71` recorded.
The heads are `structure-mismatch (pair 0` five times,
`sections-merged (pair 0` three times and `sections-merged (pair 1` once.
1 of 25 version 2 artifacts carries one,
`target-unclaimed (pair 0`.

A first probe reported 0 of 164 for the version 1 side,
because it looked for a key whose name mentions alignment and these are prose
 strings inside a findings array.
No key in any of the 56 mentions alignment at all.

### What this changes

The rename leaves `#98` and joins `#170`,
which is the same defect class:
one name over two meanings,
caught only when a reader takes the wrong one.
The field rename is free and needs no migration.
The rendered word is the part that costs comparability,
and it is worth paying,
because a refusal reporting a side index as a pair index is the reading that
 sends someone to the wrong section.

Queued behind the source freeze.

## The slate terminal now names three states, and a settlement keeps its findings

Landed 2026-08-22 across `0898c8d17`,
`8d3da278d` and `e2e8b1216`,
the first item of the coordinated schema change,
after `#170`'s two schema-free items landed as `0b64add7f`.

### The retired name is not reused

The earlier plan kept `slate-kept-standing` for the endorsed case,
on the reasoning that the surviving name would then mean what it says.
A census taken before writing any code overturned that.

Across every settled artifact on this machine,
47 scanned and 14 carrying a consolidation over 42 slices,
the terminal appears on 11 rows in `Acheron`,
`Zha_Ke`,
`gaoyanger` and `lintong`.
Every one of those rows carries the identical key set,
`chunkIndex`,
`demoted`,
`gate`,
`rewrapped`,
`shipped`,
`terminal` and `verdicts`,
with no record of the judged round.

So no legacy row can be told apart from any other.
Had the endorsed case kept the spelling,
a census over mixed artifacts could not have separated
a post-split endorsement from a pre-split row meaning one of three,
except by keying on which other fields a row happens to carry,
which is implicit versioning by another name.

All three states get new names,
and `slate-kept-standing` survives only in the artifact reader,
as `ArtifactConsolidationTerminal`,
whose docblock says what it cannot tell apart.
The 11 rows are not backfilled from logs or captures.
They are unrecoverable by construction,
and inference presented as data is worse than a named absence.

### Legacy tolerance is confined to the reader, and that was verified

Artifacts outlive pipeline digests.
Caches do not.

`openNamespacedCache` returns an empty resume map and discards the namespace
 whenever the generation stored beside it differs from the running digest,
and `openConsolidateCache` is handed the pipeline digest as that generation.
So no consolidate cache entry written before this change can ever be resumed,
which is why the producer union and the cache guard carry only the new names
while the reader carries the retired one as well.

### What the split confirmed on the first run

The test named `KEEPS THE STANDING TEXT WHEN EVERY JUDGE DECLINES`
has always described a declining panel,
and had always asserted the name a panel ENDORSING the archive would carry.
Under the split it settles as `slate-declined-standing`.

That is the defect the task predicted,
found in the repository's own test suite rather than in a run,
and it is the clearest evidence that one name for three states
was being read as the wrong one of the three.

### The decision read survives, against the plan to retire it

Splitting the terminal was expected to make
`consolidationWorthResuming` a pure terminal read.
It does not,
and the reason is worth recording so a later session does not retry it.

`slate-declined-standing` covers both the two declines a second panel might
 change and the settled `no-candidate-backed` recorded after that second panel
 has already run.
Those get opposite answers,
so the terminal alone cannot decide them.
The other four terminals now decide by name,
through `SETTLED_WITHOUT_A_GATE`,
and the persistence policy is unchanged in every case.

### A settlement carries its own findings

Folded in from the `#174` survey,
because it changes the same type
and doing it separately would rewrite that type and its reader twice.

`ConsolidationSettlement` carried no findings field,
so the produce half's findings reached a reader only where a judged round or a
 gate round happened to run.
The two terminals that end before either,
`no-standing-text` and `incumbent-only`,
dropped them on EVERY run rather than only on a resume.

Those are exactly the terminals whose findings explain them.
`produced.findings` is gather findings plus repair findings,
which is where voice loss and transport failure are recorded,
so the account of why a slate had nothing valid on it was being lost
at the terminal that reports a slate had nothing valid on it.

Per-proposal validity findings were never affected,
because `ProposalVerdict` carries its own,
so the loss was bounded to the produce half.
No lint signal existed,
because `producedFindings` is used on one branch
and so is not an unused parameter.

Three tests pin it.
Two show the loss,
and both fail against the code before this change,
where `settled.findings` reads `undefined`.
The third pins that a path reaching both rounds reports each finding ONCE,
which is the trap in fixing the first two:
the judged round is already handed the produce half's list,
so a settlement appending it again would report one lost voice twice.

## One key name stopped meaning two things

Landed 2026-08-22 as `0b64add7f` for the two schema-free items
and `6cf4ed287` for the artifact field,
completing the coordinated schema change begun by the terminal split.

### An alignment observation now names the numbering it counts in

`AlignmentFinding.pairIndex` was documented as not always a pair index,
and it was not.
A whole-document observation borrowed zero for a number it does not have,
and a refusal carried the index of the unpaired chunk on its OWN side,
whose numbering need not line up with the pairs a run produced.
All three rendered as `(pair N: ...)`,
beside a `target-unclaimed (pair N: ...)` carrying a real pair index.

The field is now `attachedTo`,
a union over the three numbering spaces,
so the whole-document case has no index at all rather than a borrowed zero.
A refusal now reads
`alignment structure-mismatch (source section 1: ...)`,
measured on live output before and after:
the same fixture rendered `(pair 1: ...)` under the previous code,
where the 1 was a source-side index.

Both blockers the task recorded against this rename were checked and refuted
 before it was made.
No scorecard renders the field,
and renaming a TypeScript field could not have changed any stored string,
because the template interpolates the value and writes the word itself.

### The comparison row's verdict is now a lane relation

`comparison[].verdict` and `laneSelection.slices[].verdict`
sat at sibling paths of one artifact under one bare key.
The first says WHICH LANES CHANGED a slice and names no winner.
The second says WHO WON,
which is the question about shipping.

Reading the wrong one neither throws nor surfaces as an absence:
taking the string form for the object form yields `undefined` for every row,
which reads as a uniform,
plausible answer.
The measured cost was a probe reporting 142 of 142 slices as audit subjects,
caught only because it printed a distribution as a control.

The comparison field becomes `laneRelation`.
The contest field keeps `verdict`,
because it is the one a reader asking about shipping wants.

### The reader takes either spelling, and that was verified on the archive

Refusing the retired key would have made every settled artifact unreadable
 to buy a name,
so the reader accepts either,
and refuses a row carrying both rather than resolving it.

VERIFIED AT THE BOUNDARY rather than by fixture.
Every settled artifact on this machine was parsed through the current reader:
47 files,
all 47 carrying a comparison,
271 rows,
and all 271 spelled under the retired key.
Zero rows were refused.

The relations read back as
138 `both-differ`,
56 `translate-only`,
36 `repair-only`,
36 `archive-stands`
and 5 `both-agree`,
which is the positive control the discipline this task recorded calls for:
a reader returning one bucket for every row would be the failure signature,
and this distribution is not that.

### What this batch did not touch

A third field spelled `verdict` sits at
`comparison[].decisionComparison.verdict`,
holding `same` or `different`.
It is nested rather than a sibling,
so a reader reaches it through a name that already says which question it
 answers,
and it was left alone deliberately rather than missed.

## The archive read was one level below the boundary it claimed

The rename check read `parseComparisonRowV2` per row.
That covers the comparison half and nothing else.
The terminal half of the same batch had only its declaration:
`slate-kept-standing` sat in the reader's allowed names with a comment saying
 the archive stays readable,
and no test anywhere carried the string,
because the split had moved every fixture onto the three new names.

`parseSettledArtifactV2` is the real boundary.
It runs `assertRecordedComparisonMatches` over the whole artifact and reads the
 consolidation beside it,
so it is the only entry point that exercises a retired terminal at all.
Reading all of it through that function reports
47 version 2 artifacts,
271 comparison rows,
271 of them under the retired relation spelling,
11 consolidation rows under the retired terminal across 9 artifact files
 spanning 4 entries,
and zero refusals.

The 11 rows and the 4 entries reconcile with the census this batch recorded
 earlier.
The distinct-slice count is smaller than 11,
because `vub-run1-20260821` keeps a second copy of its artifacts and one entry
 appears only in that copy.

### The probe reproduced the defect it was checking

The first run of that read reported zero legacy terminal rows while `rg` found
 the string in four files.
The probe asked for `artifact.consolidate.slices`.
The artifact writes `artifact.consolidation.slices`.
A wrong key on a JSON object returns `undefined`,
`?? []` turns that into an empty array,
and the count comes back zero with nothing raised anywhere.

That is the same failure this batch renamed two fields to prevent,
committed in the instrument built to measure it,
and it was caught only because a second measurement disagreed.
A count that reads as an answer is worth nothing without something independent
 saying the same number.
