/**
 Tests for the choice of section the translate probe demonstrates on: how
 much of a section's source blocks the translation carries, and which section
 carries least.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  alignDocumentSections,
  type ChunkPair,
} from '@monochromatic-dev/translation-repair-slice-reading';
import {
  coverageOf,
  sparsestPair,
} from '../../dist/final/node/index.mjs';

/**
 Original of two sections, the first holding three blocks and the second two.
 */
const SOURCE_PAGE = '## 一\n\n猫一。\n\n猫二。\n\n## 二\n\n猫三。\n';

/**
 Translation of two sections of two blocks each.
 */
const TARGET_PAGE = '## One\n\nCat one.\n\n## Two\n\nCat three.\n';

/**
 Original of two sections of two blocks each.
 */
const EVEN_SOURCE_PAGE = '## 一\n\n猫一。\n\n## 二\n\n猫二。\n';

/**
 Translation of two sections of two blocks each.
 */
const EVEN_TARGET_PAGE = '## One\n\nCat one.\n\n## Two\n\nCat two.\n';

/**
 Aligned section pairs of two pages.

 @param sourceText - original

 @param targetText - translation

 @returns Pairs the aligner commits to

 @example
 ```ts
 const pairs = pairsOf({ sourceText, targetText, },);
 ```
 */
function pairsOf(
  {
    sourceText,
    targetText,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
  },
): readonly ChunkPair[] {
  return alignDocumentSections({
    source: parseDocument({ text: sourceText, },),
    target: parseDocument({ text: targetText, },),
  },).pairs;
}

/**
 A pair with its original's blocks taken away, which only a section the
 original never held looks like.

 @param pair - pair to empty on the original side

 @returns Same pair with no source block

 @example
 ```ts
 const target-only = withoutSource({ pair, },);
 ```
 */
function withoutSource({ pair, }: { readonly pair: ChunkPair; },): ChunkPair {
  return {
    ...pair,
    source: {
      ...pair.source,
      nodes: [],
    },
  };
}

/**
 Original text of the pair a choice found.

 @param choice - what the pick made

 @returns Pair's original, or `none` where nothing was found

 @example
 ```ts
 const text = textOf({ choice, },);
 ```
 */
function textOf({ choice, }: { readonly choice: ReturnType<typeof sparsestPair>; },): string {
  if (choice.kind === 'none')
    return 'none';
  return choice.pair
    .source
    .text;
}

await describe({
  name: 'translate-probe-coverage',
  children: [
    describe({
      name: coverageOf.name,
      children: [
        it({
          name: 'DIVIDES the translation\'s blocks by the original\'s',
          fn: async () => {
            expect(coverageOf({ pair: pairsOf({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
            },)[0] as ChunkPair, },),).toBe(2 / 3,);
          },
        },),
        it({
          name: 'DIVIDES by one where the original holds no block, so the ratio is the translation\'s own count',
          fn: async () => {
            expect(coverageOf({ pair: withoutSource({ pair: pairsOf({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
            },)[0] as ChunkPair, }, ), },),).toBe(2,);
          },
        },),
      ],
    },),
    describe({
      name: sparsestPair.name,
      children: [
        it({
          name: 'PICKS the section with the lowest coverage',
          fn: async () => {
            expect(textOf({ choice: sparsestPair({ pairs: pairsOf({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
            },), },), },),).toBe('## 一\n\n猫一。\n\n猫二。',);
          },
        },),
        it({
          name: 'PICKS the first of sections that cover alike',
          fn: async () => {
            expect(textOf({ choice: sparsestPair({ pairs: pairsOf({
              sourceText: EVEN_SOURCE_PAGE,
              targetText: EVEN_TARGET_PAGE,
            },), },), },),).toBe('## 一\n\n猫一。',);
          },
        },),
        it({
          name: 'PASSES OVER a section whose original holds no block, however little it covers',
          fn: async () => {
            /**
             Pairs of which the first has no original.
             */
            const pairs = pairsOf({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
            },);
            expect(textOf({ choice: sparsestPair({ pairs: [
              withoutSource({ pair: pairs[0] as ChunkPair, },),
              pairs[1] as ChunkPair,
            ], },), },),).toBe('## 二\n\n猫三。',);
          },
        },),
        it({
          name: 'PICKS nothing from no pair, or from pairs none of which holds an original block',
          fn: async () => {
            expect(sparsestPair({ pairs: [], },),).toEqual({ kind: 'none', },);
            expect(sparsestPair({ pairs: pairsOf({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
            },)
              .map(function emptied(pair,): ChunkPair {
                return withoutSource({ pair, },);
              },), },),).toEqual({ kind: 'none', },);
          },
        },),
      ],
    },),
  ],
},);
