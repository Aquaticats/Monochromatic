# Writer calibration unit scope: why the first unit scope was wrong and which sources are eligible

The evidence that a heading-only task was refused by the archive floor while its body source sat in a sibling,
the source-authority rule that declines whole-page English originals before pool freezing,
and the finding that source availability is not incumbent accuracy.

Part of [Writer calibration unit scope](translation-repair-writer-unit-scope-2026-09-11.md),
which lists every file of this record.

## Evidence that changes the launch decision

The existing deterministic forty-row sample was read completely.
It contains 4290 source characters and 11946 incumbent characters,
with no empty side.
Heading-only and media-only tasks are not automatically invalid,
and an imperfect incumbent is not mislabeled gold in the producer instrument.

However,
`windward0032#14` pairs only the ten-character Chinese heading
with an English heading and three body paragraphs,
849 characters in total.
Its corresponding body source is in sibling `#15`,
whose incumbent contains only the final English tail.

The actual frozen validator establishes the consequence:

```text
The PAGE AS IT STANDS is 4 blocks (heading (level 2), paragraph, paragraph, paragraph)
and your translation is 1 (heading (level 2)).
Every block of the PAGE AS IT STANDS has to appear in your translation,
of the same kind and in the same order.
```

A heading-only candidate is refused,
while the complete incumbent passes.
The `lintong#0` heading-to-heading positive control passes.
Evidence is `v41-writer-heading-floor-20260911.out`
and the owned workspace's `heading-floor-check.json`.
This is the production archive floor functioning as designed against an incorrectly scoped benchmark unit.

An initial review had conditionally recommended preserving the forty rows because all writers see the same input.
The concrete floor test disproved the necessary premise:
the instrument forces preservation of body content whose source is outside the supplied task.
The independent reviewer withdrew that recommendation after seeing this probe.
Equal inputs do not establish validity of the task being ranked.

## Declared-original eligibility before pool freezing

The initial unapproved normalized pool also needed the existing source-authority admission boundary.
A provider-free review of its frozen input texts identified
`gqt/source-section/6/target-section/6` as part of a whole-page English original.
Both raw and normalized archive readings return `whole-page`.
`package/module/translation-repair/src/corpus-run/pass-entry.ts:155` (moved on 2026-10-18 to
`package/translation-repair/corpus-pass/src/corpus-run/pass-entry.ts`)
declines such an entry before any purchase.
The initial prototype's selected `hakureico` parent also requires the existing sealed-span treatment.

Apply those production rules before freezing the acquisition population and deterministic draw.
Do not buy correspondence for a production-declined entry,
use protected original-English text as a writing task,
or drop a parent only after acquisition outcomes arrive.
Recompute the eligible population and revalidate the reading journal against its identities and hashes.
This is source-authority eligibility,
not the rejected size/shape filtering.
The eligible empty-layout `ArtsEpiphany` preservation control remains a separate case.

Evidence is `prototype-original-eligibility.json` in the owned writer workspace
and `~/temp/agent/writer-prototype-original-eligibility-20260911.out`.
The probe ran without network in the configured 2 GiB,
2 CPU,
512 PID container.
No paid preparation or writing preceded this finding.

## Source availability is not incumbent accuracy

The selected `XingZ60` third-contributor parent raised a source-content question:
the source contains `补` with author/date metadata,
while the archive contains an English epitaph.
Pinned history shows an archive-only change from `Supplement` to the epitaph at
`66d280e75a8bf5b639f033615430c3117fd2c852`.
The source blame at `388360c8e23a76dc7b1a59c47c2cd06fcd52b0b6`
only wraps the existing word in a blockquote.
Neither fact establishes authorial intent or original-English authority.

An independent review initially recommended excluding unsupported positive archive content.
That rule was withdrawn before implementation:
it would also remove normal correctable defects from an intentionally imperfect writer incumbent,
such as a name-gloss addition or a reversed disclosure actor.
No population was invalidated,
no parent removed and no redraw performed on that basis.

The narrower qualification is source availability and modality compatibility.
Unknown nonempty source text remains text.
A typed editorial-placeholder classification requires an existing generic policy or parser role;
the archive-stub rule does not establish that role for source `补`.
Metadata,
media and preservation channels remain distinct,
and this is not permission to assign the epitaph special source authority.

The exact prepared-unit consumer probe resolves the floor question.
It passes `translateSliceInput` into the actual `validateTranslatedSlice` options:
both the incumbent and a same-shape `Supplement` wording witness pass strict grammar,
while a heading-only positive control fails for removing the required blockquote.
The witness is not gold and is not supplied to writers.
The archive floor does not force the epitaph's words,
so this remains an imperfect incumbent task rather than a proven scope mismatch.
Evidence is `source-floor-probe-20260912.json` and `writer-source-floor-control-20260912.out`.
The population and pool digests remain unchanged.

Apply an auditable source-channel/availability manifest before acquisition and to final prepared children.
Do not turn that manifest into factual-quality curation of incumbents,
infer placeholder intent from short text,
or make a calibration-only no-writing exception.
