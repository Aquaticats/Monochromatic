/**
 Tests for the page the grammar refuses, which ships with the archive kept at the slices that break it (ledger B1207).

 THE OWNER'S RULE IS "no matter what, we must ship". A page the strict grammar refuses used to end the entry with no
 page written; now slices go back to the archive's wording one by one, in order, until the page parses, and each one the
 page does not need is put back. Where even the archive's page is refused, the page ships as composed with a defect.

 A page "refuses" here when it holds the word NOPE, which stands for a run no writing makes parse.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  FallbackPageRefusedError,
  pageThatParses,
  refuseFallbackPage,
} from '../../dist/final/node/index.mjs';
import {
  refusingNope,
  type SourceShippedRow,
  sourceShippedArchive,
  sourceShippedSlices,
} from './source-shipped.test-fixture.ts';

/**
 Logger the fallback writes its refusal to, whose lines are not under test.
 */
const l = tagged({ tag: 'page-grammar-fallback-test', },);

/**
 Three paragraphs of the archive, none of which the grammar refuses.
 */
const ROWS: readonly SourceShippedRow[] = [
  { incumbentText: 'The cat sleeps.\n\n', sourceText: '猫在睡觉。\n\n', },
  { incumbentText: 'The cat wakes.\n\n', sourceText: '猫醒了。\n\n', },
  { incumbentText: 'The cat eats.\n\n', sourceText: '猫在吃。\n\n', },
];

/**
 Reads the three-slice page with the given wordings written over the archive's.

 @param wordings - replacement text per slice, in slice order

 @param rows - slices the page is spliced into, the archive's by default

 @returns What the fallback made of the page

 @example
 ```ts
 const parsing = parsingWith({ wordings: ['A.\n\n', 'NOPE\n\n', 'C.\n\n',], },);
 ```
 */
function parsingWith(
  {
    wordings,
    rows = ROWS,
  }: {
    readonly wordings: readonly string[];
    readonly rows?: readonly SourceShippedRow[];
  },
): ReturnType<typeof pageThatParses> {
  /**
   Replacements, one per slice.
   */
  const replacements = wordings.map(function replacementOf(replacementText, sliceIndex,) {
    return { sliceIndex, replacementText, };
  },);
  return pageThatParses({
    entryId: 'BookshopCat',
    pageText: wordings.join('',),
    replacements,
    slices: sourceShippedSlices({ rows, },),
    archiveText: sourceShippedArchive({ rows, },),
    l,
    readGrammar: refusingNope,
  },);
}

/**
 Runs a call and answers what it threw.

 @param call - call to run

 @returns Thrown value, or undefined where it threw nothing

 @example
 ```ts
 const error = thrownBy(() => { refuseFallbackPage({ parsing, policy: 'refuse', },); },);
 ```
 */
function thrownBy(call: () => void,): unknown {
  try {
    call();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

await describe({
  name: pageThatParses.name,
  concurrency: 1,
  children: [
    it({
      name: 'LEAVES A PAGE THAT PARSES AS COMPOSED, with no slice kept and no defect',
      fn: async () => {
        const parsing = parsingWith({ wordings: ['The cat naps.\n\n', 'The cat wakes.\n\n', 'The cat eats.\n\n',], },);
        expect(parsing.pageText,).toBe('The cat naps.\n\nThe cat wakes.\n\nThe cat eats.\n\n',);
        expect(parsing.keptArchiveAt,).toEqual([],);
        expect(parsing.defects,).toEqual([],);
      },
    },),

    it({
      name: 'KEEPS THE ARCHIVE AT THE ONE SLICE THAT BREAKS THE PAGE and no other: the slices put back in order before it '
        + 'are put back to their wordings, since the page does not need them',
      fn: async () => {
        const parsing = parsingWith({ wordings: ['The cat naps.\n\n', 'NOPE\n\n', 'The cat dines.\n\n',], },);
        expect(parsing.keptArchiveAt,).toEqual([1,],);
        expect(parsing.pageText,).toBe('The cat naps.\n\nThe cat wakes.\n\nThe cat dines.\n\n',);
        expect(parsing.replacements.map(function indexOf({ sliceIndex, },): number {
          return sliceIndex;
        },),).toEqual([0, 2,],);
        expect(parsing.defects,).toEqual([{
          check: 'page-grammar-archive-kept',
          message: 'entry BookshopCat kept the archive at slice 1, since the page did not parse with its wording',
        },],);
      },
    },),

    it({
      name: 'KEEPS THE ARCHIVE AT EVERY SLICE THAT BREAKS THE PAGE and names them in one defect',
      fn: async () => {
        const parsing = parsingWith({ wordings: ['NOPE\n\n', 'The cat stretches.\n\n', 'NOPE\n\n',], },);
        expect(parsing.keptArchiveAt.toSorted(function ascending(
          left,
          right,
        ): number {
          return left - right;
        },),).toEqual([0, 2,],);
        expect(parsing.pageText,).toBe('The cat sleeps.\n\nThe cat stretches.\n\nThe cat eats.\n\n',);
        expect(parsing.defects.map(function messageOf({ message, },): string {
          return message;
        },),).toEqual([
          'entry BookshopCat kept the archive at slices 0, 2, since the page did not parse with their wording',
        ],);
      },
    },),

    it({
      name: 'SHIPS THE PAGE AS COMPOSED WITH THE DEFECT page-unparseable where even the archive\'s page is refused',
      fn: async () => {
        const parsing = parsingWith({
          wordings: ['The cat naps.\n\n', 'NOPE\n\n', 'The cat eats.\n\n',],
          rows: [
            ROWS[0] ?? { incumbentText: '', sourceText: '', },
            { incumbentText: 'NOPE\n\n', sourceText: '猫醒了。\n\n', },
            ROWS[2] ?? { incumbentText: '', sourceText: '', },
          ],
        },);
        expect(parsing.keptArchiveAt,).toEqual([],);
        expect(parsing.pageText,).toBe('The cat naps.\n\nNOPE\n\nThe cat eats.\n\n',);
        expect(parsing.defects.map(function checkOf({ check, },): string {
          return check;
        },),).toEqual(['page-unparseable',],);
      },
    },),

    describe({
      name: refuseFallbackPage.name,
      concurrency: 1,
      children: [
        it({
          name: 'REFUSES A PAGE THAT KEPT THE ARCHIVE under the refuse policy, naming the defect and nothing of the page',
          fn: async () => {
            /**
             The error the refusal raised.
             */
            const error = thrownBy(() => {
              refuseFallbackPage({
                parsing: parsingWith({ wordings: ['The cat naps.\n\n', 'NOPE\n\n', 'The cat dines.\n\n',], },),
                policy: 'refuse',
              },);
            },);
            expect(error instanceof FallbackPageRefusedError,).toBe(true,);
            expect(refusalText({ error, },),).toBe(
              'the rebuilt page does not parse under the strict grammar (page-grammar-archive-kept), so the page stays '
                + 'as it was',
            );
          },
        },),

        it({
          name: 'REFUSES A PAGE THAT IS UNPARSEABLE EVEN SO under the refuse policy, naming page-unparseable',
          fn: async () => {
            /**
             The error the refusal raised.
             */
            const error = thrownBy(() => {
              refuseFallbackPage({
                parsing: parsingWith({
                  wordings: ['The cat naps.\n\n', 'NOPE\n\n', 'The cat eats.\n\n',],
                  rows: [
                    ROWS[0] ?? { incumbentText: '', sourceText: '', },
                    { incumbentText: 'NOPE\n\n', sourceText: '猫醒了。\n\n', },
                    ROWS[2] ?? { incumbentText: '', sourceText: '', },
                  ],
                },),
                policy: 'refuse',
              },);
            },);
            expect(error instanceof FallbackPageRefusedError,).toBe(true,);
            expect(refusalText({ error, },),).toBe(
              'the rebuilt page does not parse under the strict grammar (page-unparseable), so the page stays as it was',
            );
          },
        },),

        it({
          name: 'LETS A FALLBACK PAGE THROUGH under the ship policy, the default of a first publish',
          fn: async () => {
            expect(thrownBy(() => {
              refuseFallbackPage({
                parsing: parsingWith({ wordings: ['The cat naps.\n\n', 'NOPE\n\n', 'The cat dines.\n\n',], },),
                policy: 'ship',
              },);
            },),).toBeUndefined();
          },
        },),

        it({
          name: 'LETS A PAGE THAT PARSES THROUGH under the refuse policy, the control',
          fn: async () => {
            expect(thrownBy(() => {
              refuseFallbackPage({
                parsing: parsingWith({ wordings: ['The cat naps.\n\n', 'The cat wakes.\n\n', 'The cat eats.\n\n',], },),
                policy: 'refuse',
              },);
            },),).toBeUndefined();
          },
        },),
      ],
    },),
  ],
},);
