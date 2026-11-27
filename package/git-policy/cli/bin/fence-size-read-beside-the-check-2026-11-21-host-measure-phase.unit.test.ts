import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  durationsOf,
  median,
  orderOfRound,
  refusedOrderOfRound,
  type Row,
  rowAsWritten,
  summaryOfAdds,
  summaryOfRefusedAdds,
} from './fence-size-read-beside-the-check-2026-11-21-host-measure-phase.ts';

/**
 The arms one round of the recorded measurement took, in the order the measurement names them.

 @returns the arm names, which every order below is read against
 */
function armNames(): readonly string[] {
  return ['main', 'branch-serial', 'branch-overlap', 'branch-serial-again',];
}

/**
 One counted row of an arm that landed, which the summaries are read from.

 @param arm - the arm the add ran

 @param totalMs - the add's own total, from its record

 @param failed - whether the add failed

 @returns the row, with the steps and work the summaries read
 */
function countedRow({
  arm,
  totalMs,
  failed = false,
}: {
  readonly arm: string;
  readonly totalMs: number;
  readonly failed?: boolean;
}): Row {
  return {
    layout: 'loose',
    arm,
    overlap: false,
    tag: `r1p1-${arm}`,
    counted: true,
    wallMs: totalMs,
    totalMs,
    route: 'clone',
    exitCode: 0,
    failed,
    stderr: '',
    steps: {
      'git ls-tree': 2,
      checkSource: 1,
    },
    work: { fenceSizesBesideTheCheck: 3, },
    round: 1,
    position: 1,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: durationsOf.name,
      children: [
        it({
          name: 'read the time of each name of a record, in the order the record lists them',
          fn: async () => {
            expect(durationsOf({
              spent: {
                'git ls-tree': {
                  ms: 12.5,
                  calls: 1,
                },
                checkSource: {
                  ms: 0.25,
                  calls: 2,
                },
              },
            },),).toEqual({
              'git ls-tree': 12.5,
              checkSource: 0.25,
            },);
          },
        }),
      ],
    },),
    describe({
      name: median.name,
      children: [
        it({
          name: 'answer the middle value of an odd list, whatever order it was measured in',
          fn: async () => {
            expect(median({ values: [30, 10, 20,], },),).toBe(20,);
          },
        }),
        it({
          name: 'answer the upper of the two middle values of an even list',
          fn: async () => {
            expect(median({ values: [40, 10, 30, 20,], },),).toBe(30,);
          },
        }),
        it({
          name: 'answer no figure for a list with nothing in it, as a summary reads a total nobody recorded',
          fn: async () => {
            expect(median({ values: [], },),).toBeNaN();
          },
        }),
      ],
    },),
    describe({
      name: orderOfRound.name,
      children: [
        it({
          name: 'take the arms as they are named in the first round',
          fn: async () => {
            expect(orderOfRound({
              round: 0,
              armNames: armNames(),
            },),).toEqual(armNames(),);
          },
        }),
        it({
          name: 'rotate the arms by the round number, so no arm keeps a place',
          fn: async () => {
            expect(orderOfRound({
              round: 1,
              armNames: armNames(),
            },),).toEqual(['branch-serial', 'branch-overlap', 'branch-serial-again', 'main',],);
          },
        }),
        it({
          name: 'take the arms the other way round in every second cycle, so a warmed page cache is no arm\'s advantage',
          fn: async () => {
            expect(orderOfRound({
              round: 4,
              armNames: armNames(),
            },),).toEqual([...armNames(),].toReversed(),);
          },
        }),
      ],
    },),
    describe({
      name: refusedOrderOfRound.name,
      children: [
        it({
          name: 'take the arms as they are named in the first round',
          fn: async () => {
            expect(refusedOrderOfRound({
              round: 0,
              armNames: armNames(),
            },),).toEqual(armNames(),);
          },
        }),
        it({
          name: 'rotate the arms and never reverse them, which is how the refused rounds were made',
          fn: async () => {
            expect(refusedOrderOfRound({
              round: 1,
              armNames: armNames(),
            },),).toEqual(['branch-serial', 'branch-overlap', 'branch-serial-again', 'main',],);
            expect(refusedOrderOfRound({
              round: 4,
              armNames: armNames(),
            },),).toEqual(armNames(),);
          },
        }),
      ],
    },),
    describe({
      name: summaryOfAdds.name,
      children: [
        it({
          name: 'count each arm\'s landed adds and their medians, and leave a failed add out of both',
          fn: async () => {
            expect(summaryOfAdds({
              rows: [
                countedRow({
                  arm: 'main',
                  totalMs: 10,
                },),
                countedRow({
                  arm: 'main',
                  totalMs: 30,
                },),
                countedRow({
                  arm: 'main',
                  totalMs: 500,
                  failed: true,
                },),
              ],
              arms: ['main',],
            },),).toEqual({
              main: {
                n: 2,
                failed: 1,
                routes: ['clone',],
                medianTotalMs: 30,
                totals: [10, 30,],
                walls: [10, 30,],
                orders: [[1, 1,], [1, 1,],],
                stepMedians: {
                  'git ls-tree': 2,
                  checkSource: 1,
                },
                workMedians: { fenceSizesBesideTheCheck: 3, },
              },
            },);
          },
        }),
        it({
          name: 'answer no median and no route for an arm whose every add failed',
          fn: async () => {
            expect(summaryOfAdds({
              rows: [countedRow({
                arm: 'main',
                totalMs: 10,
                failed: true,
              },),],
              arms: ['main',],
            },),).toEqual({
              main: {
                n: 0,
                failed: 1,
                routes: [],
                medianTotalMs: null,
                totals: [],
                walls: [],
                orders: [],
                stepMedians: {},
                workMedians: {},
              },
            },);
          },
        }),
      ],
    },),
    describe({
      name: summaryOfRefusedAdds.name,
      children: [
        it({
          name: 'add up the read an add started, from its step and its worker-thread work',
          fn: async () => {
            expect(summaryOfRefusedAdds({
              rows: [
                {
                  ...countedRow({
                    arm: 'branch-overlap',
                    totalMs: 10,
                  },),
                  steps: {
                    'git ls-tree': 2,
                    checkSource: 4,
                  },
                },
                {
                  ...countedRow({
                    arm: 'branch-overlap',
                    totalMs: 20,
                  },),
                  steps: { checkSource: 6, },
                  work: {},
                },
              ],
              arms: ['branch-overlap',],
            },),).toEqual({
              'branch-overlap': {
                n: 2,
                routes: ['clone',],
                totals: [10, 20,],
                medianTotalMs: 20,
                readMs: [5, 0,],
                checkSourceMs: [4, 6,],
              },
            },);
          },
        }),
      ],
    },),
    describe({
      name: rowAsWritten.name,
      children: [
        it({
          name: 'write the route an add did not take as null, which is what the row files hold',
          fn: async () => {
            /**
             The row of a warm-up add, which took no route and belongs to no round.
             */
            const warmUp: Row = {
              layout: 'loose',
              arm: 'main',
              overlap: false,
              tag: 'warm-main',
              counted: false,
              wallMs: 10,
              totalMs: 10,
              exitCode: 0,
              failed: false,
              stderr: '',
              steps: {},
              work: {},
            };
            /**
             The warm-up's row as the file holds it.
             */
            const written = rowAsWritten({ row: warmUp, },);
            expect(written.route,).toBeNull();
            expect(Object.hasOwn(written, 'round',),).toBe(false,);
            expect(Object.hasOwn(written, 'position',),).toBe(false,);
          },
        }),
        it({
          name: 'keep the round and the place of a counted add',
          fn: async () => {
            /**
             The counted row as the file holds it.
             */
            const written = rowAsWritten({ row: countedRow({
              arm: 'main',
              totalMs: 10,
            },), },);
            expect(written.round,).toBe(1,);
            expect(written.position,).toBe(1,);
            expect(written.route,).toBe('clone',);
          },
        }),
      ],
    },),
  ],
},);
