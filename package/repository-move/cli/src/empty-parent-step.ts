/**
 The empty parent: one empty subvolume, snapshotted read-only into the snapshot folder and sent once, so that every other
 send can name an existing parent that holds no inode to mix up with the sent subvolume's (see `send-plan.ts`).

 @module
 */

import { join, } from 'node:path';
import type { Runner, } from './command-runner.ts';
import type { Records, } from './completion-records.ts';
import { receiveStep, } from './copy-transfer-steps.ts';
import type { MoveOptions, } from './move-options.ts';
import {
  EMPTY_PARENT_BASE_NAME,
  EMPTY_PARENT_NAME,
} from './send-plan.ts';

/**
 Makes the empty parent and sends it, each step only when its record is missing.

 @param options - the options

 @param runner - the runner

 @param records - the run's record store

 @throws MoveStopError when the received copy exists without a record, or is not a complete read-only copy

 @example
 ```ts
 import { emptyParentStep, } from './empty-parent-step.ts';

 await emptyParentStep({ options, runner, records, },);
 ```
 */
export async function emptyParentStep(
  {
    options,
    runner,
    records,
  }: {
    readonly options: MoveOptions;
    readonly runner: Runner;
    readonly records: Records;
  },
): Promise<void> {
  /**
   The empty subvolume.
   */
  const base = join(
    options.snapshotFolder,
    EMPTY_PARENT_BASE_NAME,
  );
  /**
   The empty read-only snapshot.
   */
  const snapshot = join(
    options.snapshotFolder,
    EMPTY_PARENT_NAME,
  );
  if (!await records.has({
    name: EMPTY_PARENT_NAME,
    step: 'snapshot',
  },)) {
    await runner.exec({
      command: 'btrfs',
      args: [
        'subvolume',
        'create',
        base,
      ],
    },);
    await runner.exec({
      command: 'btrfs',
      args: [
        'subvolume',
        'snapshot',
        '-r',
        base,
        snapshot,
      ],
    },);
    await records.write({
      name: EMPTY_PARENT_NAME,
      step: 'snapshot',
      data: {
        path: base,
        uuid: '-',
      },
    },);
  }
  await receiveStep({
    context: {
      options,
      runner,
      records,
      subvolume: {
        name: EMPTY_PARENT_NAME,
        path: base,
        uuid: '-',
        parentUuid: '-',
      },
    },
    snapshot,
    received: join(
      options.receivedFolder,
      EMPTY_PARENT_NAME,
    ),
    sendOptions: [],
  },);
}
