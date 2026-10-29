/**
 Tests that a page which shows the original of an untranslated passage differently from the source ships with a reported
 defect, and that a page which shows it as the source does ships with none.

 The owner's ruling of 2026-10-25 ("Add the comparison") makes the page check compare the reading of the page with the
 reading of the source, and the owner's ruling of 2026-09-27 that a run always ships makes the answer a defect and never
 a refusal. The comparison is made for the runs the composition writes as literal text, since a run that parses as
 recorded ships as the upstream site compiles it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  composeSourcePage,
  type ComposedPage,
} from '../../dist/final/node/index.mjs';
import { compareWithBracketedAlt, } from './source-reading-bracketed-alt.test-fixture.ts';
import {
  sourceShippedArchive,
  sourceShippedArtifact,
  type SourceShippedRow,
  sourceShippedSlices,
} from './source-shipped.test-fixture.ts';

/**
 The paragraph the archive covers.
 */
const OPENING_ROW: SourceShippedRow = {
  incumbentText: '## Description\n\nA tabby who kept the bookshop company.\n',
  sourceText: '## 简介\n\n一只在书店陪伴大家的虎斑猫。\n',
};

/**
 A comparison that fails for any page, standing for the fault of a page that parses and cannot be compared.

 @throws {@link Error} always, the way a reading that cannot be made does

 @example
 ```ts
 const composed = composeRows({ rows: [OPENING_ROW,], compareReadings: refusesToCompare, },);
 ```
 */
function refusesToCompare(): never {
  throw new Error('the comparison could not be made',);
}

/**
 A comparison that finds every page to differ, standing for a reading the page check reports.

 @returns Answer `differs`

 @example
 ```ts
 const composed = composeRows({ rows: [OPENING_ROW,], compareReadings: findsEveryPageDiffering, },);
 ```
 */
function findsEveryPageDiffering(): { readonly kind: 'differs'; readonly reason: string; } {
  return {
    kind: 'differs',
    reason: 'the page shows other text than the source',
  };
}

/**
 Composes the page of rows with the real grammar.

 @param rows - slices, the anchor among them

 @param compareReadings - what the page check compares the two readings with, the real comparison when absent

 @returns Composition of the rows

 @example
 ```ts
 const composed = composeRows({ rows: [OPENING_ROW,], },);
 ```
 */
function composeRows(
  {
    rows,
    compareReadings,
  }: {
    readonly rows: readonly SourceShippedRow[];
    readonly compareReadings?: (
      input: { readonly source: string; readonly carried: string; },
    ) => ReturnType<typeof compareWithBracketedAlt>;
  },
): ComposedPage {
  /**
   Logger that keeps nothing.
   */
  const l: Logger = tagged({ tag: 'source-reading-defect-test', },);
  return composeSourcePage({
    artifact: sourceShippedArtifact({ rows, },),
    slices: sourceShippedSlices({ rows, },),
    archiveText: sourceShippedArchive({ rows, },),
    entryId: 'BookshopCat',
    l,
    ...((compareReadings === undefined) ? {} : { compareReadings, }),
  },);
}

await describe({
  name: 'the defect for a page that shows the source differently',
  children: [
    it({
      name: 'REPORTS NO DEFECT FOR A RUN WRITTEN AS LITERAL TEXT THAT THE PAGE SHOWS AS THE SOURCE DOES',
      fn: async () => {
        /**
         The page of a run with a bare brace, which the grammar refuses as recorded.
         */
        const composed = composeRows({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: '她说 { 然后走了。\n',
            },
          ],
        },);
        expect(composed.step,).toBe('encoded-failing-runs',);
        expect(composed.defects,).toEqual([],);
      },
    },),
    it({
      name: 'REPORTS A DEFECT NAMING THE SLICE FOR A RUN WRITTEN AS LITERAL TEXT THAT THE PAGE SHOWS DIFFERENTLY',
      fn: async () => {
        /**
         The page of a run with a bare brace and an image whose alternative text is an angle autolink, compared with a page
         built by hand that shows the brackets the source does not.
         */
        const composed = composeRows({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: '![<https://example.net/a>](u.png) {然后走了。\n',
            },
          ],
          compareReadings: compareWithBracketedAlt,
        },);
        expect(composed.step,).toBe('encoded-failing-runs',);
        expect(composed.defects.map(function checkOf({ check, },): string {
          return check;
        },),).toEqual(['source-reads-differently',],);
        expect(composed.defects[0]?.message,).toContain('slices [1]',);
        expect(composed.defects[0]?.message,).not.toContain('example.net',);
      },
    },),
    it({
      name: 'REPORTS A DEFECT NAMING THE SLICE FOR A RUN WHOSE READING THE COMPARISON CANNOT DECIDE',
      fn: async () => {
        /**
         The page of a run with a bare brace in a footnote of a block quote that a tag ends, where the source's reading and
         the page's end the footnote at different places.
         */
        const composed = composeRows({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: '> [^1]: x {\n> <q>\n> text\n> ===\n',
            },
          ],
        },);
        expect(composed.step,).toBe('encoded-failing-runs',);
        expect(composed.defects.map(function checkOf({ check, },): string {
          return check;
        },),).toEqual(['source-reading-undecided',],);
        expect(composed.defects[0]?.message,).toContain('slices [1]',);
        expect(composed.defects[0]?.message,).not.toContain('<q>',);
      },
    },),
    it({
      name: 'REPORTS NO DEFECT FOR A RUN WHOSE LINK TARGET IS NO TEXT IN UTF-8, the target decoded as far as it reads',
      fn: async () => {
        /**
         The page of a run with a link whose target holds a percent escape that decodes to no text, and a bare brace: the page
         parses as literal text, so it ships with its original, and both readings decode the target as far as it reads, so
         the comparison is made and finds the page to show what the source shows.
         */
        const composed = composeRows({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: '[虎斑](%E0%A4) 她说 {\n',
            },
          ],
        },);
        expect(composed.step,).toBe('encoded-failing-runs',);
        expect(composed.defects,).toEqual([],);
      },
    },),
    it({
      name: 'REPORTS A DEFECT NAMING THE SLICE FOR A PAGE THAT PARSES AND WHOSE COMPARISON FAILS, through the composition',
      fn: async () => {
        /**
         The page of a run with a bare brace, which the grammar refuses as recorded and accepts as literal text, composed
         with a comparison that fails for it.
         */
        const composed = composeRows({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: '她说 { 然后走了。\n',
            },
          ],
          compareReadings: refusesToCompare,
        },);
        expect(composed.step,).toBe('encoded-failing-runs',);
        expect(composed.defects.map(function checkOf({ check, },): string {
          return check;
        },),).toEqual(['source-reading-not-compared',],);
        expect(composed.defects[0]?.message,).toContain('slices [1]',);
        expect(composed.defects[0]?.message,).not.toContain('她说',);
      },
    },),
    it({
      name: 'ASKS THE COMPARISON IT IS GIVEN, and reports what that answers',
      fn: async () => {
        /**
         The same page, composed with a comparison that finds it to differ.
         */
        const composed = composeRows({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: '她说 { 然后走了。\n',
            },
          ],
          compareReadings: findsEveryPageDiffering,
        },);
        expect(composed.defects.map(function checkOf({ check, },): string {
          return check;
        },),).toEqual(['source-reads-differently',],);
      },
    },),
    it({
      name: 'ASKS THE COMPARISON IT IS GIVEN ALSO FOR THE RUNS THAT STAY ON A PAGE SHIPPED WITHOUT SOME ORIGINALS',
      fn: async () => {
        /**
         A run nested past what the grammar admits, which no writing makes parse, and a later run with a bare brace that the
         page keeps as literal text, composed with a comparison that fails for it.
         */
        const composed = composeRows({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: `${'> '.repeat(300,)}她说 {\n`,
            },
            {
              incumbentText: '\nShe slept on the counter.\n',
              sourceText: '\n她睡在柜台上。\n',
            },
            {
              incumbentText: '',
              sourceText: '她说 { 然后走了。\n',
            },
          ],
          compareReadings: refusesToCompare,
        },);
        expect(composed.step,).toBe('dropped',);
        expect(composed.defects.map(function checkOf({ check, },): string {
          return check;
        },),).toEqual([
          'page-source-unparseable',
          'source-reading-not-compared',
        ],);
        expect(composed.defects[1]?.message,).toContain('slices [3]',);
      },
    },),
    it({
      name: 'REPORTS THE DIFFERENCE OF A RUN BESIDE A RUN SHIPPED WITHOUT ITS ORIGINAL, the dropped step among them',
      fn: async () => {
        /**
         The page of a run nested past what the grammar admits, which no writing makes parse, and a later run with an image
         whose alternative text is an angle autolink, compared with a page built by hand that shows the brackets the source
         does not.
         */
        const composed = composeRows({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: `${'> '.repeat(300,)}她说 {\n`,
            },
            {
              incumbentText: '\nShe slept on the counter.\n',
              sourceText: '\n她睡在柜台上。\n',
            },
            {
              incumbentText: '',
              sourceText: '![<https://example.net/a>](u.png) {x\n',
            },
          ],
          compareReadings: compareWithBracketedAlt,
        },);
        expect(composed.step,).toBe('dropped',);
        expect(composed.defects.map(function checkOf({ check, },): string {
          return check;
        },),).toEqual([
          'page-source-unparseable',
          'source-reads-differently',
        ],);
        expect(composed.defects[0]?.message,).toContain('slices [1]',);
        expect(composed.defects[1]?.message,).toContain('slices [3]',);
      },
    },),
    it({
      name: 'REPORTS NO DEFECT FOR A PAGE THAT SHIPS ITS ORIGINALS AS RECORDED, whatever the two readings make of them',
      fn: async () => {
        /**
         The page of a run with an element the site compiles, which a reader of plain Markdown sees as raw HTML.
         */
        const composed = composeRows({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: '<br />\n',
            },
          ],
        },);
        expect(composed.step,).toBe('as-written',);
        expect(composed.defects,).toEqual([],);
      },
    },),
  ],
},);
