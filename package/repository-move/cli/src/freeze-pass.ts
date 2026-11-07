/**
 Freeze and thaw as steps of their own, so the order freeze, rescan, baseline, final pass is reachable.

 The freeze sets `ro=true` on the main checkout and every nested subvolume and writes a manifest of what it froze
 (UUID and path). The final pass refuses to run unless everything it sends is in that manifest and still reads
 `ro=true`. Thaw sets `ro=false` on exactly the manifest's paths.

 @module
 */

import {
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import type { Runner, } from './command-runner.ts';
import {
  pathExists,
  writeTextAtomically,
} from './completion-records.ts';
import { FREEZE_MANIFEST, } from './constants.ts';
import { MoveStopError, } from './errors.ts';
import type { Subvolume, } from './inventory.ts';

/**
 What the freeze manifest holds: whether it exists, and the frozen subvolumes.
 */
export type FreezeManifest = {
  readonly present: boolean;
  readonly frozen: readonly Subvolume[];
};

/**
 Path of the freeze manifest.

 @param stateFolder - the state folder
 
 @returns the path
 */
function manifestPath(stateFolder: string,): string {
  return join(
    stateFolder,
    FREEZE_MANIFEST,
  );
}

/**
 Reads the freeze manifest.

 @param stateFolder - the state folder
 
 @returns the manifest; `present` is false when nothing was frozen
 */
export async function readFreezeManifest(stateFolder: string,): Promise<FreezeManifest> {
  if (!await pathExists(manifestPath(stateFolder,),)) {
    return {
      present: false,
      frozen: [],
    };
  }
  const text = await readFile(
    manifestPath(stateFolder,),
    'utf8',
  );
  return {
    present: true,
    frozen: text.split('\n',)
      .filter(function notBlank(row,) {
        return row !== '';
      },)
      .map(function parseRow(row,) {
        const [uuid = '', path = '',] = row.split('\t',);
        return {
          uuid,
          path,
        };
      },),
  };
}

/**
 Reads `ro` of a subvolume; print mode returns an empty string.

 @param runner - the runner
 
 @param path - the subvolume
 
 @returns the `ro=...` line
 */
async function readOnlyValue(
  {
    runner,
    path,
  }: {
    readonly runner: Runner;
    readonly path: string;
  },
): Promise<string> {
  return (await runner.exec({
    command: 'btrfs',
    args: [
      'property',
      'get',
      '-ts',
      path,
      'ro',
    ],
  },)).trim();
}

/**
 Sets `ro` on every subvolume and, in run mode, verifies each.

 @param runner - the runner
 
 @param subvolumes - what to set
 
 @param value - `true` to freeze, `false` to thaw
 
 @throws {MoveStopError} when a subvolume does not read back the value
 */
async function setReadOnly(
  {
    runner,
    subvolumes,
    value,
  }: {
    readonly runner: Runner;
    readonly subvolumes: readonly Subvolume[];
    readonly value: boolean;
  },
): Promise<void> {
  for (const { path, } of subvolumes) {
    await runner.exec({
      command: 'btrfs',
      args: [
        'property',
        'set',
        '-ts',
        path,
        'ro',
        String(value,),
      ],
    },);
  }
  for (const { path, } of subvolumes) {
    const read = await readOnlyValue({
      runner,
      path,
    },);
    if ((read !== '') && (read !== `ro=${String(value,)}`)) {
      throw new MoveStopError(`${path} reads ${read} after setting ro ${String(value,)}`,);
    }
  }
}

/**
 Freezes every subvolume and records what was frozen.

 @param subvolumes - the reconciled list, main first
 
 @param runner - the runner
 
 @param stateFolder - the state folder
 
 @param run - false in print mode, where nothing is recorded
 */
export async function freeze(
  {
    subvolumes,
    runner,
    stateFolder,
    run,
  }: {
    readonly subvolumes: readonly Subvolume[];
    readonly runner: Runner;
    readonly stateFolder: string;
    readonly run: boolean;
  },
): Promise<void> {
  await setReadOnly({
    runner,
    subvolumes,
    value: true,
  },);
  if (run) {
    await writeTextAtomically({
      path: manifestPath(stateFolder,),
      text: `${
        subvolumes.map(function toRow({
          uuid,
          path,
        },) {
          return `${uuid}\t${path}`;
        },)
          .join('\n',)
      }\n`,
    },);
    runner.log(`freeze finished; ${String(subvolumes.length,)} subvolumes frozen`,);
  }
  else {
    runner.log('print mode: nothing was frozen',);
  }
}

/**
 Thaws exactly the subvolumes the freeze recorded; in print mode with no manifest, the current list.

 @param loadCurrent - builds the reconciled list; called only in print mode without a manifest, so a thaw never
   depends on an inventory that worktrees changed since the freeze
 
 @param runner - the runner
 
 @param stateFolder - the state folder
 
 @param run - false in print mode
 
 @throws {MoveStopError} when run mode finds no manifest
 */
export async function thaw(
  {
    loadCurrent,
    runner,
    stateFolder,
    run,
  }: {
    readonly loadCurrent: () => Promise<readonly Subvolume[]>;
    readonly runner: Runner;
    readonly stateFolder: string;
    readonly run: boolean;
  },
): Promise<void> {
  const manifest = await readFreezeManifest(stateFolder,);
  if ((!manifest.present) && run) {
    throw new MoveStopError(`no ${FREEZE_MANIFEST} in ${stateFolder}: this tool froze nothing, so it thaws nothing`,);
  }
  const subvolumes = manifest.present ? manifest.frozen : await loadCurrent();
  await setReadOnly({
    runner,
    subvolumes,
    value: false,
  },);
  runner.log(
    run ? `thaw finished; ${String(subvolumes.length,)} subvolumes thawed` : 'print mode: nothing was thawed',
  );
}

/**
 Refuses the final pass unless every subvolume it will send is frozen.

 @param current - the subvolumes the final pass covers
 
 @param runner - the runner
 
 @param stateFolder - the state folder
 
 @throws {MoveStopError} when one is not in the freeze manifest or does not read `ro=true`
 */
export async function requireFrozen(
  {
    current,
    runner,
    stateFolder,
  }: {
    readonly current: readonly Subvolume[];
    readonly runner: Runner;
    readonly stateFolder: string;
  },
): Promise<void> {
  const manifest = await readFreezeManifest(stateFolder,);
  const frozen = new Set(manifest.frozen
    .map(function toKey({
      uuid,
      path,
    },) {
      return `${uuid}\t${path}`;
    },),);
  const notFrozen = current.filter(function isMissing({
    uuid,
    path,
  },) {
    return !frozen.has(`${uuid}\t${path}`,);
  },);
  if (notFrozen.length > 0) {
    throw new MoveStopError(
      `not frozen: ${
        notFrozen.map(function describe({
          path,
          uuid,
        },) {
          return `${path} (${uuid})`;
        },)
          .join(' ',)
      }. Run --pass=freeze first `
        + `(the manifest is ${manifestPath(stateFolder,)}), and again if a worktree appeared after it.`,
    );
  }
  for (const { path, } of current) {
    const read = await readOnlyValue({
      runner,
      path,
    },);
    if (read !== 'ro=true') {
      throw new MoveStopError(`${path} reads ${read}, not ro=true: it was thawed after the freeze`,);
    }
  }
}
