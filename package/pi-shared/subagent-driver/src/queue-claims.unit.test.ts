/**
 The six defects the third review of the queue of runs reproduced, each as a control that fails before its fix and passes after it: one
 entry handed to two batch units (two fillings at once, and a filling killed between its start and its removal), a completed start that
 removes a replacement queued under the same name, a lock taken from a live holder for its age alone, a reused name that makes the
 settlement delete a valid set-aside, a review that starts during a put-back and is queued again, and a newest attempt killed before its
 summary that blocks the put-back without end.

 Each control holds its two actors at a step of the code's own: a filling's `start` seam while another whole pass runs, a queue's lock
 taken by this test while a real writer asks for it, a `going` set whose one read is held while the queue's lock changes hands, and a
 record whose identity is the entry's own. No control waits on a sleep.

 The review's own scripts are beside it in the review folder, and each reproduces its defect against the commit this file answers.
 Every queue file, record and fixture here lives in a throwaway folder.

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
  BUILD_VOLUME,
  type FillingSeams,
  type FreeSpace,
  type QueueClaim,
  type QueueRecord,
  QueueLockHeldError,
  claimAnEntryOfAQueue,
  fillFreePlacesFromAQueue,
  lockFileOfAQueue,
  namesOfAnAttemptIn,
  processStartedAtOf,
  putAQueueEntryIn,
  putASetAsideEntryBack,
  putEndedReviewsBack,
  queueIdOf,
  readQueueOfRuns,
  setAsideAnEntryOfAQueue,
  setAsideRecordIsThere,
  setAsideRecordOf,
  settleTheQueueAfterAKill,
  takeAQueueEntryOut,
  withTheQueueLock,
  writeQueueOfRuns,
  writeSetAsideRecord,
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
   The folder the runs and the set-aside records live in.
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

 @param entries - the entries the queue is written with, as the queue writes them

 @returns the folder
 */
async function makeFolder(entries: readonly QueueRecord[],): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-queue-claims-',),);
  /**
   The folder the runs and the records live in.
   */
  const runs = join(path, 'runs',);
  await mkdir(runs, { recursive: true, },);
  /**
   The queue's path.
   */
  const queueFile = join(path, 'queue.json',);
  await writeQueueOfRuns({
    queueFile,
    entries,
  },);
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
 The room the build volume has, which every pass here says is ample unless its own subject is the floor.
 */
const AMPLE_ROOM: FreeSpace = {
  volume: BUILD_VOLUME,
  freeBytes: 900 * 1_024 * 1_024 * 1_024,
};

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

 @param queueFile - the queue's path

 @returns the names

 @example
 ```ts
 const names = await namesIn('/runs/q.json');
 ```
 */
async function namesIn(queueFile: string,): Promise<readonly string[]> {
  return (await readQueueOfRuns({ queueFile, },)).map(function nameOf(entry,) {
    return entry.name;
  },);
}

/**
 The seams a pass asks, whose start and set-aside the test answers and whose set-aside is the package's own.

 @param request - the folder the pass fills, the start the test answers, and the lines and starts the test keeps

 @returns the seams

 @example
 ```ts
 const seams = seamsWith({ folder, start: async () => started });
 ```
 */
function seamsWith({
  folder,
  start,
}: {
  readonly folder: AThrowawayFolder;
  readonly start: FillingSeams['start'];
},): FillingSeams {
  return {
    runsGoing: function runsGoing() {
      return Promise.resolve([],);
    },
    freeSpace: function freeSpaceNow() {
      return AMPLE_ROOM;
    },
    start,
    setAside: function setAside({
      record,
      failure,
      claim,
    },) {
      return setAsideAnEntryOfAQueue({
        queueFile: folder.queueFile,
        runsFolder: folder.runs,
        record,
        claim,
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
 The answer one start gives, naming the attempt it was asked for.

 @param folder - the folder the attempt's files are named in

 @param record - the entry the start was asked to start

 @param claim - the claim naming the attempt

 @returns what the start answers

 @example
 ```ts
 const started = startedAnswerOf({ folder, record, claim });
 ```
 */
function startedAnswerOf({
  folder,
  record,
  claim,
}: {
  readonly folder: AThrowawayFolder;
  readonly record: QueueRecord;
  readonly claim: QueueClaim;
},) {
  /**
   The names the attempt's unit is given.
   */
  const names = namesOfAnAttemptIn({
    runsFolder: folder.runs,
    name: record.name,
    claim,
  },);
  return {
    started: true as const,
    lines: [],
    batchFile: names.batchFile,
    unit: `launch-${record.name}`,
    outputFile: names.outputFile,
  };
}

/**
 The process number of a process that has ended, which its own end frees for another.

 @returns the number of a process that is gone

 @example
 ```ts
 const gonePid = await pidOfAProcessThatHasEnded();
 ```
 */
async function pidOfAProcessThatHasEnded(): Promise<number> {
  /**
   The process that ends at once.
   */
  const child = spawn(
    'true',
    [],
    { stdio: 'ignore', },
  );
  /**
   Its process number, which its own end frees.
   */
  const pid = child.pid ?? 0;
  await once(
    child,
    'close',
  );
  return pid;
}

await describe({
  name: '',
  children: [
    describe({
      name: 'one entry, two batch units',
      children: [
        it({
          name: 'is asked for once when two passes run at once, the second passing over the entry the first is starting',
          fn: async () => {
            await using folder = await makeFolder([runNamed('one-entry',),],);
            /**
             The starts the two passes asked for, in the order they asked.
             */
            const starts: string[] = [];
            /**
             Resolves once the first pass is inside its start, holding its claimed entry.
             */
            const firstPassIsStarting = Promise.withResolvers<void>();
            /**
             Resolves once the second pass has run to its end.
             */
            const secondPassHasRun = Promise.withResolvers<void>();
            /**
             The first pass, held inside the start of the entry it claimed.
             */
            const firstPass = fillFreePlacesFromAQueue({
              request: {
                queueFile: folder.queueFile,
                runsFolder: folder.runs,
                level: 1,
                startsAtMost: 1,
                lockWaitMilliseconds: 1_000,
              },
              seams: seamsWith({
                folder,
                start: async function startTheFirst({ record, claim, },) {
                  starts.push('first',);
                  firstPassIsStarting.resolve();
                  await secondPassHasRun.promise;
                  return startedAnswerOf({
                    folder,
                    record,
                    claim,
                  },);
                },
              },),
            },);
            await firstPassIsStarting.promise;
            /**
             What the second pass answered while the first held its claim.
             */
            const secondPass = await fillFreePlacesFromAQueue({
              request: {
                queueFile: folder.queueFile,
                runsFolder: folder.runs,
                level: 1,
                startsAtMost: 1,
                lockWaitMilliseconds: 1_000,
              },
              seams: seamsWith({
                folder,
                start: async function startTheSecond({ record, claim, },) {
                  starts.push('second',);
                  return startedAnswerOf({
                    folder,
                    record,
                    claim,
                  },);
                },
              },),
            },);
            secondPassHasRun.resolve();
            /**
             What the first pass answered once it finished.
             */
            const first = await firstPass;
            expect(starts,).toEqual(['first',],);
            expect(first.started,).toEqual(['one-entry',],);
            expect(secondPass.started,).toEqual([],);
            expect(await namesIn(folder.queueFile,),).toEqual([],);
          },
        },),
        it({
          name: 'is not started again after the filling that claimed it died with its start taken effect, and is started again when that start never took effect',
          fn: async () => {
            await using folder = await makeFolder([],);
            /**
             The process number of a filling that is gone.
             */
            const gonePid = await pidOfAProcessThatHasEnded();
            /**
             The claim the killed filling held, whose token names its attempt.
             */
            const claim: QueueClaim = {
              token: 'the-attempt-of-a-filling-that-was-killed',
              pid: gonePid,
              at: '2026-11-15T09:00:00.000-05:00',
            };
            /**
             The entry's own identity, minted by that claim.
             */
            const queueId = 'the-entry-of-a-filling-that-was-killed';
            await writeQueueOfRuns({
              queueFile: folder.queueFile,
              entries: [
                {
                  ...runNamed('started-before-the-kill',),
                  queueId,
                  queueClaim: claim,
                },
                {
                  ...runNamed('never-started',),
                  queueId: 'the-entry-that-never-started',
                  queueClaim: {
                    ...claim,
                    token: 'the-attempt-that-never-started',
                  },
                },
              ],
            },);
            // The killed filling got as far as its unit: its unit's output file is what says so, and it is the file the claim's own
            // token names.
            /**
             The names of the attempt that took effect.
             */
            const names = namesOfAnAttemptIn({
              runsFolder: folder.runs,
              name: 'started-before-the-kill',
              claim,
            },);
            await writeFile(names.outputFile, 'the batch started before the kill\n',);
            /**
             What the one pass answered.
             */
            const pass = await fillFreePlacesFromAQueue({
              request: {
                queueFile: folder.queueFile,
                runsFolder: folder.runs,
                level: 3,
                startsAtMost: 3,
                lockWaitMilliseconds: 1_000,
              },
              seams: seamsWith({
                folder,
                start: async function startTheSecond({ record, claim: ofThisStart, },) {
                  return startedAnswerOf({
                    folder,
                    record,
                    claim: ofThisStart,
                  },);
                },
              },),
            },);
            // The entry whose start took effect is out of the queue and never started again; the one whose start never took effect
            // had its claim given back and was started.
            expect(pass.started,).toEqual(['never-started',],);
            expect(await namesIn(folder.queueFile,),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: 'a completed start, and a replacement queued under the same name',
      children: [
        it({
          name: 'leaves the replacement where it was queued, because the removal is matched on the entry it started',
          fn: async () => {
            await using folder = await makeFolder([runNamed('same-name',),],);
            /**
             The entry the queue was given first, and the one queued under its name while its start was in flight.
             */
            const replacement: QueueRecord = {
              name: 'same-name',
              branch: 'feat/replacement',
              prompt: 'replacement user data',
              wall: 28_800,
            };
            /**
             Resolves once the first pass is inside its start.
             */
            const starting = Promise.withResolvers<void>();
            /**
             Resolves once the replacement is queued.
             */
            const replacementIsQueued = Promise.withResolvers<void>();
            /**
             The pass, held inside the start of the entry it claimed.
             */
            const filling = fillFreePlacesFromAQueue({
              request: {
                queueFile: folder.queueFile,
                runsFolder: folder.runs,
                level: 1,
                startsAtMost: 1,
                lockWaitMilliseconds: 1_000,
              },
              seams: seamsWith({
                folder,
                start: async function startIt({ record, claim, },) {
                  starting.resolve();
                  await replacementIsQueued.promise;
                  return startedAnswerOf({
                    folder,
                    record,
                    claim,
                  },);
                },
              },),
            },);
            await starting.promise;
            // The entry in flight is taken out by name, which a person asking for that name may do, and a replacement is queued.
            await takeAQueueEntryOut({
              queueFile: folder.queueFile,
              name: 'same-name',
            },);
            await putAQueueEntryIn({
              queueFile: folder.queueFile,
              entry: replacement,
              position: 0,
            },);
            replacementIsQueued.resolve();
            /**
             What the pass answered.
             */
            const outcome = await filling;
            expect(outcome.started,).toEqual(['same-name',],);
            /**
             The queue after the start was recorded.
             */
            const queue = await readQueueOfRuns({ queueFile: folder.queueFile, },);
            expect(queue,).toHaveLength(1,);
            expect(queue[0],).toEqual(replacement,);
          },
        },),
      ],
    },),
    describe({
      name: 'a lock whose holder is alive',
      children: [
        it({
          name: 'is not taken from it for its age, so a writer that waited writes over nothing and the holder\'s own change stands',
          fn: async () => {
            await using folder = await makeFolder([runNamed('base',),],);
            /**
             The entry the holder of the lock adds when it is let go.
             */
            const fromTheHolder: QueueRecord = {
              name: 'from-the-holder',
              branch: 'feat/holder',
              wall: 28_800,
            };
            /**
             Resolves once the holder has read the queue under its lock.
             */
            const holderHasRead = Promise.withResolvers<void>();
            /**
             Resolves when the holder may write.
             */
            const holderMayWrite = Promise.withResolvers<void>();
            /**
             The holder, held inside its step with the queue read and its change not yet written.
             */
            const holder = withTheQueueLock({
              queueFile: folder.queueFile,
              work: async function holdTheQueue({
                entries,
                write,
              },) {
                holderHasRead.resolve();
                await holderMayWrite.promise;
                await write([...entries, fromTheHolder,],);
              },
            },);
            await holderHasRead.promise;
            // The lock is made older than any change could be, which is what the code before this control broke it for.
            /**
             The lock's older timestamp.
             */
            const stale = new Date(Date.now() - (24 * 60 * 60 * 1_000),);
            await utimes(lockFileOfAQueue({ queueFile: folder.queueFile, },), stale, stale,);
            /**
             What a second writer answered while the holder was alive and inside its step.
             */
            let caught: unknown;
            try {
              await settleTheQueueAfterAKill({
                queueFile: folder.queueFile,
                runsFolder: folder.runs,
                report: function report() {
                  return undefined;
                },
                waitMilliseconds: 50,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(QueueLockHeldError,);
            expect(String(caught,),).toContain(`process ${String(process.pid,)}`,);
            holderMayWrite.resolve();
            await holder;
            expect(await namesIn(folder.queueFile,),).toEqual(['base', 'from-the-holder',],);
          },
        },),
        it({
          name: 'is broken when the number it names belongs to another process than the one it recorded, which is a number reused after its holder died',
          fn: async () => {
            await using folder = await makeFolder([runNamed('base',),],);
            // A lock naming this live process, with a start time this process does not have: the number is this test's, and the
            // process it named is gone.
            await writeFile(
              lockFileOfAQueue({ queueFile: folder.queueFile, },),
              `${JSON.stringify({
                pid: process.pid,
                startedAt: '0',
                at: '2026-11-15T09:00:00.000-05:00',
                token: 'a-token-of-a-process-that-is-gone',
              },)}\n`,
            );
            /**
             When this process started, which the lock does not name.
             */
            const startedAt = await processStartedAtOf({ pid: process.pid, },);
            expect(startedAt === '0',).toBe(false,);
            await settleTheQueueAfterAKill({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              report: function report() {
                return undefined;
              },
              waitMilliseconds: 200,
            },);
            expect(await namesIn(folder.queueFile,),).toEqual(['base',],);
            /**
             Whether the lock of the process that is gone is still there.
             */
            const leftBehind = await (async function isTheLockStillThere() {
              try {
                await readFile(lockFileOfAQueue({ queueFile: folder.queueFile, },), 'utf8',);
                return true;
              }
              catch (error) {
                // A lock that is not there is what a broken lock leaves, which is the answer this asks for.
                void error;
                return false;
              }
            })();
            expect(leftBehind,).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'a name queued again while a set-aside record holds the earlier entry',
      children: [
        it({
          name: 'keeps the record, which is the earlier entry\'s only copy, and says so rather than deleting it',
          fn: async () => {
            await using folder = await makeFolder([runNamed('same-name',),],);
            /**
             The claim a filling held on the earlier entry when its start failed.
             */
            const taken = await claimAnEntryOfAQueue({
              queueFile: folder.queueFile,
              record: runNamed('same-name',),
              now: Date.now,
            },);
            if (taken.kind !== 'claimed') {
              throw new Error(`the queue did not hold the entry to claim: ${taken.kind}`,);
            }
            await setAsideAnEntryOfAQueue({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              record: runNamed('same-name',),
              claim: {
                queueId: taken.queueId,
                claim: taken.claim,
              },
              reason: 'the earlier entry failed its start',
              now: Date.now,
            },);
            // A person queues other work under that name, which the queue accepts because it holds no such entry any more.
            await putAQueueEntryIn({
              queueFile: folder.queueFile,
              entry: {
                name: 'same-name',
                branch: 'feat/replacement',
                prompt: 'new work queued under the same name',
                wall: 28_800,
              },
              position: 0,
            },);
            /**
             The lines the settlement said.
             */
            const said: string[] = [];
            await settleTheQueueAfterAKill({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              report: function report(line,) {
                said.push(line,);
              },
              waitMilliseconds: 1_000,
            },);
            expect(await setAsideRecordIsThere({ runsFolder: folder.runs, name: 'same-name', },),).toBe(true,);
            expect(said.join('',),).toContain('so it is left as it is',);
            expect(await namesIn(folder.queueFile,),).toEqual(['same-name',],);
            /**
             What the record still holds: the earlier entry whole, with its branch and its prompt.
             */
            const { entry: held, } = await setAsideRecordOf({ runsFolder: folder.runs, name: 'same-name', },) as {
              readonly entry: QueueRecord;
            };
            expect(held,).toMatchObject({
              name: 'same-name',
              branch: 'feat/same-name',
              prompt: '/runs/same-name-prompt.md',
            },);
            expect(queueIdOf({ record: held, },),).toBe(taken.queueId,);
          },
        },),
      ],
    },),
    describe({
      name: 'a review that starts while the ended reviews are being put back',
      children: [
        it({
          name: 'is not queued again, because the runs going are read again inside the step that inserts',
          fn: async () => {
            await using folder = await makeFolder([],);
            /**
             The runs folder and the reviews folder the put-back reads.
             */
            const runs = join(folder.path, 'runs-with-reviews',);
            /**
             The folder one folder per review goes in.
             */
            const reviews = join(folder.path, 'reviews',);
            await mkdir(join(runs, 'review-change-1',), { recursive: true, },);
            await mkdir(reviews, { recursive: true, },);
            await writeFile(
              join(runs, 'review-change-1', 'summary.json',),
              `${JSON.stringify({
                endReason: 'model-error',
                lastAssistantText: '429 insufficient_quota',
              },)}\n`,
            );
            /**
             The set the caller read before the put-back, which does not hold the review yet.
             */
            const goingAtTheStart = new Set<string>();
            /**
             The set the runs going are in once the review has started, which the put-back reads again inside its step.
             */
            const goingNow = new Set(['review-change',],);
            /**
             What putting the ended reviews back answered.
             */
            const putBack = await putEndedReviewsBack({
              queueFile: folder.queueFile,
              runsFolder: runs,
              reviewsFolder: reviews,
              since: 0,
              going: goingAtTheStart,
              goingNow: function goingAgain() {
                return Promise.resolve(goingNow,);
              },
              now: Date.now,
              waitMilliseconds: 1_000,
            },);
            expect(putBack.putBack,).toEqual([],);
            expect(await namesIn(folder.queueFile,),).toEqual([],);
            expect(putBack.lines.join('',),).toContain('its run started while the ended reviews were read',);
          },
        },),
      ],
    },),
    describe({
      name: 'the newest attempt of a review, killed before it wrote a summary',
      children: [
        it({
          name: 'does not block the review coming back, which goes on from the newest attempt that did write a summary',
          fn: async () => {
            await using folder = await makeFolder([],);
            /**
             The runs folder and the reviews folder the put-back reads.
             */
            const runs = join(folder.path, 'runs-with-attempts',);
            /**
             The folder one folder per review goes in.
             */
            const reviews = join(folder.path, 'reviews',);
            await mkdir(join(runs, 'review-change-1',), { recursive: true, },);
            await mkdir(join(runs, 'review-change-2',), { recursive: true, },);
            await mkdir(reviews, { recursive: true, },);
            await writeFile(
              join(runs, 'review-change-1', 'summary.json',),
              `${JSON.stringify({
                endReason: 'model-error',
                lastAssistantText: '429 insufficient_quota',
              },)}\n`,
            );
            /**
             What putting the ended reviews back answered, with nothing of that name going and nothing queued.
             */
            const putBack = await putEndedReviewsBack({
              queueFile: folder.queueFile,
              runsFolder: runs,
              reviewsFolder: reviews,
              since: 0,
              going: new Set(),
              now: Date.now,
              waitMilliseconds: 1_000,
            },);
            expect(putBack.putBack,).toEqual(['review-change',],);
            /**
             The queue the put-back wrote.
             */
            const queue = await readQueueOfRuns({ queueFile: folder.queueFile, },);
            expect(queue,).toHaveLength(1,);
            expect(queue[0],).toMatchObject({
              name: 'review-change',
              sessionOf: 'review-change-1',
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'a set-aside record and its entry',
      children: [
        it({
          name: 'is finished as a set-aside when the record and the claimed entry are both there, and goes back with the identity the record holds',
          fn: async () => {
            await using folder = await makeFolder([runNamed('a',),],);
            /**
             The claim a filling held on the entry when its start failed.
             */
            const taken = await claimAnEntryOfAQueue({
              queueFile: folder.queueFile,
              record: runNamed('a',),
              now: Date.now,
            },);
            if (taken.kind !== 'claimed') {
              throw new Error(`the queue did not hold the entry to claim: ${taken.kind}`,);
            }
            await writeQueueOfRuns({
              queueFile: folder.queueFile,
              entries: [{
                ...runNamed('a',),
                queueId: taken.queueId,
                queueClaim: taken.claim,
              }],
            },);
            await writeSetAsideRecord({
              runsFolder: folder.runs,
              record: {
                name: 'a',
                entry: {
                  ...runNamed('a',),
                  queueId: taken.queueId,
                },
                reason: 'a prompt file that is not there',
                setAsideAt: '2026-11-15T09:00:00.000-05:00',
              },
            },);
            // The set-aside was interrupted: its record holds the entry's identity and the queue holds the entry this claim took, so
            // the settlement finishes it, and the entry stands only in its record.
            await settleTheQueueAfterAKill({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              report: function report() {
                return undefined;
              },
              waitMilliseconds: 1_000,
            },);
            expect(await namesIn(folder.queueFile,),).toEqual([],);
            expect(await setAsideRecordIsThere({ runsFolder: folder.runs, name: 'a', },),).toBe(true,);
            /**
             What putting the entry back answered.
             */
            const put = await putASetAsideEntryBack({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              name: 'a',
              position: 0,
            },);
            expect(put.line,).toContain('put a back at position 0 of 1',);
            expect(queueIdOf({ record: (await readQueueOfRuns({ queueFile: folder.queueFile, },))[0] as QueueRecord, },),).toBe(taken.queueId,);
            expect(await setAsideRecordIsThere({ runsFolder: folder.runs, name: 'a', },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
