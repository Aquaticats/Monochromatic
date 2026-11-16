/**
 What the queue promises, held by tests the earlier round did not have: two real processes claiming one entry at the same instant, the sweep
 that must never delete an entry a take set aside, the settlement's order of steps, the put back that must not write over a name another
 writer took, and the changeover's unfinished write.

 Each control was shown to fail with its behaviour taken out; the outputs are in the run's report.

 @module
 */

import {
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';
import {
  appendFile,
  mkdir,
  mkdtemp,
  readdir,
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
  BEING_CARRIED_ENDING,
  BEING_WRITTEN_START,
  CARRIED_ENDING,
  HALF_WRITTEN_FILE_AGE_MILLISECONDS,
  NO_ENTRY_OF_THAT_NAME,
  NO_SET_ASIDE_RECORD,
  QueueFileError,
  QUEUE_ID_FIELD,
  aClaimGivenBackOnlyIfOurs,
  aClaimOf,
  aPlaceFor,
  claimAnEntryOfAQueue,
  claimFileOf,
  convertAQueueFileIntoAFolder,
  fillFreePlacesFromAQueue,
  halfWrittenFilesSwept,
  namesInAFolderThatMayNotBeThere,
  orderOf,
  putAQueueEntryIn,
  putBackUnlessTheNameIsTaken,
  queueIdOf,
  readQueueOfRuns,
  readTheEntryNamed,
  readTheQueueReportingProblems,
  setAsideAnEntryOfAQueue,
  settleTheQueueAfterAKill,
  startedAtOf,
  takeAQueueEntryOut,
  writeTheQueueOfRuns,
} from '../dist/final/node/index.mjs';

/**
 The stand-in one claimer runs as, so two claims are two processes.
 */
const A_CLAIMER = new URL('stand-in-a-claimer-for-tests.ts', import.meta.url,).pathname;

/**
 How many rounds two real processes claim one entry at the same instant, and why: the check-then-write claim the review took out answered
 both processes `claimed` in 6 of its 50 rounds, so 50 rounds of this test find that shape with near certainty, and the exclusive link it
 stands for answers one claimer in every one of them.
 */
const ROUNDS_OF_TWO_CLAIMERS = 50;

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
 Makes a queue of the entries given in a throwaway folder.

 @param entries - what the queue holds, in order

 @returns the folders

 @example
 ```ts
 await using queue = await aQueueOf([{ name: 'a', branch: 'feat/a' }]);
 ```
 */
async function aQueueOf(entries: readonly Record<string, unknown>[],): Promise<AQueueInAThrowawayFolder> {
  /**
   The folder everything lives in.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-queue-promises-',),);
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
    entries: entries as never,
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
 One entry of the queue, named after the word given.

 @param name - the entry's name

 @returns the entry

 @example
 ```ts
 const entry = runNamed('a');
 ```
 */
function runNamed(name: string,): Record<string, unknown> {
  return {
    name,
    branch: `feat/${name}`,
    wall: 28_800,
  };
}

/**
 Asks one claimer program to claim the entry named `a` at a moment the test names.

 @param request - the queue's folder and the moment to claim at

 @returns what the claimer printed

 @example
 ```ts
 const answer = await aClaimer({ queueFolder: '/runs/q', at: '2026-11-16T00:00:00.000Z' });
 ```
 */
async function aClaimer({
  queueFolder,
  at,
}: {
  readonly queueFolder: string;
  readonly at: string;
},): Promise<string> {
  /**
   The child.
   */
  const child = spawn(
    process.execPath,
    [A_CLAIMER, queueFolder, at,],
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
  await once(
    child,
    'close',
  );
  return printed.join('',)
    .trim();
}

/**
 Says which claimers succeeded out of two that claimed at one instant.

 @param answers - what each claimer printed

 @returns how many answered `claimed`

 @example
 ```ts
 const won = howManyClaimed(['claimed', 'claimed-by-another']);
 ```
 */
function howManyClaimed(answers: readonly string[],): number {
  return answers.filter(function isClaimed(answer,) {
    return answer.includes('claimed',) && (!answer.includes('claimed-by-another',));
  },).length;
}

await describe({
  name: '',
  children: [
    it({
      name: 'hands one entry to exactly one of two real processes that claim it at the same instant, in every round',
      fn: async () => {
        /* oxlint-disable eslint/no-await-in-loop -- one round after another, each round two real processes started together, which is
           what a race control measures. See doc/lint/sequential-awaits.md. */
        for (let round = 0; round < ROUNDS_OF_TWO_CLAIMERS; round += 1) {
          await using queue = await aQueueOf([runNamed('a',),],);
          /**
           The moment both processes are told to claim at, which is one string so neither can be later by reading the clock.
           */
          const at = '2026-11-16T00:00:00.000Z';
          /**
           What the two claimers answered.
           */
          const answers = await Promise.all([
            aClaimer({ queueFolder: queue.queueFolder, at, },),
            aClaimer({ queueFolder: queue.queueFolder, at, },),
          ],);
          // Exactly one of the two was answered `claimed`, and the other was answered that another filling holds it.
          expect(`${String(round,)}: ${String(howManyClaimed(answers,),)}`,).toBe(`${String(round,)}: 1`,);
          expect(`: ${answers.filter(function isHeldByAnother(answer,) {
            return answer === 'claimed-by-another';
          },).length}`,).toBe(': 1',);
        }
        /* oxlint-enable eslint/no-await-in-loop */
      },
    },),
    it({
      name: 'keeps the entry a take renamed aside however old it is, and says where it is, while removing only a file a command was writing',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',),],);
        // An entry that waited three hours, then a take that was killed between its rename and its printing: the aside file holds the only
        // copy, and it is as old as the entry it came from.
        /**
         The aside file a killed take leaves, aged three hours.
         */
        const aside = join(queue.queueFolder, '.being-taken-a.json-1-2',);
        await writeFile(aside, `${JSON.stringify(runNamed('a',),)}\n`,);
        await rm(join(queue.queueFolder, 'a.json',),);
        /**
         What the sweep said.
         */
        const said: string[] = [];
        expect(await halfWrittenFilesSwept({
          queueFolder: queue.queueFolder,
          report: function report(line,) {
            said.push(line,);
          },
          now: function threeHoursLater() {
            return Date.now() + (3 * HALF_WRITTEN_FILE_AGE_MILLISECONDS);
          },
        },),).toBe(0,);
        expect(await readdir(queue.queueFolder,),).toContain('.being-taken-a.json-1-2');
        expect(said.join('',),).toContain('a person can put it back',);
        // A file a command was writing is a half of a write and never an entry, and it does go once it is too old to belong to a step.
        /**
         The file a killed put left while writing its entry.
         */
        const halfWritten = join(queue.queueFolder, `${BEING_WRITTEN_START}a.json-1-3`,);
        await writeFile(halfWritten, '{"name":"a"',);
        expect(await halfWrittenFilesSwept({
          queueFolder: queue.queueFolder,
          report: function report() {
            return undefined;
          },
          now: function threeHoursLater() {
            return Date.now() + (3 * HALF_WRITTEN_FILE_AGE_MILLISECONDS);
          },
        },),).toBe(1,);
        expect(await readdir(queue.queueFolder,),).not.toContain(`${BEING_WRITTEN_START}a.json-1-3`);
      },
    },),
    it({
      name: 'takes an entry out of the queue when the filling that claimed it died after its start took effect, and leaves both alone while that filling lives',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',),],);
        // The claim of a filling that is gone, whose attempt's unit wrote its output file: the entry started, so it leaves the queue.
        /**
         The entry as the queue holds it.
         */
        const held = (await readQueueOfRuns({ queueFolder: queue.queueFolder, },))[0] as Record<string, unknown>;
        /**
         The identity the queue holds.
         */
        const queueId = String(held.queueId,);
        /**
         The moment this test's own process started, which a live claim carries.
         */
        const started = await startedAtOf({ pid: process.pid, },);
        /**
         The claim a dead filling left.
         */
        const dead = aClaimOf({
          name: 'a',
          queueId,
          claim: {
            token: 'the-attempt-of-a-filling-that-is-gone',
            pid: 2_147_483_646,
            at: '2026-11-16T00:00:00.000Z',
            startedAt: '1',
          },
        },);
        await mkdir(join(queue.queueFolder, 'claimed',), { recursive: true, },);
        await writeFile(claimFileOf({ queueFolder: queue.queueFolder, name: 'a', },), `${JSON.stringify(dead,)}\n`,);
        // The attempt's own unit wrote its output file, which is the evidence a start took effect.
        await writeFile(
          join(
            queue.runsFolder,
            `batch-2026-11-16T000000-the-attempt-of-a-filling-that-is-gone-queue-entry-a.out`,
          ),
          'the unit started\n',
        );
        /**
         What the settlement said.
         */
        const said: string[] = [];
        await settleTheQueueAfterAKill({
          queueFolder: queue.queueFolder,
          runsFolder: queue.runsFolder,
          report: function report(line,) {
            said.push(line,);
          },
        },);
        // The entry is out of the queue, and the folder holds nothing but the claims folder the claim made.
        expect((await readdir(queue.queueFolder,)).filter(function isAnEntry(name,) {
          return name.endsWith('.json',);
        },),).toEqual([],);
        expect((await readdir(join(queue.queueFolder, 'claimed',),)).filter(function isAClaim(name,) {
          return name.endsWith('.claim.json',);
        },),).toEqual([],);
        expect(said.join('',),).toContain('never started again',);
        // A claim its own filling holds is left alone with its entry, whatever the record beside the runs says.
        await using second = await aQueueOf([runNamed('a',),],);
        /**
         The entry the second queue holds.
         */
        const mine = (await readQueueOfRuns({ queueFolder: second.queueFolder, },))[0] as Record<string, unknown>;
        /**
         What taking a claim on it answered.
         */
        const claimed = await claimAnEntryOfAQueue({
          queueFolder: second.queueFolder,
          record: mine as never,
          now: Date.now,
        },);
        expect(claimed.kind,).toBe('claimed',);
        // A record of the same identity beside the runs is what an interrupted set-aside leaves; the settlement must not finish it while
        // the filling holding the claim is alive.
        await writeFile(
          join(second.runsFolder, 'a-set-aside.json',),
          `${JSON.stringify({
            name: 'a',
            entry: {
              ...mine,
            },
            reason: 'the entry failed its start',
            setAsideAt: '2026-11-16T00:00:00.000Z',
          },)}\n`,
        );
        await settleTheQueueAfterAKill({
          queueFolder: second.queueFolder,
          runsFolder: second.runsFolder,
          report: function report() {
            return undefined;
          },
        },);
        expect(await readdir(second.queueFolder,),).toContain('a.json');
        expect(started.kind,).toBe('there',);
      },
    },),
    it({
      name: 'puts a claim back only while its own token is there, and never writes over a name another writer took',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',),],);
        /**
         The entry as the queue holds it.
         */
        const held = (await readQueueOfRuns({ queueFolder: queue.queueFolder, },))[0] as Record<string, unknown>;
        /**
         What taking a claim on it answered.
         */
        const claimed = await claimAnEntryOfAQueue({
          queueFolder: queue.queueFolder,
          record: held as never,
          now: Date.now,
        },);
        expect(claimed.kind,).toBe('claimed',);
        // A give-back that names another token leaves the claim: only the attempt whose token it is may give it back.
        expect(await aClaimGivenBackOnlyIfOurs({
          queueFolder: queue.queueFolder,
          name: 'a',
          token: 'a-token-that-is-not-the-claim\'s',
        },),).toBe(false,);
        expect(
          await readdir(join(queue.queueFolder, 'claimed',),),
        ).toContain('a.claim.json');
        expect(await aClaimGivenBackOnlyIfOurs({
          queueFolder: queue.queueFolder,
          name: 'a',
          token: claimed.kind === 'claimed' ? claimed.claim.token : '',
        },),).toBe(true,);
        expect(
          await readdir(join(queue.queueFolder, 'claimed',),),
        ).not.toContain('a.claim.json');
        // A file put back to a name another writer took meanwhile is never written over: the put back answers that the name was taken, and
        // both files stand, so neither writer's work is lost.
        /**
         The file a caller renamed aside, and the file another writer made at its name meanwhile.
         */
        const aside = join(queue.queueFolder, 'aside.json',);
        const taken = join(queue.queueFolder, 'b.json',);
        await writeFile(aside, `${JSON.stringify(runNamed('a',),)}\n`,);
        await writeFile(taken, `${JSON.stringify(runNamed('b',),)}\n`,);
        expect(await putBackUnlessTheNameIsTaken({ from: aside, to: taken, },),).toBe(false,);
        expect(
          JSON.parse(await readFile(taken, 'utf8',),),
        ).toMatchObject({ name: 'b', },);
        expect(
          JSON.parse(await readFile(aside, 'utf8',),),
        ).toMatchObject({ name: 'a', },);
        // And a free name takes it, which is what says the refusal is about the name being taken.
        expect(await putBackUnlessTheNameIsTaken({
          from: aside,
          to: join(queue.queueFolder, 'c.json',),
        },),).toBe(true,);
        expect(await readdir(queue.queueFolder,),).not.toContain('aside.json');
      },
    },),
    it({
      name: 'reports a file a reader cannot take, by name, and takes a file removed meanwhile as no entry rather than a failure',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',),],);
        /**
         The lines the reader said.
         */
        const said: string[] = [];
        // A file listed and then removed before it is read is one another command took out, which is a definite state.
        await writeFile(join(queue.queueFolder, 'gone.json',), `${JSON.stringify(runNamed('gone',),)}\n`,);
        const entries = await readTheQueueReportingProblems({
          queueFolder: queue.queueFolder,
          report: function report(line,) {
            said.push(line,);
          },
        },);
        expect(entries.map(function nameOf(entry,) {
          return entry.name;
        },),).toContain('a',);
        expect(await readTheEntryNamed({ queueFolder: queue.queueFolder, name: 'gone', },),).not.toBe(NO_ENTRY_OF_THAT_NAME);
        // A half-written entry file is named and passed over, and the entry behind it is read.
        await writeFile(join(queue.queueFolder, 'half.json',), '{"name":"half"',);
        /**
         The lines the reader said about the half-written file.
         */
        const second: string[] = [];
        const read = await readTheQueueReportingProblems({
          queueFolder: queue.queueFolder,
          report: function report(line,) {
            second.push(line,);
          },
        },);
        expect(read.map(function nameOf(entry,) {
          return entry.name;
        },),).toContain('a',);
        expect(second.join('',),).toContain('half.json',);
        expect(said.join('',),).not.toContain('half.json',);
      },
    },),
    it({
      name: 'puts the first entry of a queue into a folder that is not there yet, and orders two entries of one key by their names',
      fn: async () => {
        /**
         The folder everything lives in.
         */
        /**
         The folder everything lives in, removed when this test ends however it ends.
         */
        await using folder = await aFolderOfItsOwn({ prefix: 'pi-queue-first-put-', },);
        /**
         The queue's folder, which nothing has made.
         */
        const queueFolder = join(folder.path, 'queue',);
        expect(await putAQueueEntryIn({
          queueFolder,
          entry: runNamed('a',) as never,
          position: 0,
        },),).toEqual({
          position: 0,
          count: 1,
        },);
        expect((await readQueueOfRuns({ queueFolder, },)).map(function nameOf(entry,) {
          return entry.name;
        },),).toEqual(['a',],);
        // Two entries of one key are ordered by their names, which is one total order: the same queue written twice reads the same way.
        await using queue = await aQueueOf([],);
        /**
         A key both entries are written at.
         */
        const together = aPlaceFor({ entries: [], position: 0, },).order;
        for (const name of ['zed', 'alpha',]) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- one entry of this fixture after another
          await writeFile(
            join(queue.queueFolder, `${name}.json`,),
            `${JSON.stringify({
              name,
              branch: `feat/${name}`,
              order: together,
              queueId: `the-identity-of-${name}`,
            },)}\n`,
          );
        }
        /**
         The names the queue reads, in its order.
         */
        const names = (await readQueueOfRuns({ queueFolder: queue.queueFolder, },)).map(function nameOf(entry,) {
          return entry.name;
        },);
        expect(names,).toEqual(['alpha', 'zed',],);
        expect(orderOf({ record: (await readQueueOfRuns({ queueFolder: queue.queueFolder, },))[0] as never, },),).toBe(together,);
      },
    },),
    it({
      name: 'leaves a take of one rename, whose entry a person finds until the command that took it discards it',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',),],);
        /**
         What the take answered.
         */
        const taken = await takeAQueueEntryOut({
          queueFolder: queue.queueFolder,
          name: 'a',
        },);
        expect(taken.asides,).toHaveLength(1,);
        // The entry is out of the queue and whole in the file the take left, which is what a command that cannot print keeps.
        expect(await readdir(queue.queueFolder,),).not.toContain('a.json');
        /**
         The entry the aside file holds.
         */
        const kept = JSON.parse(await readFile(taken.asides[0] as string, 'utf8',),) as Record<string, unknown>;
        expect(kept.name,).toBe('a',);
        expect(queueIdOf({ record: kept as never, },),).toBe(queueIdOf({ record: (taken.taken[0] as never) ?? {}, },),);
      },
    },),
    it({
      name: 'refuses to carry a changeover further when it cannot tell an entry it was writing from one a filling started, and names the entry',
      fn: async () => {
        await using queue = await aQueueOf([],);
        /**
         The array a changeover was carrying when it was killed.
         */
        const queueFile = join(queue.path, 'queue.json',);
        await writeFile(`${queueFile}${BEING_CARRIED_ENDING}`, `${JSON.stringify([runNamed('a',), runNamed('b',),],)}\n`,);
        // `a` was written and marked unfinished when the changeover died: nothing can tell whether a filling started it, so the command
        // refuses rather than queueing a paid run twice.
        await appendFile(`${queueFile}${CARRIED_ENDING}`, 'being-written a\n',);
        /**
         What the changeover threw.
         */
        let caught: unknown;
        try {
          await convertAQueueFileIntoAFolder({
            queueFile,
            queueFolder: queue.queueFolder,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(QueueFileError,);
        expect(String(caught,),).toContain('"a"',);
        expect(String(caught,),).toContain('being-written',);
        expect(await readdir(queue.queueFolder,),).toEqual([],);
        // A name marked written is passed over, and the entries after it are carried.
        await writeFile(`${queueFile}${CARRIED_ENDING}`, 'written a\n',);
        /**
         What the second changeover did.
         */
        const done = await convertAQueueFileIntoAFolder({
          queueFile,
          queueFolder: queue.queueFolder,
        },);
        expect(done.alreadyThere,).toBe(1,);
        expect(done.entries,).toBe(1,);
        expect((await readQueueOfRuns({ queueFolder: queue.queueFolder, },)).map(function nameOf(entry,) {
          return entry.name;
        },),).toEqual(['b',],);
      },
    },),
    it({
      name: 'gives a claim back when the start throws, and when the set-aside throws, so no live filling keeps an entry',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',),],);
        /**
         The claims the queue holds after a pass whose start threw.
         */
        const afterAThrow = await aPassWhoseStartThrows({ queueFolder: queue.queueFolder, runsFolder: queue.runsFolder, },);
        expect(afterAThrow,).toEqual([],);
        // The entry is still queued, so the next pass starts it.
        expect((await readQueueOfRuns({ queueFolder: queue.queueFolder, },)).map(function nameOf(entry,) {
          return entry.name;
        },),).toEqual(['a',],);
      },
    },),
  ],
},);

/**
 Runs one pass whose start throws after the claim, and answers the claims the queue holds afterwards.

 The pass catches nothing itself, so the throw carries out of it, which is what a filling's own round does.

 @param request - the queue's folder and the runs folder

 @returns the claim files the queue holds after the pass

 @example
 ```ts
 const left = await aPassWhoseStartThrows({ queueFolder: '/runs/q', runsFolder: '/runs' });
 ```
 */
/**
 Makes one throwaway folder of a test's own, which it removes when it ends.

 @param request - the prefix its name begins with

 @returns the folder

 @example
 ```ts
 await using folder = await aFolderOfItsOwn({ prefix: 'pi-queue-first-put-' });
 ```
 */
async function aFolderOfItsOwn({ prefix, }: { readonly prefix: string },): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
}> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), prefix,),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Runs one function and answers what it threw, or that it threw nothing.

 @param run - the function

 @returns what the function threw, or that it threw nothing

 @example
 ```ts
 const thrown = await whatAThrows({ run: async function work() { return 1; } });
 ```
 */
async function whatAThrows({ run, }: { readonly run: () => Promise<unknown> },): Promise<unknown> {
  try {
    await run();
    return NOTHING_THROWN;
  }
  catch (error) {
    return error;
  }
}

/**
 Nothing was thrown, as a symbol so no thrown value can be mistaken for it.
 */
const NOTHING_THROWN: unique symbol = Symbol('this call threw nothing',);

async function aPassWhoseStartThrows({
  queueFolder,
  runsFolder,
}: {
  readonly queueFolder: string;
  readonly runsFolder: string;
},): Promise<readonly string[]> {
  /**
   What the pass threw.
   */
  const caught = await whatAThrows({ run: async function fill() {
    await fillFreePlacesFromAQueue({
      request: {
        queueFolder,
        runsFolder,
        level: 1,
        startsAtMost: 1,
        staggerMilliseconds: 1,
      },
      seams: {
        runsGoing: function runsGoing() {
          return Promise.resolve([],);
        },
        freeSpace: function freeSpaceNow() {
          return {
            volume: runsFolder,
            freeBytes: 900 * 1_024 * 1_024 * 1_024,
          };
        },
        start: function start() {
          throw new Error('the start threw after the claim was taken',);
        },
        setAside: function setAside() {
          throw new Error('the start threw, so nothing is set aside',);
        },
        wait: function wait() {
          return Promise.resolve();
        },
        report: function report() {
          return undefined;
        },
      },
    },);
  } },);
  expect(caught,).toBeInstanceOf(Error,);
  /**
   The claims the queue folder holds, or none when a queue nothing has claimed in holds no claims folder.
   */
  const held = await namesInAFolderThatMayNotBeThere({ folder: join(queueFolder, 'claimed',), },);
  return held.filter(function isAClaim(name,) {
    return name.isFile() && name.name.endsWith('.claim.json',);
  },).map(function nameOf(name,) {
    return name.name;
  },);
}

/**
 Reads one entry's set-aside record, which a test asserts about after a pass.

 @param runsFolder - the runs folder

 @param name - the entry's name

 @returns the record, or that the entry is not set aside

 @example
 ```ts
 const record = await theRecordOf({ runsFolder: '/runs', name: 'a' });
 ```
 */
export async function theRecordOf({
  runsFolder,
  name,
}: {
  readonly runsFolder: string;
  readonly name: string;
},): Promise<unknown> {
  /**
   The record's path.
   */
  const file = join(
    runsFolder,
    `${name}-set-aside.json`,
  );
  try {
    return JSON.parse(await readFile(
      file,
      'utf8',
    ),);
  }
  catch (error) {
    void error;
    return NO_SET_ASIDE_RECORD;
  }
}

/**
 Writes one entry set aside for a test's fixture.

 @param request - the queue's folder, the runs folder, the entry and the reason

 @returns the record's path

 @example
 ```ts
 const file = await setOneAside({ queueFolder: '/runs/q', runsFolder: '/runs', record, reason: 'x' });
 ```
 */
export async function setOneAside({
  queueFolder,
  runsFolder,
  record,
  reason,
  queueId,
}: {
  readonly queueFolder: string;
  readonly runsFolder: string;
  readonly record: Record<string, unknown>;
  readonly reason: string;
  readonly queueId: string;
},): Promise<string> {
  /**
   What setting it aside answered.
   */
  const written = await setAsideAnEntryOfAQueue({
    queueFolder,
    runsFolder,
    record: record as never,
    queueId,
    reason,
    now: Date.now,
  },);
  return written.file;
}
