/**
 The step that links a run's dependencies, asked of this machine's manager as a unit of its own: a fixture whose `postinstall`
 writes into the checkout must be refused there and allowed to write in the worktree, which is what says the branch's own install
 scripts run where the run is confined.

 A machine with no user manager to answer is skipped with the reason it gave, so this test says something there rather than
 failing.

 @module
 */

import { execFile, } from 'node:child_process';
import { createServer, } from 'node:net';
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
  DEPENDENCIES_RECORD_NAME,
  DEPENDENCIES_UNIT_SECONDS,
  type CommandOutput,
  fileSystemOfARun,
  gitRunnerOf,
  linkDependenciesInAUnit,
  makeTheFoldersAToolMakesIn,
  managerPresenceIn,
  runCommandWithChildProcess,
  StartRefusedError,
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
 What a call answered, or what it threw, so a call that refuses is read as a value.

 @param call - the call

 @returns its answer, or the error it threw

 @example
 ```ts
 const refused = await settledBy({ call: async () => await linkDependenciesInAUnit({ ... }), });
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
 What a fixture says the machine has: a manager whose socket is there, or one whose socket and units are not.
 */
type ManagerFixture = {
  /**
   The folder the manager's socket lives in, which this step reads.
   */
  readonly runtimeDirectory: string;

  /**
   The file that names the units this process is in, which this step reads.
   */
  readonly cgroupFile: string;

  /**
   Stops the socket, so the fixture leaves nothing listening.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes what this machine has, as a fixture: a folder holding the manager's own socket, and a cgroup file naming a unit of a manager
 or naming nothing of one.

 Why a fixture: the question decides whether an install that cannot be confined is degraded or refused, and the answer a machine
 gives must not depend on the machine a test happens to run on.

 @param withSocket - whether the manager's socket is there

 @returns the runtime folder, the cgroup file, and the way to stop the socket

 @example
 ```ts
 await using manager = await makeManagerFixture({ withSocket: true, });
 ```
 */
async function makeManagerFixture({
  withSocket,
}: {
  readonly withSocket: boolean;
},): Promise<ManagerFixture> {
  /**
   The fixture's own folder, made short on purpose: a Unix socket's path may not pass the length the kernel keeps for it, and the
   harness's scratch root is already long.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-manager-',),);
  /**
   The folder the manager keeps its socket in.
   */
  const runtimeDirectory = join(path, 'runtime',);
  await mkdir(join(runtimeDirectory, 'systemd',), { recursive: true, mode: 0o700, },);
  /**
   The file that names the units this process is in.
   */
  const cgroupFile = join(path, 'cgroup',);
  await writeFile(
    cgroupFile,
    withSocket
      ? '0::/user.slice/user-1000.slice/user@1000.service/app.slice/probe.service\n'
      : '0::/user.slice/user-1000.slice/session-1.scope\n',
  );
  /**
   The socket the manager listens on, which nothing connects to.
   */
  const socket = join(runtimeDirectory, 'systemd', 'private',);
  /**
   What listens on it, or nothing when the fixture has no manager.
   */
  const server = createServer(function accept() {});
  if (withSocket) {
    await new Promise<void>(function waitForListening(resolve,) {
      server.listen(socket, resolve,);
    },);
  }
  return {
    runtimeDirectory,
    cgroupFile,
    async [Symbol.asyncDispose]() {
      await new Promise<void>(function waitForClosing(resolve,) {
        server.close(function closed() {
          resolve();
        },);
      },);
      await rm(path, { recursive: true, force: true, maxRetries: 5, retryDelay: 200, },);
    },
  };
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
            await using folder = await makeFolder();
            /**
             The run's own folder, which this step records what it answered in.
             */
            const runFolder = join(folder.path, 'runs', 'a-1',);
            /**
             What this machine has, as a fixture: a manager's socket, so the question about the machine comes out the same
             wherever this test runs.
             */
            await using manager = await makeManagerFixture({ withSocket: true, },);
            const linked = await linkDependenciesInAUnit({
              runName: 'probe-bounds',
              checkoutFolder: '/repo',
              worktree: '/repo/.claude/worktrees/pi-a',
              runFolder,
              fileSystem: {
                foldersAUnitCannotWrite: ['/repo',],
                foldersAUnitMustWrite: ['/repo/.claude/worktrees/pi-a',],
              },
              cpuPercent: 100,
              memory: '1G',
              taskCount: 100,
              pathOfMiseEnvironment: '/bin',
              home: '/home/a',
              managerRuntimeDirectory: manager.runtimeDirectory,
              cgroupFile: manager.cgroupFile,
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
        it({
          name: 'refuses the start, naming the socket and both answers, when a manager that is there does not answer, rather than running the install unconfined',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             What this machine has, as a fixture: a manager's socket is there.
             */
            await using manager = await makeManagerFixture({ withSocket: true, },);
            /**
             The run's own folder.
             */
            const runFolder = join(folder.path, 'runs', 'a-1',);
            /**
             Every request this step made, so a test can say what it did not do.
             */
            const asked: string[] = [];
            /**
             What this step refused with.
             */
            const refused = await settledBy({
              call: async function link() {
                return await linkDependenciesInAUnit({
                  runName: 'probe-unanswered',
                  checkoutFolder: '/repo',
                  worktree: '/repo/.claude/worktrees/pi-a',
                  runFolder,
                  fileSystem: {
                    foldersAUnitCannotWrite: ['/repo',],
                    foldersAUnitMustWrite: ['/repo/.claude/worktrees/pi-a',],
                  },
                  cpuPercent: 100,
                  memory: '1G',
                  taskCount: 100,
                  pathOfMiseEnvironment: '/bin',
                  home: '/home/a',
                  managerRuntimeDirectory: manager.runtimeDirectory,
                  cgroupFile: manager.cgroupFile,
                  run: async function recorder(request,): Promise<CommandOutput> {
                    asked.push(request.command,);
                    if (request.command === 'bash') {
                      return { status: 0, stdout: '/bin/mise\n', stderr: '', };
                    }
                    return { status: 1, stdout: '', stderr: 'a manager that did not answer', };
                  },
                },);
              },
            },);
            expect(refused.error,).toBeInstanceOf(StartRefusedError,);
            /**
             What the refusal says.
             */
            const words = (refused.error as Error).message;
            expect(words,).toContain(join(manager.runtimeDirectory, 'systemd', 'private',),);
            expect(words,).toContain('a manager that did not answer',);
            // Asked twice, because a manager that missed one answer is not a missing manager, and then refused rather than
            // running the branch's install scripts with the launcher's access.
            expect(asked.filter(function isTheQuestion(command,) {
              return command === 'systemctl';
            },).length,).toBe(2,);
            expect(asked,).not.toContain('/bin/mise',);
            expect(await isThere({ path: join(runFolder, DEPENDENCIES_RECORD_NAME,), },),).toBe(false,);
          },
        },),
        it({
          name: 'degrades where this machine has no manager at all, and records in the run folder that the install ran with the launcher\'s access',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             What this machine has, as a fixture: no socket, and no unit of a manager in its cgroup.
             */
            await using manager = await makeManagerFixture({ withSocket: false, },);
            /**
             The run's own folder.
             */
            const runFolder = join(folder.path, 'runs', 'a-1',);
            /**
             A file of the checkout an install script writes to, which is what a run lacks protection from in this case.
             */
            const written = join(folder.path, 'written-by-the-install',);
            /**
             Every request this step made.
             */
            const asked: string[] = [];
            /**
             What linking answered.
             */
            const linked = await linkDependenciesInAUnit({
              runName: 'probe-no-manager',
              checkoutFolder: '/repo',
              worktree: '/repo/.claude/worktrees/pi-a',
              runFolder,
              fileSystem: {
                foldersAUnitCannotWrite: ['/repo',],
                foldersAUnitMustWrite: ['/repo/.claude/worktrees/pi-a',],
              },
              cpuPercent: 100,
              memory: '1G',
              taskCount: 100,
              pathOfMiseEnvironment: '/bin',
              home: '/home/a',
              managerRuntimeDirectory: manager.runtimeDirectory,
              cgroupFile: manager.cgroupFile,
              run: async function recorder(request,): Promise<CommandOutput> {
                asked.push(request.command,);
                if (request.command === 'bash') {
                  return { status: 0, stdout: '/bin/mise\n', stderr: '', };
                }
                await writeFile(written, 'written by an install script\n',);
                return { status: 0, stdout: 'installed\n', stderr: '', };
              },
            },);
            expect(linked.confined,).toBe(false,);
            expect(await readFile(written, 'utf8',),).toBe('written by an install script\n',);
            // The question about the manager is never asked where the machine has none: what decides is what the machine has, not
            // one answer to one question.
            expect(asked,).not.toContain('systemctl',);
            expect(linked.why,).toContain('no user manager',);
            expect(linked.why,).toContain('/repo',);
            /**
             What the run's own folder records about this.
             */
            const record = JSON.parse(await readFile(join(runFolder, DEPENDENCIES_RECORD_NAME,), 'utf8',),) as {
              readonly status: number;
              readonly confined: boolean;
              readonly why?: string;
            };
            expect(record.confined,).toBe(false,);
            expect(record.status,).toBe(0,);
            expect(record.why,).toBe(linked.why,);
          },
        },),
        it({
          name: 'reads what the machine has from its socket or from the unit the launcher runs in, never from one answer to one question',
          fn: async () => {
            /**
             A machine whose manager's socket is there.
             */
            await using withSocket = await makeManagerFixture({ withSocket: true, },);
            expect(await managerPresenceIn({
              runtimeDirectory: withSocket.runtimeDirectory,
              cgroupFile: withSocket.cgroupFile,
            },),).toEqual({ there: true, evidence: `the manager's socket is ${join(withSocket.runtimeDirectory, 'systemd', 'private',)}`, },);
            /**
             A machine whose socket is gone but whose cgroup names a unit of a manager.
             */
            await using cgroupOnly = await makeManagerFixture({ withSocket: false, },);
            /**
             A cgroup file naming the unit of this user's own manager, which is where a manager's own units sit.
             */
            const withAManager = join(cgroupOnly.runtimeDirectory, 'cgroup-with-a-manager',);
            await writeFile(withAManager, '0::/user.slice/user-1000.slice/user@1000.service/app.slice/probe.service\n',);
            expect(await managerPresenceIn({
              runtimeDirectory: cgroupOnly.runtimeDirectory,
              cgroupFile: withAManager,
            },),).toEqual({ there: true, evidence: `this launcher runs in a unit of a manager (${withAManager})`, },);
            /**
             A machine with neither.
             */
            expect((await managerPresenceIn({
              runtimeDirectory: cgroupOnly.runtimeDirectory,
              cgroupFile: cgroupOnly.cgroupFile,
            },)).there,).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
