/**
 Test of the cut-over stage that refreshes the moved checkouts: a status in every registered one, which rewrites its
 index, and `mise exec -- pnpm install --offline --frozen-lockfile` in every one whose `node_modules` holds a link that
 points nowhere.

 The stage runs real Git on real worktrees, with a stand-in `mise` and `pnpm` on the search path, because only the
 install is third-party: the stand-in repairs every dangling link the way a new install does, and records the checkout
 it was run in.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  symlink,
  utimes,
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
  createGuidedConfig,
  cutoverStages,
  execFileAsync,
  runShell,
  type GuidedConfig,
  type Stage,
} from '../dist/final/node/index.mjs';

/**
 The stage that refreshes the moved checkouts.
 */
const REFRESH_STAGE_ID = 'c20c-refresh-checkouts';

/**
 The stage that makes the two links, before which the checkouts are the old ones.
 */
const SWAP_STAGE_ID = 'c19-swap';

/**
 The stage whose table and list are compared with the frozen baseline, which reads the index hashes.
 */
const THROUGH_LINK_STAGE_ID = 'c20-through-link';

/**
 The stage that starts the commit sockets again, after which commits happen.
 */
const RELEASE_STAGE_ID = 'c21-release';

/**
 The folder of a nested worktree, below the checkout, whose index is left alone.
 */
const NESTED_WORKTREE = join('.claude', 'worktrees', 'a',);

/**
 Name of the branch the nested worktree is on.
 */
const NESTED_BRANCH = 'a';

/**
 The second worktree, which has no `node_modules`.
 */
const BARE_WORKTREE = join('worktrees', 'b',);

/**
 Name of the branch the second worktree is on.
 */
const BARE_BRANCH = 'b';

/**
 A file the fixture repository tracks, whose touch makes Git rewrite the index of a status.
 */
const TRACKED_FILE = 'tracked.txt';

/**
 A link under `node_modules` that points nowhere after a move.
 */
const DANGLING_LINK = join('node_modules', 'dep',);

/**
 Where the dangling link points, relative to its own folder, as a moved package link does.
 */
const DANGLING_TARGET = join('..', '..', '..', '.pnpm-store', 'dep',);

/**
 A link under `node_modules` that resolves, so its checkout needs no install.
 */
const WORKING_LINK = join('node_modules', 'kotlin',);

/**
 Exit status of a command that succeeded.
 */
const COMMAND_SUCCEEDED = 0;

/**
 Exit status of a check that failed on its own terms.
 */
const CHECK_FAILED = 1;

/**
 Permissions of a stand-in program: owner may read and run it.
 */
const STAND_IN_MODE = 0o700;

/**
 Deadline of a shell run, in seconds: the stage reads a few worktrees.
 */
const SHELL_DEADLINE_SECONDS = 120;

/**
 Seconds between terminate and kill for a shell run.
 */
const SHELL_KILL_GRACE_SECONDS = 5;

/**
 What the stand-in `pnpm` does.
 */
type Install = 'repair' | 'do-nothing';

/**
 Seconds of a Git call: the fixture is small.
 */
const GIT_DEADLINE_SECONDS = 60;

/**
 A disposable home folder, removed when the scope ends.
 */
type HomeFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a disposable home folder.

 @returns the folder, to hold with `await using`

 @example
 ```ts
 await using home = await makeHome();
 ```
 */
async function makeHome(): Promise<HomeFolder> {
  /**
   Path of the new folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'guided-refresh-checkouts-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Runs Git in a folder of the fixture.

 @param directory - the folder

 @param args - Git arguments

 @returns the standard output

 @example
 ```ts
 await run('/x', ['status', '--porcelain']);
 ```
 */
async function run(directory: string, args: readonly string[],): Promise<string> {
  /**
   What the call returned.
   */
  const result = await execFileAsync('/usr/bin/git', ['-C', directory, ...args,], {
    encoding: 'utf8',
    timeout: GIT_DEADLINE_SECONDS * 1_000,
  },);
  return result.stdout;
}

/**
 Makes the fixture: a checkout with one nested worktree whose `node_modules` dangles, and a second worktree with none.

 @param path - the checkout's path

 @returns nothing; the fixture is on disk

 @example
 ```ts
 await makeRepository('/x/Monochromatic');
 ```
 */
async function makeRepository(path: string,): Promise<void> {
  await mkdir(path, { recursive: true, },);
  await execFileAsync('/usr/bin/git', ['init', '--quiet', '--initial-branch=main', path,], { encoding: 'utf8', },);
  await writeFile(join(path, TRACKED_FILE,), 'the fixture repository\n',);
  await run(path, ['add', TRACKED_FILE,],);
  await run(path, [
    '-c',
    'user.email=test@example.invalid',
    '-c',
    'user.name=test',
    'commit',
    '--quiet',
    '--message=fixture',
  ],);
  await run(path, ['worktree', 'add', '--quiet', join(path, NESTED_WORKTREE,), '-b', NESTED_BRANCH,],);
  await run(path, ['worktree', 'add', '--quiet', join(path, BARE_WORKTREE,), '-b', BARE_BRANCH,],);
  // A node_modules that resolves, so this checkout needs no install.
  await mkdir(join(path, 'node_modules',), { recursive: true, },);
  await symlink(join('..', TRACKED_FILE,), join(path, WORKING_LINK,),);
  // The nested worktree's node_modules points nowhere, the way a moved checkout's relative links do.
  await mkdir(join(path, NESTED_WORKTREE, 'node_modules',), { recursive: true, },);
  await symlink(DANGLING_TARGET, join(path, NESTED_WORKTREE, DANGLING_LINK,),);
}

/**
 Writes the stand-in `mise` and `pnpm` the stage finds first on its search path.

 @param bin - the folder the stand-ins go in

 @param calls - the file the stand-in `pnpm` records the checkout it ran in

 @param install - what the stand-in `pnpm` does

 @returns nothing

 @example
 ```ts
 await makeStandIns('/x/bin', '/x/pnpm-calls.txt', 'repair');
 ```
 */
async function makeStandIns(
  {
    bin,
    calls,
    install,
  }: {
    readonly bin: string;
    readonly calls: string;
    readonly install: Install;
  },
): Promise<void> {
  await mkdir(bin, { recursive: true, },);
  await writeFile(join(bin, 'mise',), `#!/usr/bin/env bash
# The stage calls \`mise exec -- pnpm ...\`; this runs the program after the two words.
shift 2
exec "$@"
`,);
  /**
   What the stand-in does for each link under `node_modules`, which for a repair makes every dangling one resolve.
   */
  const repair = install === 'repair'
    ? `for link in node_modules/*; do
  if [ -L "$link" ] && [ ! -e "$link" ]; then
    mkdir --parents "$(realpath --canonicalize-missing -- "$link")"
  fi
done
`
    : '';
  await writeFile(join(bin, 'pnpm',), `#!/usr/bin/env bash
printf '%s\\n' "$PWD" >> '${calls}'
${repair}exit 0
`,);
  await chmod(join(bin, 'mise',), STAND_IN_MODE,);
  await chmod(join(bin, 'pnpm',), STAND_IN_MODE,);
}

/**
 The stage, its configuration and the home it reads.

 @returns the home, the configuration and the stage's command and check

 @example
 ```ts
 const subject = await refreshStage();
 ```
 */
async function refreshStage(): Promise<{
  readonly home: HomeFolder;
  readonly config: GuidedConfig;
  readonly command: string;
  readonly check: string;
}> {
  /**
   The disposable home this invocation reads.
   */
  const home = await makeHome();
  /**
   The configuration the stages read.
   */
  const config = await createGuidedConfig({
    env: {
      HOME: home.path,
      MOVE_DRIVE: join(home.path, 'drive',),
      MOVE_DESTINATION_ROOT: join(home.path, 'drive', 'repository',),
    },
    nodePath: process.execPath,
  },);
  await mkdir(config.wiz, { recursive: true, },);
  /**
   The cut-over stages.
   */
  const stages = await cutoverStages({ c: config, stamp: 'S', },);
  /**
   The stage that refreshes the checkouts.
   */
  const stage = stages.find((candidate: Stage,) => candidate.id === REFRESH_STAGE_ID,);
  if (stage === undefined)
    throw new Error(`the cut-over has no stage ${REFRESH_STAGE_ID}`,);
  if (((typeof stage.command) !== 'string') || ((typeof stage.check) !== 'string'))
    throw new Error(`stage ${REFRESH_STAGE_ID} has no plain command or check text`,);
  return {
    home,
    config,
    command: stage.command,
    check: stage.check,
  };
}

/**
 Runs the stage and then its check, with the stand-ins first on the search path.

 @param subject - the stage, its home and its configuration

 @param bin - the folder of the stand-ins

 @returns the command's and the check's exit status and everything they printed

 @example
 ```ts
 const { commandStatus, checkStatus, } = await runStage({ subject, bin });
 ```
 */
async function runStage(
  {
    subject,
    bin,
  }: {
    readonly subject: {
      readonly home: HomeFolder;
      readonly command: string;
      readonly check: string;
    };
    readonly bin: string;
  },
): Promise<{
  readonly commandStatus: number;
  readonly checkStatus: number;
  readonly printed: string;
}> {
  /**
   What the two runs printed.
   */
  const printed: string[] = [];
  const searchPath = firstOnPath(bin,);
  const commandStatus = await runShell({
    command: subject.command,
    outputFile: join(subject.home.path, 'command.out',),
    seconds: SHELL_DEADLINE_SECONDS,
    killGrace: SHELL_KILL_GRACE_SECONDS,
    outputFileEnv: join(subject.home.path, 'command.out',),
    out: (text: string,) => {
      printed.push(text,);
    },
  },);
  const checkStatus = await runShell({
    command: subject.check,
    outputFile: join(subject.home.path, 'check.out',),
    seconds: SHELL_DEADLINE_SECONDS,
    killGrace: SHELL_KILL_GRACE_SECONDS,
    outputFileEnv: join(subject.home.path, 'check.out',),
    out: (text: string,) => {
      printed.push(text,);
    },
  },);
  searchPath.putBack();
  return {
    commandStatus,
    checkStatus,
    printed: printed.join('',),
  };
}

/**
 Puts a folder first on this process's search path.

 @param bin - the folder to put first

 @returns the value that puts the previous search path back

 @example
 ```ts
 const searchPath = firstOnPath(bin);
 searchPath.putBack();
 ```
 */
function firstOnPath(bin: string,): { readonly putBack: () => void } {
  const previous = process.env.PATH;
  process.env.PATH = `${bin}:${previous ?? ''}`;
  return {
    putBack() {
      if (previous === undefined) delete process.env.PATH;
      else process.env.PATH = previous;
    },
  };
}

/**
 Reads a file the stage wrote, or an empty string when it did not.

 @param path - the file

 @returns the text

 @example
 ```ts
 const text = await readOrEmpty(path);
 ```
 */
async function readOrEmpty(path: string,): Promise<string> {
  try {
    return await readFile(path, 'utf8',);
  }
  catch (error) {
    /**
     A missing file is the answer this reader reports, not a failure.
     */
    console.error(`${path} could not be read: ${String(error,)}`,);
    return '';
  }
}

/**
 The lines of a file the stage wrote, without the empty ones.

 @param path - the file

 @returns the lines

 @example
 ```ts
 const lines = await linesOf(path);
 ```
 */
async function linesOf(path: string,): Promise<readonly string[]> {
  return (await readOrEmpty(path,)).split('\n',)
    .filter(function isLine(line,) {
      return line !== '';
    },);
}

await describe({
  name: 'the stage that refreshes the moved checkouts',
  children: [
    it({
      name: 'sits after the swap and the through-the-link checks, and before the release',
      fn: async () => {
        const subject = await refreshStage();
        /**
         The cut-over stages in order.
         */
        const stages = await cutoverStages({ c: subject.config, stamp: 'S', },);
        const order = stages.map(function idOf(stage: Stage,) {
          return stage.id;
        },);
        await subject.home[Symbol.asyncDispose]();
        expect(order.indexOf(REFRESH_STAGE_ID,),).toBeGreaterThan(order.indexOf(THROUGH_LINK_STAGE_ID,),);
        expect(order.indexOf(REFRESH_STAGE_ID,),).toBeGreaterThan(order.indexOf(SWAP_STAGE_ID,),);
        expect(order.indexOf(REFRESH_STAGE_ID,),).toBeLessThan(order.indexOf(RELEASE_STAGE_ID,),);
      },
    },),
    it({
      name: 'rewrites every index and installs only in the checkout whose links dangle',
      fn: async () => {
        const subject = await refreshStage();
        await makeRepository(subject.config.repo,);
        /**
         The folder of the stand-ins.
         */
        const bin = join(subject.home.path, 'bin',);
        /**
         The file the stand-in `pnpm` appends the checkout it ran in.
         */
        const calls = join(subject.home.path, 'pnpm-calls.txt',);
        await makeStandIns({ bin, calls, install: 'repair', },);
        /**
         The nested worktree's path.
         */
        const nested = join(subject.config.repo, NESTED_WORKTREE,);
        /**
         The main checkout's index before the stage runs.
         */
        const indexPath = join(subject.config.repo, '.git', 'index',);
        /**
         The index text before the stage runs, after a touch that leaves it stale.
         */
        const before = await touchAndRead({ subject, indexPath, },);
        const result = await runStage({ subject, bin, },);
        /**
         The lines the stage wrote about the indexes it rewrote.
         */
        const indexes = await linesOf(join(subject.config.wiz, 'checkout-indexes.txt',),);
        /**
         The checkouts the stage found without `node_modules`.
         */
        const bare = await linesOf(join(subject.config.wiz, 'checkout-without-node-modules.txt',),);
        /**
         The checkouts the stand-in `pnpm` ran in.
         */
        const installed = await linesOf(calls,);
        /**
         The main checkout's index after the stage ran.
         */
        const after = await readFile(indexPath, 'base64',);
        await subject.home[Symbol.asyncDispose]();
        expect(result.commandStatus,).toBe(COMMAND_SUCCEEDED,);
        expect(result.checkStatus,).toBe(COMMAND_SUCCEEDED,);
        expect(indexes.length,).toBe(3,);
        expect(after,).not.toBe(before,);
        expect(installed,).toEqual([nested,],);
        expect(bare.join('\n',),).toContain(join(subject.config.repo, BARE_WORKTREE,),);
      },
    },),
    it({
      name: 'fails its check, naming the path, when a link still dangles after the install',
      fn: async () => {
        const subject = await refreshStage();
        await makeRepository(subject.config.repo,);
        /**
         The folder of the stand-ins.
         */
        const bin = join(subject.home.path, 'bin',);
        await makeStandIns({
          bin,
          calls: join(subject.home.path, 'pnpm-calls.txt',),
          install: 'do-nothing',
        },);
        const result = await runStage({ subject, bin, },);
        /**
         What the stage wrote about the links it could not repair.
         */
        const left = await linesOf(join(subject.config.wiz, 'checkout-links-left.txt',),);
        await subject.home[Symbol.asyncDispose]();
        expect(result.commandStatus,).toBe(COMMAND_SUCCEEDED,);
        expect(result.checkStatus,).toBe(CHECK_FAILED,);
        expect(left.join('\n',),).toContain(join(subject.config.repo, NESTED_WORKTREE,),);
      },
    },),
  ],
},);

/**
 Gives a tracked file of the main checkout a new modification time, so the next status rewrites its index, and returns
 the index before the stage runs.

 @param subject - the stage, its home and its configuration

 @param indexPath - the main checkout's index

 @returns the index, base64, before the stage runs

 @example
 ```ts
 const before = await touchAndRead({ subject, indexPath });
 ```
 */
async function touchAndRead(
  {
    subject,
    indexPath,
  }: {
    readonly subject: {
      readonly config: GuidedConfig;
    };
    readonly indexPath: string;
  },
): Promise<string> {
  /**
   When this touch happened, in seconds since the epoch.
   */
  const now = Date.now() / 1_000;
  await utimes(join(subject.config.repo, TRACKED_FILE,), now, now,);
  return await readFile(indexPath, 'base64',);
}
