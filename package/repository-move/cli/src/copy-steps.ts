/**
 The snapshot step of one subvolume in a copy pass, and the helpers the later steps share.

 @module
 */

import { statfs, } from 'node:fs/promises';
import { join, } from 'node:path';
import type { Runner, } from './command-runner.ts';
import type { Records, } from './completion-records.ts';
import { BYTES_PER_GIB, } from './constants.ts';
import { MoveStopError, } from './errors.ts';
import type { MoveOptions, } from './move-options.ts';
import type { NamedSubvolume, } from './name-assignment.ts';

/**
 What one subvolume's steps need.
 */
export type StepContext = {
  readonly options: MoveOptions;
  readonly runner: Runner;
  readonly records: Records;
  readonly subvolume: NamedSubvolume;
};

/**
 Value of a `Label:` line of `btrfs subvolume show` output.

 @param text - the output
 
 @param label - the label without its colon
 
 @returns the value, or an empty string

 @example
 ```ts
 import { field, } from './copy-steps.ts';
 
 const result = field({ text, label, },);
 ```
 */
export function field(
  {
    text,
    label,
  }: {
    readonly text: string;
    readonly label: string;
  },
): string {
  /**
   The line of this step.
   */
  const line = text.split('\n',)
    .find(function hasLabel(row,) {
      return row.trim()
        .startsWith(`${label}:`,);
    },);
  return line === undefined ? '' : line.split(':',)
    .slice(1,)
    .join(':',)
    .trim();
}

/**
 Free space in GiB of the filesystem holding a path.

 @param path - the path
 
 @returns GiB available to the user

 @example
 ```ts
 import { freeGiB, } from './copy-steps.ts';
 
 const result = await freeGiB(path,);
 ```
 */
export async function freeGiB(path: string,): Promise<number> {
  /**
   The filesystem information read for the path.
   */
  const info = await statfs(path,);
  return (info.bavail * info.bsize) / BYTES_PER_GIB;
}

/**
 Reads one labelled field of `btrfs subvolume show`.

 @param runner - the runner
 
 @param path - the subvolume
 
 @param label - the label without its colon
 
 @returns the value

 @example
 ```ts
 import { showField, } from './copy-steps.ts';
 
 const result = await showField({ runner, path, label, },);
 ```
 */
export async function showField(
  {
    runner,
    path,
    label,
  }: {
    readonly runner: Runner;
    readonly path: string;
    readonly label: string;
  },
): Promise<string> {
  return field({
    text: await runner.exec({
      command: 'btrfs',
      args: [
        'subvolume',
        'show',
        path,
      ],
    },),
    label,
  },);
}

/**
 Takes the read-only snapshot of one subvolume and checks it is a snapshot of the subvolume the inventory named.

 @param context - the subvolume's context
 
 @param snapshot - where the snapshot goes
 
 @throws MoveStopError when the snapshot's parent UUID differs, meaning the path was replaced since the inventory

 @example
 ```ts
 import { snapshotStep, } from './copy-steps.ts';
 
 await snapshotStep({ context, snapshot, },);
 ```
 */
export async function snapshotStep(
  {
    context,
    snapshot,
  }: {
    readonly context: StepContext;
    readonly snapshot: string;
  },
): Promise<void> {
  /**
   The parts of the value this function uses, taken out of it.
   */
  const {
    options,
    runner,
    records,
    subvolume,
  } = context;
  if (await records.has({
    name: subvolume.name,
    step: 'snapshot',
  },)) {
    return;
  }
  await runner.exec({
    command: 'btrfs',
    args: [
      'subvolume',
      'snapshot',
      '-r',
      subvolume.path,
      snapshot,
    ],
  },);
  if (options.run) {
    /**
     The parent entry of the subvolume, when it has one.
     */
    const parent = await showField({
      runner,
      path: snapshot,
      label: 'Parent UUID',
    },);
    if (parent !== subvolume.uuid) {
      throw new MoveStopError(
        `${subvolume.path} was replaced since the inventory: its snapshot ${snapshot} has parent UUID ${parent}, `
          + `the inventory said ${subvolume.uuid}. The snapshot stays; take the inventory again and run again.`,
      );
    }
  }
  await records.write({
    name: subvolume.name,
    step: 'snapshot',
    data: {
      path: subvolume.path,
      uuid: subvolume.uuid,
    },
  },);
}
