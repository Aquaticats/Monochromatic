/**
 The claim of a review's folder: one start owns it at a time, a dead start's place is taken by exactly one taker, and a folder
 whose review unit still runs is kept even when the command that wrote its record has ended.

 The controls here were written before the claim learned about the review's unit and before a dead record's removal became
 exclusive; with the claim as it was, the two-taker control left both starts believing they owned the folder, the unit control
 let a takeover run under a live review, and the unknown control took a folder whose manager had not answered.

 Every folder is one the test made and throws away, and every answer from the manager is handed in by the test, so no unit is
 started on this machine and no manager is asked.

 @module
 */

import { spawn, type ChildProcess, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  type CommandOutput,
  type ReviewFolderOwner,
  ReviewFolderTakenError,
  type RunCommand,
  ownerFileOfAReview,
  processIdentityOfThisProcess,
  takeReviewFolder,
} from '../dist/final/node/index.mjs';

/**
 The change name every control here claims, whose folder and unit are named after it.
 */
const REVIEW = 'shared-change';

/**
 The review unit every record here names, as the review's run name makes it.
 */
const UNIT = `pi-review-${REVIEW}.service`;

/**
 The branch of the dead start, which the live starts here also write for.
 */
const BRANCH = 'fix/shared-change';

/**
 A start that has ended: a number no process here has, on a boot this machine is not on.
 */
function aDeadStart(): {
  readonly pid: number;
  readonly bootId: string;
  readonly startTicks: string;
} {
  return {
    pid: 999_999,
    bootId: 'a-boot-this-machine-is-not-on',
    startTicks: '0',
  };
}

/**
 A folder the test throws away.
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-review-claim-',),);
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 A record a start keeps for itself.

 @param commit - the commit the record is written for

 @param claim - the name of this claim, which a takeover removes and no other record

 @param process - the start's own identity, which a live ticket keeps

 @param unit - the review's unit, which outlives the command that writes the record

 @returns the record

 @example
 ```ts
 const record = aRecord({ commit: 'abc123', claim: 'claim-a', process: aDeadStart(), });
 ```
 */
function aRecord({
  commit,
  claim,
  process: identity,
  unit = UNIT,
}: {
  readonly commit: string;
  readonly claim: string;
  readonly process: ReviewFolderOwner['process'];
  readonly unit?: string;
}): ReviewFolderOwner {
  return {
    branch: BRANCH,
    commit,
    ranges: [`main...${BRANCH}`,],
    source: 'package/a/src',
    claimedAt: '2026-01-01T00:00:00.000Z',
    unit,
    claim,
    process: identity,
  };
}

/**
 Writes a record where a start that ended left it.
 */
async function writeOwner({
  ownerFile,
  record,
}: {
  readonly ownerFile: string;
  readonly record: ReviewFolderOwner;
},): Promise<void> {
  await mkdir(dirname(ownerFile,), { recursive: true, },);
  await writeFile(ownerFile, `${JSON.stringify(record,)}\n`, 'utf8',);
}

/**
 A manager that answers one state for the unit it is asked about.

 @param state - what the manager prints, such as `active` or `inactive`; empty text is a manager that did not answer

 @param asked - where the unit names the manager was asked about go

 @returns how a command is run, as the claim asks the manager

 @example
 ```ts
 const manager = aManagerAnswering({ state: 'inactive', });
 ```
 */
function aManagerAnswering({
  state,
  asked = [],
}: {
  readonly state: string;
  readonly asked?: string[];
},): RunCommand {
  return async function answersAboutTheUnit(request,) {
    asked.push(request.words.at(-1) ?? '',);
    /**
     What the manager prints, which is the state whether or not the unit was found.
     */
    const answered: CommandOutput = {
      status: (state === 'active') ? 0 : 3,
      stdout: `${state}\n`,
      stderr: '',
    };
    return answered;
  };
}

/**
 A manager whose answer follows a stand-in process: the review runs while the process lives and has ended once it is gone.

 @param standIn - the process that stands in for the review's unit

 @returns how a command is run, as the claim asks the manager

 @example
 ```ts
 const manager = aManagerWatching({ standIn: child, });
 ```
 */
function aManagerWatching({
  standIn,
  asked = [],
}: {
  readonly standIn: ChildProcess;
  readonly asked?: string[];
},): RunCommand {
  return async function answersAboutTheStandIn(request,) {
    asked.push(request.words.at(-1) ?? '',);
    /**
     Whether the review the record names still runs.
     */
    const runs = (standIn.exitCode === null) && (standIn.signalCode === null);
    /**
     What the manager prints.
     */
    const answered: CommandOutput = {
      status: runs ? 0 : 3,
      stdout: runs ? 'active\n' : 'inactive\n',
      stderr: '',
    };
    return answered;
  };
}

/**
 Reads the record a folder now holds.
 */
async function theOwnerShown({ ownerFile, }: { readonly ownerFile: string },): Promise<ReviewFolderOwner> {
  return JSON.parse(await readFile(ownerFile, 'utf8',),) as ReviewFolderOwner;
}

await describe({
  name: '',
  children: [
    describe({
      name: takeReviewFolder.name,
      children: [
        it({
          name: 'takes a dead start\'s folder once when two starts look at its record at the same moment, so one start owns the folder and the other is refused',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             Where the dead start left its record.
             */
            const ownerFile = ownerFileOfAReview({ reviewsFolder: folder.path, review: REVIEW, },);
            await writeOwner({
              ownerFile,
              record: aRecord({ commit: 'dead-commit', claim: 'the-dead-claim', process: aDeadStart(), },),
            },);
            /**
             What the manager answers, which is that no review runs.
             */
            const manager = aManagerAnswering({ state: 'inactive', },);
            /**
             The start's own identity, which both starts here share because they are one test process.
             */
            const identity = await processIdentityOfThisProcess();
            /**
             The moment the first start has looked at the dead record and not yet taken its place.
             */
            const looked = Promise.withResolvers<void>();
            /**
             The moment the test lets that start go on.
             */
            const release = Promise.withResolvers<void>();
            /**
             What each start answered.
             */
            const answers: string[] = [];
            /**
             Claims the folder for one commit and keeps what it answered.
             */
            async function take({
              commit,
              claim,
              afterLookingAtADeadOwner,
            }: {
              readonly commit: string;
              readonly claim: string;
              readonly afterLookingAtADeadOwner?: () => Promise<void>;
            },): Promise<void> {
              try {
                await takeReviewFolder({
                  reviewFolder: dirname(ownerFile,),
                  ownerFile,
                  branch: BRANCH,
                  unit: UNIT,
                  manager,
                  owner: aRecord({ commit, claim, process: identity, },),
                  ...((afterLookingAtADeadOwner === undefined) ? {} : { seams: { afterLookingAtADeadOwner, }, }),
                },);
                answers.push(`${claim}: took`,);
              }
              catch (error) {
                answers.push(`${claim}: ${(error instanceof ReviewFolderTakenError) ? 'refused' : String(error,)}`,);
              }
            }
            /**
             The start held between its look and its removal.
             */
            const first = take({
              commit: 'commit-a',
              claim: 'claim-a',
              afterLookingAtADeadOwner: async function waitHere() {
                looked.resolve();
                await release.promise;
              },
            },);
            await looked.promise;
            /**
             The start that runs while the first is held.
             */
            await take({ commit: 'commit-b', claim: 'claim-b', },);
            release.resolve();
            await first;
            expect(answers.filter(function took(answer,) {
              return answer.endsWith(': took',);
            },),).toHaveLength(1,);
            expect(answers.filter(function wasRefused(answer,) {
              return answer.endsWith(': refused',);
            },),).toHaveLength(1,);
            /**
             The record the folder holds.
             */
            const owner = await theOwnerShown({ ownerFile, },);
            expect(['claim-a', 'claim-b',],).toContain(owner.claim,);
            expect(owner.process.pid,).toBe(identity.pid,);
          },
        },),
        it({
          name: 'refuses a second start of one branch while the first start of it still runs, so a live start never loses its folder',
          fn: async () => {
            await using folder = await makeFolder();
            const ownerFile = ownerFileOfAReview({ reviewsFolder: folder.path, review: REVIEW, },);
            await writeOwner({
              ownerFile,
              record: aRecord({ commit: 'commit-a', claim: 'claim-a', process: await processIdentityOfThisProcess(), },),
            },);
            const manager = aManagerAnswering({ state: 'inactive', },);
            await expect(takeReviewFolder({
              reviewFolder: dirname(ownerFile,),
              ownerFile,
              branch: BRANCH,
              unit: UNIT,
              manager,
              owner: aRecord({ commit: 'commit-b', claim: 'claim-b', process: await processIdentityOfThisProcess(), },),
            },),).rejects.toBeInstanceOf(ReviewFolderTakenError,);
            expect((await theOwnerShown({ ownerFile, },)).claim,).toBe('claim-a',);
          },
        },),
        it({
          name: 'keeps a dead start\'s folder while the review unit its record names still runs, and takes it once that unit has ended',
          fn: async () => {
            await using folder = await makeFolder();
            const ownerFile = ownerFileOfAReview({ reviewsFolder: folder.path, review: REVIEW, },);
            await writeOwner({
              ownerFile,
              record: aRecord({ commit: 'dead-commit', claim: 'the-dead-claim', process: aDeadStart(), },),
            },);
            /**
             The process that stands in for the review's unit, which outlives the command that wrote the record, and which is
             killed when this test ends, however it ends.
             */
            await using theStandIn = {
              child: spawn(process.execPath, ['--input-type=module', '-e', 'setInterval(() => {}, 1000);',], {
                stdio: 'ignore',
              },),
              async [Symbol.asyncDispose]() {
                this.child.kill('SIGKILL',);
              },
            };
            /**
             The stand-in the manager watches.
             */
            const { child: standIn, } = theStandIn;
            await once(standIn, 'spawn',);
            /**
             The unit names the manager was asked about.
             */
            const asked: string[] = [];
            const manager = aManagerWatching({ standIn, asked, },);
            /**
             The start's own identity.
             */
            const identity = await processIdentityOfThisProcess();
            /**
             The refusal a start while the review runs answers.
             */
            let caught: unknown;
            try {
              await takeReviewFolder({
                reviewFolder: dirname(ownerFile,),
                ownerFile,
                branch: BRANCH,
                unit: UNIT,
                manager,
                owner: aRecord({ commit: 'commit-b', claim: 'claim-b', process: identity, },),
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(ReviewFolderTakenError,);
            expect(String(caught,),).toContain(UNIT,);
            expect((await theOwnerShown({ ownerFile, },)).claim,).toBe('the-dead-claim',);
            expect(asked,).toContain(UNIT,);
            standIn.kill('SIGKILL',);
            await once(standIn, 'exit',);
            await takeReviewFolder({
              reviewFolder: dirname(ownerFile,),
              ownerFile,
              branch: BRANCH,
              unit: UNIT,
              manager,
              owner: aRecord({ commit: 'commit-c', claim: 'claim-c', process: identity, },),
            },);
            const owner = await theOwnerShown({ ownerFile, },);
            expect(owner.claim,).toBe('claim-c',);
            expect(owner.process.pid,).toBe(identity.pid,);
          },
        },),
        it({
          name: 'takes a dead command\'s folder when its record names a unit that was never started, because no review runs to keep it',
          fn: async () => {
            await using folder = await makeFolder();
            const ownerFile = ownerFileOfAReview({ reviewsFolder: folder.path, review: REVIEW, },);
            await writeOwner({
              ownerFile,
              record: aRecord({ commit: 'dead-commit', claim: 'the-dead-claim', process: aDeadStart(), },),
            },);
            /**
             The unit names the manager was asked about.
             */
            const asked: string[] = [];
            const manager = aManagerAnswering({ state: 'inactive', asked, },);
            const identity = await processIdentityOfThisProcess();
            await takeReviewFolder({
              reviewFolder: dirname(ownerFile,),
              ownerFile,
              branch: BRANCH,
              unit: UNIT,
              manager,
              owner: aRecord({ commit: 'commit-b', claim: 'claim-b', process: identity, },),
            },);
            const owner = await theOwnerShown({ ownerFile, },);
            expect(owner.claim,).toBe('claim-b',);
            expect(asked,).toContain(UNIT,);
          },
        },),
        it({
          name: 'keeps a dead start\'s folder when the manager does not answer for its review unit, because unknown is not proof that no review runs',
          fn: async () => {
            await using folder = await makeFolder();
            const ownerFile = ownerFileOfAReview({ reviewsFolder: folder.path, review: REVIEW, },);
            await writeOwner({
              ownerFile,
              record: aRecord({ commit: 'dead-commit', claim: 'the-dead-claim', process: aDeadStart(), },),
            },);
            /**
             A manager that could not be asked, so it printed nothing.
             */
            const manager = aManagerAnswering({ state: '', },);
            let caught: unknown;
            try {
              await takeReviewFolder({
                reviewFolder: dirname(ownerFile,),
                ownerFile,
                branch: BRANCH,
                unit: UNIT,
                manager,
                owner: aRecord({ commit: 'commit-b', claim: 'claim-b', process: await processIdentityOfThisProcess(), },),
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(ReviewFolderTakenError,);
            expect(String(caught,),).toContain(UNIT,);
            expect((await theOwnerShown({ ownerFile, },)).claim,).toBe('the-dead-claim',);
          },
        },),
      ],
    },),
  ],
},);
