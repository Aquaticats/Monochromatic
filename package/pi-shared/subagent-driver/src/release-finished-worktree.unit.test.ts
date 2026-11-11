/**
 Releasing the worktree of a run that finished its work, against throwaway repositories that use their own home directory.

 Each test makes a repository, a branch and a worktree, then hands the release the run folder a finished run leaves.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  mkdir,
  mkdtemp,
  readdir,
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
  type CommandOutput,
  endedWithItsWork,
  NO_WORKTREE_BRANCH,
  releaseFinishedWorktree,
  runCommandWithChildProcess,
  startAPiRun,
  type StartPiRunOptions,
} from '../dist/final/node/index.mjs';

/**
 `execFile` with a promise result, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 The name and address a throwaway repository's commits are made under.
 */
const COMMIT_NAME = 'a test';
const COMMIT_ADDRESS = 'test@example.invalid';

/**
 The commit arguments every throwaway commit needs, so the test needs no global Git configuration.
 */
const COMMIT_IDENTITY = ['-c', `user.name=${COMMIT_NAME}`, '-c', `user.email=${COMMIT_ADDRESS}`,];

/**
 One throwaway machine: a home directory and a runtime directory of its own, so no wrapper or credential of this machine is
 reached, and a repository with a commit and a worktree on a branch.
 */
type Throwaway = {
  /**
   The home directory the test's Git commands run with.
   */
  readonly home: string;

  /**
   Where the test's Git commands keep their sockets.
   */
  readonly runtime: string;

  /**
   The repository the worktree is made from.
   */
  readonly repository: string;

  /**
   The branch the worktree checks out.
   */
  readonly branch: string;

  /**
   The worktree itself.
   */
  readonly worktree: string;

  /**
   The folder a finished run leaves, which names the run's end.
   */
  readonly runFolder: string;

  /**
   How a command is run, with this machine's own home and sockets, so nothing of the test's machine is read.
   */
  readonly run: typeof runCommandWithChildProcess;

  /**
   Removes the whole throwaway machine.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Reads one field of a caught value, which is an error object this cannot name.

 @param caught - what was caught

 @param key - the field's name

 @returns the field's value, as anything

 @example
 ```ts
 const code = fieldOf({ caught, key: 'code', },);
 ```
 */
function fieldOf({
  caught,
  key,
}: {
  readonly caught: object;
  readonly key: string;
},): unknown {
  return Reflect.get(
    caught,
    key,
  );
}

/**
 Runs one Git command in a throwaway machine and insists on an answer.

 @param machine - the machine to run in

 @param words - the command's arguments

 @param cwd - the folder to run in

 @returns what Git printed

 @example
 ```ts
 const log = await gitIn({ machine, words: ['log', '--oneline',], cwd: machine.repository, });
 ```
 */
async function gitIn({
  machine,
  words,
  cwd,
}: {
  readonly machine: Throwaway;
  readonly words: readonly string[];
  readonly cwd: string;
},): Promise<string> {
  const done = await execFileAsync(
    'git',
    [...words,],
    {
      cwd,
      encoding: 'utf8',
      env: {
        ...process.env,
        HOME: machine.home,
        XDG_RUNTIME_DIR: machine.runtime,
        GIT_ASKPASS: '',
        GIT_TERMINAL_PROMPT: '0',
      },
    },
  );
  return done.stdout;
}

/**
 Runs one Git command in a throwaway machine and answers the status it exited with.

 @param machine - the machine to run in

 @param words - the command's arguments

 @param cwd - the folder to run in

 @returns the status Git exited with

 @example
 ```ts
 const status = await gitStatusIn({ machine, words: ['status',], cwd: machine.repository, });
 ```
 */
async function gitStatusIn({
  machine,
  words,
  cwd,
}: {
  readonly machine: Throwaway;
  readonly words: readonly string[];
  readonly cwd: string;
},): Promise<number> {
  try {
    await gitIn({
      machine,
      words,
      cwd,
    },);
    return 0;
  }
  catch (error) {
    /**
     The status the failed command exited with.
     */
    const status = fieldOf({
      caught: error as object,
      key: 'code',
    },);
    return ((typeof status) === 'number') ? status : -1;
  }
}

/**
 Reads whether a path is there.

 @param path - the path to read

 @returns whether the file system holds it

 @example
 ```ts
 const there = await pathIsThere({ path: '/a/b', });
 ```
 */
async function pathIsThere({
  path,
}: {
  readonly path: string;
},): Promise<boolean> {
  try {
    await readdir(path,);
    return true;
  }
  catch (error) {
    if (fieldOf({
      caught: error as object,
      key: 'code',
    },) === 'ENOENT') {
      return false;
    }
    throw error;
  }
}

/**
 Makes the throwaway machine one test runs against.

 @returns the machine, removed when the test ends

 @example
 ```ts
 await using machine = await throwaway({});
 ```
 */
async function throwaway(): Promise<Throwaway> {
  /**
   The folder this machine lives in.
   */
  const root = await mkdtemp(join(
    scratchRoot(),
    'pi-worktree-release-',
  ),);
  /**
   The home directory, which is the only one the test's Git commands read.
   */
  const home = join(root, 'home',);
  /**
   The runtime directory of the test's Git commands.
   */
  const runtime = join(root, 'runtime',);
  /**
   The repository the worktree is made from.
   */
  const repository = join(root, 'repository',);
  /**
   The branch the worktree checks out.
   */
  const branch = 'feat/a';
  /**
   The worktree the run worked in.
   */
  const worktree = join(
    repository,
    '.claude',
    'worktrees',
    'pi-a',
  );
  /**
   The folder a finished run leaves.
   */
  const runFolder = join(root, 'runs', 'a-1',);
  await mkdir(home, { recursive: true, },);
  await mkdir(runtime, { recursive: true, },);
  await mkdir(repository, { recursive: true, },);
  await mkdir(runFolder, { recursive: true, },);
  /**
   How a command runs in this machine, with its own home and sockets and no prompt for a credential.
   */
  async function run(request: {
    readonly command: string;
    readonly words: readonly string[];
    readonly cwd?: string;
  }): Promise<CommandOutput> {
    return await runCommandWithChildProcess({
      ...request,
      environment: {
        HOME: home,
        XDG_RUNTIME_DIR: runtime,
        GIT_ASKPASS: '',
        GIT_TERMINAL_PROMPT: '0',
      },
    },);
  }
  /**
   The machine itself, before its repository is filled.
   */
  const machine: Throwaway = {
    home,
    runtime,
    repository,
    branch,
    worktree,
    runFolder,
    run,
    async [Symbol.asyncDispose]() {
      await rm(
        root,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
  await gitIn({
    machine,
    words: ['init', '--quiet', '--initial-branch=main',],
    cwd: repository,
  },);
  await writeFile(
    join(repository, 'tracked.txt',),
    'the work main holds',
    { encoding: 'utf8', },
  );
  await gitIn({
    machine,
    words: ['add', 'tracked.txt',],
    cwd: repository,
  },);
  /**
   The tree the first commit holds, which the wrapper's commit command is not needed to write: a fixture commit must not reach
   the commit service this machine runs, and `write-tree`, `commit-tree` and `update-ref` are plumbing that writes the same
   commit without it.
   */
  const tree = (await gitIn({
    machine,
    words: ['write-tree',],
    cwd: repository,
  },)).trim();
  /**
   The first commit itself, made under the fixture's own identity.
   */
  const firstCommit = (await gitIn({
    machine,
    words: [...COMMIT_IDENTITY, 'commit-tree', tree, '-m', 'the work',],
    cwd: repository,
  },)).trim();
  await gitIn({
    machine,
    words: ['update-ref', 'refs/heads/main', firstCommit,],
    cwd: repository,
  },);
  await gitIn({
    machine,
    words: ['worktree', 'add', '--quiet', '-b', branch, worktree,],
    cwd: repository,
  },);
  return machine;
}

/**
 Writes the summary a finished run leaves.

 @param machine - the machine whose run folder is written

 @param endReason - the reason the run's summary names

 @example
 ```ts
 await writeSummary({ machine, endReason: 'idle', });
 ```
 */
async function writeSummary({
  machine,
  endReason,
}: {
  readonly machine: Throwaway;
  readonly endReason: string;
},): Promise<void> {
  await writeFile(
    join(machine.runFolder, 'summary.json',),
    JSON.stringify({ endReason, turns: 3, },),
    { encoding: 'utf8', },
  );
}


/**
 Starts one run in a throwaway machine, with `mise` answered here so no install of this machine's dependencies is started.

 @param machine - the machine whose worktree the run works in

 @param endReason - what the run's summary says it ended on

 @param branch - the branch the run works on, or the repository itself

 @returns what the start answered

 @example
 ```ts
 const outcome = await startInFixture({ machine, endReason: 'idle', });
 ```
 */
async function startInFixture({
  machine,
  endReason,
  branch = machine.branch,
}: {
  readonly machine: Throwaway;
  readonly endReason: string;
  readonly branch?: string;
}): Promise<{ readonly lines: readonly string[] }> {
  await writeSummary({
    machine,
    endReason,
  },);
  /**
   How a command is run for this start: Git in the fixture's own home, and `mise` answered here.
   */
  async function run(request: {
    readonly command: string;
    readonly words: readonly string[];
    readonly cwd?: string;
    readonly environment?: Readonly<Record<string, string>>;
  }): Promise<CommandOutput> {
    if (request.command === 'mise') {
      return { status: 0, stdout: 'node\n', stderr: '', };
    }
    return await machine.run(request,);
  }
  /**
   What the start was asked for, working in the fixture's worktree.
   */
  const options: StartPiRunOptions = {
    name: 'a',
    branch,
    promptFile: join(machine.runFolder, 'prompt.md',),
    run: 1,
    provider: 'neuralwatt',
    model: 'deepseek-v4.1-flash',
    cpuPercent: 800,
    memory: '12G',
    tasks: 3_000,
    wallSeconds: 600,
    idleSeconds: 120,
    runsFolder: join(machine.runFolder, '..', '..', 'runs',),
    repository: machine.repository,
    driverEntry: '/driver.ts',
    turnCap: 8,
  };
  return await startAPiRun({
    options,
    run,
    runToTheEnd: async function unit() {
      return { status: 0, stdout: '', stderr: '', };
    },
    launchers: async function none() {
      return new Map();
    },
    runningUnits: async function units() {
      return [];
    },
    sizeATurnCap: async function sized() {
      return {
        cap: 8,
        longest: 6,
        sampleSize: 1,
        kind: 'other',
        sampleKind: 'every kind',
        line: 'turn cap 8',
      };
    },
    home: machine.home,
  },);
}

await describe({
  name: 'releasing a finished run\'s worktree',
  timeout: 180_000,
  children: [
    //region Which runs finished their work
    describe({
      name: endedWithItsWork.name,
      children: [
        it({
          name: 'counts a run that came back and one the stop file ended as finished, and no other end',
          fn: async () => {
            expect(endedWithItsWork({ endReason: 'idle', },),).toBe(true,);
            expect(endedWithItsWork({ endReason: 'stop-file', },),).toBe(true,);
            expect(endedWithItsWork({ endReason: 'turn-cap', },),).toBe(false,);
            expect(endedWithItsWork({ endReason: 'wall-clock', },),).toBe(false,);
            expect(endedWithItsWork({ endReason: 'model-error', },),).toBe(false,);
          },
        },),
      ],
    },),
    //endregion

    //region Removing, and keeping
    describe({
      name: releaseFinishedWorktree.name,
      children: [
        it({
          name: 'removes the worktree of a run that finished with nothing uncommitted, and keeps its branch',
          fn: async () => {
            await using machine = await throwaway();
            await writeSummary({
              machine,
              endReason: 'idle',
            },);
            const release = await releaseFinishedWorktree({
              run: machine.run,
              repository: machine.repository,
              worktree: machine.worktree,
              runFolder: machine.runFolder,
            },);
            expect(release.released,).toBe(true,);
            expect(await pathIsThere({ path: machine.worktree, },),).toBe(false,);
            expect(await gitStatusIn({
              machine,
              words: ['rev-parse', '--verify', '--quiet', `refs/heads/${machine.branch}`,],
              cwd: machine.repository,
            },),).toBe(0,);
          },
        },),
        it({
          name: 'keeps a worktree that holds uncommitted work, and names the paths that kept it',
          fn: async () => {
            await using machine = await throwaway();
            await writeSummary({
              machine,
              endReason: 'idle',
            },);
            await writeFile(
              join(machine.worktree, 'untracked.txt',),
              'work nobody committed',
              { encoding: 'utf8', },
            );
            const release = await releaseFinishedWorktree({
              run: machine.run,
              repository: machine.repository,
              worktree: machine.worktree,
              runFolder: machine.runFolder,
            },);
            expect(release.released,).toBe(false,);
            expect(release.lines.join('\n',),).toContain('untracked.txt',);
            expect(await pathIsThere({ path: machine.worktree, },),).toBe(true,);
          },
        },),
        it({
          name: 'keeps the worktree of a run that ended on its turn cap, even with nothing uncommitted',
          fn: async () => {
            await using machine = await throwaway();
            await writeSummary({
              machine,
              endReason: 'turn-cap',
            },);
            const release = await releaseFinishedWorktree({
              run: machine.run,
              repository: machine.repository,
              worktree: machine.worktree,
              runFolder: machine.runFolder,
            },);
            expect(release.released,).toBe(false,);
            expect(release.lines.join('\n',),).toContain('turn-cap',);
            expect(await pathIsThere({ path: machine.worktree, },),).toBe(true,);
          },
        },),
        it({
          name: 'keeps a worktree whose HEAD is detached, because its commits may be reachable from there alone',
          fn: async () => {
            await using machine = await throwaway();
            await writeSummary({
              machine,
              endReason: 'idle',
            },);
            await gitIn({
              machine,
              words: ['checkout', '--quiet', '--detach',],
              cwd: machine.worktree,
            },);
            const release = await releaseFinishedWorktree({
              run: machine.run,
              repository: machine.repository,
              worktree: machine.worktree,
              runFolder: machine.runFolder,
            },);
            expect(release.released,).toBe(false,);
            expect(release.lines.join('\n',),).toContain('detached',);
            expect(await pathIsThere({ path: machine.worktree, },),).toBe(true,);
          },
        },),
        it({
          name: 'keeps a worktree whose run left no summary, because nothing says the run finished',
          fn: async () => {
            await using machine = await throwaway();
            const release = await releaseFinishedWorktree({
              run: machine.run,
              repository: machine.repository,
              worktree: machine.worktree,
              runFolder: machine.runFolder,
            },);
            expect(release.released,).toBe(false,);
            expect(release.lines.join('\n',),).toContain('no summary',);
            expect(await pathIsThere({ path: machine.worktree, },),).toBe(true,);
          },
        },),
      ],
    },),
    //endregion

    //region The start command that releases it
    describe({
      name: startAPiRun.name,
      children: [
        it({
          name: 'gives the worktree back when the run it started finished its work',
          fn: async () => {
            await using machine = await throwaway();
            const outcome = await startInFixture({
              machine,
              endReason: 'idle',
            },);
            expect(outcome.lines.join('\n',),).toContain('worktree removed',);
            expect(await pathIsThere({ path: machine.worktree, },),).toBe(false,);
          },
        },),
        it({
          name: 'keeps the worktree of a run that ended on its turn cap, and the line says why',
          fn: async () => {
            await using machine = await throwaway();
            const outcome = await startInFixture({
              machine,
              endReason: 'turn-cap',
            },);
            expect(outcome.lines.join('\n',),).toContain('worktree kept (the run ended turn-cap)',);
            expect(await pathIsThere({ path: machine.worktree, },),).toBe(true,);
          },
        },),
        it({
          name: 'says nothing about a worktree for a run that works in the repository itself',
          fn: async () => {
            await using machine = await throwaway();
            const outcome = await startInFixture({
              machine,
              endReason: 'idle',
              branch: NO_WORKTREE_BRANCH,
            },);
            expect(outcome.lines.join('\n',),).not.toContain('worktree kept',);
            expect(outcome.lines.join('\n',),).not.toContain('worktree removed',);
          },
        },),
      ],
    },),
    //endregion
  ],
},);
