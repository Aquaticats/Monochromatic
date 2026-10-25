import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  benchBelowFloorStageFinding,
  MIN_STAGE_VOICES,
  reachableQuorum,
  rosterQuorumSize,
  shortBenchStageFinding,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: reachableQuorum.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS THE BENCH QUORUM while the reachable seats can still meet it, so a healthy bench and a bench '
            + 'short of a few seats close on the same count',
          fn: async () => {
            expect(reachableQuorum({ benchSize: 11, unreachable: 0, },),).toEqual({
              needed: 6,
              reachable: 11,
              benchQuorum: 6,
              short: false,
            },);
            expect(reachableQuorum({ benchSize: 11, unreachable: 3, },),).toEqual({
              needed: 6,
              reachable: 8,
              benchQuorum: 6,
              short: false,
            },);
          },
        },),

        it({
          name: 'CLOSES ONE VOICE BELOW EVERY LIVE SEAT where the reachable seats are exactly the bench quorum, so '
            + 'the straggler grace can open on the last of them (owner, 2026-10-22, "Let the grace open"): seven '
            + 'seats with three dark leave four, which close on three',
          fn: async () => {
            expect(reachableQuorum({ benchSize: 7, unreachable: 3, },),).toEqual({
              needed: 3,
              reachable: 4,
              benchQuorum: 4,
              short: true,
            },);
            expect(reachableQuorum({ benchSize: 11, unreachable: 5, },),).toEqual({
              needed: 5,
              reachable: 6,
              benchQuorum: 6,
              short: true,
            },);
          },
        },),

        it({
          name: 'WAITS ON EVERY LIVE SEAT UNDER THE RULE OF 2026-09-09 when the last-seat grace is off, as a reader of '
            + 'a record written before 2026-10-22 asks: seven seats with three dark need all four',
          fn: async () => {
            expect(reachableQuorum({
              benchSize: 7,
              unreachable: 3,
              lastSeatGrace: false,
            },),).toEqual({
              needed: 4,
              reachable: 4,
              benchQuorum: 4,
              short: false,
            },);
            expect(reachableQuorum({
              benchSize: 11,
              unreachable: 7,
              lastSeatGrace: false,
            },),).toEqual(reachableQuorum({
              benchSize: 11,
              unreachable: 7,
            },),);
          },
        },),

        it({
          name: 'KEEPS THE TWO-VOICE FLOOR where the reachable seats are exactly a bench quorum of two, so one judge '
            + 'alone never decides, and LEAVES a seated bench of one or two on its own quorum of one',
          fn: async () => {
            expect(reachableQuorum({ benchSize: 3, unreachable: 1, },),).toEqual({
              needed: 2,
              reachable: 2,
              benchQuorum: 2,
              short: false,
            },);
            expect(reachableQuorum({ benchSize: 4, unreachable: 2, },).needed,).toBe(MIN_STAGE_VOICES,);
            expect(reachableQuorum({ benchSize: 1, unreachable: 0, },),).toEqual({
              needed: 1,
              reachable: 1,
              benchQuorum: 1,
              short: false,
            },);
            expect(reachableQuorum({ benchSize: 2, unreachable: 0, },).needed,).toBe(1,);
          },
        },),

        it({
          name: 'SIZES THE QUORUM ON THE REACHABLE SEATS once they are fewer than the bench quorum, at half of them '
            + 'rounded up and never under the two-voice floor: seven of eleven seats dark leaves four, which close '
            + 'on two',
          fn: async () => {
            expect(reachableQuorum({ benchSize: 11, unreachable: 7, },),).toEqual({
              needed: 2,
              reachable: 4,
              benchQuorum: 6,
              short: true,
            },);
            expect(reachableQuorum({ benchSize: 11, unreachable: 6, },).needed,).toBe(
              rosterQuorumSize({ rosterSize: 5, },),
            );
            expect(reachableQuorum({ benchSize: 6, unreachable: 4, },).needed,).toBe(MIN_STAGE_VOICES,);
          },
        },),

        it({
          name: 'NEVER LETS ONE VOICE, OR NONE, BE A QUORUM: a bench with one reachable seat or none needs the floor '
            + 'it cannot reach, which reads as an outage',
          fn: async () => {
            expect(reachableQuorum({ benchSize: 11, unreachable: 10, },).needed,).toBe(MIN_STAGE_VOICES,);
            expect(reachableQuorum({ benchSize: 3, unreachable: 3, },).needed,).toBe(MIN_STAGE_VOICES,);
            expect(reachableQuorum({ benchSize: 3, unreachable: 3, },).reachable,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: shortBenchStageFinding.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the stage, the reachable seats against the bench and the quorum applied, in the wording a '
            + 'scorecard can count',
          fn: async () => {
            expect(shortBenchStageFinding({
              stage: 'archive-block-review',
              quorum: reachableQuorum({ benchSize: 11, unreachable: 7, },),
              benchSize: 11,
            },),).toBe('stage-short-bench (archive-block-review reachable 4 of 11, quorum 2)',);
          },
        },),
      ],
    },),

    describe({
      name: benchBelowFloorStageFinding.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the stage, the reachable seats against the bench and the floor, in wording apart from the '
            + 'short-bench finding, so a stage that made no call is told from one that closed on a thin bench',
          fn: async () => {
            expect(benchBelowFloorStageFinding({
              stage: 'lane-contest',
              reachable: 1,
              benchSize: 7,
              floor: 2,
            },),).toBe('stage-bench-below-floor (lane-contest reachable 1 of 7, floor 2, no call made)',);
          },
        },),
      ],
    },),
  ],
},);
