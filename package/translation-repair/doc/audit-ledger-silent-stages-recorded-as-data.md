# Audit ledger: a stage that heard nobody, recorded as data

A stage that fell short of its quorum,
how a settlement says so,
and the gates that decide from that fact whether a record is kept and resumed.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B820: a panel that heard nobody on five packets was cached as examined, because its finding no longer began with the text the gate matched

Status:
fixed on 2026-10-22 (UTC),
cache version `SLICE_CACHE_VERSION` 43 to 44.
The brief's D3:
red `ca5419874c`,
fix `c265e8df95`.
The review's S1 and N3:
red `7f53745063`,
fix `e6544017ff`,
with `068d523090`,
`b423e15be8` and `9d6095ae7e`.
Found by the read of the XingZ6018 run (build `8864316eb2`);
the first reading of its cause was corrected by the second reader.

What the run showed:
slice 85 was cached with 12 issues,
every one `needs-human` (6 minor,
5 major,
1 critical),
and the log line `chunk 85: nothing to edit, unchanged`.
Its critic stage heard 4 of 6 voices.
Its panel ran 20 rounds,
19 of them `0/4 heard` and one `0/5 heard`,
and every one closed with `no quorum (0 of 3 needed)`.
No `NOT cached` warning printed for that slice,
though it printed for slices 82,
83,
84,
86 and 87.
The stored record (`slice-cache/XingZ60/3231001d79....json`) has `sliceIndex` 85,
`changed` false,
`heardCritics` 4 and 37 findings.
The critical issue is one critic's claim and the panel never ruled on it,
so it is an unadjudicated claim,
not a confirmed mistranslation.

The cause:
the first reading said the panel stage emits no quorum-unmet finding.
It does.
`adjudication/src/panel-stage.ts` maps every packet finding to `panel-packet (<clusterId>): <finding>`,
so the record holds five findings that contain `stage-quorum-unmet (panel 0/6)` (twice) and `(panel 0/5)` (three times)
and none that begins with it.
The gate (`repair/src/repair-cache-gate.ts`) refused a settlement only for `heardCritics` of 0
or for a finding for which `silentStagesOf` held,
and that function was `finding.startsWith('stage-quorum-unmet (')`.
It saw an empty list,
and `storedOutcome` (`repair-slice-settle.ts`) stored the slice.
The select,
checker and refiner stages locate nothing,
which is why their silent slices had been refused.
A later run on that cache directory would resume slice 85 as examined;
that is read from the code path and was not run,
because the two re-entries of the run stopped in preparation.

The fix:
the fact travels as data.
`ChunkRepairOutcome` carries `silentStages`,
required on every outcome,
and the panel result names every packet it heard fewer voices than quorum on.
`silentStageLabel` spells a label such as `panel 0/6 (cluster/abc)` once
and `silentStagesOfRecord` (`stage/src/stage-silence.ts`) reads the field first,
then the findings no layer relocated.
The writers of the outcome were checked one by one:
`unchangedAfterPanel` in `repair-chunk.ts` at the three exits past the panel and at the final return,
`repair-unchanged-outcome.ts` with an empty list for the exits before the panel
(a critic shortfall is still a plain quorum-unmet finding there),
`repair-not-applicable.ts`,
and `front-matter-repair.ts` by spread.
The cache gate reads the field,
and the slice-cache reader (`slice-cache-store.ts`) refuses a record that lacks it or names a silent stage,
so the records already written are redone.
`stage-quorum.ts` spells its shortfall with `silentStageLabel` (note N3 of the review),
so the label and the data it names cannot drift.

Cache versions:
`SLICE_CACHE_VERSION` moves 43 to 44 (records stand at 43,
written by XingZ6018):
the key hashes the version,
the run shape and the texts,
not the rule that decides what may be stored.
`REFINE_CACHE_VERSION` stays at 14:
its key and its question are unchanged,
and the reader refuses a refinement whose embedded outcome lacks the field.
The account in the TSDoc of `repair/src/repair-slice-key.ts` also names the versions that stay,
as the review's M28 check asked:
lane contest 16,
consolidate 32,
consistency 7,
page title 6,
translate slice 25,
pairing 3,
and the artifact schema.
The commit of B818 moves none,
because it changes run control and not what a settlement may be cached as.

Review S1:
the refine phase's write gate (`repair/src/refine-phase-slice.ts`) asked `everyStageHeard` of the refinement's own findings,
and the reader (`isResumableRefinement`) also refused a record whose embedded outcome named a silent panel.
For a slice whose panel heard nobody the repair slice is not cached,
the refine phase still ran on that outcome and wrote its file,
and the attempt queue counts every slice-named file as progress.
By the review's reading of the code (not run),
the entry would be queued for a REATTEMPT that buys the repair slice and the refinement again,
rewrites the same file name and ends STALLED:
one whole-entry attempt wasted for each affected entry,
with the "NOT cached" warning never firing.
The fix gives both gates one predicate in the stage package,
`refinementIsResumable({ settlement })` over `silentStagesOfSettlement`,
and removes `everyStageHeard`,
which nothing reaches any more.
The warning lists both sources.
The red,
`7f53745063`,
asserts that a refinement whose outcome names a silent panel writes no refine file.

Guard:
the cases of the two commits:
the cache gate refuses a settlement whose `silentStages` is not empty,
a deaf panel leaves one,
the slice-cache reader refuses a settlement shaped like slice 85,
and `silentStageLabel` and `silentStagesOfRecord` read the field first.
The red of `ca5419874c` failed because none of the new exports or the field existed.

Recurrence:
`mistake-prevention-stored-files-and-cached-decisions.md`,
"Facts a cache gate reads":
a fact a gate depends on travels as data,
and a write gate and a read gate ask one predicate.
Earlier members of the family:
B141 and B160 in `audit-ledger-which-seats-a-quorum-counts.md`,
which gave the recheck and the probe the same reader.
