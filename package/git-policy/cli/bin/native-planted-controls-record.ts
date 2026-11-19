/**
 The record a passing run of the planted controls leaves: its shape, its digest of the control table, and its atomic write.
 */
import {
  createHash,
  randomUUID,
} from 'node:crypto';
import {
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { caughtValueText } from '@monochromatic-dev/caught-value-core/ts';

import type { PlantedControl } from './native-planted-controls.ts';
import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

/**
 Name of the record inside a campaign's evidence directory.
 */
export const plantedControlsRecordName = 'planted-controls-passed.json';

/**
 Digest that identifies the full definition of every control, whatever order the table lists them in.

 @param controls - the controls a run planted

 @returns the hex SHA-256 over the controls sorted by name, each as its name, file, replaced text, new text and the test
 that must fail, so that editing any of them changes the digest

 @example
 ```ts
 plantedControlsDigest(plantedControls);
 ```
 */
export function plantedControlsDigest(controls: readonly PlantedControl[]): string {
  return createHash('sha256')
    .update(JSON.stringify(controls.toSorted(function byName(
      left,
      right
    ) {
      return left.name
        .localeCompare(right.name);
    })
      .map(function definitionOf(control) {
        return [
          control.name,
          control.file,
          control.from,
          control.to,
          control.failing,
        ];
      })))
    .digest('hex');
}

/**
 What a record must say to prove a set of controls: the digest of their definitions and their sorted names.
 */
export type ControlsKey = {
  readonly sha256: string;
  readonly names: readonly string[];
};

/**
 The key of a set of controls.

 @param controls - the controls a campaign would plant

 @returns their digest and sorted names, which a status file keeps so that a later settlement matches records against the
 controls the campaign deferred under and not against the settling process's own table

 @example
 ```ts
 controlsKeyOf(plantedControls);
 ```
 */
export function controlsKeyOf(controls: readonly PlantedControl[]): ControlsKey {
  return {
    sha256: plantedControlsDigest(controls),
    names: controls.map(function nameOf(control) {
      return control.name;
    })
      .toSorted(function byText(
        left,
        right
      ) {
        return left.localeCompare(right);
      }),
  };
}

/**
 Build the record written once every planted control was noticed.

 @param image - full ID of the image the controls ran over

 @param limits - the Podman arguments the controls' containers ran with, which carry the memory and process limit

 @param controls - the controls that ran

 @param harness - identity of the host-side runner and judge that ran them (`plantedHarnessId`)

 @returns the record, which stands alone: it names the image, the limits, the controls and the code that judged them

 @example
 ```ts
 plantedControlsPassedRecord({ image, limits: ['--memory=3g', '--pids-limit=512'], controls: plantedControls, harness });
 ```
 */
export function plantedControlsPassedRecord({
  image,
  limits,
  controls,
  harness,
}: {
  readonly image: string;
  readonly limits: readonly string[];
  readonly controls: readonly PlantedControl[];
  readonly harness: string;
}): Record<string, unknown> {
  return {
    plantedControls: true,
    image,
    limits,
    controls: controls.map(function nameOf(control) {
      return control.name;
    }),
    controlsSha256: plantedControlsDigest(controls),
    harnessId: harness,
  };
}

/**
 Sync stall bound sized against recorded write-and-sync readings, not command-start latency.
 `doc/handover/agents-at-once-on-this-machine.md` ("Readings on the new drive") records 35 readings of 4 KiB with
 up to 13 Pi units: maxima of 72 ms on home and 2,564 ms on cargo-build.
 `doc/handover/cli-git-rust-implementation.md` records a maximum of 15,512 ms on the earlier drive with 18 agents
 held for eight hours. The bound leaves 44,488 ms beyond that largest observed time.
 */
export const SYNC_DEADLINE_MILLISECONDS: number = 60_000;

/**
 Flush a closed record by path because Node disables descriptor sync under its permission model.

 @param path - Private record to sync before rename.

 @param runSync - Command runner that enforces the sync deadline.

 @throws {@link NativeVerificationError} when `sync` cannot start or finish successfully within its deadline.

 @example
 ```ts
 await syncWrittenPath({ path: '/e/private-record.tmp', runSync: runCommand });
 ```
 */
async function syncWrittenPath({
  path,
  runSync,
}: {
  readonly path: string;
  readonly runSync: typeof runCommand;
}): Promise<void> {
  try {
    await runSync({
      command: 'sync',
      args: [
        '--data',
        '--',
        path,
      ],
      capture: true,
      deadlineMilliseconds: SYNC_DEADLINE_MILLISECONDS,
    });
  }
  catch (error) {
    /**
     The runner retains a refused spawn as a cause, which a campaign's one-line diagnostic must name too.
     */
    const detail = (error instanceof NativeVerificationError) && (error.cause !== undefined)
      ? `${String(error)} ${caughtValueText(error.cause)}`
      : String(error);
    throw new NativeVerificationError(
      `Cannot sync ${path} with GNU coreutils: ${detail}`,
      { cause: error },
    );
  }
}

/**
 Write a file so that a reader sees all of it or none of it, and a crash leaves no torn record.

 @param file - destination path

 @param text - file text

 @param runSync - Command runner for the temporary-file sync, normally the verification runner.

 @throws {@link NativeVerificationError} when the temporary file cannot be synced before publication.

 @example
 ```ts
 await writeFileAtomically({ file: '/e/native-mutation-abc/planted-controls-passed.json', text: '{}\n' });
 ```
 */
export async function writeFileAtomically({
  file,
  text,
  runSync = runCommand,
}: {
  readonly file: string;
  readonly text: string;
  readonly runSync?: typeof runCommand;
}): Promise<void> {
  /**
   Sibling path that only this writer uses, so a reader looking for the destination name never sees it.
   */
  const temporary = `${file}.${String(process.pid)}.${randomUUID()}.tmp`;
  try {
    await writeFile(
      temporary,
      text,
      {
        flag: 'wx',
        encoding: 'utf8',
      },
    );
    // Flush the closed temporary file before rename so power loss cannot publish an empty destination.
    await syncWrittenPath({
      path: temporary,
      runSync,
    });
  }
  catch (error) {
    await rm(
      temporary,
      { force: true },
    );
    throw error;
  }
  try {
    await rename(
      temporary,
      file,
    );
  }
  catch (error) {
    await rm(
      temporary,
      { force: true },
    );
    throw error;
  }
}

/**
 Write the record that the planted controls passed, in the evidence directory they wrote their logs to.

 @param evidence - the campaign's evidence directory (or its pass-1 directory)

 @param record - the record from {@link plantedControlsPassedRecord}

 @returns the path of the record

 @example
 ```ts
 await writePlantedControlsRecord({ evidence, record });
 ```
 */
export async function writePlantedControlsRecord({
  evidence,
  record,
}: {
  readonly evidence: string;
  readonly record: Readonly<Record<string, unknown>>;
}): Promise<string> {
  /**
   Where the record goes.
   */
  const file = join(
    evidence,
    plantedControlsRecordName,
  );
  await writeFileAtomically({
    file,
    text: `${JSON.stringify(
      record,
      null,
      2,
    )}\n`,
  });
  return file;
}
