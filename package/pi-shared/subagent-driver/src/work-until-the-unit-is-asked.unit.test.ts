/**
 What a start records when it refuses after the worktree and the run folder exist: the run's own folder keeps the reason, the
 worktree and the branch, so a person and a later start find them rather than a worktree and an empty folder no one can explain.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
  realpath,
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
  REFUSED_START_RECORD_NAME,
  type RunCommand,
  StartRefusedError,
  gitRunnerOf,
  runCommandWithChildProcess,
  workUntilTheUnitIsAsked,
} from '../dist/final/node/index.mjs';

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
 Makes a folder a test may write in, removed when the test ends.

 @returns the folder, whose path is its real path

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<AThrowawayFolder> {
  /**
   The folder's path, with symbolic links resolved so that Git's answers and this package's compare equal.
   */
  const path = await realpath(
    await mkdtemp(join(scratchRoot(), 'pi-refused-start-',),),
  );
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, maxRetries: 5, retryDelay: 200, },);
    },
  };
}

/**
 A stand-in command runner: answers the manager's question, finds mise, and answers the dependency unit as it is told.

 @param unitAnswer - what the dependency unit answers

 @returns the runner, and the commands it was asked for

 @example
 ```ts
 const { run } = standInRunner({ status: 1, stdout: '', stderr: 'half installed' });
 ```
 */
function standInRunner(unitAnswer: CommandOutput,): { readonly run: RunCommand; readonly asked: readonly string[] } {
  const asked: string[] = [];
  return {
    asked,
    run: async function answered(request,) {
      asked.push(request.command,);
      if (request.command === 'bash') {
        return { status: 0, stdout: '/bin/mise\n', stderr: '', };
      }
      if (request.command === 'systemctl') {
        return { status: 0, stdout: '255\n', stderr: '', };
      }
      if (request.words.some(function isTheQuestion(word,) {
        return word.includes('-manager-check');
      },)) {
        return { status: 0, stdout: '', stderr: '', };
      }
      return unitAnswer;
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: workUntilTheUnitIsAsked.name,
      children: [
        it({
          name: 'records what refused the start in the run folder and throws it on, when the install the worktree needs did not finish',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A checkout with one commit, so that the folders a unit may write can be read from Git's own answers.
             */
            const checkout = join(folder.path, 'checkout',);
            await mkdir(checkout, { recursive: true, },);
            // The Git the fixture needs: a repository with one commit.
            await runCommandWithChildProcess({
              command: 'git',
              words: ['init', '-q', '-b', 'main',],
              folder: checkout,
              environment: { GIT_CONFIG_GLOBAL: join(folder.path, 'gitconfig',), GIT_CONFIG_NOSYSTEM: '1', },
            },);
            await runCommandWithChildProcess({
              command: 'git',
              words: ['-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '-q', '--allow-empty', '-m', 'base',],
              folder: checkout,
              environment: { GIT_CONFIG_GLOBAL: join(folder.path, 'gitconfig',), GIT_CONFIG_NOSYSTEM: '1', },
            },);
            /**
             The run's worktree, which Git made, and the run's own folder, which the start made.
             */
            const worktree = join(checkout, '.claude', 'worktrees', 'pi-a',);
            await mkdir(join(checkout, '.claude', 'worktrees',), { recursive: true, },);
            await runCommandWithChildProcess({
              command: 'git',
              words: ['worktree', 'add', '-q', '-b', 'fix/run-a', worktree,],
              folder: checkout,
              environment: { GIT_CONFIG_GLOBAL: join(folder.path, 'gitconfig',), GIT_CONFIG_NOSYSTEM: '1', },
            },);
            const runFolder = join(folder.path, 'runs', 'a-1',);
            await mkdir(runFolder, { recursive: true, },);
            await mkdir(join(checkout, '.cache',), { recursive: true, },);
            const { run, asked, } = standInRunner({ status: 1, stdout: '', stderr: 'the install did not finish\n', },);
            /**
             What the start refused with.
             */
            const refused = await (async function starting(): Promise<unknown> {
              try {
                return await workUntilTheUnitIsAsked({
                  runName: 'a',
                  checkoutFolder: checkout,
                  worktree,
                  runFolder,
                  branch: 'fix/run-a',
                  readOnlyInMain: false,
                  git: gitRunnerOf(runCommandWithChildProcess,),
                  cpuPercent: 100,
                  memory: '1G',
                  taskCount: 100,
                  sessionPath: '/bin',
                  home: folder.path,
                  run,
                  report: function report() {},
                },);
              }
              catch (error) {
                return error;
              }
            }());
            expect(refused,).toBeInstanceOf(StartRefusedError,);
            /**
             What the run's own folder now says, which a person or a later start reads instead of guessing.
             */
            const record = JSON.parse(await readFile(join(runFolder, REFUSED_START_RECORD_NAME,), 'utf8',),) as {
              readonly reason: string;
              readonly worktree: string;
              readonly branch: string;
              readonly whatIsKept: string;
            };
            expect(record.worktree,).toBe(worktree,);
            expect(record.branch,).toBe('fix/run-a',);
            expect(record.reason,).toContain('dependencies were not linked',);
            expect(record.reason,).toContain('the install did not finish',);
            expect(record.whatIsKept,).toContain('the worktree and this run folder are kept',);
            // The install was asked for once, in the unit of the run's own name, and the manager's question was asked too.
            expect(asked.filter(function isTheInstall(command,) {
              return command === 'systemd-run';
            },).length,).toBe(2,);
          },
        },),
        it({
          name: 'records nothing when everything the step does works, and answers the folders a unit may write',
          fn: async () => {
            await using folder = await makeFolder();
            const checkout = join(folder.path, 'checkout',);
            await mkdir(checkout, { recursive: true, },);
            await runCommandWithChildProcess({ command: 'git', words: ['init', '-q', '-b', 'main',], folder: checkout, environment: { GIT_CONFIG_GLOBAL: join(folder.path, 'gitconfig',), GIT_CONFIG_NOSYSTEM: '1', }, },);
            const worktree = join(checkout, 'run-a',);
            await mkdir(worktree, { recursive: true, },);
            await writeFile(join(worktree, '.git',), `gitdir: ${join(checkout, '.git',)}\n`,);
            const runFolder = join(folder.path, 'runs', 'a-1',);
            await mkdir(runFolder, { recursive: true, },);
            await mkdir(join(checkout, '.cache',), { recursive: true, },);
            const { run, } = standInRunner({ status: 0, stdout: 'installed\n', stderr: '', },);
            /**
             What the step answered for the folders a unit may write.
             */
            const fileSystem = await workUntilTheUnitIsAsked({
              runName: 'a',
              checkoutFolder: checkout,
              worktree,
              runFolder,
              branch: 'fix/run-a',
              readOnlyInMain: false,
              git: gitRunnerOf(runCommandWithChildProcess,),
              cpuPercent: 100,
              memory: '1G',
              taskCount: 100,
              sessionPath: '/bin',
              home: folder.path,
              run,
              report: function report() {},
            },);
            expect(fileSystem.foldersAUnitCannotWrite,).toContain(checkout,);
            expect(fileSystem.foldersAUnitMustWrite,).toContain(worktree,);
            expect(fileSystem.foldersAUnitMustWrite,).toContain(runFolder,);
            // The run's own folder holds no refusal record, which is what a start that reached the unit leaves.
            await expect(readFile(join(runFolder, REFUSED_START_RECORD_NAME,), 'utf8',),).rejects.toThrow(REFUSED_START_RECORD_NAME,);
          },
        },),
      ],
    },),
  ],
},);
