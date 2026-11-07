/**
 Throwaway fixtures for the unit tests of the move tools: a main checkout with registered worktrees whose subvolume
 status comes from marker files (`btrfs-test-double.ts`), the inventory the root `btrfs subvolume list -u` would
 print for them, and a runner for the built copy tool with the double first on `PATH`.

 @module
 */

import { spawnSync, } from 'node:child_process';
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

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

export type Fixture = {
  readonly base: string;
  readonly main: string;
  readonly binFolder: string;
  readonly doubleLog: string;
  readonly inventory: string;
};

export type RunResult = { readonly status: number | null; readonly stdout: string; readonly stderr: string; };

function git({ args, cwd, }: { args: readonly string[]; cwd: string; },): void {
  const result = spawnSync(GIT, ['-c', 'user.name=t', '-c', 'user.email=t@t', ...args,], { cwd, encoding: 'utf8', },);
  if (result.status !== 0) {
    throw new Error(`git ${args.join(' ',)} failed: ${result.stderr}`,);
  }
}

/**
 Makes a worktree folder that the double treats as a source subvolume with the given UUID.

 @param fixture - the fixture
 @param name - worktree name under `main/wt`
 @param uuid - the subvolume UUID the marker records
 */
export function addWorktree({ fixture, name, uuid, }: { fixture: Fixture; name: string; uuid: string; },): void {
  git({ args: ['worktree', 'add', '--quiet', join(fixture.main, 'wt', name,), '-b', name,], cwd: fixture.main, },);
  writeFileSync(join(fixture.main, 'wt', name, '.stub-subvolume',), `${uuid}\n`,);
}

/**
 Unregisters and deletes a worktree folder.

 @param fixture - the fixture
 @param name - worktree name under `main/wt`
 */
export function removeWorktree({ fixture, name, }: { fixture: Fixture; name: string; },): void {
  git({ args: ['worktree', 'remove', '--force', join(fixture.main, 'wt', name,),], cwd: fixture.main, },);
}

/**
 Writes the inventory file as `btrfs subvolume list -u` prints it, one row per subvolume, plus an unrelated row.

 @param fixture - the fixture
 @param rows - names under `main/wt` with their UUIDs; the main checkout is always `u-main`
 */
export function writeInventory({ fixture, rows, }: { fixture: Fixture; rows: readonly (readonly [string, string])[]; },): void {
  const lines = [
    `ID 100 gen 9 top level 5 uuid u-unrelated path other/unrelated/volume`,
    `ID 256 gen 9 top level 5 uuid u-main path ${INVENTORY_TOP}/main`,
    ...rows.map(([name, uuid,], index,) =>
      `ID ${String(300 + index,)} gen 9 top level 256 uuid ${uuid} path ${INVENTORY_TOP}/main/wt/${name}`
    ),
  ];
  writeFileSync(fixture.inventory, `${lines.join('\n',)}\n`,);
}

/**
 Creates a fixture with the given worktrees, each UUID `u-<name>`.

 @param names - worktree names
 @returns the fixture; remove it with `removeFixture`
 */
export function makeFixture({ names, }: { names: readonly string[]; },): Fixture {
  const base = mkdtempSync(join(scratchRoot(), 'repository-move-',),);
  const main = join(base, 'main',);
  mkdirSync(main,);
  git({ args: ['init', '--quiet',], cwd: main, },);
  writeFileSync(join(main, 'file',), 'x',);
  git({ args: ['add', 'file',], cwd: main, },);
  git({ args: ['commit', '--quiet', '--message', 'initial',], cwd: main, },);
  writeFileSync(join(main, '.stub-subvolume',), 'u-main\n',);
  const binFolder = join(base, 'bin',);
  mkdirSync(binFolder,);
  chmodSync(DOUBLE, 0o755,);
  symlinkSync(DOUBLE, join(binFolder, 'btrfs',),);
  for (const folder of ['snap', 'recv', 'state', 'final',]) {
    mkdirSync(join(base, folder,),);
  }
  const fixture: Fixture = {
    base,
    main,
    binFolder,
    doubleLog: join(base, 'double.log',),
    inventory: join(base, 'inventory.txt',),
  };
  for (const name of names) {
    addWorktree({ fixture, name, uuid: `u-${name}`, },);
  }
  writeInventory({ fixture, rows: names.map((name,) => [name, `u-${name}`,] as const), },);
  return fixture;
}

/**
 Deletes a fixture folder.

 @param fixture - the fixture
 */
export function removeFixture({ fixture, }: { fixture: Fixture; },): void {
  rmSync(fixture.base, { recursive: true, force: true, },);
}

/**
 Runs the built copy tool against the fixture in run mode with the double on `PATH`.

 @param fixture - the fixture
 @param pass - `first`, `final`, `freeze` or `thaw`
 @param extra - further arguments
 @param env - further environment, for the double's knobs
 @returns exit status and both streams
 */
export function runCopyTool(
  { fixture, pass, extra = [], env = {}, }: {
    fixture: Fixture;
    pass: string;
    extra?: readonly string[];
    env?: Readonly<Record<string, string>>;
  },
): RunResult {
  const result = spawnSync('node', [
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
  return { status: result.status, stdout: result.stdout, stderr: result.stderr, };
}
