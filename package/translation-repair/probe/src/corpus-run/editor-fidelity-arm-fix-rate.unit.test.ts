/**
 Tests for the warning the read step prints when the default effort fixes nearly every unit the arms share: at a true
 fix rate of 0.93 and above the repeat warning cannot see unequal repeat agreement, and the false pick chance with it
 reached 0.13 and more in the simulation, so a verdict of high or low there stands under a caveat. The share is counted
 over the units every arm holds, compared by cross-multiplying the counts, and printed for each arm of the default
 effort.

 The logs are the arm-log fixture's, written by the probe's own writers. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

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
 What the warning of an arm of the default effort ends with: what the repeat warning cannot see at this rate and what
 the simulation reached.
 */
const WARNING_END = 'the repeat warning cannot see unequal repeat agreement at fix rates this high, so any verdict of high or '
  + 'low stands under the caveat that its chance of being a false pick can be above 0.05, whether or not a WARN repeat '
  + 'agreement line printed; the simulation reached 0.142 at 0.93, 0.173 at 0.95 and 0.241 at 0.97 when the default '
  + 'effort repeats itself far less than the others';

/**
 A bench of three arms over the same units, the default effort fixing the leading ones.

 @param units - how many units each arm walks
 @param mediumFixed - how many leading units the default effort fixes
 @param othersFixed - how many leading units the high and the low effort fix

 @returns The three logs, high first

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
          name: 'WARNS AT 93 PERCENT OF THE SHARED UNITS AND NOT AT 92, counting exactly: 93 of 100 units warns and says how '
            + 'many of how many, and 92 of 100 says nothing',
          fn: async () => {
            expect(fixRateLinesOf({ arms: benchOf({ units: 100, mediumFixed: 93, othersFixed: 100, },), },),).toStrictEqual([
              `WARN high fix rate: medium cleanly fixed 93 of 100 shared units (93 percent), at or above 93 percent; ${WARNING_END}`,
            ],);
            expect(fixRateLinesOf({ arms: benchOf({ units: 100, mediumFixed: 92, othersFixed: 100, },), },),).toStrictEqual([],);
          },
        },),

        it({
          name: 'COUNTS BY CROSS-MULTIPLYING ON A BENCH OF ONE WHOLE PASS: 82 of 88 units is under 93.2 percent and warns, 81 of 88 '
            + 'is under 92.1 percent and does not',
          fn: async () => {
            expect(fixRateLinesOf({ arms: benchOf({ units: 88, mediumFixed: 82, othersFixed: 88, },), },),).toHaveLength(1,);
            expect(fixRateLinesOf({ arms: benchOf({ units: 88, mediumFixed: 81, othersFixed: 88, },), },),).toStrictEqual([],);
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
          name: 'COUNTS ONLY THE UNITS EVERY ARM HOLDS: the default effort fixes 139 of the 150 units the arms share, which is '
            + 'under 93 percent, though it fixes 95.8 percent of the 264 it walked',
          fn: async () => {
            /**
             The high and the default effort walk 264 units, the low effort the leading 150 of them.
             */
            const arms = [
              armOf({ arm: 'high', cardId: HIGH_CARD, rows: walkOf({ units: 264, fixedAt: () => true, },), countedUsd: 2.5, },),
              armOf({
                arm: 'medium',
                cardId: MEDIUM_CARD,
                rows: walkOf({ units: 264, fixedAt: ({ unit, },) => (unit >= 150) || (unit < 139), },),
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
          name: 'WARNS OF EACH ARM OF THE DEFAULT EFFORT THAT FIXES 93 PERCENT, naming it, when a second medium arm is read',
          fn: async () => {
            const [high, medium, low,] = benchOf({ units: 100, mediumFixed: 100, othersFixed: 100, },);
            /**
             A second arm of the default effort that fixes 90 of the 100 units.
             */
            const second = armOf({
              arm: 'medium-b',
              cardId: MEDIUM_CARD,
              rows: walkOf({ units: 100, fixedAt: ({ unit, },) => unit < 90, },),
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
            + 'the default effort fixes 82 of 88 units, and still ends on the verdict',
          fn: async () => {
            const run = await readAsBuilt({
              arms: benchOf({ units: 88, mediumFixed: 82, othersFixed: 88, },),
              prefix: 'editor-fidelity-fix-rate-',
            },);
            expect(run.code,).toBe(0,);
            const repeatAt = run.stdout.findIndex(function isRepeat(line,): boolean {
              return line.startsWith('REPEAT high: ',);
            },);
            expect(run.stdout[repeatAt + 3]?.startsWith('WARN high fix rate: medium cleanly fixed 82 of 88 shared units (93 percent)',),).toBe(true,);
            expect(run.stdout[repeatAt + 4]?.startsWith('CHECKS ',),).toBe(true,);
            expect(run.stdout.at(-1,)?.startsWith('VERDICT ',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
