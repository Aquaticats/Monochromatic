# Audit ledger: the consistency pass, page definitions recorded and carries gated by generation

Paths in this record are the layout after 2026-10-18,
under `package/translation-repair`.

The independent review of the repairs in
[Audit ledger: the consistency pass, voices lost and places left unlocated](audit-ledger-consistency-pass-voices-lost-and-places-left-unlocated.md)
(B656 to B664) left three items open,
and the lead closed them on 2026-10-18 on top of `98d9089318`:
the artifact recount that cannot see the page's definitions (B665,
reopening B661),
the carry reader that kept a tolerance generation 18 does not need (B666,
closing the question left open in B664),
and a refusal text that still named the quotes of the place (B667,
left as it was in B662).
The test commit is `f5dc1482dc` and the fix commit `7410c9948f`.

The generation that followed,
19,
gates the stricter recount of containment and inline code that the sol review
made necessary (B682).
It was written on 2026-10-19 (UTC) from the notes of the port (`s245/w187-notes.md`).

## B665: generation 18 records the page definitions that move a slot

Decided on 2026-10-18 by the lead,
who reopened B661 for the generation that is new and unshipped.

Outcome written on 2026-10-18.
Red `f5dc1482dc`,
fixed in `7410c9948f`.

What changed:
generation 18 carries what its reader needs,
so the limit of B661 stays only for generation 17 and earlier.
`slotInSpan` and `locateWords` take an optional `definitions` string in place of reading the page's,
and the counter (`locatedRendering` in `consistency-count.ts`) puts on a located rendering of a frame
the `definitions` that move its slot,
chosen by `definitionsMovingSlotOf` (`consistency-slot-definitions.ts`).
`recordedRendering` in `pass-consistency-apply-record.ts` writes the field,
`parseConsistencyRendering` reads it from generation 18 and refuses it before,
and the recount (`expectedSlotOf` in `artifact-two-lane-read-consistency-span-slot.ts`,
split out of the span reader to keep it under the line limit)
reads the slot with the recorded definitions.
The predicate is `artifactGenerationRecordsDefinitions`.

Field shape:
`definitions`,
an optional string on a located rendering,
which sits on the occurrence record at `consistency.groups[i].occurrences[j].rendering`.
It is per place and not per group or per artifact,
because the set that moves a slot depends on the place's own span,
and most places have none.
The value is the page's definitions in the form `pageDefinitionsOf` returns,
the entries in page order and separated by a newline,
for example `[b]: https://x.example/b`.
It is the smallest faithful form:
the counter reads the slot with the whole set of the page's definitions,
drops every entry whose absence leaves the slot as it was,
and records what remains.
It records nothing where the span's slot is the same with no definitions at all,
which is nearly every place,
so a page's footnotes are not copied into each place.
A whole footnote of several paragraphs is one entry.
If the reduced set does not read back as itself through `pageDefinitionsOf`
the whole set is recorded in its place.
In the faithfulness check,
`locateWords` and `slotInSpan` with `apostrophes: 'outside'` use the page only through `pageDefinitionsOf(page.text)`,
so passing that string is exactly what passing the page was.

What the reader refuses at generation 18:
definitions that are not a string,
that are empty,
or that `pageDefinitionsOf` does not return unchanged (a sentence,
or definitions with other text among them),
at `...rendering.definitions`;
definitions on a place of a group that is not a refrain,
or that leave the slot as the span alone gives it,
with the refusal "expected the definitions that change the slot the usable ballots give inside those words,
and none where they change none";
and a place that records no definitions where they would move the slot,
which is the slot refusal as before,
because the recount then expects the slot of the span alone and the pass recorded none.
A generation 17 file with the key is refused as an unknown key,
and one without it is recounted over the span alone,
with the pin of B661 kept for that generation
(`artifact-two-lane-read-consistency-span.unit.test.ts`).

Cache and digest:
no cache key reads the field or the generation.
The consistency cache key hashes the sheet and `CONSISTENCY_CACHE_VERSION` (still 5),
and the cache holds the ballots,
while the renderings are counted again on every pass,
so no stored answer changes.
The pipeline digest (`glossary/src/corpus-run/pipeline-digest.ts`) hashes the built `dist` tree,
so it moves with this commit as with any code change and never reads an artifact field,
and the field changes it in no other way.
No page is rewritten.

Test:
`consistency-slot-in-span.unit.test.ts`,
`consistency-span-agreement.unit.test.ts`,
`consistency-slot-definitions.unit.test.ts` for the reduction,
`pass-consistency-apply-record.unit.test.ts`,
`artifact-two-lane-read-consistency-rendering.unit.test.ts`,
`artifact-two-lane-read-consistency-span-slot.unit.test.ts`,
`artifact-two-lane-read-consistency-span-located.unit.test.ts`
and `artifact-two-lane-read-consistency-span.unit.test.ts`.

Recurrence:
`mistake-prevention-stored-files-and-cached-decisions.md`,
"Files a later run reads back".

## B666: the carry reader keeps its tolerance for generation 17 and earlier only

Decided on 2026-10-18 by the lead,
who closed the question left open in B664.

Outcome written on 2026-10-18.
Red `f5dc1482dc`,
fixed in `7410c9948f`.

What changed:
`assertCarriesRecount` takes the generation.
A carry whose first count reached its quorum,
gave no candidate a strict majority
and names finalists under `runOffChoiceFinalistsOf`
must record its `runOff` from generation 18,
and is refused at `...carries[i].runOff` without it.
Generation 17 and earlier read such a carry as it was written,
since the runs of those generations asked the split and went no further.
The predicate is `artifactGenerationRequiresCarryRunOff`.
A first count that missed its quorum needs no run-off at either generation.

Test:
the carry tests in `artifact-two-lane-read-consistency-carry-run-off.unit.test.ts`
(a three-way split and a lone leader are refused at 18 and accepted at 17)
and the fixture and call sites of `artifact-two-lane-read-consistency-carry-recount.unit.test.ts`.
The accept list lost the 1-1 carry with no majority and the earlier run's split with no run-off,
which are now the generation 17 case.
The guard demo switched off the generation check and failed the new case,
and the case passed again with the check restored.

Reader check:
`read-all.ts` over the same five `artifacts` directories with this build accepted all five artifacts
(generations 14,
14,
15,
17 and 17),
output in `w184-notes/reader-check-2.txt`.

## B667: the located-words refusal named the quotes of the place and not of all usable ballots

Found on 2026-10-18 and left as it was in B662.

Outcome written on 2026-10-18.
Red `f5dc1482dc`,
fixed in `7410c9948f`.

The refusal of a located rendering that is not the agreed words read
"words that a strict majority of the quotes naming the place contain",
while the recount counts the quotes of more than half of all the usable ballots (the rule of B641 and B662).
It now reads "words that the quotes of more than half of all the usable ballots contain,
as the place's English holds them".
The two tests that pinned the old text in `artifact-two-lane-read-consistency-span.unit.test.ts`
were changed first and failed at the red commit,
and pass with the text.

## B682: the stricter recount of containment and inline code is gated by a new artifact generation

Found on 2026-10-18 (UTC) in the port of the sol review round (B675 to B681) onto `e87411447d` (`w189`).

Outcome written on 2026-10-19 (UTC).
Tests `6b6f340737` (the options of the consistency package,
which failed because the options were ignored) and `e204ed2681` (the artifact,
which fails to load without `ARTIFACT_SCHEMA_VERSION_V19`),
options `1dc3932a1d`,
the mint `550b14452c` without the gate,
the slot containment case `fe4bd68315`,
and the gate `b462a030a9`.

Why a generation:
the round of the sol review made the recount of an artifact stricter in two ways (B675 and B677),
and the stricter recount would refuse an artifact that was right under the count its pass used.
The first plan was to let the change ride inside generation 18,
because the stored artifacts then known were all of generation 17.
That did not hold:
a live pass was writing generation 18 files,
and `corpus-pass-TianqiChen66626` held one when this was checked (M265).

What a file of generation 18 is read by,
and what 19 changes (`ARTIFACT_SCHEMA_VERSION_V19`,
`artifact/src/corpus-run/artifact-two-lane-contract.ts`):

- The writer stamps 19 (`artifact-two-lane-build.ts`),
  and `pass-schema-guard.ts` writes and expects 19,
  so a build of 19 refuses to resume a run directory stamped 18 (`assertResumableSchemaGeneration`).
  That is expected,
  and the V19 TSDoc says so.
  Files of 18 stay readable by `readSettledArtifact`.
- `artifactGenerationCutsContainmentAtJoints` (19 and later):
  `containsWords` asks `cutsWord` about the edges of a match.
  Before 19,
  only a letter,
  a digit or a combining mark beside the match bars containment,
  so `cat` is contained in `cat's bowl`.
  The option is `edges: 'locator' | 'letters'` (`ContainmentEdges`) on `containsWords`,
  `containedQuoteWordsOf`,
  `agreedSlotWordsOf` and `slotInSpan`.
  The default is the locator's,
  which is all that any pass and any reader of a page uses.
- `artifactGenerationReadsLineEndingsAsCodePadding` (19 and later):
  a line ending pads inline code as a space does.
  Before 19,
  a literal space alone pads.
  The option is `codePadding: 'spaces' | 'spaces-and-line-endings'` (`CodePadding`) on `locateWords` and `slotInSpan`.
  It is passed on through `cutsShownWord`,
  `unitKindsOf`,
  `transparentUnitsOf`,
  `rememberedViewOf`,
  `proseViewOf` and `markupRangesOf` to `codeContentOf`,
  and the remembered parse is keyed by it as well as by the English and the definitions.
- The callers pass the generation:
  `expectedSlotOf` passes both options,
  `assertSpanRenderingAgrees` passes the first to `containedQuoteWordsOf`,
  and `asLocatedBySpan` takes `generation` and passes the second to `locateWords`.
- Not gated:
  the refusal of a run-off beside an unasked carry (B680) applies to every generation,
  since no pass writes one.
- No cache version moves:
  counting is not stored,
  so `CONSISTENCY_CACHE_VERSION` stays at 5.

Proof:
without the gate,
the generation 18 cases fail with the whole refusal text,
`words that the quotes of more than half of all the usable ballots contain` for `cat` beside `cat's bowl`,
and the slot refusal for the padded code.
Each of four points where the generation is handed on was removed in turn,
and the generation test failed each time.

Reader check (read-only):
the reader of this build accepted six files of generations 14,
14,
14,
15,
17 and 17,
and the generation 18 file of `corpus-pass-TianqiChen66626` (two groups).
That file does not show that the gate was needed.
It shows that a file the live pass wrote is read.

Found on the way:
the TSDoc of `locateWords` (`consistency-fold.ts`) said that definitions are stored "generation 18 for a slot,
19 for the words".
The second half described the withdrawn attempt of B673,
and `b9ca743e06` removes it.
The source-scan test `clock times written with no zone` failed at `e87411447d` on `16:58` and `17:00` with no zone
in the handover,
a file that this work had not changed;
`57165b3f07` zones the corrected time,
and also rewrites the handover sentence about the withdrawn generation 19 to say that the number went to this gate.

Recurrence:
`mistake-prevention-stored-files-and-cached-decisions.md`,
"Files a later run reads back".
