/**
 Tests that each bench has the floor the stages leaning on it decide at, and that a return is read as a lift over the
 floor, over the quorum, over both or over neither.

 Fixtures are cat-themed invention; readings are scripted and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  type BenchName,
  benchFloorOf,
  liftOf,
} from '../../dist/final/node/index.mjs';

//region Bench floor tests

/**
 Reading of a wide bench with the stated seats reachable.

 @param reachable - how many of the three seats a wet provider serves
 @param quorum - voices the bench settles on

 @returns Reading

 @example
 ```ts
 const reading = readingOf({ reachable: 1, quorum: 4, },);
 ```
 */
function readingOf(
  {
    reachable,
    quorum,
  }: {
    readonly reachable: number;
    readonly quorum: number;
  },
): Parameters<typeof liftOf>[0]['now'] {
  /**
   Seats the bench holds, in bench order.
   */
  const seats = [
    SEAT_HYPER_ONLY,
    SEAT_HYPER_VISION,
    SEAT_SYNTHETIC_VISION_WITHHELD,
  ] as const;
  return {
    name: 'wide',
    seats,
    quorum,
    reachable: seats.slice(0, reachable,),
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: `${benchFloorOf.name} and ${liftOf.name}`,
      concurrency: 1,
      children: [
        it({
          name: 'GIVES EVERY BENCH THE FLOOR OF THE STAGES THAT LEAN ON IT: two voices for the wide, select, slate and '
            + 'writing benches, and the reading pair for the readers',
          fn: async () => {
            /**
             Every bench name, in the order the lines print them.
             */
            const names: readonly BenchName[] = [
              'wide',
              'select',
              'slate',
              'editors',
              'refiners',
              'translators',
              'readers',
            ];
            expect(names.map(function floorOfName(name,): number {
              return benchFloorOf({ name, },);
            },),).toEqual([2, 2, 2, 2, 2, 2, 2,],);
          },
        },),
        it({
          name: 'READS A RETURN THAT LIFTS A BENCH OVER ITS FLOOR BUT NOT TO ITS QUORUM as a lift to the floor only: '
            + 'one reachable seat of three with a quorum of four becomes two, which is the floor and short of the '
            + 'quorum',
          fn: async () => {
            expect(liftOf({
              now: readingOf({ reachable: 1, quorum: 4, },),
              after: readingOf({ reachable: 2, quorum: 4, },),
            },),).toEqual({ toFloor: true, toQuorum: false, },);
          },
        },),
        it({
          name: 'READS A RETURN THAT LIFTS A BENCH TO ITS QUORUM as a lift over both thresholds when the bench stood '
            + 'under the floor, and as a lift to the quorum alone when it already stood over the floor',
          fn: async () => {
            expect({
              fromUnderFloor: liftOf({
                now: readingOf({ reachable: 1, quorum: 3, },),
                after: readingOf({ reachable: 3, quorum: 3, },),
              },),
              fromOverFloor: liftOf({
                now: readingOf({ reachable: 2, quorum: 3, },),
                after: readingOf({ reachable: 3, quorum: 3, },),
              },),
            },).toEqual({
              fromUnderFloor: { toFloor: true, toQuorum: true, },
              fromOverFloor: { toFloor: false, toQuorum: true, },
            },);
          },
        },),
        it({
          name: 'READS A RETURN THAT CROSSES NEITHER THRESHOLD as no lift: a bench that stays under the floor, and a '
            + 'bench that already reads its quorum',
          fn: async () => {
            expect({
              staysUnder: liftOf({
                now: readingOf({ reachable: 0, quorum: 4, },),
                after: readingOf({ reachable: 1, quorum: 4, },),
              },),
              alreadyFull: liftOf({
                now: readingOf({ reachable: 3, quorum: 3, },),
                after: readingOf({ reachable: 3, quorum: 3, },),
              },),
            },).toEqual({
              staysUnder: { toFloor: false, toQuorum: false, },
              alreadyFull: { toFloor: false, toQuorum: false, },
            },);
          },
        },),
      ],
    },),
  ],
},);

//endregion Bench floor tests
