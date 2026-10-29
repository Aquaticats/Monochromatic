/**
 Tests the page-level footnote guard and the page guards it runs beside.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { TranslationRepairInterruptedError, } from '@monochromatic-dev/translation-repair-translate-stage';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  assertPageFootnotesIntact,
  type WouldShipSource,
} from '@monochromatic-dev/translation-repair-page-check';
import { assertPageGuards, } from '../../dist/final/node/index.mjs';

/**
 Archive wording with no footnote at all.
 */
const TARGET = 'The cat sleeps in warm sunlight.\n';

/**
 Archive wording that already refers to a note it never defines.
 */
const DANGLING = 'The cat sleeps in warm sunlight[^1].\n';

/**
 One content slice spanning the whole target.

 @param targetText - archive the slice covers

 @returns Single-slice preparation

 @example
 ```ts
 const slices = slicesOver({ targetText: TARGET, },);
 ```
 */
function slicesOver({ targetText, }: { readonly targetText: string; },): readonly ChunkPair[] {
  return [{
    source: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: 2,
      text: '猫。',
    },
    target: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: targetText.length,
      text: targetText,
    },
  },];
}

/**
 Builds a final-stage source selecting one replacement over one slice.

 @param incumbent - archive text of the slice

 @param text - wording the final contest selects

 @returns Narrow artifact source read by the publication assembler

 @example
 ```ts
 const artifact = artifactShipping({ incumbent: TARGET, text: TARGET, });
 ```
 */
function artifactShipping(
  {
    incumbent,
    text,
  }: {
    readonly incumbent: string;
    readonly text: string;
  },
): WouldShipSource {
  return {
    comparison: [{
      sliceIndex: 0,
      incumbentKind: 'present',
      incumbentText: incumbent,
      repairText: incumbent,
      translateText: text,
      laneRelation: 'both-differ',
      repairOutcome: { kind: 'decided', acceptedText: incumbent, },
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
 Reads the interruption a guard throws, or fails the case.

 @param run - guard call under test

 @returns Interruption thrown

 @example
 ```ts
 const error = interruptionOf({ run: () => assertPageFootnotesIntact({ ... },), },);
 ```
 */
function interruptionOf({ run, }: { readonly run: () => void; },): TranslationRepairInterruptedError {
  try {
    run();
  } catch (error) {
    if (error instanceof TranslationRepairInterruptedError)
      return error;
    throw error;
  }
  throw new Error('the guard did not throw',);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assertPageFootnotesIntact.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a page whose footnote graph is as whole as the archive',
          fn: async () => {
            expect(() => assertPageFootnotesIntact({
              artifact: artifactShipping({ incumbent: TARGET, text: 'The cat naps in warm sunlight.\n', },),
              slices: slicesOver({ targetText: TARGET, },),
              targetText: TARGET,
            },),).not.toThrow();
          },
        },),
        it({
          name: 'PAUSES a page that ships a reference with no note the archive never carried (the nineteenth '
            + 'hakureico pass of 2026-09-09 shipped “Mayday”[^1] with no [^1] definition anywhere)',
          fn: async () => {
            /**
             The interruption the guard raises.
             */
            const error = interruptionOf({
              run: () => assertPageFootnotesIntact({
                artifact: artifactShipping({ incumbent: TARGET, text: DANGLING, },),
                slices: slicesOver({ targetText: TARGET, },),
                targetText: TARGET,
              },),
            },);
            expect(error.reason,).toBe('page-footnote-integrity',);
            expect(error.findings,).toEqual([
              'page-footnote-integrity (count 1)',
              'page-footnote-unresolved-reference gfm 1',
            ],);
          },
        },),
        it({
          name: 'ACCEPTS a page that carries the same dangling reference the archive already carried, since a '
            + 'defect the archive brought is never blamed on the page',
          fn: async () => {
            expect(() => assertPageFootnotesIntact({
              artifact: artifactShipping({ incumbent: DANGLING, text: 'The cat naps in warm sunlight[^1].\n', },),
              slices: slicesOver({ targetText: DANGLING, },),
              targetText: DANGLING,
            },),).not.toThrow();
          },
        },),
      ],
    },),

    describe({
      name: assertPageGuards.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'runs the footnote guard after the carried guard, so a page that passes the carried check still '
            + 'pauses on a dangling reference',
          fn: async () => {
            /**
             The interruption the guards raise.
             */
            const error = interruptionOf({
              run: () => assertPageGuards({
                artifact: artifactShipping({ incumbent: TARGET, text: DANGLING, },),
                slices: slicesOver({ targetText: TARGET, },),
                targetText: TARGET,
                carried: [{
                  position: 1,
                  sliceIndex: 1,
                  sourceText: '猫在阳光下睡觉。',
                  evidence: ['cat sleeps in warm sunlight',],
                },],
              },),
            },);
            expect(error.reason,).toBe('page-footnote-integrity',);
          },
        },),
        it({
          name: 'raises the carried guard first when the page lost its carried region',
          fn: async () => {
            /**
             The interruption the guards raise.
             */
            const error = interruptionOf({
              run: () => assertPageGuards({
                artifact: artifactShipping({ incumbent: TARGET, text: 'The cat waits by the window[^1].\n', },),
                slices: slicesOver({ targetText: TARGET, },),
                targetText: TARGET,
                carried: [{
                  position: 1,
                  sliceIndex: 1,
                  sourceText: '猫在阳光下睡觉。',
                  evidence: ['cat sleeps in warm sunlight',],
                },],
              },),
            },);
            expect(error.reason,).toBe('carried-evidence-lost',);
          },
        },),
      ],
    },),
  ],
},);
