/**
 Writes what a review of one change reads: the diff of its ranges, the record of what it was given, and its brief.

 The order of the writes is part of what this file checks. The record is written before the brief, so that no review folder holds a
 brief without the record its review states its ranges against: a run killed between two writes leaves a record nothing has used
 yet rather than a brief whose review can only be answered as one that cannot be told.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
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
  type GitQuestion,
  ReviewGitError,
  writeReviewBrief,
} from '../dist/final/node/index.mjs';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The change every brief here is written for.

 @returns the change's name

 @example
 ```ts
 const review = aReview();
 ```
 */
function aReview(): string {
  return 'rewrite-the-gate';
}

/**
 The files a change touches, which the git questions here answer with.

 @returns the files, from the top of the checkout

 @example
 ```ts
 const files = theFilesTouched();
 ```
 */
function theFilesTouched(): readonly string[] {
  return ['package/a/src/gate.rs',];
}

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
  const path = await mkdtemp(join(scratchRoot(), 'pi-review-brief-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Answers the git questions one brief is written from: the diff of the range, the files it touched, and the files the commit holds.

 @param files - the files the range touched and the commit holds

 @returns the answers, by the question asked

 @example
 ```ts
 const git = gitAnswering({ files: theFilesTouched(), },);
 ```
 */
function gitAnswering({ files, }: { readonly files: readonly string[] },): GitQuestion {
  return async function answer({ words, }: {
    readonly words: readonly string[];
    readonly folder: string;
  },): Promise<CommandOutput> {
    /**
     What git prints for this question, which is a path per line unless it is asked for the diff itself.
     */
    const stdout = words.includes('diff',) && (!words.includes('--name-only',))
      ? 'a diff of the range\n'
      : `${files.join('\n',)}\n`;
    return {
      status: 0,
      stdout,
      stderr: '',
    };
  };
}

/**
 The request every brief in this file is written from.

 @param reviewsFolder - the folder that holds one folder per review

 @param git - how the git questions are answered

 @returns the request

 @example
 ```ts
 const request = aBriefRequest({ reviewsFolder: '/reviews', git: gitAnswering({ files: theFilesTouched(), },), },);
 ```
 */
function aBriefRequest({
  reviewsFolder,
  git,
}: {
  readonly reviewsFolder: string;
  readonly git: GitQuestion;
}): Parameters<typeof writeReviewBrief>[0] {
  return {
    git,
    checkoutFolder: '/repo',
    review: aReview(),
    commit: 'abc1234',
    ranges: ['main...fix/rewrite-the-gate',],
    source: 'package/a/src',
    focus: 'data lost or overwritten',
    reviewsFolder,
    runsFolder: '/runs',
    rulesFile: '/repo/doc/rules-every-pi-brief-carries.md',
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: writeReviewBrief.name,
      children: [
        it({
          name: 'writes the diff, the record of what the review was given and the brief, and answers the files it names',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder that holds one folder per review.
             */
            const reviewsFolder = join(folder.path, 'reviews',);
            /**
             What writing the brief answered.
             */
            const briefed = await writeReviewBrief(aBriefRequest({
              reviewsFolder,
              git: gitAnswering({ files: theFilesTouched(), },),
            },),);
            expect(briefed.written,).toBe(true,);
            expect(await readFile(join(reviewsFolder, aReview(), 'change.diff',), 'utf8',),)
              .toBe('a diff of the range\n',);
            expect(
              JSON.parse(await readFile(
                join(reviewsFolder, aReview(), 'what-the-review-read.json',),
                'utf8',
              ),),
            ).toEqual({
              review: aReview(),
              commit: 'abc1234',
              ranges: ['main...fix/rewrite-the-gate',],
              source: 'package/a/src',
              files: theFilesTouched(),
            },);
          },
        },),
        it({
          name: 'writes the record of what the review was given before the brief, so no folder holds a brief without its record',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder that holds one folder per review.
             */
            const reviewsFolder = join(folder.path, 'reviews',);
            // A directory where the record goes makes its write fail after the diff was written, which is the state a process
            // killed between the record's write and the brief's leaves; a brief without a record is what must not be left.
            await mkdir(join(reviewsFolder, aReview(), 'what-the-review-read.json',), { recursive: true, },);
            await expect(
              writeReviewBrief(aBriefRequest({
                reviewsFolder,
                git: gitAnswering({ files: theFilesTouched(), },),
              },),),
            ).rejects.toThrow();
            await expect(
              readFile(join(reviewsFolder, aReview(), 'brief.md',),),
            ).rejects.toThrow(MISSING_PATH_CODE,);
          },
        },),
        it({
          name: 'leaves the record of what the review was given when the brief cannot be written, which is a state nothing reads',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder that holds one folder per review.
             */
            const reviewsFolder = join(folder.path, 'reviews',);
            /**
             The record of what the review was given, which the writer puts in place before the brief.
             */
            const readingFile = join(reviewsFolder, aReview(), 'what-the-review-read.json',);
            // A directory where the brief goes fails its write alone, so what is left is the state a kill between the two writes
            // leaves once the record goes first.
            await mkdir(join(reviewsFolder, aReview(), 'brief.md',), { recursive: true, },);
            await expect(
              writeReviewBrief(aBriefRequest({
                reviewsFolder,
                git: gitAnswering({ files: theFilesTouched(), },),
              },),),
            ).rejects.toThrow();
            expect(
              JSON.parse(await readFile(readingFile, 'utf8',),),
            ).toEqual({
              review: aReview(),
              commit: 'abc1234',
              ranges: ['main...fix/rewrite-the-gate',],
              source: 'package/a/src',
              files: theFilesTouched(),
            },);
          },
        },),
        it({
          name: 'writes nothing and says nothing to review when the change leaves no file a review reads',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             What writing the brief answered for a change that touches only a note.
             */
            const briefed = await writeReviewBrief(aBriefRequest({
              reviewsFolder: join(folder.path, 'reviews',),
              git: gitAnswering({ files: ['package/a/src/NOTES.md',], },),
            },),);
            expect(briefed.written,).toBe(false,);
            expect(briefed.line,).toContain('nothing to review',);
          },
        },),
        it({
          name: 'refuses to answer when git did not answer with a success, naming the question and what git said',
          fn: async () => {
            await using folder = await makeFolder();
            await expect(
              writeReviewBrief(aBriefRequest({
                reviewsFolder: join(folder.path, 'reviews',),
                git: async function fails() {
                  return {
                    status: 128,
                    stdout: '',
                    stderr: 'fatal: bad revision',
                  };
                },
              },),),
            ).rejects.toThrow(ReviewGitError,);
          },
        },),
      ],
    },),
  ],
},);
