# Translation repair history 2026-08-19 to 2026-08-21: damage instruments that see past one slice

The assembly-level repetition check,
wired into both lanes,
the introduced-defect probe given the window,
what is verified and what is only predicted,
the consistency test that replaced an unsound prediction,
and a documentation trap hit twice.

Part of [Translation repair history: 2026-08-19 to 2026-08-21](translation-repair-history-2026-08-19-to-21.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The damage instruments now see past one slice

Three changes landed 2026-08-20,
all from `#66`,
and the ordering among them is the finding
rather than a preference.

```text
assembly-repetition.ts      NEW    document-scale repetition, no model and no quota
repair-assemble.ts          wired  emits the finding where the footnote guard runs
translate-assemble.ts       wired  same, because the check found damage in that lane too
introduced-defect-wire.ts   window the probe now sees the neighbours, fence included
introduced-defect-probe.ts  window forwards them
repair-chunk.ts             window one windowFragment now feeds four stages
```

WHY REPETITION IS CHECKED AT ASSEMBLY and not inside a stage:
`repair-assemble.ts` already
carries the argument in its own words,
for footnotes.
"A footnote is a relation BETWEEN
slices,
and every stage works inside one,
so this is the only layer that can see it."
Repetition is the same shape,
which is why `#66` found the probe structurally unable to
report `lintong`'s duplication however it was tuned.

THE ARCHIVE IS WHAT MAKES IT DECIDABLE,
and `#128` put it in the artifact hours earlier.
A phrase the archive says once and the shipped document says twice is a repetition this
pipeline added.
Counting against the archive rather than against a rule about prose inherits
the author's own judgement about acceptable repetition,
so refrains and repeated names never
fire.

BOTH LANES,
and that was not the plan.
The damage was found in `lintong`'s REPAIR lane,
and
running the check over the settled pool immediately produced one finding in `saurikissa`'s
translate lane and one in `dogesir_`'s,
where the archive never carried the wording at all and
the document says it twice.
Writing a slice from its source rather than editing an incumbent
does not stop a lane repeating itself.

### What is verified, and what is only predicted

VERIFIED:
the function,
by six cases including the three it must NOT fire on.
That the call
ships in the same built chunk as the assembler.

CORRECTED 2026-08-19,
and the correction is the important part:
the sentence that used to
stand here claimed the check "reproduces both the known `lintong` duplication and the known
lane split" on real data.
It does not,
and the reading that produced that claim was made
against a RECONSTRUCTED archive rather than the real one.
See
`doc/audit/an-archive-rebuilt-from-the-ledger-is-not-the-archive.md`.

NOT VERIFIED:
that a finding actually lands in an artifact written by a live run.
The unit
tests cover the function and the dist check covers the call's presence,
but neither is an
artifact carrying `introduced-repetition`.

THE CHECKABLE PREDICTION,
RE-RECORDED 2026-08-19 because the first wording was unsound.
It
said the next run settling `lintong` MUST produce a repair-lane `introduced-repetition`,
and
that rested on the belief that the archive says the duplicated phrase once.
The archive says
it twice,
so a `lintong` artifact carrying NO finding is now the expected good outcome,
and
the old wording would have told the next reader to call a healthy run broken wiring.

The sound test is a CONSISTENCY test,
and it needs no prediction about the corpus at all.
Run `findIntroducedRepetitions` offline over the artifact's OWN `archiveText` and assembled
text,
then compare against the findings the artifact recorded:

```text
offline fires, artifact records it        wiring is good
offline fires, artifact silent            WIRING IS BROKEN
offline silent, artifact silent           consistent, and says nothing about wiring
offline silent, artifact records one      the shipped path sees something offline cannot
```

Only the second row is a defect.
Reading the third as one is the mistake the first wording
would have caused.

### A trap hit twice in one session

Inserting a new declaration between an existing TSDoc block and the declaration it documents
orphans the doc,
and `tsdoc(require-tsdoc)` then reports the ORIGINAL declaration as
undocumented,
which reads like an unrelated regression.
It happened in
`artifact-v2-read-contract.ts` and again in `introduced-defect-wire.ts`.
Put a new constant
ABOVE the whole doc-plus-declaration pair,
never between them.
