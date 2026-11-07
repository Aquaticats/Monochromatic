/**
 The send and receive of one snapshot with the check of the received copy, and the placement of the copy at its final
 path.

 @module
 */

import {
  join,
  relative,
} from 'node:path';
import { pathExists, } from './completion-records.ts';
import { MoveStopError, } from './errors.ts';
import {
  showField,
  type StepContext,
} from './copy-steps.ts';

/**
 Sends one snapshot into the received folder and verifies the received copy.

 @param context - the subvolume's context
 
 @param snapshot - the snapshot to send
 
 @param received - where the received copy appears
 
 @param sendOptions - the `-p` and `-c` arguments of the send
 
 @throws MoveStopError when a received folder has no record, or the copy is not complete and read-only
 */
export async function receiveStep(
  {
    context,
    snapshot,
    received,
    sendOptions,
  }: {
    readonly context: StepContext;
    readonly snapshot: string;
    readonly received: string;
    readonly sendOptions: readonly string[];
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
    step: 'received',
  },)) {
    return;
  }
  if (options.run && await pathExists(received,)) {
    throw new MoveStopError(
      `${received} exists without a completion record: a receive was interrupted. Inspect it, delete it with `
        + `'btrfs subvolume delete ${received}' yourself, and run again. Nothing was deleted by this script.`,
    );
  }
  /**
   The send piped into the receive, as shell text.
   */
  const pipeline = `btrfs send ${[
    ...sendOptions,
    snapshot,
  ].join(' ',)} | btrfs receive ${options.receivedFolder}`;
  await runner.exec({
    command: 'bash',
    args: [
      '-o',
      'pipefail',
      '-c',
      pipeline,
    ],
    transfer: true,
  },);
  if (!options.run) {
    return;
  }
  /**
   The `ro` value read back from btrfs.
   */
  const readOnly =
    (await runner.exec({
      command: 'btrfs',
      args: [
        'property',
        'get',
        '-ts',
        received,
        'ro',
      ],
    },)).trim();
  /**
   The received UUID read back from the received copy.
   */
  const receivedUuid = await showField({
    runner,
    path: received,
    label: 'Received UUID',
  },);
  /**
   The UUID of the snapshot that was sent.
   */
  const sentUuid = await showField({
    runner,
    path: snapshot,
    label: 'UUID',
  },);
  if ((readOnly !== 'ro=true') || (receivedUuid !== sentUuid)
    || (sentUuid === '')
    || (sentUuid === '-')) {
    throw new MoveStopError(
      `${received} is not a complete read-only copy of ${snapshot} (${readOnly}, received UUID `
        + `${receivedUuid}, sent UUID ${sentUuid})`,
    );
  }
  await records.write({
    name: subvolume.name,
    step: 'received',
    data: {
      path: subvolume.path,
      uuid: sentUuid,
    },
  },);
}

/**
 Places the received copy at its final path as a writable subvolume.

 @param context - the subvolume's context
 
 @param received - the received copy
 
 @throws MoveStopError when the receive has no record, or the placed subvolume is not writable
 */
export async function placeStep(
  {
    context,
    received,
  }: {
    readonly context: StepContext;
    readonly received: string;
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
    step: 'placed',
  },)) {
    return;
  }
  if (options.run && (!await records.has({
    name: subvolume.name,
    step: 'received',
  },))) {
    throw new MoveStopError(`refusing to place ${subvolume.name}: no completion record for its receive`,);
  }
  /**
   Whether this subvolume is the main checkout.
   */
  const isMain = subvolume.path === options.mainPath;
  /**
   The path the step writes to or reads from.
   */
  const target = isMain ? options.finalPath : join(
    options.finalPath,
    relative(
      options.mainPath,
      subvolume.path,
    ),
  );
  if (!isMain) {
    // The stub of a nested subvolume in the received parent is an empty directory of inode 2 (measured on the 860).
    await runner.exec({
      command: 'rmdir',
      args: [target,],
    },);
  }
  await runner.exec({
    command: 'btrfs',
    args: [
      'subvolume',
      'snapshot',
      received,
      target,
    ],
  },);
  if (options.run) {
    /**
     Whether this run writes records.
     */
    const writable =
      (await runner.exec({
        command: 'btrfs',
        args: [
          'property',
          'get',
          '-ts',
          target,
          'ro',
        ],
      },)).trim();
    if (writable !== 'ro=false') {
      throw new MoveStopError(`${target} is not writable after placement`,);
    }
  }
  await records.write({
    name: subvolume.name,
    step: 'placed',
    data: { target, },
  },);
}
