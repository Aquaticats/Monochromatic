/**
 How a campaign that deferred its planted controls to a controls unit is settled.

 A campaign started by `native:mutation:parallel` skips the controls while another unit runs them, so its mutant results
 hold only once that unit leaves a matching record. The whole obligation is kept in `planted-controls-status.json`
 (`native-planted-controls-status-file.ts`): the image, the limits the campaign ran under, the digest and names of the
 controls and the identity of the runner and judge it deferred under, and the controls unit with its systemd invocation.
 Settling matches records against those saved keys, never against the settling process's own table or code, and moves the
 status to `validated` (with the record, which later settlements check again instead of trusting the word),
 `unvalidated` (the invocation is gone and left no record) or leaves it `pending` while the invocation runs. A status that
 cannot be read, or that disagrees with the manifest's `plantedControlsDeferredTo`, is invalid, and invalid fails closed.
 */
import { join } from 'node:path';

import { passOneDirectoryName } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/campaign-passes.ts';

import { readGateRecord } from './native-gate-proof.ts';
import {
  passingPlantedControls,
  provesControls,
} from './native-planted-controls-proof.ts';
import {
  type PlantedObligation,
  type PlantedStatus,
  readPlantedStatus,
  writePlantedStatus,
} from './native-planted-controls-status-file.ts';
import {
  inspectUnit,
  isInvocationRunning,
  type UnitState,
} from './native-planted-controls-unit.ts';

/**
 How a directory's planted-controls status stands after settling.
 */
export type SettledDirectory =
  | {
    readonly directory: string;
    readonly state: 'validated' | 'pending' | 'unvalidated' | 'retired';
    readonly status: PlantedStatus
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
 Settle one directory's planted-controls status against the records and the unit that exist now.

 @param directory - the campaign's evidence directory or its pass-1 directory

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run

 @param evidencePrefix - name prefix of the campaigns' directories

 @param inspect - asks systemd about the controls unit

 @returns the settled directory, none when it deferred nothing

 @example
 ```ts
 await settleDirectory({ directory, evidenceRoot, evidencePrefix: 'native-mutation-', inspect: inspectUnit });
 ```
 */
async function settleDirectory({
  directory,
  evidenceRoot,
  evidencePrefix,
  inspect,
}: {
  readonly directory: string;
  readonly evidenceRoot: string;
  readonly evidencePrefix: string;
  readonly inspect: (unit: string) => Promise<UnitState>;
}): Promise<readonly SettledDirectory[]> {
  /**
   The status file's state.
   */
  const read = await readPlantedStatus(directory);
  /**
   The campaign's manifest as the only element, or none; it names the unit the campaign deferred to.
   */
  const [manifest] = await readGateRecord(join(
    directory,
    'manifest.json',
  ));
  /**
   The unit the manifest says the campaign deferred to, as the only element, or none.
   */
  const manifestUnit = (typeof manifest?.plantedControlsDeferredTo) === 'string'
    ? [manifest?.plantedControlsDeferredTo as string]
    : [];
  if (read.kind === 'invalid')
    return [{
      directory,
      state: 'invalid',
      reason: read.reason,
    }];
  if (read.kind === 'absent')
    return manifestUnit.length === 0
      ? []
      : [{
        directory,
        state: 'invalid',
        reason: `the manifest records a deferral to ${String(manifestUnit[0])} but ${directory} has no status file`,
      }];
  /**
   The status the file holds.
   */
  const { status } = read;
  if (status.status === 'retired')
    return manifestUnit.length === 0
      ? [{
        directory,
        state: 'retired',
        status,
      }]
      : [{
        directory,
        state: 'invalid',
        reason: `the manifest records a deferral to ${String(manifestUnit[0])} but the status is retired`,
      }];
  if ((manifest !== undefined) && (manifestUnit[0] !== status.unit))
    return [{
      directory,
      state: 'invalid',
      reason: `the status defers to ${status.unit} but the manifest says ${manifestUnit[0] ?? 'nothing'}`,
    }];
  /**
   The saved record, as the only element, when it still proves the obligation: the word `validated` is not trusted.
   */
  const saved = (status.status === 'validated') && (status.record !== undefined)
    && await provesControls({
      file: status.record,
      image: status.image,
      limits: status.limits,
      key: status.controls,
      harness: status.harness,
    })
    ? [status.record]
    : [];
  /**
   A proving record found now, as the only element, or none.
   */
  const found = saved.length > 0 ? saved : await proofOf({
    obligation: status,
    evidenceRoot,
    evidencePrefix,
  });
  /**
   Whether the controls unit's invocation still runs; not asked when a record already proves the obligation.
   */
  const running = found.length > 0 ? false : isInvocationRunning({
    state: await inspect(status.unit),
    invocation: status.invocation,
  });
  /**
   The proving record as the only element, or none; the unit may have published it between the lookup above and the
   answer that it ended, so one more look is taken when it is seen to have ended.
   */
  const record = (found.length > 0) || running ? found : await proofOf({
    obligation: status,
    evidenceRoot,
    evidencePrefix,
  });
  /**
   The state this settlement reaches.
   */
  const state = record.length > 0 ? 'validated' : (running ? 'pending' : 'unvalidated');
  /**
   The status as it stands now.
   */
  const settled: PlantedStatus = {
    ...status,
    status: state,
    ...(record[0] === undefined ? {} : { record: record[0] }),
  };
  if ((settled.status !== status.status) || (record[0] !== status.record))
    await writePlantedStatus({
      evidence: directory,
      status: settled,
    });
  return [{
    directory,
    state,
    status: settled,
  }];
}

/**
 Settle the planted-controls status of one campaign (and of its pass-1 directory).

 @param evidence - the campaign's evidence directory

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run

 @param evidencePrefix - name prefix of the campaigns' directories

 @param inspect - asks systemd about the controls unit; defaults to `systemctl --user show`

 @returns the settled directories, none when the campaign deferred nothing

 @example
 ```ts
 await settlePlantedStatus({ evidence, evidenceRoot, evidencePrefix: 'native-mutation-' });
 ```
 */
export async function settlePlantedStatus({
  evidence,
  evidenceRoot,
  evidencePrefix,
  inspect = inspectUnit,
}: {
  readonly evidence: string;
  readonly evidenceRoot: string;
  readonly evidencePrefix: string;
  readonly inspect?: (unit: string) => Promise<UnitState>;
}): Promise<readonly SettledDirectory[]> {
  return (await Promise.all([
    evidence,
    join(
      evidence,
      passOneDirectoryName,
    ),
  ].map(function settleIn(directory) {
    return settleDirectory({
      directory,
      evidenceRoot,
      evidencePrefix,
      inspect,
    });
  }))).flat();
}
