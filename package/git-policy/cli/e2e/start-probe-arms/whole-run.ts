/**
 The whole of the start probe's guest run, and the arm-by-arm pieces every step of it shares: build the handler in the
 guest unless this run was told the guest holds it, run one arm, run every arm in the order the design reads them, and
 create, prepare, build, run and destroy in the order a run owes.

 The driver keeps the command line, the steps it names and the pulled logs it proves; this module is what those steps do
 to the guest, and it lives beside the arms it drives.
 */

import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import { recipe, } from '../../bin/mvm-guest-host-command.ts';
import { runWholeGuestRun, } from '../../bin/mvm-guest-whole-run.ts';
import { buildInGuest, } from './crate-build.ts';
import {
  probeDatasetArm,
  red as redArm,
} from './fixture-arms.ts';
import { green as greenArm, } from './green-arm.ts';
import {
  m1Key,
  m1ReadWrite,
} from './key-arms.ts';
import { m3Kill, } from './kill-matrix-arm.ts';
import { noConfiguration as noConfigurationArm, } from './no-configuration-arm.ts';
import {
  gateSkip,
  m9Busy,
} from './refusal-arms.ts';
import { m7Scan, } from './scan-arm.ts';
import { m2Controls, } from './table-window-arm.ts';
import {
  timings,
  timingsBootFirst,
} from './timing-arms.ts';

/**
 One arm of the run: what it needs is the guest and where its logs go.
 */
export type Arm = (armArguments: { readonly chosen: GuestFacts; }) => Promise<void>;

/**
 Build the handler in the guest unless this run was told the guest holds it, then run one arm.

 @param chosen - the run's guest and log folder

 @param arm - the arm to run after the build

 @param noBuild - whether this run was told the guest already holds the build

 @throws Error naming what refused

 @example
 ```ts
 await armed({ chosen, arm: greenArm, noBuild: false });
 ```
 */
export async function armed({
  chosen,
  arm,
  noBuild,
}: {
  readonly chosen: GuestFacts;
  readonly arm: Arm;
  readonly noBuild: boolean;
},): Promise<void> {
  if (!noBuild)
    await buildInGuest({ chosen, },);
  await arm({ chosen, },);
}

/**
 Every arm the whole run performs, in the order the design reads them.

 @param chosen - the run's guest and log folder

 @throws Error naming the arm that refused

 @example
 ```ts
 await everyArm({ chosen });
 ```
 */
export async function everyArm({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await probeDatasetArm({ chosen, },);
  await greenArm({ chosen, },);
  await noConfigurationArm({ chosen, },);
  await m1Key({ chosen, },);
  await m1ReadWrite({ chosen, },);
  await m2Controls({ chosen, },);
  await m3Kill({ chosen, },);
  await m7Scan({ chosen, },);
  await m9Busy({ chosen, },);
  await gateSkip({ chosen, },);
}

/**
 Make the guest this run acts on.

 @param chosen - the run's guest and log folder

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await createTheGuest({ chosen });
 ```
 */
async function createTheGuest({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await recipe({
    chosen,
    step: 'create',
  },);
}

/**
 Install OpenZFS 2.4.4 and the Rust toolchain in the guest.

 @param chosen - the run's guest and log folder

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await prepareTheGuest({ chosen });
 ```
 */
async function prepareTheGuest({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await recipe({
    chosen,
    step: 'prepare',
  },);
}

/**
 Destroy the guest.

 @param chosen - the run's guest and log folder

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await destroyTheGuest({ chosen });
 ```
 */
async function destroyTheGuest({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await recipe({
    chosen,
    step: 'destroy',
  },);
}

/**
 Create, prepare, build, run every arm, and destroy, destroying even when a step refused.

 @param chosen - the run's guest and log folder

 @param keepGuest - whether the run keeps the guest for a person to inspect

 @throws Error naming the step that refused, after the guest it created is gone

 @example
 ```ts
 await all({ chosen, keepGuest: false });
 ```
 */
export async function all({
  chosen,
  keepGuest,
}: {
  readonly chosen: GuestFacts;
  readonly keepGuest: boolean;
},): Promise<void> {
  await runWholeGuestRun({
    label: 'git-policy probe guest',
    chosen,
    keepGuest,
    steps: {
      create: createTheGuest,
      prepare: prepareTheGuest,
      work: workStep,
      destroy: destroyTheGuest,
    },
  },);
}

/**
 Build the handler in the guest and run every arm of the whole run.

 @param chosen - the run's guest and log folder

 @throws Error naming the arm that refused

 @example
 ```ts
 await workStep({ chosen });
 ```
 */
async function workStep({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await buildInGuest({ chosen, },);
  await everyArm({ chosen, },);
}

/**
 What one step of this driver's command line does to the guest.
 */
type StepRun = {
  /**
   The run's guest and log folder.
   */
  readonly chosen: GuestFacts;
  /**
   Whether this run was told the guest already holds the build.
   */
  readonly noBuild: boolean;
  /**
   Whether the run keeps the guest for a person to inspect.
   */
  readonly keepGuest: boolean;
};

/**
 One step of this driver's command line.
 */
type Step = (run: StepRun) => Promise<void>;

/**
 One arm, as a step of this driver's command line.

 @param arm - the arm the step runs

 @returns a step that builds the handler unless this run was told not to, then runs that arm

 @example
 ```ts
 const step = armStep(greenArm);
 ```
 */
function armStep(arm: Arm,): Step {
  return async function runTheArm({
    chosen,
    noBuild,
  }: StepRun) {
    await armed({
      chosen,
      arm,
      noBuild,
    },);
  };
}

/**
 Make the guest this run acts on, as a step of the driver's command line.

 @param chosen - the run's guest and log folder

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await createStep({ chosen });
 ```
 */
async function createStep({ chosen }: StepRun,): Promise<void> {
  await createTheGuest({ chosen, },);
}

/**
 Install the guest's tools, as a step of the driver's command line.

 @param chosen - the run's guest and log folder

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await prepareStep({ chosen });
 ```
 */
async function prepareStep({ chosen }: StepRun,): Promise<void> {
  await prepareTheGuest({ chosen, },);
}

/**
 Run the probe's dataset arm, which makes the fixture a probe needs.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status

 @example
 ```ts
 await probeDatasetStep({ chosen });
 ```
 */
async function probeDatasetStep({ chosen }: StepRun,): Promise<void> {
  await probeDatasetArm({ chosen, },);
}

/**
 Run the raised-deadline timing arm, which is the one control the shipped deadline refuses.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status

 @example
 ```ts
 await timingsBootFirstStep({ chosen });
 ```
 */
async function timingsBootFirstStep({ chosen }: StepRun,): Promise<void> {
  await timingsBootFirst({ chosen, },);
}

/**
 Take the guest away, as a step of the driver's command line.

 @param chosen - the run's guest and log folder

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await destroyStep({ chosen });
 ```
 */
async function destroyStep({ chosen }: StepRun,): Promise<void> {
  await destroyTheGuest({ chosen, },);
}

/**
 Create, prepare, build, run every arm and destroy, as a step of the driver's command line.

 @param chosen - the run's guest and log folder

 @param keepGuest - whether the run keeps the guest for a person to inspect

 @throws Error naming the step that refused, after the guest it created is gone

 @example
 ```ts
 await allStep({ chosen, keepGuest: false });
 ```
 */
async function allStep({
  chosen,
  keepGuest,
}: StepRun,): Promise<void> {
  await all({
    chosen,
    keepGuest,
  },);
}

/**
 Every step this driver knows, by the name its command line gives it.

 @returns the steps, in the order the design reads them

 @example
 ```ts
 const steps = stepsOf();
 ```
 */
export function stepsOf(): ReadonlyMap<string, Step> {
  return new Map([
    [
      'create',
      createStep,
    ],
    [
      'prepare',
      prepareStep,
    ],
    [
      'red',
      armStep(redArm),
    ],
    [
      'probe-dataset',
      probeDatasetStep,
    ],
    [
      'green',
      armStep(greenArm),
    ],
    [
      'no-configuration',
      armStep(noConfigurationArm),
    ],
    [
      'm1-key',
      armStep(m1Key),
    ],
    [
      'm1-read-write',
      armStep(m1ReadWrite),
    ],
    [
      'm2-controls',
      armStep(m2Controls),
    ],
    [
      'm3-kill',
      armStep(m3Kill),
    ],
    [
      'm7-scan',
      armStep(m7Scan),
    ],
    [
      'm9-busy',
      armStep(m9Busy),
    ],
    [
      'gate-skip',
      armStep(gateSkip),
    ],
    [
      'timings',
      armStep(timings),
    ],
    [
      'timings-boot-first',
      timingsBootFirstStep,
    ],
    [
      'destroy',
      destroyStep,
    ],
    [
      'all',
      allStep,
    ],
  ],);
}
