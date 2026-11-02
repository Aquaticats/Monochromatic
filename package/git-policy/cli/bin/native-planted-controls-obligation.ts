/**
 What a campaign that deferred its planted controls to a controls unit still owes, and whether it is met.

 A campaign started by `native:mutation:parallel` skips the controls while another unit runs them, so its mutant results
 hold only once that unit leaves a matching record. The obligations are written with the campaign's manifest, as the list
 `plantedControlsDeferred`: each holds the image, the limits the campaign ran under, the digest and names of the controls,
 the identity of the runner and judge, and the controls unit with its systemd invocation and log. Nothing else is stored.

 Whether an obligation is met is a question asked on demand, never an answer kept: a matching record exists (found with
 the same reader a campaign uses, by the saved keys and not by the asking process's own table or code), or it does not.
 With no record, it is `pending` while that very invocation runs, and `unvalidated` once it ended.

 An obligation ends only when something pays it. Each time a campaign writes a manifest it carries forward the obligations
 of the manifests it replaces (its own directory's, and the pass-1 directory's), and drops one only when this attempt's
 proof or local pass satisfies that obligation's own keys, or when the person asked to skip the controls. A manifest that
 is missing after the campaign finished, or an obligation the running process expects that the manifest lacks, is invalid.
 */
import { join } from 'node:path';

import { passOneDirectoryName } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/campaign-passes.ts';

import {
  exists,
  manifested,
  obligationIdentity,
  type PlantedObligation,
} from './native-planted-controls-manifest.ts';
import { passingPlantedControls } from './native-planted-controls-proof.ts';
import {
  inspectUnit,
  isInvocationRunning,
  type UnitState,
} from './native-planted-controls-unit.ts';
import { NativeVerificationError } from './native-verification-process.ts';

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
 Look up one obligation.

 @param obligation - what the campaign deferred under

 @param directory - the directory whose manifest holds it

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run

 @param evidencePrefix - name prefix of the campaigns' directories

 @param inspect - asks systemd about the controls unit

 @returns how it stands

 @example
 ```ts
 await lookUp({ obligation, directory, evidenceRoot, evidencePrefix: 'native-mutation-', inspect: inspectUnit });
 ```
 */
async function lookUp({
  obligation,
  directory,
  evidenceRoot,
  evidencePrefix,
  inspect,
}: {
  readonly obligation: PlantedObligation;
  readonly directory: string;
  readonly evidenceRoot: string;
  readonly evidencePrefix: string;
  readonly inspect: (unit: string) => Promise<UnitState>;
}): Promise<Validation> {
  /**
   A proving record as the only element, or none.
   */
  const [found] = await proofOf({
    obligation,
    evidenceRoot,
    evidencePrefix,
  });
  if (found !== undefined)
    return {
      directory,
      state: 'validated',
      obligation,
      record: found,
    };
  if (isInvocationRunning({
    state: await inspect(obligation.unit),
    invocation: obligation.invocation,
  }))
    return {
      directory,
      state: 'pending',
      obligation,
    };
  /**
   The record once more: the unit may have published it between the lookup above and the answer that it ended.
   */
  const [late] = await proofOf({
    obligation,
    evidenceRoot,
    evidencePrefix,
  });
  return late === undefined
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
    };
}

/**
 Validate the obligations one directory's manifest holds.

 @param directory - the campaign's evidence directory or its pass-1 directory

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run

 @param evidencePrefix - name prefix of the campaigns' directories

 @param inspect - asks systemd about the controls unit

 @returns one entry per obligation; one `invalid` entry for an unreadable manifest, an incomplete obligation, or a
 directory whose campaign finished (it has `exit.json`) without a manifest; none for a directory that deferred nothing or
 that has not run yet

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
   What the directory's manifest holds.
   */
  const read = await manifested(directory);
  if (read.kind === 'invalid')
    return [{
      directory,
      state: 'invalid',
      reason: read.reason,
    }];
  if (read.kind === 'absent')
    return (await exists(join(
      directory,
      'exit.json',
    )))
      ? [{
        directory,
        state: 'invalid',
        reason: `${directory} holds a finished campaign (exit.json) but no manifest, so what it deferred cannot be known`,
      }]
      : [];
  return await Promise.all(read.list
    .map(function lookUpEach(obligation) {
      return lookUp({
        obligation,
        directory,
        evidenceRoot,
        evidencePrefix,
        inspect,
      });
    }));
}

/**
 Validate the obligations of one campaign.

 A two-pass campaign's own manifest, once pass 2 wrote it, carries forward the pass-1 obligations that are still
 outstanding, so it speaks for the whole campaign; before that (and when pass 2 never ran) the pass-1 manifest is read.

 @param evidence - the campaign's evidence directory

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run

 @param evidencePrefix - name prefix of the campaigns' directories

 @param inspect - asks systemd about the controls unit; defaults to `systemctl --user show`

 @returns one entry per outstanding obligation, none for a campaign that deferred nothing

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
  /**
   Whether the campaign's own directory speaks for it: it has a manifest, or its campaign finished.
   */
  const speaks = ((await manifested(evidence)).kind !== 'absent')
    || (await exists(join(
      evidence,
      'exit.json',
    )));
  return await validateDirectory({
    directory: speaks ? evidence : join(
      evidence,
      passOneDirectoryName,
    ),
    evidenceRoot,
    evidencePrefix,
    inspect,
  });
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
 Fail when a campaign's obligations are unreadable, lost, or unmet because the controls unit ended without a matching record.

 Pending is not a failure: the unit still runs, and its log says where the proof will be.

 @param validations - the results for the campaign's directories

 @param expected - the obligations the running process itself decided to keep, each of which the manifests must still hold

 @throws NativeVerificationError naming each directory that is invalid or unvalidated, and each expected obligation the
 manifests lost, so a campaign whose controls failed can never pass

 @example
 ```ts
 assertNotUnmet({ validations: await validatePlanted({ evidence, evidenceRoot, evidencePrefix }), expected: [] });
 ```
 */
export function assertNotUnmet({
  validations,
  expected = [],
}: {
  readonly validations: readonly Validation[];
  readonly expected?: readonly PlantedObligation[];
}): void {
  /**
   Results that are invalid or unvalidated.
   */
  const failed = validations.filter(function isFailed(validation) {
    return (validation.state === 'invalid') || (validation.state === 'unvalidated');
  });
  /**
   Expected obligations that no validation accounts for: the manifest that held them was lost or rewritten.
   */
  const lost = expected.filter(function isLost(obligation) {
    return !validations.some(function accounts(validation) {
      return (validation.state !== 'invalid') && (obligationIdentity(validation.obligation) === obligationIdentity(obligation));
    });
  });
  if ((failed.length > 0) || (lost.length > 0))
    throw new NativeVerificationError(`The mutants of this campaign were scored while a controls unit was to prove the planted controls, and that is not established. ${failed.map(function describe(validation) {
      return describeValidation(validation);
    })
      .join(' ')}${lost.map(function describeLost(obligation) {
      return ` The manifest no longer holds the obligation on unit ${obligation.unit} (log ${obligation.log}).`;
    })
        .join('')} The results are not validated; inspect the controls unit's log, then start this campaign again (it resumes) once a record exists.`);
}
