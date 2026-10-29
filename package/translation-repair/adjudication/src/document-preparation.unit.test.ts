/**
 Tests for the lane-neutral preparation both drivers slice from.

 The property that matters is that ONE document pair yields ONE slicing. Two
 lanes preparing separately would each report slices that look right on their
 own, and nothing downstream could tell that a repair outcome and a translate
 outcome for "slice 4" described different spans.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { assertPlacementLayout, } from '@monochromatic-dev/translation-repair-corpus-source';
import { spliceSlices, } from '@monochromatic-dev/translation-repair-page-pass';
import {
  isInsertionChunk,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { isLineStructured, } from '@monochromatic-dev/translation-repair-slice-reading';
import { prepareDocumentPair, } from '../dist/final/node/index.mjs';

/**
 Lines in the verse fixture: enough for the slice budget to split the section
 into four with a short tail.
 */
const VERSE_LINES = 50;

/**
 Stanza the verse fixture repeats, source side.
 */
const VERSE_SOURCE: readonly string[] = [
  '猫猫走过屋顶',
  '月亮照在窗台',
  '风吹过树梢',
  '夜里没有声音',
  '猫猫回到家中',
];

/**
 Stanza the verse fixture repeats, target side.
 */
const VERSE_TARGET: readonly string[] = [
  'The cat walks the roof',
  'The moon lights the sill',
  'Wind moves the branches',
  'No sound in the night',
  'The cat comes home',
];

/**
 Original with front matter declaring a name, and two sections.
 */
const SOURCE_TEXT = `---
name: 猫猫
handle: mao
---

## 第一节

猫猫喜欢追蝴蝶。

猫猫也喜欢晒太阳。

## 第二节

窗台上有一只鸟。
`;

/**
 Translation of the same document, section for section.
 */
const TARGET_TEXT = `---
name: Maomao
handle: mao
---

## Section one

The cat likes chasing butterflies.

The cat also likes sunbathing.

## Section two

There is a bird on the windowsill.
`;

/**
 Byte order mark a page may open with.
 */
const MARK = '\uFEFF';

/**
 Original with metadata and two paragraphs, for the pages that may open with a mark.
 */
const MARKABLE_SOURCE = '---\nname: 猫猫\n---\n\n猫在睡觉。\n\n猫在吃鱼。\n';

/**
 Archive with metadata and the same two paragraphs rendered.
 */
const MARKABLE_ARCHIVE = '---\nname: Mittens\n---\n\nThe cat naps.\n\nThe cat eats fish.\n';

/**
 Archive carrying the paragraphs and no metadata, which the lanes insert it into.
 */
const MARKABLE_ARCHIVE_WITHOUT_METADATA = 'The cat naps.\n\nThe cat eats fish.\n';

/**
 Metadata a lane writes over the front matter slice.
 */
const REWRITTEN_METADATA = '---\nname: Whiskers\n---\n';

/**
 Slices a pair prepares and the page assembled by rewriting the metadata slice.

 @param sourceText - original page, opening with the mark or not

 @param targetText - archive page, opening with the mark or not

 @returns Every slice's texts in order, whether every slice's offsets index
 the texts as written, and the assembled page

 @example
 ```ts
 const { texts, assembled, } = assembleRewrittenMetadata({ sourceText, targetText, },);
 ```
 */
function assembleRewrittenMetadata(
  {
    sourceText,
    targetText,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
  },
): {
  readonly texts: readonly (readonly string[])[];
  readonly offsetsIndexText: boolean;
  readonly assembled: string;
} {
  /**
   Slices prepared for the pair.
   */
  const { slices, } = prepareDocumentPair({
    sourceText,
    targetText,
  },);

  /**
   The metadata slice, which the pair always carries here.
   */
  const metadata = slices.find(function isMetadata(slice,): boolean {
    return slice.syntax === 'front-matter';
  },);
  if (metadata === undefined)
    throw new Error('the original declares metadata, so the pair has a metadata slice',);

  return {
    texts: slices.map(function toTexts(slice,): readonly string[] {
      return [slice.source.text, slice.target.text,];
    },),
    offsetsIndexText: slices.every(function indexes(slice,): boolean {
      return (sourceText.slice(
        slice.source.startOffset,
        slice.source.endOffset,
      ) === slice.source.text) && (targetText.slice(
        slice.target.startOffset,
        slice.target.endOffset,
      ) === slice.target.text);
    },),
    assembled: spliceSlices({
      targetText,
      slices,
      replacements: [{ sliceIndex: metadata.target.sliceIndex, replacementText: REWRITTEN_METADATA, },],
    },),
  };
}

await describe({
  name: prepareDocumentPair.name,
  children: [
    it({
      name: 'pairs the sections and carries COMPLETE chunk spans, offsets and '
        + 'text on both sides, since a lane reading only the text would splice '
        + 'its result back over the wrong span',
      fn: async () => {
        const { slices, alignmentPairCount, } = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
        },);
        expect(alignmentPairCount,).toBeGreaterThan(0,);
        expect(slices.length,).toBeGreaterThanOrEqual(alignmentPairCount,);
        for (const slice of slices) {
          expect(SOURCE_TEXT.slice(
            slice.source
              .startOffset,
            slice.source
              .endOffset,
          ),).toBe(slice.source
            .text,);
          expect(TARGET_TEXT.slice(
            slice.target
              .startOffset,
            slice.target
              .endOffset,
          ),).toBe(slice.target
            .text,);
        }
      },
    },),

    it({
      name: 'PUTS VISIBLE FRONT MATTER IN EXPLICIT SLICE ZERO before body slicing',
      fn: async () => {
        const { slices, } = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
        },);
        expect(slices.at(0,)?.syntax,).toBe('front-matter',);
        expect(slices.at(0,)?.source.text,).toBe('---\nname: 猫猫\nhandle: mao\n---\n',);
        expect(slices.at(0,)?.target.text,).toBe('---\nname: Maomao\nhandle: mao\n---\n',);
        expect(slices.at(1,)?.syntax,).toBeUndefined();
      },
    },),

    it({
      name: 'WRITES A LEADING MARK WHERE THE ARCHIVE HAD IT AND NOWHERE ELSE, whether the original, the archive or both '
        + 'open with one and whether the archive has metadata, so a pair opening with a mark slices to the same texts '
        + 'and assembles to the mark before what the pair without one assembles to',
      fn: async () => {
        for (
          const archive of [MARKABLE_ARCHIVE, MARKABLE_ARCHIVE_WITHOUT_METADATA,]
        ) {
          /**
           What the pair without any mark prepares and assembles to.
           */
          const plain = assembleRewrittenMetadata({
            sourceText: MARKABLE_SOURCE,
            targetText: archive,
          },);
          /**
           Original and archive of each pairing of marks.
           */
          const pairs: readonly (readonly [string, string,])[] = [
            [MARK + MARKABLE_SOURCE, MARK + archive,],
            [MARK + MARKABLE_SOURCE, archive,],
            [MARKABLE_SOURCE, MARK + archive,],
          ];
          for (const [sourceText, targetText,] of pairs) {
            /**
             The same pair with the marks its pages open with.
             */
            const marked = assembleRewrittenMetadata({
              sourceText,
              targetText,
            },);
            expect(marked.texts,).toEqual(plain.texts,);
            expect(marked.offsetsIndexText,).toBe(true,);
            expect(marked.assembled,).toBe(
              targetText.startsWith(MARK,) ? `${MARK}${plain.assembled}` : plain.assembled,
            );
          }
        }
      },
    },),

    it({
      name: 'MAKES NO METADATA SLICE UNDER THE ARCHIVE\'S AUTHORITY, the owner\'s rule of 2026-09-08, marking '
        + 'the preparation without the legacy identity, so the body slices start at zero and the archive\'s '
        + 'front matter ships as it stands',
      fn: async () => {
        const prepared = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          frontMatterAuthority: 'archive',
        },);
        expect(prepared.slices.some(function isMetadata(slice,): boolean {
          return slice.syntax === 'front-matter';
        },),).toBe(false,);
        expect(prepared.slices.at(0,)?.target.sliceIndex,).toBe(0,);
        expect(prepared.frontMatterAuthority,).toBe('archive',);
        expect(prepared.legacyIdentity,).toBeUndefined();
        /**
         The same pair rendered, for the count the archive's authority removes.
         */
        const rendered = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
        },);
        expect(rendered.slices.length,).toBe(prepared.slices.length + 1,);
        expect(rendered.frontMatterAuthority,).toBeUndefined();
      },
    },),

    it({
      name: 'indexes slices GLOBALLY and in document order, which is what lets '
        + 'a cached outcome and a spliced replacement name the same slice. '
        + 'Per-section indexes would collide the moment any section subdivided',
      fn: async () => {
        const { slices, } = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
        },);
        expect(slices.map(function toIndex(slice,): number {
          return slice.target
            .sliceIndex;
        },),).toEqual(slices.map(function toPosition(
          _unused,
          position,
        ): number {
          return position;
        },),);
      },
    },),

    it({
      name: 'collects declared names from BOTH sides, which no slice text '
        + 'carries: front matter is document-level, so this is the only path by '
        + 'which a declared correspondence reaches a prompt',
      fn: async () => {
        const { identityContext, } = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
        },);
        expect(identityContext,).toContain('猫猫',);
        expect(identityContext,).toContain('Maomao',);
      },
    },),

    it({
      name: 'carries the COMMUNITY TERMS the source holds into the identity context, with the '
        + 'archive\'s rendering, and none for a source holding no term (owner, 2026-09-09)',
      fn: async () => {
        const { identityContext, } = prepareDocumentPair({
          sourceText: `${SOURCE_TEXT}\n\n小猫最爱的玩偶是超天酱。\n`,
          targetText: `${TARGET_TEXT}\n\nHer favourite plush toy was KAngel.\n`,
        },);
        expect(identityContext,).toContain('COMMUNITY TERMS',);
        expect(identityContext,).toContain('- 超天酱: "KAngel", "Needy Streamer Overload" (',);
        const { identityContext: plain, } = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
        },);
        expect((plain ?? '').includes('COMMUNITY TERMS',),).toBe(false,);
      },
    },),

    it({
      name: 'PRESERVES TARGET CONTRIBUTOR HANDLES as declared identity forms',
      fn: async () => {
        const prepared = prepareDocumentPair({
          sourceText: '## 简介\n\n猫睡着了。\n\n条目贡献：一体, 小雪\n',
          targetText: '## Description\n\nThe cat slept.\n\nContributors for this entry: One Body, [Snow](https://example.test/snow)\n',
        },);
        expect(prepared.declaredNames,).toContain('One Body',);
        expect(prepared.declaredNames,).toContain('Snow',);
        expect(prepared.declaredNames,).not.toContain('小雪',);
      },
    },),

    it({
      name: 'NAMES THE SUBJECT\'S OWN DECLARED FORMS apart from the contributor handles, so a floor can '
        + 'tell another form of the person from another person (XingZ60 slice 80)',
      fn: async () => {
        const prepared = prepareDocumentPair({
          sourceText: '---\nname: 猫猫\ninfo:\n    alias: 咪咪, Z60\n---\n\n## 简介\n\n猫睡着了。\n\n条目贡献：一体, 小雪\n',
          targetText: '---\nname: Maomao\ninfo:\n    alias: Mimi, Z60\n---\n\n## Description\n\nThe cat slept.\n\n'
            + 'Contributors for this entry: One Body, [Snow](https://example.test/snow)\n',
        },);
        expect(prepared.declaredSubjectForms,).toEqual(['Maomao', 'Mimi', 'Z60',],);
        expect(prepared.declaredNames,).toContain('One Body',);
        expect(prepared.declaredSubjectForms,).not.toContain('One Body',);
      },
    },),

    it({
      name: 'OMITS the identity block when neither side declares anything, '
        + 'rather than passing an empty one, so a prompt never carries a '
        + 'heading with nothing under it',
      fn: async () => {
        expect(prepareDocumentPair({
          sourceText: '猫猫喜欢追蝴蝶。\n',
          targetText: 'The cat likes chasing butterflies.\n',
        },).identityContext,).toBeUndefined();
      },
    },),

    it({
      name: 'reports alignment findings in scorecard-stable wording when the '
        + 'two sides disagree structurally, since those findings are how a '
        + 'wrongly paired document is recognised afterwards',
      fn: async () => {
        const { alignmentFindings, } = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          // One section short, which is the shape that made a whole document
          // pair off by one in the corpus.
          targetText: `---
name: Maomao
handle: mao
---

## Section one

The cat likes chasing butterflies.

The cat also likes sunbathing.
`,
        },);
        expect(alignmentFindings.length,).toBeGreaterThan(0,);
        for (const finding of alignmentFindings)
          expect(finding.startsWith('alignment ',),).toBe(true,);
      },
    },),

    it({
      name: 'RENDERS A REFUSAL WITHOUT THE WORD PAIR, because the number in a '
        + 'refusal is an index on the refused chunk\'s OWN side while the number '
        + 'in a target-unclaimed finding is a real pair index. Rendered the same '
        + 'way, a reader meeting one could not tell which numbering it counted '
        + 'in, and would be sent to the wrong section of the document',
      fn: async () => {
        const { alignmentFindings, } = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          // One section short, the shape that refuses rather than pairs.
          targetText: `---
name: Maomao
handle: mao
---

## Section one

The cat likes chasing butterflies.

The cat also likes sunbathing.
`,
        },);

        /**
         Findings the ALIGNER wrote, which are the ones that reported a side
         index under the pair wording.
         */
        const mismatches = alignmentFindings
          .filter(function isMismatch(finding,): boolean {
            return finding.includes('structure-mismatch',);
          },);

        expect(mismatches.length,).toBeGreaterThan(0,);
        for (const finding of mismatches)
          expect(finding.includes('(pair ',),).toBe(false,);
      },
    },),
    it({
      name: 'EXPOSES TARGET BLOCKS THE ROSTER LEFT UNCLAIMED as structured offsets, because publication safety must not parse diagnostic prose to find inherited wording no quality stage reviewed',
      fn: async () => {
        const prepared = prepareDocumentPair({
          sourceText: 'Cats nap.',
          targetText: 'Cats nap.\n\nAn unclaimed aside.',
          blockPairings: new Map([[
            0,
            [{ source: 0, target: 0, },],
          ],]),
        },);

        expect(prepared.unclaimedTargetBlocks,).toEqual([{
          location: {
            kind: 'aligned-pair',
            pairIndex: 0,
          },
          blockId: 'block/1',
          startOffset: 11,
          endOffset: 30,
        },],);
      },
    },),

    it({
      name: 'CARRIES CORROBORATED PARAGRAPH SPLITS AND MERGES THROUGH SLICING because block wire permits repeated position on either side',
      fn: async () => {
        const split = prepareDocumentPair({
          sourceText: 'Cats nap.',
          targetText: 'Cats nap.\n\nThey dream.',
          blockPairings: new Map([[
            0,
            [
              { source: 0, target: 0, },
              { source: 0, target: 1, },
            ],
          ],]),
        },);
        const merged = prepareDocumentPair({
          sourceText: 'Cats nap.\n\nThey dream.',
          targetText: 'Cats nap and dream.',
          blockPairings: new Map([[
            0,
            [
              { source: 0, target: 0, },
              { source: 1, target: 0, },
            ],
          ],]),
        },);

        expect(split.slices,).toHaveLength(1);
        expect(split.slices[0]?.target.text,).toContain('They dream.');
        expect(merged.slices,).toHaveLength(1);
        expect(merged.slices[0]?.source.text,).toContain('They dream.');
      },
    },),

    it({
      name: 'REVIEWS TARGET-ONLY PROSE THROUGH DETERMINISTIC SLICING when no roster pairing exists, rather than misclassifying folded context as unclaimed bypass',
      fn: async () => {
        const prepared = prepareDocumentPair({
          sourceText: 'Cats nap.',
          targetText: 'Cats nap.\n\nAn adjacent archive aside.',
        },);

        expect(prepared.unclaimedTargetBlocks,).toEqual([]);
        expect(prepared.slices.some(function carriesAside(slice,): boolean {
          return slice.target.text.includes('archive aside',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'EXPOSES WHOLE TARGET SECTION OUTSIDE ALIGNMENT because no slice can review its archive wording',
      fn: async () => {
        const prepared = prepareDocumentPair({
          sourceText: '## Cats\n\nCats nap.',
          targetText: '## Cats\n\nCats nap.\n\n## Aside\n\nArchive-only section.',
        },);

        expect(prepared.unclaimedTargetBlocks.some(function isTargetSection(block,): boolean {
          return block.location.kind === 'target-section';
        },),).toBe(true,);
      },
    },),

    it({
      name: 'GIVES THE SAME SLICING EVERY TIME for one pair, which is the '
        + 'whole reason preparation is shared: two lanes that sliced separately '
        + 'would drift the moment either changed a budget, and each would still '
        + 'report slices that look right on their own',
      fn: async () => {
        expect(prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
        },),).toEqual(prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
        },),);
      },
    },),

    it({
      name: 'inherits the line-structure verdict from the enclosing CHUNK, so '
        + 'a verse section subdividing into short slices does not lose the rule '
        + 'on the slices that need it most',
      fn: async () => {
        // FIFTY BLANK-LINE-SEPARATED LINES, the shape the predicate reads, long
        // enough that the slice budget splits the one section into four and the
        // last slice holds too few lines to read as verse on its own. Until
        // 2026-09-27 this case used five lines joined by single newlines: one
        // block, one slice, never line-structured, so its loop ran zero times
        // (ledger T1).
        /**
         Stanza lines the fixture repeats, source and target in step.
         */
        const lines = Array.from({ length: VERSE_LINES, }, function lineAt(
          _unused,
          index,
        ): {
          readonly source: string;
          readonly target: string;
        } {
          /**
           Which of the five stanza lines this one repeats.
           */
          const stanza = index % VERSE_SOURCE.length;
          return {
            source: `${VERSE_SOURCE[stanza] ?? ''}${String(index,)}`,
            target: `${VERSE_TARGET[stanza] ?? ''} ${String(index,)}`,
          };
        },);
        const { slices, lineStructuredSliceIndices, } = prepareDocumentPair({
          sourceText: `## 诗\n\n${
            lines
              .map(function sourceOf({ source, },): string {
                return source;
              },)
              .join('\n\n',)
          }\n`,
          targetText: `## Verse\n\n${
            lines
              .map(function targetOf({ target, },): string {
                return target;
              },)
              .join('\n\n',)
          }\n`,
        },);
        /**
         Slices whose own text is not line-structured.
         */
        const notVerseAlone = slices.filter(function readsAsProse(slice,): boolean {
          return !isLineStructured({ text: slice.source.text, },);
        },);
        expect(slices.length,).toBeGreaterThan(1,);
        // THE POSITIVE CONTROL: some slice is governed only by its chunk.
        expect(notVerseAlone.length,).toBeGreaterThan(0,);
        expect(lineStructuredSliceIndices.size,).toBe(slices.length,);
        for (const slice of slices)
          expect(lineStructuredSliceIndices.has(slice.target.sliceIndex,),).toBe(true,);
      },
    },),

    it({
      name: 'KEEPS THE SCORER for a section the roster agreed nothing for',
      fn: async () => {
        // ONE section, long enough that the 400-character budget subdivides it.
        // The shared fixture cannot show this: its sections each fit in a
        // single slice, so a collapsed section and a properly sliced one look
        // identical and the case would pass however the code read an empty map.
        const source = '## 一节\n\n'
          + `${'猫猫在窗台上看鸟，看了整整一个下午，尾巴一直轻轻摆动。'.repeat(3,)}\n\n`
          + `${'傍晚的时候，猫猫跳下窗台，走到门口等着有人回家。'.repeat(3,)}\n\n`
          + `${'第二天早上，猫猫又回到窗台，那只鸟已经不在那里了。'.repeat(3,)}\n`;
        const target = '## One\n\n'
          + `${'The cat watched the birds from the sill all afternoon, her tail moving. '.repeat(3,)}\n\n`
          + `${'Towards evening she jumped down and waited by the door for someone. '.repeat(3,)}\n\n`
          + `${'Next morning she returned to the sill, and the bird had gone. '.repeat(3,)}\n`;

        /**
         Reads a slicing as the spans it carries, which is what a later stage
         sees; counts alone would pass a slicing that moved every boundary.

         @param prepared - preparation to read

         @returns One span per slice, in slice order

         @example
         ```ts
         const spans = spansOf(prepared,);
         ```
         */
        function spansOf(prepared: ReturnType<typeof prepareDocumentPair>,): readonly string[] {
          return prepared.slices
            .map(function toSpan(slice,): string {
              return `${String(slice.target.startOffset,)}..${String(slice.target.endOffset,)}`;
            },);
        }

        /**
         How the deterministic aligner slices this pair.
         */
        const scorer = prepareDocumentPair({
          sourceText: source,
          targetText: target,
        },);

        // POSITIVE CONTROL, and the reason the fixture is this long: the ONE
        // section has to subdivide, or a collapse into a single slice is
        // indistinguishable from correct slicing.
        expect(scorer.slices.length,).toBeGreaterThan(1,);

        // What the roster agreeing nothing for a section produces: a map that
        // is present and holds no entry for it. The pairing stage records that
        // as falling back to scoring, so it has to slice exactly as the scorer
        // does rather than reading the miss as an empty pairing.
        const unpaired = prepareDocumentPair({
          sourceText: source,
          targetText: target,
          blockPairings: new Map(),
        },);

        expect(spansOf(unpaired,),).toStrictEqual(spansOf(scorer,),);
      },
    },),
    it({
      name:
        'ECHOES the section pairing it consumed and OMITS the field when the deterministic aligner decided, '
          + 'so an artifact records the value slicing was built on rather than a copy assembled beside it',
      fn: async () => {
        /**
         Prepared by the deterministic aligner, which supplies no pairing.
         */
        const deterministic = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
        },);
        expect(Object.hasOwn(
          deterministic,
          'sectionPairing',
        ),).toBe(false,);

        /**
         Pairing a caller supplies, crossing the sections on purpose so the
         echo cannot be mistaken for the aligner's own index order.
         */
        const supplied = [
          {
            source: 0,
            target: 1,
          },
          {
            source: 1,
            target: 1,
          },
        ].slice(
          0,
          1,
        );

        /**
         Prepared on that pairing.
         */
        const paired = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          sectionPairing: supplied,
        },);
        expect(paired.sectionPairing,).toEqual(supplied,);

        // POSITIVE CONTROL: the supplied pairing has to change the slicing, or
        // the echo would be the only visible difference and a preparation that
        // ignored the pairing while echoing it would pass.
        expect(paired.slices.length,).not
          .toBe(deterministic.slices.length,);
      },
    },),

    it({
      name: 'SLICES THE WHOLE ORIGINAL AS INSERTION ANCHORS when the archive carries no body, whether it is empty or '
        + 'only a byte order mark, so an archive with nothing in it ships the original under a note instead of an '
        + 'empty page, and the layout the preparation asserts accepts every anchor at the one offset',
      fn: async () => {
        /**
         Byte order mark a page may open with.
         */
        const mark = String.fromCodePoint(0xFEFF,);
        for (
          const [targetText, offset,] of [
            ['', 0,],
            [mark, 1,],
          ] as const
        ) {
          for (const frontMatterAuthority of ['rendered', 'archive',] as const) {
            /**
             Preparation of the original against an archive with no body.
             */
            const { slices, } = prepareDocumentPair({
              sourceText: SOURCE_TEXT,
              targetText,
              frontMatterAuthority,
            },);
            /**
             Slices carrying the body, apart from the metadata slice.
             */
            const body = slices.filter(function isBody(slice,): boolean {
              return slice.syntax !== 'front-matter';
            },);

            expect(body.length,).toBeGreaterThanOrEqual(2,);
            expect(slices.map(function toAnchoredOffset(slice,): number | undefined {
              return isInsertionChunk(slice.target,) ? slice.target.startOffset : undefined;
            },),).toEqual(slices.map(function toOffset(): number {
              return offset;
            },),);
            // EVERY NODE OF THE ORIGINAL'S BODY IS IN A SLICE, or a passage would be dropped from the page.
            for (const node of parseDocument({ text: SOURCE_TEXT, },).nodes) {
              expect(
                body.some(function carries(slice,): boolean {
                  return slice.source.text.includes(node.text,);
                },),
              ).toBe(true,);
            }
            assertPlacementLayout({
              slices,
              targetText,
            },);
          }
        }
      },
    },),
  ],
},);
