/**
 Runs the built copy tool against a fixture in run mode, with the btrfs test double first on `PATH`.

 @module
 */

import { execFile, } from 'node:child_process';
import { join, } from 'node:path';
import { promisify, } from 'node:util';
import type { Fixture, } from './move-fixture.ts';

/**
 What a run of the copy tool printed and how it ended.
 */
export type RunResult = {
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Runs a program and waits for it to end.
 */
const execFileAsync = promisify(execFile,);

/**
 Name of the folder inside the fixture that stands for the filesystem's top level in the inventory paths.
 */
export const INVENTORY_TOP = 'fx';

/**
 Path of the built copy tool the tests spawn.
 */
export const COPY_TOOL: string = new URL(
  '../dist/final/node/move-repository.mjs',
  import.meta.url,
).pathname;

/**
 Runs the built copy tool against the fixture in run mode with the double on `PATH`.

 @param fixture - the fixture
 
 @param pass - `first`, `final`, `freeze`, `thaw`, `external-first` or `external-final`
 
 @param extra - further arguments
 
 @param env - further environment, for the double's knobs
 
 @param mainPath - the main checkout as the caller spells it; the fixture's own folder when absent
 
 @param snapshotFolder - the snapshot folder; the fixture's own `snap` folder when absent
 
 @param receivedFolder - the received folder; the fixture's own `recv` folder when absent
 
 @param inventoryBase - the folder the inventory's top-level name maps to, as the caller spells it; the fixture's own folder when absent
 
 @returns exit status and both streams

 @example
 ```ts
 import { runCopyTool, } from './move-fixture-run.ts';
 
 const result = await runCopyTool({ fixture, pass, extra, env, mainPath, inventoryBase, },);
 ```
 */
export async function runCopyTool(
  {
    fixture,
    pass,
    extra = [],
    env = {},
    mainPath = fixture.main,
    inventoryBase = fixture.base,
    snapshotFolder = join(fixture.base, 'snap',),
    receivedFolder = join(fixture.base, 'recv',),
  }: {
    readonly fixture: Fixture;
    readonly pass: string;
    readonly extra?: readonly string[];
    readonly env?: Readonly<Record<string, string>>;
    readonly mainPath?: string;
    readonly inventoryBase?: string;
    readonly snapshotFolder?: string;
    readonly receivedFolder?: string;
  },
): Promise<RunResult> {
  try {
    /**
     What the call returned.
     */
    const result = await execFileAsync(
      'node',
      [
        COPY_TOOL,
        `--pass=${pass}`,
        `--main=${mainPath}`,
        `--snapshots=${snapshotFolder}`,
        `--received=${receivedFolder}`,
        `--final=${join(
        fixture.base,
        'final',
        'Monochromatic',
      )}`,
        `--state=${join(
        fixture.base,
        'state',
      )}`,
        `--inventory=${fixture.inventory}`,
        `--inventory-main=${INVENTORY_TOP}/main`,
        `--inventory-prefix=${INVENTORY_TOP}/=${inventoryBase}/`,
        '--run',
        '--no-scope',
        '--subvolume-marker=.stub-subvolume',
        '--min-free-gib=0',
        ...extra,
      ],
      {
        encoding: 'utf8',
        env: {
          ...process.env,
          PATH: `${fixture.binFolder}:${process.env
            .PATH
            ?? ''}`,
          BTRFS_DOUBLE_LOG: fixture.doubleLog,
          ...env,
        },
      },
    );
    return {
      status: 0,
      stdout: result.stdout,
      stderr: result.stderr,
    };
  }
  catch (error) {
    /**
     The caught failure, in the shape the caller reads.
     */
    const failure = error as {
      readonly code: number;
      readonly stdout: string;
      readonly stderr: string;
    };
    return {
      status: failure.code,
      stdout: failure.stdout,
      stderr: failure.stderr,
    };
  }
}
