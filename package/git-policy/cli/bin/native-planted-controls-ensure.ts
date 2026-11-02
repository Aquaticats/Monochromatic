/**
 The decision a campaign (or the controls-only unit) takes about the planted controls: skip, defer to a unit, or run and
 record. The record's shape is in `native-planted-controls-record.ts` and its reader in `native-planted-controls-proof.ts`.
 */
import { containerLimitsOf } from './native-gate-proof.ts';
import { plantedHarnessDigest } from './native-planted-controls-harness.ts';
import {
  plantedControlsPassedRecord,
  writePlantedControlsRecord,
} from './native-planted-controls-record.ts';
import { passingPlantedControls } from './native-planted-controls-proof.ts';
import {
  type PlantedControl,
  plantedControls,
  type PlantedControlSetting,
} from './native-planted-controls.ts';

/**
 Print one line of the decision to the console.

 @param message - the line

 @example
 ```ts
 printLine('Planted controls skipped.');
 ```
 */
function printLine(message: string): void {
  console.log(message);
}

/**
 Decide whether a campaign runs the planted controls, and record a pass so that later campaigns need not.

 The controls are skipped, in this order, when `NATIVE_MUTATION_SKIP_PLANTED=1` is set (a person's claim that another
 campaign over this snapshot ran them), when a record proves they passed on this image at no more than this campaign's
 limits, or when a controls unit named by `besideUnit` runs them (no record yet, so this campaign's results hold only
 once that unit ends with one). Otherwise they run, and the record is written only after every control was noticed; a
 failure leaves none.

 A resumed campaign is held to the same rule: mutants scored by an earlier attempt say nothing about a control added or
 a limit lowered since, so a campaign that has scored mutants but has no matching record runs the controls once.

 @param setting - tested image ID, container bounds, scratch and evidence directories

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run

 @param evidencePrefix - name prefix of the campaigns' directories

 @param skipRequested - whether the person set `NATIVE_MUTATION_SKIP_PLANTED=1`

 @param besideUnit - the unit that runs these controls beside this campaign, when `native:mutation:parallel` started one
 and no record proves them yet; this campaign then skips them without proof and says which unit holds it

 @param run - runs the controls and throws when one was not noticed

 @param controls - the controls `run` plants; defaults to the table the campaign runs

 @param log - prints one line of the decision; defaults to the console, and tests collect the lines

 @param harness - digest of the runner and judge; defaults to the digest of their source

 @example
 ```ts
 await ensurePlantedControls({ setting, evidenceRoot, evidencePrefix, skipRequested: false, run: runPlantedControls });
 ```
 */
export async function ensurePlantedControls({
  setting,
  evidenceRoot,
  evidencePrefix,
  skipRequested,
  besideUnit,
  run,
  controls = plantedControls,
  log = printLine,
  harness,
}: {
  readonly setting: PlantedControlSetting;
  readonly evidenceRoot: string;
  readonly evidencePrefix: string;
  readonly skipRequested: boolean;
  readonly besideUnit?: string;
  readonly run: (setting: PlantedControlSetting) => Promise<void>;
  readonly controls?: readonly PlantedControl[];
  readonly log?: (message: string) => void;
  readonly harness?: string;
}): Promise<void> {
  if (skipRequested) {
    log('Planted controls skipped: another campaign over this snapshot ran them.');
    return;
  }
  /**
   Digest of the runner and judge, which a record must name to count and a new record names.
   */
  const harnessDigest = harness ?? await plantedHarnessDigest({});
  /**
   The memory and process limit this campaign's containers run with, as the only element, or none.
   */
  const [limits] = containerLimitsOf(setting.limits);
  if (limits !== undefined) {
    /**
     Path of a record that proves the controls passed on this image, as the only element, or none.
     */
    const [proof] = await passingPlantedControls({
      evidenceRoot,
      evidencePrefix,
      image: setting.base,
      limits,
      controls,
      harness: harnessDigest,
    });
    if (proof !== undefined) {
      log(`Planted controls skipped, because ${proof} proves they already passed on this image.`);
      return;
    }
    if (besideUnit !== undefined) {
      log(`Planted controls skipped here, without proof: unit ${besideUnit} runs them for image ${setting.base} beside this campaign and has not recorded a pass. The results of this campaign hold only once that unit ends with a planted-controls-passed.json for this image (its log ends with the record path, or with the control that was not noticed).`);
      return;
    }
    log('Planted controls: run, because no campaign evidence proves they already passed on this image.');
  }
  await run(setting);
  if (limits === undefined) {
    console.error('The planted controls passed, but their container arguments hold no readable --memory and --pids-limit, so no record was written.');
    return;
  }
  try {
    log(`Planted controls passed; recorded in ${await writePlantedControlsRecord({
      evidence: setting.evidence,
      record: plantedControlsPassedRecord({
        image: setting.base,
        limits: setting.limits,
        controls,
        harness: harnessDigest,
      }),
    })}.`);
  }
  catch (error) {
    // The record only saves a later campaign time; failing to write it must not discard a pass that took minutes.
    console.error(`The planted controls passed, but their record could not be written, so a later campaign runs them again: ${String(error)}`);
  }
}
