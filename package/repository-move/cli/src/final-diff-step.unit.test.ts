/**
 Test of the final pass's diff step, which asks btrfs what changed between a subvolume's two snapshots.

 The step's command is `btrfs send --no-data -p <first> <final> | btrfs receive --dump` under `-o pipefail`. Measured
 on throwaway subvolumes (`${HOME}/temp/agent/repository-move-pipe-measure`, 2026-11-08): a real `--no-data` stream ends
 exactly at its end record, `--dump` leaves at that record without reading further, and a stream cut short makes it exit
 1 with `ERROR: short read from stream`. The cut-over of 2026-11-08 stopped this step with status 141 eight times: a
 sender still writing when the receiver left. This test holds a complete dump beside a sender that keeps writing after
 the end record, which is that shape, and a dump cut short, which must still fail the step.

 @module
 */

import {
  chmod,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  createRunner,
  diffStep,
  MoveStopError,
  parseMoveOptions,
  startDeadlineClock,
} from '../dist/final/node/index.mjs';

/**
 The name the step gives the subvolume in its record.
 */
const SUBVOLUME_NAME = 'f000';

/**
 The subvolume's path, as the copy tool names it.
 */
const SUBVOLUME_PATH = '/source/f000';

/**
 The subvolume's UUID.
 */
const SUBVOLUME_UUID = '11111111-2222-3333-4444-555555555555';

/**
 The first-pass snapshot of the subvolume.
 */
const FIRST_SNAPSHOT = '/snapshots/s000';

/**
 The final snapshot of the subvolume.
 */
const FINAL_SNAPSHOT = '/snapshots/f000';

/**
 A line `btrfs receive --dump` prints for the stream's own header, which the step does not count as a change.
 */
const HEADER_LINE = `snapshot ./${SUBVOLUME_NAME} uuid=${SUBVOLUME_UUID}`;

/**
 A line `btrfs receive --dump` prints for one command of the stream, which the step counts as a change.
 */
const CHANGE_LINE = 'write ./file offset=0 len=1';

/**
 The record marker `btrfs send` writes last.
 */
const END_LINE = 'end';

/**
 Bytes the stand-in sender writes after the end record, more than a pipe holds, so a reader that leaves at the end
 record ends that sender with SIGPIPE.
 */
const BYTES_AFTER_END = 512 * 1_024;

/**
 The commands a complete dump with one change holds.
 */
const ONE_CHANGE = 1;

/**
 The commands a dump holding only the stream's header holds.
 */
const NO_CHANGE = 0;

/**
 Permissions of the stand-in `btrfs`: owner may read and run it.
 */
const STAND_IN_MODE = 0o700;

/**
 A disposable scratch folder, removed when the scope ends.
 */
type Scratch = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 What the stand-in btrfs does when the dump reaches the end record.
 */
type Dump = 'complete' | 'short-read';

/**
 What the stand-in sender writes.
 */
type Stream = {
  /**
   Command lines the sender writes before the end record.
   */
  readonly changes: number;
};

/**
 Makes a disposable scratch folder.

 @returns the folder, to hold with `await using`

 @example
 ```ts
 await using scratch = await makeScratch();
 ```
 */
async function makeScratch(): Promise<Scratch> {
  /**
   Path of the new folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'final-diff-step-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 The stand-in `btrfs`: a sender that writes a stream and then keeps writing, and a receiver that dumps it.

 @param paddingFile - the file whose bytes the sender writes after the end record

 @param dump - how the receiver ends

 @param changes - command lines the sender writes before the end record

 @returns the script's text

 @example
 ```ts
 const script = standIn({ paddingFile: '/x/padding', dump: 'complete', changes: 1, });
 ```
 */
function standIn(
  {
    paddingFile,
    dump,
    changes,
  }: {
    readonly paddingFile: string;
    readonly dump: Dump;
    readonly changes: number;
  },
): string {
  /**
   What the receiver does after printing the last line it read, when its stream is cut short.
   */
  const shortRead = dump === 'short-read'
    ? `        printf 'ERROR: short read from stream: expected 10 read 6\\n' >&2\n        exit 1\n`
    : '';
  /**
   The command lines the sender writes, one `printf` each.
   */
  const changeLines = Array.from({ length: changes, }, () => `    printf '%s\\n' '${CHANGE_LINE}'\n`,)
    .join('',);
  return `#!/usr/bin/env bash
case "$1" in
  send)
    printf '%s\\n' '${HEADER_LINE}'
${changeLines}    printf '%s\\n' '${END_LINE}'
    cat '${paddingFile}'
    ;;
  receive)
    while IFS= read -r line; do
      if [ "$line" = '${END_LINE}' ]; then
        exit 0
      fi
      printf '%s\\n' "$line"
${shortRead}    done
    exit 0
    ;;
  *)
    exit 2
    ;;
esac
`;
}

/**
 What one run of the step did.
 */
type StepRun = {
  readonly error: unknown;
  readonly records: readonly { readonly data: Record<string, string> }[];
  readonly logged: readonly string[];
};

/**
 Runs the diff step against a stand-in btrfs that prints the given stream.

 @param dump - how the stand-in receiver ends

 @param stream - what the stand-in sender writes

 @param bytesAfterEnd - bytes the stand-in sender writes after the end record

 @returns the scratch folder, the error it threw, the records it wrote and what it logged

 @example
 ```ts
 const { run, } = await runStep({ dump: 'complete', stream: { changes: 1, }, bytesAfterEnd: 0, });
 ```
 */
async function runStep(
  {
    dump,
    stream,
    bytesAfterEnd,
  }: {
    readonly dump: Dump;
    readonly stream: Stream;
    readonly bytesAfterEnd: number;
  },
): Promise<{
  readonly scratch: Scratch;
  readonly run: StepRun;
}> {
  /**
   The folder this run reads and writes.
   */
  const scratch = await makeScratch();
  /**
   The file the stand-in sender writes after the end record.
   */
  const paddingFile = join(scratch.path, 'padding',);
  await writeFile(paddingFile, 'x'.repeat(bytesAfterEnd,),);
  /**
   The stand-in program, found first on the shell's path.
   */
  const program = join(scratch.path, 'btrfs',);
  await writeFile(program, standIn({
    paddingFile,
    dump,
    changes: stream.changes,
  },),);
  await chmod(program, STAND_IN_MODE,);
  /**
   The step's options: a run of the final pass, with no systemd scope.
   */
  const options = parseMoveOptions([
    '--pass=final',
    `--main=${join(scratch.path, 'main',)}`,
    `--snapshots=${join(scratch.path, 'snapshots',)}`,
    `--received=${join(scratch.path, 'received',)}`,
    `--final=${join(scratch.path, 'final',)}`,
    `--state=${join(scratch.path, 'state',)}`,
    '--run',
    '--no-scope',
  ],);
  /**
   The runner of this invocation, as the copy tool builds it.
   */
  const runner = createRunner({
    options,
    clock: startDeadlineClock({ totalSeconds: options.totalDeadlineSeconds, },),
  },);
  /**
   The records the step wrote.
   */
  const records: { readonly data: Record<string, string> }[] = [];
  /**
   What the step logged.
   */
  const logged: string[] = [];
  /**
   The search path this run gives its commands, with the stand-in first.
   */
  const path = `${scratch.path}:${process.env.PATH ?? ''}`;
  const savedPath = process.env.PATH;
  let error: unknown = null;
  process.env.PATH = path;
  try {
    await diffStep({
      context: {
        options,
        runner: {
          exec: runner.exec,
          log: (message: string,) => {
            logged.push(message,);
          },
        },
        records: {
          has: async () => false,
          write: async (entry: { readonly data: object },) => {
            records.push({ data: entry.data as Record<string, string> });
          },
        },
        subvolume: {
          name: SUBVOLUME_NAME,
          path: SUBVOLUME_PATH,
          uuid: SUBVOLUME_UUID,
        },
      },
      firstSnapshot: FIRST_SNAPSHOT,
      snapshot: FINAL_SNAPSHOT,
    },);
  }
  catch (caught) {
    error = caught;
  }
  finally {
    process.env.PATH = savedPath;
  }
  return {
    scratch,
    run: { error, records, logged },
  };
}

await describe({
  name: 'the diff step of the final pass',
  children: [
    it({
      name: 'records one change from a complete dump',
      fn: async () => {
        const { scratch, run, } = await runStep({
          dump: 'complete',
          stream: { changes: ONE_CHANGE, },
          bytesAfterEnd: 0,
        },);
        await scratch[Symbol.asyncDispose]();
        expect(run.error,).toBeNull();
        expect(run.records.length,).toBe(1,);
        expect(run.records[0]?.data.changed,).toBe('yes',);
        expect(run.records[0]?.data.commands,).toBe(String(ONE_CHANGE,),);
        expect(run.logged.join('\n',),).toContain('changed since the first pass',);
      },
    },),
    it({
      name: 'records no change from a dump holding only the stream header',
      fn: async () => {
        const { scratch, run, } = await runStep({
          dump: 'complete',
          stream: { changes: NO_CHANGE, },
          bytesAfterEnd: 0,
        },);
        await scratch[Symbol.asyncDispose]();
        expect(run.error,).toBeNull();
        expect(run.records[0]?.data.changed,).toBe('no',);
        expect(run.records[0]?.data.commands,).toBe(String(NO_CHANGE,),);
      },
    },),
    it({
      name: 'does not fail when the sender is still writing after a complete dump',
      fn: async () => {
        const { scratch, run, } = await runStep({
          dump: 'complete',
          stream: { changes: ONE_CHANGE, },
          bytesAfterEnd: BYTES_AFTER_END,
        },);
        await scratch[Symbol.asyncDispose]();
        expect(run.error,).toBeNull();
        expect(run.records[0]?.data.changed,).toBe('yes',);
      },
    },),
    it({
      name: 'fails on a dump cut short',
      fn: async () => {
        const { scratch, run, } = await runStep({
          dump: 'short-read',
          stream: { changes: ONE_CHANGE, },
          bytesAfterEnd: BYTES_AFTER_END,
        },);
        await scratch[Symbol.asyncDispose]();
        expect(run.error,).toBeInstanceOf(MoveStopError,);
        expect(run.records.length,).toBe(0,);
      },
    },),
  ],
},);
