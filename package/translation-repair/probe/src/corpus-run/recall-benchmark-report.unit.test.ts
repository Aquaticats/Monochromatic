/**
 Tests for what the recall benchmark prints and the one refusal that replaces
 a scorecard measuring nothing, over invented scorecards.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import type { RepairScorecard, } from '@monochromatic-dev/translation-repair-repair';
import type { BenchmarkEntry, } from '@monochromatic-dev/translation-repair-archive';
import {
  recallPlanLine,
  recallScorecardLines,
  recallStartLine,
  refuseUnmeasuredScorecard,
} from '../../dist/final/node/index.mjs';

/**
 Tip every line names.
 */
const TIP = 'c'.repeat(40,);

/**
 Entry carrying the named number of seeds.

 @param entryId - corpus id

 @param seeds - how many seeds the entry plants

 @returns Entry

 @example
 ```ts
 const entry = entryWith({ entryId: 'mittens', seeds: 2, },);
 ```
 */
function entryWith(
  {
    entryId,
    seeds,
  }: {
    readonly entryId: string;
    readonly seeds: number;
  },
): BenchmarkEntry {
  return {
    entryId,
    sourceText: '小猫打盹。\n',
    targetText: 'The kitten naps.\n',
    seeds: Array.from(
      { length: seeds, },
      function seedAt(
        _unused,
        index,
      ): BenchmarkEntry['seeds'][number] {
        return {
          id: `seed/omission-${String(index,)}`,
          category: 'accuracy/omission',
          kind: 'deletion',
          needle: 'The kitten naps.',
          replacement: '',
        };
      },
    ),
  };
}

/**
 Scorecard with every count at the given values and every rate at a fixed
 reading, so a line shows which field it printed.

 @param counts - counts a case varies

 @returns Scorecard

 @example
 ```ts
 const scorecard = scorecardWith({ dispatchedEntries: 2, plantedSeeds: 3, policyDeclinedSeeds: 0, },);
 ```
 */
function scorecardWith(
  {
    dispatchedEntries,
    plantedSeeds,
    policyDeclinedSeeds,
  }: {
    readonly dispatchedEntries: number;
    readonly plantedSeeds: number;
    readonly policyDeclinedSeeds: number;
  },
): RepairScorecard {
  return {
    dispatchedEntries,
    coverage: 0.5,
    plantedSeeds,
    detectedSeeds: 2,
    seedDetectionRate: 0.6666,
    policyDeclinedSeeds,
    seedDetectionRateExcludingPolicy: 0.75,
    nonDerivableSeeds: 1,
    seedDetectionRateExcludingUnfair: 0.8,
    judgedSeeds: 4,
    restoredSeeds: 2,
    partialSeeds: 1,
    seededRepairRate: 0.5,
    seededRepairRateLenient: 0.75,
    lexicalUniverse: 3,
    lexicalRestoredSeeds: 1,
    lexicalRepairRate: 0.3333,
    statusCounts: { unchanged: 1, },
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: recallStartLine.name,
      concurrency: 1,
      children: [
        it({
          name: 'NAMES the tip, the entries, every seed they plant, each band\'s count and the budget',
          fn: async () => {
            expect(recallStartLine({
              tip: TIP,
              choice: {
                chosen: [
                  entryWith({ entryId: 'mittens', seeds: 2, },),
                  entryWith({ entryId: 'tabby', seeds: 1, },),
                ],
                perBand: {
                  small: 1,
                  medium: 1,
                  large: 0,
                },
              },
              budgetMs: 43_200_000,
            },),).toBe(
              `START tip=${TIP} entries=2 seeds=3 perBand={"small":1,"medium":1,"large":0} budget=43200000ms`,
            );
          },
        },),

        it({
          name: 'NAMES no entry and no seed when nothing was chosen',
          fn: async () => {
            expect(recallStartLine({
              tip: TIP,
              choice: {
                chosen: [],
                perBand: {
                  small: 0,
                  medium: 0,
                  large: 0,
                },
              },
              budgetMs: 1_000,
            },),).toBe(
              `START tip=${TIP} entries=0 seeds=0 perBand={"small":0,"medium":0,"large":0} budget=1000ms`,
            );
          },
        },),
      ],
    },),

    describe({
      name: recallPlanLine.name,
      concurrency: 1,
      children: [
        it({
          name: 'LISTS the chosen entries in order, after the tip and the constructed client',
          fn: async () => {
            expect(recallPlanLine({
              tip: TIP,
              chosen: [
                entryWith({ entryId: 'mittens', seeds: 1, },),
                entryWith({ entryId: 'tabby', seeds: 1, },),
              ],
            },),).toBe(`PLAN ok tip=${TIP} client=constructed entries=mittens,tabby`,);
          },
        },),
      ],
    },),

    describe({
      name: recallScorecardLines.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS the scorecard, where it was kept, the derivability line and the repair line, with no note '
            + 'when no seed was declined for policy',
          fn: async () => {
            expect(recallScorecardLines({
              scorecard: scorecardWith({
                dispatchedEntries: 2,
                plantedSeeds: 3,
                policyDeclinedSeeds: 0,
              },),
              keptAt: '/cats/runs/recall-scorecard/stamp.json',
            },),).toEqual([
              'SCORECARD dispatched=2 coverage=0.500 planted=3 detected=2 detectionRate=0.667 policyDeclined=0 '
                + 'detectionRateExcludingPolicy=0.750',
              'SCORECARD kept at /cats/runs/recall-scorecard/stamp.json',
              'DERIVABILITY nonDerivable=1 detectionExcludingUnfair=0.800',
              'REPAIR judged=4 restored=2 partial=1 strict=0.500 lenient=0.750',
            ],);
          },
        },),

        it({
          name: 'PRINTS the policy note between the kept line and the derivability line when a seed was declined '
            + 'for policy',
          fn: async () => {
            expect(recallScorecardLines({
              scorecard: scorecardWith({
                dispatchedEntries: 2,
                plantedSeeds: 3,
                policyDeclinedSeeds: 1,
              },),
              keptAt: '/cats/runs/recall-scorecard/stamp.json',
            },),).toEqual([
              'SCORECARD dispatched=2 coverage=0.500 planted=3 detected=2 detectionRate=0.667 policyDeclined=1 '
                + 'detectionRateExcludingPolicy=0.750',
              'SCORECARD kept at /cats/runs/recall-scorecard/stamp.json',
              'NOTE policyDeclined counts seeds the panel ruled a source defect at the seed region rather than a '
                + 'translation error. Those are the policy working, not recall failing, which is why the '
                + 'excluding-policy rate sits beside the raw one instead of replacing it.',
              'DERIVABILITY nonDerivable=1 detectionExcludingUnfair=0.800',
              'REPAIR judged=4 restored=2 partial=1 strict=0.500 lenient=0.750',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: refuseUnmeasuredScorecard.name,
      concurrency: 1,
      children: [
        it({
          name: 'REFUSES a scorecard over no entry and no seed, naming both in the plural and where it is kept',
          fn: async () => {
            const refusal = caught(function refuseNothing(): unknown {
              return refuseUnmeasuredScorecard({
                scorecard: scorecardWith({
                  dispatchedEntries: 0,
                  plantedSeeds: 0,
                  policyDeclinedSeeds: 0,
                },),
                keptAt: '/cats/runs/recall-scorecard/stamp.json',
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: the bench dispatched 0 entries and planted 0 seeds, so none of its rates '
                + 'measures anything; the scorecard is kept at /cats/runs/recall-scorecard/stamp.json',
            );
          },
        },),

        it({
          name: 'REFUSES a scorecard over one entry and no seed, saying the entry in the singular',
          fn: async () => {
            const refusal = caught(function refuseNoSeed(): unknown {
              return refuseUnmeasuredScorecard({
                scorecard: scorecardWith({
                  dispatchedEntries: 1,
                  plantedSeeds: 0,
                  policyDeclinedSeeds: 0,
                },),
                keptAt: '/cats/runs/recall-scorecard/stamp.json',
              },);
            },);

            expect(String(refusal,),).toBe(
              'StatedRefusalError: the bench dispatched 1 entry and planted 0 seeds, so none of its rates '
                + 'measures anything; the scorecard is kept at /cats/runs/recall-scorecard/stamp.json',
            );
          },
        },),

        it({
          name: 'REFUSES a scorecard over no entry and one seed, saying the seed in the singular',
          fn: async () => {
            const refusal = caught(function refuseNoEntry(): unknown {
              return refuseUnmeasuredScorecard({
                scorecard: scorecardWith({
                  dispatchedEntries: 0,
                  plantedSeeds: 1,
                  policyDeclinedSeeds: 0,
                },),
                keptAt: '/cats/runs/recall-scorecard/stamp.json',
              },);
            },);

            expect(String(refusal,),).toBe(
              'StatedRefusalError: the bench dispatched 0 entries and planted 1 seed, so none of its rates '
                + 'measures anything; the scorecard is kept at /cats/runs/recall-scorecard/stamp.json',
            );
          },
        },),

        it({
          name: 'LETS a scorecard over one entry and one seed pass',
          fn: async () => {
            expect(refuseUnmeasuredScorecard({
              scorecard: scorecardWith({
                dispatchedEntries: 1,
                plantedSeeds: 1,
                policyDeclinedSeeds: 0,
              },),
              keptAt: '/cats/runs/recall-scorecard/stamp.json',
            },),).toBeUndefined();
          },
        },),
      ],
    },),
  ],
},);
