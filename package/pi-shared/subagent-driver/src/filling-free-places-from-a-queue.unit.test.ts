/**
 One pass of filling the free places from a queue: the entries started in the queue's order while places are free, an entry whose own
 start fails set aside at that failure with the next entry started in the same pass, a failure of the machine leaving the entry in
 place and stopping the pass, and the pass's own starts counted against the places so no place is filled twice.

 Every test writes its own queue and runs folder in a throwaway folder.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
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
  BUILD_VOLUME,
  FILLING_STARTS_A_ROUND,
  type FillingSeams,
  type FreeSpace,
  type GoingRun,
  type QueueRecord,
  mayStartNow,
  poolNamed,
  QWEN_PLAN_QWEN4_MAX_PREVIEW,
  readQueueOfRuns,
  NO_SET_ASIDE_RECORD,
  setAsideAnEntryOfAQueue,
  setAsideRecordOf,
  shapeOfAQueueEntry,
  summaryTextOf,
  talliesOf,
  fillFreePlacesFromAQueue,
} from '../dist/final/node/index.mjs';

/**
 A folder a test may write in, with its runs and its queue, removed when the test ends.
 */
type AThrowawayFolder = {
  /**
   The folder's path.
   */
  readonly path: string;

  /**
   The folder the runs live in.
   */
  readonly runs: string;

  /**
   The queue's path.
   */
  readonly queueFile: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A folder a test may write in, with its runs and its queue, removed when the test ends.

 @param entries - what the queue holds

 @returns the folder
 */
async function makeFolder(entries: readonly Record<string, unknown>[],): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-filling-',),);
  /**
   The folder the runs live in.
   */
  const runs = join(path, 'runs',);
  await mkdir(runs, { recursive: true, },);
  /**
   The queue's path.
   */
  const queueFile = join(path, 'queue.json',);
  await writeFile(
    queueFile,
    `[\n${entries.map(function asALine(entry,) {
      return `  ${JSON.stringify(entry,)}`;
    },).join(',\n',)}\n]\n`,
  );
  return {
    path,
    runs,
    queueFile,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 The room the build volume has, which every test here says is ample unless its own subject is the floor.
 */
const AMPLE_ROOM: FreeSpace = {
  volume: BUILD_VOLUME,
  freeBytes: 900 * 1_024 * 1_024 * 1_024,
};

/**
 A fresh run of the queue, named after the word given.

 @param name - the run's name

 @returns the entry

 @example
 ```ts
 const entry = runNamed('b');
 ```
 */
function runNamed(name: string,): QueueRecord {
  return {
    name,
    branch: `feat/${name}`,
    prompt: `/runs/${name}-prompt.md`,
    wall: 28_800,
  };
}

/**
 A review of a range of a change, named after the word given.

 @param name - the review's name

 @returns the entry
 */
function reviewNamed(name: string,): QueueRecord {
  return {
    name,
    reviewOfRange: `main...${name}`,
    commit: name,
    source: 'src',
  };
}

/**
 What one pass answered, and what its seams were asked.

 @param request - the queue and the folder its records go in, the level, and how many entries the pass may start

 @param answerOf - what starting one entry answers, by the entry's name

 @param runs - the runs going when the pass counts them

 @returns the pass's answer and what its seams were asked
 */
async function fillOnce({
  folder,
  level,
  startsAtMost,
  answerOf,
  runs = [],
  freeSpace = AMPLE_ROOM,
}: {
  readonly folder: AThrowawayFolder;
  readonly level: number;
  readonly startsAtMost?: number;
  readonly answerOf: (name: string) => {
    readonly started: boolean;
    readonly kind?: 'the-entry' | 'the-machine';
    readonly reason?: string;
  };
  readonly runs?: readonly GoingRun[];
  readonly freeSpace?: FreeSpace;
}): Promise<{
  readonly outcome: Awaited<ReturnType<typeof fillFreePlacesFromAQueue>>;
  readonly started: readonly string[];
  readonly lines: readonly string[];
  readonly waits: readonly number[];
}> {
  /**
   The entries the pass started, in the order it started them.
   */
  const started: string[] = [];
  /**
   The lines the pass said.
   */
  const lines: string[] = [];
  /**
   How long the pass waited between one start and the next.
   */
  const waits: number[] = [];
  /**
   The seams the pass asks, whose set-aside is the package's own, so the record it leaves is the one a person reads.
   */
  const seams: FillingSeams = {
    runsGoing: function runsGoing() {
      return Promise.resolve(runs,);
    },
    freeSpace: function freeSpaceNow() {
      return freeSpace;
    },
    start: function startOne({ record, },) {
      /**
       What the test says this entry's start answers.
       */
      const answer = answerOf(record.name,);
      if (answer.started) {
        started.push(record.name,);
        return Promise.resolve({
          started: true,
          lines: [`${record.name}: started`],
          batchFile: join(folder.runs, `${record.name}-batch.json`,),
          unit: `launch-${record.name}`,
          outputFile: join(folder.runs, `${record.name}-batch.out`,),
        },);
      }
      return Promise.resolve({
        started: false,
        failure: {
          kind: answer.kind ?? 'the-entry',
          reason: answer.reason ?? 'the entry could not be read',
          output: answer.reason ?? 'the entry could not be read',
        },
      },);
    },
    setAside: function setAside({ record, failure, },) {
      return setAsideAnEntryOfAQueue({
        queueFile: folder.queueFile,
        runsFolder: folder.runs,
        record,
        reason: failure.reason,
        now: Date.now,
      },);
    },
    wait: function wait(milliseconds,) {
      waits.push(milliseconds,);
      return Promise.resolve();
    },
    report: function report(line,) {
      lines.push(line,);
    },
  };
  /**
   What the pass answered.
   */
  const outcome = await fillFreePlacesFromAQueue({
    request: {
      queueFile: folder.queueFile,
      runsFolder: folder.runs,
      level,
      ...(startsAtMost === undefined ? {} : { startsAtMost, }),
    },
    seams,
  },);
  return {
    outcome,
    started,
    lines,
    waits,
  };
}

/**
 The names a queue holds, in its order.

 @param queueFile - the queue's path

 @returns the names
 */
async function namesIn(queueFile: string,): Promise<readonly string[]> {
  return (await readQueueOfRuns({ queueFile, },)).map(function nameOf(entry,) {
    return entry.name;
  },);
}

await describe({
  name: '',
  children: [
    describe({
      name: 'a pass with an entry whose own start fails',
      children: [
        it({
          name: 'sets that entry aside with its reason and the time, and starts the entry behind it in the same pass',
          fn: async () => {
            await using folder = await makeFolder([
              reviewNamed('fails',),
              runNamed('b',),
            ],);
            /**
             What the one pass answered.
             */
            const pass = await fillOnce({
              folder,
              level: 1,
              answerOf: function answerOf(name,) {
                return (name === 'fails')
                  ? {
                    started: false,
                    kind: 'the-entry',
                    reason: 'nothing to review under src in main...fails',
                  }
                  : { started: true, };
              },
            },);
            expect(pass.started,).toEqual(['b',],);
            expect(pass.outcome.setAsideNames,).toEqual(['fails',],);
            expect(pass.outcome.stoppedBecause,).toBeUndefined();
            expect(await namesIn(folder.queueFile,),).toEqual([],);
            // The entry set aside is still there to read and put back, with the reason and the time its start failed at.
            expect(await setAsideRecordOf({ runsFolder: folder.runs, name: 'fails', },),).toMatchObject({
              name: 'fails',
              entry: reviewNamed('fails',),
              reason: 'nothing to review under src in main...fails',
            },);
            /**
             The record the pass left.
             */
            const record = await setAsideRecordOf({ runsFolder: folder.runs, name: 'fails', },) as { readonly setAsideAt: string };
            expect(record.setAsideAt,).toContain('T',);
            expect(pass.lines.join('',),).toContain('set aside fails (the-entry)',);
          },
        },),
        it({
          name: 'sets aside an entry no shape reads, saying which field it names, and starts the entry behind it',
          fn: async () => {
            await using folder = await makeFolder([
              {
                name: 'odd',
                branch: 'feat/odd',
                reviewer: 'someone',
              },
              runNamed('b',),
            ],);
            /**
             What the one pass answered.
             */
            const pass = await fillOnce({
              folder,
              level: 1,
              answerOf: function answerOf(name,) {
                return (name === 'odd')
                  ? {
                    started: false,
                    kind: 'the-entry',
                    reason: 'it names a field a run does not read: reviewer',
                  }
                  : { started: true, };
              },
            },);
            expect(pass.outcome.setAsideNames,).toEqual(['odd',],);
            expect(pass.started,).toEqual(['b',],);
            expect((await setAsideRecordOf({ runsFolder: folder.runs, name: 'odd', },) as { readonly reason: string }).reason,)
              .toContain('reviewer',);
          },
        },),
        it({
          name: 'drops a record whose entry never left the queue, which a kill between the two leaves, and tries the entry again',
          fn: async () => {
            await using folder = await makeFolder([runNamed('a',),],);
            await writeFile(
              join(folder.runs, 'a-set-aside.json',),
              `${JSON.stringify({
                name: 'a',
                entry: runNamed('a',),
                reason: 'a prompt file that is not there',
                setAsideAt: '2026-11-14T09:00:00.000-05:00',
              },)}\n`,
            );
            /**
             What the one pass answered.
             */
            const pass = await fillOnce({
              folder,
              level: 1,
              answerOf: function answerOf() {
                return { started: true, };
              },
            },);
            expect(pass.started,).toEqual(['a',],);
            expect(pass.outcome.setAsideNames,).toEqual([],);
            expect(pass.lines.join('',),).toContain('never left the queue',);
            expect(await setAsideRecordOf({ runsFolder: folder.runs, name: 'a', },),).toBe(NO_SET_ASIDE_RECORD,);
          },
        },),
      ],
    },),
    describe({
      name: 'a pass with a failure of the machine',
      children: [
        it({
          name: 'leaves that entry in the queue, starts nothing behind it, and says why it stopped',
          fn: async () => {
            await using folder = await makeFolder([
              runNamed('a',),
              runNamed('b',),
            ],);
            /**
             What the one pass answered.
             */
            const pass = await fillOnce({
              folder,
              level: 2,
              answerOf: function answerOf(name,) {
                return (name === 'a')
                  ? {
                    started: false,
                    kind: 'the-machine',
                    reason: 'the drive is under its floor',
                  }
                  : { started: true, };
              },
            },);
            expect(pass.started,).toEqual([],);
            expect(pass.outcome.setAsideNames,).toEqual([],);
            expect(pass.outcome.stoppedBecause,).toBe('the machine said no: the drive is under its floor',);
            expect(await namesIn(folder.queueFile,),).toEqual(['a', 'b',],);
            expect(await setAsideRecordOf({ runsFolder: folder.runs, name: 'a', },),).toBe(NO_SET_ASIDE_RECORD,);
            expect(pass.lines.join('',),).toContain('left a in place at position 0 of 2',);
          },
        },),
        it({
          name: 'starts nothing at all while the drive is under the floor a run needs, and keeps every entry',
          fn: async () => {
            await using folder = await makeFolder([runNamed('a',),],);
            /**
             What the one pass answered.
             */
            const pass = await fillOnce({
              folder,
              level: 4,
              answerOf: function answerOf() {
                return { started: true, };
              },
              freeSpace: {
                volume: BUILD_VOLUME,
                freeBytes: 1,
              },
            },);
            expect(pass.started,).toEqual([],);
            expect(pass.outcome.stoppedBecause,).toContain('under the floor',);
            expect(await namesIn(folder.queueFile,),).toEqual(['a',],);
          },
        },),
      ],
    },),
    describe({
      name: 'the places a pass fills',
      children: [
        it({
          name: 'counts the runs going against the level, starting nothing at the level',
          fn: async () => {
            await using folder = await makeFolder([runNamed('a',),],);
            /**
             What the one pass answered at a level two runs already fill.
             */
            const pass = await fillOnce({
              folder,
              level: 2,
              runs: [
                { name: 'x', membership: { holds: 'no-pool', }, },
                { name: 'y', membership: { holds: 'no-pool', }, },
              ],
              answerOf: function answerOf() {
                return { started: true, };
              },
            },);
            expect(pass.outcome.placesFreeAtTheStart,).toBe(0,);
            expect(pass.started,).toEqual([],);
            expect(await namesIn(folder.queueFile,),).toEqual(['a',],);
          },
        },),
        it({
          name: 'counts a run it starts against the places, so one free place fills once',
          fn: async () => {
            await using folder = await makeFolder([
              runNamed('a',),
              runNamed('b',),
            ],);
            /**
             What the one pass answered with one place free.
             */
            const pass = await fillOnce({
              folder,
              level: 2,
              runs: [{ name: 'x', membership: { holds: 'no-pool', }, },],
              answerOf: function answerOf() {
                return { started: true, };
              },
            },);
            expect(pass.outcome.placesFreeAtTheStart,).toBe(1,);
            expect(pass.started,).toEqual(['a',],);
            expect(await namesIn(folder.queueFile,),).toEqual(['b',],);
          },
        },),
        it({
          name: 'starts the entries in the queue order, waiting between one start and the next until it is told to start no more',
          fn: async () => {
            await using folder = await makeFolder([
              runNamed('b',),
              runNamed('a',),
              runNamed('c',),
            ],);
            /**
             What the one pass answered, told to start at most two.
             */
            const pass = await fillOnce({
              folder,
              level: 9,
              startsAtMost: 2,
              answerOf: function answerOf() {
                return { started: true, };
              },
            },);
            expect(pass.started,).toEqual(['b', 'a',],);
            expect(pass.waits,).toHaveLength(1,);
            expect(await namesIn(folder.queueFile,),).toEqual(['c',],);
          },
        },),
        it({
          name: 'starts at most the two entries a round is given when the caller names no number',
          fn: async () => {
            await using folder = await makeFolder([
              runNamed('a',),
              runNamed('b',),
              runNamed('c',),
            ],);
            /**
             What the one pass answered.
             */
            const pass = await fillOnce({
              folder,
              level: 9,
              answerOf: function answerOf() {
                return { started: true, };
              },
            },);
            expect(FILLING_STARTS_A_ROUND,).toBe(2,);
            expect(pass.started,).toHaveLength(2,);
          },
        },),
        it({
          name: 'stops with nothing started when every pool the entries ask for is full, and sets nothing aside',
          fn: async () => {
            await using folder = await makeFolder([reviewNamed('a',),],);
            /**
             How full the reviewing pool is.
             */
            const full = Array.from({ length: QWEN_PLAN_QWEN4_MAX_PREVIEW.limit, }, function asARun(_, index,) {
              return {
                name: `review-going-${String(index,)}`,
                membership: poolNamed({
                  provider: QWEN_PLAN_QWEN4_MAX_PREVIEW.provider,
                  model: QWEN_PLAN_QWEN4_MAX_PREVIEW.model,
                },),
              } satisfies GoingRun;
            },);
            /**
             What the one pass answered.
             */
            const pass = await fillOnce({
              folder,
              level: 9,
              runs: full,
              answerOf: function answerOf() {
                return { started: true, };
              },
            },);
            expect(pass.started,).toEqual([],);
            expect(pass.outcome.setAsideNames,).toEqual([],);
            expect(pass.outcome.stoppedBecause,).toContain('may start now',);
            expect(await namesIn(folder.queueFile,),).toEqual(['a',],);
          },
        },),
      ],
    },),
    describe({
      name: mayStartNow.name,
      children: [
        it({
          name: 'lets an entry start while its pool has room, and refuses it at the cap the reviewing model is held to',
          fn: async () => {
            /**
             The counts with no run going.
             */
            const empty = talliesOf({ runs: [], },);
            expect(mayStartNow({
              record: reviewNamed('a',),
              tallies: empty,
            },),).toBe(true,);
            /**
             The counts with the reviewing pool at its cap.
             */
            const full = talliesOf({
              runs: Array.from({ length: QWEN_PLAN_QWEN4_MAX_PREVIEW.limit, }, function asARun(_, index,) {
                return {
                  name: `r${String(index,)}`,
                  membership: poolNamed({
                    provider: QWEN_PLAN_QWEN4_MAX_PREVIEW.provider,
                    model: QWEN_PLAN_QWEN4_MAX_PREVIEW.model,
                  },),
                } satisfies GoingRun;
              },),
            },);
            expect(summaryTextOf(full,),).toContain(`${QWEN_PLAN_QWEN4_MAX_PREVIEW.name} ${String(QWEN_PLAN_QWEN4_MAX_PREVIEW.limit,)}/`,);
            expect(mayStartNow({
              record: reviewNamed('a',),
              tallies: full,
            },),).toBe(false,);
          },
        },),
        it({
          name: 'lets an entry the filling cannot read start, because it has to be taken out of the way',
          fn: async () => {
            expect(shapeOfAQueueEntry({ record: { name: 'odd', }, },).kind,).toBe('unreadable',);
            expect(mayStartNow({
              record: { name: 'odd', },
              tallies: talliesOf({ runs: [], },),
            },),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
