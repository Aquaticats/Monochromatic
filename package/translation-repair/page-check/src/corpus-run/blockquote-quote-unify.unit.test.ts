/**
 Guards class one hundred sixty-eight (TianqiChen6667, 2026-09-26): the
 archive sets every quoted message as a bare blockquote, yet the page wrapped
 one blockquote's paragraphs in “…” and left the rest bare, so one page quoted
 its messages two ways. The page-assembly pass reads the archive's
 blockquote convention and, where the archive's blockquotes are mostly bare,
 unwraps each page blockquote paragraph that is one quotation from its first
 mark to its last.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import { unwrapBlockquoteQuotes, } from '../../dist/final/node/index.mjs';

/**
 One slice over an archive text, starting at an offset.

 @param sliceIndex - where the slice stands

 @param target - archive text

 @param startOffset - where the archive text starts in the page

 @returns Prepared pair

 @example
 ```ts
 const slice = pair({ sliceIndex: 0, target: '> She napped.', startOffset: 0, },);
 ```
 */
function pair(
  {
    sliceIndex,
    target,
    startOffset,
  }: {
    readonly sliceIndex: number;
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
      endOffset: 6,
      text: '>「她打盹。」',
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
 Text the page carries at slice 2 after the pass, where slices 0 and 1 are
 archive blockquotes no lane replaced and a lane wrote slice 2.

 @param archive - archive text of slices 0 and 1

 @param replacement - what a lane wrote for slice 2

 @returns Slice 2's text after the pass

 @example
 ```ts
 thirdSlice({ archive: ['> Purr.', '> Nap.',], replacement: '> “Meow.”', },);
 ```
 */
function thirdSlice(
  {
    archive,
    replacement,
  }: {
    readonly archive: readonly string[];
    readonly replacement: string;
  },
): string {
  /**
   Lane rows: slice 2 alone.
   */
  const replacements: readonly SliceReplacement[] = [{
    sliceIndex: 2,
    replacementText: replacement,
  },];
  /**
   Archive slices, then the lane's slice over an archive blockquote.
   */
  const texts = [
    ...archive,
    '> The cat napped.',
  ];
  /**
   Prepared pairs laid end to end with a blank line between.
   */
  const slices = texts.map(function toPair(
    target,
    sliceIndex,
  ): ChunkPair {
    /**
     Where this slice's archive text starts.
     */
    const startOffset = texts
      .slice(
      0,
      sliceIndex,
      )
      .reduce(
      function past(
        offset,
        earlier,
      ): number {
        return offset + earlier.length + 2;
      },
      0,
      );
    return pair({
      sliceIndex,
      target,
      startOffset,
    },);
  },);
  /**
   The page after the pass.
   */
  const unified = unwrapBlockquoteQuotes({
    slices,
    replacements,
    archiveOriginalSpans: [],
  },);
  return unified.replacements
    .find(function isThird(row,): boolean {
      return row.sliceIndex === 2;
    },)
    ?.replacementText ?? replacement;
}

await describe({
  name: 'unwrapBlockquoteQuotes (class one hundred sixty-eight)',
  children: [
    it({
      name: 'UNWRAPS a quoted blockquote paragraph where the archive quotes its messages bare',
      fn: async () => {
        expect(thirdSlice({
          archive: [
            '> Purr, said the kitten.\n>\n> Nap time now.',
            '> The whiskers twitched.',
          ],
          replacement: '> “My... nap?\n> I want a warmer rug.”\n>\n> “Please stay by the window.”',
        },),).toBe('> My... nap?\n> I want a warmer rug.\n>\n> Please stay by the window.',);
      },
    },),
    it({
      name: 'LEAVES a paragraph holding a quotation inside it',
      fn: async () => {
        expect(thirdSlice({
          archive: [
            '> Purr, said the kitten.',
            '> The whiskers twitched.',
          ],
          replacement: '> “The kitten said “meow” twice,” wrote the cat.',
        },),).toBe('> “The kitten said “meow” twice,” wrote the cat.',);
      },
    },),
    it({
      name: 'LEAVES quoted blockquotes where the archive quotes its messages',
      fn: async () => {
        expect(thirdSlice({
          archive: [
            '> “Purr, said the kitten.”',
            '> “The whiskers twitched.”',
          ],
          replacement: '> “Please stay by the window.”',
        },),).toBe('> “Please stay by the window.”',);
      },
    },),
    it({
      name: 'UNWRAPS a paragraph quoted in straight doubles, and LEAVES one holding a straight quotation inside it',
      fn: async () => {
        expect([
          thirdSlice({
            archive: [
              '> Purr, said the kitten.',
              '> The whiskers twitched.',
            ],
            replacement: '> "Please stay by the window."',
          },),
          thirdSlice({
            archive: [
              '> Purr, said the kitten.',
              '> The whiskers twitched.',
            ],
            replacement: '> "The kitten said "meow" twice," wrote the cat.',
          },),
        ],).toEqual([
          '> Please stay by the window.',
          '> "The kitten said "meow" twice," wrote the cat.',
        ],);
      },
    },),
    it({
      name: 'LEAVES a quoted paragraph outside any blockquote',
      fn: async () => {
        expect(thirdSlice({
          archive: [
            '> Purr, said the kitten.',
            '> The whiskers twitched.',
          ],
          replacement: '“Please stay by the window.”',
        },),).toBe('“Please stay by the window.”',);
      },
    },),
  ],
},);
