/**
 Tests that a source-only passage another slice already renders is settled as carried with nothing reported, and that
 one the page has no room to be missing is settled unfilled as not corroborated, with a warning and a finding that name
 it, neither having bought anything.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  capturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  carriedElsewhere,
  notCorroborated,
} from '../dist/final/node/index.mjs';

//region Unbought slice tests

await describe({
  name: 'slices settled without a purchase',
  concurrency: 1,
  children: [
    it({
      name: 'SETTLES A PASSAGE CARRIED ELSEWHERE as carried, reporting nothing and saying so in one info line',
      fn: async () => {
        const lines: string[] = [];
        const settlement = carriedElsewhere({ sliceIndex: 5, l: capturingLogger({ messages: lines, },), },);
        expect({
          settlement,
          lines,
        },).toEqual({
          settlement: {
            outcome: { kind: 'carried', sliceIndex: 5, },
            refusedCacheFindings: [],
            unfilledFindings: [],
          },
          lines: ['[carriedElsewhere] slice 5: source passage already carried elsewhere',],
        },);
      },
    },),
    it({
      name: 'SETTLES A PASSAGE THE PAGE HAS NO ROOM TO BE MISSING as unfilled and not corroborated, with the finding '
        + 'that names the chunk and one warning saying nothing was bought',
      fn: async () => {
        const lines: string[] = [];
        const settlement = notCorroborated({ sliceIndex: 6, l: capturingLogger({ messages: lines, },), },);
        expect({
          outcome: settlement.outcome,
          refused: settlement.refusedCacheFindings,
          findingsNameTheChunk: settlement.unfilledFindings.every(function namesChunk(finding,): boolean {
            return finding.endsWith('chunk 6',);
          },),
          findingCount: settlement.unfilledFindings.length,
          said: lines.some(function saysNothingBought(line,): boolean {
            return line.startsWith('[notCorroborated] slice 6: ',)
              && line.includes('nothing was bought and the passage stays as it is',);
          },),
        },).toEqual({
          outcome: {
            kind: 'unfilled',
            unfilled: { sliceIndex: 6, reason: 'not-corroborated', findings: [], },
          },
          refused: [],
          findingsNameTheChunk: true,
          findingCount: 1,
          said: true,
        },);
      },
    },),
  ],
},);

//endregion Unbought slice tests
