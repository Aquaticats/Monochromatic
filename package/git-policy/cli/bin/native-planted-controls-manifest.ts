/**
 The obligations a campaign's manifest holds: their shape, how they are read from a manifest, and how two are told apart.
 A malformed obligation is never repaired or dropped; the reader says the manifest is invalid.
 */
import {
  readFile,
  stat,
} from 'node:fs/promises';
import { join } from 'node:path';

import { memoryBytes } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/container-limits.ts';
import { isMissing } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/fs-guards.ts';

import type { ExecutionLimits } from './native-gate-proof.ts';
import type { ControlsKey } from './native-planted-controls-record.ts';

/**
 Name of the manifest field that holds the obligation.
 */
export const obligationField = 'plantedControlsDeferred';

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
   The log of that unit, which ends with the record path or with the control that was not noticed.
   */
  readonly log: string;
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
 Read an obligation from the manifest field.

 @param value - the parsed field

 @returns the obligation as the only element, or no element when any part is missing or malformed (nothing is dropped or
 coerced)

 @example
 ```ts
 obligationOf(manifest.plantedControlsDeferred);
 ```
 */
export function obligationOf(value: unknown): readonly PlantedObligation[] {
  if (!isFields(value))
    return [];
  /**
   The fields of the parsed object, each still of unknown type.
   */
  const {
    image,
    unit,
    log,
    invocation,
    limits,
    controls,
    harness,
  } = value;
  if (((typeof image) !== 'string') || ((typeof unit) !== 'string')
    || ((typeof log) !== 'string')
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
    || (pidsLimit < 1)
    || ((typeof sha256) !== 'string')
    || (!Array.isArray(names))
    || (!names.every(function isText(entry: unknown): entry is string {
      return (typeof entry) === 'string';
    })))
    return [];
  try {
    if (memoryBytes(memory) < 1)
      return [];
  }
  catch (error) {
    console.error(`The recorded memory limit ${memory} is not a memory size: ${String(error)}`);
    return [];
  }
  return [{
    image,
    unit,
    log,
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
 Read a text file.

 @param file - path

 @returns the text as the only element, or no element when the file does not exist

 @throws Error when the file exists but cannot be read, so unreadable evidence is never taken for absent evidence

 @example
 ```ts
 await readText('/e/native-mutation-abc/manifest.json');
 ```
 */
async function readText(file: string): Promise<readonly string[]> {
  try {
    return [await readFile(
      file,
      'utf8',
    )];
  }
  catch (error) {
    if (isMissing(error))
      return [];
    throw error;
  }
}

/**
 Tell whether a file exists.

 @param file - path

 @returns false only when it does not exist

 @throws Error when it cannot be examined for another reason

 @example
 ```ts
 await exists('/e/native-mutation-abc/exit.json');
 ```
 */
export async function exists(file: string): Promise<boolean> {
  try {
    await stat(file);
    return true;
  }
  catch (error) {
    if (isMissing(error))
      return false;
    throw error;
  }
}

/**
 Parse JSON text without throwing.

 @param text - the text

 @returns the parsed value, or the reason it is not JSON

 @example
 ```ts
 parseJson('{}');
 ```
 */
function parseJson(text: string): {
  readonly value: unknown;
  readonly failure: string
} {
  try {
    return {
      value: JSON.parse(text),
      failure: '',
    };
  }
  catch (error) {
    return {
      value: undefined,
      failure: String(error),
    };
  }
}

/**
 What a directory's manifest says about obligations.
 */
export type Manifested =
  | { readonly kind: 'absent' }
    | {
      readonly kind: 'obligations';
      readonly list: readonly PlantedObligation[]
    }
    | {
      readonly kind: 'invalid';
      readonly reason: string
    };

/**
 A text that is the same for two obligations exactly when every part of them is the same, whatever order their fields were
 built in.

 @param obligation - the obligation

 @returns the text

 @example
 ```ts
 obligationIdentity(obligation) === obligationIdentity(sameObligation);
 ```
 */
export function obligationIdentity(obligation: PlantedObligation): string {
  return JSON.stringify([
    obligation.image,
    obligation.unit,
    obligation.log,
    obligation.invocation,
    obligation.harness,
    obligation.limits
      .memory,
    obligation.limits
      .pidsLimit,
    obligation.controls
      .sha256,
    obligation.controls
      .names,
  ]);
}

/**
 Read the obligations a directory's manifest holds.

 @param directory - the campaign's evidence directory or its pass-1 directory

 @returns `absent` for a directory without a manifest, the obligations (none when the field is missing), or `invalid` for
 a manifest that cannot be read or whose obligations are not a list of complete obligations

 @example
 ```ts
 await manifested('/e/native-mutation-abc');
 ```
 */
export async function manifested(directory: string): Promise<Manifested> {
  /**
   Path of the manifest.
   */
  const file = join(
    directory,
    'manifest.json',
  );
  /**
   The manifest's text, or none when the directory has no manifest.
   */
  const [text] = await readText(file);
  if (text === undefined)
    return { kind: 'absent' };
  /**
   The parsed manifest, or the reason it cannot be read.
   */
  const parsed = parseJson(text);
  if (!isFields(parsed.value))
    return {
      kind: 'invalid',
      reason: `${file} cannot be read as a JSON object (${parsed.failure})`,
    };
  if (!Object.hasOwn(
    parsed.value,
    obligationField,
  ))
    return {
      kind: 'obligations',
      list: [],
    };
  /**
   The field, which must be a list of complete obligations.
   */
  const field = parsed.value[obligationField];
  /**
   The obligations that parse, one per entry.
   */
  const list = Array.isArray(field) ? field.flatMap(obligationOf) : [];
  return (Array.isArray(field) && (list.length === field.length))
    ? {
      kind: 'obligations',
      list,
    }
    : {
      kind: 'invalid',
      reason: `${file} holds a ${obligationField} that is not a list of complete obligations (image, unit, log, invocation, limits, controls, harness)`,
    };
}
