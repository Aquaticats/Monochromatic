/**
 The fix brief writer against disposable folders: what it refuses, and what it leaves where. Every fixture here is a folder the
 test made and throws away, and the command runs as its own process so a refusal is judged by the status and the lines it prints.
 Nothing starts a run, a unit or a Pi.

 @module
 */

import {
  execFile,
} from 'node:child_process';
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  CHANGE_DIFF_FILE_NAME,
  REVIEW_FILE_NAME,
  REVIEW_READING_FILE_NAME,
} from '../dist/final/node/index.mjs';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

/**
 The command this file judges, run from source the way its own task runs it.
 */
const COMMAND = new URL('write-a-fix-brief-for-each-review.ts', import.meta.url,).pathname;

/**
 The status of a command that refused.
 */
const REFUSED_STATUS = 1;

/**
 The status of a command that did all it was asked.
 */
const CLEAN_STATUS = 0;

/**
 The review both tests here answer.

 @returns its name

 @example
 ```ts
 const review = aReview();
 ```
 */
function aReview(): string {
  return 'a-round-1';
}

/**
 The commit the review recorded it read.

 @returns the commit's id

 @example
 ```ts
 const base = aBase();
 ```
 */
function aBase(): string {
  return 'abc1234';
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
  const path = await mkdtemp(join(scratchRoot(), 'pi-fix-briefs-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes one review's three files into a reviews folder, so a fix has something to answer.

 @param reviewsFolder - the folder the reviews live in

 @example
 ```ts
 await writeAReviewInto(folder.path);
 ```
 */
async function writeAReviewInto(reviewsFolder: string,): Promise<void> {
  /**
   The review's own folder.
   */
  const folder = join(reviewsFolder, aReview(),);
  await mkdir(folder, { recursive: true, },);
  await writeFile(join(folder, REVIEW_FILE_NAME,), '# the defects\n',);
  await writeFile(join(folder, CHANGE_DIFF_FILE_NAME,), 'diff --git a/x b/x\n',);
  await writeFile(
    join(folder, REVIEW_READING_FILE_NAME,),
    `${JSON.stringify({
      review: aReview(),
      commit: aBase(),
      source: 'package/a/src',
      ranges: ['main...fix/a',],
    },)}\n`,
  );
}

/**
 Runs the fix brief writer with the words a test gives it, from the checkout so the rules file it names is the real one.

 @param words - the command's own words

 @returns the status it exited with and both streams it printed

 @example
 ```ts
 const ran = await runTheCommand(words);
 ```
 */
async function runTheCommand(words: readonly string[],): Promise<{
  readonly status: number;
  readonly printed: string
}> {
  try {
    /**
     What the command printed.
     */
    const answered = await execFileAsync(
      process.execPath,
      [
        COMMAND,
        ...words,
      ],
      {
        cwd: new URL('../', import.meta.url,).pathname,
        encoding: 'utf8',
      },
    );
    return {
      status: CLEAN_STATUS,
      printed: `${answered.stdout}${answered.stderr}`,
    };
  }
  catch (error) {
    /**
     How the command ended, read field by field because a refused command carries its status on the error.
     */
    const failed = error as {
      readonly code?: unknown;
      readonly stdout?: unknown;
      readonly stderr?: unknown;
    };
    return {
      status: (typeof failed.code) === 'number' ? failed.code : REFUSED_STATUS,
      printed: `${String(failed.stdout,)}${String(failed.stderr,)}`,
    };
  }
}

/**
 Reads the batch list the words name, or says it was not written.

 @param folder - the test's own folder

 @returns the list's text, or the words that say no list was written

 @example
 ```ts
 const list = await batchListOrNothing(folder.path);
 ```
 */
async function batchListOrNothing(folder: string,): Promise<string> {
  try {
    return await readFile(
      join(folder, 'batch.json',),
      'utf8',
    );
  }
  catch (error) {
    // A list that was not written is what a refused command must leave behind.
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return 'not written';
    }
    throw error;
  }
}

/**
 The words that name one review, one fix run and one folder set.

 @param folder - the test's own folder

 @param name - the name the fix run gets

 @returns the command's words

 @example
 ```ts
 const words = wordsFor({ folder, name: 'a-late-runtime' });
 ```
 */
function wordsFor({
  folder,
  name,
}: {
  readonly folder: string;
  readonly name: string;
}): readonly string[] {
  return [
    `--review=${aReview()}`,
    `--name=${name}`,
    '--branch-prefix=fix/git-policy-cli-',
    '--package=package/a',
    `--reviews-folder=${join(folder, 'reviews',)}`,
    `--runs-folder=${join(folder, 'runs',)}`,
    `--batch-file=${join(folder, 'batch.json',)}`,
  ];
}

await describe({
  name: '',
  children: [
    describe({
      name: 'the file each brief is written to',
      children: [
        it({
          name: 'is kept when it already holds other text, which is a person\'s brief, and the refusal names it',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder the briefs are written into.
             */
            const runsFolder = join(folder.path, 'runs',);
            await mkdir(runsFolder, { recursive: true, },);
            await writeAReviewInto(join(folder.path, 'reviews',),);
            /**
             The words this test runs the command with.
             */
            const name = 'a-late-runtime';
            const words = wordsFor({
              folder: folder.path,
              name,
            },);
            /**
             The brief already at the path this command would write, as an earlier round left it.
             */
            const kept = '# a brief nobody may replace\n';
            await writeFile(join(runsFolder, `${name}-prompt.md`,), kept,);
            /**
             What running the command ended with.
             */
            const ran = await runTheCommand(words,);
            expect(ran.status,).toBe(REFUSED_STATUS,);
            expect(ran.printed,).toContain(`${name}-prompt.md`,);
            expect(ran.printed,).toContain('already there with other text',);
            // Nothing was discarded, and no batch list was written for a fix that was not planned.
            expect(await readFile(join(runsFolder, `${name}-prompt.md`,), 'utf8',),).toBe(kept,);
            expect(await batchListOrNothing(folder.path,),).toBe('not written',);
          },
        },),
        it({
          name: 'is written again, with no refusal and no second list entry, when it already holds exactly what would be written',
          fn: async () => {
            await using folder = await makeFolder();
            await writeAReviewInto(join(folder.path, 'reviews',),);
            /**
             The words this test runs the command with, twice.
             */
            const words = wordsFor({
              folder: folder.path,
              name: 'a-late-runtime',
            },);
            expect((await runTheCommand(words,)).status,).toBe(CLEAN_STATUS,);
            /**
             What the second run of the same words ended with.
             */
            const again = await runTheCommand(words,);
            expect(again.status,).toBe(CLEAN_STATUS,);
            expect(again.printed,).toContain('a-late-runtime: brief at',);
          },
        },),
      ],
    },),
    describe({
      name: 'the name a fix run gets',
      children: [
        it({
          name: 'is refused when a run folder of that name is already in the runs folder, because a batch entry of that'
            + ' name restarts inside that run',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder the runs live in.
             */
            const runsFolder = join(folder.path, 'runs',);
            await writeAReviewInto(join(folder.path, 'reviews',),);
            /**
             The name that is already a run's.
             */
            const name = 'a-late-runtime';
            await mkdir(join(runsFolder, `${name}-1`,), { recursive: true, },);
            /**
             What running the command ended with.
             */
            const ran = await runTheCommand(wordsFor({
              folder: folder.path,
              name,
            },),);
            expect(ran.status,).toBe(REFUSED_STATUS,);
            expect(ran.printed,).toContain(`${name}-1`,);
            expect(ran.printed,).toContain('this name is taken',);
          },
        },),
        it({
          name: 'is refused when a brief of another fix already holds it, so two fixes never share one brief path',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder the briefs are written into.
             */
            const runsFolder = join(folder.path, 'runs',);
            await writeAReviewInto(join(folder.path, 'reviews',),);
            /**
             The name another fix's brief already holds.
             */
            const name = 'a-late-runtime';
            await mkdir(runsFolder, { recursive: true, },);
            await writeFile(join(runsFolder, `${name}-start-failure.log`,), 'a start that failed\n',);
            /**
             What running the command ended with.
             */
            const ran = await runTheCommand(wordsFor({
              folder: folder.path,
              name,
            },),);
            expect(ran.status,).toBe(REFUSED_STATUS,);
            expect(ran.printed,).toContain(`${name}-start-failure.log`,);
          },
        },),
      ],
    },),
  ],
},);
