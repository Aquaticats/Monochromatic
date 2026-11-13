/**
 The check as a command: what it prints for each review, the status it answers with, and the restart it writes without starting
 anything.

 The fixtures are folders this test makes and throws away, and the command runs as its own process so its status and its lines are
 what a caller would see. No flag here asks for a start, so no run, unit or Pi is started.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  mkdir,
  mkdtemp,
  readdir,
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
  CHECK_LIMITS_LINE,
  REVIEW_FILE_NAME,
  REVIEW_READING_FILE_NAME,
  REVIEW_STATEMENT_HEADING,
} from '../dist/final/node/index.mjs';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

/**
 The command this file judges, run from source the way its own task runs it.
 */
const COMMAND = new URL('check-a-review-that-ended.ts', import.meta.url,).pathname;

/**
 The change the fixture reviews.

 @returns the review's name

 @example
 ```ts
 const review = aReview();
 ```
 */
function aReview(): string {
  return 'a-round-1';
}

/**
 The commit the fixture reviews.

 @returns the commit's id

 @example
 ```ts
 const commit = aCommit();
 ```
 */
function aCommit(): string {
  return 'abc1234def5678';
}

/**
 The diff the fixture review was given.

 @returns the diff's text

 @example
 ```ts
 const diff = aDiff();
 ```
 */
function aDiff(): string {
  return [
    'diff --git a/package/a/src/one.rs b/package/a/src/one.rs',
    '--- a/package/a/src/one.rs',
    '+++ b/package/a/src/one.rs',
    '@@ -1 +1 @@',
    '-old',
    '+new',
    '',
  ].join('\n',);
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
  const path = await mkdtemp(join(scratchRoot(), 'pi-review-command-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes a review that read the whole diff and ran nothing, with the run and session it left.

 @param folder - the folder everything is written under

 @returns the runs folder, so a test can say no run was added to it

 @example
 ```ts
 const runsFolder = await writeAReviewThatRanNothing(folder.path);
 ```
 */
async function writeAReviewThatRanNothing(folder: string,): Promise<string> {
  const reviewsFolder = join(folder, 'reviews',);
  const runsFolder = join(folder, 'runs',);
  const reviewFolder = join(reviewsFolder, aReview(),);
  const runFolder = join(runsFolder, `review-${aReview()}-1`,);
  const sessionFile = join(folder, 'session.jsonl',);
  await mkdir(join(reviewFolder, 'reproductions',), { recursive: true, },);
  await mkdir(runFolder, { recursive: true, },);
  await writeFile(join(reviewFolder, CHANGE_DIFF_FILE_NAME,), aDiff(),);
  await writeFile(
    join(reviewFolder, REVIEW_READING_FILE_NAME,),
    `${JSON.stringify({
      review: aReview(),
      commit: aCommit(),
      source: 'package/a/src',
      ranges: ['main...fix/a',],
    },)}\n`,
  );
  await writeFile(
    join(reviewFolder, REVIEW_FILE_NAME,),
    `# Review: a change\n\n${REVIEW_STATEMENT_HEADING}\n\n\`\`\`review-statement\n${JSON.stringify({
      commit: aCommit(),
      diff: {
        lines: 6,
        linesRead: 6,
      },
      files: [{
        path: 'package/a/src/one.rs',
        read: 'whole',
      },],
      commands: [{
        command: 'rg --count change package/a/src',
        exit: 0,
      },],
      findings: [],
    }, undefined, 2)}\n\`\`\`\n\n## Findings\n\nNothing.\n`,
  );
  await writeFile(sessionFile, `${JSON.stringify({
    type: 'session',
    version: 3,
    cwd: '/worktrees/pi-review-a',
  },)}\n${JSON.stringify({
    type: 'message',
    message: {
      role: 'assistant',
      content: [{
        type: 'toolCall',
        id: 'c1',
        name: 'bash',
        arguments: { command: 'rg --count change package/a/src', },
      },],
    },
  },)}\n${JSON.stringify({
    type: 'message',
    message: {
      role: 'toolResult',
      toolCallId: 'c1',
      content: [{
        type: 'text',
        text: '3',
      },],
      isError: false,
    },
  },)}\n`,);
  await writeFile(
    join(runFolder, 'summary.json',),
    `${JSON.stringify({
      endReason: 'idle',
      turns: 20,
      sessionStats: { sessionFile, },
      lastAssistantText: 'No defect found.',
    },)}\n`,
  );
  return runsFolder;
}

/**
 Runs the command with the words a test gives it.

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
      status: 0,
      printed: answered.stdout,
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
      status: (typeof failed.code) === 'number' ? failed.code : 1,
      printed: `${String(failed.stdout,)}${String(failed.stderr,)}`,
    };
  }
}

await describe({
  name: '',
  children: [
    describe({
      name: 'check-a-review-that-ended',
      children: [
        it({
          name: 'prints one verdict per review, writes the entry that starts it again, and answers 1 while it needs one',
          fn: async () => {
            await using folder = await makeFolder();
            const runsFolder = await writeAReviewThatRanNothing(folder.path,);
            const ran = await runTheCommand([
              `--review=${aReview()}`,
              `--reviews-folder=${join(folder.path, 'reviews',)}`,
              `--runs-folder=${runsFolder}`,
              '--rules-file=/repo/rules.md',
            ],);
            expect(ran.status,).toBe(1,);
            expect(ran.printed,).toContain(`review-${aReview()}: ran nothing`,);
            expect(ran.printed,).toContain(CHECK_LIMITS_LINE,);
            // The restart's brief, diff, reading and batch list were written, and no run folder was made beside the earlier one.
            const restarted = join(
              folder.path,
              'reviews',
              `${aReview()}-restart-2`,
            );
            expect(await readdir(restarted,),).toContain('brief.md',);
            expect(await readdir(restarted,),).toContain('change.diff',);
            expect(await readdir(runsFolder,),).toEqual([`review-${aReview()}-1`,],);
            const brief = await readFile(join(restarted, 'brief.md',), 'utf8',);
            expect(brief,).toContain('This is the second run of the review',);
            const list = await readFile(join(
              folder.path,
              'reviews',
              aReview(),
              'restart-2-batch.json',
            ), 'utf8',);
            expect(JSON.parse(list,),).toHaveLength(1,);
          },
        },),
        it({
          name: 'says which run is still pending instead of writing a third, and answers 1',
          fn: async () => {
            await using folder = await makeFolder();
            const runsFolder = await writeAReviewThatRanNothing(folder.path,);
            const words = [
              `--review=${aReview()}`,
              `--reviews-folder=${join(folder.path, 'reviews',)}`,
              `--runs-folder=${runsFolder}`,
              '--rules-file=/repo/rules.md',
            ];
            expect((await runTheCommand(words,)).status,).toBe(1,);
            const again = await runTheCommand(words,);
            expect(again.status,).toBe(1,);
            expect(again.printed,).toContain('has not ended, so no later run was written',);
            expect(await readdir(join(folder.path, 'reviews',),),).toEqual([aReview(), `${aReview()}-restart-2`,],);
          },
        },),
        it({
          name: 'answers 0 for a review that is complete, and writes no restart beside it',
          fn: async () => {
            await using folder = await makeFolder();
            const runsFolder = await writeAReviewThatRanNothing(folder.path,);
            // The same review, now with a command that ran a program the worktree built.
            const program = '/worktrees/pi-review-a/package/a/target/debug/a';
            await writeFile(
              join(folder.path, 'reviews', aReview(), REVIEW_FILE_NAME,),
              `# Review: a change\n\n${REVIEW_STATEMENT_HEADING}\n\n\`\`\`review-statement\n${JSON.stringify({
                commit: aCommit(),
                diff: {
                  lines: 6,
                  linesRead: 6,
                },
                files: [{
                  path: 'package/a/src/one.rs',
                  read: 'whole',
                },],
                commands: [{
                  command: `${program} --version`,
                  exit: 0,
                  ranBuiltProgram: program,
                },],
                findings: [],
              }, undefined, 2)}\n\`\`\`\n\n## Findings\n\nNothing.\n`,
            );
            await writeFile(
              join(folder.path, 'session.jsonl',),
              `${JSON.stringify({
                type: 'session',
                version: 3,
                cwd: '/worktrees/pi-review-a',
              },)}\n${JSON.stringify({
                type: 'message',
                message: {
                  role: 'assistant',
                  content: [{
                    type: 'toolCall',
                    id: 'c1',
                    name: 'bash',
                    arguments: { command: `${program} --version`, },
                  },],
                },
              },)}\n`,
            );
            const ran = await runTheCommand([
              `--review=${aReview()}`,
              `--reviews-folder=${join(folder.path, 'reviews',)}`,
              `--runs-folder=${runsFolder}`,
              '--rules-file=/repo/rules.md',
            ],);
            expect(ran.status,).toBe(0,);
            expect(ran.printed,).toContain(`review-${aReview()}: complete`,);
            expect(await readdir(join(folder.path, 'reviews',),),).toEqual([aReview(),],);
          },
        },),
        it({
          name: 'answers 1 and names the missing review when no run of it ever finished',
          fn: async () => {
            await using folder = await makeFolder();
            const ran = await runTheCommand([
              '--review=nothing-here',
              `--reviews-folder=${join(folder.path, 'reviews',)}`,
              `--runs-folder=${join(folder.path, 'runs',)}`,
              '--rules-file=/repo/rules.md',
            ],);
            expect(ran.status,).toBe(1,);
            expect(ran.printed,).toContain('review-nothing-here: no review file',);
            expect(ran.printed,).toContain('no finished run of review-nothing-here is in',);
          },
        },),
      ],
    },),
  ],
},);
