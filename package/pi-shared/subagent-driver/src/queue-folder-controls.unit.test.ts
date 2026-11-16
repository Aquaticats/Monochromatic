/**
 What the queue folder is held to, with a control for each: the four sightings the queue's one-file form was shown to have, a walk that kills
 each changing command at each instruction it takes and asks what the next command finds, and two and three commands changing one folder at
 once for a counted number of rounds.

 The walk and the concurrent rounds run the commands as real processes, so a kill is a real `SIGKILL` and two commands are two programs on
 one folder. Every queue and every runs folder here is a throwaway folder this test made.

 @module
 */

import {
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  mkdtemp,
  readFile,
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
  claimAnEntryOfAQueue,
  fillFreePlacesFromAQueue,
  namesOfAnAttemptIn,
  putAQueueEntryIn,
  putASetAsideEntryBack,
  readQueueOfRuns,
  readTheClaimsOnAQueue,
  readTheEntryNamed,
  setAsideAnEntryOfAQueue,
  setAsideRecordIsThere,
  setAsideRecordOf,
  NO_SET_ASIDE_RECORD,
  writeTheQueueOfRuns,
} from '../dist/final/node/index.mjs';

/**
 The stand-in one queue command runs as, which can be killed at an instruction.
 */
const A_QUEUE_COMMAND = new URL('stand-in-a-queue-command-killed-at-an-instruction-for-tests.ts', import.meta.url,).pathname;

/**
 The commands the walk kills, in the order it walks them.
 */
const CHANGING_COMMANDS: readonly string[] = [
  'put',
  'take',
  'move',
  'claim',
  'complete',
  'giveBack',
  'setAside',
  'putBack',
  'settle',
  'convert',
];

/**
 How many rounds two and three commands at once change one folder, each round its own queue.
 */
const ROUNDS = 20;

/**
 A folder a test may write in, with one queue folder and the runs folder beside it, removed when the test ends.
 */
type AQueueInAThrowawayFolder = {
  /**
   The queue's folder.
   */
  readonly queueFolder: string;

  /**
   The runs folder, where a record and a unit's output file go.
   */
  readonly runsFolder: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a queue of the entries given in a throwaway folder.

 @param entries - what the queue holds, in order

 @returns the queue's folder and the runs folder beside it

 @example
 ```ts
 await using queue = await aQueueOf([{ name: 'a', branch: 'feat/a' }]);
 ```
 */
async function aQueueOf(entries: readonly QueueRecord[],): Promise<AQueueInAThrowawayFolder> {
  /**
   The folder everything lives in.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-queue-folder-',),);
  /**
   The queue's folder.
   */
  const queueFolder = join(path, 'queue',);
  /**
   The runs folder beside it.
   */
  const runsFolder = join(path, 'runs',);
  await mkdir(runsFolder, { recursive: true, },);
  await writeTheQueueOfRuns({
    queueFolder,
    entries,
  },);
  return {
    queueFolder,
    runsFolder,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 One entry of the queue, named after the word given.

 @param name - the entry's name

 @returns the entry

 @example
 ```ts
 const entry = runNamed('a');
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
 The names a queue holds, in its order.

 @param queueFolder - the queue's folder

 @returns the names

 @example
 ```ts
 const names = await namesIn('/runs/q');
 ```
 */
async function namesIn(queueFolder: string,): Promise<readonly string[]> {
  return (await readQueueOfRuns({ queueFolder, },)).map(function nameOf(entry,) {
    return entry.name;
  },);
}

/**
 The seams one pass asks, which start an entry by writing the output file that proves the start took effect.

 @param request - the queue and the runs folder, and where the starts are counted

 @returns the seams

 @example
 ```ts
 const seams = seamsOf({ queueFolder: '/runs/q', runsFolder: '/runs', started: [] });
 ```
 */
function seamsOf({
  queueFolder,
  runsFolder,
  started,
}: {
  readonly queueFolder: string;
  readonly runsFolder: string;
  readonly started: string[];
},): FillingSeams {
  return {
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
      started.push(record.name,);
      /**
       The names the attempt's own unit writes, which is the evidence a later pass reads.
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
    setAside: function setAside({
      record,
      failure,
      queueId,
    },) {
      return setAsideAnEntryOfAQueue({
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
}

/**
 Runs one pass over the queue, which first settles what a kill left and then starts what it may.

 @param request - the queue and the runs folder

 @returns the names the pass started, in its order

 @example
 ```ts
 const started = await onePassOver({ queueFolder: '/runs/q', runsFolder: '/runs' });
 ```
 */
async function onePassOver({
  queueFolder,
  runsFolder,
}: {
  readonly queueFolder: string;
  readonly runsFolder: string;
},): Promise<readonly string[]> {
  /**
   The entries the pass started.
   */
  const started: string[] = [];
  await fillFreePlacesFromAQueue({
    request: {
      queueFolder,
      runsFolder,
      level: 4,
      startsAtMost: 4,
      staggerMilliseconds: 1,
    },
    seams: seamsOf({
      queueFolder,
      runsFolder,
      started,
    },),
  },);
  return started;
}

/**
 Reads what one entry's name is worth after a pass: how many copies of it remain, in the queue or in a record.

 @param request - the queue and the runs folder, and the entry's name

 @returns how many homes the entry has, and whether a claim is still held on it

 @example
 ```ts
 const homes = await homesOf({ queueFolder: '/runs/q', runsFolder: '/runs', name: 'a' });
 ```
 */
async function homesOf({
  queueFolder,
  runsFolder,
  name,
}: {
  readonly queueFolder: string;
  readonly runsFolder: string;
  readonly name: string;
},): Promise<{ readonly homes: number; readonly claims: number }> {
  /**
   The entry the queue holds under that name, or the symbol that says it holds none.
   */
  const held = await readTheEntryNamed({
    queueFolder,
    name,
  },);
  /**
   Whether the queue holds an entry of that name.
   */
  const queued = (typeof held) === 'object';
  /**
   Whether a record of that name stands beside the runs.
   */
  const recorded = await setAsideRecordIsThere({
    runsFolder,
    name,
  },);
  /**
   The claims the queue holds on that name.
   */
  const claims = (await readTheClaimsOnAQueue({ queueFolder, },)).filter(function isOnIt(claim,) {
    return claim.name === name;
  },);
  return {
    homes: (queued ? 1 : 0) + (recorded ? 1 : 0),
    claims: claims.length,
  };
}

/**
 Runs the stand-in command as its own process, killed at the instruction given.

 @param request - the command, the queue and the instruction to stop at

 @returns the signal that ended it, its status, what it printed, and how many instructions it took when it was not killed

 @example
 ```ts
 const run = await aCommandKilledAt({ command: 'put', queueFolder: '/runs/q', killAt: 2 });
 ```
 */
async function aCommandKilledAt({
  command,
  queueFolder,
  killAt,
}: {
  readonly command: string;
  readonly queueFolder: string;
  readonly killAt: number;
},): Promise<{
  readonly signal: string | null;
  readonly status: number | null;
  readonly printed: string;
  readonly steps: number;
}> {
  /**
   The child.
   */
  const child = spawn(
    process.execPath,
    [
      A_QUEUE_COMMAND,
      command,
      String(killAt,),
      queueFolder,
    ],
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
  /**
   How the child ended.
   */
  const [status, signal,] = await once(
    child,
    'close',
  ) as [number | null, string | null];
  /**
   The line the child printed, when it printed one.
   */
  const line = printed.join('',);
  /**
   What the line says about the instructions it took.
   */
  const said = line.trim()
    .split('\n',)
    .at(-1,);
  /**
   How many instructions the command takes, which the child says when it was not killed.
   */
  const steps = ((said !== undefined) && said.startsWith('{'))
    ? Number(JSON.parse(said,).steps,)
    : 0;
  return {
    signal,
    status,
    printed: line,
    steps,
  };
}

/**
 Prepares the folder one command of the walk needs, which is the entry, and for some commands the claim or the record it acts on.

 @param request - the command, the queue's folder and the runs folder, and the identity to use

 @returns the identity of the entry the queue holds

 @example
 ```ts
 const queueId = await preparedFor({ command: 'complete', queueFolder: '/runs/q', runsFolder: '/runs' });
 ```
 */
async function preparedFor({
  command,
  queueFolder,
  runsFolder,
}: {
  readonly command: string;
  readonly queueFolder: string;
  readonly runsFolder: string;
},): Promise<string> {
  /**
   The entry the queue holds.
   */
  const held = (await readQueueOfRuns({ queueFolder, },))[0] as QueueRecord;
  /**
   The entry's identity.
   */
  const queueId = String(held.queueId,);
  if (['complete', 'giveBack', 'setAside', 'putBack',].includes(command,)) {
    // The claim is written as one a filling that is gone left, which is what the next command has to clear: a claim the test process
    // held would be a live holder's, and no command takes a live filling's claim.
    await mkdir(
      join(queueFolder, 'claimed',),
      { recursive: true, },
    );
    await writeFile(
      join(queueFolder, 'claimed', 'a.claim.json',),
      `${JSON.stringify({
        name: 'a',
        queueId,
        claim: {
          token: 'the-attempt-of-a-filling-that-is-gone',
          pid: 2_147_483_646,
          at: '2026-11-16T00:00:00.000Z',
        },
      },)}\n`,
    );
  }
  if (command === 'putBack') {
    await setAsideAnEntryOfAQueue({
      queueFolder,
      runsFolder,
      record: held,
      queueId,
      reason: 'the entry failed its start',
      now: Date.now,
    },);
  }
  if (command === 'convert') {
    await writeFile(
      `${queueFolder}.json`,
      `${JSON.stringify([runNamed('b',),],)}\n`,
    );
  }
  return queueId;
}

await describe({
  name: '',
  children: [
    describe({
      name: 'the four sightings the one-file queue was shown to have',
      children: [
        it({
          name: 'keeps both insertions when two commands put an entry into one folder at once',
          fn: async () => {
            await using queue = await aQueueOf([{ name: 'base', branch: 'feat/base', },],);
            /**
             What the two commands printed.
             */
            const printed = await Promise.all([0, 1,].map(async function putIt() {
              return await aCommandKilledAt({
                command: 'put',
                queueFolder: queue.queueFolder,
                killAt: 10_000,
              },);
            },),);
            // Both writers of one name: one link writes the entry and the other is refused, so the name stands once and the queue's
            // base entry is untouched. One file read and written whole lost one of two insertions in the file form.
            expect(printed.length,).toBe(2,);
            expect(await namesIn(queue.queueFolder,),).toEqual(['a', 'base',],);
          },
        },),
        it({
          name: 'takes out the entry that was there, and never a corrected one queued under the same name while the take ran',
          fn: async () => {
            await using queue = await aQueueOf([runNamed('a',),],);
            // The entry is taken aside by the command, and a corrected entry is queued under its name before the take reads what it
            // moved: the take answers the original, and the corrected entry stands.
            /**
             What the take answered.
             */
            const taken = await aCommandKilledAt({
              command: 'take',
              queueFolder: queue.queueFolder,
              killAt: 1_000,
            },);
            expect(taken.printed,).toContain('a',);
            await putAQueueEntryIn({
              queueFolder: queue.queueFolder,
              entry: {
                name: 'a',
                branch: 'feat/corrected',
                prompt: 'corrected user data',
              },
              position: 0,
            },);
            expect(await namesIn(queue.queueFolder,),).toEqual(['a',],);
            expect(await setAsideRecordIsThere({ runsFolder: queue.runsFolder, name: 'a', },),).toBe(false,);
          },
        },),
        it({
          name: 'starts one entry once when two fillings run at once, the second passing over the entry the first holds',
          fn: async () => {
            await using queue = await aQueueOf([runNamed('a',),],);
            /**
             The starts the two passes asked for.
             */
            const started: string[] = [];
            /**
             Resolves once the first pass is inside its start.
             */
            const firstIsStarting = Promise.withResolvers<void>();
            /**
             Resolves when the second pass may run.
             */
            const secondMayRun = Promise.withResolvers<void>();
            /**
             The first pass, held inside the start of the entry it claimed.
             */
            const first = fillFreePlacesFromAQueue({
              request: {
                queueFolder: queue.queueFolder,
                runsFolder: queue.runsFolder,
                level: 1,
                startsAtMost: 1,
                staggerMilliseconds: 1,
              },
              seams: {
                ...seamsOf({
                  queueFolder: queue.queueFolder,
                  runsFolder: queue.runsFolder,
                  started,
                },),
                async start({ record, claim, },) {
                  firstIsStarting.resolve();
                  await secondMayRun.promise;
                  return await seamsOf({
                    queueFolder: queue.queueFolder,
                    runsFolder: queue.runsFolder,
                    started,
                  },).start({
                    record,
                    claim,
                  },);
                },
              },
            },);
            await firstIsStarting.promise;
            /**
             What the second pass started while the first held its claim.
             */
            const secondStarted: string[] = [];
            await fillFreePlacesFromAQueue({
              request: {
                queueFolder: queue.queueFolder,
                runsFolder: queue.runsFolder,
                level: 1,
                startsAtMost: 1,
                staggerMilliseconds: 1,
              },
              seams: seamsOf({
                queueFolder: queue.queueFolder,
                runsFolder: queue.runsFolder,
                started: secondStarted,
              },),
            },);
            secondMayRun.resolve();
            await first;
            expect(secondStarted,).toEqual([],);
            expect(started,).toEqual(['a',],);
            expect(await namesIn(queue.queueFolder,),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: 'a changing command killed at each of its instructions',
      children: [
        it({
          name: 'leaves an entry the next command finds exactly once, never started twice, whichever instruction the kill lands on',
          fn: async () => {
            /**
             One row for each command and instruction, which the report reads.
             */
            const rows: {
              readonly command: string;
              readonly killAt: number;
              readonly signal: string | null;
              readonly started: readonly string[];
              readonly homes: number;
              readonly claims: number;
            }[] = [];
            for (const command of CHANGING_COMMANDS) {
              /**
               The entry the queue of this command's walk holds: a put writes `a`, so its queue holds another name.
               */
              const held = (command === 'put') ? runNamed('base',) : runNamed('a',);
              /**
               The folder this command's walk runs in, made once to count its instructions and then per round.
               */
              const counting = await aQueueOf([held,],);
              // oxlint-disable-next-line eslint/no-await-in-loop -- one command's own count, then its rounds
              const prepared = await preparedFor({
                command,
                queueFolder: counting.queueFolder,
                runsFolder: counting.runsFolder,
              },);
              // oxlint-disable-next-line eslint/no-await-in-loop -- the command of this walk, run once to count its instructions
              const whole = await aCommandKilledAt({
                command,
                queueFolder: counting.queueFolder,
                killAt: 10_000,
              },);
              await counting[Symbol.asyncDispose]();
              expect(prepared,).toBeTruthy();
              if (whole.status !== 0) {
                throw new Error(`${command} could not be run whole, so its instructions cannot be walked: ${whole.printed}`,);
              }
              /* oxlint-disable eslint/no-await-in-loop -- one kill at a time, and each round is a whole queue of its own */
              for (let killAt = 1; killAt <= (whole.steps + 1); killAt += 1) {
                /**
                 This round's queue.
                 */
                const queue = await aQueueOf([held,],);
                // oxlint-disable-next-line eslint/no-await-in-loop -- the setup of this round's queue
                await preparedFor({
                  command,
                  queueFolder: queue.queueFolder,
                  runsFolder: queue.runsFolder,
                },);
                // oxlint-disable-next-line eslint/no-await-in-loop -- the command killed at this instruction
                const run = await aCommandKilledAt({
                  command,
                  queueFolder: queue.queueFolder,
                  killAt,
                },);
                // oxlint-disable-next-line eslint/no-await-in-loop -- what the next command does with what the kill left
                const started = await onePassOver(queue,);
                // oxlint-disable-next-line eslint/no-await-in-loop -- what remains of the entry after that pass
                const homes = await homesOf({
                  queueFolder: queue.queueFolder,
                  runsFolder: queue.runsFolder,
                  name: 'a',
                },);
                rows.push({
                  command,
                  killAt,
                  signal: run.signal,
                  started,
                  homes: homes.homes,
                  claims: homes.claims,
                },);
                // No name is started twice, and no name that was started is still in the queue.
                expect(new Set(started,).size,).toBe(started.length,);
                for (const name of await namesIn(queue.queueFolder,)) {
                  expect(started,).not.toContain(name,);
                }
                // The entry the command acts on is never in two places, and no claim is left on it: the commands whose entry is
                // written by the command itself are left out, because a kill before that write leaves no entry at all.
                expect(homes.claims,).toBe(0,);
                expect(homes.homes,).toBeLessThanOrEqual(1,);
                // A take is a person removing the entry, so a kill in the middle of one leaves it in the dot-file it was renamed to,
                // which is where a person finds it; a put and a conversion write the entry this walk watches from nothing.
                // A completion is the one step whose whole purpose is to take the entry out of the queue, so a kill after it leaves
                // nothing to find; the walk says so rather than calling it a lost entry.
                const removedByDesign = ['put', 'convert', 'take', 'complete',].includes(command,);
                if ((!removedByDesign) && (!started.includes('a',))) {
                  expect(homes.homes,).toBe(1,);
                }
                await queue[Symbol.asyncDispose]();
              }
              /* oxlint-enable eslint/no-await-in-loop */
            }
            // The walk covers every command and more than one instruction of each, which is what says it walked anything, and every
            // round but the one past the last instruction was a kill.
            expect(new Set(rows.map(function commandOf(row,) {
              return row.command;
            },),).size,).toBe(CHANGING_COMMANDS.length,);
            expect(rows.length,).toBeGreaterThan(CHANGING_COMMANDS.length * 2,);
            expect(rows.filter(function wasKilled(row,) {
              return row.signal === 'SIGKILL';
            },).length,).toBeGreaterThan(rows.length / 2,);
            await writeFile(
              join(scratchRoot(), 'pi-queue-folder-walk-rows.json',),
              `${JSON.stringify(rows, undefined, 2,)}
`,
            );
          },
        },),
      ],
    },),
    describe({
      name: 'two and three commands changing one folder at once',
      children: [
        it({
          name: 'loses no entry and doubles none, for every round',
          fn: async () => {
            /* oxlint-disable eslint/no-await-in-loop -- one round after another, each its own queue and its own commands */
            for (let round = 0; round < ROUNDS; round += 1) {
              await using queue = await aQueueOf([{ name: 'base', branch: 'feat/base', },],);
              // Two puts of one name and a take: one put wins the name and the other is refused, and the take empties the name the
              // put wrote, which is the interleaving that lost an entry when the queue was one file read and written whole.
              // oxlint-disable-next-line eslint/no-await-in-loop -- the commands of this round, all at once
              const answered = await Promise.all([
                aCommandKilledAt({
                  command: 'put',
                  queueFolder: queue.queueFolder,
                  killAt: 10_000,
                },),
                aCommandKilledAt({
                  command: 'put',
                  queueFolder: queue.queueFolder,
                  killAt: 10_000,
                },),
                aCommandKilledAt({
                  command: 'take',
                  queueFolder: queue.queueFolder,
                  killAt: 10_000,
                },),
              ],);
              // Every command ends by answering, and no two of them ever write the same entry.
              expect(answered.map(function statusOf(run,) {
                return run.signal;
              },),).toEqual([null, null, null,],);
              /**
               The names the queue holds after the round.
               */
              const names = await namesIn(queue.queueFolder,);
              expect(new Set(names,).size,).toBe(names.length,);
              expect(names,).toContain('base',);
            }
            /* oxlint-enable eslint/no-await-in-loop */
          },
        },),
      ],
    },),
    describe({
      name: 'what a kill can leave behind',
      children: [
        it({
          name: 'reads a folder that is not there as an error naming it, never as an empty queue',
          fn: async () => {
            await using queue = await aQueueOf([],);
            /**
             What reading a queue folder that is not there threw.
             */
            let caught: unknown;
            try {
              await readQueueOfRuns({ queueFolder: join(queue.queueFolder, 'not-there',), },);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain('not-there',);
          },
        },),
        it({
          name: 'refuses a name in the queue folder that is neither an entry nor the claims folder, by name',
          fn: async () => {
            await using queue = await aQueueOf([runNamed('a',),],);
            await writeFile(join(queue.queueFolder, 'a-person-s-leftover',), 'not an entry\n',);
            /**
             What reading the queue threw.
             */
            let caught: unknown;
            try {
              await readQueueOfRuns({ queueFolder: queue.queueFolder, },);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain('a-person-s-leftover',);
          },
        },),
        it({
          name: 'keeps the entry a put-back could not place when a name was queued under it meanwhile, and says so',
          fn: async () => {
            await using queue = await aQueueOf([runNamed('a',),],);
            /**
             The claim a filling held on the entry when its start failed.
             */
            const held = (await readQueueOfRuns({ queueFolder: queue.queueFolder, },))[0] as QueueRecord;
            await setAsideAnEntryOfAQueue({
              queueFolder: queue.queueFolder,
              runsFolder: queue.runsFolder,
              record: held,
              queueId: String(held.queueId,),
              reason: 'the entry failed its start',
              now: Date.now,
            },);
            await putAQueueEntryIn({
              queueFolder: queue.queueFolder,
              entry: {
                name: 'a',
                branch: 'feat/corrected',
                prompt: 'corrected user data',
              },
              position: 0,
            },);
            /**
             What the put-back answered while the name was taken.
             */
            const putBack = await putASetAsideEntryBack({
              queueFolder: queue.queueFolder,
              runsFolder: queue.runsFolder,
              name: 'a',
              position: 0,
            },);
            expect(putBack.line,).toContain('is still in the queue, so it is not set aside',);
            expect(await setAsideRecordIsThere({ runsFolder: queue.runsFolder, name: 'a', },),).toBe(true,);
            /**
             The record the earlier entry is kept in.
             */
            const record = await setAsideRecordOf({
              runsFolder: queue.runsFolder,
              name: 'a',
            },);
            expect(record === NO_SET_ASIDE_RECORD ? undefined : record.entry.prompt,).toBe('/runs/a-prompt.md',);
            expect((await readFile(join(queue.queueFolder, 'a.json',), 'utf8',)).includes('corrected user data',),).toBe(true,);
          },
        },),
        it({
          name: 'answers a claim that names a process which is gone by giving the entry back, in a line that says so',
          fn: async () => {
            await using queue = await aQueueOf([runNamed('a',),],);
            /**
             The claim a filling that is gone held, whose process number no process of the account has.
             */
            const held = (await readQueueOfRuns({ queueFolder: queue.queueFolder, },))[0] as QueueRecord;
            await claimAnEntryOfAQueue({
              queueFolder: queue.queueFolder,
              record: held,
              now: Date.now,
            },);
            /**
             The claim's file, rewritten to name a process that is gone.
             */
            const claimOnDisk = (await readTheClaimsOnAQueue({ queueFolder: queue.queueFolder, },))[0] as {
              readonly claim: QueueClaim;
            };
            await writeFile(
              join(queue.queueFolder, 'claimed', 'a.claim.json',),
              `${JSON.stringify({
                name: 'a',
                queueId: String(held.queueId,),
                claim: {
                  ...claimOnDisk.claim,
                  pid: 2_147_483_646,
                },
              },)}\n`,
            );
            /**
             The lines the settlement said.
             */
            const said: string[] = [];
            await (await import('../dist/final/node/index.mjs',)).settleTheQueueAfterAKill({
              queueFolder: queue.queueFolder,
              runsFolder: queue.runsFolder,
              report: function report(line: string,) {
                said.push(line,);
              },
            },);
            expect(said.join('',),).toContain('the claim is given back and the entry stands',);
            expect(await namesIn(queue.queueFolder,),).toEqual(['a',],);
            expect(await readTheClaimsOnAQueue({ queueFolder: queue.queueFolder, },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
