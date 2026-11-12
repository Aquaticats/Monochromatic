/**
 What a launcher settles before it makes anything: which checkout it works in, that the prompt can be read, that the base names a
 commit, and that the run's unit name is free.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
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
  CHECKOUT_MARKER,
  CheckoutNotFoundError,
  ENTRY_REASON_CHARACTERS,
  type CommandOutput,
  type GitRunner,
  baseProblemIn,
  checkoutFolderOf,
  nameProblemIn,
  piProblemIn,
  promptProblemIn,
  reasonInFailedStart,
  startRefusalsIn,
  unitNameOfGiven,
} from '../dist/final/node/index.mjs';

/**
 Pi's own words when a session extension could not be loaded, which is the failure this check exists for.
 */
const EXTENSION_LOAD_FAILURE = 'Failed to load extension "/repo/index.mjs": Failed to load extension: Cannot find module '
  + "'@monochromatic-dev/async-time-core/ts'";

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
  const path = await mkdtemp(join(scratchRoot(), 'pi-before-a-start-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 A git stand-in that answers every question with the same answer and records what it was asked.

 @param answer - what every question is answered with

 @returns the runner, and what it was asked

 @example
 ```ts
 const git = answering({ status: 0, stdout: '', stderr: '', });
 ```
 */
function answering(answer: CommandOutput,): {
  readonly git: GitRunner;
  readonly asked: { readonly words: readonly string[]; readonly folder: string }[];
} {
  /**
   What git was asked.
   */
  const asked: { readonly words: readonly string[]; readonly folder: string }[] = [];
  return {
    asked,
    git: async function gitIn(request,) {
      asked.push({
        words: request.words,
        folder: request.folder,
      },);
      return answer;
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: checkoutFolderOf.name,
      children: [
        it({
          name: 'finds the checkout above the package folder a command of this package runs in',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The checkout a run's worktree belongs in.
             */
            const checkout = join(folder.path, 'checkout',);
            /**
             The folder a command of this package is started in, inside the package it belongs to.
             */
            const packageFolder = join(checkout, 'package', 'pi-shared', 'subagent-driver',);
            await mkdir(join(checkout, CHECKOUT_MARKER,), { recursive: true, },);
            await mkdir(packageFolder, { recursive: true, },);
            expect(await checkoutFolderOf({ from: packageFolder, },),).toBe(checkout,);
          },
        },),
        it({
          name: 'takes a folder a worktree marks with a file for a checkout, which is what a linked worktree carries',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A worktree, whose marker is a file naming the checkout it was made from.
             */
            const worktree = join(folder.path, 'pi-a',);
            await mkdir(join(worktree, 'package',), { recursive: true, },);
            await writeFile(join(worktree, CHECKOUT_MARKER,), 'gitdir: /checkout/.git/worktrees/pi-a\n',);
            expect(await checkoutFolderOf({ from: join(worktree, 'package',), },),).toBe(worktree,);
          },
        },),
        it({
          name: 'refuses when no folder above holds a marker, saying how to name the checkout instead',
          fn: async () => {
            await using folder = await makeFolder();
            try {
              await checkoutFolderOf({ from: join(folder.path, 'nowhere',), },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(CheckoutNotFoundError,);
              expect(String(error,),).toContain('--checkout-folder',);
              expect(String(error,),).toContain('nowhere',);
              return;
            }
            throw new Error('a folder under no checkout was taken for a checkout',);
          },
        },),
      ],
    },),
    describe({
      name: promptProblemIn.name,
      children: [
        it({
          name: 'says nothing about a prompt file that can be read',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A prompt file a run could be given.
             */
            const prompt = join(folder.path, 'a-prompt.md',);
            await writeFile(prompt, 'Answer with one line.\n',);
            expect(await promptProblemIn({ path: prompt, },),).toEqual({},);
          },
        },),
        it({
          name: 'names a prompt file that is not there, one that is no regular file, and one that cannot be read',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A path nothing was written at.
             */
            const missing = join(folder.path, 'nothing-here.md',);
            expect((await promptProblemIn({ path: missing, },)).problem,).toContain('is not there',);
            /**
             A folder where a prompt file was expected.
             */
            const directory = join(folder.path, 'a-folder',);
            await mkdir(directory,);
            expect((await promptProblemIn({ path: directory, },)).problem,).toContain('is not a regular file',);
            /**
             A file this user may not read.
             */
            const shut = join(folder.path, 'shut.md',);
            await writeFile(shut, 'x\n',);
            await chmod(shut, 0o000,);
            expect((await promptProblemIn({ path: shut, },)).problem,).toContain('cannot be read',);
          },
        },),
      ],
    },),
    describe({
      name: baseProblemIn.name,
      children: [
        it({
          name: 'says nothing when the base names a commit, and asks for the commit of it, never for whatever else it could name',
          fn: async () => {
            /**
             A git that resolves the base.
             */
            const git = answering({
              status: 0,
              stdout: 'a-commit\n',
              stderr: '',
            },);
            /**
             The commit a review starts at.
             */
            const base = 'fd3ede1fad';
            expect(await baseProblemIn({
              git: git.git,
              checkoutFolder: '/repo',
              base,
            },),).toEqual({},);
            expect(git.asked[0]?.words.at(-1),).toBe(`${base}^{commit}`,);
            expect(git.asked[0]?.folder,).toBe('/repo',);
          },
        },),
        it({
          name: 'names a base that resolves to nothing, with what git complained about',
          fn: async () => {
            /**
             The base a broken list names.
             */
            const base = 'fix/no-such-branch';
            /**
             What git answers for a base nothing names.
             */
            const git = answering({
              status: 1,
              stdout: '',
              stderr: 'fatal: Needed a single revision\n',
            },);
            expect((await baseProblemIn({
              git: git.git,
              checkoutFolder: '/repo',
              base,
            },)).problem,).toContain(base,);
            expect((await baseProblemIn({
              git: git.git,
              checkoutFolder: '/repo',
              base,
            },)).problem,).toContain('Needed a single revision',);
          },
        },),
      ],
    },),
    describe({
      name: nameProblemIn.name,
      children: [
        it({
          name: 'says nothing when the manager holds no unit of that name, which it answers with not-found or with nothing',
          fn: async () => {
            /**
             The run a batch list names.
             */
            const runName = 'copy-route';
            expect(await nameProblemIn({
              run: async function answered() {
                return {
                  status: 0,
                  stdout: 'not-found\n',
                  stderr: '',
                };
              },
              runName,
            },),).toEqual({},);
            expect(await nameProblemIn({
              run: async function answered() {
                return {
                  status: 1,
                  stdout: '',
                  stderr: 'Failed to connect to bus\n',
                };
              },
              runName,
            },),).toEqual({},);
          },
        },),
        it({
          name: 'names the unit a run of that name would collide with',
          fn: async () => {
            /**
             The run a batch list names.
             */
            const runName = 'copy-route';
            /**
             What the manager says about a unit it holds.
             */
            const { problem, } = await nameProblemIn({
              run: async function answered() {
                return {
                  status: 0,
                  stdout: 'loaded\n',
                  stderr: '',
                };
              },
              runName,
            },);
            expect(problem,).toContain(unitNameOfGiven({ name: runName, },),);
            expect(problem,).toContain(runName,);
          },
        },),
      ],
    },),
    describe({
      name: startRefusalsIn.name,
      children: [
        it({
          name: 'gathers every reason a start is refused, so a reader fixes them all at once',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The reasons one start that can work for none of them is refused.
             */
            const refusals = await startRefusalsIn({
              promptFile: join(folder.path, 'no-prompt.md',),
              git: answering({
                status: 1,
                stdout: '',
                stderr: 'fatal: bad revision\n',
              },).git,
              checkoutFolder: folder.path,
              base: 'no-such-branch',
              run: async function answered(request,) {
                return (request.command === 'pi')
                  ? {
                    status: 0,
                    stdout: 'Usage: pi\n',
                    stderr: '',
                  }
                  : {
                    status: 0,
                    stdout: 'active\n',
                    stderr: '',
                  };
              },
              runName: 'copy-route',
              sessionPath: '/bin',
              home: folder.path,
            },);
            expect(refusals.length,).toBe(3,);
          },
        },),
        it({
          name: 'refuses nothing when every check passes',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A prompt file a run could be given.
             */
            const prompt = join(folder.path, 'a-prompt.md',);
            await writeFile(prompt, 'Answer with one line.\n',);
            expect(await startRefusalsIn({
              promptFile: prompt,
              git: answering({
                status: 0,
                stdout: 'a-commit\n',
                stderr: '',
              },).git,
              checkoutFolder: folder.path,
              base: 'main',
              run: async function answered() {
                return {
                  status: 0,
                  stdout: 'not-found\n',
                  stderr: '',
                };
              },
              runName: 'copy-route',
              sessionPath: '/bin',
              home: folder.path,
            },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: piProblemIn.name,
      children: [
        it({
          name: 'says nothing when Pi prints its help and loads the extensions this session names',
          fn: async () => {
            expect(await piProblemIn({
              run: async function answered() {
                return {
                  status: 0,
                  stdout: 'Usage: pi [options]\n',
                  stderr: 'Warning: No models match pattern "hyper/kimi-k3"\n',
                };
              },
              sessionPath: '/bin',
              home: '/home/a',
              folder: '/repo',
            },),).toEqual({},);
          },
        },),
        it({
          name: 'names the extension Pi could not load, which is a start that would end before its first turn',
          fn: async () => {
            /**
             What this check is asked.
             */
            const asked: {
              readonly command: string;
              readonly words: readonly string[];
              readonly environment?: Readonly<Record<string, string>>
            }[] = [];
            const problem = await piProblemIn({
              run: async function answered(request,) {
                asked.push({
                  command: request.command,
                  words: request.words,
                  ...(request.environment === undefined ? {} : { environment: request.environment, }),
                },);
                return {
                  status: 0,
                  stdout: 'Usage: pi [options]\n',
                  stderr: `Error: ${EXTENSION_LOAD_FAILURE}\nHint: Start without extensions using "pi -ne".\n`,
                };
              },
              sessionPath: '/session/bin',
              home: '/home/a',
              folder: '/repo',
            },);
            expect(problem.problem,).toContain('Cannot find module',);
            expect(problem.problem,).toContain('would end before its first turn',);
            expect(asked[0]?.command,).toBe('pi',);
            expect(asked[0]?.words,).toEqual(['--help',],);
            expect(asked[0]?.environment?.PATH,).toBe('/session/bin',);
            expect(asked[0]?.environment?.HOME,).toBe('/home/a',);
          },
        },),
        it({
          name: 'names a Pi that would not start at all, saying what it answered',
          fn: async () => {
            expect((await piProblemIn({
              run: async function answered() {
                return {
                  status: 1,
                  stdout: '',
                  stderr: 'pi: error while loading shared libraries\n',
                };
              },
              sessionPath: '/bin',
              home: '/home/a',
              folder: '/repo',
            },)).problem,).toContain('error while loading shared libraries',);
          },
        },),
      ],
    },),
    describe({
      name: reasonInFailedStart.name,
      children: [
        it({
          name: 'reads the message of an uncaught error, dropping the place, the code, the frames and the version Node ends with',
          fn: async () => {
            expect(reasonInFailedStart({
              output: [
                'file:///pkg/src/x.ts:12',
                '    throw new GitCommandFailedError(',
                '          ^',
                '',
                'GitCommandFailedError: git worktree add failed for /repo/pi-a',
                '    at makeWorktree (file:///pkg/src/x.ts:12:11)',
                'Node.js v26.13.0',
                '',
              ].join('\n',),
            },).reason,).toBe('GitCommandFailedError: git worktree add failed for /repo/pi-a',);
          },
        },),
        it({
          name: 'reads the line a gate or a launcher printed as its own answer, which is a reason with no stack at all',
          fn: async () => {
            expect(reasonInFailedStart({ output: 'NO SLOT: qwen-plan-deepseek 18/18, not started: a\n', },).reason,)
              .toBe('NO SLOT: qwen-plan-deepseek 18/18, not started: a',);
          },
        },),
        it({
          name: 'answers nothing when a start printed only a stack and a version, and cuts a reason too long to scan',
          fn: async () => {
            expect(reasonInFailedStart({ output: '    at x (file:///p.ts:1:1)\nNode.js v26.13.0\n', },),).toEqual({},);
            expect(reasonInFailedStart({ output: '', },),).toEqual({},);
            /**
             A reason far longer than one line should carry.
             */
            const long = `Error: ${'x'.repeat(ENTRY_REASON_CHARACTERS * 2,)}`;
            expect(reasonInFailedStart({ output: `${long}\n`, },).reason?.length,).toBe(ENTRY_REASON_CHARACTERS,);
          },
        },),
      ],
    },),
  ],
},);
