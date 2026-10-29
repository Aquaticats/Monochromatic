/**
 Tests for the warning the read step prints when the default effort fixes nine in ten of the units the arms share or more:
 the rule states its false pick bounds for true fix rates up to 0.90, above which the false pick chance reached 0.108 at
 0.91 and 0.13 and more from 0.93 in the simulation and the repeat warning can miss unequal repeat agreement, so a verdict
 of high or low there stands under a caveat. The share observed is counted over the units every arm holds, compared by
 cross-multiplying the counts, and printed for each arm of the default effort.

 The logs are the arm-log fixture's, written by the probe's own writers. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArmLog,
  fixRateLinesOf,
} from '../../dist/final/node/index.mjs';
import {
  HIGH_CARD,
  LOW_CARD,
  MEDIUM_CARD,
} from '../editor-fidelity.test-fixture.ts';
import {
  armOf,
  walkOf,
} from './editor-fidelity-arm-walk.test-fixture.ts';
import { readAsBuilt, } from './editor-fidelity-arm-read-built.test-fixture.ts';

/**
 What the warning of an arm of the default effort ends with: what the repeat warning can miss at this rate and what the
 committed fixture read.
 */
const WARNING_END = 'at fix rates this high the repeat warning can miss unequal repeat agreement, so any verdict of high or '
  + 'low stands under the caveat that its chance of being a false pick can be above 0.05, whether or not a WARN repeat '
  + 'agreement line printed; the committed fixture read 0.108 at 0.91, 0.123 at 0.92, about 0.13 at 0.93 and 0.175 at '
  + '0.95 when the default effort repeats itself far less than the others';

/**
 A bench of three arms over the same units, the default effort fixing the leading ones.

 @param units - how many units each arm walks
 @param mediumFixed - how many leading units the default effort fixes
 @param othersFixed - how many leading units the high and the low effort fix

 @returns Three logs, high first

 @example
 ```ts
 const arms = benchOf({ units: 100, mediumFixed: 93, othersFixed: 100, },);
 ```
 */
function benchOf(
  {
    units,
    mediumFixed,
    othersFixed,
  }: {
    readonly units: number;
    readonly mediumFixed: number;
    readonly othersFixed: number;
  },
): readonly ArmLog[] {
  return [
    armOf({
      arm: 'high',
      cardId: HIGH_CARD,
      rows: walkOf({ units, fixedAt: ({ unit, },) => unit < othersFixed, },),
      countedUsd: 2.5,
    },),
    armOf({
      arm: 'medium',
      cardId: MEDIUM_CARD,
      rows: walkOf({ units, fixedAt: ({ unit, },) => unit < mediumFixed, },),
      declaredUsd: 2.5,
      countedUsd: 1.5,
    },),
    armOf({
      arm: 'low',
      cardId: LOW_CARD,
      rows: walkOf({ units, fixedAt: ({ unit, },) => unit < othersFixed, },),
      declaredUsd: 4,
      countedUsd: 0.875,
    },),
  ];
}

await describe({
  name: 'the high fix rate warning of the read step',
  children: [
    describe({
      name: fixRateLinesOf.name,
      children: [
        it({
          name: 'WARNS AT 90 PERCENT OF THE SHARED UNITS AND NOT AT 89, the line where the rule\'s stated region of true fix '
            + 'rates ends, counting exactly: 90 of 100 units warns and says how many of how many, and 89 of 100 says nothing',
          fn: async () => {
            expect(fixRateLinesOf({ arms: benchOf({ units: 100, mediumFixed: 90, othersFixed: 100, },), },),).toStrictEqual([
              `WARN high fix rate: medium cleanly fixed 90 of 100 shared units (90 percent), at or above 90 percent; ${WARNING_END}`,
            ],);
            expect(fixRateLinesOf({ arms: benchOf({ units: 100, mediumFixed: 89, othersFixed: 100, },), },),).toStrictEqual([],);
          },
        },),

        it({
          name: 'COUNTS BY CROSS-MULTIPLYING ON A BENCH OF ONE WHOLE PASS: 80 of 88 units is 90.9 percent and warns, 79 of 88 '
            + 'is 89.8 percent and does not, and 82 of 88 (93.2 percent) still does',
          fn: async () => {
            expect(fixRateLinesOf({ arms: benchOf({ units: 88, mediumFixed: 80, othersFixed: 88, },), },),).toHaveLength(1,);
            expect(fixRateLinesOf({ arms: benchOf({ units: 88, mediumFixed: 79, othersFixed: 88, },), },),).toStrictEqual([],);
            expect(fixRateLinesOf({ arms: benchOf({ units: 88, mediumFixed: 82, othersFixed: 88, },), },),).toHaveLength(1,);
          },
        },),

        it({
          name: 'WARNS OF THE DEFAULT EFFORT ONLY: the high and the low effort fixing every unit while the default effort fixes '
            + 'half says nothing',
          fn: async () => {
            expect(fixRateLinesOf({ arms: benchOf({ units: 100, mediumFixed: 50, othersFixed: 100, },), },),).toStrictEqual([],);
          },
        },),

        it({
          name: 'COUNTS ONLY THE UNITS EVERY ARM HOLDS: the default effort fixes 133 of the 150 units the arms share, which is '
            + 'under 90 percent, though it fixes 93.6 percent of the 264 it walked',
          fn: async () => {
            /**
             The high and the default effort walk 264 units, the low effort the leading 150 of them.
             */
            const arms = [
              armOf({ arm: 'high', cardId: HIGH_CARD, rows: walkOf({ units: 264, fixedAt: () => true, },), countedUsd: 2.5, },),
              armOf({
                arm: 'medium',
                cardId: MEDIUM_CARD,
                rows: walkOf({ units: 264, fixedAt: ({ unit, },) => (unit >= 150) || (unit < 133), },),
                declaredUsd: 2.5,
                countedUsd: 1.5,
              },),
              armOf({
                arm: 'low',
                cardId: LOW_CARD,
                rows: walkOf({ units: 150, fixedAt: () => true, },),
                declaredUsd: 4,
                countedUsd: 0.875,
              },),
            ];
            expect(fixRateLinesOf({ arms, },),).toStrictEqual([],);
          },
        },),

        it({
          name: 'SAYS NOTHING WHEN THE LOGS SHARE NO UNIT, one log read alone or two without a low effort among them, as the '
            + 'runbook reads the high arm before the others exist',
          fn: async () => {
            const [high, medium,] = benchOf({ units: 100, mediumFixed: 100, othersFixed: 100, },);
            expect(fixRateLinesOf({ arms: [high as ArmLog,], },),).toStrictEqual([],);
            expect(fixRateLinesOf({ arms: [high as ArmLog, medium as ArmLog,], },),).toStrictEqual([],);
          },
        },),

        it({
          name: 'WARNS OF EACH ARM OF THE DEFAULT EFFORT THAT FIXES 90 PERCENT, naming it, when a second medium arm is read',
          fn: async () => {
            const [high, medium, low,] = benchOf({ units: 100, mediumFixed: 100, othersFixed: 100, },);
            /**
             A second arm of the default effort that fixes 89 of the 100 units.
             */
            const second = armOf({
              arm: 'medium-b',
              cardId: MEDIUM_CARD,
              rows: walkOf({ units: 100, fixedAt: ({ unit, },) => unit < 89, },),
              declaredUsd: 3,
              countedUsd: 1.5,
            },);
            const said = fixRateLinesOf({ arms: [high as ArmLog, medium as ArmLog, second, low as ArmLog,], },);
            expect(said,).toHaveLength(1,);
            expect(said[0],).toContain('WARN high fix rate: medium cleanly fixed 100 of 100 shared units (100 percent)',);
          },
        },),
      ],
    },),

    describe({
      name: 'the high fix rate warning as the built read step prints it',
      children: [
        it({
          name: 'PRINTS THE WARNING AFTER THE REPEAT LINES AND BEFORE THE COUNT OF CHECKS, over the logs of one whole pass in which '
            + 'the default effort fixes 80 of 88 units, and still ends on the verdict',
          fn: async () => {
            const run = await readAsBuilt({
              arms: benchOf({ units: 88, mediumFixed: 80, othersFixed: 88, },),
              prefix: 'editor-fidelity-fix-rate-',
            },);
            expect(run.code,).toBe(0,);
            const repeatAt = run.stdout.findIndex(function isRepeat(line,): boolean {
              return line.startsWith('REPEAT high: ',);
            },);
            expect(run.stdout[repeatAt + 3]?.startsWith('WARN high fix rate: medium cleanly fixed 80 of 88 shared units (91 percent)',),).toBe(true,);
            expect(run.stdout[repeatAt + 4]?.startsWith('CHECKS ',),).toBe(true,);
            expect(run.stdout.at(-1,)?.startsWith('VERDICT ',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
