/**
 Tests for the lines an editor fidelity arm writes about how far it went: how many trials it planned of the pool, where
 the spend guard capped it, and how many it completed. The read step takes the number of trials the later arms walk from
 the first arm's log, so the wording is pinned here and not left to the run's own test. Fixtures are cat-themed
 invention only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  cappedLine,
  plannedLine,
  trialsLine,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'editor fidelity progress lines',
  children: [
    describe({
      name: plannedLine.name,
      children: [
        it({
          name: 'SAYS HOW MANY OF THE POOL\'S TRIALS THE ARM PLANNED, with the noun agreeing with the pool',
          fn: async () => {
            expect(plannedLine({ planned: 41, total: 88, },),).toBe('planned: 41 of 88 trials',);
            expect(plannedLine({ planned: 1, total: 1, },),).toBe('planned: 1 of 1 trial',);
          },
        },),
        it({
          name: 'SAYS THE PASSES when the arm walks the pool more than once, counting asks, with the pool\'s size in each',
          fn: async () => {
            expect(plannedLine({ planned: 120, total: 176, passes: 2, },),).toBe(
              'planned: 120 of 176 asks (2 passes over the 88 trials of the pool)',
            );
            expect(plannedLine({ planned: 41, total: 88, passes: 1, },),).toBe('planned: 41 of 88 trials',);
          },
        },),
      ],
    },),

    describe({
      name: cappedLine.name,
      children: [
        it({
          name: 'SAYS THE UNIT AT WHICH THE GUARD CAPPED THE ARM and how many units were completed before it, then the '
            + 'guard\'s own sentence',
          fn: async () => {
            expect(cappedLine({ completed: 41, planned: 88, said: 'the guard said so', },),).toBe(
              'CAPPED at unit 42 of 88: 41 units completed; the guard said so',
            );
            expect(cappedLine({ completed: 1, planned: 3, said: 'x', },),).toBe(
              'CAPPED at unit 2 of 3: 1 unit completed; x',
            );
            expect(cappedLine({ completed: 0, planned: 3, said: 'x', },),).toBe(
              'CAPPED at unit 1 of 3: 0 units completed; x',
            );
          },
        },),
      ],
    },),

    describe({
      name: trialsLine.name,
      children: [
        it({
          name: 'SAYS HOW MANY UNITS WERE COMPLETED of the planned, which is the count the later arms are told to walk, and keeps '
            + 'the number the runbook reads after the TRIALS marker',
          fn: async () => {
            expect(trialsLine({ completed: 41, planned: 88, },),).toBe('TRIALS completed 41 of 88 units',);
            expect(trialsLine({ completed: 88, planned: 88, },),).toBe('TRIALS completed 88 of 88 units',);
            expect(trialsLine({ completed: 1, planned: 1, },),).toBe('TRIALS completed 1 of 1 unit',);
            expect(trialsLine({ completed: 150, planned: 264, },).startsWith('TRIALS completed 150 ',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
