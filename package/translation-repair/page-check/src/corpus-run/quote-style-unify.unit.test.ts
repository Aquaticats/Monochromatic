/**
 Guards class one hundred forty-two (XingZ6012, 2026-09-26): the page wrote
 its quotes curly (134 curly double marks, 108 curly apostrophes), yet four
 archive paragraphs no lane rewrote kept their straight marks ("take it
 slow", "didn't"), because the typography restoration reads only the text a
 lane replaced. The page-assembly pass reads every slice and brings its
 prose quotes to the page's majority style, outside tags and code.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass';
import { unifyQuoteStyle, } from '../../dist/final/node/index.mjs';
import { pairAt, } from '@monochromatic-dev/translation-repair-slice-reading/ts/corpus-run/archive-slice-pair.test-fixture.ts';

/**
 Text the unified page carries at slice 0.

 @param archive - slice 0's archive text, which no lane replaced

 @param replacement - what a lane wrote for slice 1

 @returns Slice 0's text after the pass

 @example
 ```ts
 firstSlice({ archive: 'The cat said "meow".', replacement: '“Purr.” “Warm.”', },);
 ```
 */
function firstSlice(
  {
    archive,
    replacement,
  }: {
    readonly archive: string;
    readonly replacement: string;
  },
): string {
  /**
   Lane rows: slice 1 alone.
   */
  const replacements: readonly SliceReplacement[] = [{
    sliceIndex: 1,
    replacementText: replacement,
  },];
  /**
   The page after the pass.
   */
  const unified = unifyQuoteStyle({
    slices: [
      pairAt({
        sliceIndex: 0,
        target: archive,
        startOffset: 0,
      },),
      pairAt({
        sliceIndex: 1,
        target: 'The cat napped.',
        startOffset: archive.length + 2,
      },),
    ],
    replacements,
    archiveOriginalSpans: [],
  },);
  return unified.replacements
    .find(function isFirst(row,): boolean {
      return row.sliceIndex === 0;
    },)
    ?.replacementText ?? archive;
}

/**
 A page that writes its marks curly, which the first slice's archive text stands beside.
 */
const CURLY_LANE = '“Purr,” said the cat. “Warm.” “Soft.” It’s the cat’s nap.';

/**
 Archive text that holds a mark outside prose, with the text the page pass writes for it (n156, W1): the mark stays as written,
 the prose mark beside it curls. Each shape is one the n155 review ran at the page pass.
 */
const OUTSIDE_PROSE: readonly (readonly [string, string, string,])[] = [
  [
    'a link destination',
    'See [Peter](https://en.wikipedia.org/wiki/Peter_O\'Toole) now, it\'s fine.',
    'See [Peter](https://en.wikipedia.org/wiki/Peter_O\'Toole) now, it’s fine.',
  ],
  [
    'a link title',
    'See [the dogs](https://x.example/d "The dogs") now, it\'s fine.',
    'See [the dogs](https://x.example/d "The dogs") now, it’s fine.',
  ],
  ['a bare address', 'Go to https://x.example/O\'Toole now, it\'s fine.', 'Go to https://x.example/O\'Toole now, it’s fine.'],
  ['a tilde fence', 'Run:\n\n~~~js\nconsole.log("hi");\n~~~\n\nIt\'s done.', 'Run:\n\n~~~js\nconsole.log("hi");\n~~~\n\nIt’s done.'],
  ['an expression', 'Value {"a"} here, it\'s fine.', 'Value {"a"} here, it’s fine.'],
  [
    'a tag with a greater-than inside an expression',
    'See <Photo ok={a > b} alt="d" title="x" /> now, it\'s fine.',
    'See <Photo ok={a > b} alt="d" title="x" /> now, it’s fine.',
  ],
  ['double-backtick code', 'Run ``it\'s`` now, it\'s fine.', 'Run ``it\'s`` now, it’s fine.'],
  ['inline math', 'Let $f\'(x)$ be the slope, it\'s fine.', 'Let $f\'(x)$ be the slope, it’s fine.'],
];

await describe({
  name: 'unifyQuoteStyle (class one hundred forty-two)',
  children: [
    it({
      name: 'CURLS an archive slice\'s straight quotes and apostrophe on a page written curly',
      fn: async () => {
        expect(firstSlice({
          archive: 'The cat said "meow" and didn\'t stop.',
          replacement: '“Purr,” said the cat. “Warm.” It’s the cat’s nap.',
        },),).toBe('The cat said “meow” and didn’t stop.',);
      },
    },),
    it({
      name: 'LEAVES a page whose majority is straight',
      fn: async () => {
        expect(firstSlice({
          archive: 'The cat said "meow" and "purr" and didn\'t stop, wasn\'t it.',
          replacement: '“Purr,” it’s warm.',
        },),).toBe('The cat said "meow" and "purr" and didn\'t stop, wasn\'t it.',);
      },
    },),
    it({
      name: 'LEAVES tag attributes and code spans straight',
      fn: async () => {
        // The period moves inside the curled quote (class one hundred
        // eighty-one, Canadian Press style); the attribute and the code span
        // stay as written.
        expect(firstSlice({
          archive: '<Ring text="meow" /> The cat typed `say "hi"` and said "hi".',
          replacement: '“Purr,” said the cat. “Warm.” “Soft.” It’s the cat’s nap.',
        },),).toBe('<Ring text="meow" /> The cat typed `say "hi"` and said “hi.”',);
      },
    },),
    ...OUTSIDE_PROSE.map(function outsideProseCase([shape, archive, expected,],) {
      return it({
        name: `LEAVES A MARK IN ${shape} AS THE PAGE WROTE IT and curls the prose mark beside it, on a page written curly (n156, W1)`,
        fn: async () => {
          expect(firstSlice({
            archive,
            replacement: CURLY_LANE,
          },),).toBe(expected,);
        },
      },);
    },),
    it({
      name: 'COUNTS ONLY PROSE MARKS TOWARD THE PAGE\'S STYLE: straight marks in an address, a fence and an expression do not '
        + 'outvote the curly prose (n156, W1)',
      fn: async () => {
        /**
         Archive text whose straight marks all stand outside prose, except one apostrophe.
         */
        const archive = 'Go to https://x.example/a\'b and https://x.example/c\'d.\n\n~~~js\nlog("a", "b");\n~~~\n\nValue {"a"}, it\'s fine.';
        expect(firstSlice({
          archive,
          replacement: '“Purr,” said the cat. It’s the cat’s nap.',
        },),).toBe('Go to https://x.example/a\'b and https://x.example/c\'d.\n\n~~~js\nlog("a", "b");\n~~~\n\nValue {"a"}, it’s fine.',);
      },
    },),
  ],
},);
