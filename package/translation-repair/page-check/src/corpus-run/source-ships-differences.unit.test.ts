/**
 Tests for the defect that names the runs a page shows differently from the source.

 The composition's own cases are in `source-reading-defect.unit.test.ts`; these hand the function the runs and the slices
 it is asked about, so the branches it takes without the composition (a run it cannot compare, one it cannot decide) are
 reached.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  differenceDefectsOf,
  sourceRunsOf,
  wouldShipTextPerSlice,
} from '../../dist/final/node/index.mjs';
import {
  sourceShippedArtifact,
  type SourceShippedRow,
} from './source-shipped.test-fixture.ts';

/**
 The paragraph the archive covers.
 */
const OPENING_ROW: SourceShippedRow = {
  incumbentText: '## Description\n\nA tabby who kept the bookshop company.\n',
  sourceText: '## 简介\n\n一只在书店陪伴大家的虎斑猫。\n',
};

/**
 What the function answers for the runs of rows, with the slices written as literal text, and what it logged.

 @param rows - slices, the anchor among them

 @param encoded - slices written as literal text

 @returns Defects and lines the function wrote at the warn and info levels

 @example
 ```ts
 const { defects, } = differenceFor({ rows: [OPENING_ROW,], encoded: [], },);
 ```
 */
function differenceFor(
  {
    rows,
    encoded,
  }: {
    readonly rows: readonly SourceShippedRow[];
    readonly encoded: readonly number[];
  },
): {
  readonly defects: ReturnType<typeof differenceDefectsOf>;
  readonly lines: readonly string[];
} {
  /**
   Lines written.
   */
  const lines: string[] = [];
  /**
   Logger keeping its lines.
   */
  const l: Logger = {
    ...tagged({ tag: 'source-ships-differences-test', },),
    warn: function keepWarn(message: string,): void {
      lines.push(`warn ${message}`,);
    },
    info: function keepInfo(message: string,): void {
      lines.push(`info ${message}`,);
    },
  };
  /**
   What each slice would carry.
   */
  const slices = wouldShipTextPerSlice({ artifact: sourceShippedArtifact({ rows, },), },);
  return {
    defects: differenceDefectsOf({
      runs: sourceRunsOf({ slices, },),
      encoded,
      slices,
      entryId: 'BookshopCat',
      l,
    },),
    lines,
  };
}

await describe({
  name: differenceDefectsOf.name,
  children: [
    it({
      name: 'REPORTS NOTHING FOR A RUN THE PAGE CARRIES AS RECORDED, whatever the two readings make of it',
      fn: async () => {
        expect(differenceFor({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: '![<https://example.net/a>](u.png)\n',
            },
          ],
          encoded: [],
        },).defects,).toEqual([],);
      },
    },),
    it({
      name: 'REPORTS NOTHING FOR A RUN WRITTEN AS LITERAL TEXT THAT THE PAGE SHOWS AS THE SOURCE DOES',
      fn: async () => {
        expect(differenceFor({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: '她说 { 然后走了。\n',
            },
          ],
          encoded: [1,],
        },).defects,).toEqual([],);
      },
    },),
    it({
      name: 'NAMES THE SLICES OF EACH RUN THE PAGE SHOWS DIFFERENTLY IN ONE DEFECT, and no text of the page',
      fn: async () => {
        /**
         Two runs apart by a paragraph the archive covers, each holding an image with an angle autolink.
         */
        const { defects, } = differenceFor({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: '![<https://example.net/a>](u.png) {\n',
            },
            {
              incumbentText: '\nShe slept on the counter.\n',
              sourceText: '\n她睡在柜台上。\n',
            },
            {
              incumbentText: '',
              sourceText: '![<https://example.net/b>](u.png) {\n',
            },
          ],
          encoded: [
            1,
            3,
          ],
        },);
        expect(defects.map(function checkOf({ check, },): string {
          return check;
        },),).toEqual(['source-reads-differently',],);
        expect(defects[0]?.message,).toContain('slices [1, 3]',);
        expect(defects[0]?.message,).not.toContain('example.net',);
      },
    },),
    it({
      name: 'LOGS A RUN IT CANNOT COMPARE AND REPORTS IT IN A DEFECT OF ITS OWN, a text nested past what the reading admits',
      fn: async () => {
        /**
         A run nested past the bound of the plain reading, which the page cannot carry as text either.
         */
        const { defects, lines, } = differenceFor({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: `${'> '.repeat(300,)}她说 {\n`,
            },
          ],
          encoded: [1,],
        },);
        expect(defects.map(function checkOf({ check, },): string {
          return check;
        },),).toEqual(['source-reading-not-compared',],);
        expect(defects[0]?.message,).toContain('slices [1]',);
        expect(defects[0]?.message,).not.toContain('她说',);
        expect(lines.map(function levelOf(line,): string {
          return line.split(' ', 1,)[0] ?? '';
        },),).toEqual(['warn',],);
      },
    },),
    it({
      name: 'LOGS A RUN IT CANNOT COMPARE THOUGH THE PAGE PARSES, a link target the reading cannot decode',
      fn: async () => {
        /**
         A run with a link whose target holds a percent escape that is no text in UTF-8 and a bare brace, which the grammar
         refuses as recorded: the page it is written as parses, and both readings decode the target before they compare.
         */
        const { defects, lines, } = differenceFor({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: '[虎斑](%E0%A4) 她说 {\n',
            },
          ],
          encoded: [1,],
        },);
        expect(defects.map(function checkOf({ check, },): string {
          return check;
        },),).toEqual(['source-reading-not-compared',],);
        expect(defects[0]?.message,).toContain('slices [1]',);
        expect(defects[0]?.message,).not.toContain('%E0',);
        expect(lines.map(function levelOf(line,): string {
          return line.split(' ', 1,)[0] ?? '';
        },),).toEqual(['warn',],);
        expect(lines[0],).toContain('could not be compared (URIError)',);
      },
    },),
    it({
      name: 'LOGS A RUN IT CANNOT DECIDE AND REPORTS IT IN A DEFECT OF ITS OWN, a footnote in a block quote that a tag ends',
      fn: async () => {
        /**
         A block quote whose footnote the source's reading and the page's end at different places.
         */
        const { defects, lines, } = differenceFor({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: '> [^1]: x\n> <q>\n> text\n> ===\n',
            },
          ],
          encoded: [1,],
        },);
        expect(defects.map(function checkOf({ check, },): string {
          return check;
        },),).toEqual(['source-reading-undecided',],);
        expect(defects[0]?.message,).toContain('slices [1]',);
        expect(defects[0]?.message,).not.toContain('<q>',);
        expect(lines.map(function levelOf(line,): string {
          return line.split(' ', 1,)[0] ?? '';
        },),).toEqual(['info',],);
      },
    },),
    it({
      name: 'REPORTS A RUN THAT DIFFERS AND A BLOCK OF IT THAT CANNOT BE DECIDED IN TWO DEFECTS, naming the same slices',
      fn: async () => {
        /**
         A block quote whose footnote the two readings end at different places, and an image whose alternative text is an
         angle autolink after it.
         */
        const { defects, } = differenceFor({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: '> [^1]: x\n> <q>\n> text\n> ===\n\n![<https://example.net/a>](u.png)\n',
            },
          ],
          encoded: [1,],
        },);
        expect(defects.map(function checkOf({ check, },): string {
          return check;
        },),).toEqual([
          'source-reads-differently',
          'source-reading-undecided',
        ],);
        expect(defects[0]?.message,).toContain('slices [1]',);
        expect(defects[1]?.message,).toContain('slices [1]',);
      },
    },),
  ],
},);
