/**
 Tests for paragraph-bound slice subdivision:
 a generous budget keeps the section whole, a small budget groups
 paragraphs without ever splitting one, coverage is contiguous with
 exact document bytes, mismatched paragraph counts merge
 monotonically, and the global base index lands on every slice.
 Fixtures are cat-themed invention mirroring corpus structure only.

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
  makeInsertionChunk,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  alignDocumentSections,
  chunkByHeadings,
  type ChunkPair,
} from '@monochromatic-dev/translation-repair-slice-reading';
import { subdivideChunkPair, } from '../dist/final/node/index.mjs';

/**
 Original with one section of three short paragraphs.
 */
const SOURCE_TEXT = `## 猫的一天

小猫早晨在窗台晒太阳。

小猫中午在垫子上打盹。

小猫晚上在院子里追蝴蝶。
`;

/**
 Translation mirroring the section with three paragraphs.
 */
const TARGET_TEXT = `## A cat's day

The kitten suns on the windowsill each morning.

The kitten naps on its cushion at noon.

The kitten chases butterflies in the yard at night.
`;

/**
 Translation collapsing the three source paragraphs into two.
 */
const TARGET_TWO_PARAGRAPHS = `## A cat's day

The kitten suns on the windowsill each morning and naps on its cushion at noon.

The kitten chases butterflies in the yard at night.
`;

/**
 Builds a marked paragraph of an exact character length: an `M{index}`
 tag that both sides' corresponding paragraphs carry, padded to size with
 cat-themed filler. Equal-length marker tags let a slice's source and
 target marker sets be compared for drift.
 */
function markedParagraph(
  {
    index,
    size,
  }: {
    readonly index: number;
    readonly size: number;
  },
): string {
  /**
   Marker tag matching paragraphs share across the two sides.
   */
  const tag = `M${String(index,)} `;
  return tag
    + 'cat '
      .repeat(Math.ceil((size - tag.length) / 'cat '.length,),)
      .slice(
      0,
      size - tag.length,
      );
}

/**
 Joins marked paragraphs of the given span sizes into one document, so a
 source and target built from equal-length size lists share paragraph
 count and marker order while diverging in per-paragraph length.
 */
function markedDocument(
  { sizes, }: { readonly sizes: readonly number[]; },
): string {
  return `${sizes
    .map(function toParagraph(
      size,
      index,
    ) {
      return markedParagraph({ index, size, },);
    },)
    .join('\n\n',)}\n`;
}

/**
 Marker indices the equal-count lockstep fixtures carry, in order.
 */
const LOCKSTEP_MARKERS = [
  0,
  1,
  2,
  3,
  4,
  5,
];

/**
 Extracts the ordered marker tags a slice side contains, by substring
 presence so no inline regex is needed.
 */
function sliceMarkers(
  { text, }: { readonly text: string; },
): string {
  return LOCKSTEP_MARKERS
    .filter(function present(index,) {
      return text.includes(`M${String(index,)} `,);
    },)
    .map(function toTag(index,) {
      return `M${String(index,)}`;
    },)
    .join(',',);
}

/**
 Builds the single aligned section pair of a fixture document pair.
 */
function alignedPair(
  {
    source,
    target,
  }: {
    readonly source: string;
    readonly target: string;
  },
): ChunkPair {
  /**
   Alignment over the parsed fixture pair.
   */
  const alignment = alignDocumentSections({
    source: parseDocument({ text: source, },),
    target: parseDocument({ text: target, },),
  },);

  /**
   Only pair of the single-section fixtures.
   */
  const [pair,] = alignment.pairs;
  if (pair === undefined)
    throw new Error('fixture must align into exactly one section pair',);
  return pair;
}

//region Insertion subdivision
// A section nothing rendered has no target runs to frame subdivision by, and
// used to come back as ONE slice however long its original was.

/**
 Original section of eight short paragraphs, none of them near the budget on
 its own, together well past it: 330 characters over eight blocks. This is the corpus shape: `XingZ60`'s two
 unrendered sections hold 6 and 23 blocks whose largest member is 384
 characters, against a budget of 400.
 */
const UNRENDERED_SOURCE_TEXT = `## 猫猫的一天

${
  [
    '小猫早晨在窗台上晒太阳，尾巴一摇一摇，看着院子里的麻雀发呆，直到太阳升高才慢慢站起来伸个懒腰。',
    '她跳下窗台，先去水碗边喝了几口水，又绕着桌腿走了两圈，像是在巡视自己的领地一样认真。',
    '中午的时候她在沙发上睡着了，四只爪子朝天，肚皮一起一伏，谁走过去都不肯睁眼看一下。',
    '下午她发现了一只飞进屋里的蝴蝶，追着它从客厅跑到卧室，撞翻了一个空纸盒也毫不在意。',
    '蝴蝶从窗缝飞走以后，她在窗台上又坐了很久，尾巴不停地拍打着玻璃，像是在抱怨什么。',
    '傍晚她听见门口的脚步声，立刻跑到门边等着，耳朵竖得笔直，喉咙里发出很轻的呼噜声。',
    '晚饭她吃得很快，吃完还把碗推到墙角，然后回头看了一眼，好像在说这里已经没有东西了。',
    '睡觉前她跳上床，在枕头边转了三圈才躺下，把头埋进被子里，一整夜都没有再动过。',
  ].join('\n\n')
}
`;

/**
 Where in the translation this section's rendering belongs.
 */
const ANCHOR_OFFSET = 120;

/**
 Slice budget these cases measure against.

 SCALED DOWN FROM THE PRODUCTION 400 so the fixture can stay short enough to
 read. What matters is the RATIO of section to budget: this section runs 330
 characters over eight blocks, so at 120 it must split about three ways, which
 is the same pressure `XingZ60`'s 1459-character unrendered section is under
 at 400.
 */
const INSERTION_BUDGET = 120;

//endregion Insertion subdivision

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: subdivideChunkPair.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'keeps one slice under a generous budget',
          fn: async () => {
            const pair = alignedPair({
              source: SOURCE_TEXT,
              target: TARGET_TEXT,
            },);
            const slices = subdivideChunkPair({
              pair,
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
              baseIndex: 0,
              budget: 10_000,
            },);
            expect(slices,).toHaveLength(1,);
            expect(slices[0]?.target
              .startOffset,).toBe(pair.target
                .startOffset,);
            expect(slices[0]?.target
              .endOffset,).toBe(pair.target
                .endOffset,);
          },
        },),

        it({
          name: 'splits paragraphs into contiguous byte-exact slices under a small budget',
          fn: async () => {
            const pair = alignedPair({
              source: SOURCE_TEXT,
              target: TARGET_TEXT,
            },);
            const slices = subdivideChunkPair({
              pair,
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
              baseIndex: 0,
              budget: 50,
            },);
            expect(slices.length,).toBeGreaterThan(1,);

            // Coverage opens and closes on the section's own offsets.
            expect(slices[0]?.target
              .startOffset,).toBe(pair.target
                .startOffset,);
            expect(slices.at(-1,)?.target
              .endOffset,).toBe(pair.target
                .endOffset,);
            for (const [index, slice,] of slices.entries()) {
              // Slice text is the exact document byte range it claims.
              expect(slice.target
                .text,).toBe(TARGET_TEXT.slice(
                  slice.target
                    .startOffset,
                  slice.target
                    .endOffset,
                ),);
              expect(slice.source
                .text,).toBe(SOURCE_TEXT.slice(
                  slice.source
                    .startOffset,
                  slice.source
                    .endOffset,
                ),);

              /**
               Preceding slice for monotone ordering.
               */
              const previous = slices[index - 1];
              if (previous !== undefined) {
                expect(slice.target
                  .startOffset,).toBeGreaterThanOrEqual(previous.target
                    .endOffset,);
              }
            }
          },
        },),

        it({
          name: 'merges the wider side monotonically on paragraph-count mismatch',
          fn: async () => {
            const pair = alignedPair({
              source: SOURCE_TEXT,
              target: TARGET_TWO_PARAGRAPHS,
            },);
            const slices = subdivideChunkPair({
              pair,
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TWO_PARAGRAPHS,
              baseIndex: 0,
              budget: 30,
            },);
            expect(slices.length,).toBeGreaterThan(0,);
            // Both sides stay fully covered despite the mismatch.
            expect(slices[0]?.source
              .startOffset,).toBe(pair.source
                .startOffset,);
            expect(slices.at(-1,)?.source
              .endOffset,).toBe(pair.source
                .endOffset,);
            expect(slices[0]?.target
              .startOffset,).toBe(pair.target
                .startOffset,);
            expect(slices.at(-1,)?.target
              .endOffset,).toBe(pair.target
                .endOffset,);
          },
        },),

        it({
          name: 'pairs equal node counts in lockstep without off-by-one drift (Arita regression)',
          fn: async () => {
            /**
             Dense original: small adjacent nodes merge on odd boundaries.
             */
            const source = markedDocument({ sizes: [35, 10, 10, 35, 10, 10,], },);

            /**
             Longer translation: same paragraph count, mirrored sizes so the
             independent-budget grouping would merge on even boundaries and
             drift the pairing by one (the Arita non-translation false block).
             */
            const target = markedDocument({ sizes: [10, 10, 35, 10, 10, 35,], },);
            const pair = alignedPair({
              source,
              target,
            },);
            // Equal paragraph counts are the lockstep precondition.
            expect(pair.source
              .nodes
              .length,).toBe(pair.target
                .nodes
                .length,);

            const slices = subdivideChunkPair({
              pair,
              sourceText: source,
              targetText: target,
              baseIndex: 0,
              budget: 40,
            },);

            /**
             Ordered markers gathered across every slice's original side.
             */
            const covered: string[] = [];
            for (const slice of slices) {
              // Corresponding paragraphs stay in the same slice: no drift.
              expect(sliceMarkers({ text: slice.source
                .text, },),).toBe(sliceMarkers({ text: slice.target
                  .text, },),);
              covered.push(sliceMarkers({ text: slice.source
                .text, },),);
            }
            // Every marker is covered exactly once, in document order.
            expect(covered.join(',',),).toBe('M0,M1,M2,M3,M4,M5',);
          },
        },),

        it({
          name: 'stamps the global base index onto every slice',
          fn: async () => {
            const pair = alignedPair({
              source: SOURCE_TEXT,
              target: TARGET_TEXT,
            },);
            const slices = subdivideChunkPair({
              pair,
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
              baseIndex: 5,
              budget: 50,
            },);
            for (const [index, slice,] of slices.entries()) {
              expect(slice.target
                .sliceIndex,).toBe(5 + index,);
              expect(slice.source
                .sliceIndex,).toBe(5 + index,);
            }
          },
        },),

        it({
          name: 'stamps the base index on a section whose translation is a bare heading against an original with prose. '
            + 'The pair arrives holding its SECTION index, and a path that returned it untouched let two slices of one '
            + 'document share an index once any earlier section subdivided; slice identity is what the cache key and '
            + 'the splice both rest on',
          fn: async () => {
            /**
             Original whose section has prose against a translation whose
             matching section is a bare heading, which the grouping folds into
             one slice.
             */
            const onlySource = '## 猫的一天\n\n小猫早晨在窗台晒太阳。\n';

            /**
             Translation carrying the heading and nothing under it.
             */
            const emptyTarget = '## A cat\'s day\n';

            /**
             Slices of a section pair one side left empty.
             */
            const slices = subdivideChunkPair({
              pair: alignedPair({
                source: onlySource,
                target: emptyTarget,
              },),
              sourceText: onlySource,
              targetText: emptyTarget,
              baseIndex: 7,
            },);
            expect(slices,).toHaveLength(1,);
            for (const slice of slices) {
              expect(slice.source
                .sliceIndex,).toBe(7,);
              expect(slice.target
                .sliceIndex,).toBe(7,);
            }
          },
        },),
      ],
    },),

    describe({
      name: `${subdivideChunkPair.name} on a section nothing rendered`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SLICES it by the ORIGINAL, since the translation side offers no runs to frame by, and '
            + 'a whole unrendered section asked as one unit is a translation call several times the '
            + 'budget every other slice is held to',
          fn: async () => {
            /**
             Original section as the aligner hands it over.
             */
            const [sourceChunk,] = chunkByHeadings({
              document: parseDocument({ text: UNRENDERED_SOURCE_TEXT, },),
            },);
            if (sourceChunk === undefined)
              throw new Error('the fixture should parse to one section',);

            /**
             Slices this section subdivides into.
             */
            const slices = subdivideChunkPair({
              pair: {
                source: sourceChunk,
                target: makeInsertionChunk({
                  sliceIndex: 0,
                  offset: ANCHOR_OFFSET,
                },),
              },
              sourceText: UNRENDERED_SOURCE_TEXT,
              targetText: UNRENDERED_SOURCE_TEXT,
              baseIndex: 0,
              budget: INSERTION_BUDGET,
            },);
            expect(slices.length,).toBeGreaterThan(1,);

            // NO PARAGRAPH SPLIT, so every slice stays inside the budget unless one
            // block alone exceeds it, which this fixture has none of.
            for (const slice of slices)
              expect(slice.source
                .text
                .length,).toBeLessThanOrEqual(INSERTION_BUDGET,);
          },
        },),

        it({
          name: 'KEEPS every original block, in order, so slicing a section nobody translated cannot '
            + 'lose part of the passage it exists to write',
          fn: async () => {
            const [sourceChunk,] = chunkByHeadings({
              document: parseDocument({ text: UNRENDERED_SOURCE_TEXT, },),
            },);
            if (sourceChunk === undefined)
              throw new Error('the fixture should parse to one section',);
            const slices = subdivideChunkPair({
              pair: {
                source: sourceChunk,
                target: makeInsertionChunk({
                  sliceIndex: 0,
                  offset: ANCHOR_OFFSET,
                },),
              },
              sourceText: UNRENDERED_SOURCE_TEXT,
              targetText: UNRENDERED_SOURCE_TEXT,
              baseIndex: 0,
              budget: INSERTION_BUDGET,
            },);
            expect(slices.flatMap(function toIds(slice,) {
              return slice.source
                .nodes
                .map(function toId(node,) {
                  return node.id;
                },);
            },),).toStrictEqual(sourceChunk.nodes
              .map(function toId(node,) {
                return node.id;
              },),);
          },
        },),

        it({
          name: 'WRITES every slice at the SAME boundary and stamps them in order, which is the shape '
            + '`spliceSlices` orders several insertions at one offset by. The section has one place to '
            + 'go, and its slices go there one after another',
          fn: async () => {
            const [sourceChunk,] = chunkByHeadings({
              document: parseDocument({ text: UNRENDERED_SOURCE_TEXT, },),
            },);
            if (sourceChunk === undefined)
              throw new Error('the fixture should parse to one section',);
            const slices = subdivideChunkPair({
              pair: {
                source: sourceChunk,
                target: makeInsertionChunk({
                  sliceIndex: 0,
                  offset: ANCHOR_OFFSET,
                },),
              },
              sourceText: UNRENDERED_SOURCE_TEXT,
              targetText: UNRENDERED_SOURCE_TEXT,
              baseIndex: 7,
              budget: INSERTION_BUDGET,
            },);
            for (const [at, slice,] of slices.entries()) {
              expect(isInsertionChunk(slice.target,),).toBe(true,);
              expect(slice.target
                .startOffset,).toBe(ANCHOR_OFFSET,);
              expect(slice.target
                .endOffset,).toBe(ANCHOR_OFFSET,);
              expect(slice.source
                .sliceIndex,).toBe(7 + at,);
              expect(slice.target
                .sliceIndex,).toBe(7 + at,);
            }
          },
        },),
      ],
    },),
  ],
},);
