/**
 Tests for the plugin that adds GFM tables and strikethrough to a parser and no other part of GFM.

 Two differentials bound it: on a text with no pipe and no tilde the parser it makes builds the tree of CommonMark with the
 site's math, the plain reading it replaces, node for node and position for position; on a text with pipes and tildes and
 none of the other parts of GFM (a footnote, an address, a task list item) it builds the tree of the full GFM reading.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import remarkMath from 'remark-math';
import remarkParse from 'remark-parse';
import { unified, } from 'unified';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  parseGfmMathBody,
  parseMarkdownMathBody,
  withTablesAndStrikethrough,
} from '../dist/final/node/index.mjs';
import {
  integerBelow,
  randomOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/seeded-draws.test-fixture.ts';

/**
 Pieces a text is built from where the differentials need no pipe and no tilde, and the parts of GFM the plugin leaves off.
 */
const COMMON: readonly string[] = [
  'a',
  '猫 ',
  '`x',
  '`',
  '$',
  '*',
  '_',
  '{c}',
  '<b>',
  '</b>',
  '<!-- c -->',
  '\n',
  '\n\n',
  '- ',
  '> ',
  '# ',
  '    ',
  '```\n',
  '[t](u "v")',
  '[r]: u\n',
  '1. ',
  '---\n',
  '\\',
  '  \n',
];

/**
 The parts of GFM the plugin leaves off, which the second differential never uses.
 */
const LEFT_OFF: readonly string[] = [
  '[^1]: {',
  '[^1]',
  'www.a.b',
  'https://x.y/z',
  '- [ ] t',
  '- [x] t',
  'a@b.cc',
];

/**
 Pieces only the second differential uses: the pipes of a table and the tildes of strikethrough.
 */
const TABLE_AND_TILDE: readonly string[] = [
  '|',
  '| a | b |\n| - | - |\n',
  '| c | `d |\n',
  '~~',
  '~',
  'x~~y',
  String.raw`\|`,
];

/**
 Builds a text from pieces chosen by a generator with a fixed seed.

 @param pieces - pieces to choose among
 @param seed - seed of the generator
 @param count - how many texts to build

 @returns Texts of one to nine pieces each

 @example
 ```ts
 const texts = textsOf({ pieces: COMMON, seed: 7, count: 100, },);
 ```
 */
function textsOf(
  {
    pieces,
    seed,
    count,
  }: {
    readonly pieces: readonly string[];
    readonly seed: number;
    readonly count: number;
  },
): readonly string[] {
  /**
   The generator.
   */
  const rand = randomOf({ seed, },);

  return Array.from(
    { length: count, },
    function built(): string {
      return Array.from(
        { length: 1 + integerBelow({ rand, bound: 9, }), },
        function piece(): string {
          return pieces[integerBelow({ rand, bound: pieces.length, })] ?? '';
        },
      )
        .join('',);
    },
  );
}

/**
 The tree of CommonMark with the site's math and nothing else, the plain reading before it read tables.

 @param body - text

 @returns Tree as JSON text, positions included

 @example
 ```ts
 const json = commonMarkJson({ body: 'a\n', },);
 ```
 */
function commonMarkJson({ body, }: { readonly body: string; },): string {
  return JSON.stringify(unified()
    .use(remarkParse,)
    .use(remarkMath,)
    .parse(body,),);
}

await describe({
  name: withTablesAndStrikethrough.name,
  children: [
    it({
      name: 'BUILDS THE TREE OF COMMONMARK WITH MATH FOR A TEXT WITH NO PIPE AND NO TILDE, footnotes, addresses and task '
        + 'list items among the texts',
      fn: async () => {
        /**
         Texts with no pipe and no tilde, and some with the parts of GFM the plugin leaves off.
         */
        const texts = textsOf({
          pieces: [...COMMON, ...LEFT_OFF,],
          seed: 20_261_025,
          count: 4_000,
        },);
        /**
         How many texts the full GFM reading builds another tree for, which shows the differential can tell the readings apart.
         */
        const gfmDiffers = { count: 0, };
        for (const body of texts) {
          /**
           The tree of CommonMark with math, as JSON.
           */
          const commonMark = commonMarkJson({ body, },);
          /**
           The tree the plugin builds, and the tree of the full GFM reading, as JSON.
           */
          const ours = JSON.stringify(parseMarkdownMathBody({ body, },),);
          const gfm = JSON.stringify(parseGfmMathBody({ body, },),);
          expect(ours,).toBe(commonMark,);
          if (gfm !== commonMark)
            gfmDiffers.count += 1;
        }
        expect(gfmDiffers.count,).toBeGreaterThan(200,);
      },
    },),
    it({
      name: 'BUILDS THE TREE OF THE FULL GFM READING FOR A TEXT WITH PIPES AND TILDES AND NONE OF THE OTHER PARTS',
      fn: async () => {
        /**
         Texts with pipes and tildes and none of the parts the plugin leaves off.
         */
        const texts = textsOf({
          pieces: [...COMMON, ...TABLE_AND_TILDE,],
          seed: 31_415_926,
          count: 4_000,
        },);
        /**
         How many texts hold a table or a strikethrough, so the differential is not one of plain texts.
         */
        const read = { tables: 0, deletes: 0, };
        for (const body of texts) {
          /**
           The tree the plugin builds, and the tree of the full GFM reading, as JSON.
           */
          const ours = JSON.stringify(parseMarkdownMathBody({ body, },),);
          const gfm = JSON.stringify(parseGfmMathBody({ body, },),);
          expect(ours,).toBe(gfm,);
          if (ours.includes('"type":"table"',))
            read.tables += 1;
          if (ours.includes('"type":"delete"',))
            read.deletes += 1;
        }
        expect(read.tables,).toBeGreaterThan(100,);
        expect(read.deletes,).toBeGreaterThan(100,);
      },
    },),
  ],
},);
