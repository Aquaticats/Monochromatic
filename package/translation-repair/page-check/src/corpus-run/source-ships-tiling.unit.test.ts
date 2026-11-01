/**
 Tests for the composition run over what an artifact alone records, which a page on disk is judged against.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  composeOverTiling,
  type PageGrammarReading,
  tilingOf,
} from '../../dist/final/node/index.mjs';
import {
  refusingNope,
  sourceShippedArchive,
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
 A grammar that parses everything.

 @returns Parse

 @example
 ```ts
 const reading = parsingEverything();
 ```
 */
function parsingEverything(): PageGrammarReading {
  return { kind: 'parses', };
}

await describe({
  name: 'source ships tiling',
  children: [
    describe({
      name: tilingOf.name,
      children: [
        it({
          name: 'TILES THE ARCHIVE WITH THE SLICES\' ARCHIVE WORDINGS END TO END, a span for each wording and an '
            + 'insertion at the offset the archive stands at for each slice with none',
          fn: async () => {
            const { slices, archiveText, } = tilingOf({ artifact: sourceShippedArtifact({ rows: ROWS, },), },);
            expect(archiveText,).toBe(sourceShippedArchive({ rows: ROWS, },),);
            expect(slices.map(function targetOf({ target, },): string {
              return `${target.kind} ${String(target.startOffset,)}-${String(target.endOffset,)}`;
            },),).toEqual(['content 0-55', 'insertion 55-55', 'content 55-82', 'insertion 82-82',],);
          },
        },),
      ],
    },),

    describe({
      name: composeOverTiling.name,
      children: [
        it({
          name: 'COMPOSES THE PAGE AS WRITTEN WHEN THE GRAMMAR ACCEPTS IT, and leaves off the runs no writing makes '
            + 'parse when it does not, the steps the publisher takes',
          fn: async () => {
            const artifact = sourceShippedArtifact({ rows: ROWS, },);
            expect(composeOverTiling({
              artifact,
              readGrammar: parsingEverything,
            },).step,).toBe('as-written',);
            const dropped = composeOverTiling({
              artifact,
              readGrammar: refusingNope,
            },);
            expect(dropped.step,).toBe('dropped',);
            expect(dropped.slices,).toEqual([1,],);
          },
        },),
      ],
    },),
  ],
},);
