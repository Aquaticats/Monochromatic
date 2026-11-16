/**
 The step that links a run's dependencies, asked of this machine's manager as a unit of its own: a fixture whose `postinstall`
 writes into the checkout must be refused there and allowed to write in the worktree, which is what says the branch's own install
 scripts run where the run is confined.

 A machine with no user manager to answer is skipped with the reason it gave, so this test says something there rather than
 failing.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  access,
  mkdir,
  mkdtemp,
  readFile,
  realpath,
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
  DEPENDENCIES_CLIENT_SECONDS,
  DEPENDENCIES_UNIT_SECONDS,
  type CommandOutput,
  fileSystemOfARun,
  gitRunnerOf,
  linkDependenciesInAUnit,
  makeTheFoldersAToolMakesIn,
  runCommandWithChildProcess,
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
 How this test asks git a question, which is how the package asks it.
 */
const git = gitRunnerOf(runCommandWithChildProcess,);

/**
 Variables that turn every prompt git could ask into a failure.
 */
const GIT_ENVIRONMENT: Readonly<Record<string, string>> = {
  GIT_ASKPASS: '',
  GIT_TERMINAL_PROMPT: '0',
};

/**
 Seconds one question to the manager may take.
 */
const MANAGER_PATIENCE_SECONDS = 60;

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
    await mkdtemp(join(scratchRoot(), 'pi-dependencies-unit-',),),
  );
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Says whether a path is there.

 @param path - the path to look for

 @returns whether it is there

 @example
 ```ts
 const there = await isThere({ path: '/w/installed.txt', });
 ```
 */
async function isThere({ path }: { readonly path: string },): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    // A path that is not there is the answer this asks about; any other failure would hide what the test meant to check.
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return false;
    }
    throw error;
  }
}

/**
 Runs one git command in a folder.

 @param folder - where git runs

 @param words - its arguments

 @returns its standard output, trimmed

 @throws Error when git fails, which no fixture step may

 @example
 ```ts
 await gitIn({ folder: '/repo', words: ['init', '--initial-branch=main',], });
 ```
 */
async function gitIn({ folder, words, }: { readonly folder: string; readonly words: readonly string[] },): Promise<string> {
  /**
   What git answered.
   */
  const answered = await execFileAsync('git', [...words,], {
    cwd: folder,
    env: {
      ...process.env,
      ...GIT_ENVIRONMENT,
    },
    encoding: 'utf8',
    timeout: 60_000,
    killSignal: 'SIGKILL',
  },);
  return answered.stdout.trim();
}

/**
 Asks this machine's manager whether it answers at all.

 @returns what it said when it does not answer, or nothing when it does

 @example
 ```ts
 const { reason, } = await noManagerReason();
 ```
 */
async function noManagerReason(): Promise<{ readonly reason?: string }> {
  try {
    await execFileAsync('systemd-run', [
      '--user',
      '--wait',
      '--collect',
      '--quiet',
      `--unit=probe-dependencies-manager-${String(process.pid,)}`,
      '--property=RuntimeMaxSec=30',
      '--property=CPUQuota=100%',
      '--property=MemoryMax=256M',
      '--property=TasksMax=64',
      '/bin/true',
    ], {
      encoding: 'utf8',
      timeout: MANAGER_PATIENCE_SECONDS * 1_000,
      killSignal: 'SIGKILL',
    },);
    return {};
  }
  catch (error) {
    return { reason: `no user manager answered ${String(error,)}`, };
  }
}

/**
 Why every test here is skipped when this machine's manager does not answer, asked once for the whole file.
 */
const { reason: NO_MANAGER_REASON, } = await noManagerReason();

await describe({
  name: '',
  children: [
    describe({
      name: linkDependenciesInAUnit.name,
      children: [
        it({
          name: 'runs a worktree install inside a capped unit, so its postinstall is refused the checkout and allowed the worktree',
          fn: async () => {
            if (NO_MANAGER_REASON !== undefined) {
              console.log(`[linkDependenciesInAUnit] skipped: ${NO_MANAGER_REASON}`,);
              return;
            }
            await using folder = await makeFolder();
            /**
             The checkout the run works from, whose file the install must not be able to change.
             */
            const checkout = join(folder.path, 'checkout',);
            await mkdir(checkout, { recursive: true, },);
            /**
             A file of the checkout that a postinstall tries to overwrite.
             */
            const protectedFile = join(checkout, 'user-data.txt',);
            await writeFile(protectedFile, 'original user data\n',);
            await gitIn({ folder: checkout, words: ['init', '--initial-branch=main',], });
            await writeFile(
              join(checkout, 'package.json',),
              `${JSON.stringify({
                name: 'disposable-dependencies-probe',
                private: true,
                scripts: {
                  postinstall: String.raw`node -e "const fs=require('node:fs'); fs.writeFileSync('installed.txt', 'installed\n'); fs.writeFileSync('../../../user-data.txt', 'overwritten by install lifecycle\n')"`,
                },
              }, undefined, 2,)}\n`,
            );
            await gitIn({ folder: checkout, words: ['add', '--', 'user-data.txt', 'package.json',], });
            await gitIn({
              folder: checkout,
              words: ['-c', 'user.name=Disposable', '-c', 'user.email=disposable@example.invalid', 'commit', '--message', 'base', '--', 'user-data.txt', 'package.json',],
            },);
            /**
             The worktree the install runs in, which Git made.
             */
            const worktree = join(checkout, '.claude', 'worktrees', 'pi-probe',);
            await mkdir(join(checkout, '.claude', 'worktrees',), { recursive: true, },);
            await gitIn({ folder: checkout, words: ['worktree', 'add', '-b', 'probe-dependencies', worktree,], });
            await writeFile(join(worktree, 'pnpm-lock.yaml',), [
              "lockfileVersion: '9.0'",
              'settings:',
              '  autoInstallPeers: true',
              '  excludeLinksFromLockfile: false',
              'importers:',
              '  .: {}',
              '',
            ].join('\n',),);
            /**
             The run's own folder.
             */
            const runFolder = join(folder.path, 'runs', 'probe-dependencies-1',);
            await mkdir(runFolder, { recursive: true, },);
            // What a start does before it asks for any unit: the folder a run's tools install themselves into is made, so the
            // unit's writable paths all exist when systemd builds its namespace.
            await makeTheFoldersAToolMakesIn({ checkoutFolder: checkout, },);
            /**
             What the run's unit may and may not write, which this step gets too.
             */
            const fileSystem = await fileSystemOfARun({
              git,
              checkoutFolder: checkout,
              worktree,
              runFolder,
              worksInTheCheckoutItself: false,
            },);
            /**
             What linking the dependencies answered.
             */
            const linked = await linkDependenciesInAUnit({
              runName: `probe-dependencies-${String(process.pid,)}`,
              checkoutFolder: checkout,
              worktree,
              runFolder,
              fileSystem,
              cpuPercent: 200,
              memory: '2G',
              taskCount: 512,
              pathOfMiseEnvironment: process.env.PATH ?? '',
              home: process.env.HOME ?? '',
              run: runCommandWithChildProcess,
            },);
            // The postinstall ran, which the marker it writes where the unit may write says, and its write to the checkout was
            // refused by the operating system rather than done with the launcher's access.
            expect(linked.output,).toContain('postinstall',);
            expect(await isThere({ path: join(worktree, 'installed.txt',), },),).toBe(true,);
            expect(await readFile(protectedFile, 'utf8',),).toBe('original user data\n',);
            // A lifecycle script that cannot do what it was written to do fails the install, and the step answers that status.
            expect(linked.status,).not.toBe(0,);
          },
        },),
        it({
          name: 'waits for the dependency unit as long as that unit may run, never for a default shorter than the install',
          fn: async () => {
            /**
             Every request the step made of its command runner.
             */
            const asked: {
              readonly command: string;
              readonly words: readonly string[];
              readonly timeoutMilliseconds?: number;
            }[] = [];
            /**
             What the step answered, with a runner that records every request and answers as the machine would.
             */
            const linked = await linkDependenciesInAUnit({
              runName: 'probe-bounds',
              checkoutFolder: '/repo',
              worktree: '/repo/.claude/worktrees/pi-a',
              runFolder: '/runs/a-1',
              fileSystem: {
                foldersAUnitCannotWrite: ['/repo',],
                foldersAUnitMustWrite: ['/repo/.claude/worktrees/pi-a',],
              },
              cpuPercent: 100,
              memory: '1G',
              taskCount: 100,
              pathOfMiseEnvironment: '/bin',
              home: '/home/a',
              run: async function recorder(request,): Promise<CommandOutput> {
                asked.push({
                  command: request.command,
                  words: request.words,
                  ...((request.timeoutMilliseconds === undefined) ? {} : { timeoutMilliseconds: request.timeoutMilliseconds, }),
                },);
                if (request.command === 'bash') {
                  return {
                    status: 0,
                    stdout: '/bin/mise\n',
                    stderr: '',
                  };
                }
                if (request.command === 'systemctl') {
                  return {
                    status: 0,
                    stdout: '255\n',
                    stderr: '',
                  };
                }
                return {
                  status: 0,
                  stdout: '',
                  stderr: '',
                };
              },
            },);
            expect(linked.confined,).toBe(true,);
            /**
             The request that started the unit.
             */
            const unitRequest = asked.find(function startsTheUnit(request,) {
              return request.command === 'systemd-run';
            },);
            expect(unitRequest,).toBeDefined();
            // The unit may run for its own bound, so the client that waits for it must outlive that bound: a shorter default
            // would leave the install running while the run starts.
            expect(unitRequest?.timeoutMilliseconds,).toBeGreaterThanOrEqual(DEPENDENCIES_UNIT_SECONDS * 1_000,);
            expect(unitRequest?.timeoutMilliseconds,).toBe(DEPENDENCIES_CLIENT_SECONDS * 1_000,);
          },
        },),
      ],
    },),
  ],
},);
