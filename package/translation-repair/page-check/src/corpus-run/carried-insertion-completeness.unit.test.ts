/**
 Tests final boundary for passages proven rendered elsewhere before lanes.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { TranslationRepairInterruptedError, } from '@monochromatic-dev/translation-repair-translate-stage';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  assertCarriedInsertionsRemain,
  type WouldShipSource,
} from '../../dist/final/node/index.mjs';
import {
  sourceShippedArchive,
  sourceShippedArtifact,
  type SourceShippedRow,
  sourceShippedSlices,
} from './source-shipped.test-fixture.ts';

/**
 Archive wording carrying proof region.
 */
const TARGET = 'The cat sleeps in warm sunlight.\n';

/**
 One content slice spanning whole target.
 */
const SLICES: readonly ChunkPair[] = [{
  source: {
    sliceIndex: 0,
    nodes: [],
    startOffset: 0,
    endOffset: 2,
    text: '猫。',
  },
  target: {
    sliceIndex: 0,
    nodes: [],
    startOffset: 0,
    endOffset: TARGET.length,
    text: TARGET,
  },
},];

/**
 Builds final-stage source selecting one replacement.

 @param text - wording final contest selects

 @returns Narrow artifact source read by publication assembler

 @example
 ```ts
 const artifact = artifactShipping({ text: TARGET, });
 ```
 */
function artifactShipping({ text, }: { readonly text: string; },): WouldShipSource {
  return {
    comparison: [{
      sliceIndex: 0,
      incumbentKind: 'present',
      incumbentText: TARGET,
      repairText: TARGET,
      translateText: text,
      laneRelation: 'both-differ',
      repairOutcome: { kind: 'decided', acceptedText: TARGET, },
      translateOutcome: { kind: 'decided', acceptedText: text, },
      decisionComparison: { kind: 'comparable', verdict: 'different', },
      repairDelivery: { kind: 'incumbent-retained', },
      translateDelivery: { kind: 'replacement-shipped', },
    },],
    consolidation: { kind: 'not-run', },
    laneSelection: {
      kind: 'contested',
      slices: [{
        sliceIndex: 0,
        verdict: { kind: 'lane-won', lane: 'translate', },
        ballots: [],
        usable: 3,
      },],
    },
  } as unknown as WouldShipSource;
}

/**
 Carried-complete proof shared by cases.
 */
const CARRIED = [{
  position: 1,
  sliceIndex: 1,
  sourceText: '猫在阳光下睡觉。',
  evidence: ['cat sleeps in warm sunlight',],
},];

await describe({
  name: assertCarriedInsertionsRemain.name,
  children: [
    it({
      name: 'ACCEPTS final page retaining exact carried region',
      fn: async () => {
        expect(() => assertCarriedInsertionsRemain({
          artifact: artifactShipping({ text: TARGET, }),
          slices: SLICES,
          targetText: TARGET,
          carried: CARRIED,
        },),).not.toThrow();
      },
    },),
    it({
      name: 'ACCEPTS a page whose semantic wrap broke the carried region across lines, reading it as the '
        + 'anchoring did (the fifth yuki418330012 launch of 2026-09-08 shipped the credits line as four lines)',
      fn: async () => {
        expect(() => assertCarriedInsertionsRemain({
          artifact: artifactShipping({ text: 'The cat sleeps\nin warm sunlight.\n', }),
          slices: SLICES,
          targetText: TARGET,
          carried: CARRIED,
        },),).not.toThrow();
      },
    },),
    it({
      name: 'PAUSES when final stage removes carried region instead of publishing omission',
      fn: async () => {
        expect(() => assertCarriedInsertionsRemain({
          artifact: artifactShipping({ text: 'The cat waits by the window.\n', }),
          slices: SLICES,
          targetText: TARGET,
          carried: CARRIED,
        },),).toThrow(TranslationRepairInterruptedError,);
      },
    },),
    it({
      name: 'PAUSES when only the original an unfilled anchor ships carries the region, since that original is no '
        + 'wording of the pipeline and keeps no passage the lanes admitted',
      fn: async () => {
        /**
         An archive slice without the region, and an anchor nobody filled whose original holds it.
         */
        const rows: readonly SourceShippedRow[] = [
          {
            incumbentText: 'The cat waits by the window.\n',
            sourceText: '猫在等。\n',
          },
          {
            incumbentText: '',
            sourceText: 'The cat sleeps in warm sunlight.\n',
          },
        ];
        expect(() => assertCarriedInsertionsRemain({
          artifact: sourceShippedArtifact({ rows, },),
          slices: sourceShippedSlices({ rows, },),
          targetText: sourceShippedArchive({ rows, },),
          carried: CARRIED,
        },),).toThrow(TranslationRepairInterruptedError,);
      },
    },),
    it({
      name: 'ACCEPTS a page with no carried passage whatever it ships, since there is no region to lose',
      fn: async () => {
        expect(() => assertCarriedInsertionsRemain({
          artifact: artifactShipping({ text: 'The cat waits by the window.\n', }),
          slices: SLICES,
          targetText: TARGET,
          carried: [],
        },),).not.toThrow();
      },
    },),
    it({
      name: 'PAUSES for a carried passage admitted with no region recorded, naming it as having none',
      fn: async () => {
        const interruption = caught(function act(): unknown {
          return assertCarriedInsertionsRemain({
            artifact: artifactShipping({ text: TARGET, }),
            slices: SLICES,
            targetText: TARGET,
            carried: [{
              position: 1,
              sliceIndex: 1,
              sourceText: '猫在阳光下睡觉。',
              evidence: [],
            },],
          },);
        },);
        if (!(interruption instanceof TranslationRepairInterruptedError))
          throw new Error('expected the passage with no region to pause the page',);
        expect(interruption.reason,).toBe('carried-evidence-lost',);
        expect(interruption.findings,).toEqual([
          'carried-insertion-evidence-lost (count 1)',
          'carried-insertion-evidence-lost slice 1: "(no region recorded)"',
        ],);
      },
    },),
  ],
},);
