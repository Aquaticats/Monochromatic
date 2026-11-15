/**
 The settings of one run: the number its folder takes, the turn cap the finished runs give it, the unit it is started as, the
 worktree it works in, and the session file it resumes. Nothing here starts a unit, a worktree or a process.

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
  type CommandOutput,
  BASE_OF_A_RUN_NAMING_NONE,
  IDLE_SECONDS_OF_A_RUN,
  GitCommandFailedError,
  NEURALWATT_STANDARD,
  NO_FINISHED_RUN,
  NO_WORKTREE_BRANCH,
  RUNS_OF_ONE_KIND_TO_TRUST,
  SessionFileUnreadableError,
  TURN_CAP_MARGIN,
  UNIT_GRACE_SECONDS,
  UNIT_IO_WEIGHT,
  WorktreeFolderOccupiedError,
  type GitRunner,
  finishedTurnsIn,
  kindOfRun,
  WORKTREE_ADD_TIMEOUT_MILLISECONDS,
  RunFolderTakenError,
  RunNumbersExhaustedError,
  WorktreeTakenError,
  claimRunFolder,
  makeWorktree,
  nextRunNumber,
  worktreeProofFileOf,
  parseWorktreeLauncherOptions,
  removeWorktree,
  runtimeMaxSecondsOf,
  sessionFileInSummary,
  sessionFileOfRun,
  turnCapSizing,
  unitCommandOf,
  worktreeFolderOf,
  worktreeOutcomeOf,
} from '../dist/final/node/index.mjs';

/**
 A command's answer, as a stand-in gives it.

 @param status - the status the command ended with

 @param stdout - what it printed

 @returns the answer

 @example
 ```ts
 const answered = answered(0, '');
 ```
 */
function answered(status: number, stdout = '',): CommandOutput {
  return {
    status,
    stdout,
    stderr: '',
  };
}

/**
 A git stand-in that answers every question with the same output and records what it was asked.

 @param answers - what each question is answered with, in order, or the same answer for all

 @returns the runner, and what it was asked

 @example
 ```ts
 const git = standInGit([answered(0, '')]);
 ```
 */
function standInGit(answers: readonly CommandOutput[],): {
  readonly git: GitRunner;
  readonly asked: { readonly words: readonly string[]; readonly folder: string; readonly timeoutMilliseconds?: number }[];
} {
  /**
   What git was asked.
   */
  const asked: { readonly words: readonly string[]; readonly folder: string; readonly timeoutMilliseconds?: number }[] = [];
  return {
    asked,
    git: async function gitIn(request,) {
      asked.push({
        words: request.words,
        folder: request.folder,
        ...(request.timeoutMilliseconds === undefined ? {} : { timeoutMilliseconds: request.timeoutMilliseconds, }),
      },);
      return answers[Math.min(asked.length, answers.length) - 1] ?? answered(0,);
    },
  };
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
  const path = await mkdtemp(join(scratchRoot(), 'pi-run-settings-',),);
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
      name: nextRunNumber.name,
      children: [
        it({
          name: 'takes the next number after every folder of that name, so a restart never writes over an earlier run',
          fn: async () => {
            expect(nextRunNumber({
              runName: 'copy-route',
              entries: ['copy-route-1', 'copy-route-4', 'other-9', 'copy-route-x', 'copy-route-'],
            },),).toBe(5,);
          },
        },),
        it({
          name: 'refuses to name a number after the largest a run can be counted to, which adding one would leave unchanged',
          fn: async () => {
            expect(nextRunNumber({
              runName: 'copy-route',
              entries: [`copy-route-${String(Number.MAX_SAFE_INTEGER - 1,)}`,],
            },),).toBe(Number.MAX_SAFE_INTEGER,);
            expect(() => nextRunNumber({
              runName: 'copy-route',
              entries: [`copy-route-${String(Number.MAX_SAFE_INTEGER,)}`,],
            },),).toThrow(RunNumbersExhaustedError,);
            try {
              nextRunNumber({
                runName: 'copy-route',
                entries: [`copy-route-${String(Number.MAX_SAFE_INTEGER,)}`,],
              },);
            }
            catch (error) {
              // The error names the folder that has nowhere to go, and what a person does about it.
              expect(String(error,),).toContain(`copy-route-${String(Number.MAX_SAFE_INTEGER,)}`,);
              return;
            }
            throw new Error('a number that cannot rise was named',);
          },
        },),
        it({
          name: 'takes the first number when no folder of that name is there, and reads a name inside another name as that other run',
          fn: async () => {
            expect(nextRunNumber({
              runName: 'copy-route',
              entries: [],
            },),).toBe(1,);
            expect(nextRunNumber({
              runName: 'copy',
              entries: ['copy-route-4',],
            },),).toBe(1,);
          },
        },),
      ],
    },),
    describe({
      name: claimRunFolder.name,
      children: [
        it({
          name: 'makes the runs folder when it is not there, and claims the first free number of the run',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The runs folder, which no run has written into yet.
             */
            const runsFolder = join(
              folder.path,
              'runs',
            );
            expect(await claimRunFolder({
              runsFolder,
              runName: 'copy-route',
            },),).toEqual({
              runNumber: 1,
              runFolder: join(
                runsFolder,
                'copy-route-1',
              ),
            },);
            expect((await readdir(runsFolder,)).toSorted(),).toEqual(['copy-route-1',],);
          },
        },),
        it({
          name: 'gives two starts of one name that read the runs folder at the same instant a folder each, the one that loses taking the number after it',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The runs folder both starts list before either makes its folder.
             */
            const runsFolder = join(
              folder.path,
              'runs',
            );
            /**
             What both starts claimed, each its own call.
             */
            const claimed = await Promise.all([
              claimRunFolder({
                runsFolder,
                runName: 'copy-route',
              },),
              claimRunFolder({
                runsFolder,
                runName: 'copy-route',
              },),
            ],);
            /**
             The numbers both starts claimed, smallest first.
             */
            const numbers = claimed.map(function number(one,) {
              return one.runNumber;
            },)
              .toSorted(function lowerFirst(left, right,) {
                return left - right;
              },);
            expect(numbers,).toEqual([
              1,
              2,
            ],);
            expect(new Set(claimed.map(function folderOf(one,) {
              return one.runFolder;
            },)).size,).toBe(2,);
            expect((await readdir(runsFolder,)).toSorted(),).toEqual(['copy-route-1', 'copy-route-2',],);
          },
        },),
        it({
          name: 'refuses to claim a number at the largest a run can be counted to, where no number follows',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The runs folder, whose highest number has nowhere left to go.
             */
            const runsFolder = join(
              folder.path,
              'runs',
            );
            await mkdir(runsFolder, { recursive: true, },);
            await mkdir(join(
              runsFolder,
              `copy-route-${String(Number.MAX_SAFE_INTEGER - 1,)}`,
            ),);
            /**
             A number claimed from a folder whose next number is the largest a run can be counted to, which is still a number.
             */
            const claimed = await claimRunFolder({
              runsFolder,
              runName: 'copy-route',
            },);
            expect(claimed.runNumber,).toBe(Number.MAX_SAFE_INTEGER,);
            await expect(claimRunFolder({
              runsFolder,
              runName: 'copy-route',
            },),).rejects.toThrow(RunNumbersExhaustedError,);
          },
        },),
        it({
          name: 'refuses a number the caller named that another start holds, and claims one the caller named that is free',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The runs folder the starts write into.
             */
            const runsFolder = join(
              folder.path,
              'runs',
            );
            expect((await claimRunFolder({
              runsFolder,
              runName: 'copy-route',
              wantedNumber: 4,
            },)).runNumber,).toBe(4,);
            try {
              await claimRunFolder({
                runsFolder,
                runName: 'copy-route',
                wantedNumber: 4,
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(RunFolderTakenError,);
              expect(String(error,),).toContain('copy-route-4',);
              // The message says how to start the run instead, because a run folder one run holds is never written into again.
              expect(String(error,),).toContain('without --run=4',);
              expect((await claimRunFolder({
                runsFolder,
                runName: 'copy-route',
                wantedNumber: 9,
              },)).runNumber,).toBe(9,);
              return;
            }
            throw new Error('a number the caller named was claimed twice',);
          },
        },),
      ],
    },),
    describe({
      name: kindOfRun.name,
      children: [
        it({
          name: 'marks an audit by its prefix, a fix round by the runs it was named for, and everything else as other work',
          fn: async () => {
            expect(kindOfRun({ runName: 'sol-audit-1', },),).toBe('audit',);
            expect(kindOfRun({ runName: 'kimi-never-stuck-audit-findings-fixed-2', },),).toBe('fix round',);
            expect(kindOfRun({ runName: 'copy-route', },),).toBe('other',);
          },
        },),
      ],
    },),
    describe({
      name: turnCapSizing.name,
      children: [
        it({
          name: 'reads this kind alone once enough runs of it have finished',
          fn: async () => {
            /**
             Enough finished fix rounds, one of them the longest, and one much longer run of another kind.
             */
            const finished = [
              ...Array.from(
                { length: RUNS_OF_ONE_KIND_TO_TRUST, },
                function aFixRound(_unused, index,) {
                  return {
                    kind: 'fix round',
                    turns: 10 + index,
                  };
                },
              ),
              {
                kind: 'other',
                turns: 400,
              },
            ];
            expect(turnCapSizing({
              runName: 'a-findings-fixed-1',
              finished,
            },),).toEqual({
              longestTurns: 10 + (RUNS_OF_ONE_KIND_TO_TRUST - 1),
              sampleRuns: RUNS_OF_ONE_KIND_TO_TRUST,
              sampleKind: 'fix round',
            },);
          },
        },),
        it({
          name: 'reads every kind together while too few runs of this kind have finished to say anything',
          fn: async () => {
            expect(turnCapSizing({
              runName: 'copy-route',
              finished: [
                {
                  kind: 'fix round',
                  turns: 12,
                },
                {
                  kind: 'other',
                  turns: 40,
                },
              ],
            },),).toEqual({
              longestTurns: 40,
              sampleRuns: 2,
              sampleKind: 'every kind',
            },);
            expect(TURN_CAP_MARGIN,).toBeGreaterThan(1,);
          },
        },),
        it({
          name: 'answers that no finished run says anything when the folder holds none',
          fn: async () => {
            expect(turnCapSizing({
              runName: 'copy-route',
              finished: [],
            },),).toBe(NO_FINISHED_RUN,);
          },
        },),
      ],
    },),
    describe({
      name: finishedTurnsIn.name,
      children: [
        it({
          name: 'reads the turns of every run that ended idle, and leaves out one that ended otherwise, one with no summary, and a plain file',
          fn: async () => {
            await using folder = await makeFolder();
            await mkdir(join(folder.path, 'a-findings-fixed-1',), { recursive: true, },);
            await writeFile(join(folder.path, 'a-findings-fixed-1', 'summary.json',), JSON.stringify({
              endReason: 'idle',
              turns: 120,
            },),);
            await mkdir(join(folder.path, 'b-1',), { recursive: true, },);
            await writeFile(join(folder.path, 'b-1', 'summary.json',), JSON.stringify({
              endReason: 'wall-clock',
              turns: 300,
            },),);
            await mkdir(join(folder.path, 'c-1',), { recursive: true, },);
            await writeFile(join(folder.path, 'plain-file',), 'not a run',);
            expect(await finishedTurnsIn({ runsFolder: folder.path, },),).toEqual([{
              kind: 'fix round',
              turns: 120,
            },],);
          },
        },),
      ],
    },),
    describe({
      name: sessionFileInSummary.name,
      children: [
        it({
          name: 'reads the session file a summary names',
          fn: async () => {
            expect(sessionFileInSummary({
              shown: JSON.stringify({ sessionStats: { sessionFile: '/sessions/a.jsonl', }, },),
              runFolder: 'a-1',
            },),).toBe('/sessions/a.jsonl',);
          },
        },),
        it({
          name: 'refuses a summary that names no session file, and one that is no JSON object, naming the run to read',
          fn: async () => {
            expect(() => sessionFileInSummary({
              shown: JSON.stringify({ sessionStats: {}, },),
              runFolder: 'a-1',
            },),).toThrow(SessionFileUnreadableError,);
            expect(() => sessionFileInSummary({
              shown: '[]',
              runFolder: 'a-1',
            },),).toThrow(SessionFileUnreadableError,);
          },
        },),
        it({
          name: 'reads the summary of a run in a runs folder, and says which file it could not read',
          fn: async () => {
            await using folder = await makeFolder();
            await mkdir(join(folder.path, 'a-1',), { recursive: true, },);
            await writeFile(join(folder.path, 'a-1', 'summary.json',), JSON.stringify({ sessionStats: { sessionFile: '/s.jsonl', }, },),);
            expect(await sessionFileOfRun({
              runsFolder: folder.path,
              runFolder: 'a-1',
            },),).toBe('/s.jsonl',);
            try {
              await sessionFileOfRun({
                runsFolder: folder.path,
                runFolder: 'gone-1',
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(SessionFileUnreadableError,);
              expect(String(error,),).toContain('gone-1',);
              return;
            }
            throw new Error('a run whose summary is missing answered with a session file',);
          },
        },),
      ],
    },),
    describe({
      name: unitCommandOf.name,
      children: [
        it({
          name: 'starts the driver as a unit with every cap, the mise PATH and the home it runs with',
          fn: async () => {
            /**
             What starting one run answers.
             */
            const command = unitCommandOf({
              runName: 'copy-route',
              runNumber: 4,
              cpuPercent: 800,
              memory: '12G',
              taskCount: 3_000,
              wallSeconds: 7_200,
              pathOfMiseEnvironment: '/mise/bin:/usr/bin',
              home: '/home/a',
              node: '/mise/bin/node',
              driverEntry: '/pkg/src/drive-pi-subagent.ts',
              runFolder: '/runs/copy-route-4',
              workingFolder: '/repo/.claude/worktrees/pi-copy-route',
              promptFile: '/runs/copy-route-prompt.md',
              provider: 'neuralwatt',
              model: 'deepseek-v4.1-flash',
              idleSeconds: IDLE_SECONDS_OF_A_RUN,
              turnCap: 400,
            },);
            expect(command.command,).toBe('systemd-run',);
            expect(command.words,).toEqual([
              '--user',
              '--wait',
              '--unit=pi-copy-route@4.service',
              '--collect',
              `--property=RuntimeMaxSec=${String(7_200 + UNIT_GRACE_SECONDS,)}`,
              '--property=MemoryMax=12G',
              '--property=CPUQuota=800%',
              `--property=TasksMax=${String(3_000,)}`,
              `--property=IOWeight=${String(UNIT_IO_WEIGHT,)}`,
              '--setenv=PATH=/mise/bin:/usr/bin',
              '--setenv=HOME=/home/a',
              '/mise/bin/node',
              '/pkg/src/drive-pi-subagent.ts',
              '--run-folder=/runs/copy-route-4',
              '--working-folder=/repo/.claude/worktrees/pi-copy-route',
              '--prompt-file=/runs/copy-route-prompt.md',
              '--provider=neuralwatt',
              '--model=deepseek-v4.1-flash',
              '--wall-seconds=7200',
              `--idle-seconds=${String(IDLE_SECONDS_OF_A_RUN,)}`,
              '--turn-cap=400',
            ],);
          },
        },),
        it({
          name: 'hands the driver the session file a resumed run needs',
          fn: async () => {
            expect(unitCommandOf({
              runName: 'a',
              runNumber: 2,
              cpuPercent: 1,
              memory: '1G',
              taskCount: 1,
              wallSeconds: 1,
              pathOfMiseEnvironment: '/bin',
              home: '/home/a',
              node: '/bin/node',
              driverEntry: '/pkg/src/drive-pi-subagent.ts',
              runFolder: '/runs/a-2',
              workingFolder: '/repo',
              promptFile: '/runs/a-prompt.md',
              provider: 'neuralwatt',
              model: 'deepseek-v4.1-flash',
              idleSeconds: 1,
              turnCap: 1,
              session: '/sessions/a.jsonl',
            },).words,).toContain('--session=/sessions/a.jsonl',);
          },
        },),
        it({
          name: 'lets the unit outlive the run only by what the driver needs to put it away',
          fn: async () => {
            expect(runtimeMaxSecondsOf({ wallSeconds: 60, },),).toBe(60 + UNIT_GRACE_SECONDS,);
          },
        },),
      ],
    },),
    describe({
      name: worktreeFolderOf.name,
      children: [
        it({
          name: 'puts a run worktree where every worktree of the repository lives, named after the run',
          fn: async () => {
            expect(worktreeFolderOf({
              checkoutFolder: '/repo',
              runName: 'copy-route',
            },),).toBe('/repo/.claude/worktrees/pi-copy-route',);
          },
        },),
      ],
    },),
    describe({
      name: makeWorktree.name,
      children: [
        it({
          name: 'adds a worktree on the branch it names when the branch exists, and starts one at the base it names when the branch is new',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The commit a branch that does not exist yet starts at.
             */
            const base = 'fix/a-reviewed-branch';
            /**
             The proof every making writes.
             */
            const claim = {
              runName: 'a',
              runFolder: join(
                folder.path,
                'runs',
                'a-1',
              ),
            };
            /**
             A git that says the branch exists, then makes the worktree.
             */
            const existing = standInGit([
              answered(0, 'refs/heads/feat/a\n'),
              answered(0, ''),
              answered(0, ''),
            ],);
            expect(await makeWorktree({
              git: existing.git,
              checkoutFolder: folder.path,
              worktree: join(folder.path, '.claude', 'worktrees', 'pi-a',),
              branch: 'feat/a',
              base,
              claim,
            },),).toContain('existing branch feat/a',);
            expect(existing.asked[2]?.words,).toEqual([
              'worktree',
              'add',
              join(folder.path, '.claude', 'worktrees', 'pi-a',),
              'feat/a',
            ],);
            /**
             A git that says the branch is new, then makes the worktree.
             */
            const fresh = standInGit([
              answered(1, ''),
              answered(0, ''),
              answered(0, ''),
            ],);
            expect(await makeWorktree({
              git: fresh.git,
              checkoutFolder: folder.path,
              worktree: join(folder.path, '.claude', 'worktrees', 'pi-b',),
              branch: 'feat/b',
              base,
              claim,
            },),).toContain(`new branch feat/b at ${base}`,);
            expect(fresh.asked[2]?.words,).toContain('-b',);
            expect(fresh.asked[2]?.words.at(-1),).toBe(base,);
          },
        },),
        it({
          name: 'writes the proof of this making before it asks git, and again once the making finished, naming the run folder and the branch',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The worktree this making makes.
             */
            const worktree = join(folder.path, '.claude', 'worktrees', 'pi-a',);
            /**
             The proof file beside it.
             */
            const proofFile = worktreeProofFileOf({ worktree, },);
            /**
             What the proof said while git was asked to make the worktree, which is what a reader finds if this start is killed.
             */
            const duringTheAdd: string[] = [];
            /**
             A git that says the branch is new and reports the proof's state as it is asked for the worktree.
             */
            async function gitIn(request: Parameters<GitRunner>[0],): Promise<CommandOutput> {
              if ((request.words[0] === 'worktree') && (request.words[1] === 'add')) {
                duringTheAdd.push(await readFile(proofFile, 'utf8',),);
              }
              return ((request.words[0] === 'rev-parse') ? answered(1, '') : answered(0, ''));
            }
            await makeWorktree({
              git: gitIn,
              checkoutFolder: folder.path,
              worktree,
              branch: 'feat/a',
              base: BASE_OF_A_RUN_NAMING_NONE,
              claim: {
                runName: 'a',
                runFolder: join(folder.path, 'runs', 'a-1',),
              },
            },);
            expect(duringTheAdd.length,).toBe(1,);
            expect(duringTheAdd[0],).toContain('"state": "making"',);
            expect(duringTheAdd[0],).toContain('"runFolder": "a-1"',);
            expect(duringTheAdd[0],).toContain('"branch": "feat/a"',);
            /**
             What the proof says once the making finished.
             */
            const finished = JSON.parse(await readFile(proofFile, 'utf8',),) as { readonly state?: unknown };
            expect(finished.state,).toBe('made',);
          },
        },),
        it({
          name: 'tries a worktree add again after a killed attempt, clearing only what this start own making left, and asks for a bound of its own',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A git whose first add is killed and whose second succeeds.
             */
            const git = standInGit([
              answered(1, ''),
              answered(0, ''),
              {
                status: (-1),
                stdout: '',
                stderr: '',
                problem: 'did not answer within 120000 milliseconds',
              },
              answered(0, ''),
              answered(0, 'refs/heads/feat/a\n'),
              answered(0, ''),
              answered(0, ''),
            ],);
            /**
             The worktree the second attempt makes.
             */
            const worktree = join(folder.path, '.claude', 'worktrees', 'pi-a',);
            /**
             The line the launcher prints.
             */
            const line = await makeWorktree({
              git: git.git,
              checkoutFolder: folder.path,
              worktree,
              branch: 'feat/a',
              base: 'main',
              claim: {
                runName: 'a',
                runFolder: join(folder.path, 'runs', 'a-1',),
              },
            },);
            expect(line,).toContain('new branch feat/a at main',);
            // Each attempt writes the proof of this making and prunes the registration a killed add may have left, and the killed
            // attempt's leftover is cleared only because its proof is this start's own.
            expect(git.asked.map(function question(ask,) {
              return ask.words.join(' ',);
            },),).toEqual([
              'rev-parse --verify --quiet refs/heads/feat/a',
              'worktree prune --expire=now',
              `worktree add -b feat/a ${worktree} main`,
              'worktree prune --expire=now',
              // The killed attempt made the branch, so the second attempt takes it rather than asking for a new one.
              'rev-parse --verify --quiet refs/heads/feat/a',
              'worktree prune --expire=now',
              `worktree add ${worktree} feat/a`,
            ],);
            expect(git.asked[2]?.timeoutMilliseconds,).toBe(WORKTREE_ADD_TIMEOUT_MILLISECONDS,);
          },
        },),
        it({
          name: 'answers the last of its attempts when every worktree add is killed, and removes the branch this start made',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A git whose every add is killed, whose prunes answer, and whose branch is there because a killed add made it.
             */
            const git = standInGit([
              answered(1, ''),
              answered(0, ''),
              {
                status: (-1),
                stdout: '',
                stderr: '',
                problem: 'did not answer within 900000 milliseconds',
              },
              answered(0, ''),
              answered(0, 'refs/heads/feat/a\n'),
              answered(0, ''),
              {
                status: (-1),
                stdout: '',
                stderr: '',
                problem: 'did not answer within 900000 milliseconds',
              },
              answered(0, 'refs/heads/feat/a\n'),
              answered(0, ''),
            ],);
            await expect(makeWorktree({
              git: git.git,
              checkoutFolder: folder.path,
              worktree: join(folder.path, '.claude', 'worktrees', 'pi-a',),
              branch: 'feat/a',
              base: 'main',
              claim: {
                runName: 'a',
                runFolder: join(folder.path, 'runs', 'a-1',),
              },
            },),).rejects.toThrow('did not answer within 900000 milliseconds',);
            // The branch no worktree holds is deleted the way git refuses when it holds anything unmerged, never by force.
            expect(git.asked.at(-1)?.words,).toEqual([
              'branch',
              '--delete',
              'feat/a',
            ],);
          },
        },),
        it({
          name: 'keeps a branch git refuses to delete, and names it in the line that says the start failed',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             What git says about a branch that is not fully merged.
             */
            const refusal = 'error: the branch feat/a is not fully merged';
            /**
             A git whose every add is killed and whose deletion git refuses.
             */
            const git = standInGit([
              answered(1, ''),
              answered(0, ''),
              {
                status: (-1),
                stdout: '',
                stderr: '',
                problem: 'did not answer within 900000 milliseconds',
              },
              answered(0, ''),
              answered(0, 'refs/heads/feat/a\n'),
              answered(0, ''),
              {
                status: (-1),
                stdout: '',
                stderr: '',
                problem: 'did not answer within 900000 milliseconds',
              },
              answered(0, 'refs/heads/feat/a\n'),
              {
                status: 1,
                stdout: '',
                stderr: `${refusal}\n`,
              },
            ],);
            try {
              await makeWorktree({
                git: git.git,
                checkoutFolder: folder.path,
                worktree: join(folder.path, '.claude', 'worktrees', 'pi-a',),
                branch: 'feat/a',
                base: 'main',
                claim: {
                  runName: 'a',
                  runFolder: join(folder.path, 'runs', 'a-1',),
                },
              },);
            }
            catch (error) {
              expect(String(error,),).toContain('feat/a this start made is still there',);
              expect(String(error,),).toContain('not fully merged',);
              return;
            }
            throw new Error('a worktree add that never worked was taken for one that did',);
          },
        },),
        it({
          name: 'leaves a worktree that no proof says this run made exactly as it is, asking git nothing',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A folder that is already a worktree, which nothing here made.
             */
            const worktree = join(folder.path, 'pi-a',);
            await mkdir(join(worktree, '.git',), { recursive: true, },);
            /**
             A git nobody should reach.
             */
            const git = standInGit([answered(0, ''),],);
            try {
              await makeWorktree({
                git: git.git,
                checkoutFolder: folder.path,
                worktree,
                branch: 'feat/a',
                base: BASE_OF_A_RUN_NAMING_NONE,
                claim: {
                  runName: 'a',
                  runFolder: join(folder.path, 'runs', 'a-1',),
                },
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(WorktreeTakenError,);
              expect(String(error,),).toContain(worktree,);
              expect(String(error,),).toContain('no proof',);
              expect(git.asked,).toEqual([],);
              await expect(
                readFile(join(worktree, '.git', 'HEAD',),),
              ).rejects.toThrow('ENOENT',);
              return;
            }
            throw new Error('a worktree with no proof was taken for this start\'s',);
          },
        },),
        it({
          name: 'takes up a worktree whose proof says another run folder made it on the branch this start asks for',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The worktree that stands there.
             */
            const worktree = join(folder.path, 'pi-a',);
            await mkdir(join(worktree, '.git',), { recursive: true, },);
            await writeFile(worktreeProofFileOf({ worktree, },), JSON.stringify({
              path: worktree,
              runName: 'a',
              runFolder: 'a-1',
              branch: 'feat/a',
              state: 'made',
              writtenAt: new Date(0,).toISOString(),
            },),);
            /**
             A git nobody should reach.
             */
            const git = standInGit([answered(0, ''),],);
            expect(await makeWorktree({
              git: git.git,
              checkoutFolder: folder.path,
              worktree,
              branch: 'feat/a',
              base: BASE_OF_A_RUN_NAMING_NONE,
              claim: {
                runName: 'a',
                runFolder: join(folder.path, 'runs', 'a-2',),
              },
            },),).toContain('made by run folder a-1 on branch feat/a',);
            expect(git.asked,).toEqual([],);
          },
        },),
        it({
          name: 'leaves a worktree whose proof names another branch, so a start asked for one branch is never handed a worktree on another',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The worktree another start made on another branch.
             */
            const worktree = join(folder.path, 'pi-a',);
            await mkdir(join(worktree, '.git',), { recursive: true, },);
            await writeFile(worktreeProofFileOf({ worktree, },), JSON.stringify({
              path: worktree,
              runName: 'a',
              runFolder: 'a-1',
              branch: 'feat/first',
              state: 'made',
              writtenAt: new Date(0,).toISOString(),
            },),);
            try {
              await makeWorktree({
                git: standInGit([answered(0, ''),],).git,
                checkoutFolder: folder.path,
                worktree,
                branch: 'feat/second',
                base: BASE_OF_A_RUN_NAMING_NONE,
                claim: {
                  runName: 'a',
                  runFolder: join(folder.path, 'runs', 'a-2',),
                },
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(WorktreeTakenError,);
              expect(String(error,),).toContain('feat/first',);
              expect(String(error,),).toContain('feat/second',);
              return;
            }
            throw new Error('a worktree on another branch was taken for this start\'s',);
          },
        },),
        it({
          name: 'leaves a worktree whose making has not finished while its maker may still be making it, and clears one whose making was killed at its own bound',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             When this test reads the time.
             */
            const nowMilliseconds = 1_700_000_000_000;
            /**
             The worktree a making left.
             */
            const worktree = join(folder.path, '.claude', 'worktrees', 'pi-a',);
            await mkdir(join(worktree, '.git',), { recursive: true, },);
            await writeFile(join(worktree, 'half.txt',), 'half a copy',);
            /**
             What that making wrote beside it.
             */
            function proofOfTheMaking(when: number,): string {
              return JSON.stringify({
                path: worktree,
                runName: 'a',
                runFolder: 'a-1',
                branch: 'feat/a',
                state: 'making',
                writtenAt: new Date(when,).toISOString(),
              },);
            }
            await writeFile(worktreeProofFileOf({ worktree, },), proofOfTheMaking(nowMilliseconds,),);
            try {
              await makeWorktree({
                git: standInGit([answered(0, ''),],).git,
                checkoutFolder: folder.path,
                worktree,
                branch: 'feat/a',
                base: BASE_OF_A_RUN_NAMING_NONE,
                claim: {
                  runName: 'a',
                  runFolder: join(folder.path, 'runs', 'a-1',),
                },
                now: function now() {
                  return nowMilliseconds;
                },
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(WorktreeTakenError,);
              expect(String(error,),).toContain('making has not finished',);
            }
            // The same proof, older than a making\'s own bound, names an add killed at that bound: its partial checkout is cleared,
            // and the worktree is made again.
            await writeFile(worktreeProofFileOf({ worktree, },), proofOfTheMaking((nowMilliseconds - WORKTREE_ADD_TIMEOUT_MILLISECONDS) - 120_000,),);
            /**
             A git that answers the clearing's prune, says the branch is new, and makes the worktree.
             */
            const git = standInGit([
              answered(0, ''),
              answered(1, ''),
              answered(0, ''),
              answered(0, ''),
            ],);
            expect(await makeWorktree({
              git: git.git,
              checkoutFolder: folder.path,
              worktree,
              branch: 'feat/a',
              base: BASE_OF_A_RUN_NAMING_NONE,
              claim: {
                runName: 'a',
                runFolder: join(folder.path, 'runs', 'a-1',),
              },
              now: function now() {
                return nowMilliseconds;
              },
            },),).toContain('new branch feat/a',);
            await expect(readFile(join(worktree, 'half.txt',), 'utf8',),).rejects.toThrow('ENOENT',);
          },
        },),
        it({
          name: 'removes a folder that is empty and is no worktree, and refuses one that holds anything',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             An empty folder where the worktree belongs.
             */
            const empty = join(folder.path, 'pi-a',);
            await mkdir(empty,);
            await makeWorktree({
              git: standInGit([
                answered(1, ''),
                answered(0, ''),
                answered(0, ''),
              ],).git,
              checkoutFolder: folder.path,
              worktree: empty,
              branch: 'feat/a',
              base: BASE_OF_A_RUN_NAMING_NONE,
              claim: {
                runName: 'a',
                runFolder: join(folder.path, 'runs', 'a-1',),
              },
            },);
            /**
             A folder holding a file a person wrote.
             */
            const held = join(folder.path, 'pi-b',);
            await mkdir(held,);
            await writeFile(join(held, 'notes.md',), 'keep me',);
            try {
              await makeWorktree({
                git: standInGit([answered(1, ''), answered(0, ''),],).git,
                checkoutFolder: folder.path,
                worktree: held,
                branch: 'feat/b',
                base: BASE_OF_A_RUN_NAMING_NONE,
                claim: {
                  runName: 'b',
                  runFolder: join(folder.path, 'runs', 'b-1',),
                },
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(WorktreeFolderOccupiedError,);
              expect(String(error,),).toContain('holds something',);
              return;
            }
            throw new Error('a folder holding a file was taken for a worktree',);
          },
        },),
        it({
          name: 'says what git answered when it refused to make the worktree',
          fn: async () => {
            await using folder = await makeFolder();
            try {
              await makeWorktree({
                git: standInGit([
                  answered(1, ''),
                  answered(0, ''),
                  {
                    status: 128,
                    stdout: '',
                    stderr: 'fatal: a branch named feat/a already exists',
                  },
                  answered(0, ''),
                  {
                    status: 128,
                    stdout: '',
                    stderr: 'fatal: a branch named feat/a already exists',
                  },
                ],).git,
                checkoutFolder: folder.path,
                worktree: join(folder.path, 'pi-a',),
                branch: 'feat/a',
                base: BASE_OF_A_RUN_NAMING_NONE,
                claim: {
                  runName: 'a',
                  runFolder: join(folder.path, 'runs', 'a-1',),
                },
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(GitCommandFailedError,);
              expect(String(error,),).toContain('already exists',);
              return;
            }
            throw new Error('a refused worktree add was reported as a worktree',);
          },
        },),
      ],
    },),
    describe({
      name: worktreeOutcomeOf.name,
      children: [
        it({
          name: 'removes the worktree of a run that finished its work on a branch with nothing uncommitted',
          fn: async () => {
            expect(worktreeOutcomeOf({
              readOnlyInMain: false,
              endReason: 'idle',
              onABranch: true,
              status: 0,
              statusText: '',
            },),).toEqual({ action: 'remove', },);
            expect(worktreeOutcomeOf({
              readOnlyInMain: false,
              endReason: 'stop-file',
              onABranch: true,
              status: 0,
              statusText: '  \n',
            },),).toEqual({ action: 'remove', },);
          },
        },),
        it({
          name: 'keeps the worktree of a run that ended on a cap or an error, or of a detached HEAD, and says which',
          fn: async () => {
            expect(worktreeOutcomeOf({
              readOnlyInMain: false,
              endReason: 'turn-cap',
              onABranch: true,
              status: 0,
              statusText: '',
            },),).toEqual({
              action: 'keep',
              line: 'worktree kept (run ended turn-cap, on a branch)',
            },);
            expect(worktreeOutcomeOf({
              readOnlyInMain: false,
              endReason: 'idle',
              onABranch: false,
              status: 0,
              statusText: '',
            },),).toEqual({
              action: 'keep',
              line: 'worktree kept (run ended idle, detached HEAD)',
            },);
            /**
             What the outcome says about a run that wrote no summary.
             */
            const nothingSaid = worktreeOutcomeOf({
              readOnlyInMain: false,
              onABranch: true,
              status: 0,
              statusText: '',
            },);
            expect((nothingSaid.action === 'keep') ? nothingSaid.line : '',).toContain('run ended undefined',);
          },
        },),
        it({
          name: 'keeps a worktree that holds uncommitted work, naming the first lines git printed',
          fn: async () => {
            /**
             What the outcome says about a worktree with a modified file.
             */
            const outcome = worktreeOutcomeOf({
              readOnlyInMain: false,
              endReason: 'idle',
              onABranch: true,
              status: 0,
              statusText: ' M src/a.ts\n?? src/b.ts\n',
            },);
            expect(outcome,).toEqual({
              action: 'keep',
              line: 'worktree kept, it holds uncommitted work (git status 0): M src/a.ts, ?? src/b.ts',
            },);
            expect(worktreeOutcomeOf({
              readOnlyInMain: false,
              endReason: 'idle',
              onABranch: true,
              status: 3,
              statusText: '',
            },).action,).toBe('keep',);
          },
        },),
        it({
          name: 'says nothing at all for a run that works in the checkout itself',
          fn: async () => {
            expect(worktreeOutcomeOf({
              readOnlyInMain: true,
              endReason: 'idle',
              onABranch: false,
              status: 0,
              statusText: '',
            },),).toEqual({ action: 'keep-quietly', },);
          },
        },),
      ],
    },),
    describe({
      name: removeWorktree.name,
      children: [
        it({
          name: 'removes the worktree without ever forcing it, and reports what git said',
          fn: async () => {
            /**
             A git that refuses the removal, as it does for a worktree holding work.
             */
            const git = standInGit([{
              status: 1,
              stdout: '',
              stderr: "fatal: '/repo/pi-a' contains modified or untracked files",
            },],);
            expect(await removeWorktree({
              git: git.git,
              checkoutFolder: '/repo',
              worktree: '/repo/pi-a',
            },),).toContain('contains modified or untracked files',);
            expect(git.asked[0]?.words,).toEqual([
              'worktree',
              'remove',
              '/repo/pi-a',
            ],);
            expect(git.asked[0]?.words,).not.toContain('--force',);
          },
        },),
      ],
    },),
    describe({
      name: parseWorktreeLauncherOptions.name,
      children: [
        it({
          name: 'reads every required option, and takes the pool defaults a launcher has always had',
          fn: async () => {
            /**
             What a complete command line asks for.
             */
            const options = parseWorktreeLauncherOptions({
              commandLine: [
                '--name=copy-route',
                '--branch=feat/copy-route',
                '--prompt=/runs/prompt.md',
                '--cpu=800',
                '--memory=12G',
                '--tasks=3000',
                '--wall=7200',
              ],
              checkoutFolder: '/repo',
            },);
            expect(options,).toEqual({
              name: 'copy-route',
              branch: 'feat/copy-route',
              base: BASE_OF_A_RUN_NAMING_NONE,
              promptFile: '/runs/prompt.md',
              cpuPercent: 800,
              memory: '12G',
              taskCount: 3_000,
              wallSeconds: 7_200,
              provider: NEURALWATT_STANDARD.provider,
              model: NEURALWATT_STANDARD.model,
              idleSeconds: IDLE_SECONDS_OF_A_RUN,
              runsFolder: join(process.env.HOME ?? '', 'temp', 'agent', 'pi-runs',),
              checkoutFolder: '/repo',
            },);
          },
        },),
        it({
          name: 'reads the run number, the session, the caps and the folders a caller names',
          fn: async () => {
            expect(parseWorktreeLauncherOptions({
              commandLine: [
                '--name=a',
                '--branch=none',
                '--base=fix/a-reviewed-branch',
                '--prompt=p.md',
                '--cpu=8',
                '--memory=1G',
                '--tasks=10',
                '--wall=60',
                '--run=4',
                '--session=/s.jsonl',
                '--provider=opencode-go',
                '--model=deepseek-v4.1-flash',
                '--turn-cap=40',
                '--idle-seconds=5',
                '--runs-folder=/runs',
                '--checkout-folder=/elsewhere',
              ],
              checkoutFolder: '/repo',
            },),).toEqual({
              name: 'a',
              branch: NO_WORKTREE_BRANCH,
              base: 'fix/a-reviewed-branch',
              promptFile: 'p.md',
              cpuPercent: 8,
              memory: '1G',
              taskCount: 10,
              wallSeconds: 60,
              runNumber: 4,
              session: '/s.jsonl',
              provider: 'opencode-go',
              model: 'deepseek-v4.1-flash',
              turnCap: 40,
              idleSeconds: 5,
              runsFolder: '/runs',
              checkoutFolder: '/elsewhere',
            },);
          },
        },),
        it({
          name: 'refuses a name that carries the character telling a run name from its folder number, and a run number past what a run can count to',
          fn: async () => {
            /**
             A command line every option of which is usable.
             */
            const usable = [
              '--name=a',
              '--branch=feat/a',
              '--prompt=p.md',
              '--cpu=8',
              '--memory=1G',
              '--tasks=10',
              '--wall=60',
            ];
            expect(() => parseWorktreeLauncherOptions({
              commandLine: [...usable, '--name=copy@route',],
              checkoutFolder: '/repo',
            },),).toThrow('may not carry "@"',);
            // The largest number a run can be counted to is one a folder can carry; the number after it cannot be reached.
            expect(parseWorktreeLauncherOptions({
              commandLine: [...usable, `--run=${String(Number.MAX_SAFE_INTEGER,)}`,],
              checkoutFolder: '/repo',
            },).runNumber,).toBe(Number.MAX_SAFE_INTEGER,);
            expect(() => parseWorktreeLauncherOptions({
              commandLine: [...usable, '--run=9007199254740992',],
              checkoutFolder: '/repo',
            },),).toThrow('whole number a run can count to',);
            expect(() => parseWorktreeLauncherOptions({
              commandLine: [...usable, '--run=1.5',],
              checkoutFolder: '/repo',
            },),).toThrow('whole number a run can count to',);
          },
        },),
        it({
          name: 'refuses a command line that misses an option, names an unknown one, or gives a limit it cannot use',
          fn: async () => {
            expect(() => parseWorktreeLauncherOptions({
              commandLine: ['--name=a',],
              checkoutFolder: '/repo',
            },),).toThrow('--branch is required',);
            expect(() => parseWorktreeLauncherOptions({
              commandLine: ['--name=a', '--nothing=1',],
              checkoutFolder: '/repo',
            },),).toThrow('Usage:',);
            expect(() => parseWorktreeLauncherOptions({
              commandLine: [
                '--name=a',
                '--branch=b',
                '--prompt=p',
                '--cpu=0',
                '--memory=1G',
                '--tasks=1',
                '--wall=1',
              ],
              checkoutFolder: '/repo',
            },),).toThrow('--cpu must be a number above zero',);
          },
        },),
      ],
    },),
  ],
},);
