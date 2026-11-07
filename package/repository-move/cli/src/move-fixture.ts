/**
 Throwaway fixtures for the unit tests of the move tools: a main checkout with registered worktrees whose subvolume
 status comes from marker files (`btrfs-test-double.ts`), and the inventory the root `btrfs subvolume list -u` would
 print for them.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { INVENTORY_TOP, } from './move-fixture-run.ts';
import {
  execFileAsync,
} from '../dist/final/node/index.mjs';

export {
  COPY_TOOL,
  runCopyTool,
  type RunResult,
} from './move-fixture-run.ts';

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
 Runs a program and waits for it.
 */

/**
 Git's own program, never the repository's wrapper.
 */
const GIT = '/usr/bin/git';

/**
 Path of the test double, made executable by this module before use.
 */
const DOUBLE: string = new URL(
  'btrfs-test-double.ts',
  import.meta.url,
).pathname;

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
  {
    args,
    cwd,
  }: {
    readonly args: readonly string[];
    readonly cwd: string;
  },
): Promise<void> {
  await execFileAsync(
    GIT,
    [
      '-c',
      'user.name=t',
      '-c',
      'user.email=t@t',
      ...args,
    ],
    { cwd, },
  );
}

/**
 Makes a worktree folder that the double treats as a source subvolume with the given UUID.

 @param fixture - the fixture
 
 @param name - worktree name under `main/wt`
 
 @param uuid - the subvolume UUID the marker records

 @example
 ```ts
 import { addWorktree, } from './move-fixture.ts';
 
 await addWorktree({ fixture, name, uuid, },);
 ```
 */
export async function addWorktree(
  {
    fixture,
    name,
    uuid,
  }: {
    readonly fixture: Fixture;
    readonly name: string;
    readonly uuid: string;
  },
): Promise<void> {
  await git({
    args: [
      'worktree',
      'add',
      '--quiet',
      join(
        fixture.main,
        'wt',
        name,
      ),
      '-b',
      name,
    ],
    cwd: fixture.main,
  },);
  await writeFile(
    join(
      fixture.main,
      'wt',
      name,
      '.stub-subvolume',
    ),
    `${uuid}\n`,
  );
}

/**
 Unregisters and deletes a worktree folder.

 @param fixture - the fixture
 
 @param name - worktree name under `main/wt`

 @example
 ```ts
 import { removeWorktree, } from './move-fixture.ts';
 
 await removeWorktree({ fixture, name, },);
 ```
 */
export async function removeWorktree(
  {
    fixture,
    name,
  }: {
    readonly fixture: Fixture;
    readonly name: string;
  },
): Promise<void> {
  await git({
    args: [
      'worktree',
      'remove',
      '--force',
      join(
        fixture.main,
        'wt',
        name,
      ),
    ],
    cwd: fixture.main,
  },);
}

/**
 Writes the inventory file as `btrfs subvolume list -u` prints it, one row per subvolume, plus an unrelated row.

 @param fixture - the fixture
 
 @param rows - names under `main/wt` with their UUIDs and, as a third element, the UUID of the subvolume each was snapshotted from (the dash when absent); the main checkout is always `u-main`
 
 @param withParents - false writes the older listing without the parent column

 @example
 ```ts
 import { writeInventory, } from './move-fixture.ts';
 
 await writeInventory({ fixture, rows, },);
 ```
 */
export async function writeInventory(
  {
    fixture,
    rows,
    withParents = true,
  }: {
    readonly fixture: Fixture;
    readonly rows: readonly (readonly [
      string,
      string
    ] | readonly [
      string,
      string,
      string
    ])[];
    readonly withParents?: boolean;
  },
): Promise<void> {
  /**
   The columns of one row between `top level` and the path: with `btrfs subvolume list -q` the parent and received
   UUIDs come first, without it only the UUID.

   @param uuid - the subvolume UUID

   @param parentUuid - the UUID of the subvolume it was snapshotted from, the dash for none

   @returns the text of the columns
   */
  function columns(
    {
      uuid,
      parentUuid,
    }: {
      readonly uuid: string;
      readonly parentUuid: string;
    },
  ): string {
    return withParents ? `parent_uuid ${parentUuid} received_uuid - uuid ${uuid}` : `uuid ${uuid}`;
  }
  /**
   The lines of text this step works on.
   */
  const lines = [
    `ID 100 gen 9 top level 5 ${columns({
      uuid: 'u-unrelated',
      parentUuid: '-',
    },)} path other/unrelated/volume`,
    `ID 256 gen 9 top level 5 ${columns({
      uuid: 'u-main',
      parentUuid: '-',
    },)} path ${INVENTORY_TOP}/main`,
    ...rows.map(function toRow(
      row,
      index,
    ) {
      /**
       The name and UUID, and the UUID of the subvolume it was snapshotted from (the dash when the row has none).
       */
      const [name, uuid,] = row;
      /**
       The UUID of the subvolume this row was snapshotted from, the dash for none.
       */
      const parentUuid = row.at(2,) ?? '-';
      return `ID ${String(FIRST_WORKTREE_ID + index,)} gen 9 top level 256 ${columns({
        uuid,
        parentUuid,
      },)} path ${INVENTORY_TOP}/main/wt/${name}`;
    },),
  ];
  await writeFile(
    fixture.inventory,
    `${lines.join('\n',)}\n`,
  );
}

/**
 Creates a fixture with the given worktrees, each UUID `u-<name>`.

 @param names - worktree names
 
 @returns the fixture; declare it with `await using` so its folder is removed at the end of the scope

 @example
 ```ts
 import { makeFixture, } from './move-fixture.ts';
 
 const result = await makeFixture({ names, },);
 ```
 */
export async function makeFixture(
  { names, }: {
    readonly names: readonly string[];
  },
): Promise<Fixture> {
  /**
   The first number a new copy name takes.
   */
  const base = await mkdtemp(join(
    scratchRoot(),
    'repository-move-',
  ),);
  /**
   The main checkout.
   */
  const main = join(
    base,
    'main',
  );
  await mkdir(main,);
  await git({
    args: [
      'init',
      '--quiet',
    ],
    cwd: main,
  },);
  await writeFile(
    join(
      main,
      'file',
    ),
    'x',
  );
  await git({
    args: [
      'add',
      'file',
    ],
    cwd: main,
  },);
  await git({
    args: [
      'commit',
      '--quiet',
      '--message',
      'initial',
    ],
    cwd: main,
  },);
  await writeFile(
    join(
      main,
      '.stub-subvolume',
    ),
    'u-main\n',
  );
  /**
   The folder that holds the test double as `btrfs`.
   */
  const binFolder = join(
    base,
    'bin',
  );
  await mkdir(binFolder,);
  await chmod(
    DOUBLE,
    EXECUTABLE_MODE,
  );
  await symlink(
    DOUBLE,
    join(
      binFolder,
      'btrfs',
    ),
  );
  await Promise.all([
    'snap',
    'recv',
    'state',
    'final',
  ].map(async function makeFolder(folder,) {
    await mkdir(join(
      base,
      folder,
    ),);
  },),);
  /**
   The fixture folder.
   */
  const fixture: Fixture = {
    base,
    main,
    binFolder,
    doubleLog: join(
      base,
      'double.log',
    ),
    inventory: join(
      base,
      'inventory.txt',
    ),
    async [Symbol.asyncDispose]() {
      await rm(
        base,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
  for (const name of names) {
    await addWorktree({
      fixture,
      name,
      uuid: `u-${name}`,
    },);
  }
  await writeInventory({
    fixture,
    rows: names.map(function toRow(name,) {
      return [
        name,
        `u-${name}`,
      ] as const;
    },),
  },);
  return fixture;
}
