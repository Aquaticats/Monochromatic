/**
 The step a launcher takes after its unit ended: which processes are the run's own work, the note a resumed agent gets, the words
 it is resumed with, and the bound on the wait.

 The folder the run's own work is read from is the seam: a test writes its own process folder of working-folder links and never
 reads this machine's process table.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  symlink,
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
  WAKE_LIMIT,
  WORK_POLL_MILLISECONDS,
  type ProcessAtAFolder,
  isOwnFolder,
  ownComponentOf,
  processesAtTheirFolders,
  processesStillWorkingFor,
  wakePromptOf,
  wakeResumeWordsOf,
  wakeIfItsOwnWorkOutlivesItsTurn,
  workWaitMillisecondsOf,
} from '../dist/final/node/index.mjs';

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
  const path = await mkdtemp(join(scratchRoot(), 'pi-wake-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes one process into a process folder: the folder it works in and the program it runs.

 @param request - the process folder, the process's number and the folder it works in

 @example
 ```ts
 await writeProcess({ folder: processes, pid: '12', worksIn: '/repo/.claude/worktrees/pi-a' });
 ```
 */
async function writeProcess({
  folder,
  pid,
  worksIn,
  command = 'cargo',
}: {
  readonly folder: string;
  readonly pid: string;
  readonly worksIn: string;
  readonly command?: string;
}): Promise<void> {
  await mkdir(join(folder, pid,), { recursive: true, },);
  await symlink(worksIn, join(folder, pid, 'cwd',),);
  await writeFile(join(folder, pid, 'comm',), `${command}\n`,);
}

await describe({
  name: '',
  children: [
    describe({
      name: ownComponentOf.name,
      children: [
        it({
          name: 'reads the name a folder of a review or an audit carries, and leaves every other run name whole',
          fn: async () => {
            expect(ownComponentOf({ runName: 'kimi-review-a-round-8', },),).toBe('a-round-8',);
            expect(ownComponentOf({ runName: 'kimi-audit-b', },),).toBe('b',);
            expect(ownComponentOf({ runName: 'copy-route', },),).toBe('copy-route',);
          },
        },),
      ],
    },),
    describe({
      name: isOwnFolder.name,
      children: [
        it({
          name: 'answers for the whole worktree, a folder inside it, and a folder carrying the run name as one of its own components',
          fn: async () => {
            expect(isOwnFolder({
              folder: '/repo/.claude/worktrees/pi-a',
              ownFolders: ['/repo/.claude/worktrees/pi-a',],
              ownComponent: 'a',
            },),).toBe(true,);
            expect(isOwnFolder({
              folder: '/repo/.claude/worktrees/pi-a/package/x',
              ownFolders: ['/repo/.claude/worktrees/pi-a',],
              ownComponent: 'a',
            },),).toBe(true,);
            expect(isOwnFolder({
              folder: '/home/user/temp/agent/kimi-reviews/a-round-8',
              ownFolders: ['/repo/.claude/worktrees/pi-kimi-review-a-round-8',],
              ownComponent: 'a-round-8',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'never answers for a folder that merely holds the run name inside a longer name, which tied launchers to another agent reviews',
          fn: async () => {
            /**
             The run of the five round 211 re-merge launchers that matched the Kimi review of its own branch.
             */
            const folder = '/repo/.claude/worktrees/pi-kimi-review-kimi-findings-merge-asides-never-lose-a-save-round-208';
            expect(isOwnFolder({
              folder,
              ownFolders: ['/repo/.claude/worktrees/pi-kimi-findings-merge-asides-never-lose-a-save',],
              ownComponent: 'kimi-findings-merge-asides-never-lose-a-save',
            },),).toBe(false,);
            expect(isOwnFolder({
              folder: '/home/user/temp/agent/kimi-reviews/a',
              ownFolders: [],
              ownComponent: 'a',
            },),).toBe(true,);
            expect(isOwnFolder({
              folder: '/home/user/temp/agent/kimi-reviews/a-round-208',
              ownFolders: [],
              ownComponent: 'a',
            },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: processesAtTheirFolders.name,
      children: [
        it({
          name: 'reads the folder every process of a process folder works in and the program it runs, and leaves a process that has none out',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The process folder a test writes.
             */
            const processes = join(folder.path, 'proc',);
            await writeProcess({
              folder: processes,
              pid: '12',
              worksIn: '/repo/.claude/worktrees/pi-a',
            },);
            await writeProcess({
              folder: processes,
              pid: '13',
              worksIn: '/elsewhere',
              command: 'sleep',
            },);
            await mkdir(join(processes, '14',), { recursive: true, },);
            await mkdir(join(processes, 'not-a-number',), { recursive: true, },);
            /**
             What each readable process is, smallest number first.
             */
            const read = (await processesAtTheirFolders({ processFolder: processes, },))
              .map(function pidOf(process,) {
                return process.pid;
              },)
              .toSorted(function smallestFirst(
                left,
                right,
              ) {
                return left - right;
              },);
            expect(read,).toEqual([
              12,
              13,
            ],);
            expect(await processesAtTheirFolders({ processFolder: join(folder.path, 'not-a-folder',), },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: processesStillWorkingFor.name,
      children: [
        it({
          name: 'names the run own work and its own folder, leaves another agent folder whose name holds this run name out, and never names the launcher itself',
          fn: async () => {
            /**
             The processes a machine might report.
             */
            const processes: readonly ProcessAtAFolder[] = [
              {
                pid: 12,
                folder: '/repo/.claude/worktrees/pi-a',
                command: 'cargo',
              },
              {
                pid: 13,
                folder: '/runs/a-1/reproductions',
                command: 'node',
              },
              {
                pid: 14,
                folder: '/repo/.claude/worktrees/pi-kimi-review-a-round-208',
                command: 'cargo',
              },
              {
                pid: 15,
                folder: '/home/user/temp/agent/kimi-reviews/a-round-208',
                command: 'node',
              },
            ];
            expect(processesStillWorkingFor({
              runName: 'a',
              worktree: '/repo/.claude/worktrees/pi-a',
              runFolder: '/runs/a-1',
              processes,
              excludingPid: 15,
            },),).toEqual(['12 cargo', '13 node',],);
            expect(processesStillWorkingFor({
              runName: 'kimi-review-a-round-208',
              worktree: '/repo/.claude/worktrees/pi-kimi-review-a-round-208',
              runFolder: '/runs/kimi-review-a-round-208-1',
              processes,
              excludingPid: 0,
            },),).toEqual(['14 cargo', '15 node',],);
          },
        },),
      ],
    },),
    describe({
      name: wakePromptOf.name,
      children: [
        it({
          name: 'says what the run left running when it still runs, and says it has ended when it has, and asks the agent to continue',
          fn: async () => {
            expect(wakePromptOf({
              stillWorking: ['12 cargo',],
              wallSeconds: 600,
            },),).toBe('Work you left running when your turn ended is still running after 600 s: 12 cargo. Read its results and continue your task from where you stopped. Ending your turn again while something runs is fine; you are resumed when it ends.\n',);
            expect(wakePromptOf({
              stillWorking: [],
              wallSeconds: 600,
            },),).toContain('The work you left running when your turn ended has now ended.',);
          },
        },),
      ],
    },),
    describe({
      name: wakeResumeWordsOf.name,
      children: [
        it({
          name: 'replaces the prompt and the session, drops the run number so the resumed run writes a folder of its own, and keeps every other word',
          fn: async () => {
            expect(wakeResumeWordsOf({
              commandLine: [
                '--name=a',
                '--branch=feat/a',
                '--prompt=/runs/a-prompt.md',
                '--run=1',
                '--session=/sessions/old.jsonl',
                '--wall=600',
              ],
              wakePrompt: '/runs/a-1/wake-prompt.md',
              sessionFile: '/sessions/new.jsonl',
            },),).toEqual([
              '--name=a',
              '--branch=feat/a',
              '--wall=600',
              '--prompt=/runs/a-1/wake-prompt.md',
              '--session=/sessions/new.jsonl',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: wakeIfItsOwnWorkOutlivesItsTurn.name,
      children: [
        it({
          name: 'does nothing for a run that left no work of its own running',
          fn: async () => {
            /**
             The lines the step reported.
             */
            const reported: string[] = [];
            expect(await wakeIfItsOwnWorkOutlivesItsTurn({
              runName: 'a',
              wallSeconds: 600,
              wakesSoFar: 0,
              readOwnWork: async function noWork() {
                return [];
              },
              report: function report(line,) {
                reported.push(line,);
              },
              wait: async function noWait() {
                // Nothing is waited for when the first look finds nothing.
              },
              now: Date.now,
              resume: async function noResume() {
                throw new Error('a run with no work of its own was resumed',);
              },
            },),).toEqual({ step: 'nothing-to-wake', },);
            expect(reported,).toEqual([],);
          },
        },),
        it({
          name: 'waits for the run own work and resumes its session once it ends, and the note it leaves asks for no owner',
          fn: async () => {
            /**
             What each look at the run's own work answers, one after another.
             */
            const looks: readonly (readonly string[])[] = [['12 cargo',], ['12 cargo',], [],];
            /**
             Which look is next, and how the wait passes time, both of which the seams below move.
             */
            const clock = { at: 0, next: 0, };
            /**
             The lines the step reported.
             */
            const reported: string[] = [];
            /**
             What resuming the session was asked with.
             */
            const asked: (readonly string[])[] = [];
            expect(await wakeIfItsOwnWorkOutlivesItsTurn({
              runName: 'a',
              wallSeconds: 600,
              wakesSoFar: 0,
              readOwnWork: async function nextLook() {
                /**
                 What this look answers.
                 */
                const answer = looks[clock.next] ?? [];
                clock.next += 1;
                return answer;
              },
              report: function report(line,) {
                reported.push(line,);
              },
              wait: async function moveTheClock(milliseconds,) {
                clock.at += milliseconds;
              },
              now: function now() {
                return clock.at;
              },
              resume: async function resume({ stillWorking, },) {
                asked.push(stillWorking,);
                return {
                  resumed: true,
                  status: 0,
                };
              },
            },),).toEqual({
              step: 'resumed',
              status: 0,
            },);
            expect(reported[0],).toContain('waiting for it, then resuming the session',);
            expect(asked,).toEqual([[],],);
            /**
             The note the resumed agent reads.
             */
            const note = wakePromptOf({
              stillWorking: [],
              wallSeconds: 600,
            },);
            expect(note,).not.toContain('owner',);
            expect(note,).not.toContain('the user said',);
            expect(note,).not.toContain('I ',);
            expect(note,).not.toContain(' me ',);
            expect(note,).not.toContain(' my ',);
          },
        },),
        it({
          name: 'keeps the worktree when the run has been woken as often as it may be, and when its session cannot be resumed',
          fn: async () => {
            /**
             What the step answers for a run woken as often as it may be.
             */
            const tooManyWakes = await wakeIfItsOwnWorkOutlivesItsTurn({
              runName: 'a',
              wallSeconds: 600,
              wakesSoFar: WAKE_LIMIT,
              readOwnWork: async function ownWork() {
                return ['12 cargo',];
              },
              report: function report() {
                // Nothing to report in this test.
              },
              wait: async function noWait() {
                throw new Error('a run woken as often as it may be was waited for',);
              },
              now: Date.now,
              resume: async function noResume() {
                throw new Error('a run woken as often as it may be was resumed',);
              },
            },);
            expect(tooManyWakes,).toEqual({
              step: 'work-still-runs',
              ownWork: ['12 cargo',],
            },);
            /**
             What each look answers for the run whose summary names no session: the work is there, then it ends.
             */
            const looks: readonly (readonly string[])[] = [['12 cargo',], [],];
            /**
             Which look is next, which is a field of an object so the seam below can move it.
             */
            const next = { at: 0, };
            /**
             What the step answers for a run whose summary names no session.
             */
            const noSession = await wakeIfItsOwnWorkOutlivesItsTurn({
              runName: 'a',
              wallSeconds: 600,
              wakesSoFar: 0,
              readOwnWork: async function nextLook() {
                /**
                 What this look answers.
                 */
                const answer = looks[next.at] ?? [];
                next.at += 1;
                return answer;
              },
              report: function report() {
                // Nothing to report in this test.
              },
              wait: async function noWait() {
                // The first look already found the work, and the second found none.
              },
              now: Date.now,
              resume: async function resume() {
                return {
                  resumed: false,
                  problem: 'a-1 names no session file',
                };
              },
            },);
            expect(noSession,).toEqual({
              step: 'could-not-resume',
              ownWork: [],
              problem: 'a-1 names no session file',
            },);
          },
        },),
      ],
    },),
    describe({
      name: workWaitMillisecondsOf.name,
      children: [
        it({
          name: 'bounds the wait for a run own work by that run own wall clock',
          fn: async () => {
            expect(workWaitMillisecondsOf({ wallSeconds: 600, },),).toBe(600_000,);
            expect(WAKE_LIMIT,).toBeGreaterThan(0,);
            expect(WORK_POLL_MILLISECONDS,).toBeGreaterThan(0,);
          },
        },),
      ],
    },),
  ],
},);
