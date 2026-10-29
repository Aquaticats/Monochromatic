import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { runIntroducedDefectProbe, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import { singleRegionTally, } from './probe-single-tally.ts';
import type {
  PriorIssueList,
  SensitivityArm,
} from './probe-sensitivity-arms.ts';
import { SOURCE_TEXT, } from './probe-sensitivity-input.ts';
import {
  sensitivityArmLines,
  sensitivityNotes,
  sensitivityOpening,
} from './probe-sensitivity-print.ts';
import {
  RUN_MODELS,
  RUN_PER_CALL_TIMEOUT_MS,
} from '@monochromatic-dev/translation-repair-run/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Probe sensitivity run
// Asks the roster about every arm in order and prints what it said, which is
// the whole of the probe sensitivity runner once the process has handed it a
// client.

/**
 Runs one arm and prints what the probe said about it.

 @param arm - region, list, issue, and framing to send

 @param client - client shared by every arm

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @example
 ```ts
 await probeOne({ arm: SENSITIVITY_ARMS[0], client, stageSettings, },);
 ```
 */
async function probeOne(
  {
    arm,
    client,
    stageSettings,
  }: {
    readonly arm: SensitivityArm;
    readonly client: ModelClient;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   Report for this single region.
   */
  const report = await runIntroducedDefectProbe({
    client,
    proberModelIds: RUN_MODELS.checkerModelIds,
    sourceText: SOURCE_TEXT,
    baselineText: arm.baselineText,
    regions: [arm.region,],
    issues: arm.issues,
    identityContext: '',
    editKind: arm.editKind,
    disclosure: arm.disclosure,
    signal: new AbortController().signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l: tagged({ tag: 'probe-sensitivity', },),
    stageSettings,
  },);

  for (
    const line of sensitivityArmLines({
      arm,
      report,
      tally: singleRegionTally({ report, },),
    },)
  )
    console.log(line,);
}

/**
 Runs every arm in order and prints how to read the lines.

 @param client - one client for the whole instrument, counted on the run-wide
 seat tally

 @param arms - arms to ask about, in the order they run

 @param production - list production sends, which the opening and the closing
 notes name

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @example
 ```ts
 await runSensitivity({ client, arms: SENSITIVITY_ARMS, production: PRODUCTION_LIST, stageSettings, },);
 ```
 */
export async function runSensitivity(
  {
    client,
    arms,
    production,
    stageSettings,
  }: {
    readonly client: ModelClient;
    readonly arms: readonly SensitivityArm[];
    readonly production: PriorIssueList;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  console.log(
    sensitivityOpening({
      production,
      armCount: arms.length,
    },),
  );

  // Sequential so this never competes with a running corpus pass for the
  // per-model stream slots.
  /* oxlint-disable no-await-in-loop -- sequential by design, see comment */
  for (const arm of arms)
    await probeOne({
      arm,
      client,
      stageSettings,
    },);
  /* oxlint-enable no-await-in-loop */

  for (const note of sensitivityNotes({ production, },))
    console.log(note,);
}

//endregion Probe sensitivity run
