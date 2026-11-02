/**
 What a campaign that deferred its planted controls to a controls unit still owes, and how it is settled.

 A campaign started by `native:mutation:parallel` skips the controls while another unit runs them, so its mutant results
 hold only once that unit leaves a matching record. The deferral is written beside the campaign's evidence as
 `planted-controls-status.json` (`pending`), and settled to `validated` when a matching record exists, or to `unvalidated`
 when the controls unit is gone and left none. Settling runs when the campaign finishes and again on request
 (`native:mutation:reconcile`), so results are never mistaken for validated ones by what a log once said.
 */
import { join } from 'node:path';

import {
  passOneDirectoryName,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/campaign-passes.ts';
import { runCaptured } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/run-captured.ts';

import {
  containerLimitsOf,
  readGateRecord,
} from './native-gate-proof.ts';
import { passingPlantedControls } from './native-planted-controls-proof.ts';
import { writeFileAtomically } from './native-planted-controls-record.ts';

/**
 Name of the status file inside a campaign's evidence directory (or its pass-1 directory).
 */
export const plantedStatusName = 'planted-controls-status.json';

/**
 The state of a deferred campaign's planted controls.
 */
export type PlantedStatus = {
  /**
   `pending` while the controls unit may still leave a record, `validated` once a matching record exists, and
   `unvalidated` when the unit is gone and left none.
   */
  readonly status: 'pending' | 'validated' | 'unvalidated';
  readonly image: string;
  readonly unit: string;
  /**
   The Podman arguments of the campaign, whose memory and process limit a record must have been made under or above.
   */
  readonly limits: readonly string[];
  /**
   The record that validated the campaign.
   */
  readonly record?: string;
};

/**
 Tell whether a transient unit is still running.

 @param unit - unit name without the `.service` suffix

 @returns true while systemd reports the unit active, activating or deactivating; false when it is inactive, failed or
 unknown (a `--collect` unit that ended is unloaded, so it is unknown)

 @example
 ```ts
 await isUnitRunning('cli-git-mutpar-w5-planted-aaaaaaaaaaaa');
 ```
 */
export async function isUnitRunning(unit: string): Promise<boolean> {
  /**
   What `systemctl show` printed.
   */
  const shown = await runCaptured({
    program: 'systemctl',
    args: [
      '--user',
      'show',
      '--property=ActiveState',
      '--value',
      unit,
    ],
    capture: true,
  });
  return [
    'active',
    'activating',
    'deactivating'
  ].includes(shown.stdout
    .trim());
}

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
 Check that a value is one of the status names.

 @param value - parsed JSON

 @returns true for `pending`, `validated` and `unvalidated`

 @example
 ```ts
 isStatusName('pending');
 ```
 */
function isStatusName(value: unknown): value is PlantedStatus['status'] {
  return (value === 'pending') || (value === 'validated')
    || (value === 'unvalidated');
}

/**
 Read a status file.

 @param evidence - a directory that may hold one

 @returns the status as the only element, or no element when the file is absent or not a usable status

 @example
 ```ts
 await readPlantedStatus(evidence);
 ```
 */
async function readPlantedStatus(evidence: string): Promise<readonly PlantedStatus[]> {
  /**
   The parsed file as the only element, or none when it is absent, torn or not an object.
   */
  const [record] = await readGateRecord(join(
    evidence,
    plantedStatusName,
  ));
  if (record === undefined)
    return [];
  if ((!isStatusName(record.status)) || ((typeof record.image) !== 'string')
    || ((typeof record.unit) !== 'string')
    || (!Array.isArray(record.limits)))
    return [];
  return [{
    status: record.status,
    image: record.image as string,
    unit: record.unit as string,
    limits: record.limits
      .filter(function isText(entry: unknown): entry is string {
        return (typeof entry) === 'string';
      }),
  }];
}

/**
 Settle the planted-controls status of one campaign (and of its pass-1 directory) against the records that exist now.

 @param evidence - the campaign's evidence directory

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run

 @param evidencePrefix - name prefix of the campaigns' directories

 @param unitRunning - tells whether the controls unit still runs; defaults to systemd's answer

 @returns the settled statuses, none when the campaign deferred nothing

 @example
 ```ts
 await settlePlantedStatus({ evidence, evidenceRoot, evidencePrefix: 'native-mutation-' });
 ```
 */
export async function settlePlantedStatus({
  evidence,
  evidenceRoot,
  evidencePrefix,
  unitRunning = isUnitRunning,
}: {
  readonly evidence: string;
  readonly evidenceRoot: string;
  readonly evidencePrefix: string;
  readonly unitRunning?: (unit: string) => Promise<boolean>;
}): Promise<readonly PlantedStatus[]> {
  return (await Promise.all([
    evidence,
    join(
      evidence,
      passOneDirectoryName,
    ),
  ].map(async function settleIn(directory): Promise<readonly PlantedStatus[]> {
    /**
     The status this directory holds, as the only element, or none.
     */
    const [status] = await readPlantedStatus(directory);
    if ((status === undefined) || (status.status === 'validated'))
      return status === undefined ? [] : [status];
    /**
     The limits the deferring campaign ran with, as the only element, or none when they are unreadable.
     */
    const [limits] = containerLimitsOf(status.limits);
    /**
     A record that proves the controls for the image at those limits, as the only element, or none.
     */
    const [record] = limits === undefined ? [] : await passingPlantedControls({
      evidenceRoot,
      evidencePrefix,
      image: status.image,
      limits,
    });
    /**
     The status as it stands now.
     */
    const settled: PlantedStatus = record === undefined
      ? {
        ...status,
        status: (await unitRunning(status.unit)) ? 'pending' : 'unvalidated',
      }
      : {
        ...status,
        status: 'validated',
        record,
      };
    if (settled.status !== status.status)
      await writePlantedStatus({
        evidence: directory,
        status: settled,
      });
    return [settled];
  }))).flat();
}
