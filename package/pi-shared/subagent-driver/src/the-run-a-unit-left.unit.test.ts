/**
 What a unit left in a run's folder, and the line a launcher prints when the run's worktree has gone after its unit ended. The
 point of both is that a run which ran is reported as the run it was: its feed is written from the first moment and its summary
 last, so either says a run happened, whatever its unit's status was and whether or not the shared worktree is still there.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { execFile, } from 'node:child_process';
import { join, } from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  theRunAUnitLeftIn,
  theRunThatOutlivedItsWorktreeLineOf,
  theWorktreeThatWentLineIn,
} from '../dist/final/node/index.mjs';

/**
 `execFile` with a promise result, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 What a call answered, or what it threw, so a call that may refuse is read without a promise method.

 @param call - the call

 @returns its answer, or the error it threw

 @example
 ```ts
 const answer = await settledBy({ call: async () => fileSystemOfARun({ ... }), });
 ```
 */
async function settledBy<Answer>({ call, }: { readonly call: () => Promise<Answer> },): Promise<{
  readonly value?: Answer;
  readonly error?: unknown;
}> {
  try {
    return { value: await call(), };
  }
  catch (error) {
    return { error, };
  }
}

/**
 Waits longer than a report should ever take, and answers what it waited for.

 @param request - how long to wait

 @param request.milliseconds - the bound

 @returns why the caller stopped waiting

 @example
 ```ts
 const answer = await waitForTheTest({ milliseconds: 2_000, });
 ```
 */
async function waitForTheTest({ milliseconds, }: { readonly milliseconds: number },): Promise<{ readonly stillWaiting: true }> {
  await wait(milliseconds,);
  return { stillWaiting: true, };
}

/**
 A folder a test may write in, removed when the test ends.
 */
type AThrowawayFolder = {
  /**
   The folder's path.
   */
  readonly path: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A folder a test may write in, removed when the test ends.

 @returns the folder

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-run-left-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: theRunAUnitLeftIn.name,
      children: [
        it({
          name: 'says no run happened for a run folder a unit never wrote to, and names the files it holds when it holds any',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A run folder the launcher made and no run wrote to.
             */
            const runFolder = join(folder.path, 'a-1',);
            await mkdir(runFolder,);
            expect(await theRunAUnitLeftIn({ runFolder, },),).toEqual({
              ran: false,
              kept: [],
            },);
            await writeFile(join(runFolder, 'unrelated.txt',), 'not a run',);
            expect(await theRunAUnitLeftIn({ runFolder, },),).toEqual({
              ran: false,
              kept: ['unrelated.txt',],
            },);
          },
        },),
        it({
          name: 'says a run happened from its summary alone, and reads how it ended',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A run folder holding the summary a run writes last.
             */
            const runFolder = join(folder.path, 'a-1',);
            await mkdir(runFolder,);
            await writeFile(
              join(runFolder, 'summary.json',),
              JSON.stringify({ endReason: 'pi-exited', },),
            );
            expect(await theRunAUnitLeftIn({ runFolder, },),).toEqual({
              ran: true,
              endReason: 'pi-exited',
              kept: ['summary.json',],
            },);
          },
        },),
        it({
          name: 'says a run happened from its feed alone, which it writes from the first moment and which names no ending',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A run folder holding only the feed, which is what a run killed before its summary leaves.
             */
            const runFolder = join(folder.path, 'a-1',);
            await mkdir(runFolder,);
            await writeFile(
              join(runFolder, 'feed.jsonl',),
              '{"t":0,"kind":"turn_start","n":1}\n',
            );
            await writeFile(join(runFolder, 'stderr.log',), '',);
            expect(await theRunAUnitLeftIn({ runFolder, },),).toEqual({
              ran: true,
              kept: ['feed.jsonl', 'stderr.log',],
            },);
          },
        },),
        it({
          name: 'says no run happened for a folder that is not there, which no run made',
          fn: async () => {
            expect(await theRunAUnitLeftIn({ runFolder: join(scratchRoot(), 'pi-run-left-not-there',), },),).toEqual({
              ran: false,
              kept: [],
            },);
          },
        },),
      ],
    },),
    describe({
      name: theRunThatOutlivedItsWorktreeLineOf.name,
      children: [
        it({
          name: 'says the run ended as its summary says, where what it kept is, and where its branch and commits are',
          fn: async () => {
            /**
             The line a reader gets for a run that ran and failed, whose worktree another launcher removed.
             */
            const line = theRunThatOutlivedItsWorktreeLineOf({
              runName: 'copy-route',
              worktree: '/repo/.claude/worktrees/pi-copy-route',
              runFolder: '/runs/copy-route-4',
              branch: 'fix/copy-route',
              unitStatus: 1,
              left: {
                ran: true,
                endReason: 'pi-exited',
                kept: ['feed.jsonl', 'summary.json',],
              },
            },);
            expect(line,).toContain('the run ended as pi-exited',);
            expect(line,).toContain('status 1',);
            expect(line,).toContain('/repo/.claude/worktrees/pi-copy-route',);
            expect(line,).toContain('copy-route',);
            expect(line,).toContain('/runs/copy-route-4',);
            expect(line,).toContain('feed.jsonl, summary.json',);
            expect(line,).toContain('fix/copy-route',);
            // What is kept is the branch and its commits; nothing claims the worktree's uncommitted files survived, because a
            // removal takes them with it.
            expect(line,).toContain('every commit on it are in the Git directory the worktree named',);
            expect(line,).toContain('what the worktree held that was never committed, including the files Git ignores, is not there any more',);
            expect(line,).not.toContain('is not lost',);
            expect(line,).not.toContain('Nothing of that work is lost',);
            expect(line,).not.toContain('no run happened',);
            expect(line,).not.toContain('the unit did not start',);
          },
        },),
        it({
          name: 'says the run ended before it wrote a summary when its feed is all it left',
          fn: async () => {
            /**
             The line a reader gets when the run's summary never got written.
             */
            const line = theRunThatOutlivedItsWorktreeLineOf({
              runName: 'a',
              worktree: '/repo/.claude/worktrees/pi-a',
              runFolder: '/runs/a-1',
              branch: 'fix/a',
              unitStatus: 137,
              left: {
                ran: true,
                kept: ['feed.jsonl',],
              },
            },);
            expect(line,).toContain('the run ended before it wrote a summary',);
            expect(line,).toContain('status 137',);
            expect(line,).toContain('feed.jsonl',);
            expect(line,).not.toContain('no run happened',);
          },
        },),
      ],
    },),
    describe({
      name: 'a run folder whose summary is a named pipe',
      children: [
        it({
          name: 'is reported without waiting for a writer, and the line says that how the run ended cannot be read',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run's folder, which holds a feed and a summary a concurrent command replaced with a named pipe.
             */
            const runFolder = join(folder.path, 'runs', 'a-1',);
            await mkdir(runFolder, { recursive: true, },);
            await writeFile(join(runFolder, 'feed.jsonl',), '{"t":0,"kind":"turn_start","n":1}\n',);
            await execFileAsync('mkfifo', [join(runFolder, 'summary.json',),], { encoding: 'utf8', timeout: 5_000, },);
            /**
             What the report answered, or the fact that it was still waiting when the test stopped waiting.
             */
            const answer = await Promise.race([
              settledBy({
                call: async function report() {
                  return await theWorktreeThatWentLineIn({
                    worktree: join(folder.path, 'worktree-that-is-gone',),
                    runName: 'a',
                    runFolder,
                    branch: 'fix/a',
                    unitStatus: 1,
                  },);
                },
              },),
              waitForTheTest({ milliseconds: 2_000, },),
            ],);
            expect(answer,).toHaveProperty('value',);
            /**
             The line the report wrote.
             */
            const line = (answer as { readonly value?: { readonly line?: string; }; }).value?.line ?? '';
            expect(line,).toContain('summary is not an ordinary file',);
            expect(line,).toContain('cannot be read',);
            expect(line,).not.toContain('is not lost',);
          },
        },),
      ],
    },),
    describe({
      name: theWorktreeThatWentLineIn.name,
      children: [
        it({
          name: 'answers nothing while the worktree is there, so a run that is going says nothing about a worktree',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A worktree that is there.
             */
            const worktree = join(folder.path, 'worktree',);
            await mkdir(worktree,);
            expect(await theWorktreeThatWentLineIn({
              worktree,
              runName: 'a',
              runFolder: join(folder.path, 'runs', 'a-1',),
              branch: 'fix/a',
              unitStatus: 0,
            },),).toEqual({},);
          },
        },),
        it({
          name: 'says a run happened when its folder holds what a run leaves, and says no run happened when it holds nothing',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The worktree another launcher removed.
             */
            const worktree = join(folder.path, 'worktree',);
            /**
             The run's folder, which holds a summary when a run happened.
             */
            const runFolder = join(folder.path, 'runs', 'a-1',);
            await mkdir(runFolder, { recursive: true, },);
            /**
             The line for a run that left nothing.
             */
            const { line: nothingLeft, } = await theWorktreeThatWentLineIn({
              worktree,
              runName: 'a',
              runFolder,
              branch: 'fix/a',
              unitStatus: 226,
            },);
            expect(nothingLeft,).toContain('the unit did not start',);
            expect(nothingLeft,).toContain('no run happened',);
            expect(nothingLeft,).toContain(runFolder,);
            await writeFile(
              join(runFolder, 'summary.json',),
              JSON.stringify({ endReason: 'pi-exited', },),
            );
            /**
             The line for a run that left its summary.
             */
            const { line: ran, } = await theWorktreeThatWentLineIn({
              worktree,
              runName: 'a',
              runFolder,
              branch: 'fix/a',
              unitStatus: 1,
            },);
            expect(ran,).toContain('the run ended as pi-exited',);
            expect(ran,).toContain('summary.json',);
            expect(ran,).not.toContain('no run happened',);
          },
        },),
      ],
    },),
  ],
},);
