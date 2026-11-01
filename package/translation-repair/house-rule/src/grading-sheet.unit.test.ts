/**
 Tests for the grading sheet a human grades a drawn sample on: the source
 line of a candidate that anchors no quote says why it has none, since a
 claim anchored at an empty insertion point and a claim anchored at nothing
 ask a grader for different things.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  formatGradingSheet,
  type GradingCandidate,
} from '../dist/final/node/index.mjs';

/**
 Candidate with no source quote and no anchor kind, which each case gives the
 kind it reads.
 */
const UNQUOTED: GradingCandidate = {
  entryId: 'Kitten',
  band: 'small',
  issueId: 'i/1',
  category: 'accuracy/addition',
  severity: 'major',
  summary: 'The cat naps twice.',
  sourceAnchor: 'unanchored',
  sourceQuotes: [],
  targetQuotes: ['The cat naps.',],
};

await describe({
  name: formatGradingSheet.name,
  children: [
    it({
      name: 'SAYS WHY A CANDIDATE WITHOUT A SOURCE QUOTE HAS NONE: an insertion point has no text to quote, and an '
        + 'unanchored claim points at nothing in the original',
      fn: async () => {
        /**
         Sheet written for the two candidates.
         */
        const sheet = formatGradingSheet({
          sample: [
            {
              ...UNQUOTED,
              sourceAnchor: 'insertion-point',
            },
            {
              ...UNQUOTED,
              issueId: 'i/2',
            },
          ],
          seed: 'meow',
          bar: 0.9,
          corpusSha: 'feedfac',
          drawDigest: 'digest-of-this-draw',
        },);
        // The candidate blocks are the whole of what this case asserts, not the header.
        /**
         Where the first candidate's block opens.
         */
        const blocksFrom = sheet.indexOf('### 1.',);
        expect(sheet.slice(blocksFrom,),).toBe([
          '### 1. grade: [ ]  (Y = real defect · N = false positive)',
          '- entry: Kitten · band: small',
          '- category: accuracy/addition · severity: major',
          '- claim: The cat naps twice.',
          '- zh source: (insertion point: the claim anchors a position in the original, which has no text at it)',
          '- en target: “The cat naps.”',
          '',
          '### 2. grade: [ ]  (Y = real defect · N = false positive)',
          '- entry: Kitten · band: small',
          '- category: accuracy/addition · severity: major',
          '- claim: The cat naps twice.',
          '- zh source: (NO source anchor: the claim points at nothing in the original, so nothing here can confirm it)',
          '- en target: “The cat naps.”',
          '',
        ].join('\n',),);
      },
    },),
  ],
},);
