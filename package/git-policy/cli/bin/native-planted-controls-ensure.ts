/**
 The decision a campaign (or the controls-only unit) takes about the planted controls: skip, defer to a unit, or run and
 record. The record's shape is in `native-planted-controls-record.ts` and its reader in `native-planted-controls-proof.ts`.
 */
import { containerLimitsOf } from './native-gate-proof.ts';
import { NativeVerificationError } from './native-verification-process.ts';
import { plantedHarnessId } from './native-planted-controls-harness.ts';
import {
  controlsKeyOf,
  plantedControlsPassedRecord,
  writePlantedControlsRecord,
} from './native-planted-controls-record.ts';
import { passingPlantedControls } from './native-planted-controls-proof.ts';
import type { PlantedObligation } from './native-planted-controls-obligation.ts';
import {
  inspectUnit,
  isInvocationRunning,
  type UnitState,
} from './native-planted-controls-unit.ts';
import type { PlantedUnit } from './native-planted-controls-units.ts';
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
 What {@link ensurePlantedControls} decided and did.
 */
export type PlantedDecision =
  | { readonly kind: 'requested' }
    | {
      readonly kind: 'proved';
      readonly record: string
    }
    | {
      readonly kind: 'deferred';
      readonly obligation: PlantedObligation
    }
    | {
      readonly kind: 'ran';
      readonly record?: string
    };

/**
 The arguments of {@link ensurePlantedControls}.
 */
export type PlantedControlsCall = {
  readonly setting: PlantedControlSetting;
  readonly evidenceRoot: string;
  readonly evidencePrefix: string;
  readonly skipRequested: boolean;
  readonly beside?: PlantedUnit;
  readonly run: (setting: PlantedControlSetting) => Promise<void>;
  readonly controls?: readonly PlantedControl[];
  readonly log?: (message: string) => void;
  readonly harness?: string;
  readonly inspect?: (unit: string) => Promise<UnitState>;
  readonly requirePublished?: boolean;
};

/**
 Decide whether a campaign runs the planted controls, and record a pass so that later campaigns need not.

 The controls are skipped, in this order, when `NATIVE_MUTATION_SKIP_PLANTED=1` is set (a person's claim that another
 campaign over this snapshot ran them), when a record proves they passed on this image at no more than this campaign's
 limits, or when a controls unit named by `beside` still runs them. A deferral is not a pass: the decision carries the
 obligation, which the campaign's manifest keeps (see `native-planted-controls-obligation.ts`), and the campaign's
 results hold only once a matching record exists. A named unit that is no longer running and left no record is not
 waited for: the campaign runs the controls itself. Otherwise they run, and the record is written only after every
 control was noticed; a failure leaves none.

 A resumed campaign is held to the same rule: mutants scored by an earlier attempt say nothing about a control added or
 a limit lowered since, so a campaign that has scored mutants but has no matching record runs the controls once. An
 attempt that does not defer writes a manifest without an obligation, which ends an earlier attempt's obligation.

 @param call - tested image ID, container bounds, scratch and evidence directories (`setting`), the evidence root and
 prefix the records are looked for under, whether the person asked to skip, the unit that runs the controls beside this
 campaign when `native:mutation:parallel` started one (`beside`), the runner of the controls (`run`), and optionally the
 controls, a log printer, the runner and judge identity, the unit inspector, and `requirePublished`, which makes the
 controls-only producer throw when it could not publish a record

 @returns what was decided, with the record's path when one proves or was written, or the obligation when deferred

 @throws NativeVerificationError when `requirePublished` is set and no record was published

 @example
 ```ts
 await ensurePlantedControls({ setting, evidenceRoot, evidencePrefix, skipRequested: false, run: runPlantedControls });
 ```
 */
export async function ensurePlantedControls(call: PlantedControlsCall): Promise<PlantedDecision> {
  /**
   The call's fields, with the defaults applied.
   */
  const {
    setting,
    evidenceRoot,
    evidencePrefix,
    skipRequested,
    beside,
    run,
    controls = plantedControls,
    log = printLine,
    harness,
    inspect = inspectUnit,
    requirePublished = false,
  } = call;
  if (skipRequested) {
    log('Planted controls skipped: another campaign over this snapshot ran them.');
    return { kind: 'requested' };
  }
  /**
   Identity of the runner and judge, which a record must name to count and a new record names.
   */
  const harnessId = harness ?? plantedHarnessId;
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
      key: controlsKeyOf(controls),
      harness: harnessId,
    });
    if (proof !== undefined) {
      log(`Planted controls skipped, because ${proof} proves they already passed on this image.`);
      return {
        kind: 'proved',
        record: proof,
      };
    }
    if (beside !== undefined) {
      /**
       What systemd reports of the controls unit now; its invocation is what the obligation is tied to.
       */
      const unitState = await inspect(beside.unit);
      if (isInvocationRunning({
        state: unitState,
        invocation: unitState.invocation,
      })) {
        log(`Planted controls skipped here, without proof: unit ${beside.unit} (invocation ${unitState.invocation}) runs them for image ${setting.base} beside this campaign and has not recorded a pass. The results of this campaign hold only once that unit leaves a planted-controls-passed.json for this image; its log ${beside.log} ends with the record path or with the control that was not noticed. This campaign's manifest keeps the obligation, and finalization and native:mutation:reconcile look for the record and fail when that invocation ended without one.`);
        return {
          kind: 'deferred',
          obligation: {
            image: setting.base,
            unit: beside.unit,
            log: beside.log,
            invocation: unitState.invocation,
            limits,
            controls: controlsKeyOf(controls),
            harness: harnessId,
          },
        };
      }
      log(`Planted controls: unit ${beside.unit} is no longer running and left no record for this image, so this campaign runs them itself.`);
    }
    else
      log('Planted controls: run, because no campaign evidence proves they already passed on this image.');
  }
  await run(setting);
  if (limits === undefined) {
    if (requirePublished)
      throw new NativeVerificationError('The planted controls passed, but their container arguments hold no readable --memory and --pids-limit, so no record could be written.');
    console.error('The planted controls passed, but their container arguments hold no readable --memory and --pids-limit, so no record was written.');
    return { kind: 'ran' };
  }
  try {
    /**
     Path of the record just written.
     */
    const record = await writePlantedControlsRecord({
      evidence: setting.evidence,
      record: plantedControlsPassedRecord({
        image: setting.base,
        limits: setting.limits,
        controls,
        harness: harnessId,
      }),
    });
    log(`Planted controls passed; recorded in ${record}.`);
    return {
      kind: 'ran',
      record,
    };
  }
  catch (error) {
    if (requirePublished)
      throw new NativeVerificationError(`The planted controls passed, but their record could not be written, so the campaigns that wait for it never get it: ${String(error)}`);
    // The record only saves a later campaign time; failing to write it must not discard a pass that took minutes.
    console.error(`The planted controls passed, but their record could not be written, so a later campaign runs them again: ${String(error)}`);
    return { kind: 'ran' };
  }
}
