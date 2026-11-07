/**
 Throwaway fixtures for the unit tests of the move tools: a main checkout with registered worktrees whose subvolume
 status comes from marker files (`btrfs-test-double.ts`), the inventory the root `btrfs subvolume list -u` would
 print for them, and a runner for the built copy tool with the double first on `PATH`.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  chmod,
  mkdir,
  mkdtemp,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { promisify, } from 'node:util';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

/**
 A fixture folder and the places inside it.
 */
export type Fixture = {
  readonly base: string;
  readonly main: string;
  readonly binFolder: string;
  readonly doubleLog: string;
  readonly inventory: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 What a run of the copy tool printed and how it ended.
 */
export type RunResult = {
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
};

const execFileAsync = promisify(execFile,);

/**
 Git's own program, never the repository's wrapper.
 */
const GIT = '/usr/bin/git';

/**
 Name of the folder inside the fixture that stands for the filesystem's top level in the inventory paths.
 */
const INVENTORY_TOP = 'fx';

/**
 Path of the built copy tool the tests spawn.
 */
export const COPY_TOOL: string = new URL('../dist/final/node/move-repository.mjs', import.meta.url,).pathname;

/**
 Path of the test double, made executable by this module before use.
 */
const DOUBLE: string = new URL('./btrfs-test-double.ts', import.meta.url,).pathname;

/**
 Mode that makes the test double executable.
 */
const EXECUTABLE_MODE = 0o755;

/**
 Number the first inventory row of a worktree gets; the rows count up from it.
 */
const FIRST_WORKTREE_ID = 300;

/**
 Runs Git in a folder with a fixed author.

 @param args - Git arguments
 @param cwd - the folder
 @throws Error when Git fails
 */
async function git(
  { args, cwd, }: {
    readonly args: readonly string[];
    readonly cwd: string;
  },
): Promise<void> {
  await execFileAsync(GIT, ['-c', 'user.name=t', '-c', 'user.email=t@t', ...args,], { cwd, },);
}

/**
 Makes a worktree folder that the double treats as a source subvolume with the given UUID.

 @param fixture - the fixture
 @param name - worktree name under `main/wt`
 @param uuid - the subvolume UUID the marker records
 */
export async function addWorktree(
  { fixture, name, uuid, }: {
    readonly fixture: Fixture;
    readonly name: string;
    readonly uuid: string;
  },
): Promise<void> {
  await git({ args: ['worktree', 'add', '--quiet', join(fixture.main, 'wt', name,), '-b', name,], cwd: fixture.main, },);
  await writeFile(join(fixture.main, 'wt', name, '.stub-subvolume',), `${uuid}\n`,);
}

/**
 Unregisters and deletes a worktree folder.

 @param fixture - the fixture
 @param name - worktree name under `main/wt`
 */
export async function removeWorktree(
  { fixture, name, }: {
    readonly fixture: Fixture;
    readonly name: string;
  },
): Promise<void> {
  await git({ args: ['worktree', 'remove', '--force', join(fixture.main, 'wt', name,),], cwd: fixture.main, },);
}

/**
 Writes the inventory file as `btrfs subvolume list -u` prints it, one row per subvolume, plus an unrelated row.

 @param fixture - the fixture
 @param rows - names under `main/wt` with their UUIDs; the main checkout is always `u-main`
 */
export async function writeInventory(
  { fixture, rows, }: {
    readonly fixture: Fixture;
    readonly rows: readonly (readonly [string, string])[];
  },
): Promise<void> {
  const lines = [
    'ID 100 gen 9 top level 5 uuid u-unrelated path other/unrelated/volume',
    `ID 256 gen 9 top level 5 uuid u-main path ${INVENTORY_TOP}/main`,
    ...rows.map(function toRow([name, uuid,], index,) {
      return `ID ${String(FIRST_WORKTREE_ID + index,)} gen 9 top level 256 uuid ${uuid} path ${INVENTORY_TOP}/main/wt/${name}`;
    },),
  ];
  await writeFile(fixture.inventory, `${lines.join('\n',)}\n`,);
}

/**
 Creates a fixture with the given worktrees, each UUID `u-<name>`.

 @param names - worktree names
 @returns the fixture; declare it with `await using` so its folder is removed at the end of the scope
 */
export async function makeFixture(
  { names, }: {
    readonly names: readonly string[];
  },
): Promise<Fixture> {
  const base = await mkdtemp(join(scratchRoot(), 'repository-move-',),);
  const main = join(base, 'main',);
  await mkdir(main,);
  await git({ args: ['init', '--quiet',], cwd: main, },);
  await writeFile(join(main, 'file',), 'x',);
  await git({ args: ['add', 'file',], cwd: main, },);
  await git({ args: ['commit', '--quiet', '--message', 'initial',], cwd: main, },);
  await writeFile(join(main, '.stub-subvolume',), 'u-main\n',);
  const binFolder = join(base, 'bin',);
  await mkdir(binFolder,);
  await chmod(DOUBLE, EXECUTABLE_MODE,);
  await symlink(DOUBLE, join(binFolder, 'btrfs',),);
  await Promise.all(['snap', 'recv', 'state', 'final',].map(async function makeFolder(folder,) {
    await mkdir(join(base, folder,),);
  },),);
  const fixture: Fixture = {
    base,
    main,
    binFolder,
    doubleLog: join(base, 'double.log',),
    inventory: join(base, 'inventory.txt',),
    async [Symbol.asyncDispose]() {
      await rm(base, { recursive: true, force: true, },);
    },
  };
  for (const name of names) {
    await addWorktree({ fixture, name, uuid: `u-${name}`, },);
  }
  await writeInventory({
    fixture,
    rows: names.map(function toRow(name,) {
      return [name, `u-${name}`,] as const;
    },),
  },);
  return fixture;
}

/**
 Runs the built copy tool against the fixture in run mode with the double on `PATH`.

 @param fixture - the fixture
 @param pass - `first`, `final`, `freeze`, `thaw`, `external-first` or `external-final`
 @param extra - further arguments
 @param env - further environment, for the double's knobs
 @returns exit status and both streams
 */
export async function runCopyTool(
  { fixture, pass, extra = [], env = {}, }: {
    readonly fixture: Fixture;
    readonly pass: string;
    readonly extra?: readonly string[];
    readonly env?: Readonly<Record<string, string>>;
  },
): Promise<RunResult> {
  try {
    const result = await execFileAsync('node', [
      COPY_TOOL,
      `--pass=${pass}`,
      `--main=${fixture.main}`,
      `--snapshots=${join(fixture.base, 'snap',)}`,
      `--received=${join(fixture.base, 'recv',)}`,
      `--final=${join(fixture.base, 'final', 'Monochromatic',)}`,
      `--state=${join(fixture.base, 'state',)}`,
      `--inventory=${fixture.inventory}`,
      `--inventory-main=${INVENTORY_TOP}/main`,
      `--inventory-prefix=${INVENTORY_TOP}/=${fixture.base}/`,
      '--run',
      '--no-scope',
      '--subvolume-marker=.stub-subvolume',
      '--min-free-gib=0',
      ...extra,
    ], {
      encoding: 'utf8',
      env: {
        ...process.env,
        PATH: `${fixture.binFolder}:${process.env['PATH'] ?? ''}`,
        BTRFS_DOUBLE_LOG: fixture.doubleLog,
        ...env,
      },
    },);
    return { status: 0, stdout: result.stdout, stderr: result.stderr, };
  }
  catch (error) {
    const failure = error as {
      readonly code: number;
      readonly stdout: string;
      readonly stderr: string;
    };
    return { status: failure.code, stdout: failure.stdout, stderr: failure.stderr, };
  }
}
