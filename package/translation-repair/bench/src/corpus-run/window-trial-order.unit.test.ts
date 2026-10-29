/**
 Tests for which call position each arm gets.

 WHAT THESE PIN is that the wide arm is not always the last call. It used to
 be, on every slice, which put the treatment and the position on the same
 variable: anything drifting across a slice's three back-to-back calls landed
 entirely on the wide arm, and in the direction the trial predicts, since a
 degraded round declines and a decline keeps the archive. The two narrow arms
 cannot detect that, because they sit at two positions and the wide arm sits at
 a third neither of them ever occupies.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup';
import {
  armOrderFor,
  TRIAL_ARM_SET,
  TRIAL_ARMS,
} from '../../dist/final/node/index.mjs';

/**
 Slices to draw orders over, enough for every position to appear.
 */
const SLICE_COUNT = 60;

/**
 Orders two positions smallest first.

 @param left - one position

 @param right - other position

 @returns Negative when `left` comes first

 @example
 ```ts
 const ordered = positions.toSorted(ascending,);
 ```
 */
function ascending(
  left: number,
  right: number,
): number {
  return left - right;
}

/**
 Orders two arm names by code point, the same on every machine; this said
 code unit while it asked the runtime's locale (ledger B95).

 @param left - one arm

 @param right - other arm

 @returns Negative when `left` comes first

 @example
 ```ts
 const ordered = arms.toSorted(alphabetical,);
 ```
 */
function alphabetical(
  left: string,
  right: string,
): number {
  return compareCodePoints({
    left,
    right,
  },);
}

await describe({
  name: armOrderFor.name,
  children: [
    it({
      name: 'PUTS THE WIDE ARM AT EVERY POSITION across a draw, which is the whole point: with it '
        + 'fixed last, any drift across a slice\'s three calls is the same variable as the '
        + 'treatment and no statistic separates them afterwards',
      fn: async () => {
        /**
         Where the wide arm landed for each of many slices.
         */
        const positions = new Set(Array.from(
          { length: SLICE_COUNT, },
          function toPosition(
            _unused,
            sliceIndex,
          ) {
            return armOrderFor({
              protocol: 'protocol-one',
              entryId: 'Mittens',
              sliceIndex,
            },)
              .indexOf(TRIAL_ARMS.wide,);
          },
        ),);

        expect([...positions,].toSorted(ascending,),).toEqual([0,
          1,
          2,],);
      },
    },),
    it({
      name: 'returns all three arms exactly once, whatever the digest said, so no slice can buy '
        + 'one arm twice and none at all',
      fn: async () => {
        for (let sliceIndex = 0; sliceIndex < SLICE_COUNT; sliceIndex += 1) {
          /**
           This slice's order.
           */
          const order = armOrderFor({
            protocol: 'protocol-one',
            entryId: 'Mittens',
            sliceIndex,
          },);
          expect(order.length,).toBe(TRIAL_ARM_SET.length,);
          expect([...new Set(order,),].toSorted(alphabetical,),)
            .toEqual([...TRIAL_ARM_SET,].toSorted(alphabetical,),);
        }
      },
    },),
    it({
      name: 'is DETERMINISTIC, so a resumed run and a rerun for verification assign the same slice '
        + 'the same order rather than quietly measuring a different thing the second time',
      fn: async () => {
        expect(armOrderFor({
          protocol: 'protocol-one',
          entryId: 'Mittens',
          sliceIndex: 7,
        },),).toEqual(armOrderFor({
          protocol: 'protocol-one',
          entryId: 'Mittens',
          sliceIndex: 7,
        },),);
      },
    },),
    it({
      name: 'keeps the narrow arms in their relative order wherever they land, which costs nothing '
        + 'since being the same treatment twice is their entire purpose',
      fn: async () => {
        for (let sliceIndex = 0; sliceIndex < SLICE_COUNT; sliceIndex += 1) {
          /**
           This slice's order.
           */
          const order = armOrderFor({
            protocol: 'protocol-one',
            entryId: 'Mittens',
            sliceIndex,
          },);
          expect(order.indexOf(TRIAL_ARMS.narrowFirst,)
            < order.indexOf(TRIAL_ARMS.narrowSecond,),).toBe(true,);
        }
      },
    },),
    it({
      name: 'reshuffles when the protocol moves, so a rerun under a new roster or corpus pin does '
        + 'not repeat one assignment forever and carry its position effects with it',
      fn: async () => {
        /**
         Wide positions under each protocol, over the same slices.
         */
        const under = ['protocol-one',
          'protocol-two',].map(function toPositions(protocol,) {
            return Array.from(
              { length: SLICE_COUNT, },
              function toPosition(
                _unused,
                sliceIndex,
              ) {
                return armOrderFor({
                  protocol,
                  entryId: 'Mittens',
                  sliceIndex,
                },)
                  .indexOf(TRIAL_ARMS.wide,);
              },
            );
          },);

        expect(under[0],).not
          .toEqual(under[1],);
      },
    },),
  ],
},);
