/**
 `start-reviews-of-finished-branches` against a disposable repository: a branch whose brief cannot be written ends the command with
 a failure, says why in one line, and starts nothing.

 The repository is built with `git fast-import`, which is plumbing and needs no commit service, so this runs on a machine whose Git
 wrapper refuses to commit without a user service manager. Nothing here starts a Pi run or a systemd unit: the only branch asked
 for is refused before any start is made.

 @module
 */

import {
  execFile,
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  mkdtemp,
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

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

/**
 The command this file runs, from source the way its own task runs it.
 */
const COMMAND = new URL('start-reviews-of-finished-branches.ts', import.meta.url,).pathname;

/**
 The status a command that refused ends with.
 */
const REFUSED_STATUS = 1;

/**
 The branch whose review cannot be briefed, and the review it is read as.
 */
const BRANCH = 'fix/brief-failure';

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
  const path = await mkdtemp(join(scratchRoot(), 'pi-start-reviews-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Reads the environment every Git and the command run in: the fixture's own home, no session bus, no configuration of this machine,
 and no prompt.

 @param folder - the fixture's folder

 @returns the environment

 @example
 ```ts
 const environment = fixtureEnvironmentOf(folder.path);
 ```
 */
function fixtureEnvironmentOf(folder: string,): Record<string, string> {
  /**
   The environment.
   */
  const environment: Record<string, string> = {};
  for (const [name, value,] of Object.entries(process.env,)) {
    if (value !== undefined) {
      environment[name] = value;
    }
  }
  environment.HOME = join(folder, 'home',);
  environment.XDG_RUNTIME_DIR = join(folder, 'runtime',);
  environment.XDG_CACHE_HOME = join(folder, 'home', '.cache',);
  environment.GIT_CONFIG_GLOBAL = '/dev/null';
  environment.GIT_CONFIG_SYSTEM = '/dev/null';
  environment.GIT_TERMINAL_PROMPT = '0';
  environment.GIT_ASKPASS = '';
  // A fixture's own home does not keep a wrapper off this machine's bus, and no fixture here needs a service manager.
  delete environment.DBUS_SESSION_BUS_ADDRESS;
  return environment;
}

/**
 Runs one Git command in a fixture.

 @param folder - the fixture's folder

 @param words - the command's words

 @returns what it printed

 @example
 ```ts
 const head = await gitIn({ folder: repo, words: ['rev-parse', 'main'] },);
 ```
 */
async function gitIn({
  folder,
  words,
}: {
  readonly folder: string;
  readonly words: readonly string[];
}): Promise<string> {
  /**
   What the command answered.
   */
  const answered = await execFileAsync(
    'git',
    [...words,],
    {
      cwd: folder,
      env: fixtureEnvironmentOf(folder,),
      encoding: 'utf8',
    },
  );
  return answered.stdout;
}

/**
 Adds one commit through `git fast-import`, whose stream is fed on the command's standard input.

 @param folder - the fixture's folder

 @param branch - the ref the commit lands on

 @param message - the commit's message

 @param files - the paths to write, each with its content

 @param from - the commit this one follows, or nothing for the first commit of a ref

 @returns nothing

 @example
 ```ts
 await fastImportCommit({ folder: repo, branch: 'main', message: 'base', files: new Map([['src/gate.ts', 'a']]), },);
 ```
 */
async function fastImportCommit({
  folder,
  branch,
  message,
  files,
  from,
}: {
  readonly folder: string;
  readonly branch: string;
  readonly message: string;
  readonly files: ReadonlyMap<string, string>;
  readonly from?: string;
}): Promise<void> {
  /**
   The lines of the stream fast-import reads.
   */
  const lines: string[] = [
    `commit refs/heads/${branch}`,
    'committer probe <probe@example.invalid> 1700000000 +0000',
    `data ${String(Buffer.byteLength(message, 'utf8',),)}`,
    message,
  ];
  if (from !== undefined) {
    lines.push(`from ${from}`,);
  }
  for (const [path, content,] of files) {
    lines.push(
      `M 100644 inline ${path}`,
      `data ${String(Buffer.byteLength(content, 'utf8',),)}`,
      content,
    );
  }
  lines.push('',);
  /**
   The command's process.
   */
  const child = spawn(
    'git',
    ['fast-import', '--quiet',],
    {
      cwd: folder,
      env: fixtureEnvironmentOf(folder,),
    },
  );
  /**
   What it said on standard error, for a refusal, one chunk at a time.
   */
  const said: string[] = [];
  child.stderr.setEncoding('utf8',);
  child.stderr.on('data', function remember(chunk: string,) {
    said.push(chunk,);
  },);
  child.stdin.end(`${lines.join('\n',)}\n`,);
  /**
   What the process reported when it closed.
   */
  const closed: readonly unknown[] = await once(
    child,
    'close',
  );
  if (closed[0] !== 0) {
    throw new Error(`git fast-import answered ${String(closed[0],)}: ${said.join('',)}`,);
  }
}

/**
 Builds a disposable repository whose `main` holds a base commit and whose review branch changes one file.

 @param folder - the test's folder

 @returns the repository's folder

 @example
 ```ts
 const repo = await makeRepository(folder.path);
 ```
 */
async function makeRepository(folder: string,): Promise<string> {
  /**
   The repository's folder.
   */
  const repo = join(folder, 'repo',);
  await mkdir(repo, { recursive: true, },);
  await mkdir(join(folder, 'home',), { recursive: true, },);
  await mkdir(join(folder, 'runtime',), { recursive: true, },);
  await gitIn({
    folder: repo,
    words: ['init', '--quiet', '--initial-branch=main',],
  },);
  await fastImportCommit({
    folder: repo,
    branch: 'main',
    message: 'base',
    files: new Map([['src/gate.ts', 'export const version = "base";\n',],],),
  },);
  /**
   The base commit.
   */
  const base = (await gitIn({
    folder: repo,
    words: ['rev-parse', 'main',],
  },)).trim();
  await fastImportCommit({
    folder: repo,
    branch: BRANCH,
    message: 'change the gate',
    files: new Map([['src/gate.ts', 'export const version = "changed";\n',],],),
    from: base,
  },);
  return repo;
}

/**
 Runs the command for the fixture's branch.

 @param repo - the fixture's folder

 @param reviewsFolder - where the review's own files go

 @param runsFolder - where the runs go

 @returns the status it ended with and both streams

 @example
 ```ts
 const ran = await runTheCommand({ repo, reviewsFolder, runsFolder });
 ```
 */
async function runTheCommand({
  repo,
  reviewsFolder,
  runsFolder,
}: {
  readonly repo: string;
  readonly reviewsFolder: string;
  readonly runsFolder: string;
}): Promise<{ readonly status: number; readonly printed: string }> {
  try {
    /**
     What the command printed.
     */
    const answered = await execFileAsync(
      process.execPath,
      [
        COMMAND,
        `--branch=${BRANCH}`,
        '--integration=main',
        '--source=src',
        `--reviews-folder=${reviewsFolder}`,
        `--runs-folder=${runsFolder}`,
      ],
      {
        cwd: repo,
        env: fixtureEnvironmentOf(repo,),
        encoding: 'utf8',
        timeout: 60_000,
        killSignal: 'SIGKILL',
      },
    );
    return {
      status: 0,
      printed: `${answered.stdout}${answered.stderr}`,
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
      status: (typeof failed.code) === 'number' ? failed.code : REFUSED_STATUS,
      printed: `${String(failed.stdout,)}${String(failed.stderr,)}`,
    };
  }
}

await describe({
  name: '',
  children: [
    describe({
      name: COMMAND.slice(COMMAND.lastIndexOf('/',) + 1,),
      children: [
        it({
          name: 'ends with a failure, says why in one line, and starts nothing when the only branch\'s brief cannot be written',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The fixture's repository.
             */
            const repo = await makeRepository(folder.path,);
            /**
             Where the review's own files go, with a regular file where the brief writer would make the reproductions folder.
             */
            const reviewsFolder = join(folder.path, 'reviews',);
            await mkdir(join(reviewsFolder, 'brief-failure',), { recursive: true, },);
            await writeFile(join(reviewsFolder, 'brief-failure', 'reproductions',), 'a regular file blocks the brief writer\n',);
            /**
             What the command answered.
             */
            const ran = await runTheCommand({
              repo,
              reviewsFolder,
              runsFolder: join(folder.path, 'runs',),
            },);
            expect(ran.status,).toBe(REFUSED_STATUS,);
            expect(ran.printed,).toContain('its brief could not be written',);
            expect(ran.printed,).toContain('EEXIST',);
            expect(ran.printed,).not.toContain(': started',);
            expect(ran.printed,).toContain('reviews ended with statuses none',);
          },
          timeout: 120_000,
        },),
      ],
    },),
  ],
},);
