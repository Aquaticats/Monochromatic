/**
 A run's unit, built by the function the launcher builds it with, asked of this machine's manager: can a scope be started from
 inside it, and does the checkout stay read-only around it?

 Why this test exists: a review found that a `systemd-run --user --scope` started inside a run's unit ended with "Argument list too
 long" from the bus client while a direct write to the checkout was refused with `EROFS`. The cause was not in the properties this
 package writes: with the unit's own `XDG_RUNTIME_DIR` the same properties start a scope, and with a runtime directory that holds no
 manager socket every property set fails the same way, measured with the sets bisected. The unit now carries the runtime directory
 the launcher reached the manager through, and this test holds the two together: a unit whose environment does not happen to carry
 that folder still starts a scope.

 A machine with no user manager to answer is skipped with the reason it gave, so this test says something there rather than failing.

 @module
 */

import { execFile, } from 'node:child_process';
import {
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
  fileSystemOfARun,
  gitRunnerOf,
  makeTheFoldersAToolMakesIn,
  runCommandWithChildProcess,
  unitCommandOf,
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
 How long one question to the manager may take, in seconds.
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
    await mkdtemp(join(scratchRoot(), 'pi-scope-in-the-run-',),),
  );
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, maxRetries: 5, retryDelay: 200, },);
    },
  };
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
      `--unit=probe-scope-manager-${String(process.pid,)}`,
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
 What the probe in the unit wrote into the run's folder.
 */
type ScopeReport = {
  /**
   What a direct write into the checkout answered.
   */
  readonly checkoutWrite: { readonly status: number; readonly output: string };

  /**
   What a write into the run's worktree answered.
   */
  readonly worktreeWrite: { readonly status: number; readonly output: string };

  /**
   What starting a scope answered.
   */
  readonly scope: { readonly status: number; readonly output: string };

  /**
   What the scope's own write was read back as.
   */
  readonly scopedRead: { readonly status: number; readonly output: string };
};

/**
 A command that either answered or ended with a status, read as one value.

 @param call - the call, which either answers with its printed output or throws

 @returns the status it ended with, and what it printed either way

 @example
 ```ts
 const ended = await endedOrFailed({ call: async () => await execFileAsync('systemd-run', [...], {}), });
 ```
 */
async function endedOrFailed({ call }: { readonly call: () => Promise<{ readonly stdout: string; readonly stderr: string }> },): Promise<{
  readonly status: number;
  readonly output: string;
}> {
  try {
    const answered = await call();
    return { status: 0, output: `${answered.stdout}${answered.stderr}`, };
  }
  catch (error) {
    return {
      status: statusOf({ failure: error, },),
      output: `${textOf({ failure: error, name: 'stdout', },)}${textOf({ failure: error, name: 'stderr', },)}${textOf({ failure: error, name: 'message', },)}`,
    };
  }
}

/**
 One status a failed command reported.

 @param failure - what the command threw

 @returns that status, or one when it carried none

 @example
 ```ts
 const status = statusOf({ failure: error, });
 ```
 */
function statusOf({ failure }: { readonly failure: unknown },): number {
  /**
   What the failure carried where a status would be, which is only a number when the command ended with one.
   */
  const code: unknown = (((typeof failure) === 'object') && (failure !== null)) ? Reflect.get(failure, 'code',) : undefined;
  return ((typeof code) === 'number') ? code : (-1);
}

/**
 One text a failed command reported.

 @param failure - what the command threw

 @param name - the field's name

 @returns that text, or nothing when it was not one

 @example
 ```ts
 const printed = textOf({ failure: error, name: 'stderr', });
 ```
 */
function textOf({ failure, name }: { readonly failure: unknown; readonly name: string },): string {
  /**
   What the failure carried under that name, which is only a text when it reported one.
   */
  const value: unknown = (((typeof failure) === 'object') && (failure !== null)) ? Reflect.get(failure, name,) : undefined;
  return ((typeof value) === 'string') ? value : '';
}

/**
 Why every test here is skipped when this machine's manager does not answer, asked once for the whole file.
 */
const { reason: NO_MANAGER_REASON, } = await noManagerReason();

await describe({
  name: '',
  children: [
    describe({
      name: unitCommandOf.name,
      children: [
        it({
          name: 'starts a scope inside the unit the launcher builds, whose write lands in the run own worktree, while the checkout stays read-only',
          fn: async () => {
            if (NO_MANAGER_REASON !== undefined) {
              console.log(`[unitCommandOf] a scope inside the run's unit: skipped: ${NO_MANAGER_REASON}`,);
              return;
            }
            await using folder = await makeFolder();
            /**
             The checkout the run must not change, with one commit and the folder a tool makes in it.
             */
            const checkout = join(folder.path, 'checkout',);
            await mkdir(checkout, { recursive: true, },);
            await gitIn({ folder: checkout, words: ['init', '--initial-branch=main',], });
            await gitIn({
              folder: checkout,
              words: ['-c', 'user.name=Disposable', '-c', 'user.email=disposable@example.invalid', 'commit', '--allow-empty', '--message', 'base',],
            },);
            await writeFile(join(checkout, 'user-data.txt',), 'original user data\n',);
            /**
             The worktree the run works in, which Git made.
             */
            const worktree = join(checkout, '.claude', 'worktrees', 'pi-a',);
            await mkdir(join(checkout, '.claude', 'worktrees',), { recursive: true, },);
            await gitIn({ folder: checkout, words: ['worktree', 'add', '-b', 'agent-branch', worktree,], });
            /**
             The run's own folder.
             */
            const runFolder = join(folder.path, 'runs', 'a-1',);
            await mkdir(runFolder, { recursive: true, },);
            await makeTheFoldersAToolMakesIn({ checkoutFolder: checkout, },);
            /**
             The prompt file the unit's words name, which this probe never reads.
             */
            const promptFile = join(folder.path, 'prompt.md',);
            await writeFile(promptFile, 'a prompt this probe does not read\n',);
            /**
             What the run's unit may and may not write.
             */
            const fileSystem = await fileSystemOfARun({
              git,
              checkoutFolder: checkout,
              worktree,
              runFolder,
              worksInTheCheckoutItself: false,
            },);
            /**
             The words the launcher builds the run's unit with, and the probe this test runs in place of the driver.
             */
            const unit = unitCommandOf({
              runName: 'probe-scope-in-the-run-unit',
              cpuPercent: 200,
              memory: '2G',
              taskCount: 256,
              wallSeconds: 60,
              pathOfMiseEnvironment: '/usr/local/bin:/usr/bin:/bin',
              home: process.env.HOME ?? '',
              node: process.execPath,
              driverEntry: join(import.meta.dirname, 'scope-probe-inside-the-run-unit.ts',),
              runFolder,
              workingFolder: worktree,
              promptFile,
              provider: 'disposable',
              model: 'disposable',
              idleSeconds: 10,
              turnCap: 1,
              fileSystem,
              ...((process.env.XDG_RUNTIME_DIR === undefined) ? {} : { runtimeDirectory: process.env.XDG_RUNTIME_DIR, }),
            },);
            /**
             The launcher's own words with this test's unit name: everything else is what the launcher hands the manager, and the
             name is this test's so that no unit of a run's own name is started here.
             */
            const wordsThisTestOwns = unit.words.map(function namedForThisTest(word,) {
              return word.startsWith('--unit=',) ? `--unit=probe-scope-in-the-run-unit-${String(process.pid,)}` : word;
            },);
            // The unit carries the folder the launcher itself reached the manager through, so a run's scopes do not depend on the
            // manager's own environment having it. `UnsetEnvironment=` cannot model the other side of that, because it removes a
            // unit's own `Environment=` entry together with the inherited one (measured on systemd 262), so the word is asserted
            // here and its effect on the run is measured by the scope the probe starts.
            expect(unit.words,).toContain(`--setenv=XDG_RUNTIME_DIR=${String(process.env.XDG_RUNTIME_DIR,)}`,);
            /**
             What the unit itself answered.
             */
            const ended = await endedOrFailed({
              call: async function runTheUnit() {
                return await execFileAsync(unit.command, [
                  ...wordsThisTestOwns,
                  `--checkout=${checkout}`,
                ], {
                  encoding: 'utf8',
                  timeout: 120_000,
                  killSignal: 'SIGKILL',
                },);
              },
            },);
            // Nine answers about a unit that ended in its own words are worth more than a status alone, so what it printed is kept
            // for the failure that names it.
            expect({ status: ended.status, printed: ended.output.slice(0, 300), },).toEqual({ status: 0, printed: ended.output.slice(0, 300), },);
            expect(ended.status,).toBe(0,);
            /**
             What the probe wrote, which says what each attempt answered.
             */
            const report = JSON.parse(await readFile(join(runFolder, 'scope-report.json',), 'utf8',),) as ScopeReport;
            // The checkout is read-only in this unit: the direct write ends with the operating system's own refusal.
            expect({ refused: report.checkoutWrite.status !== 0, output: report.checkoutWrite.output.slice(0, 300), },).toEqual({
              refused: true,
              output: report.checkoutWrite.output.slice(0, 300),
            },);
            expect(report.checkoutWrite.output,).toContain('EROFS',);
            // The run's own worktree is writable, and what was written there reads back.
            expect(report.worktreeWrite,).toEqual({ status: 0, output: 'written in the worktree', },);
            // The scope started, which is what every brief on this machine asks a run to do, and its own write landed in the run's
            // worktree and reads back.
            expect({ scope: report.scope.status, output: report.scope.output.slice(0, 300), },).toEqual({ scope: 0, output: '', },);
            expect(report.scopedRead,).toEqual({ status: 0, output: 'written by a scope', },);
            // The checkout's own file is untouched, and no run wrote into it through the scope.
            expect(await readFile(join(checkout, 'user-data.txt',), 'utf8',),).toBe('original user data\n',);
          },
        },),
      ],
    },),
  ],
},);
