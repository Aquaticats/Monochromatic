/**
 Tests for which source runs of a page can ship and which cannot, and for the artifact read without the record of the
 runs that cannot.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  droppedRunSets,
  type PageGrammarReading,
  withoutSourceRecordAt,
  wouldShipTextPerSlice,
} from '../../dist/final/node/index.mjs';
import {
  sourceShippedArtifact,
  type SourceShippedRow,
} from './source-shipped.test-fixture.ts';

/**
 Rows with a paragraph the archive covers, a run holding the word NOPE, a paragraph and a run that is plain.
 */
const ROWS: readonly SourceShippedRow[] = [
  {
    incumbentText: '## Description\n\nA tabby who kept the bookshop company.\n',
    sourceText: '## 简介\n\n一只在书店陪伴大家的虎斑猫。\n',
  },
  {
    incumbentText: '',
    sourceText: 'NOPE 她说\n',
  },
  {
    incumbentText: '\nShe slept on the counter.\n',
    sourceText: '\n她睡在柜台上。\n',
  },
  {
    incumbentText: '',
    sourceText: '她又说\n',
  },
];

/**
 A grammar that refuses a text holding NOPE, written or not.

 @param pageText - page or fragment

 @returns Refusal for a text holding the word, a parse otherwise

 @example
 ```ts
 const reading = refusingNope({ pageText: 'NOPE', },);
 ```
 */
function refusingNope({ pageText, }: { readonly pageText: string; },): PageGrammarReading {
  return pageText.includes('NOPE',)
    ? {
      kind: 'refused',
      refusal: 'at 1:1 (test/nope)',
    }
    : { kind: 'parses', };
}

/**
 A grammar that refuses a text holding a brace that nothing escapes.

 @param pageText - page or fragment

 @returns Refusal for a text holding such a brace, a parse otherwise

 @example
 ```ts
 const reading = refusingBareBrace({ pageText: '{', },);
 ```
 */
function refusingBareBrace({ pageText, }: { readonly pageText: string; },): PageGrammarReading {
  return pageText
      .replaceAll(
        String.raw`\{`,
        '',
      )
      .includes('{',)
    ? {
      kind: 'refused',
      refusal: 'at 1:1 (test/brace)',
    }
    : { kind: 'parses', };
}

await describe({
  name: 'source ships runs',
  children: [
    describe({
      name: droppedRunSets.name,
      children: [
        it({
          name: 'LISTS THE RUN NO WRITING MAKES PARSE FIRST AND EVERY RUN LAST, the order the third step tries them in',
          fn: async () => {
            expect(droppedRunSets({
              artifact: sourceShippedArtifact({ rows: ROWS, },),
              readGrammar: refusingNope,
            },),).toEqual([[1,], [1, 3,],],);
          },
        },),

        it({
          name: 'LISTS EVERY RUN ALONE WHEN EACH PARSES AS LITERAL TEXT, since the page may fail for what the runs '
            + 'make together',
          fn: async () => {
            expect(droppedRunSets({
              artifact: sourceShippedArtifact({ rows: ROWS, },),
              readGrammar: function parsing(): PageGrammarReading {
                return { kind: 'parses', };
              },
            },),).toEqual([[1, 3,],],);
          },
        },),

        it({
          name: 'READS A RUN AS LITERAL TEXT, so a run a brace refuses as written is not one no writing makes parse',
          fn: async () => {
            expect(droppedRunSets({
              artifact: sourceShippedArtifact({
                rows: [
                  ROWS[0] as SourceShippedRow,
                  {
                    incumbentText: '',
                    sourceText: '{ 她说\n',
                  },
                ],
              },),
              readGrammar: refusingBareBrace,
            },),).toEqual([[1,],],);
          },
        },),

        it({
          name: 'LISTS NOTHING FOR AN ARTIFACT WHOSE SLICES SHIP NO ORIGINAL',
          fn: async () => {
            expect(droppedRunSets({
              artifact: sourceShippedArtifact({ rows: [ROWS[0] as SourceShippedRow, ROWS[2] as SourceShippedRow,], },),
              readGrammar: refusingNope,
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: withoutSourceRecordAt.name,
      children: [
        it({
          name: 'READS THE NAMED SLICES AS SILENT AND LEAVES THE OTHERS SHIPPING THEIR ORIGINAL',
          fn: async () => {
            const artifact = sourceShippedArtifact({ rows: ROWS, },);
            expect(wouldShipTextPerSlice({
              artifact: withoutSourceRecordAt({
                artifact,
                slices: [1,],
              },),
            },).map(function kindOf({ reading, },): string {
              return reading.kind;
            },),).toEqual(['wording', 'nothing-ships', 'wording', 'source-ships',],);
            expect(wouldShipTextPerSlice({
              artifact: withoutSourceRecordAt({
                artifact,
                slices: [],
              },),
            },).map(function kindOf({ reading, },): string {
              return reading.kind;
            },),).toEqual(['wording', 'source-ships', 'wording', 'source-ships',],);
          },
        },),
      ],
    },),
  ],
},);
