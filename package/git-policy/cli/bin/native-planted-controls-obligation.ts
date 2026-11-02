/**
 What a campaign that deferred its planted controls to a controls unit still owes, and whether it is met.

 A campaign started by `native:mutation:parallel` skips the controls while another unit runs them, so its mutant results
 hold only once that unit leaves a matching record. The obligation is written once, with the campaign's manifest, as
 `plantedControlsDeferred`: the image, the limits the campaign ran under, the digest and names of the controls, the
 identity of the runner and judge, and the controls unit with its systemd invocation and log. Nothing else is stored.

 Whether the obligation is met is a question asked on demand, never an answer kept: a matching record exists (found with
 the same reader a campaign uses, by the saved keys and not by the asking process's own table or code), or it does not.
 With no record, the campaign is `pending` while that very invocation runs, and `unvalidated` once it ended. A manifest
 whose field is malformed is `invalid`. A later attempt that runs the controls itself writes a manifest without the field,
 which ends the obligation by construction.
 */
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

import { passOneDirectoryName } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/campaign-passes.ts';
import { memoryBytes } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/container-limits.ts';
import { isMissing } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/fs-guards.ts';

import type { ExecutionLimits } from './native-gate-proof.ts';
import { NativeVerificationError } from './native-verification-process.ts';
import { passingPlantedControls } from './native-planted-controls-proof.ts';
import type { ControlsKey } from './native-planted-controls-record.ts';
import {
  inspectUnit,
  isInvocationRunning,
  type UnitState,
} from './native-planted-controls-unit.ts';

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
 How a directory's obligation stands.
 */
export type Validation =
  | {
    readonly directory: string;
    readonly state: 'validated';
    readonly obligation: PlantedObligation;
    readonly record: string
  }
    | {
      readonly directory: string;
      readonly state: 'pending' | 'unvalidated';
      readonly obligation: PlantedObligation
    }
    | {
      readonly directory: string;
      readonly state: 'invalid';
      readonly reason: string
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
 Look for a record that proves an obligation.

 @param obligation - what the campaign deferred under

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run

 @param evidencePrefix - name prefix of the campaigns' directories

 @returns the record's path as the only element, or none

 @example
 ```ts
 await proofOf({ obligation, evidenceRoot, evidencePrefix: 'native-mutation-' });
 ```
 */
async function proofOf({
  obligation,
  evidenceRoot,
  evidencePrefix,
}: {
  readonly obligation: PlantedObligation;
  readonly evidenceRoot: string;
  readonly evidencePrefix: string;
}): Promise<readonly string[]> {
  return await passingPlantedControls({
    evidenceRoot,
    evidencePrefix,
    image: obligation.image,
    limits: obligation.limits,
    key: obligation.controls,
    harness: obligation.harness,
  });
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
 Validate the obligation one directory's manifest holds.

 @param directory - the campaign's evidence directory or its pass-1 directory

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run

 @param evidencePrefix - name prefix of the campaigns' directories

 @param inspect - asks systemd about the controls unit

 @returns how the obligation stands, none when the directory has no manifest or its manifest defers nothing

 @example
 ```ts
 await validateDirectory({ directory, evidenceRoot, evidencePrefix: 'native-mutation-', inspect: inspectUnit });
 ```
 */
async function validateDirectory({
  directory,
  evidenceRoot,
  evidencePrefix,
  inspect,
}: {
  readonly directory: string;
  readonly evidenceRoot: string;
  readonly evidencePrefix: string;
  readonly inspect: (unit: string) => Promise<UnitState>;
}): Promise<readonly Validation[]> {
  /**
   Path of the manifest.
   */
  const file = join(
    directory,
    'manifest.json',
  );
  /**
   The manifest's text, or none when the directory has no manifest yet.
   */
  const [text] = await readText(file);
  if (text === undefined)
    return [];
  /**
   The parsed manifest, or the reason it cannot be read.
   */
  const parsed = parseJson(text);
  if (!isFields(parsed.value))
    return [{
      directory,
      state: 'invalid',
      reason: `${file} cannot be read as a JSON object (${parsed.failure})`,
    }];
  if (!Object.hasOwn(
    parsed.value,
    obligationField,
  ))
    return [];
  /**
   The obligation as the only element, or none when it is malformed.
   */
  const [obligation] = obligationOf(parsed.value[obligationField]);
  if (obligation === undefined)
    return [{
      directory,
      state: 'invalid',
      reason: `${file} holds a ${obligationField} that is not a complete obligation (image, unit, log, invocation, limits, controls, harness)`,
    }];
  /**
   A proving record as the only element, or none.
   */
  const [found] = await proofOf({
    obligation,
    evidenceRoot,
    evidencePrefix,
  });
  if (found !== undefined)
    return [{
      directory,
      state: 'validated',
      obligation,
      record: found,
    }];
  if (isInvocationRunning({
    state: await inspect(obligation.unit),
    invocation: obligation.invocation,
  }))
    return [{
      directory,
      state: 'pending',
      obligation,
    }];
  /**
   The record once more: the unit may have published it between the lookup above and the answer that it ended.
   */
  const [late] = await proofOf({
    obligation,
    evidenceRoot,
    evidencePrefix,
  });
  return [late === undefined
    ? {
      directory,
      state: 'unvalidated',
      obligation,
    }
    : {
      directory,
      state: 'validated',
      obligation,
      record: late,
    }];
}

/**
 Validate the obligation of one campaign (and of its pass-1 directory).

 @param evidence - the campaign's evidence directory

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run

 @param evidencePrefix - name prefix of the campaigns' directories

 @param inspect - asks systemd about the controls unit; defaults to `systemctl --user show`

 @returns one entry per directory that deferred, none for a campaign that deferred nothing

 @example
 ```ts
 await validatePlanted({ evidence, evidenceRoot, evidencePrefix: 'native-mutation-' });
 ```
 */
export async function validatePlanted({
  evidence,
  evidenceRoot,
  evidencePrefix,
  inspect = inspectUnit,
}: {
  readonly evidence: string;
  readonly evidenceRoot: string;
  readonly evidencePrefix: string;
  readonly inspect?: (unit: string) => Promise<UnitState>;
}): Promise<readonly Validation[]> {
  return (await Promise.all([
    evidence,
    join(
      evidence,
      passOneDirectoryName,
    ),
  ].map(function validateIn(directory) {
    return validateDirectory({
      directory,
      evidenceRoot,
      evidencePrefix,
      inspect,
    });
  }))).flat();
}

/**
 Say how a directory's obligation stands, with where the proof lives.

 @param validation - the result for one directory

 @returns one line for the log: the record path, or "no record yet" with the controls unit, its invocation and its log

 @example
 ```ts
 describeValidation(validation);
 ```
 */
export function describeValidation(validation: Validation): string {
  if (validation.state === 'invalid')
    return `Planted controls of ${validation.directory}: invalid, ${validation.reason}.`;
  /**
   The unit the obligation names.
   */
  const { obligation } = validation;
  /**
   Where the unit and its log are.
   */
  const where = `unit ${obligation.unit} (invocation ${obligation.invocation}), log ${obligation.log}`;
  if (validation.state === 'validated')
    return `Planted controls of ${validation.directory}: validated by ${validation.record}.`;
  return validation.state === 'pending'
    ? `Planted controls of ${validation.directory}: no record yet; ${where} still runs, so the mutant results hold only once it leaves a record.`
    : `Planted controls of ${validation.directory}: unvalidated; ${where} ended without a matching record.`;
}

/**
 Fail when a campaign's obligation is unreadable or unmet because the controls unit ended without a matching record.

 Pending is not a failure: the unit still runs, and its log says where the proof will be.

 @param validations - the results for the campaign's directories

 @throws NativeVerificationError naming each directory that is invalid or unvalidated, so a campaign whose controls failed
 can never pass

 @example
 ```ts
 assertNotUnmet(await validatePlanted({ evidence, evidenceRoot, evidencePrefix }));
 ```
 */
export function assertNotUnmet(validations: readonly Validation[]): void {
  /**
   Results that are invalid or unvalidated.
   */
  const failed = validations.filter(function isFailed(validation) {
    return (validation.state === 'invalid') || (validation.state === 'unvalidated');
  });
  if (failed.length > 0)
    throw new NativeVerificationError(`The mutants of this campaign were scored while a controls unit was to prove the planted controls, and that is not established. ${failed.map(function describe(validation) {
      return describeValidation(validation);
    })
      .join(' ')} The results are not validated; inspect the controls unit's log, then start this campaign again (it resumes) once a record exists.`);
}
