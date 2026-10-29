/**
 Tests for converting a roster's pairing into alignment steps.

 THE PROPERTY THAT MATTERS: every block appears exactly once on its own side.
 The grouper measures characters per step, so a block counted twice inflates a
 run past its budget and cuts the document somewhere it should not.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  BlockPairingError,
  blockPairingToSteps,
} from '../dist/final/node/index.mjs';

/**
 Counts how often each index appears on one side of the steps.

 @param steps - converted steps

 @param side - which side to count

 @returns Appearances per index

 @example
 ```ts
 const seen = appearances({ steps, side: 'source', },);
 ```
 */
function appearances(
  {
    steps,
    side,
  }: {
    readonly steps: readonly { readonly kind: string; readonly sourceIndex?: number; readonly targetIndex?: number; }[];
    readonly side: 'source' | 'target';
  },
): ReadonlyMap<number, number> {
  /**
   Appearances so far.
   */
  const seen = new Map<number, number>();
  for (const step of steps) {
    /**
     Index this step contributes on the side being counted.
     */
    const index = (side === 'source') ? step.sourceIndex : step.targetIndex;
    if (index === undefined)
      continue;
    seen.set(
      index,
      (seen.get(index,) ?? 0) + 1,
    );
  }
  return seen;
}

await describe({
  name: blockPairingToSteps.name,
  children: [
    it({
      name: 'CARRIES one original rendered by two translation blocks',
      fn: async () => {
        const steps = blockPairingToSteps({
          pairs: [
            {
              source: 0,
              target: 0,
            },
            {
              source: 0,
              target: 1,
            },
          ],
          sourceCount: 1,
          targetCount: 2,
        },);
        expect(steps.length,).toBe(2,);
        expect(steps[0]?.kind,).toBe('paired',);
        // The SECOND rendering carries its own text and must not re-count the
        // original's characters.
        expect(steps[1]?.kind,).toBe('target-only',);

        /**
         The continuation step, narrowed so its own field is readable.
         */
        const continuation = steps.find(function isContinuation(step,) {
          return (step.kind === 'target-only') && (step.targetIndex === 1);
        },);
        // IT MUST NOT BE CUT AWAY from the original it renders, or the critics
        // see a passage with no source beside it.
        expect((continuation?.kind === 'target-only') && continuation.continuesPairing,).toBe(true,);
      },
    },),
    it({
      name: 'COUNTS every block exactly once on its own side',
      fn: async () => {
        const steps = blockPairingToSteps({
          pairs: [
            {
              source: 0,
              target: 0,
            },
            {
              source: 2,
              target: 2,
            },
            {
              source: 2,
              target: 3,
            },
          ],
          sourceCount: 4,
          targetCount: 5,
        },);
        const sources = appearances({
          steps,
          side: 'source',
        },);
        const targets = appearances({
          steps,
          side: 'target',
        },);
        for (let at = 0; at < 4; at += 1)
          expect(sources.get(at,) ?? 0,).toBe(1,);
        for (let at = 0; at < 5; at += 1)
          expect(targets.get(at,) ?? 0,).toBe(1,);
      },
    },),
    it({
      name: 'EMITS a translation block once when a caller gives the same pair twice',
      fn: async () => {
        const steps = blockPairingToSteps({
          pairs: [
            {
              source: 0,
              target: 0,
            },
            {
              source: 0,
              target: 1,
            },
            {
              source: 0,
              target: 1,
            },
            {
              source: 1,
              target: 2,
            },
          ],
          sourceCount: 2,
          targetCount: 3,
        },);
        expect(steps,).toStrictEqual([
          {
            kind: 'paired',
            sourceIndex: 0,
            targetIndex: 0,
          },
          {
            kind: 'target-only',
            targetIndex: 1,
            continuesPairing: true,
          },
          {
            kind: 'paired',
            sourceIndex: 1,
            targetIndex: 2,
          },
        ],);
      },
    },),
    it({
      name: 'NAMES an unpaired original as source-only rather than dropping it',
      fn: async () => {
        const steps = blockPairingToSteps({
          pairs: [
            {
              source: 1,
              target: 0,
            },
          ],
          sourceCount: 2,
          targetCount: 1,
        },);
        expect(steps.some(function isSourceOnly(step,) {
          return (step.kind === 'source-only') && (step.sourceIndex === 0);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'MARKS an unpaired translation as standing alone, not continuing a pairing',
      fn: async () => {
        const steps = blockPairingToSteps({
          pairs: [
            {
              source: 0,
              target: 0,
            },
          ],
          sourceCount: 1,
          targetCount: 2,
        },);

        /**
         The genuinely unpaired translation block.
         */
        const lone = steps.find(function isLast(step,) {
          return (step.kind === 'target-only') && (step.targetIndex === 1);
        },);
        expect(lone,).toBeDefined();
        // A block nothing renders stands alone, so the grouper may cut before
        // it. Marking it cohesive would glue unrelated text into one slice.
        expect((lone?.kind === 'target-only') && (lone.continuesPairing === true),).toBe(false,);
      },
    },),
    it({
      name: 'NAMES an unpaired translation as target-only rather than dropping it',
      fn: async () => {
        const steps = blockPairingToSteps({
          pairs: [
            {
              source: 0,
              target: 0,
            },
          ],
          sourceCount: 1,
          targetCount: 3,
        },);
        expect(steps.filter(function isTargetOnly(step,) {
          return step.kind === 'target-only';
        },).length,).toBe(2,);
      },
    },),
    it({
      name: 'KEEPS document order, so a run never gathers text from two places',
      fn: async () => {
        const steps = blockPairingToSteps({
          pairs: [
            {
              source: 0,
              target: 0,
            },
            {
              source: 1,
              target: 2,
            },
          ],
          sourceCount: 2,
          targetCount: 3,
        },);

        /**
         Translation indices in the order the steps emit them.
         */
        const order = steps
          .flatMap(function toTarget(step,): readonly number[] {
            return (step.kind === 'source-only') ? [] : [ step.targetIndex, ];
          },);
        expect(order,).toEqual([
          0,
          1,
          2,
        ],);
      },
    },),
    it({
      name: 'RETURNS every block as unpaired when the roster agreed on nothing',
      fn: async () => {
        const steps = blockPairingToSteps({
          pairs: [],
          sourceCount: 2,
          targetCount: 2,
        },);
        expect(steps.length,).toBe(4,);
      },
    },),
    it({
      name: 'KEEPS an original that merges into a claimed block and then splits',
      fn: async () => {
        // THE SHAPE THAT DELETED A CLOSING MESSAGE IN PRODUCTION: the second
        // original shares the first block with the original before it, AND
        // renders further blocks of its own. Placing an original only at its
        // first rendering never placed this one, so its text left the document.
        const steps = blockPairingToSteps({
          pairs: [
            {
              source: 0,
              target: 0,
            },
            {
              source: 1,
              target: 0,
            },
            {
              source: 1,
              target: 1,
            },
            {
              source: 1,
              target: 2,
            },
          ],
          sourceCount: 2,
          targetCount: 3,
        },);

        /**
         Appearances of each original among the steps.
         */
        const sources = appearances({
          steps,
          side: 'source',
        },);
        expect(sources.get(0,),).toBe(1,);
        expect(sources.get(1,),).toBe(1,);

        /**
         Appearances of each translation block among the steps.
         */
        const targets = appearances({
          steps,
          side: 'target',
        },);
        expect(targets.get(0,),).toBe(1,);
        expect(targets.get(1,),).toBe(1,);
        expect(targets.get(2,),).toBe(1,);
      },
    },),
    it({
      name: 'KEEPS a trailing original the roster left unpaired',
      fn: async () => {
        // AN ORIGINAL NOBODY RENDERED is evidence of untranslated text, so it
        // has to reach a slice rather than vanish between alignment and the
        // lanes.
        const steps = blockPairingToSteps({
          pairs: [
            {
              source: 0,
              target: 0,
            },
          ],
          sourceCount: 2,
          targetCount: 1,
        },);
        expect(appearances({
          steps,
          side: 'source',
        },).get(1,),).toBe(1,);
      },
    },),
    it({
      name: 'PAIRS an interior gap the roster left unplaced on both sides as one merge (class one hundred twelve)',
      fn: async () => {
        // THE ROSTER PAIRED THE BLOCKS AROUND A PASSAGE AND NOTHING INSIDE IT.
        // Read as bare steps, the originals become insertions and the
        // translation blocks ride into the next paired slice, so the page
        // renders the passage once through the fold and keeps the archive's
        // rendering beside it (mikaela15, 2026-09-24). Between two paired
        // steps, unplaced originals and unplaced translation blocks belong to
        // each other: the first pair opens the rendering and the rest ride
        // along on both sides.
        const steps = blockPairingToSteps({
          pairs: [
            { source: 0, target: 0, },
            { source: 4, target: 3, },
          ],
          sourceCount: 5,
          targetCount: 4,
        },);
        expect(steps,).toEqual([
          { kind: 'paired', sourceIndex: 0, targetIndex: 0, },
          { kind: 'paired', sourceIndex: 1, targetIndex: 1, },
          { kind: 'source-only', sourceIndex: 2, continuesPairing: true, },
          { kind: 'source-only', sourceIndex: 3, continuesPairing: true, },
          { kind: 'target-only', targetIndex: 2, continuesPairing: true, },
          { kind: 'paired', sourceIndex: 4, targetIndex: 3, },
        ],);

        /**
         A leading gap stays bare: nothing paired stands before it.
         */
        const leading = blockPairingToSteps({
          pairs: [{ source: 2, target: 2, },],
          sourceCount: 3,
          targetCount: 3,
        },);
        expect(leading,).toEqual([
          { kind: 'source-only', sourceIndex: 0, },
          { kind: 'source-only', sourceIndex: 1, },
          { kind: 'target-only', targetIndex: 0, },
          { kind: 'target-only', targetIndex: 1, },
          { kind: 'paired', sourceIndex: 2, targetIndex: 2, },
        ],);

        /**
         A gap with originals alone stays bare: they are insertions.
         */
        const sourcesOnly = blockPairingToSteps({
          pairs: [
            { source: 0, target: 0, },
            { source: 2, target: 1, },
          ],
          sourceCount: 3,
          targetCount: 2,
        },);
        expect(sourcesOnly,).toEqual([
          { kind: 'paired', sourceIndex: 0, targetIndex: 0, },
          { kind: 'source-only', sourceIndex: 1, },
          { kind: 'paired', sourceIndex: 2, targetIndex: 1, },
        ],);
      },
    },),
    it({
      name: 'REFUSES a pairing that names a translation block past the last one, with the block and the count',
      fn: async () => {
        // A STORED PAIRING REACHES HERE UNCHECKED AGAINST ITS BLOCKS: an
        // artifact's recipe is read for shape and order, and carved over a
        // text whose section may hold fewer blocks than the pairing names.
        // Converted, the step named a block nobody has, the grouper dropped
        // it, and the original it paired joined whichever block stood beside
        // it.
        const refusal = caught(function convertsPastTheTargets(): unknown {
          return blockPairingToSteps({
            pairs: [{ source: 0, target: 3, },],
            sourceCount: 2,
            targetCount: 1,
          },);
        },);
        expect(refusal,).toBeInstanceOf(BlockPairingError,);
        expect(String(refusal,),).toBe('BlockPairingError: pairing names translation block 3, and there are 1',);
      },
    },),
    it({
      name: 'REFUSES a pairing that names an original block past the last one, with the block and the count',
      fn: async () => {
        const refusal = caught(function convertsPastTheOriginals(): unknown {
          return blockPairingToSteps({
            pairs: [
              { source: 0, target: 0, },
              { source: 5, target: 0, },
            ],
            sourceCount: 2,
            targetCount: 1,
          },);
        },);
        expect(refusal,).toBeInstanceOf(BlockPairingError,);
        expect(String(refusal,),).toBe('BlockPairingError: pairing names original block 5, and there are 2',);
      },
    },),
    it({
      name: 'REFUSES a negative block and a fractional one on either side, which name no block at all',
      fn: async () => {
        const negativeTarget = caught(function convertsANegativeTarget(): unknown {
          return blockPairingToSteps({
            pairs: [{ source: 0, target: -1, },],
            sourceCount: 2,
            targetCount: 1,
          },);
        },);
        expect(String(negativeTarget,),).toBe('BlockPairingError: pairing names translation block -1, and there are 1',);
        const negativeSource = caught(function convertsANegativeSource(): unknown {
          return blockPairingToSteps({
            pairs: [{ source: -1, target: 0, },],
            sourceCount: 2,
            targetCount: 1,
          },);
        },);
        expect(String(negativeSource,),).toBe('BlockPairingError: pairing names original block -1, and there are 2',);
        const fractionalTarget = caught(function convertsAFractionalTarget(): unknown {
          return blockPairingToSteps({
            pairs: [{ source: 0, target: 0.5, },],
            sourceCount: 2,
            targetCount: 1,
          },);
        },);
        expect(String(fractionalTarget,),).toBe('BlockPairingError: pairing names translation block 0.5, and there are 1',);
        const fractionalSource = caught(function convertsAFractionalSource(): unknown {
          return blockPairingToSteps({
            pairs: [{ source: 0.5, target: 0, },],
            sourceCount: 2,
            targetCount: 1,
          },);
        },);
        expect(String(fractionalSource,),).toBe('BlockPairingError: pairing names original block 0.5, and there are 2',);
      },
    },),
  ],
},);
