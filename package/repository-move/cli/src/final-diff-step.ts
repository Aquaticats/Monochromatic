/**
 The question of what the final pass changed since the first pass, answered by btrfs itself: an incremental
 `btrfs send --no-data` between the two snapshots of one subvolume (one history, so the comparison is safe) holds one
 command per change and only the header of the new snapshot when nothing changed. The answer is recorded, so the
 verification before and after the cut-over can leave out the copies that cannot differ from the already verified
 first-pass copy.

 @module
 */

import { join, } from 'node:path';
import { readFile, } from 'node:fs/promises';
import { pathExists, } from './completion-records.ts';
import { parseTextFields, } from './json-text-fields.ts';
import type { StepContext, } from './copy-steps.ts';

/**
 Starts every line of a header the `receive --dump` of an unchanged pair prints; any other line is a change.
 */
const HEADER_WORD = 'snapshot';

/**
 The shell command whose standard output is the dump of what changed between two snapshots.

 Under the runner's `-o pipefail`, a `btrfs send` that is still writing when `btrfs receive --dump` leaves at the
 stream's end record ends with SIGPIPE, which pipefail reports as 141 and which stopped the final pass of the cut-over
 on 2026-11-08 eight times. Measured on throwaway subvolumes (`${HOME}/temp/agent/repository-move-pipe-measure`):
 `--dump` exits at the end record and reads nothing after it, and a stream cut short makes it exit 1 with
 `ERROR: short read from stream`. The `cat` therefore leaves a reader on the pipe after the dump, so the sender always
 finishes, while a dump cut short still fails through the receiver's own status.

 @param firstSnapshot - the first-pass snapshot of the same subvolume

 @param snapshot - the final snapshot

 @returns the command

 @example
 ```ts
 import { diffCommand, } from './final-diff-step.ts';

 const command = diffCommand({ firstSnapshot: '/a', snapshot: '/b', },);
 ```
 */
function diffCommand(
  {
    firstSnapshot,
    snapshot,
  }: {
    readonly firstSnapshot: string;
    readonly snapshot: string;
  },
): string {
  return `btrfs send --no-data -p ${firstSnapshot} ${snapshot} | { btrfs receive --dump; dump=$?; cat > /dev/null; exit $dump; }`;
}

/**
 Asks btrfs what changed between a subvolume's first-pass and final snapshots and records the answer.

 @param context - the subvolume's context

 @param firstSnapshot - the first-pass snapshot of the same subvolume

 @param snapshot - the final snapshot

 @example
 ```ts
 import { diffStep, } from './final-diff-step.ts';

 await diffStep({ context, firstSnapshot, snapshot, },);
 ```
 */
export async function diffStep(
  {
    context,
    firstSnapshot,
    snapshot,
  }: {
    readonly context: StepContext;
    readonly firstSnapshot: string;
    readonly snapshot: string;
  },
): Promise<void> {
  /**
   The parts of the value this function uses, taken out of it.
   */
  const {
    runner,
    records,
    subvolume,
  } = context;
  if (await records.has({
    name: subvolume.name,
    step: 'diff',
  },)) {
    return;
  }
  /**
   The commands the stream holds, one line each.
   */
  const commands = (await runner.exec({
    command: 'bash',
    args: [
      '--noprofile',
      '--norc',
      '-o',
      'pipefail',
      '-c',
      diffCommand({ firstSnapshot, snapshot, },),
    ],
    transfer: true,
  },)).split('\n',)
    .filter(function isChange(line,) {
      return (line !== '') && (!line.startsWith(HEADER_WORD,));
    },);
  if (!context.options
    .run) {
    return;
  }
  runner.log(
    `${subvolume.name}: ${
      commands.length === 0 ? 'unchanged since the first pass' : `changed since the first pass (${String(commands.length,)} commands)`
    }`,
  );
  await records.write({
    name: subvolume.name,
    step: 'diff',
    data: {
      changed: commands.length === 0 ? 'no' : 'yes',
      commands: String(commands.length,),
    },
  },);
}

/**
 Tells whether btrfs found nothing changed in a final copy since the first pass.

 @param stateFolder - the state folder

 @param name - the final copy's name

 @returns true only when a record says no change; a copy with no record (it has no first-pass copy) counts as changed

 @example
 ```ts
 import { foundUnchanged, } from './final-diff-step.ts';

 const unchanged = await foundUnchanged({ stateFolder, name: 'f001', },);
 ```
 */
export async function foundUnchanged(
  {
    stateFolder,
    name,
  }: {
    readonly stateFolder: string;
    readonly name: string;
  },
): Promise<boolean> {
  /**
   The record file.
   */
  const path = join(
    stateFolder,
    `${name}.diff.json`,
  );
  if (!await pathExists(path,)) {
    return false;
  }
  return parseTextFields(await readFile(
    path,
    'utf8',
  ),)
    .changed
    === 'no';
}
