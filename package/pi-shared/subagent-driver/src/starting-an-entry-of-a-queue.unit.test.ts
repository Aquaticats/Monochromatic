/**
 Starting one entry of a queue: the run's own batch list written and its unit asked for, a review of a range whose change leaves
 nothing to read answered as the entry's own failure, a git that refused because a lock is held answered as the machine's, and a unit
 the manager refused answered as the machine's too.

 No test starts a unit or a run: the commands are answered by a seam, and the batch list a start writes is read back from a throwaway
 runs folder.

 @module
 */

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
  type CommandOutput,
  REVIEW_WALL_SECONDS,
  gitRefusalIsTheMachines,
  reviewRunEntryOf,
  startAnEntryOfAQueue,
  type StartOfAnEntry,
} from '../dist/final/node/index.mjs';

/**
 A folder a test may write in, with the checkout, the runs and the reviews it needs, removed when the test ends.
 */
type AThrowawayFolder = {
  /**
   The folder's path.
   */
  readonly path: string;

  /**
   The checkout a review's range is read from, which is a folder a checkout marks.
   */
  readonly checkout: string;

  /**
   The folder the runs live in.
   */
  readonly runs: string;

  /**
   The folder one folder per review goes in.
   */
  readonly reviews: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A folder a test may write in, with the checkout, the runs and the reviews it needs, removed when the test ends.

 @returns the folder
 */
async function makeFolder(): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-queue-start-',),);
  /**
   The checkout a review is read from, marked the way a checkout marks itself.
   */
  const checkout = join(path, 'checkout',);
  /**
   The folder the runs live in.
   */
  const runs = join(path, 'runs',);
  /**
   The folder one folder per review goes in.
   */
  const reviews = join(path, 'reviews',);
  await mkdir(join(checkout, '.git',), { recursive: true, },);
  await mkdir(runs, { recursive: true, },);
  await mkdir(reviews, { recursive: true, },);
  return {
    path,
    checkout,
    runs,
    reviews,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 A run of the queue, named `a`.
 */
const A_RUN = {
  name: 'a',
  branch: 'feat/a',
  prompt: '/runs/a-prompt.md',
  base: 'abc1234',
  wall: 28_800,
};

/**
 A review of a range of a change, named `review-b`.
 */
const A_REVIEW_OF_A_RANGE = {
  name: 'review-b',
  reviewOfRange: 'main...abc1234',
  commit: 'abc1234',
  source: 'src',
};

/**
 Answers the commands a start runs: the environment's `PATH`, and the manager's own words.

 @param request - what the command was asked, and whether the manager refuses

 @returns the answer
 */
function commandsAnswering({
  systemdRefuses,
  systemdProblem,
}: {
  readonly systemdRefuses?: boolean;
  readonly systemdProblem?: string;
},): (request: {
  readonly command: string;
  readonly words: readonly string[];
}) => Promise<CommandOutput> {
  return function run({
    command,
    words,
  }: {
    readonly command: string;
    readonly words: readonly string[];
  },): Promise<CommandOutput> {
    if (command === 'mise') {
      return Promise.resolve({
        status: 0,
        stdout: '/bin:/usr/bin\n',
        stderr: '',
      },);
    }
    if ((command === 'systemd-run') && (systemdRefuses === true)) {
      return Promise.resolve({
        status: 1,
        stdout: '',
        stderr: systemdProblem ?? 'the manager refused',
      },);
    }
    return Promise.resolve({
      status: 0,
      stdout: `${command} ${words.join(' ',)}\n`,
      stderr: '',
    },);
  };
}

/**
 Answers git's questions about one change: the files it touched, the tree the reviewed commit holds, and the diff.

 @param request - the files the range touched, and whether a question refuses

 @returns the answer for one question
 */
function gitAnswering({
  touched,
  refusal,
}: {
  readonly touched: readonly string[];
  readonly refusal?: string;
},): (request: {
  readonly words: readonly string[];
  readonly folder: string;
}) => Promise<CommandOutput> {
  return function git({
    words,
  }: {
    readonly words: readonly string[];
    readonly folder: string;
  },): Promise<CommandOutput> {
    if (refusal !== undefined) {
      return Promise.resolve({
        status: 128,
        stdout: '',
        stderr: refusal,
      },);
    }
    if (words.includes('--name-only',)) {
      return Promise.resolve({
        status: 0,
        stdout: `${touched.join('\n',)}\n`,
        stderr: '',
      },);
    }
    if (words.includes('ls-tree',)) {
      return Promise.resolve({
        status: 0,
        stdout: `${touched.join('\n',)}\n`,
        stderr: '',
      },);
    }
    return Promise.resolve({
      status: 0,
      stdout: 'diff --git a/src/a.ts b/src/a.ts\n',
      stderr: '',
    },);
  };
}

/**
 Reads the one-entry batch list a start wrote.

 @param started - what the start answered, when it started something

 @returns the entry the list holds
 */
async function entryOfTheList(started: Extract<StartOfAnEntry, { readonly started: true }>,): Promise<readonly unknown[]> {
  return JSON.parse(await readFile(started.batchFile, 'utf8',),) as readonly unknown[];
}

await describe({
  name: '',
  children: [
    describe({
      name: startAnEntryOfAQueue.name,
      children: [
        it({
          name: 'writes the run its own one-entry batch list and asks the manager for its unit, saying where the unit writes',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             What starting the run answered.
             */
            const started = await startAnEntryOfAQueue({
              request: {
                record: A_RUN,
                runsFolder: folder.runs,
                reviewsFolder: folder.reviews,
                checkoutFolder: folder.checkout,
                home: folder.path,
                now: Date.now,
              },
              seams: {
                git: gitAnswering({ touched: [], },),
                run: commandsAnswering({},),
              },
            },);
            expect(started.started,).toBe(true,);
            if (!started.started) {
              return;
            }
            expect(started.unit,).toContain('launch-batch-',);
            expect(started.unit,).toContain('-queue-entry-a',);
            expect(started.outputFile,).toBe(started.batchFile.replace('.json', '.out',),);
            expect(await entryOfTheList(started,),).toEqual([A_RUN,],);
          },
        },),
        it({
          name: 'answers a review of a range whose change leaves nothing to read as the entry\'s own failure, naming what it read',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             What starting the review answered.
             */
            const started = await startAnEntryOfAQueue({
              request: {
                record: A_REVIEW_OF_A_RANGE,
                runsFolder: folder.runs,
                reviewsFolder: folder.reviews,
                checkoutFolder: folder.checkout,
                home: folder.path,
                now: Date.now,
              },
              seams: {
                git: gitAnswering({ touched: ['src/a.test.ts',], },),
                run: commandsAnswering({},),
              },
            },);
            expect(started.started,).toBe(false,);
            if (started.started) {
              return;
            }
            expect(started.failure.kind,).toBe('the-entry',);
            expect(started.failure.reason,).toBe('nothing to review under src in main...abc1234',);
          },
        },),
        it({
          name: 'writes the brief of a review whose change has code to read, and starts the review run the package names',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             What starting the review answered.
             */
            const started = await startAnEntryOfAQueue({
              request: {
                record: A_REVIEW_OF_A_RANGE,
                runsFolder: folder.runs,
                reviewsFolder: folder.reviews,
                checkoutFolder: folder.checkout,
                home: folder.path,
                now: Date.now,
              },
              seams: {
                git: gitAnswering({ touched: ['src/a.ts',], },),
                run: commandsAnswering({},),
              },
            },);
            expect(started.started,).toBe(true,);
            if (!started.started) {
              return;
            }
            expect(await readFile(join(folder.reviews, 'b', 'brief.md',), 'utf8',),).toContain('abc1234',);
            expect(await entryOfTheList(started,),).toEqual([
              reviewRunEntryOf({
                review: A_REVIEW_OF_A_RANGE,
                reviewsFolder: folder.reviews,
              },),
            ],);
            expect(reviewRunEntryOf({
              review: A_REVIEW_OF_A_RANGE,
              reviewsFolder: '/reviews',
            },),).toMatchObject({
              name: 'review-b',
              branch: 'test/review-b',
              base: 'abc1234',
              prompt: '/reviews/b/brief.md',
              wall: REVIEW_WALL_SECONDS,
            },);
          },
        },),
        it({
          name: 'answers a git that refused because a lock is held as the machine\'s, and a range that names no commit as the entry\'s',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The git refusal a held lock leaves.
             */
            const locked = 'fatal: Unable to create \'/repo/.git/index.lock\': File exists.';
            /**
             What starting the review answered with a lock held.
             */
            const held = await startAnEntryOfAQueue({
              request: {
                record: A_REVIEW_OF_A_RANGE,
                runsFolder: folder.runs,
                reviewsFolder: folder.reviews,
                checkoutFolder: folder.checkout,
                home: folder.path,
                now: Date.now,
              },
              seams: {
                git: gitAnswering({
                  touched: [],
                  refusal: locked,
                },),
                run: commandsAnswering({},),
              },
            },);
            expect(held.started,).toBe(false,);
            expect(held.started ? {} : held.failure.kind,).toBe('the-machine',);
            expect(gitRefusalIsTheMachines({ reason: locked, },),).toBe(true,);
            expect(gitRefusalIsTheMachines({ reason: 'fatal: bad revision \'main...nope\'', },),).toBe(false,);
            /**
             What starting a review whose range names no commit answered.
             */
            const unknown = await startAnEntryOfAQueue({
              request: {
                record: A_REVIEW_OF_A_RANGE,
                runsFolder: folder.runs,
                reviewsFolder: folder.reviews,
                checkoutFolder: folder.checkout,
                home: folder.path,
                now: Date.now,
              },
              seams: {
                git: gitAnswering({
                  touched: [],
                  refusal: 'fatal: bad revision \'main...abc1234\'',
                },),
                run: commandsAnswering({},),
              },
            },);
            expect(unknown.started,).toBe(false,);
            expect(unknown.started ? {} : unknown.failure.kind,).toBe('the-entry',);
          },
        },),
        it({
          name: 'answers a unit the manager refused as the machine\'s, saying what the manager said',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             What starting the run answered.
             */
            const started = await startAnEntryOfAQueue({
              request: {
                record: A_RUN,
                runsFolder: folder.runs,
                reviewsFolder: folder.reviews,
                checkoutFolder: folder.checkout,
                home: folder.path,
                now: Date.now,
              },
              seams: {
                git: gitAnswering({ touched: [], },),
                run: commandsAnswering({
                  systemdRefuses: true,
                  systemdProblem: 'Unit launch-batch-a.service already exists.',
                },),
              },
            },);
            expect(started.started,).toBe(false,);
            if (started.started) {
              return;
            }
            expect(started.failure.kind,).toBe('the-machine',);
            expect(started.failure.reason,).toContain('already exists',);
          },
        },),
        it({
          name: 'answers an entry no shape reads as the entry\'s own failure, naming the field it does not read',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             What starting the entry answered.
             */
            const started = await startAnEntryOfAQueue({
              request: {
                record: {
                  name: 'odd',
                  branch: 'feat/odd',
                  reviewer: 'someone',
                },
                runsFolder: folder.runs,
                reviewsFolder: folder.reviews,
                checkoutFolder: folder.checkout,
                home: folder.path,
                now: Date.now,
              },
              seams: {
                git: gitAnswering({ touched: [], },),
                run: commandsAnswering({},),
              },
            },);
            expect(started.started,).toBe(false,);
            if (started.started) {
              return;
            }
            expect(started.failure.kind,).toBe('the-entry',);
            expect(started.failure.reason,).toContain('reviewer',);
          },
        },),
      ],
    },),
    describe({
      name: reviewRunEntryOf.name,
      children: [
        it({
          name: 'names the review run after the change alone, on the branch and from the commit the review reads',
          fn: async () => {
            expect(reviewRunEntryOf({
              review: {
                name: 'review-c',
                reviewOfRange: 'main...abc1234',
                commit: 'abc1234',
                source: 'src',
              },
              reviewsFolder: '/reviews',
            },),).toMatchObject({
              name: 'review-c',
              branch: 'test/review-c',
              base: 'abc1234',
            },);
          },
        },),
      ],
    },),
  ],
},);
