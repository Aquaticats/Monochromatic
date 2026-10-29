/**
 Tests for section chunking and PARTIAL automatic alignment:
 heading-bounded partition, preamble handling, exact offsets, equal-shape
 index pairing, and refusal of sections the headings cannot pair.
 Fixtures are cat-themed invention only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  isInsertionChunk,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  alignDocumentSections,
  chunkByHeadings,
  describeAlignmentAttachment,
} from '../dist/final/node/index.mjs';

/**
 Source whose headings carry romanised names, which is the `XingZ60` shape:
 the only shape where the aligner has evidence to anchor on, and therefore the
 only one where a missing section can be proven missing rather than merely
 unpairable.
 */
const ANCHORED_SOURCE = `---
name: whiskers
---

开场白：猫猫登场。

## 其一：Mittens

猫猫喜欢晒太阳，尾巴一摇一摇。

## 其二：Boots

猫猫每天下午都在窗台上打盹，直到太阳落下。

## 其三：Paws

猫猫和邻居家的黑猫是好朋友，它们常常一起追蝴蝶。
`;

/**
 Translation of that source missing its middle section, and short enough that
 the page has room to be missing it.
 */
const SHORT_TARGET = `---
name: whiskers
---

Prologue: the cat arrives.

## Mittens

The cat loves sunbathing, tail swishing.

## Paws

The cat and the black cat next door are friends.
`;

/**
 The same missing section, in a translation that runs LONGER than its source
 predicts. Nothing about the alignment differs; only the length does.
 */
const LONG_TARGET = `---
name: whiskers
---

Prologue: the cat arrives, stepping softly across the warm boards of the porch outside.

## Mittens

The cat loves sunbathing, tail swishing slowly back and forth in the light, and will lie there for hours together without once looking up at anybody who happens to pass by the window on the street.

## Paws

The cat and the black cat from the house next door are the firmest of friends, and the two of them spend whole afternoons chasing butterflies across the long grass at the bottom of the garden until neither can be bothered to chase anything further.
`;

/**
 Two-section fixture with front matter and a preamble paragraph.
 */
const SOURCE_FIXTURE = `---
name: whiskers
---

开场白：猫猫登场。

## 简介

猫猫喜欢晒太阳。

猫猫也追蝴蝶。

## 习性

> 「喵。」
`;

/**
 Mirrored translation fixture with identical block structure.
 */
const TARGET_FIXTURE = `---
name: whiskers
---

Prologue: the cat arrives.

## Introduction

The cat loves sunbathing.

The cat also chases butterflies.

## Habits

> "Meow."
`;

/**
 Translation fixture whose extra section breaks the mirror:
 same preamble shape, but three sections against the source's two.
 */
const EXTRA_SECTION_FIXTURE = `---
name: whiskers
---

Prologue: the cat arrives.

## Introduction

The cat loves sunbathing.

## Habits

The cat also chases butterflies.

## Legacy

> "Meow."
`;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: chunkByHeadings.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'partitions nodes into preamble and heading-bounded chunks',
          fn: async () => {
            /** Parsed source fixture. */
            const doc = parseDocument({ text: SOURCE_FIXTURE, },);
            /** Chunks of the source fixture. */
            const chunks = chunkByHeadings({ document: doc, },);

            expect(chunks.map(function toLeadingKind(chunk,): string {
              return chunk.nodes[0]?.kind ?? '';
            },),).toEqual(['paragraph', 'heading', 'heading',],);
            expect(chunks.map(function toNodeCount(chunk,): number {
              return chunk.nodes.length;
            },),).toEqual([1, 3, 2,],);
            // Every node lands in exactly one chunk, order preserved.
            expect(chunks.flatMap(function toNodes(chunk,) {
              return chunk.nodes;
            },),).toEqual(doc.nodes,);
          },
        },),

        it({
          name: 'slices chunk text exactly from node offsets',
          fn: async () => {
            /** Parsed source fixture. */
            const doc = parseDocument({ text: SOURCE_FIXTURE, },);
            for (const chunk of chunkByHeadings({ document: doc, },)) {
              expect(chunk.text,).toBe(doc.text.slice(
                chunk.startOffset,
                chunk.endOffset,
              ),);
              expect(chunk.text.startsWith(chunk.nodes[0]?.text ?? '',),).toBe(true,);
            }
          },
        },),

        it({
          name: 'returns no chunks for an empty body',
          fn: async () => {
            expect(chunkByHeadings({
              document: parseDocument({ text: '---\nname: n\n---\n', },),
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: alignDocumentSections.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'pairs mirrored documents index by index without findings',
          fn: async () => {
            /** Alignment of the mirrored fixtures. */
            const alignment = alignDocumentSections({
              source: parseDocument({ text: SOURCE_FIXTURE, },),
              target: parseDocument({ text: TARGET_FIXTURE, },),
            },);

            expect(alignment.findings,).toEqual([],);
            expect(alignment.pairs,).toHaveLength(3,);
            expect(alignment.pairs[1]?.source.nodes[0]?.kind,).toBe('heading',);
            expect(alignment.pairs[1]?.target.nodes[0]?.kind,).toBe('heading',);
          },
        },),

        it({
          name: 'REFUSES sections it cannot pair rather than merging them '
            + 'proportionally, and the document still settles. Proportional merging '
            + 'produced a confident wrong pairing, which fed critics the wrong '
            + 'original and made every issue filed on that entry noise',
          fn: async () => {
            /**
             Sides whose section counts differ and whose headings share nothing.
             */
            const alignment = alignDocumentSections({
              source: parseDocument({ text: SOURCE_FIXTURE, },),
              target: parseDocument({ text: EXTRA_SECTION_FIXTURE, },),
            },);

            // Nothing is forced, so nothing is paired, and that is the point.
            expect(alignment.pairs,).toHaveLength(0,);
            expect(alignment.findings.map(function toKind(finding,): string {
              return finding.kind;
            },),).toContain('structure-mismatch',);
          },
        },),

        it({
          name: 'NEVER blocks: a document the aligner cannot pair at all still '
            + 'returns, with findings and no pairs, because the pipeline yields '
            + 'output for every entry whatever the input and an entry that refuses '
            + 'to settle vanishes from every measurement silently',
          fn: async () => {
            /**
             One-section source against a three-section target, sharing nothing.
             */
            const alignment = alignDocumentSections({
              source: parseDocument({ text: '## 简介\n\n猫猫喜欢晒太阳。\n', },),
              target: parseDocument({
                text: '## One\n\nA cat.\n\n## Two\n\nAnother cat.\n\n## Three\n\nThird cat.\n',
              },),
            },);

            expect(alignment.pairs,).toHaveLength(0,);
            expect(alignment.findings.length,).toBeGreaterThan(0,);
          },
        },),

        it({
          name: 'PAIRS a leading-kind mismatch with equal counts instead of '
            + 'reporting a structure mismatch. That report was spurious: with one '
            + 'chunk on each side the old test could only fail on leading node '
            + 'kinds, so it described an asymmetric preamble and called it structure',
          fn: async () => {
            /**
             Preamble-led source against a headings-only target, equal counts.
             */
            const alignment = alignDocumentSections({
              source: parseDocument({ text: SOURCE_FIXTURE, },),
              target: parseDocument({
                text: '## One\n\nA cat.\n\n## Two\n\nAnother cat.\n\n## Three\n\nThird cat.\n',
              },),
            },);

            expect(alignment.pairs,).toHaveLength(3,);
            expect(alignment.findings,).toHaveLength(0,);
          },
        },),
        it({
          name: 'ANCHORS A MISSING SECTION FOR INSERTION, which is what turns a report into a repair. '
            + 'The aligner proves every optimal alignment skips it at the same boundary, and the page '
            + 'is measurably shorter than its source predicts, so both signatures agree it is absent '
            + 'rather than merged somewhere',
          fn: async () => {
            /**
             A translation missing its middle section.
             */
            const alignment = alignDocumentSections({
              source: parseDocument({ text: ANCHORED_SOURCE, },),
              target: parseDocument({ text: SHORT_TARGET, },),
            },);

            /**
             Pairs whose translation side names a place rather than covering text.
             */
            const anchors = alignment.pairs
              .filter(function isAnchor(pair,) {
                return isInsertionChunk(pair.target,);
              },);

            expect(anchors.length,).toBe(1,);
            // The section belongs before `## Paws`, which begins here. An anchor at
            // the end of the previous section instead would write it inside
            // `## Mittens`.
            expect(anchors[0]
              ?.target
              .startOffset,)
              .toBe(SHORT_TARGET.indexOf('## Paws',),);
          },
        },),

        it({
          name: 'SAYS WHERE the insertion goes IN THE FINDING an operator reads, since the anchoring '
            + 'cases beside this one assert the PAIR rather than the sentence, and the refusal wording '
            + 'is the only other thing that field ever carries: a finding naming no offset reads like '
            + 'one more section the aligner gave up on',
          fn: async () => {
            /**
             That same document, whose middle section is anchored rather than
             refused.
             */
            const alignment = alignDocumentSections({
              source: parseDocument({ text: ANCHORED_SOURCE, },),
              target: parseDocument({ text: SHORT_TARGET, },),
            },);

            /**
             What the findings say about the source side.
             */
            const sourceSide = alignment.findings
              .map(function toDetail(finding,) {
                return finding.detail;
              },)
              .filter(function isSourceSide(detail,) {
                return detail.startsWith('source-only',);
              },);

            expect(sourceSide.length,).toBe(1,);
            expect(sourceSide[0],).toContain(
              `anchored for insertion at offset ${String(SHORT_TARGET.indexOf('## Paws',),)}`,
            );
          },
        },),
        it({
          name: 'KEEPS EVERY OTHER SECTION PAIRED AND IN ORDER around an anchor, since a document '
            + 'whose insertion displaced a real pairing would repair the wrong passages either side '
            + 'of the hole it filled',
          fn: async () => {
            /**
             That same document.
             */
            const alignment = alignDocumentSections({
              source: parseDocument({ text: ANCHORED_SOURCE, },),
              target: parseDocument({ text: SHORT_TARGET, },),
            },);

            expect(alignment.pairs.map(function toShape(pair,) {
              return isInsertionChunk(pair.target,) ? 'anchor' : 'paired';
            },),)
              .toStrictEqual([
                'paired',
                'paired',
                'anchor',
                'paired',
              ],);
          },
        },),

        it({
          name: 'REFUSES THE SAME MISSING SECTION when the page is not short, which is the second '
            + 'signature doing the work the aligner cannot: a page carrying more English than its '
            + 'source predicts is likelier to have merged that section somewhere than to have '
            + 'dropped it, and writing it in would duplicate content',
          fn: async () => {
            /**
             The same source and the same gap, in a longer translation.
             */
            const alignment = alignDocumentSections({
              source: parseDocument({ text: ANCHORED_SOURCE, },),
              target: parseDocument({ text: LONG_TARGET, },),
            },);

            expect(alignment.pairs
              .filter(function isAnchor(pair,) {
                return isInsertionChunk(pair.target,);
              },)
              .length,)
              .toBe(0,);

            expect(alignment.findings.map(function toDetail(finding,) {
              return finding.detail;
            },),)
              .toStrictEqual(['source-only (forced-gap); not anchored (page-not-short)',],);
          },
        },),

        it({
          name: 'NAMES WHY IT REFUSED rather than reporting one silent absence, because the reasons '
            + 'want opposite remedies: a section that may already be on the page is a duplication '
            + 'risk, and a page that looks complete is a merge the aligner misread',
          fn: async () => {
            /**
             The anchored source against a translation whose one surviving section
             shares no evidence with any of it, so every pairing stays possible.
             */
            const alignment = alignDocumentSections({
              source: parseDocument({ text: ANCHORED_SOURCE, },),
              target: parseDocument({ text: SHORT_TARGET.replaceAll('Mittens', 'Sunbeam',)
                .replace('## Paws', '## Naps',), },),
            },);

            expect(alignment.pairs
              .filter(function isAnchor(pair,) {
                return isInsertionChunk(pair.target,);
              },)
              .length,)
              .toBe(0,);

            // The page IS short here, so a gate reporting only "not inserted" would
            // read identically to the "REFUSES THE SAME MISSING SECTION when the
            // page is not short" case, and an operator
            // could not tell a duplication risk from a merge the aligner misread.
            expect(alignment.findings
              .map(function toDetail(finding,) {
                return finding.detail;
              },)
              .filter(function isSourceSide(detail,) {
                return detail.startsWith('source-only',);
              },),)
              .toStrictEqual([
                // Four, not three: with nothing to anchor on, the PREAMBLE is as
                // unpairable as the three sections are.
                'source-only (ambiguous); not anchored (may-pair)',
                'source-only (ambiguous); not anchored (may-pair)',
                'source-only (ambiguous); not anchored (may-pair)',
                'source-only (ambiguous); not anchored (may-pair)',
              ],);
          },
        },),

        it({
          name: 'GIVES EVERY SOURCE CHUNK AN INSERTION PAIR when the archive carries no body, whether it is empty, '
            + 'whitespace, a byte order mark or front matter alone, since a pair-less alignment ships a 0-byte page '
            + 'where the commitment is the original under a note',
          fn: async () => {
            /**
             Byte order mark a page may open with.
             */
            const mark = String.fromCodePoint(0xFEFF,);
            /**
             The front matter an archive may open with and carry no body after.
             */
            const metadata = '---\nname: n\n---\n';
            /**
             Archives with no body, and the offset a note belongs at in each: after the mark and the front
             matter, never inside either.
             */
            const bodiless: readonly (readonly [string, string, number,])[] = [
              ['empty', '', 0,],
              ['whitespace only', '  \n\n', 0,],
              ['a byte order mark only', mark, 1,],
              ['front matter only', metadata, metadata.length,],
              ['a mark and front matter', `${mark}${metadata}`, metadata.length + 1,],
              ['front matter and whitespace', `${metadata}\n`, metadata.length,],
            ];
            /**
             Chunks the original is cut into, which the alignment must account for one by one.
             */
            const sourceChunks = chunkByHeadings({ document: parseDocument({ text: SOURCE_FIXTURE, },), },);
            expect(sourceChunks,).toHaveLength(3,);

            /**
             What each archive's alignment holds, by the archive's name.
             */
            const observed = Object.fromEntries(bodiless.map(function observe([name, text,],) {
              /**
               Alignment of the original against this archive.
               */
              const alignment = alignDocumentSections({
                source: parseDocument({ text: SOURCE_FIXTURE, },),
                target: parseDocument({ text, },),
              },);
              return [
                name,
                {
                  sources: alignment.pairs.map(function toSourceText(pair,): string {
                    return pair.source.text;
                  },),
                  anchors: alignment.pairs.map(function toAnchor(pair,): boolean {
                    return isInsertionChunk(pair.target,);
                  },),
                  offsets: alignment.pairs.map(function toOffset(pair,): number {
                    return pair.target.startOffset;
                  },),
                  // THE FINDING STAYS, since the archive still carries no body.
                  findings: alignment.findings.length,
                },
              ];
            },),);
            /**
             What each must hold: one anchor per original chunk, at the offset after the mark and the metadata.
             */
            const expected = Object.fromEntries(bodiless.map(function expectFor([name, , offset,],) {
              return [
                name,
                {
                  sources: sourceChunks.map(function toText(chunk,): string {
                    return chunk.text;
                  },),
                  anchors: sourceChunks.map(function toTrue(): boolean {
                    return true;
                  },),
                  offsets: sourceChunks.map(function toOffset(): number {
                    return offset;
                  },),
                  findings: 1,
                },
              ];
            },),);
            expect(observed,).toEqual(expected,);
          },
        },),
      ],
    },),

    describe({
      name: describeAlignmentAttachment.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE WHOLE DOCUMENT rather than borrowing index zero, which is '
            + 'the reading that made an observation about the two documents '
            + 'indistinguishable from one about the first pair',
          fn: async () => {
            expect(
              describeAlignmentAttachment({ attachedTo: { kind: 'whole-document', }, },),
            ).toBe('whole document',);
          },
        },),

        it({
          name: 'NAMES THE ORIGINAL SIDE AND ITS OWN NUMBERING for a refusal there, '
            + 'because a refused chunk holds only the index it has on its own side '
            + 'and that side need not count the same way the pairs do',
          fn: async () => {
            expect(
              describeAlignmentAttachment({
                attachedTo: {
                  kind: 'source-section',
                  index: 3,
                },
              },),
            ).toBe('source section 3',);
          },
        },),

        it({
          name: 'NAMES THE TRANSLATED SIDE SEPARATELY, so two refusals carrying the '
            + 'same number on opposite sides no longer render as one place',
          fn: async () => {
            expect(
              describeAlignmentAttachment({
                attachedTo: {
                  kind: 'target-section',
                  index: 3,
                },
              },),
            ).toBe('target section 3',);
          },
        },),

        it({
          name: 'RENDERS A REAL REFUSAL WITHOUT THE WORD PAIR, which is the whole '
            + 'defect: the aligner reported a side index under the same wording a '
            + 'genuine pair index uses, and a reader meeting one could not tell '
            + 'which numbering the number lived in',
          fn: async () => {
            /**
             Sides whose section counts differ and whose headings share nothing,
             so every section is refused rather than paired.
             */
            const alignment = alignDocumentSections({
              source: parseDocument({ text: SOURCE_FIXTURE, },),
              target: parseDocument({ text: EXTRA_SECTION_FIXTURE, },),
            },);

            expect(alignment.findings.length,).toBeGreaterThan(0,);
            for (const finding of alignment.findings)
              expect(
                describeAlignmentAttachment({ attachedTo: finding.attachedTo, },)
                  .includes('pair',),
              ).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
