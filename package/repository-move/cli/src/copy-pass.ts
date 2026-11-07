/**
 The first and final passes: per subvolume a read-only snapshot, a send piped into a receive, a check of the received
 copy, and (final pass only) placement of the copy at its final path.

 Every record is keyed by the subvolume's UUID plus its path, through `name-assignment.ts`.

 @module
 */

import {
  mkdir,
  statfs,
} from 'node:fs/promises';
import {
  join,
  relative,
} from 'node:path';
import type { Runner, } from './command-runner.ts';
import {
  createRecords,
  pathExists,
  readSnapshotEntries,
  type Records,
  writeTextAtomically,
} from './completion-records.ts';
import {
  BYTES_PER_GIB,
  FINAL_PREFIX,
  FIRST_PASS_MANIFEST,
  FIRST_PREFIX,
  STATE_FOLDER_MODE,
} from './constants.ts';
import { MoveStopError, } from './errors.ts';
import { requireFrozen, } from './freeze-pass.ts';
import type { Subvolume, } from './inventory.ts';
import type { MoveOptions, } from './move-options.ts';
import {
  assignNames,
  type NamedSubvolume,
} from './name-assignment.ts';

/**
 What one subvolume's steps need.
 */
type StepContext = {
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
 */
function field(
  {
    text,
    label,
  }: {
    readonly text: string;
    readonly label: string;
  },
): string {
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
 */
async function freeGiB(path: string,): Promise<number> {
  const info = await statfs(path,);
  return Number(info.bavail * info.bsize,) / BYTES_PER_GIB;
}

/**
 Reads one labelled field of `btrfs subvolume show`.

 @param runner - the runner
 
 @param path - the subvolume
 
 @param label - the label without its colon
 
 @returns the value
 */
async function showField(
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
 
 @throws {MoveStopError} when the snapshot's parent UUID differs, meaning the path was replaced since the inventory
 */
async function snapshotStep(
  {
    context,
    snapshot,
  }: {
    readonly context: StepContext;
    readonly snapshot: string;
  },
): Promise<void> {
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

/**
 Sends one snapshot into the received folder and verifies the received copy.

 @param context - the subvolume's context
 
 @param snapshot - the snapshot to send
 
 @param received - where the received copy appears
 
 @param sendOptions - the `-p` and `-c` arguments of the send
 
 @throws {MoveStopError} when a received folder has no record, or the copy is not complete and read-only
 */
async function receiveStep(
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
  const receivedUuid = await showField({
    runner,
    path: received,
    label: 'Received UUID',
  },);
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
 
 @throws {MoveStopError} when the receive has no record, or the placed subvolume is not writable
 */
async function placeStep(
  {
    context,
    received,
  }: {
    readonly context: StepContext;
    readonly received: string;
  },
): Promise<void> {
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
  const isMain = subvolume.path === options.mainPath;
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

/**
 Runs the first or the final pass over the reconciled subvolumes.

 @param options - the options; `pass` is `first` or `final`
 
 @param runner - the runner
 
 @param subvolumes - main first, then nested
 
 @throws {MoveStopError} when any step stops the run
 */
export async function runCopyPass(
  {
    options,
    runner,
    subvolumes,
  }: {
    readonly options: MoveOptions;
    readonly runner: Runner;
    readonly subvolumes: readonly Subvolume[];
  },
): Promise<void> {
  const isFinal = options.pass === 'final';
  const prefix = isFinal ? FINAL_PREFIX : FIRST_PREFIX;
  const records = createRecords({
    stateFolder: options.stateFolder,
    writable: options.run,
  },);
  const {
    named,
    changes,
  } = await assignNames({
    current: subvolumes,
    ledger: await readSnapshotEntries({
      stateFolder: options.stateFolder,
      prefix,
    },),
    prefix,
    folders: [
      options.snapshotFolder,
      options.stateFolder,
    ],
  },);
  for (const change of changes) {
    runner.log(`resume: ${change.message}`,);
  }
  const firstLedger = await readSnapshotEntries({
    stateFolder: options.stateFolder,
    prefix: FIRST_PREFIX,
  },);
  if (options.run) {
    await mkdir(
      options.stateFolder,
      {
        recursive: true,
        mode: STATE_FOLDER_MODE,
      },
    );
    if (isFinal) {
      await requireFrozen({
        current: subvolumes,
        runner,
        stateFolder: options.stateFolder,
      },);
    }
    else {
      await writeTextAtomically({
        path: join(
          options.stateFolder,
          FIRST_PASS_MANIFEST,
        ),
        text: `${
          named.map(function toRow({
            name,
            path,
            uuid,
          },) {
            return `${name}\t${path}\t${uuid}`;
          },)
            .join('\n',)
        }\n`,
      },);
    }
  }
  const sent: string[] = [];
  for (const subvolume of named) {
    const context: StepContext = {
      options,
      runner,
      records,
      subvolume,
    };
    const snapshot = join(
      options.snapshotFolder,
      subvolume.name,
    );
    const received = join(
      options.receivedFolder,
      subvolume.name,
    );
    runner.log(`${subvolume.name} ${subvolume.path}`,);
    if (options.run && (await freeGiB(options.receivedFolder,) < options.minFreeGiB)) {
      throw new MoveStopError(
        `the 860 has under ${
          String(options.minFreeGiB,)
        } GiB free before ${subvolume.name}; stopping with every earlier step recorded`,
      );
    }
    await snapshotStep({
      context,
      snapshot,
    },);
    const parent = isFinal
      ? firstLedger.find(function isThisSubvolume(entry,) {
        return (entry.uuid === subvolume.uuid) && (entry.path === subvolume.path);
      },)
      : undefined;
    if (isFinal && (parent === undefined)) {
      runner.log(`${subvolume.path} has no first-pass copy, so its final send is a full send`,);
    }
    if ((parent !== undefined) && options.run
      && (!await records.has({
        name: parent.name,
        step: 'received',
      },))) {
      throw new MoveStopError(
        `the first-pass copy ${parent.name} of ${subvolume.path} never finished; run the first pass again`,
      );
    }
    const clones = options.cloneSources === 'main' ? sent.slice(
      0,
      1,
    ) : sent;
    const sendOptions = [
      ...(parent === undefined ? [] : [
        '-p',
        join(
          options.snapshotFolder,
          parent.name,
        ),
      ]),
      ...clones.flatMap(function cloneArguments(earlier,) {
        return [
          '-c',
          join(
            options.snapshotFolder,
            earlier,
          ),
        ];
      },),
    ];
    await receiveStep({
      context,
      snapshot,
      received,
      sendOptions,
    },);
    if (isFinal) {
      await placeStep({
        context,
        received,
      },);
    }
    sent.push(subvolume.name,);
  }
  runner.log(
    options.run
      ? `pass ${options.pass} finished; ${String(named.length,)} subvolumes recorded in ${options.stateFolder}`
      : 'print mode: nothing was run',
  );
}
