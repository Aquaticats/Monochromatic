# Translation repair history 2026-08-21 to 2026-08-23: giving the naturalness lane's auditor its neighbours

The naturalness lane's introduced-defect probe,
which had no window,
how the window was threaded and which index it addresses by,
an invariant about chunk indices that nothing asserts,
the cache key change,
and the gate.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## 2026-08-23: the naturalness lane's auditor was blind to neighbours, and now is not

`#68`'s last open item,
closed.

`refine-slice-settle.ts` called `runIntroducedDefectProbe` with no window,
and no window was in scope anywhere in that file.
The accuracy lane's probe has had one since `#107`.
So one lane's auditor reasoned about a slice alone
while the other's reasoned about a slice in context,
and any difference between their findings could have been the lanes differing
or the windows differing,
with no reading of the numbers able to separate the two.

WHY IT MATTERS MOST FOR THIS LANE SPECIFICALLY.
The naturalness lane rewrites for fluency.
The commonest fluent rewrite of a paragraph
that repeats what the paragraph next door already said
is to drop the repetition.
Judged alone that is a deletion.
Judged with the neighbour visible it is the redundancy it was.
So the lane whose characteristic edit needs the neighbour most
was the one that never had it.

### What was threaded, and the one judgement call in it

The chain is `repair-translation.ts:541` to `refineSettledSlices`
to `runRefinePhase` to `settleRefinedSlice`.
The phase already receives `slices`,
so it computes the window itself with `neighbouringSource` and `neighbouringIncumbent`,
the same helpers the accuracy driver uses,
rather than carrying it on `RefinedSliceSettlement`,
which is the cached type and is better kept lean.

THE JUDGEMENT CALL WAS WHICH INDEX TO ADDRESS BY.
`neighbouringSource` throws on a position the entry does not have,
and the phase loop already tolerates an outcome with no prepared slice behind it:
`sourceText` and `incumbentText` both fall back rather than fail.
A window that threw where those fall back
would turn a tolerated shape into a crash.
So the window is computed only when `prepared` is present,
which means the lookup found an element,
which means the index is an integer inside the array
and neither helper can throw.

It addresses by the stamped `chunkIndex`,
like everything else in that loop.
That agrees with the position
because `document-preparation.ts:272` stamps `baseIndex: slices.length`,
a running counter over emitted slices,
so a prepared slice's stamp IS its position.

### An invariant nothing asserts, found while doing it

`refine-phase.ts` reaches its slice by `slices[outcome.chunkIndex]`,
array position indexed by a stamped value.
`repair-refine-step.ts` in the same feature builds a Map from `chunkIndex` to text
and looks up through it,
which is correct whether or not the two agree.
Two files of one feature therefore disagree about whether the identity is a position or a stamp.

Production is safe today,
measured rather than assumed:
`document-preparation.ts:272` and `seed-detection.ts:118`
both pass a running counter as `baseIndex`,
and `slice-pair.ts:288` re-indexes the one path that used to leak a section index.
`corpus-run/probe-relabel-case.ts:197` passes the PAIR index instead,
which is the shape `seed-detection.ts:106` records as having been silently wrong before.
That one is a probe rather than a production path.

Nothing asserts the invariant anywhere,
and `#99` records that `chunkIndex` does not carry one meaning across the codebase.
Recorded on `#94`,
which owns lane index contracts.

### Cache

`refineSliceKey` now covers both window sides,
labelled,
so a source-only and an incumbent-only window carrying identical text cannot share a key.
That is `#126` exactly,
which found the unlabelled version of this hazard at the accuracy key.
Absent and empty key alike,
because `introduced-defect-wire.ts:418` renders no nearby block for either,
so a document-edge slice is asked what a caller without the parameter asks.
`REFINE_CACHE_VERSION` moves to 2
for the documented reason that it moves when the probe's question moves.

DELIBERATE COST:
the next resume rebuys refinement for every cached entry.

### Gate

Written first and proven able to fail,
per GFP.
`refine-window-threading.unit.test.ts` run against the pre-fix build
failed with `expected '' to include 'ZQPREVSRC'`,
an empty window half,
which is the defect itself;
its lone-slice control passed in the same run,
so the assertion discriminates rather than always failing.
Both pass after the rebuild.

Suite 532 PASS / 0 FAIL / exit 0,
up one suite from the 531 baseline for the new file.
`lint` 0 warnings 0 errors,
`lint:types` exit 0.

Two lint findings during the work were both mine and both fixed:
an inserted block had landed between the key's TSDoc and its declaration,
and the test accumulated offsets with a spreading `reduce`.
The offsets are now found by searching the assembled document for each unique paragraph,
which cannot disagree with the text it describes the way a parallel running total can.
