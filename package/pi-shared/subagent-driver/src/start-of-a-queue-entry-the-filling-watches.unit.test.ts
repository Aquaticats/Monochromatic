/**
 The record of one start the filling made: the entry out of the queue only while its run exists, and the entry put back or set aside
 with the launcher's own reason otherwise.

 The manager and the runs folder are seams: a test answers `systemctl show` itself and writes the run folders it describes, so no
 unit, worktree or Pi run is started.

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
  WATCHED_START_FILE_SUFFIX,
  NO_SET_ASIDE_RECORD,
  type QueueRecord,
  type RunCommand,
  isAWatchedStartFile,
  putAQueueEntryIn,
  readQueueOfRuns,
  removeWatchedStart,
  runExistsFor,
  setAsideRecordOf,
  settleWatchedStarts,
  watchedStartFileOf,
  watchedStartIn,
  watchedStartsIn,
  WatchedStartError,
  writeWatchedStart,
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
 Writes the answer the batch that started one entry says its start ended with.

 @param folder - the folder the runs live in

 @param name - the entry's own name

 @param answer - what the answer says

 @example
 ```ts
 await writeStartAnswer({ folder, name: 'a', answer: { name: 'a', status: 0, gaveUpWaitingForAPlace: false, line: 'a: status 0' }, },);
 ```
 */
async function writeStartAnswer({
  folder,
  name,
  answer,
}: {
  readonly folder: AThrowawayFolder;
  readonly name: string;
  readonly answer: {
    readonly name: string;
    readonly status: number;
    readonly gaveUpWaitingForAPlace: boolean;
    readonly line: string;
    readonly keptOutputAt?: string;
  };
},): Promise<void> {
  await writeFile(
    join(
      folder.runs,
      `${name}-start-answer.json`,
    ),
    `${JSON.stringify(answer,)}\n`,
  );
}

/**
 A folder a test may write in, with its runs and its queue, removed when the test ends.

 @param entries - what the queue holds

 @returns the folder
 */
async function makeFolder(entries: readonly Record<string, unknown>[],): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-watched-start-',),);
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
    `${JSON.stringify(entries,)}\n`,
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
 The entry a test starts, which is one run a queue of runs may hold.

 @param name - its name

 @returns the entry
 */
function anEntry({ name, }: { readonly name: string },): QueueRecord {
  return {
    name,
    branch: `feat/${name}`,
  };
}

/**
 Writes the record of one start, as the filling writes it before the batch's unit is asked for.

 @param request - the folder, the entry, the unit's state and what the batch printed

 @returns the record's path

 @example
 ```ts
 await writeStartOf({ folder, name: 'a', state: 'inactive', printed: 'ALREADY RUNNING: a\n', });
 ```
 */
async function writeStartOf({
  folder,
  name,
  printed,
}: {
  readonly folder: AThrowawayFolder;
  readonly name: string;
  readonly printed: string;
}): Promise<string> {
  /**
   The batch list beside the runs, which names the unit and its output.
   */
  const batchFile = join(
    folder.runs,
    `batch-2026-11-14T162841-queue-entry-${name}.json`,
  );
  await writeFile(
    batchFile,
    `${JSON.stringify([{
      name,
      branch: `feat/${name}`,
    },],)}\n`,
  );
  await writeFile(
    join(
      folder.runs,
      `batch-2026-11-14T162841-queue-entry-${name}.out`,
    ),
    printed,
  );
  return await writeWatchedStart({
    runsFolder: folder.runs,
    record: anEntry({ name, },),
    start: {
      runName: name,
      batchFile,
      outputFile: join(
        folder.runs,
        `batch-2026-11-14T162841-queue-entry-${name}.out`,
      ),
      unit: `launch-batch-2026-11-14T162841-queue-entry-${name}`,
    },
    now: Date.now,
  },);
}

/**
 Answers `systemctl show` for the units a test names, and a unit it does not name as one that is not there.

 @param states - the state each unit is in

 @returns how one command is run
 */
function unitsInTheStates(states: ReadonlyMap<string, string>,): RunCommand {
  return async function systemctl({ words, },) {
    /**
     The unit the question is about, which follows `show`.
     */
    const unitName = words[2] ?? '';
    return {
      status: 0,
      stdout: `${states.get(unitName,) ?? 'inactive'}\n`,
      stderr: '',
    };
  };
}

/**
 Writes one finished run's folder and summary.

 @param runs - the folder the runs live in

 @param name - the run's name

 @returns what the summary's path is
 */
async function writeFinishedRun({
  runs,
  name,
}: {
  readonly runs: string;
  readonly name: string;
},): Promise<string> {
  /**
   The run's own folder.
   */
  const folder = join(
    runs,
    `${name}-1`,
  );
  await mkdir(folder, { recursive: true, },);
  /**
   The summary's path.
   */
  const summary = join(folder, 'summary.json',);
  await writeFile(
    summary,
    '{"endReason":"idle"}\n',
  );
  return summary;
}

await describe({
  name: '',
  children: [
    describe({
      name: watchedStartIn.name,
      children: [
        it({
          name: 'reads a record this package wrote, and refuses anything else with the file named',
          fn: async () => {
            await using folder = await makeFolder([],);
            await writeStartOf({
              folder,
              name: 'a',
              printed: '',
            },);
            /**
             The records the folder holds.
             */
            const records = await watchedStartsIn({ runsFolder: folder.runs, },);
            expect(records,).toHaveLength(1,);
            expect(records[0]?.name,).toBe('a',);
            expect(records[0]?.runName,).toBe('a',);
            expect(records[0]?.unit,).toBe('launch-batch-2026-11-14T162841-queue-entry-a',);
            /**
             What a file that is not a record is refused with.
             */
            let refused: unknown;
            try {
              watchedStartIn({
                shown: '{"name":"a"}',
                file: '/runs/a-queue-start.json',
              },);
            }
            catch (error) {
              refused = error;
            }
            expect(refused,).toBeInstanceOf(WatchedStartError,);
            expect((refused as Error).message,).toContain('/runs/a-queue-start.json',);
          },
        },),
        it({
          name: 'keeps every record removed when its entry was settled, and lists only the start it still holds',
          fn: async () => {
            await using folder = await makeFolder([],);
            await writeStartOf({
              folder,
              name: 'a',
              printed: '',
            },);
            expect(isAWatchedStartFile({
              file: watchedStartFileOf({
                runsFolder: '/runs',
                name: 'a',
              },),
            },),).toBe(true,);
            await removeWatchedStart({
              runsFolder: folder.runs,
              name: 'a',
            },);
            expect(await watchedStartsIn({ runsFolder: folder.runs, },),).toEqual([],);
            // A record that was already gone is no failure: settling the same start twice must be safe.
            await removeWatchedStart({
              runsFolder: folder.runs,
              name: 'a',
            },);
          },
        },),
      ],
    },),
    describe({
      name: runExistsFor.name,
      children: [
        it({
          name: 'reads a run whose unit is going as there, and a run folder that finished after the start as there too',
          fn: async () => {
            await using folder = await makeFolder([],);
            await writeStartOf({
              folder,
              name: 'a',
              printed: '',
            },);
            /**
             The record the questions are about.
             */
            const [record] = await watchedStartsIn({ runsFolder: folder.runs, },);
            expect(record,).toBeDefined();
            if (record === undefined) {
              return;
            }
            expect(await runExistsFor({
              runsFolder: folder.runs,
              record,
              run: unitsInTheStates(new Map([['pi-a.service', 'active',],],),),
            },),).toEqual({
              there: true,
              because: 'unit pi-a.service is active',
            },);
            /**
             The summary of a run that finished before the start, which is no run of this start.
             */
            const summary = await writeFinishedRun({
              runs: folder.runs,
              name: 'a',
            },);
            /**
             A moment before the start, which leaves that summary out.
             */
            const before = new Date(Date.parse(record.startedAt,) - 60_000,);
            await utimes(
              summary,
              before,
              before,
            );
            expect(await runExistsFor({
              runsFolder: folder.runs,
              record,
              run: unitsInTheStates(new Map(),),
            },),).toEqual({ there: false, },);
            /**
             The same summary written after the start, which is this start's run.
             */
            const after = new Date(Date.parse(record.startedAt,) + 60_000,);
            await utimes(
              summary,
              after,
              after,
            );
            expect(await runExistsFor({
              runsFolder: folder.runs,
              record,
              run: unitsInTheStates(new Map(),),
            },),).toEqual({
              there: true,
              because: 'run folder a-1 holds a summary written after the start',
            },);
          },
        },),
      ],
    },),
    describe({
      name: settleWatchedStarts.name,
      children: [
        it({
          name: 'puts an entry back at the head of the queue when its launcher only gave up waiting for a place, and says so once',
          fn: async () => {
            await using folder = await makeFolder([{
              name: 'b',
              branch: 'feat/b',
            },],);
            await writeStartOf({
              folder,
              name: 'a',
              printed: 'WAITING: a asks for a pool\nno slot freed before the deadline: --name=a\n',
            },);
            await writeStartAnswer({
              folder,
              name: 'a',
              answer: {
                name: 'a',
                status: 3,
                gaveUpWaitingForAPlace: true,
                line: 'a: status 3 no slot freed before the deadline: --name=a',
              },
            },);
            /**
             What settling found.
             */
            const settled = await settleWatchedStarts({
              runsFolder: folder.runs,
              queueFile: folder.queueFile,
              run: unitsInTheStates(new Map(),),
              now: Date.now,
              report: function report(problem,) {
                throw new Error(`nothing was expected to be reported: ${problem}`);
              },
            },);
            expect(settled,).toHaveLength(1,);
            expect(settled[0]?.outcome,).toBe('put-back',);
            expect(settled[0]?.line,).toContain('put a back at the head of the queue',);
            expect(settled[0]?.line,).toContain('no slot freed before the deadline',);
            expect((await readQueueOfRuns({ queueFile: folder.queueFile, },)).map(function nameOf(entry,) {
              return entry.name;
            },),).toEqual(['a', 'b',],);
            // The record is gone, which is what makes the filling say it once.
            expect(await watchedStartsIn({ runsFolder: folder.runs, },),).toEqual([],);
            expect(await settleWatchedStarts({
              runsFolder: folder.runs,
              queueFile: folder.queueFile,
              run: unitsInTheStates(new Map(),),
              now: Date.now,
              report: () => {},
            },),).toEqual([],);
          },
        },),
        it({
          name: 'sets an entry aside with the launcher own reason when its launcher failed, and the record can be read back and put back',
          fn: async () => {
            await using folder = await makeFolder([],);
            await writeStartOf({
              folder,
              name: 'a',
              printed: 'ALREADY RUNNING: a\na: status 4 ALREADY RUNNING: a (all it printed: /runs/a-start-failure.log)\n1 of 1 entries failed: a\n',
            },);
            await writeStartAnswer({
              folder,
              name: 'a',
              answer: {
                name: 'a',
                status: 4,
                gaveUpWaitingForAPlace: false,
                line: 'a: status 4 ALREADY RUNNING: a',
                keptOutputAt: '/runs/a-start-failure.log',
              },
            },);
            /**
             What settling found.
             */
            const settled = await settleWatchedStarts({
              runsFolder: folder.runs,
              queueFile: folder.queueFile,
              run: unitsInTheStates(new Map(),),
              now: Date.now,
              report: () => {},
            },);
            expect(settled[0]?.outcome,).toBe('set-aside',);
            expect(settled[0]?.line,).toContain('set aside a (the-launcher)',);
            expect(settled[0]?.line,).toContain('ALREADY RUNNING: a',);
            /**
             The record the set-aside wrote, which holds the entry whole and the reason it was set aside for.
             */
            const record = await setAsideRecordOf({
              runsFolder: folder.runs,
              name: 'a',
            },);
            expect(typeof record,).toBe('object',);
            if ((typeof record) === 'object') {
              expect(record.reason,).toBe('a: status 4 ALREADY RUNNING: a',);
              expect(record.entry,).toEqual(anEntry({ name: 'a', },),);
            }
            expect(await readQueueOfRuns({ queueFile: folder.queueFile, },),).toEqual([],);
          },
        },),
        it({
          name: 'keeps the entry out of the queue and says so once when its run exists',
          fn: async () => {
            await using folder = await makeFolder([],);
            await writeStartOf({
              folder,
              name: 'a',
              printed: '',
            },);
            await writeFinishedRun({
              runs: folder.runs,
              name: 'a',
            },);
            const settled = await settleWatchedStarts({
              runsFolder: folder.runs,
              queueFile: folder.queueFile,
              run: unitsInTheStates(new Map(),),
              now: Date.now,
              report: () => {},
            },);
            expect(settled,).toHaveLength(1,);
            expect(settled[0]?.outcome,).toBe('the-run-exists',);
            expect(await watchedStartsIn({ runsFolder: folder.runs, },),).toEqual([],);
            expect(await readQueueOfRuns({ queueFile: folder.queueFile, },),).toEqual([],);
          },
        },),
        it({
          name: 'leaves a start alone while the batch that was asked for it is still going, and settles it in the round after',
          fn: async () => {
            await using folder = await makeFolder([],);
            await writeStartOf({
              folder,
              name: 'a',
              printed: '',
            },);
            expect(await settleWatchedStarts({
              runsFolder: folder.runs,
              queueFile: folder.queueFile,
              run: unitsInTheStates(new Map([[
                'launch-batch-2026-11-14T162841-queue-entry-a.service',
                'active',
              ],],),),
              now: Date.now,
              report: () => {},
            },),).toEqual([{
              name: 'a',
              outcome: 'still-going',
            },],);
            expect(await watchedStartsIn({ runsFolder: folder.runs, },),).toHaveLength(1,);
          },
        },),
        it({
          name: 'drops the record of a start whose entry never left the queue, which a kill between the two leaves, and tries the entry again',
          fn: async () => {
            await using folder = await makeFolder([],);
            await writeStartOf({
              folder,
              name: 'a',
              printed: '',
            },);
            await putAQueueEntryIn({
              queueFile: folder.queueFile,
              entry: anEntry({ name: 'a', },),
              position: 0,
            },);
            const dropped = await settleWatchedStarts({
              runsFolder: folder.runs,
              queueFile: folder.queueFile,
              run: unitsInTheStates(new Map(),),
              now: Date.now,
              report: () => {},
            },);
            expect(dropped[0]?.outcome,).toBe('never-left-the-queue',);
            expect((await readQueueOfRuns({ queueFile: folder.queueFile, },)).map(function nameOf(entry,) {
              return entry.name;
            },),).toEqual(['a',],);
            expect(await watchedStartsIn({ runsFolder: folder.runs, },),).toEqual([],);
          },
        },),
        it({
          name: 'keeps the record of a start whose batch is going and takes its entry out of the queue, so one batch is started for it',
          fn: async () => {
            await using folder = await makeFolder([],);
            await writeStartOf({
              folder,
              name: 'a',
              printed: '',
            },);
            // The state a kill leaves: the record, the entry and the batch's unit all remain.
            await putAQueueEntryIn({
              queueFile: folder.queueFile,
              entry: anEntry({ name: 'a', },),
              position: 0,
            },);
            /**
             What settling found.
             */
            const settled = await settleWatchedStarts({
              runsFolder: folder.runs,
              queueFile: folder.queueFile,
              run: unitsInTheStates(new Map([[
                'launch-batch-2026-11-14T162841-queue-entry-a.service',
                'active',
              ],],),),
              now: Date.now,
              report: function report(problem,) {
                throw new Error(`nothing was expected to be reported: ${problem}`);
              },
            },);
            expect(settled[0]?.outcome,).toBe('still-going',);
            // The record stays, because the start it watches has not settled.
            expect(await watchedStartsIn({ runsFolder: folder.runs, },),).toHaveLength(1,);
            // The entry leaves the queue, which is what the filling that was killed meant to do.
            expect(await readQueueOfRuns({ queueFile: folder.queueFile, },),).toEqual([],);
          },
        },),
        it({
          name: 'puts an entry back at the head when the launcher answer says it gave up waiting for a place, whatever the batch output last said',
          fn: async () => {
            await using folder = await makeFolder([],);
            await writeStartOf({
              folder,
              name: 'a',
              printed: 'a: status 3 no slot freed before the deadline: --name=a\n1 of 1 entries failed: a\n',
            },);
            // Why the start failed is a field the launcher writes, and the batch's last line decides nothing.
            await writeFile(
              join(
                folder.runs,
                'a-start-answer.json',
              ),
              `${JSON.stringify({
                name: 'a',
                status: 3,
                gaveUpWaitingForAPlace: true,
                line: 'a: status 3 no slot freed before the deadline: --name=a',
              },)}\n`,
            );
            /**
             What settling found.
             */
            const settled = await settleWatchedStarts({
              runsFolder: folder.runs,
              queueFile: folder.queueFile,
              run: unitsInTheStates(new Map(),),
              now: Date.now,
              report: function report(problem,) {
                throw new Error(`nothing was expected to be reported: ${problem}`);
              },
            },);
            expect(settled[0]?.outcome,).toBe('put-back',);
            expect((await readQueueOfRuns({ queueFile: folder.queueFile, },)).map(function nameOf(entry,) {
              return entry.name;
            },),).toEqual(['a',],);
            expect(await setAsideRecordOf({
              runsFolder: folder.runs,
              name: 'a',
            },),).toBe(NO_SET_ASIDE_RECORD,);
          },
        },),
        it({
          name: 'leaves the record of a start it cannot tell apart, whose entry is still queued and whose batch is not going yet',
          fn: async () => {
            await using folder = await makeFolder([],);
            await writeStartOf({
              folder,
              name: 'a',
              printed: '',
            },);
            // The state a concurrent settler meets while the filling that wrote the record is asking for its unit: the entry has
            // not been taken out yet, and the batch is not visible to the manager.
            await putAQueueEntryIn({
              queueFile: folder.queueFile,
              entry: anEntry({ name: 'a', },),
              position: 0,
            },);
            /**
             What the second settlement said.
             */
            const said: string[] = [];
            /**
             What settling found, which is that it cannot tell whether the start is in flight.
             */
            const settled = await settleWatchedStarts({
              runsFolder: folder.runs,
              queueFile: folder.queueFile,
              run: unitsInTheStates(new Map(),),
              now: Date.now,
              report: function report(problem,) {
                said.push(problem,);
              },
            },);
            expect(settled[0]?.outcome,).toBe('cannot-tell',);
            expect(settled[0]?.line,).toContain('could not be told apart',);
            // The record is the only way back to that start, so a settlement that cannot prove the start ended leaves it, and leaves
            // the entry where it is.
            expect(await watchedStartsIn({ runsFolder: folder.runs, },),).toHaveLength(1,);
            expect((await readQueueOfRuns({ queueFile: folder.queueFile, },)).map(function nameOf(entry,) {
              return entry.name;
            },),).toEqual(['a',],);
            expect(await setAsideRecordOf({
              runsFolder: folder.runs,
              name: 'a',
            },),).toBe(NO_SET_ASIDE_RECORD,);
            expect(said,).toEqual([],);
          },
        },),
        it({
          name: 'reports a start it cannot settle and goes on to the next one',
          fn: async () => {
            await using folder = await makeFolder([],);
            await writeStartOf({
              folder,
              name: 'a',
              printed: '',
            },);
            await writeStartOf({
              folder,
              name: 'b',
              printed: 'ALREADY RUNNING: b\n',
            },);
            /**
             What the settling reported.
             */
            const reported: string[] = [];
            /**
             What settling found.
             */
            const settled = await settleWatchedStarts({
              runsFolder: folder.runs,
              // A queue that cannot be read is one start that cannot be settled, and no reason to leave the other one.
              queueFile: join(folder.path, 'no-folder', 'queue.json',),
              run: unitsInTheStates(new Map(),),
              now: Date.now,
              report: function report(problem,) {
                reported.push(problem,);
              },
            },);
            expect(settled,).toEqual([],);
            expect(reported,).toHaveLength(2,);
            expect(reported[0],).toContain('could not be settled');
            // Both records are still there, so the next round tries them again rather than losing the entries.
            expect(await watchedStartsIn({ runsFolder: folder.runs, },),).toHaveLength(2,);
          },
        },),
      ],
    },),
    describe({
      name: WATCHED_START_FILE_SUFFIX,
      children: [
        it({
          name: 'ends every record of a start, and no other file beside the runs is one',
          fn: async () => {
            expect('-queue-start.json'.endsWith(WATCHED_START_FILE_SUFFIX,),).toBe(true,);
            expect(isAWatchedStartFile({ file: 'a-set-aside.json', },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
