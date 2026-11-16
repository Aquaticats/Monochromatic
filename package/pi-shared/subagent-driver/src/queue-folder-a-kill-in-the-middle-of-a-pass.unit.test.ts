/**
 The two kills the earlier round could not make: a filling pass killed between its claim, its start and its completion, and a settlement killed at
 each of its own steps with a dead claim whose unit was made.

 After each kill two further passes run, and the entries the queue, the records and the starts hold are counted: an entry is started at most
 once and is in at most one place. Each control was shown to fail with its behaviour taken out; the failing outputs are in the run's report.

 The commands run as real processes, and the step counter is installed in the child before the command's own modules are imported, so a kill
 lands between two file-system steps of the command itself.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  mkdtemp,
  readdir,
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
  type FillingSeams,
  type QueueClaim,
  type QueueRecord,
  aClaimOf,
  claimFileOf,
  fillFreePlacesFromAQueue,
  namesInAFolderThatMayNotBeThere,
  namesOfAnAttemptIn,
  putAQueueEntryIn,
  readQueueOfRuns,
  setAsideRecordOf,
  NO_SET_ASIDE_RECORD,
  writeTheQueueOfRuns,
} from '../dist/final/node/index.mjs';

/**
 The stand-in one queue command runs as, which can be killed at an instruction.
 */
const A_QUEUE_COMMAND = new URL('stand-in-a-queue-command-killed-at-an-instruction-for-tests.ts', import.meta.url,).pathname;

/**
 A folder a test may write in, with a queue folder and a runs folder, removed when the test ends.
 */
type AQueueInAThrowawayFolder = {
  /**
   The folder everything lives in.
   */
  readonly path: string;

  /**
   The queue's folder.
   */
  readonly queueFolder: string;

  /**
   The runs folder.
   */
  readonly runsFolder: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a queue of one entry named `a` in a throwaway folder.

 @returns the folders

 @example
 ```ts
 await using queue = await aQueueOfOne();
 ```
 */
async function aQueueOfOne(): Promise<AQueueInAThrowawayFolder> {
  /**
   The folder everything lives in.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-queue-kill-walk-',),);
  /**
   The queue's folder.
   */
  const queueFolder = join(path, 'queue',);
  /**
   The runs folder.
   */
  const runsFolder = join(path, 'runs',);
  await mkdir(queueFolder, { recursive: true, },);
  await mkdir(runsFolder, { recursive: true, },);
  await writeTheQueueOfRuns({
    queueFolder,
    entries: [{
      name: 'a',
      branch: 'feat/a',
      prompt: '/runs/a-prompt.md',
      wall: 28_800,
    },],
  },);
  return {
    path,
    queueFolder,
    runsFolder,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Runs the stand-in command as its own process, killed before the instruction named.

 @param request - the command, the queue and the runs folder, and the instruction to stop before

 @returns the signal that ended it, or that it finished, and how many instructions it took

 @example
 ```ts
 const run = await aCommandKilledAt({ command: 'settle', queueFolder: '/runs/q', runsFolder: '/runs', killAt: 2 });
 ```
 */
async function aCommandKilledAt({
  command,
  queueFolder,
  runsFolder,
  killAt,
}: {
  readonly command: string;
  readonly queueFolder: string;
  readonly runsFolder: string;
  readonly killAt: number;
},): Promise<{
  readonly signal: string;
  readonly steps: number;
}> {
  /**
   The child.
   */
  const child = spawn(
    process.execPath,
    [A_QUEUE_COMMAND, command, String(killAt,), queueFolder, runsFolder,],
    { stdio: ['ignore', 'pipe', 'pipe',], },
  );
  /**
   What the child printed.
   */
  const printed: string[] = [];
  child.stdout.on('data', (chunk: Buffer,) => {
    printed.push(chunk.toString(),);
  },);
  child.stderr.on('data', (chunk: Buffer,) => {
    printed.push(chunk.toString(),);
  },);
  const closed: readonly unknown[] = await once(
    child,
    'close',
  );
  /**
   What the child printed as an object, when it printed one.
   */
  const last = printed.join('',)
    .trim()
    .split('\n',)
    .at(-1,);
  /**
   How many instructions the child counted, when it printed one.
   */
  const steps = ((last !== undefined) && last.startsWith('{')) ? ((JSON.parse(last,) as { readonly steps: number }).steps) : 0;
  return {
    signal: ((typeof closed[1]) === 'string') ? closed[1] : '',
    steps,
  };
}

/**
 Runs one filling pass and answers the names it started, counting a start the seam is asked for once.

 @param request - the queue and the runs folder, and the names already started

 @returns the names this pass started

 @example
 ```ts
 const started = await aPassOver({ queueFolder: '/runs/q', runsFolder: '/runs', started: [] });
 ```
 */
async function aPassOver({
  queueFolder,
  runsFolder,
  started,
}: {
  readonly queueFolder: string;
  readonly runsFolder: string;
  readonly started: string[];
},): Promise<readonly string[]> {
  /**
   The names this pass started.
   */
  const mine: string[] = [];
  /**
   The seams this pass asks, whose start answers as a unit that was made.
   */
  const seams: FillingSeams = {
    runsGoing: function runsGoing() {
      return Promise.resolve([],);
    },
    freeSpace: function freeSpaceNow() {
      return {
        volume: BUILD_VOLUME,
        freeBytes: 900 * 1_024 * 1_024 * 1_024,
      };
    },
    async start({ record, claim, },) {
      mine.push(record.name,);
      started.push(record.name,);
      /**
       The names this attempt's unit would write.
       */
      const names = namesOfAnAttemptIn({
        runsFolder,
        name: record.name,
        claim,
      },);
      await writeFile(names.outputFile, 'the unit started\n',);
      return {
        started: true,
        lines: [],
        batchFile: names.batchFile,
        unit: `launch-${record.name}`,
        outputFile: names.outputFile,
      };
    },
    async setAside({ record, failure, queueId, },) {
      const { setAsideAnEntryOfAQueue, } = await import('../dist/final/node/index.mjs',);
      return await setAsideAnEntryOfAQueue({
        queueFolder,
        runsFolder,
        record,
        queueId,
        reason: failure.reason,
        now: Date.now,
      },);
    },
    wait: function wait() {
      return Promise.resolve();
    },
    report: function report() {
      return undefined;
    },
  };
  await fillFreePlacesFromAQueue({
    request: {
      queueFolder,
      runsFolder,
      level: 2,
      startsAtMost: 2,
      staggerMilliseconds: 1,
    },
    seams,
  },);
  return mine;
}

/**
 Reads how many homes one entry has: the queue, a record beside the runs, and nothing else.

 @param request - the queue and the runs folder

 @returns how many places the entry named `a` stands in, and whether a claim names it

 @example
 ```ts
 const homes = await homesOfA({ queueFolder: '/runs/q', runsFolder: '/runs' });
 ```
 */
async function homesOfA({
  queueFolder,
  runsFolder,
}: {
  readonly queueFolder: string;
  readonly runsFolder: string;
},): Promise<{ readonly homes: number; readonly claimed: boolean }> {
  /**
   The entries the queue holds.
   */
  const entries = await readQueueOfRuns({ queueFolder, },);
  /**
   The record beside the runs, when there is one.
   */
  const record = await setAsideRecordOf({
    runsFolder,
    name: 'a',
  },);
  /**
   The claims the queue holds.
   */
  /**
   The names in the claims folder, or none when a queue nothing has claimed in holds no such folder.
   */
  const inTheClaimsFolder = await namesInAFolderThatMayNotBeThere({ folder: join(queueFolder, 'claimed',), },);
  return {
    homes: (entries.some(function isA(entry,) {
      return entry.name === 'a';
    },) ? 1 : 0) + ((record === NO_SET_ASIDE_RECORD) ? 0 : 1),
    claimed: inTheClaimsFolder.some(function isTheClaims(claim,) {
      return claim.isFile() && claim.name
        .endsWith('.claim.json',);
    },),
  };
}

await describe({
  name: '',
  children: [
    it({
      name: 'starts one entry at most once when the settlement is killed at each of its own steps, and two further passes recover',
      fn: async () => {
        // The state a kill leaves: a filling made its unit and died before it removed the entry, so the entry is queued, a dead claim names the
        // attempt, and the attempt's own unit wrote its output file.
        /**
         The rows of the walk: what the command was, the instruction it was killed at, and what the passes did.
         */
        const rows: { readonly killAt: number; readonly signal: string; readonly started: readonly string[]; readonly homes: number }[] = [];
        /* oxlint-disable eslint/no-await-in-loop -- one kill at a time, and each round is a fixture of its own */
        for (let killAt = 1; killAt <= 8; killAt += 1) {
          await using queue = await aQueueOfOne();
          /**
           The entry as the queue holds it.
           */
          const held = (await readQueueOfRuns({ queueFolder: queue.queueFolder, },))[0] as QueueRecord;
          /**
           The claim the dead filling left, whose attempt's unit wrote its output file.
           */
          const claim: QueueClaim = {
            token: 'the-attempt-of-a-filling-that-died',
            pid: 2_147_483_646,
            at: '2026-11-16T00:00:00.000Z',
            startedAt: '1',
          };
          await mkdir(join(queue.queueFolder, 'claimed',), { recursive: true, },);
          await writeFile(
            claimFileOf({ queueFolder: queue.queueFolder, name: 'a', },),
            `${JSON.stringify(aClaimOf({
              name: 'a',
              queueId: String(held.queueId,),
              claim,
            },),)}\n`,
          );
          await writeFile(
            namesOfAnAttemptIn({
              runsFolder: queue.runsFolder,
              name: 'a',
              claim,
            },).outputFile,
            'the unit started\n',
          );
          // The settlement is killed before instruction `killAt`, and then two filling passes recover it.
          /**
           What the killed settlement answered.
           */
          const run = await aCommandKilledAt({
            command: 'settle',
            queueFolder: queue.queueFolder,
            runsFolder: queue.runsFolder,
            killAt,
          },);
          /**
           The names the two passes started, in order.
           */
          const started: string[] = [];
          // oxlint-disable-next-line eslint/no-await-in-loop -- the first recovering pass
          await aPassOver({ queueFolder: queue.queueFolder, runsFolder: queue.runsFolder, started, },);
          // oxlint-disable-next-line eslint/no-await-in-loop -- the second recovering pass
          await aPassOver({ queueFolder: queue.queueFolder, runsFolder: queue.runsFolder, started, },);
          // oxlint-disable-next-line eslint/no-await-in-loop -- what the passes left
          const homes = await homesOfA({ queueFolder: queue.queueFolder, runsFolder: queue.runsFolder, },);
          rows.push({
            killAt,
            signal: run.signal,
            started,
            homes: homes.homes,
          },);
          // The dead filling's attempt started the entry once, and its unit's output file is the evidence: no recovering pass may start it
          // again, and nothing may leave it queued beside that start.
          expect(`${String(killAt,)}: started ${started.join(',',)}`,).toBe(`${String(killAt,)}: started `,);
          expect(`${String(killAt,)}: homes ${String(homes.homes,)}`,).toBe(`${String(killAt,)}: homes 0`,);
          expect(homes.claimed,).toBe(false,);
        }
        /* oxlint-enable eslint/no-await-in-loop */
        // The walk killed the settlement at more than one instruction, which is what says it walked anything.
        expect(rows.filter(function wasKilled(row,) {
          return row.signal === 'SIGKILL';
        },).length,).toBeGreaterThan(1,);
      },
    },),
    it({
      name: 'starts one entry at most once when a whole filling pass is killed between its claim, its start and its completion',
      fn: async () => {
        /**
         The rows of the walk, one per instruction the pass was killed at.
         */
        const rows: { readonly killAt: number; readonly steps: number; readonly started: readonly string[]; readonly homes: number }[] = [];
        /* oxlint-disable eslint/no-await-in-loop -- one kill at a time, and each round is a fixture of its own */
        for (let killAt = 1; killAt <= 8; killAt += 1) {
          await using queue = await aQueueOfOne();
          /**
           What the killed pass answered.
           */
          const run = await aCommandKilledAt({
            command: 'claim',
            queueFolder: queue.queueFolder,
            runsFolder: queue.runsFolder,
            killAt,
          },);
          /**
           The names the two recovering passes started.
           */
          const started: string[] = [];
          // oxlint-disable-next-line eslint/no-await-in-loop -- the first recovering pass
          await aPassOver({ queueFolder: queue.queueFolder, runsFolder: queue.runsFolder, started, },);
          // oxlint-disable-next-line eslint/no-await-in-loop -- the second recovering pass
          await aPassOver({ queueFolder: queue.queueFolder, runsFolder: queue.runsFolder, started, },);
          // oxlint-disable-next-line eslint/no-await-in-loop -- what the passes left
          const homes = await homesOfA({ queueFolder: queue.queueFolder, runsFolder: queue.runsFolder, },);
          rows.push({
            killAt,
            steps: run.steps,
            started,
            homes: homes.homes,
          },);
          // A claim killed before it was linked leaves the entry queued and unclaimed, so the passes start it once; a claim linked and then
          // killed leaves a dead claim with no evidence, so the passes give it back and start it once. Never twice, never lost.
          expect(started.length,).toBe(1,);
          expect(homes.homes,).toBe(0,);
        }
        /* oxlint-enable eslint/no-await-in-loop */
        expect(rows.length,).toBeGreaterThan(2,);
      },
    },),
  ],
},);
