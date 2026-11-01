/**
 Guards ledger A4 (owner, 2026-09-27, "Archive's English"): where the archive
 gave a link its own destination (an English page for a Chinese one), a lane
 that wrote the original's destination back is pointed at the archive's
 again at page assembly. A swap of destinations both sides carry, and an
 original destination the archive replaced two ways, are left alone.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  replacedDestinations,
  restoreArchiveDestinations,
} from '../../dist/final/node/index.mjs';

/**
 Original destination of the cat-scratch article.
 */
const ORIGINAL_HREF = 'https://zh.example.org/wiki/cat-scratch';

/**
 Archive's English destination for the same article.
 */
const ARCHIVE_HREF = 'https://en.example.org/wiki/Cat-scratch_disease';

/**
 One slice pairing an original text with an archive text.

 @param sliceIndex - where the slice stands

 @param source - original text

 @param target - archive text

 @param startOffset - where the archive text starts in the page

 @returns Prepared pair

 @example
 ```ts
 const slice = pair({ sliceIndex: 0, source: '猫。', target: 'Cat.', startOffset: 0, },);
 ```
 */
function pair(
  {
    sliceIndex,
    source,
    target,
    startOffset,
  }: {
    readonly sliceIndex: number;
    readonly source: string;
    readonly target: string;
    readonly startOffset: number;
  },
): ChunkPair {
  return {
    source: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: source.length,
      text: source,
    },
    target: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset,
      endOffset: startOffset + target.length,
      text: target,
    },
  };
}

/**
 Slice 1's text after the pass, where slice 0 carries the link the archive
 localized and a lane wrote slice 1.

 @param replacement - what a lane wrote for slice 1

 @returns Slice 1's text after the pass

 @example
 ```ts
 secondSlice({ replacement: `See [cat scratch](${ORIGINAL_HREF}).`, },);
 ```
 */
function secondSlice({ replacement, }: { readonly replacement: string; },): string {
  /**
   Archive's first slice, carrying its English destination.
   */
  const archive = `The cat had [cat-scratch disease](${ARCHIVE_HREF}).`;

  /**
   The page after the pass.
   */
  const page = restoreArchiveDestinations({
    slices: [
      pair({
        sliceIndex: 0,
        source: `猫得了[猫抓病](${ORIGINAL_HREF})。`,
        target: archive,
        startOffset: 0,
      },),
      pair({
        sliceIndex: 1,
        source: `医生也提到了[猫抓病](${ORIGINAL_HREF})。`,
        target: 'The vet mentioned it too.',
        startOffset: archive.length + 2,
      },),
    ],
    replacements: [{ sliceIndex: 1, replacementText: replacement, },],
    archiveOriginalSpans: [],
  },);
  return page.replacements
    .find(function isSecond(row,): boolean {
      return row.sliceIndex === 1;
    },)
    ?.replacementText ?? replacement;
}

await describe({
  name: 'restoreArchiveDestinations (ledger A4)',
  children: [
    it({
      name: 'POINTS A LANE\'S ORIGINAL DESTINATION BACK AT THE ARCHIVE\'S, with or without a link title',
      fn: async () => {
        expect(secondSlice({
          replacement: `The vet mentioned [cat scratch fever](${ORIGINAL_HREF}) too.`,
        },),).toBe(`The vet mentioned [cat scratch fever](${ARCHIVE_HREF}) too.`,);
        expect(secondSlice({
          replacement: `The vet mentioned [cat scratch fever](${ORIGINAL_HREF} "article") too.`,
        },),).toBe(`The vet mentioned [cat scratch fever](${ARCHIVE_HREF} "article") too.`,);
        expect(secondSlice({
          replacement: 'The vet mentioned it too.',
        },),).toBe('The vet mentioned it too.',);
      },
    },),
    it({
      name: 'LEAVES A SWAP OF DESTINATIONS BOTH SIDES CARRY, and an original destination replaced two ways',
      fn: async () => {
        expect(replacedDestinations({
          slices: [
            pair({
              sliceIndex: 0,
              source: '[猫](https://a.example/1) [狗](https://a.example/2)',
              target: '[Cat](https://a.example/2) [Dog](https://a.example/1)',
              startOffset: 0,
            },),
          ],
        },).size,).toBe(0,);
        expect(replacedDestinations({
          slices: [
            pair({
              sliceIndex: 0,
              source: `[猫抓病](${ORIGINAL_HREF})`,
              target: '[cat scratch](https://en.example.org/one)',
              startOffset: 0,
            },),
            pair({
              sliceIndex: 1,
              source: `[猫抓病](${ORIGINAL_HREF})`,
              target: '[cat scratch](https://en.example.org/two)',
              startOffset: 40,
            },),
          ],
        },).size,).toBe(0,);
      },
    },),
    it({
      name: 'PAIRS ONLY SLICES WITH EQUAL LINK COUNTS, reading one replaced destination where they are equal',
      fn: async () => {
        expect([
          ...replacedDestinations({
            slices: [
              pair({
                sliceIndex: 0,
                source: `[猫抓病](${ORIGINAL_HREF})`,
                target: `[cat scratch](${ARCHIVE_HREF}) and [more](https://en.example.org/more)`,
                startOffset: 0,
              },),
            ],
          },).entries(),
        ],).toEqual([],);
        expect([
          ...replacedDestinations({
            slices: [
              pair({
                sliceIndex: 0,
                source: `[猫抓病](${ORIGINAL_HREF})`,
                target: `[cat scratch](${ARCHIVE_HREF})`,
                startOffset: 0,
              },),
            ],
          },).entries(),
        ],).toEqual([
          [
            ORIGINAL_HREF,
            ARCHIVE_HREF,
          ],
        ],);
      },
    },),
  ],
},);
