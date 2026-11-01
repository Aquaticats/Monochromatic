/**
 Guards class one hundred thirty-eight (XingZ6011, 2026-09-26): the
 original credited a song with 「——来自《[title](url)》[^9]，作者 handle」,
 and the signature reader, which ends a name at 《, took 来自 ("from") for
 the signer. With no archive rendering, its pinyin reading stood as the
 authority and the restore replaced the page's whole credit, link and
 footnote marker included, with that reading; the orphaned definition then
 cost the page a destination. A credit's lead word names no one.

 Also guards class one hundred thirty-nine (XingZ6012, 2026-09-26): the
 original's 「—— handle【album】《title》」 came back from the bench as
 「—— Yuli [album], “title”」; the page-side name ran to the comma, took the
 bracketed album for part of the name and the restore replaced both with the
 bare reading, deleting the album. Cat-themed invention throughout; no corpus
 content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import { restoreContributorNames, } from '../../dist/final/node/index.mjs';

/**
 One slice the archive does not carry.

 @param source - original text

 @returns Prepared pair with an empty archive text

 @example
 ```ts
 const slice = unarchived({ source: '——猫猫，2024 年', },);
 ```
 */
function unarchived({ source, }: { readonly source: string; },): ChunkPair {
  return {
    source: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: source.length,
      text: source,
    },
    target: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: 0,
      text: '',
    },
  };
}

await describe({
  name: 'restoreContributorNames on source credits (class one hundred thirty-eight)',
  children: [
    it({
      name: 'LEAVES a credit whose lead word is 来自, with its link and marker',
      fn: async () => {
        /**
         The page's credit line.
         */
        const page = '<p style="text-align: end;">——From “Yarn Ball” ([毛线球](https://example.com/yarn))[^1], by Maomao Official</p>';
        /**
         What the restore makes of it.
         */
        const restored = restoreContributorNames({
          slices: [unarchived({ source: '<p style="text-align: end;">——来自《[毛线球](https://example.com/yarn)》[^1]，作者 猫猫Official</p>', },),],
          replacements: [{
            sliceIndex: 0,
            replacementText: page,
          },],
        },);
        expect(restored.findings,).toEqual([],);
        expect(restored.replacements[0]?.replacementText,).toBe(page,);
      },
    },),
    it({
      name: 'KEEPS the album a page renders in square brackets after the signer',
      fn: async () => {
        /**
         The page's credit line, the album in ASCII brackets where the original writes 【】.
         */
        const page = '<p style="text-align: end;">—— Maomao [毛线球Yarn], “Meow Meow”</p>';
        /**
         What the restore makes of it.
         */
        const restored = restoreContributorNames({
          slices: [unarchived({ source: '<p style="text-align: end;">—— 猫猫【毛线球Yarn】《喵喵》</p>', },),],
          replacements: [{
            sliceIndex: 0,
            replacementText: page,
          },],
        },);
        expect(restored.replacements[0]?.replacementText,).toBe(page,);
      },
    },),
  ],
},);
