/**
 The writer of review briefs against disposable folders: a change of tests and fixtures alone gets a brief that lists each
 changed test and asks for a break planted under it, a change of code is described as it always was, and a range that changes
 nothing a review reads is refused with words that say so.

 Every git answer here is handed in by the test, so no checkout or worktree is involved, and every folder is one the test made and
 throws away. Nothing starts a run.

 @module
 */

import { existsSync, } from 'node:fs';
import {
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
  type BriefOfOneReview,
  type CommandOutput,
  FOCUS_OF_A_CHANGE_REVIEW,
  briefFileOfAReview,
  readingFileOfAReview,
  writeReviewBrief,
} from '../dist/final/node/index.mjs';

/**
 The change a review reads.

 @returns the change's name

 @example
 ```ts
 const review = aReview();
 ```
 */
function aReview(): string {
  return 'a-change-to-tests';
}

/**
 The commit under review.

 @returns the commit's id

 @example
 ```ts
 const commit = aCommit();
 ```
 */
function aCommit(): string {
  return 'abc1234';
}

/**
 The folder whose changes are reviewed.

 @returns the folder's path, from the top of the checkout

 @example
 ```ts
 const source = aSourceFolder();
 ```
 */
function aSourceFolder(): string {
  return 'package/a/src';
}

/**
 What changed, as `git diff` takes a range.

 @returns the range

 @example
 ```ts
 const range = aRange();
 ```
 */
function aRange(): string {
  return 'main...fix/rewrite-the-gate';
}

/**
 The code file a change of code leaves to read.

 @returns the file's path, from the top of the checkout

 @example
 ```ts
 const code = aCodeFile();
 ```
 */
function aCodeFile(): string {
  return `${aSourceFolder()}/gate.rs`;
}

/**
 The changed test file a change to tests alone leaves to read.

 @returns the file's path, from the top of the checkout

 @example
 ```ts
 const test = aTestFile();
 ```
 */
function aTestFile(): string {
  return `${aSourceFolder()}/gate_tests.rs`;
}

/**
 The changed fixture a change to tests alone leaves to read.

 @returns the file's path, from the top of the checkout

 @example
 ```ts
 const fixture = aFixtureFile();
 ```
 */
function aFixtureFile(): string {
  return `${aSourceFolder()}/stand-in-git.ts`;
}

/**
 The shim package's own source and type file, whose language a review does not read.

 @returns their paths, from the top of the checkout

 @example
 ```ts
 const source = sourceFilesNoReviewReads();
 ```
 */
function sourceFilesNoReviewReads(): readonly string[] {
  return [
    `${aSourceFolder()}/index.cjs`,
    `${aSourceFolder()}/index.d.cts`,
  ];
}

/**
 The rules file the brief names first.

 @returns the file's path

 @example
 ```ts
 const rules = aRulesFile();
 ```
 */
function aRulesFile(): string {
  return '/repo/package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md';
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
  const path = await mkdtemp(join(scratchRoot(), 'pi-review-briefs-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 A stand-in git whose range changes exactly the paths given, which the reviewed commit still holds.

 @param paths - the paths the change touched, each from the top of the checkout

 @returns a git that answers the three questions one reading asks

 @example
 ```ts
 const git = gitTouching({ paths: [aTestFile()], },);
 ```
 */
function gitTouching({ paths, }: { readonly paths: readonly string[] },): (
  request: { readonly words: readonly string[] },
) => Promise<CommandOutput> {
  return async function git({ words, }: { readonly words: readonly string[] },): Promise<CommandOutput> {
    /**
     The paths as git prints them, one to a line.
     */
    const listed = (paths.length === 0) ? '' : `${paths.join('\n',)}\n`;
    return {
      status: 0,
      stdout: (words[0] === 'ls-tree') ? listed : ((words[1] === '--name-only') ? listed : 'a diff of them'),
      stderr: '',
    };
  };
}

/**
 Runs the writer for one change, in a folder of its own.

 @param paths - the paths the change touched, each from the top of the checkout

 @returns what the writer answered, the reviews folder it wrote into, and the folder to throw away

 @example
 ```ts
 await using written = await writeAChangeThrough([aTestFile()]);
 ```
 */
async function writeAChangeThrough(paths: readonly string[],): Promise<{
  readonly briefed: BriefOfOneReview
  readonly reviewsFolder: string
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder this test throws away.
   */
  const folder = await makeFolder();
  /**
   The folder one review of the change keeps its own files in.
   */
  const reviewsFolder = join(folder.path, 'reviews',);
  /**
   What the writer answered.
   */
  const briefed = await writeReviewBrief({
    git: gitTouching({ paths, },),
    checkoutFolder: '/repo',
    review: aReview(),
    commit: aCommit(),
    ranges: [aRange(),],
    source: aSourceFolder(),
    focus: FOCUS_OF_A_CHANGE_REVIEW,
    reviewsFolder,
    runsFolder: join(folder.path, 'runs',),
    rulesFile: aRulesFile(),
  },);
  return {
    briefed,
    reviewsFolder,
    [Symbol.asyncDispose]: async function dispose() {
      await folder[Symbol.asyncDispose]();
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: writeReviewBrief.name,
      children: [
        it({
          name: 'writes a brief for a change of tests and fixtures alone, listing each changed test and fixture and asking for a break planted under each test, which is what makes such a change reviewable at all',
          fn: async () => {
            await using written = await writeAChangeThrough([aTestFile(), aFixtureFile(),],);
            /**
             The brief the writer wrote.
             */
            const brief = await readFile(
              briefFileOfAReview({
                reviewsFolder: written.reviewsFolder,
                review: aReview(),
              },),
              'utf8',
            );
            expect(written.briefed.written,).toBe(true,);
            expect(written.briefed.line,).toContain('2 files to read',);
            expect(brief,).toContain(`Review a change to the tests under \`${aSourceFolder()}\`.`,);
            expect(brief,).toContain('the change is to them alone',);
            expect(brief,).toContain(`- \`${aTestFile()}\``,);
            expect(brief,).toContain(`- \`${aFixtureFile()}\``,);
            expect(brief,).toContain('For each changed test above, plant a break under it',);
            expect(brief,).toContain('run that test by name, and say whether it fails',);
            expect(brief,).toContain('a test that still passes with the break in is a finding',);
            expect(brief,).toContain('Say what you read and what you ran',);
            expect(brief,).toContain('A review that ends with no file is lost',);
          },
        },),
        it({
          name: 'keeps the record of what the review read for a change of tests alone, so a fix that answers it starts at the commit it read',
          fn: async () => {
            await using written = await writeAChangeThrough([aTestFile(),],);
            /**
             What the writer recorded the review read.
             */
            const reading = JSON.parse(await readFile(
              readingFileOfAReview({
                reviewsFolder: written.reviewsFolder,
                review: aReview(),
              },),
              'utf8',
            ),) as { readonly commit?: unknown };
            expect(reading.commit,).toBe(aCommit(),);
          },
        },),
        it({
          name: 'writes the brief of a change of code as it always did, naming the code and asking for no break, even when the range changed a test beside it',
          fn: async () => {
            await using written = await writeAChangeThrough([aCodeFile(), aTestFile(),],);
            /**
             The brief the writer wrote.
             */
            const brief = await readFile(
              briefFileOfAReview({
                reviewsFolder: written.reviewsFolder,
                review: aReview(),
              },),
              'utf8',
            );
            expect(written.briefed.written,).toBe(true,);
            expect(brief,).toContain(`Review a change under \`${aSourceFolder()}\`.`,);
            expect(brief,).toContain(`- \`${aCodeFile()}\``,);
            expect(brief,).not.toContain(`- \`${aTestFile()}\``,);
            expect(brief,).not.toContain('plant a break',);
          },
        },),
        it({
          name: 'refuses a range that changes nothing under the source folder, in words that say so and never say tests, and writes no brief',
          fn: async () => {
            await using written = await writeAChangeThrough([],);
            expect(written.briefed,).toEqual({
              written: false,
              line: `nothing to review under ${aSourceFolder()} in ${aRange()}`,
            },);
            expect(written.briefed.line.toLowerCase(),).not.toContain('test',);
            /**
             Where the review would keep its brief.
             */
            const briefFile = briefFileOfAReview({
              reviewsFolder: written.reviewsFolder,
              review: aReview(),
            },);
            expect(existsSync(briefFile,),).toBe(false,);
          },
        },),
        it({
          name: 'refuses a change of a source file whose language a review does not read beside its test, because a brief of the test alone would hide the change itself',
          fn: async () => {
            await using written = await writeAChangeThrough([...sourceFilesNoReviewReads(), aTestFile(),],);
            expect(written.briefed,).toEqual({
              written: false,
              line: `nothing to review under ${aSourceFolder()} in ${aRange()}`,
            },);
          },
        },),
      ],
    },),
  ],
},);
