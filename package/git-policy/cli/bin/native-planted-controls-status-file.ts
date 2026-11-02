/**
 The planted-controls status file a deferring campaign leaves beside its evidence: its shape, how it is written and read,
 and how a later attempt retires it. Reading tells an absent file from a valid one and from one that cannot be used.
 */
import {
  stat,
} from 'node:fs/promises';
import { join } from 'node:path';

import { isMissing } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/fs-guards.ts';
import { memoryBytes } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/container-limits.ts';

import {
  type ExecutionLimits,
  readGateRecord,
} from './native-gate-proof.ts';
import {
  type ControlsKey,
  writeFileAtomically,
} from './native-planted-controls-record.ts';

/**
 Name of the status file inside a campaign's evidence directory (or its pass-1 directory).
 */
export const plantedStatusName = 'planted-controls-status.json';

/**
 Everything a campaign deferred under: what a record must prove, and the invocation expected to leave it.
 */
export type PlantedObligation = {
  readonly image: string;
  /**
   The controls unit the campaign deferred to.
   */
  readonly unit: string;
  /**
   The systemd invocation of that unit at the time of the deferral, so a later unit that reuses the name does not keep
   the obligation alive.
   */
  readonly invocation: string;
  /**
   The memory and process limit the campaign ran with, which a record must have been made at or under.
   */
  readonly limits: ExecutionLimits;
  readonly controls: ControlsKey;
  /**
   Identity of the runner and judge the campaign deferred under.
   */
  readonly harness: string;
};

/**
 The state of a deferred campaign's planted controls.
 */
export type PlantedStatus =
  | (PlantedObligation & {
    readonly status: 'pending' | 'validated' | 'unvalidated';
    /**
     The record that validated the campaign.
     */
    readonly record?: string;
  })
    | {
      readonly status: 'retired';
    /**
     What the later attempt did instead: `requested`, `proved` or `ran`.
     */
      readonly retiredBy: string;
    };

/**
 What reading a status file found.
 */
export type StatusRead =
  | { readonly kind: 'absent' }
    | {
      readonly kind: 'valid';
      readonly status: PlantedStatus
    }
    | {
      readonly kind: 'invalid';
      readonly reason: string
    };

/**
 Write a campaign's planted-controls status.

 @param evidence - the campaign's evidence directory (or its pass-1 directory)

 @param status - the status

 @example
 ```ts
 await writePlantedStatus({ evidence, status });
 ```
 */
export async function writePlantedStatus({
  evidence,
  status,
}: {
  readonly evidence: string;
  readonly status: PlantedStatus;
}): Promise<void> {
  await writeFileAtomically({
    file: join(
      evidence,
      plantedStatusName,
    ),
    text: `${JSON.stringify(
      status,
      null,
      2,
    )}\n`,
  });
}

/**
 Check that a value is a list of texts.

 @param value - parsed JSON

 @returns true for an array whose entries are all texts

 @example
 ```ts
 isTextList(['a']);
 ```
 */
function isTextList(value: unknown): value is readonly string[] {
  return Array.isArray(value) && value.every(function isText(entry: unknown) {
    return (typeof entry) === 'string';
  });
}

/**
 Check that a value is a record.

 @param value - parsed JSON

 @returns true for a non-null object that is not an array

 @example
 ```ts
 isFields({});
 ```
 */
function isFields(value: unknown): value is Readonly<Record<string, unknown>> {
  return (value !== null) && ((typeof value) === 'object')
    && (!Array.isArray(value));
}

/**
 Check that a value is one of the states a deferral can be in before it is retired.

 @param value - parsed JSON

 @returns true for `pending`, `validated` and `unvalidated`

 @example
 ```ts
 isStatusName('pending');
 ```
 */
function isStatusName(value: unknown): value is 'pending' | 'validated' | 'unvalidated' {
  return (value === 'pending') || (value === 'validated')
    || (value === 'unvalidated');
}

/**
 Read the obligation fields of a parsed status.

 @param record - the parsed status file

 @returns the obligation as the only element, or no element when any field is missing or malformed (nothing is dropped
 or coerced)

 @example
 ```ts
 obligationOf(record);
 ```
 */
function obligationOf(record: Readonly<Record<string, unknown>>): readonly PlantedObligation[] {
  /**
   The fields of the parsed status, each still of unknown type.
   */
  const {
    image,
    unit,
    invocation,
    limits,
    controls,
    harness,
  } = record;
  if (((typeof image) !== 'string') || ((typeof unit) !== 'string')
    || ((typeof invocation) !== 'string')
    || ((typeof harness) !== 'string')
    || (!isFields(limits))
    || (!isFields(controls)))
    return [];
  /**
   The limits' fields.
   */
  const {
    memory,
    pidsLimit,
  } = limits;
  /**
   The controls key's fields.
   */
  const {
    sha256,
    names,
  } = controls;
  if (((typeof memory) !== 'string') || ((typeof pidsLimit) !== 'number')
    || (!Number.isInteger(pidsLimit))
    || ((typeof sha256) !== 'string')
    || (!isTextList(names)))
    return [];
  try {
    if (memoryBytes(memory) < 1)
      return [];
  }
  catch (error) {
    console.error(`The status memory limit ${memory} is not a memory size: ${String(error)}`);
    return [];
  }
  return [{
    image,
    unit,
    invocation,
    harness,
    limits: {
      memory,
      pidsLimit,
    },
    controls: {
      sha256,
      names,
    },
  }];
}

/**
 Read a status file, telling an absent one from a valid one and from one that cannot be used.

 @param evidence - a directory that may hold one

 @returns `absent` when there is no file, `valid` with the status, or `invalid` with the reason

 @example
 ```ts
 await readPlantedStatus(evidence);
 ```
 */
export async function readPlantedStatus(evidence: string): Promise<StatusRead> {
  /**
   Path of the status file.
   */
  const file = join(
    evidence,
    plantedStatusName,
  );
  /**
   The parsed file as the only element, or none when it is absent, torn or not an object.
   */
  const [record] = await readGateRecord(file);
  if (record === undefined)
    return (await fileExists(file))
      ? {
        kind: 'invalid',
        reason: `${file} exists but cannot be read as a JSON object`,
      }
      : { kind: 'absent' };
  /**
   The fields that tell which state the file records.
   */
  const {
    status,
    retiredBy,
    record: savedRecord,
  } = record;
  if (status === 'retired')
    return (typeof retiredBy) === 'string'
      ? {
        kind: 'valid',
        status: {
          status: 'retired',
          retiredBy,
        },
      }
      : {
        kind: 'invalid',
        reason: `${file} is retired without saying by what`,
      };
  if (!isStatusName(status))
    return {
      kind: 'invalid',
      reason: `${file} has the status ${JSON.stringify(status)}`,
    };
  /**
   The obligation as the only element, or none when it is incomplete.
   */
  const [obligation] = obligationOf(record);
  if (obligation === undefined)
    return {
      kind: 'invalid',
      reason: `${file} does not hold a complete obligation (image, unit, invocation, limits, controls, harness)`,
    };
  return {
    kind: 'valid',
    status: {
      ...obligation,
      status,
      ...((typeof savedRecord) === 'string' ? { record: savedRecord } : {}),
    },
  };
}

/**
 Tell whether a file exists.

 @param file - path

 @returns false only when it does not exist

 @example
 ```ts
 await fileExists('/e/planted-controls-status.json');
 ```
 */
async function fileExists(file: string): Promise<boolean> {
  try {
    await stat(file);
    return true;
  }
  catch (error) {
    // Only a missing file is absent; one that cannot even be examined is there, and unreadable.
    return !isMissing(error);
  }
}

/**
 Retire a deferred status after the campaign ran the controls itself, found a proof, or was told to skip.

 @param evidence - the campaign's evidence directory (or its pass-1 directory)

 @param by - what the campaign did instead

 @example
 ```ts
 await retirePlantedStatus({ evidence, by: 'ran' });
 ```
 */
export async function retirePlantedStatus({
  evidence,
  by,
}: {
  readonly evidence: string;
  readonly by: string;
}): Promise<void> {
  /**
   What the directory holds now.
   */
  const read = await readPlantedStatus(evidence);
  // An earlier deferral that is unreadable is abandoned too: this attempt's own observation replaces it.
  if ((read.kind === 'absent') || ((read.kind === 'valid') && (read.status
    .status
    === 'retired')))
    return;
  await writePlantedStatus({
    evidence,
    status: {
      status: 'retired',
      retiredBy: by,
    },
  });
}
