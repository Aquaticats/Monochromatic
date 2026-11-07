/**
 The first and final passes: per subvolume a read-only snapshot, a send piped into a receive, a check of the received
 copy, and (final pass only) placement of the copy at its final path.

 Every record is keyed by the subvolume's UUID plus its path, through `name-assignment.ts`.

 @module
 */

import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import type { Runner, } from './command-runner.ts';
import {
  createRecords,
  readSnapshotEntries,
  writeTextAtomically,
} from './completion-records.ts';
import {
  FINAL_PREFIX,
  FIRST_PASS_MANIFEST,
  FIRST_PREFIX,
  STATE_FOLDER_MODE,
} from './constants.ts';
import { MoveStopError, } from './errors.ts';
import { requireFrozen, } from './freeze-pass.ts';
import type { Subvolume, } from './inventory.ts';
import type { MoveOptions, } from './move-options.ts';
import { assignNames, } from './name-assignment.ts';
import {
  freeGiB,
  snapshotStep,
  type StepContext,
} from './copy-steps.ts';
import {
  placeStep,
  receiveStep,
} from './copy-transfer-steps.ts';

/**
 Runs the first or the final pass over the reconciled subvolumes.

 @param options - the options; `pass` is `first` or `final`
 
 @param runner - the runner
 
 @param subvolumes - main first, then nested
 
 @throws MoveStopError when any step stops the run
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
  /**
   The is final of this step.
   */
  const isFinal = options.pass === 'final';
  /**
   The prefix of this step.
   */
  const prefix = isFinal ? FINAL_PREFIX : FIRST_PREFIX;
  /**
   The run's record store.
   */
  const records = createRecords({
    stateFolder: options.stateFolder,
    writable: options.run,
  },);
  /**
   The parts of the value this function uses, taken out of it.
   */
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
  /**
   The first ledger of this step.
   */
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
  /**
   The sent of this step.
   */
  const sent: string[] = [];
  for (const subvolume of named) {
    /**
     The context of this step.
     */
    const context: StepContext = {
      options,
      runner,
      records,
      subvolume,
    };
    /**
     The snapshot of this step.
     */
    const snapshot = join(
      options.snapshotFolder,
      subvolume.name,
    );
    /**
     The received of this step.
     */
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
    /**
     The parent entry of the subvolume, when it has one.
     */
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
    /**
     The earlier copies used as clone sources.
     */
    const clones = options.cloneSources === 'main' ? sent.slice(
      0,
      1,
    ) : sent;
    /**
     The send options of this step.
     */
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
