# Audit ledger: an archive with no body ships an empty page

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

An entry whose archive file is present but holds no body passed intake,
was cut into no slice,
and shipped a page of 0 bytes with no defect and no warning.
The commitment it broke is in `package/translation-repair/doc/design-commitments-shipping-and-the-archive.md`:
a slice with no archive text and no backed wording ships its source text under a note.
Round w374 reproduced the defect on the built packages,
fixed the slicing,
made the publisher report an empty page,
and measured one neighbouring case it did not fix.

## B1045: an archive with no body was cut into no slice

Status:
fixed on 2026-10-29 (UTC) in `tr-w374`,
red `77ec40623f`,
fix `c2eb1538b1`,
roster-path case `384e47d58f`.

`slice-cache/src/corpus-run/pass-eligibility.ts` drops an entry only when its archive file is missing.
An archive that is empty,
whitespace,
a byte order mark,
or front matter alone has no chunk,
and `alignDocumentSections` (`slice-reading/src/chunk-document.ts`) returned `pairs: []` for it
with the finding "one side has no content",
before `placeInsertions`,
the only producer of an anchor for an untranslated section,
could run.
`prepareDocumentPair` then yielded no body slice,
and `publishFixedPage` wrote the empty page.
A control with one anchor slice over the same empty archive shipped the note and the original,
so only the slicing was wrong.

The fix:
for a non-empty original against a translation with no chunk,
`alignDocumentSections` returns one pair per original chunk,
each an insertion anchor at the translation's insertion offset,
which is the end of its front matter,
or the width of its byte order mark when it has none.
The whole-document finding stays and now ends with the offset.
`placeInsertions` is not asked,
because it reads the translation's sections and this one has none.
An original with no content still yields no pair.

The red cases,
four,
each failing for the reason named:
- `chunk-document.unit.test.ts`:
  one case over six archives
  (empty,
  whitespace,
  byte order mark,
  front matter,
  mark and front matter,
  front matter and whitespace)
  found `pairs: []` for all six.
- `document-preparation.unit.test.ts`:
  one case found 0 body slices for the empty archive under the rendered authority.
- `publish-fixed.unit.test.ts` and `source-shipped-page.unit.test.ts`:
  see B1046.

The layout check:
`assertPlacementLayout` refuses a span only when it starts before the previous span's end,
and an insertion ends where it starts,
so several anchors at one offset pass as written
(the header of `corpus-source/src/placement-layout.ts` lists several insertions at one boundary as legal).
Nothing was loosened.
`prepareDocumentPair` runs the layout,
coverage,
contiguity and container assertions over the new slices,
and the adjudication case passes them for an empty archive and for a mark-only archive,
under both front matter authorities.

Cache versions:
none moved.
- Every slice cache key hashes its version,
  the shape of the run and the slice's own texts.
  An entry with no body was cut into no slice before,
  so no stored record belongs to a slice the change adds,
  and a stored record whose texts equal a new slice's texts
  (an original section against an empty archive)
  answers the same question the new slice asks.
- No pairing is bought or stored for such an entry:
  the section round returns before asking when the translation has no section,
  and a block round returns `empty` when a side has no block
  (`prepare-section-round.ts`,
  `prepare-block-pairing.ts`).
  `prepare-unaligned-archive.unit.test.ts` runs the roster path over an empty archive with a client that fails on any call,
  and it cuts one anchor per section and asks nobody.
  `PAIRING_CACHE_VERSION` stays.
- The artifact shape is unchanged,
  so `ARTIFACT_SCHEMA_VERSION_V22` stays.
- Corpus pin `a41fc607ea` holds 0 of 92 complete pairs with an archive of this kind,
  so no stored run directory is affected,
  and every cache is opened under the pipeline digest of the build besides.

## B1046: a page that ships no body while its original has one was not reported

Status:
fixed on 2026-10-29 (UTC) in `tr-w374`,
red `77ec40623f`,
fix `82e207c636`.

`publishFixedPage` composed the page from the slices and wrote it.
With no slice and an empty archive the page was 0 bytes,
and none of its content checks looked at whether the page held anything:
the front matter check named the missing metadata and nothing else.
The new check `page-without-body` (`pageWithoutBodyDefects`,
`page-check/src/corpus-run/publish-defects.ts`)
parses the page and the original,
and reports one defect when the original holds a block and the page holds none,
naming the entry and the original's block count only.
Front matter and a byte order mark are no block,
so a page that is only metadata is reported too.
An original with no body is not a defect,
and a page with a body is not.
The page still ships,
like every other content check
(the owner's answer of 2026-09-27,
"Ship with defect reported").
The report is not a thrown refusal,
so it adds no class to the list of messages that name only operations and counts.
It is exported from `page-check/src/index-reading.ts`,
because `index.ts` sits at its line bound.

The red case in `publish-fixed.unit.test.ts` failed with the one defect `front-matter`
for an empty page,
and the case in `source-shipped-page.unit.test.ts` read an empty page where it expected the note.

## B1047: an archive that does not align with two original sections, through the roster

Status:
measured on 2026-10-29 (UTC) in `tr-w374`,
test `d9fb8cb850`.
Not fixed:
a stated limit,
because no real run has shown it
(the count over real runs closes this entry).

Round w373 saw that a non-empty archive that does not align with two original sections
(one heading,
or one paragraph)
was cut into no slice by the deterministic aligner.
`prepare-unaligned-archive.unit.test.ts` measures the real path,
`prepareDocumentPairWithRoster`,
with the same scripted reply from both voices,
for an original of two heading sections against each of the two archives:
- The deterministic preparation cuts 0 slices for both archives,
  as round w373 measured.
- When the roster pairs nothing,
  the preparation falls back to the deterministic aligner and cuts 0 slices,
  and every original section is refused as `source-only (ambiguous)`.
- When the roster pairs one original section with the archive's only section,
  the preparation cuts 2 slices,
  one of them an anchor inside the paired section,
  and the original section the roster left out is in no slice.
  Its finding reads `source-only (roster-unpaired); not anchored (beyond-shortfall)`,
  so the page ships without that section's original.
  The result is the same for either section and for both archives.

So the count is 0 only when the roster pairs nothing,
and the case that matters is the one where the roster pairs a single section:
a section refused as not anchored is left off the page.

The count over real runs was taken on 2026-10-29 (UTC),
read-only,
over every stored artifact and corpus-pass log under the agent's temp directory
and the pooled runs directories.
289 artifacts carry `preparation.alignmentFindings`:
24 from corpus passes over 4 entries,
265 from scratch and validation runs over 42 entries.
`not anchored (` appears in none of them and in no run log.
The only real roster that paired part of an original is entry XingZ60's,
13 of 15 sections,
in 21 artifacts,
5 of them corpus passes:
XingZ6015,
XingZ6017,
XingZ6020,
XingZ6021
and XingZ6022.
Its two unpaired sections,
13 and 14,
read `source-only (roster-unpaired); anchored for insertion at offset 33184` every time,
so they reached the page.
No roster paired zero sections.
The search was proven able to find the refusal:
the same text matches 32 times in this round's own measurement log
and in several mutation logs.
What the count cannot see:
runs before commit `1fb20af076` (2026-10-23) never logged alignment findings,
so their artifacts are the only record,
and runs before 2026-08-18 keep no `preparation` field at all.

The owner's rule of 2026-10-25 lands a round on real impact
and makes a gap that only invented inputs show into a stated limit and a separate later item.
This is such a gap,
so nothing changes for it:
a section the roster leaves unpaired that cannot be anchored is left off the page,
and the alignment finding names it.
Whether `beyond-shortfall` is the right refusal,
and whether such a section should ship its original under a note,
is the later item,
to be taken up if a real run ever shows the refusal.

## Ids used

B1045 to B1047.
B1048 and B1049 are unused.
