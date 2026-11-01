# The heading aligner plan: unwired modules, the unpaired sections and the size of the change

Part of [Two features were built and never wired, and what to do about the aligner](wire-the-heading-aligner.md),
which lists every file of this record.

## The same check, run systematically

The aligner was found by asking whether a built feature fires.
Asking it of
 every module rather than one turns up two more cases.

Method:
walk imports from the real entry points,
`repair-translation.ts` and
 the `corpus-run/` drivers,
and do NOT traverse the barrel files.
Barrels
 re-export without depending,
so a traversal that walks through them reports
 every module reachable,
which is the first answer this produced and it was
 worthless.
The control that makes the result meaningful is
 `align-sections-order.ts`,
which is independently known to be uncalled and
 must therefore appear.

```text
  155 source modules, 146 reachable, 8 unreachable

  align-sections-order.ts     imported by nothing but barrels
  heading-affinity.ts         imported only by align-sections-order.ts
  derivability-probe.ts       imported only by recall-barrel.ts
  derivability-wire.ts        imported by derivability-probe.ts and recall-barrel.ts
  probe-barrel.ts             re-export layer
  recall-barrel.ts            re-export layer
  refine-barrel.ts            re-export layer
  sheet-barrel.ts             re-export layer
```

Four are pure re-export layers and are fine.
The other four are two features.

### The derivability probe is also unwired

`runDerivabilityProbe` was added in `616264bdb`.
Its own header describes what
 it is for:
an ensemble audit of whether deleted sentences are fully derivable
 from the Chinese source,
whose only power is to EXCUSE a partial restoration,
 with the median deliberately rounded toward more derivable so the burden of
 proof sits on the excuse.

Nothing calls it.
`recall-benchmark.ts`,
the driver behind the
 `recall-benchmark` task,
imports its dependencies individually rather than
 through a barrel and never mentions derivability.
The restoration judge beside
 it IS reached;
the derivability probe is not.

The consequence is specific rather than cosmetic:
recall measurements charge a
 partial restoration against the pipeline even where the deleted sentence was
 not derivable from the source,
which is exactly the case this probe was built
 to excuse.
Any recall figure taken so far was computed without it.

Whether that is a defect or a deliberate pause is not something the code says.
It is raised here rather than answered.

### Why this shape of defect is invisible

All three features are exported from `index.ts`,
so a consumer outside the
 package could import them and nothing looks dead.
All three have passing unit
 tests,
so coverage does not fall.
Neither `tsc` nor the linter objects to an
 export with no internal consumer,
because that is the normal shape of a public
 API.

The cheap guard is the walk above,
rooted at the entry points and refusing to
 traverse barrels.
It takes a second to run and would have caught `110fc3909`
 the day it landed.

### The same check one level finer, which found nothing serious

Modules can fire while the values they produce go unread,
which is how the
 footnote graph hid.
Checking every field of the two central types for a reader
 outside its declaring file:

```text
  ChunkRepairOutcome, 16 fields: one with no reader
    nonTranslationContradicted

  RepairDocument, 6 fields: one with no reader
    documentHash
```

`nonTranslationContradicted` is a false alarm,
and worth writing down as one.
The value is stored by `repair-chunk.ts` and read by nothing,
but the FEATURE
 fires:
`chunk-critic-phase.ts` branches on `screening.contradicted` and
 `non-translation-evidence.ts` dismisses contradicted votes at the point the
 evidence is computed.
Only the persisted copy is unread,
which makes it
 provenance rather than a defect.

`documentHash` is genuinely dead.
`parseDocument` hashes the full document text
 on every call and nothing outside its own test ever reads the result.
Drift
 detection,
the obvious candidate purpose,
uses the per-span `nodeHash`
 instead.

Left in place,
on measurement rather than on caution.
Over a 33451-character
 entry,
five timed runs of a hundred iterations each:

```text
  parseDocument        median 13.077 ms   band 12.931 to 13.179
  hashContent alone    median  0.048 ms   band  0.048 to  0.049
```

The hash is 0.4% of a parse.
Removing it would change a published type for no
 measurable gain,
so it stays until something else touches that type.

So the finer-grained pass turned up no second aligner.
That is the useful
 outcome:
the unwired-MODULE check earns its keep,
and the unread-FIELD check
 did not,
at least on these two types.

## What the three unpaired sections actually are

`#70` blocks this proposal,
and it is stated as "an unpaired source section
 needs a destination".
That is abstract.
There are exactly three of them at this
 pin,
and they are not one problem:

```text
  XingZ60  ### 其十：锦心          915 chars, 12 body lines, opens with an
                                   HTML <details> disclosure block
  XingZ60  ## 致曾划过夜空的流星    1459 chars, 22 body lines, prose
  XIEPT2   (To-Do)                    7 chars, ZERO body lines
```

The `XIEPT2` one needs no decision.
It is a seven-character placeholder with no
 body,
sitting in the English where the Chinese has a section.
There is nothing
 to translate into it and nothing to lose by leaving it unpaired,
so it does
 not belong in the same question as the other two.

That leaves TWO genuinely untranslated sections,
2374 characters between them,
 both in ONE entry.
So `#70`'s blast radius is a single document's trailing
 pair,
not a corpus-wide policy.

The 915-character one is also the "915 characters against nothing" shape the
 handover cites as unrepairable.
Worth knowing that its body opens with an HTML
 disclosure block rather than prose,
so whatever policy is chosen has to survive
 markup and not only sentences.

None of this decides the question.
It shrinks it:
the choice is what to do with
 two trailing sections of one entry,
with a placeholder that can be dropped from
 consideration entirely.

## How big the change actually is

This document called Option B "small",
then "clearly NOT small",
then "small"
 again,
as later sections corrected earlier ones.
None of the three carried a
 number.
Here is the enumeration instead,
so nobody has to weigh adjectives:

-   `chunkUnits`,
    one label plus a preamble flag per chunk.
    13 lines in the
    prototype.
-   `pairScore`,
    the structure veto plus the existing affinity.
    6 lines.
-   One constant for the penalty.
-   Replacing the `structure-mismatch` branch in `alignDocumentSections`.
-   AND a signature change to `alignHeadings`,
    which is the part every earlier
    estimate missed.

That last one is not optional and was nearly written down wrong.
`alignHeadings`
 takes two `readonly string[]` and calls `headingAffinity` at three points
 inside itself.
It has no scoring hook.
So the structural term cannot reach it
 without either giving it one or passing units instead of strings,
and its 10
 existing test cases have to be reviewed against whichever is chosen.

The prototype sidestepped this by reimplementing Needleman-Wunsch,
36 lines,
 which is why the prototype looked smaller than the change.
A prototype that
 copies the thing it means to modify does not measure the modification.

So:
bounded and enumerable,
roughly 20 lines of new logic plus an interface
 change to a tested function.
Not "small" without qualification,
and not the
 pure wiring the first version of this document claimed.

## Verification notes

Both load-bearing claims were checked at the level the recommendation needs,
 not at the level that was convenient.

The aligner claim rested on `alignHeadings` producing the right HEADING
 pairing,
which does not by itself prove that wiring it fixes a CHUNK-level
 aligner.
The heading-to-chunk correspondence was therefore measured
 separately,
and is exact on all 184 documents at the pin.

The derivability claim rested on a search for the word "derivability",
which
 would miss a consumer importing a symbol whose name lacks it.
Re-run against
 `runDerivabilityProbe`,
`SeedDerivability` and `DERIVABILITY_RESPONSE_FORMAT`
 specifically,
the only consumer outside the module's own files is
 `recall-barrel.ts`,
and neither `repair-benchmark.ts` nor
 `restoration-judge.ts` mentions it at all.
