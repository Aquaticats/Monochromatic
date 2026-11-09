/**
 One look at the Pi runs going now: which are past a limit the finished runs set, which are not, and how long a reported run is
 left alone. Every run folder is one this test writes and throws away, and every unit is one it hands over.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  MILLISECONDS_PER_MINUTE,
  RECHECK_MILLISECONDS,
  type RunningPiUnit,
  lookAtPiRuns,
  standInRunsFolder,
} from '../dist/final/node/index.mjs';


/**
 A call that ended after thirty seconds, which the call limit is computed from.
 */
const SHORT_CALL = 30_000;

/**
 A call that ended after ten minutes, which is then the call limit.
 */
const LONG_CALL = 10 * MILLISECONDS_PER_MINUTE;

/**
 A finished run of an hour.
 */
const AN_HOUR = 60 * MILLISECONDS_PER_MINUTE;

/**
 A finished run of ninety minutes, which is then how long a run may last.
 */
const NINETY_MINUTES = 90 * MILLISECONDS_PER_MINUTE;

/**
 The moment a look in these tests reads the time at.
 */
const NOW = 1_800_000_000_000;

/**
 The finished runs every test here compares against: two of them, so the limits are the longer run and the longer call.
 @returns the runs folder, and how to remove it
 @example
 ```ts
 await using runs = await finishedRuns();
 ```
 */
async function finishedRuns() {
  return await standInRunsFolder({
    runs: [
      {
        name: 'alpha-1',
        summary: {
          endReason: 'idle',
          totalMilliseconds: AN_HOUR,
        },
        feedLines: [
          {
            t: 0,
            kind: 'tool_start',
            tool: 'bash',
          },
          {
            t: SHORT_CALL,
            kind: 'tool_end',
            tool: 'bash',
          },
        ],
      },
      {
        name: 'beta-1',
        summary: {
          endReason: 'idle',
          totalMilliseconds: NINETY_MINUTES,
        },
        feedLines: [
          {
            t: 0,
            kind: 'tool_start',
            tool: 'bash',
          },
          {
            t: LONG_CALL,
            kind: 'tool_end',
            tool: 'bash',
          },
        ],
      },
    ],
  },);
}

/**
 Reads one running unit.

 @param name - the run's name

 @param elapsedMilliseconds - how long it has been up

 @returns the unit a look is handed

 @example
 ```ts
 const unit = unitOf({ name: 'alpha', elapsedMilliseconds: 1_000, },);
 ```
 */
function unitOf({
  name,
  elapsedMilliseconds,
}: {
  readonly name: string;
  readonly elapsedMilliseconds: number;
},): RunningPiUnit {
  return {
    name,
    unitName: `pi-${name}.service`,
    elapsedMilliseconds,
    mainPid: 4_242,
  };
}

await describe({
  name: lookAtPiRuns.name,
  children: [
    it({
      name: 'reports a run past the limit the finished runs set, naming the folder to read',
      fn: async () => {
        await using runs = await finishedRuns();
        /**
         What the look found.
         */
        const look = await lookAtPiRuns({
          runsFolder: runs.path,
          units: [
            unitOf({
              name: 'alpha',
              elapsedMilliseconds: 2 * AN_HOUR,
            },),
          ],
          checked: {},
          now: NOW,
          recheckMilliseconds: RECHECK_MILLISECONDS,
        },);
        expect(look.lines,).toHaveLength(1,);
        expect(look.lines[0],).toContain('OVERDUE pi-alpha.service',);
        expect(look.lines[0],).toContain('90% of 2 finished runs were done within 90 min',);
        expect(look.lines[0],).toContain(runs.entryPathOf('alpha-1',),);
        expect(look.reported['pi-alpha.service'],).toBe(NOW,);
      },
    },),
    it({
      name: 'reports nothing for a run inside the limit',
      fn: async () => {
        await using runs = await finishedRuns();
        /**
         What the look found.
         */
        const look = await lookAtPiRuns({
          runsFolder: runs.path,
          units: [
            unitOf({
              name: 'alpha',
              elapsedMilliseconds: NINETY_MINUTES,
            },),
          ],
          checked: {},
          now: NOW,
          recheckMilliseconds: RECHECK_MILLISECONDS,
        },);
        expect(look.lines,).toEqual([],);
        expect(look.anyRunning,).toBe(true,);
        expect(look.reported,).toEqual({},);
        expect(look.evidence.overdueMilliseconds,).toBe(NINETY_MINUTES,);
      },
    },),
    it({
      name: 'reports a call open past what the finished calls say one takes',
      fn: async () => {
        await using runs = await standInRunsFolder({
          runs: [
            {
              name: 'alpha-1',
              summary: {
                endReason: 'idle',
                totalMilliseconds: AN_HOUR,
              },
              feedLines: [
                {
                  t: 0,
                  kind: 'tool_start',
                  tool: 'bash',
                },
                {
                  t: LONG_CALL,
                  kind: 'tool_end',
                  tool: 'bash',
                },
              ],
            },
            {
              name: 'gamma-1',
              feedLines: [
                {
                  t: 0,
                  kind: 'tool_start',
                  tool: 'bash',
                  args: 'mise run //package/x:test',
                },
              ],
            },
          ],
        },);
        /**
         What the look found.
         */
        const look = await lookAtPiRuns({
          runsFolder: runs.path,
          units: [
            unitOf({
              name: 'gamma',
              elapsedMilliseconds: LONG_CALL + MILLISECONDS_PER_MINUTE,
            },),
          ],
          checked: {},
          now: NOW,
          recheckMilliseconds: RECHECK_MILLISECONDS,
        },);
        expect(look.lines,).toHaveLength(1,);
        expect(look.lines[0],).toContain('STUCK? pi-gamma.service',);
        expect(look.lines[0],).toContain('bash open 11 min',);
        expect(look.lines[0],).toContain('99.9% of 1 finished calls ended within 10 min',);
        expect(look.reported['open-call:pi-gamma.service'],).toBe(NOW,);
      },
    },),
    it({
      name: 'holds a reported run back until its recheck interval passes, and reports it again after it',
      fn: async () => {
        await using runs = await finishedRuns();
        /**
         The unit that is past the limit.
         */
        const unit = unitOf({
          name: 'alpha',
          elapsedMilliseconds: 2 * AN_HOUR,
        },);
        /**
         What the first look found.
         */
        const first = await lookAtPiRuns({
          runsFolder: runs.path,
          units: [unit,],
          checked: {},
          now: NOW,
          recheckMilliseconds: RECHECK_MILLISECONDS,
        },);
        expect(first.lines,).toHaveLength(1,);
        /**
         A look one millisecond before the interval has passed.
         */
        const tooSoon = await lookAtPiRuns({
          runsFolder: runs.path,
          units: [unit,],
          checked: first.reported,
          now: (NOW + RECHECK_MILLISECONDS) - 1,
          recheckMilliseconds: RECHECK_MILLISECONDS,
        },);
        expect(tooSoon.lines,).toEqual([],);
        /**
         A look at the moment the interval has passed.
         */
        const afterTheInterval = await lookAtPiRuns({
          runsFolder: runs.path,
          units: [unit,],
          checked: first.reported,
          now: NOW + RECHECK_MILLISECONDS,
          recheckMilliseconds: RECHECK_MILLISECONDS,
        },);
        expect(afterTheInterval.lines,).toHaveLength(1,);
        expect(afterTheInterval.lines[0],).toContain('OVERDUE pi-alpha.service',);
      },
    },),
    it({
      name: 'reports every run older than a limit the caller set, and leaves the recheck interval as it was',
      fn: async () => {
        await using runs = await finishedRuns();
        /**
         What the look with the caller's own limit found.
         */
        const look = await lookAtPiRuns({
          runsFolder: runs.path,
          units: [
            unitOf({
              name: 'alpha',
              elapsedMilliseconds: 2 * AN_HOUR,
            },),
          ],
          checked: { 'pi-alpha.service': NOW, },
          now: NOW,
          recheckMilliseconds: RECHECK_MILLISECONDS,
          limitMinutes: 5,
        },);
        expect(look.lines,).toHaveLength(1,);
        expect(look.lines[0],).toContain('within 5 min',);
        expect(look.reported,).toEqual({ 'pi-alpha.service': NOW, },);
      },
    },),
    it({
      name: 'reports a run whose folder is not there yet, saying where it looked',
      fn: async () => {
        await using runs = await finishedRuns();
        /**
         What the look found.
         */
        const look = await lookAtPiRuns({
          runsFolder: runs.path,
          units: [
            unitOf({
              name: 'delta',
              elapsedMilliseconds: 2 * AN_HOUR,
            },),
          ],
          checked: {},
          now: NOW,
          recheckMilliseconds: RECHECK_MILLISECONDS,
        },);
        expect(look.lines,).toHaveLength(1,);
        expect(look.lines[0],).toContain('no feed yet',);
        expect(look.lines[0],).toContain(`no run folder named delta-<number> under ${runs.path}`,);
      },
    },),
    it({
      name: 'reports that no run is going and no line, so the wait can end',
      fn: async () => {
        await using runs = await finishedRuns();
        /**
         What the look found.
         */
        const look = await lookAtPiRuns({
          runsFolder: runs.path,
          units: [],
          checked: {},
          now: NOW,
          recheckMilliseconds: RECHECK_MILLISECONDS,
        },);
        expect(look.lines,).toEqual([],);
        expect(look.anyRunning,).toBe(false,);
      },
    },),
  ],
},);
