/**
 Reading the runs of a machine from its folders: the turns a running run's feed records, the runs that ended since a time, and
 waiting, bounded, for a unit to come up. Every fixture here is a folder or a state a test hands over, so no unit is started and
 no real runs folder is read.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  utimes,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  type EndedRun,
  type RunningPiUnit,
  ageTextOf,
  endedRunIn,
  endedRunLineOf,
  endedRunsSince,
  feedOfARunningRun,
  runningRunLineOf,
  runningRunLinesOf,
  timeIn,
  turnsInFeed,
  waitedLineOf,
  waitForUnitToBecomeActive,
} from '../dist/final/node/index.mjs';

/**
 Milliseconds in a second, so the ages this file states are derived rather than guessed.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 Milliseconds in a minute.
 */
const MILLISECONDS_PER_MINUTE = 60 * MILLISECONDS_PER_SECOND;

/**
 A folder a test throws away.

 @returns the folder, and how to remove it

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-reading-runs-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 One Pi run going now, as systemd lists it.

 @param name - the run's name

 @param elapsedMilliseconds - how long its unit has been up

 @returns the unit

 @example
 ```ts
 const unit = aUnit({ name: 'a', elapsedMilliseconds: 0 });
 ```
 */
function aUnit({
  name,
  elapsedMilliseconds,
}: {
  readonly name: string;
  readonly elapsedMilliseconds: number;
}): RunningPiUnit {
  return {
    name,
    unitName: `pi-${name}.service`,
    elapsedMilliseconds,
    mainPid: 4_242,
  };
}

/**
 Writes one run's feed.

 @param runFolder - the run's folder

 @param turns - how many turns the feed records

 @example
 ```ts
 await writeAFeed({ runFolder, turns: 3 });
 ```
 */
async function writeAFeed({
  runFolder,
  turns,
}: {
  readonly runFolder: string;
  readonly turns: number;
}): Promise<void> {
  await mkdir(runFolder, { recursive: true, },);
  await writeFile(
    join(runFolder, 'feed.jsonl',),
    Array.from(
      { length: turns, },
      function record(_, at,) {
        return `${JSON.stringify({
          kind: 'turn_start',
          n: at + 1,
        })}\n`;
      },
    ).join('',),
  );
}

await describe({
  name: '',
  children: [
    describe({
      name: turnsInFeed.name,
      children: [
        it({
          name: 'counts the turn starts a feed records, and nothing else it holds',
          fn: async () => {
            expect(turnsInFeed({ feedShown: '', },),).toBe(0,);
            // The feed's own lines, each ended by its line feed, as the driver writes them.
            expect(turnsInFeed({
              feedShown: `${[
                JSON.stringify({ kind: 'state', },),
                JSON.stringify({ kind: 'turn_start', n: 1, },),
                JSON.stringify({ kind: 'assistant', text: 'hi', },),
                JSON.stringify({ kind: 'turn_start', n: 2, },),
              ].join('\n',)}\n`,
            },),).toBe(2,);
          },
        },),
        it({
          name: 'counts the turns of a feed its driver is still writing, where the last line is not a record yet',
          fn: async () => {
            // The driver has written two whole lines and part of a third, which is what a reader of a running run's feed sees:
            // parsing that fragment throws `Unterminated string in JSON`, and the two turns it counts are not in doubt.
            expect(turnsInFeed({
              feedShown: `${JSON.stringify({ kind: 'turn_start', n: 1, },)}\n${
                JSON.stringify({ kind: 'turn_start', n: 2, },)
              }\n{"kind":"assistant","text":"a reply the driver is still wri`,
            },),).toBe(2,);
          },
        },),
      ],
    },),
    describe({
      name: ageTextOf.name,
      children: [
        it({
          name: 'says an age in minutes and seconds, which is how long a coordinator has been waiting',
          fn: async () => {
            expect(ageTextOf(0,),).toBe('0m 0s',);
            expect(ageTextOf((2 * MILLISECONDS_PER_MINUTE) + (5 * MILLISECONDS_PER_SECOND),),).toBe('2m 5s',);
          },
        },),
      ],
    },),
    describe({
      name: runningRunLinesOf.name,
      children: [
        it({
          name: 'says the turns and the quiet of the run going now, names the run with no folder yet, and never a run that is'
            + ' not there',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The runs folder, which holds one run's feed.
             */
            const runsFolder = join(folder.path, 'runs',);
            await writeAFeed({
              runFolder: join(runsFolder, 'with-a-feed-1',),
              turns: 3,
            },);
            /**
             When this test reads the ages, which is one minute after both feeds were last written.
             */
            const now = Date.now() + (MILLISECONDS_PER_MINUTE);
            const lines = await runningRunLinesOf({
              units: [
                aUnit({
                  name: 'with-a-feed',
                  elapsedMilliseconds: 5 * MILLISECONDS_PER_MINUTE,
                },),
                aUnit({
                  name: 'without-a-folder',
                  elapsedMilliseconds: MILLISECONDS_PER_MINUTE,
                },),
              ],
              runsFolder,
              now: function clock() {
                return now;
              },
            },);
            expect(lines.length,).toBe(2,);
            expect(lines[0],).toContain('with-a-feed: turns 3',);
            expect(lines[0],).toContain('up 5m 0s',);
            expect(lines[0],).toContain('pi-with-a-feed.service',);
            expect(lines[1],).toContain('without-a-folder: up 1m 0s, no run folder yet',);
          },
        },),
        it({
          name: 'says a run whose folder holds no feed yet has written none',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The runs folder, which holds one run's empty folder.
             */
            const runsFolder = join(folder.path, 'runs',);
            await mkdir(join(runsFolder, 'a-1',), { recursive: true, },);
            const lines = await runningRunLinesOf({
              units: [aUnit({ name: 'a', elapsedMilliseconds: 0, },),],
              runsFolder,
              now: Date.now,
            },);
            expect(lines[0],).toContain('a: up 0m 0s, no feed written yet',);
          },
        },),
      ],
    },),
    describe({
      name: runningRunLineOf.name,
      children: [
        it({
          name: 'prints one line, whatever the feed says, naming the unit systemd holds',
          fn: async () => {
            expect(runningRunLineOf({
              name: 'a',
              unit: aUnit({ name: 'a', elapsedMilliseconds: 0, },),
              said: {
                turns: 4,
                quietMilliseconds: 3 * MILLISECONDS_PER_MINUTE,
              },
            },),).toBe('a: turns 4, feed written 3m 0s ago, up 0m 0s; pi-a.service',);
          },
        },),
      ],
    },),
    describe({
      name: endedRunsSince.name,
      children: [
        it({
          name: 'answers with the runs whose summaries were written since the time, oldest first, and never one that ended before',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The runs folder, which holds one run that ended before the time and one that ended after it.
             */
            const runsFolder = join(folder.path, 'runs',);
            /**
             The time the caller asks from.
             */
            const since = Date.now();
            for (const [name, when,] of [
              ['before-1', since - MILLISECONDS_PER_MINUTE,],
              ['after-1', since + MILLISECONDS_PER_SECOND,],
            ] as const) {
              // oxlint-disable-next-line eslint/no-await-in-loop -- the two folders of this fixture, each made before the next
              await mkdir(join(runsFolder, name,), { recursive: true, },);
              // oxlint-disable-next-line eslint/no-await-in-loop -- the two folders of this fixture, each written before the next
              await writeFile(
                join(runsFolder, name, 'summary.json',),
                `${JSON.stringify({
                  endReason: 'idle',
                  lastAssistantText: `${name} is done`,
                  piExit: { code: 0, },
                },)}\n`,
              );
              // oxlint-disable-next-line eslint/no-await-in-loop -- the two folders of this fixture, each made before the next
              await utimes(
                join(runsFolder, name, 'summary.json',),
                when / MILLISECONDS_PER_SECOND,
                when / MILLISECONDS_PER_SECOND,
              );
            }
            const ended = await endedRunsSince({
              runsFolder,
              since,
            },);
            expect(ended.map((run,) => run.folder,),).toEqual(['after-1',],);
            expect(ended[0]?.endReason,).toBe('idle',);
            expect(ended[0]?.piStatus,).toBe(0,);
            expect(ended[0]?.lastMessage,).toBe('after-1 is done',);
          },
        },),
        it({
          name: 'leaves out a run folder whose summary is missing or names no end',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The runs folder, which holds a run still going, a batch list beside the runs, and a file that is no summary.
             */
            const runsFolder = join(folder.path, 'runs',);
            await mkdir(join(runsFolder, 'going-1',), { recursive: true, },);
            await writeFile(join(runsFolder, 'a-batch.json',), '[]\n',);
            await mkdir(join(runsFolder, 'odd-1',), { recursive: true, },);
            await writeFile(join(runsFolder, 'odd-1', 'summary.json',), '{"something":"else"}\n',);
            const ended = await endedRunsSince({
              runsFolder,
              since: 0,
            },);
            expect(ended,).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: endedRunIn.name,
      children: [
        it({
          name: 'reads the reason, the status and the head of the last message, with the whole message cut to one line',
          fn: async () => {
            /**
             A message longer than one line keeps, with a line ending in it.
             */
            const text = `${'a'.repeat(400,)}\nand the rest`;
            const read = endedRunIn({
              summaryShown: JSON.stringify({
                endReason: 'wall-clock',
                lastAssistantText: text,
                piExit: { code: 1, },
              },),
            },);
            expect(read.read?.endReason,).toBe('wall-clock',);
            expect(read.read?.piStatus,).toBe(1,);
            expect(read.read?.lastMessage.endsWith('...',),).toBe(true,);
            expect(read.read?.lastMessage,).not.toContain('\n',);
          },
        },),
      ],
    },),
    describe({
      name: endedRunLineOf.name,
      children: [
        it({
          name: 'prints when the run ended, its folder, its reason, Pi\'s status and the head of its message',
          fn: async () => {
            /**
             The run this test prints.
             */
            const run: EndedRun = {
              folder: 'a-1',
              endedAtMilliseconds: 0,
              endReason: 'idle',
              piStatus: 0,
              lastMessage: 'done',
            };
            expect(endedRunLineOf(run,),).toBe(`${new Date(0,)
              .toISOString()}\ta-1\tidle\tpi 0\tdone`,);
          },
        },),
      ],
    },),
    describe({
      name: timeIn.name,
      children: [
        it({
          name: 'reads a time a person wrote, and answers nothing for text that names no time',
          fn: async () => {
            expect(timeIn({ written: '2026-11-12T10:00:00-05:00', },).since,).toBe(Date.parse('2026-11-12T10:00:00-05:00',),);
            expect(timeIn({ written: 'yesterday-ish', },).since,).toBeUndefined();
          },
        },),
      ],
    },),
    describe({
      name: waitForUnitToBecomeActive.name,
      children: [
        it({
          name: 'ends as soon as the unit is active, saying how long it took and how many looks it took',
          fn: async () => {
            /**
             The states the manager answers, in order.
             */
            const states = ['activating', 'active',];
            /**
             How many times the manager has been asked.
             */
            const asked = { looks: 0, };
            /**
             When the wait is asked the time, which the wait itself moves on by what it waits for.
             */
            let now = 0;
            const waited = await waitForUnitToBecomeActive({
              stateOf: async function stateOf() {
                const state = states[asked.looks] ?? 'failed';
                asked.looks += 1;
                return state;
              },
              deadline: 60 * MILLISECONDS_PER_SECOND,
              periodMilliseconds: MILLISECONDS_PER_SECOND,
              wait: async function waitAMoment(milliseconds,) {
                now += milliseconds;
              },
              now: function clock() {
                return now;
              },
            },);
            expect(waited.becameActive,).toBe(true,);
            expect(waited.state,).toBe('active',);
            expect(waited.looks,).toBe(2,);
            expect(waited.waitedMilliseconds,).toBe(MILLISECONDS_PER_SECOND,);
          },
        },),
        it({
          name: 'ends at its deadline saying what the unit was doing, rather than waiting for a unit that never comes up',
          fn: async () => {
            /**
             When the wait is asked the time, which the wait itself moves a minute on.
             */
            let now = 0;
            const waited = await waitForUnitToBecomeActive({
              stateOf: async function stateOf() {
                return 'failed';
              },
              deadline: 3 * MILLISECONDS_PER_MINUTE,
              periodMilliseconds: MILLISECONDS_PER_SECOND,
              wait: async function waitAMinute() {
                now += MILLISECONDS_PER_MINUTE;
              },
              now: function clock() {
                return now;
              },
            },);
            expect(waited.becameActive,).toBe(false,);
            expect(waited.state,).toBe('failed',);
            expect(waitedLineOf({
              unitName: 'launch-a.service',
              waited,
            },),).toContain('not active after 180s',);
            expect(waitedLineOf({
              unitName: 'launch-a.service',
              waited,
            },),).toContain('its state failed',);
          },
        },),
      ],
    },),
    describe({
      name: waitedLineOf.name,
      children: [
        it({
          name: 'says a unit came up, with how long it took, which is what a coordinator reads before watching the batch',
          fn: async () => {
            expect(waitedLineOf({
              unitName: 'launch-a.service',
              waited: {
                state: 'active',
                becameActive: true,
                waitedMilliseconds: 2 * MILLISECONDS_PER_SECOND,
                looks: 3,
              },
            },),).toBe('launch-a.service: active after 2s, 3 looks',);
          },
        },),
      ],
    },),
    describe({
      name: feedOfARunningRun.name,
      children: [
        it({
          name: 'answers nothing for a run that has written no feed yet',
          fn: async () => {
            await using folder = await makeFolder();
            expect(await feedOfARunningRun({
              feedFile: join(folder.path, 'feed.jsonl',),
              now: Date.now,
            },),).toEqual({},);
          },
        },),
      ],
    },),
  ],
},);
