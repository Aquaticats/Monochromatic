/**
 The queue folder held to what a race and a kill can do to it: a review put back with no identity, one bad file stopping every entry, a move
 that writes a started entry back, a settling pass that removes a live filling's claim, a claim whose holder's number was reused, a start that
 answers a machine failure after its unit exists, a set-aside that writes over an earlier entry's only copy, a take that removes an entry
 before it is shown, and the changeover from the one file.

 Each control is one finding of the independent review of `7f9f312e34`, at the seam where the finding's decision is made, so the control fails
 on the code before the change and passes on it. The queue and the runs folder are a throwaway folder of each test's own.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
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
  BEING_CARRIED_ENDING,
  BUILD_VOLUME,
  CARRIED_ENDING,
  type FillingSeams,
  HALF_WRITTEN_FILE_AGE_MILLISECONDS,
  NO_SET_ASIDE_RECORD,
  type QueueClaim,
  type QueueRecord,
  aClaimGivenBackOnlyIfOurs,
  aClaimOf,
  aClaimRemovedOnlyIfItSays,
  claimAnEntryOfAQueue,
  claimFileOf,
  convertAQueueFileIntoAFolder,
  discardTheEntryTaken,
  fillFreePlacesFromAQueue,
  halfWrittenFilesSwept,
  moveAQueueEntryTo,
  namesOfAnAttemptIn,
  orderOf,
  putAQueueEntryIn,
  putEndedReviewsBack,
  queueIdOf,
  readQueueOfRuns,
  readTheEntryNamed,
  readTheClaimsOnAQueue,
  readTheQueueReportingProblems,
  setAsideAnEntryOfAQueue,
  setAsideFileOf,
  setAsideRecordOf,
  startedAtOf,
  takeAQueueEntryOut,
  theHolderOfAClaim,
  writeSetAsideRecord,
  writeTheQueueOfRuns,
} from '../dist/final/node/index.mjs';

/**
 A folder a test may write in, with one queue folder and the runs folder beside it, removed when the test ends.
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
async function aQueueOf(entries: readonly QueueRecord[],): Promise<AQueueInAThrowawayFolder> {
  /**
   The folder everything lives in.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-queue-race-',),);
  /**
   The queue's folder.
   */
  const queueFolder = join(path, 'queue',);
  /**
   The runs folder beside it.
   */
  const runsFolder = join(path, 'runs',);
  await mkdir(queueFolder, { recursive: true, },);
  await mkdir(runsFolder, { recursive: true, },);
  await writeTheQueueOfRuns({
    queueFolder,
    entries,
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
 A claim's facts, written by a filling that is no longer there.

 @param pid - the process number the claim names

 @param startedAt - the moment that process is said to have started, when it is said

 @returns the claim

 @example
 ```ts
 const claim = aClaimOfAFillingThatIsGone({ pid: 2_147_483_646 });
 ```
 */
function aClaimOfAFillingThatIsGone({
  pid = 2_147_483_646,
  startedAt,
}: {
  readonly pid?: number;
  readonly startedAt?: string;
},): QueueClaim {
  return {
    token: `the-attempt-of-${String(pid,)}`,
    pid,
    at: '2026-11-16T00:00:00.000Z',
    ...((startedAt === undefined) ? {} : { startedAt, }),
  };
}

/**
 The seams one pass asks, whose start answers what the test says.

 @param request - the queue and the runs folder, and what a start answers

 @returns the seams

 @example
 ```ts
 const seams = seamsOf({ queueFolder: '/runs/q', runsFolder: '/runs', start: 'started' });
 ```
 */
function seamsOf({
  queueFolder,
  runsFolder,
  start,
}: {
  readonly queueFolder: string;
  readonly runsFolder: string;
  readonly start: 'started' | 'the-machine-after-its-unit',
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
      /**
       The names this attempt's unit would write.
       */
      const names = namesOfAnAttemptIn({
        runsFolder,
        name: record.name,
        claim,
      },);
      if (start === 'the-machine-after-its-unit') {
        // The manager made the unit while the command that asked for it was killed at its own bound, so the evidence of a start is there
        // and the answer is still the machine's failure.
        await writeFile(names.outputFile, 'the unit started\n',);
        return {
          started: false,
          failure: {
            kind: 'the-machine',
            reason: 'systemd-run was killed after 30 s',
            output: 'systemd-run was killed after 30 s',
          },
        };
      }
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

await describe({
  name: '',
  children: [
    it({
      name: 'puts a review back with an identity, so the next pass starts it instead of throwing on every round',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('other',),],);
        /**
         The review run's attempt folder and its summary, which is what the put-back reads.
         */
        const attempt = join(queue.runsFolder, 'review-change-1',);
        await mkdir(attempt, { recursive: true, },);
        await writeFile(
          join(attempt, 'summary.json',),
          `${JSON.stringify({
            endReason: 'model-error',
            lastAssistantText: '429 insufficient_quota',
          },)}\n`,
        );
        /**
         What putting the ended reviews back answered.
         */
        const putBack = await putEndedReviewsBack({
          queueFolder: queue.queueFolder,
          runsFolder: queue.runsFolder,
          reviewsFolder: join(queue.path, 'reviews',),
          since: 0,
          going: new Set(),
          now: Date.now,
        },);
        expect(putBack.putBack,).toEqual(['review-change',],);
        /**
         The review as the queue holds it.
         */
        const review = await readTheEntryNamed({
          queueFolder: queue.queueFolder,
          name: 'review-change',
        },);
        // A put mints the identity every claim matches on; a put-back that wrote the entry without one made every pass throw.
        expect(typeof (review as QueueRecord).queueId,).toBe('string',);
        /**
         What one pass answered.
         */
        const started: string[] = [];
        await fillFreePlacesFromAQueue({
          request: {
            queueFolder: queue.queueFolder,
            runsFolder: queue.runsFolder,
            level: 4,
            startsAtMost: 4,
            staggerMilliseconds: 1,
          },
          seams: {
            ...seamsOf({
              queueFolder: queue.queueFolder,
              runsFolder: queue.runsFolder,
              start: 'started',
            },),
            start({ record, claim, },) {
              started.push(record.name,);
              /**
               The names this attempt's unit would write.
               */
              const names = namesOfAnAttemptIn({
                runsFolder: queue.runsFolder,
                name: record.name,
                claim,
              },);
              return Promise.resolve({
                started: true as const,
                lines: [],
                batchFile: names.batchFile,
                unit: `launch-${record.name}`,
                outputFile: names.outputFile,
              },);
            },
          },
        },);
        expect(started,).toEqual(['review-change', 'other',],);
        expect(await namesIn(queue.queueFolder,),).toEqual([],);
      },
    },),
    it({
      name: 'reports one bad file by name and starts the entry behind it, so one file never stops the queue',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',),],);
        // An empty file is what a power cut leaves of a write that was never finished, and a foreign file is what a person leaves in the
        // folder: neither may stop the entry that is queued.
        await writeFile(join(queue.queueFolder, 'an-empty-entry.json',), '',);
        await writeFile(join(queue.queueFolder, 'notes.txt',), 'a person\'s notes\n',);
        /**
         The lines the reader said.
         */
        const said: string[] = [];
        /**
         The entries the reader took.
         */
        const entries = await readTheQueueReportingProblems({
          queueFolder: queue.queueFolder,
          report: function report(line,) {
            said.push(line,);
          },
        },);
        expect(entries.map(function nameOf(entry,) {
          return entry.name;
        },),).toEqual(['a',],);
        expect(said.join('',),).toContain('an-empty-entry.json',);
        expect(said.join('',),).toContain('notes.txt',);
        // The pass starts the good entry and says each file it passed over.
        /**
         The lines the pass said.
         */
        const passLines: string[] = [];
        /**
         The names the pass started.
         */
        const started: string[] = [];
        await fillFreePlacesFromAQueue({
          request: {
            queueFolder: queue.queueFolder,
            runsFolder: queue.runsFolder,
            level: 2,
            startsAtMost: 2,
            staggerMilliseconds: 1,
          },
          seams: {
            ...seamsOf({
              queueFolder: queue.queueFolder,
              runsFolder: queue.runsFolder,
              start: 'started',
            },),
            report: function report(line,) {
              passLines.push(line,);
            },
            start({ record, claim, },) {
              started.push(record.name,);
              /**
               The names this attempt's unit would write.
               */
              const names = namesOfAnAttemptIn({
                runsFolder: queue.runsFolder,
                name: record.name,
                claim,
              },);
              return Promise.resolve({
                started: true as const,
                lines: [],
                batchFile: names.batchFile,
                unit: `launch-${record.name}`,
                outputFile: names.outputFile,
              },);
            },
          },
        },);
        expect(started,).toEqual(['a',],);
        expect(passLines.join('',),).toContain('notes.txt',);
        expect(passLines.join('',),).toContain('an-empty-entry.json',);
      },
    },),
    it({
      name: 'refuses a move while a filling holds the claim, so an entry a filling started is never written back into the queue',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',), runNamed('b',),],);
        /**
         The entry as the queue holds it.
         */
        const held = (await readQueueOfRuns({ queueFolder: queue.queueFolder, },))[0] as QueueRecord;
        // A filling claims the entry; the move then reads the queue and tries to write the entry's place, which is the write that once put a
        // started entry back.
        /**
         What taking a claim on the entry answered, standing for the filling.
         */
        const claimed = await claimAnEntryOfAQueue({
          queueFolder: queue.queueFolder,
          record: held,
          now: Date.now,
        },);
        expect(claimed.kind,).toBe('claimed',);
        /**
         What the move answered.
         */
        const moved = await moveAQueueEntryTo({
          queueFolder: queue.queueFolder,
          name: 'a',
          position: 1,
        },);
        expect(moved.from,).toBeUndefined();
        expect(moved.to,).toBeUndefined();
        /**
         The entry as it stands, whose place the move must not have changed.
         */
        const after = await readTheEntryNamed({
          queueFolder: queue.queueFolder,
          name: 'a',
        },);
        expect(orderOf({ record: after as QueueRecord, },),).toBe(orderOf({ record: held, },),);
        // And the move writes it once the claim is gone, which is what says the refusal is about the claim and not about moving at all.
        await aClaimGivenBackOnlyIfOurs({
          queueFolder: queue.queueFolder,
          name: 'a',
          token: claimed.kind === 'claimed' ? claimed.claim.token : '',
        },);
        expect((await moveAQueueEntryTo({
          queueFolder: queue.queueFolder,
          name: 'a',
          position: 1,
        },)).to,).toBe(1,);
      },
    },),
    it({
      name: 'leaves a live filling\'s claim alone when a settling pass that decided about an earlier one removes its claim',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',),],);
        /**
         The entry as the queue holds it.
         */
        const held = (await readQueueOfRuns({ queueFolder: queue.queueFolder, },))[0] as QueueRecord;
        /**
         The identity the queue holds for it.
         */
        const queueId = String(held.queueId,);
        // A dead filling's claim is what a settling pass decided about.
        await mkdir(
          join(queue.queueFolder, 'claimed',),
          { recursive: true, },
        );
        await writeFile(
          claimFileOf({
            queueFolder: queue.queueFolder,
            name: 'a',
          },),
          `${JSON.stringify(aClaimOf({
            name: 'a',
            queueId,
            claim: aClaimOfAFillingThatIsGone({},),
          },),)}\n`,
        );
        /**
         The claim the dying filling held, as a pass reads it.
         */
        const dead = (await readClaims(queue.queueFolder,))[0] as {
          readonly queueId: string;
          readonly claim: QueueClaim;
        };
        // Another pass clears the dead claim, and a second filling then claims the entry while the settling pass still has the dead claim
        // in front of it: that is the sequence a race between two settling passes makes.
        expect(await aClaimGivenBackOnlyIfOurs({
          queueFolder: queue.queueFolder,
          name: 'a',
          token: dead.claim.token,
        },),).toBe(true,);
        /**
         What the second filling was answered.
         */
        const second = await claimAnEntryOfAQueue({
          queueFolder: queue.queueFolder,
          record: held,
          now: Date.now,
        },);
        expect(second.kind,).toBe('claimed',);
        // The settling pass removes only the claim it decided about, which is the dead one: the live filling keeps its own.
        /**
         Whether the removal took a claim.
         */
        const removed = await aClaimRemovedOnlyIfItSays({
          queueFolder: queue.queueFolder,
          name: 'a',
          holds: function isTheDeadOne(claim,) {
            return claim.claim.token === dead.claim.token;
          },
        },);
        expect(removed,).toBe(false,);
        /**
         The claims the queue holds now, which is the live filling's own.
         */
        const left = await readClaims(queue.queueFolder,);
        expect(left.map(function tokenOf(claim,) {
          return claim.claim.token;
        },),).toEqual([second.kind === 'claimed' ? second.claim.token : '',],);
      },
    },),
    it({
      name: 'tells a claim whose process number was given to another process from a holder that is there, by the moment the process started',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',),],);
        /**
         The moment this test's own process started.
         */
        const started = await startedAtOf({ pid: process.pid, },);
        expect(started.kind,).toBe('there',);
        /**
         The moment the claim says the holder started.
         */
        const theRealMoment = started.kind === 'there' ? started.startedAt : '';
        /**
         A claim naming this live process at another moment, which is what a number reused after its holder died looks like.
         */
        const reused = await theHolderOfAClaim({
          claim: aClaimOf({
            name: 'a',
            queueId: 'q1',
            claim: aClaimOfAFillingThatIsGone({
              pid: process.pid,
              startedAt: '0',
            },),
          },),
        },);
        expect(reused.kind,).toBe('another-process',);
        /**
         A claim naming this live process at its own moment, which is a filling that is still starting its entry.
         */
        const mine = await theHolderOfAClaim({
          claim: aClaimOf({
            name: 'a',
            queueId: 'q1',
            claim: aClaimOfAFillingThatIsGone({
              pid: process.pid,
              startedAt: theRealMoment,
            },),
          },),
        },);
        expect(mine.kind,).toBe('there',);
        /**
         A claim naming no process of this account at all.
         */
        const gone = await theHolderOfAClaim({
          claim: aClaimOf({
            name: 'a',
            queueId: 'q1',
            claim: aClaimOfAFillingThatIsGone({},),
          },),
        },);
        expect(gone.kind,).toBe('gone',);
      },
    },),
    it({
      name: 'takes an entry as started when its unit exists, though its start answered a failure of the machine',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',),],);
        /**
         The units the seam made, which is one per start attempt.
         */
        const started: string[] = [];
        /**
         What the one pass answered.
         */
        const outcome = await fillFreePlacesFromAQueue({
          request: {
            queueFolder: queue.queueFolder,
            runsFolder: queue.runsFolder,
            level: 2,
            startsAtMost: 2,
            staggerMilliseconds: 1,
          },
          seams: {
            ...seamsOf({
              queueFolder: queue.queueFolder,
              runsFolder: queue.runsFolder,
              start: 'the-machine-after-its-unit',
            },),
            start({ record, claim, },) {
              started.push(record.name,);
              /**
               The names this attempt's unit would write.
               */
              const names = namesOfAnAttemptIn({
                runsFolder: queue.runsFolder,
                name: record.name,
                claim,
              },);
              return seamsOf({
                queueFolder: queue.queueFolder,
                runsFolder: queue.runsFolder,
                start: 'the-machine-after-its-unit',
              },).start({
                record,
                claim,
              },).then(async function asTheMachine(answer,) {
                void names;
                return answer;
              },);
            },
          },
        },);
        // The unit exists, so the entry has started: it is out of the queue, one unit was made for it, and no second attempt follows.
        expect(started,).toEqual(['a',],);
        expect(outcome.started,).toEqual(['a',],);
        expect(await namesIn(queue.queueFolder,),).toEqual([],);
        expect(outcome.stoppedBecause,).toContain('the machine said no',);
      },
    },),
    it({
      name: 'keeps an earlier entry\'s record when a later entry of the same name is set aside',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',),],);
        /**
         The first version of the entry, which is the one whose prompt exists nowhere else.
         */
        const first = (await readQueueOfRuns({ queueFolder: queue.queueFolder, },))[0] as QueueRecord;
        await setAsideAnEntryOfAQueue({
          queueFolder: queue.queueFolder,
          runsFolder: queue.runsFolder,
          record: first,
          queueId: String(first.queueId,),
          reason: 'the first version failed its start',
          now: Date.now,
        },);
        // A person queues other work under that name.
        await putAQueueEntryIn({
          queueFolder: queue.queueFolder,
          entry: {
            name: 'a',
            branch: 'feat/second-version',
            prompt: '/runs/second-prompt.md',
            wall: 28_800,
          },
          position: 0,
        },);
        /**
         The second version of the entry.
         */
        const second = (await readQueueOfRuns({ queueFolder: queue.queueFolder, },))[0] as QueueRecord;
        /**
         What setting the second version aside answered.
         */
        const written = await writeSetAsideRecord({
          runsFolder: queue.runsFolder,
          record: {
            name: 'a',
            entry: {
              ...second,
              queueId: String(second.queueId,),
            },
            reason: 'the second version failed its start',
            setAsideAt: new Date().toISOString(),
          },
        },);
        expect(written.keptTheEarlierRecordAt,).toBeDefined();
        /**
         What the kept record of the first version holds.
         */
        const kept = JSON.parse(await readFile(written.keptTheEarlierRecordAt ?? '', 'utf8',),) as {
          readonly entry: QueueRecord;
        };
        expect(kept.entry.branch,).toBe('feat/a',);
        expect(kept.entry.prompt,).toBe('/runs/a-prompt.md',);
        // And the record under the plain name is the second version's, which is the entry that is queued now.
        expect((await setAsideRecordOf({ runsFolder: queue.runsFolder, name: 'a', },) as { readonly entry: QueueRecord }).entry.branch,)
          .toBe('feat/second-version',);
      },
    },),
    it({
      name: 'refuses to take an entry a filling is starting, and leaves the entry in the queue until the person has been shown it',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',),],);
        /**
         The entry as the queue holds it.
         */
        const held = (await readQueueOfRuns({ queueFolder: queue.queueFolder, },))[0] as QueueRecord;
        /**
         What taking a claim on it answered, standing for a filling that is starting it.
         */
        const claimed = await claimAnEntryOfAQueue({
          queueFolder: queue.queueFolder,
          record: held,
          now: Date.now,
        },);
        expect(claimed.kind,).toBe('claimed',);
        /**
         What the take threw while the filling held the claim.
         */
        let caught: unknown;
        try {
          await takeAQueueEntryOut({
            queueFolder: queue.queueFolder,
            name: 'a',
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(String(caught,),).toContain('is being started by the filling of process',);
        // With the filling gone, a person takes it anyway, and the entry stands in the file the take left aside until it is discarded.
        /**
         What the forced take answered.
         */
        const taken = await takeAQueueEntryOut({
          queueFolder: queue.queueFolder,
          name: 'a',
          force: true,
        },);
        expect(taken.taken.map(function nameOf(entry,) {
          return entry.name;
        },),).toEqual(['a',],);
        expect(taken.asides,).toHaveLength(1,);
        /**
         What the file the take left aside holds.
         */
        const aside = JSON.parse(await readFile(taken.asides[0] ?? '', 'utf8',),) as QueueRecord;
        expect(aside.name,).toBe('a',);
        await discardTheEntryTaken({ aside: taken.asides[0] ?? '', },);
        expect(await readdir(queue.queueFolder,),).not.toContain(taken.asides[0]);
      },
    },),
    it({
      name: 'removes the files a killed command left on its way to a name, once they are too old for any step still running',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',),],);
        await mkdir(join(queue.queueFolder, 'claimed',), { recursive: true, },);
        /**
         The files a killed command left, one in the queue's folder and one in the claims folder.
         */
        const left = [
          join(queue.queueFolder, '.being-written-a.json-1-2',),
          join(queue.queueFolder, 'claimed', '.being-written-a.claim.json-1-3',),
        ];
        for (const path of left) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- one file of this test's own fixture after another
          await writeFile(path, '{"name":"a"}\n',);
        }
        /**
         The lines the sweep said.
         */
        const said: string[] = [];
        expect(await halfWrittenFilesSwept({
          queueFolder: queue.queueFolder,
          report: function report(line,) {
            said.push(line,);
          },
          now: function nowLater() {
            return Date.now() + HALF_WRITTEN_FILE_AGE_MILLISECONDS + 1_000;
          },
        },),).toBe(2,);
        expect(await readdir(queue.queueFolder,),).not.toContain('.being-written-a.json-1-2');
        expect(said.join('',),).toContain('never finished',);
        // A file written a moment ago is left alone, because a command may still be writing it.
        await writeFile(join(queue.queueFolder, '.being-written-b.json-1-4',), '{}\n',);
        expect(await halfWrittenFilesSwept({
          queueFolder: queue.queueFolder,
          report: function report() {
            return undefined;
          },
        },),).toBe(0,);
      },
    },),
    it({
      name: 'carries the array over once, and passes over an entry an earlier unfinished run already wrote, so no entry is started twice',
      fn: async () => {
        await using queue = await aQueueOf([],);
        /**
         The array the old scripts wrote.
         */
        const queueFile = join(queue.path, 'queue.json',);
        await writeFile(
          queueFile,
          `${JSON.stringify([runNamed('a',), runNamed('b',),],)}\n`,
        );
        /**
         What the first changeover did.
         */
        const first = await convertAQueueFileIntoAFolder({
          queueFile,
          queueFolder: queue.queueFolder,
          now: function atAStatedMoment() {
            return Date.parse('2026-11-16T12:00:00.000Z',);
          },
        },);
        expect(first.entries,).toBe(2,);
        expect(await namesIn(queue.queueFolder,),).toEqual(['a', 'b',],);
        expect(first.arrayKeptAt,).toContain('.converted-2026-11-16T120000.000Z',);
        // A second changeover of another array keeps its own copy, and never writes over the first.
        await writeFile(
          queueFile,
          `${JSON.stringify([runNamed('c',),],)}\n`,
        );
        /**
         What the second changeover did.
         */
        const second = await convertAQueueFileIntoAFolder({
          queueFile,
          queueFolder: queue.queueFolder,
          now: function aMomentLater() {
            return Date.parse('2026-11-16T12:05:00.000Z',);
          },
        },);
        expect(second.arrayKeptAt,).not.toBe(first.arrayKeptAt,);
        expect(
          JSON.parse(await readFile(first.arrayKeptAt, 'utf8',),),
        ).toHaveLength(2,);
        expect(
          JSON.parse(await readFile(second.arrayKeptAt, 'utf8',),),
        ).toHaveLength(1,);
        // A run that was killed after writing `d` and before the end of the changeover is finished by the next run, which never writes
        // the name it already recorded.
        await using secondQueue = await aQueueOf([],);
        /**
         The array of the run that was killed.
         */
        const killedFile = join(secondQueue.path, 'queue.json',);
        await writeFile(
          killedFile,
          `${JSON.stringify([runNamed('d',), runNamed('e',),],)}\n`,
        );
        await rm(killedFile,);
        await writeFile(`${killedFile}${BEING_CARRIED_ENDING}`, `${JSON.stringify([runNamed('d',), runNamed('e',),],)}\n`,);
        await writeFile(`${killedFile}${CARRIED_ENDING}`, 'd\n',);
        // The filling started `d` before the changeover was finished, so it is out of the folder and must never be queued again.
        /**
         What the finished changeover did.
         */
        const finished = await convertAQueueFileIntoAFolder({
          queueFile: killedFile,
          queueFolder: secondQueue.queueFolder,
        },);
        expect(finished.entries,).toBe(1,);
        expect(await namesIn(secondQueue.queueFolder,),).toEqual(['e',],);
      },
    },),
  ],
},);

/**
 Reads the claims one queue holds.

 @param queueFolder - the queue's folder

 @returns the claims

 @example
 ```ts
 const claims = await readClaims('/runs/q');
 ```
 */
async function readClaims(queueFolder: string,): Promise<readonly { readonly name: string; readonly queueId: string; readonly claim: QueueClaim }[]> {
  return await readTheClaimsOnAQueue({ queueFolder, },);
}
