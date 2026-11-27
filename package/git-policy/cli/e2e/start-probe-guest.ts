#!/usr/bin/env node
/**
 Drive the root start probe's arms in the throwaway `mvm` guest.

 The probe mounts, unshares a mount namespace and reads `/proc/spl/kstat/zfs`, so no control on a person's machine may
 run it. This driver borrows the ZFS binding's guest recipe for the parts that are not the probe's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest, prepares it with
 OpenZFS 2.4.4 and the Rust toolchain, then builds this crate's handler inside it, prepares the probe dataset the design
 names, runs the arms, and destroys the guest.

 Usage: node e2e/start-probe-guest.ts <create|prepare|red|probe-dataset|green|no-configuration|m1-key|m1-read-write|m2-controls|m3-kill|m7-scan|m9-busy|timings|timings-boot-first|destroy|all> [--name <guest>]
          [--log-dir <dir>] [--keep-guest]

 `no-configuration` is the project's answer of 2026-11-22 for the probe: with no configuration file the record passes,
 its source is the design's own name in the pool the mounted checkout dataset names, and a dataset renamed away leaves
 the `ENOENT` record whose line names the two setup commands.

 `red` runs the arm before the probe exists and requires the refusal that says the mode is not one of this build's,
 which is what proves the green run below is a change and not an absence that always passed. `green` requires the
 record's own words: the attach arm passed, the mount the probe made is gone from every namespace afterwards, and a
 filtered `move_mount` fails that step with `EPERM`.

 `m1-key` is M1's unloaded-key control: the fixture's encrypted dataset attaches and passes with its key loaded and
 answers `EACCES` from `fsconfig(create)` with the key unloaded. `m1-read-write` is M1's read-only against read-write
 control: a read-only request for the probe dataset beside a read-write superblock of it answers `EBUSY`, while the
 probe's read-only attachment of that dataset's `ready` snapshot passes, because a snapshot is the documented
 exception.

 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The arms themselves are in `e2e/start-probe-arms/`, one file per group of them.
 */
import { mkdir, } from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import type { GuestRun, } from './guest-host/guest-run.ts';
import { runWithAGuestThatIsDestroyed, } from './guest-host/guest-run-lifecycle.ts';
import { recipe, } from './guest-host/host-side-commands.ts';
import {
  readLog,
  requireMarkers,
} from './guest-host/log-markers.ts';
import {
  type RunArguments,
  runArguments,
} from './guest-host/run-arguments.ts';
import { buildInGuest, } from './start-probe-arms/crate-build.ts';
import {
  probeDatasetArm,
  red as redArm,
} from './start-probe-arms/fixture-arms.ts';
import {
  green as greenArm,
  greenLog,
} from './start-probe-arms/green-arm.ts';
import {
  m1Key,
  m1ReadWrite,
} from './start-probe-arms/key-arms.ts';
import { m3Kill, } from './start-probe-arms/kill-matrix-arm.ts';
import {
  noConfiguration as noConfigurationArm,
  noConfigurationLog,
} from './start-probe-arms/no-configuration-arm.ts';
import {
  greenMarkers,
  noConfigurationMarkers,
} from './start-probe-arms/probe-fixture.ts';
import {
  gateSkip,
  m9Busy,
} from './start-probe-arms/refusal-arms.ts';
import { m7Scan, } from './start-probe-arms/scan-arm.ts';
import { m2Controls, } from './start-probe-arms/table-window-arm.ts';
import {
  timings,
  timingsBootFirst,
} from './start-probe-arms/timing-arms.ts';

/**
 The guest name a run uses when the command line names none.
 */
const defaultGuestName = 'git-policy-start-probe';

/**
 The log folder a run uses when the command line names none, relative to the
 package's own folder.
 */
const defaultLogFolder = 'target/start-probe-guest-logs';

/**
 The mode the run's own log folder is made with: readable and writable by the
 account that runs the driver, and by nobody else.
 */
const logFolderMode = 0o700;

/**
 The step, the guest name, the log directory, whether `all` keeps the guest and
 whether an arm rebuilds.

 The arms run the build they were started with unless `--no-build` says the guest
 already holds it: a build of two release profiles costs most of an hour in the
 guest, and a round that runs one arm again does not need another.
 */
type Options = {
  /**
   The step the command line names.
   */
  readonly step: string;
  /**
   The guest's `mvm` name.
   */
  readonly name: string;
  /**
   The folder this run writes its logs in.
   */
  readonly logDir: string;
  /**
   Whether the guest is kept for a person to inspect.
   */
  readonly keepGuest: boolean;
  /**
   Whether an arm skips the guest's build.
   */
  readonly noBuild: boolean;
};

/**
 The step and the values the command line gave it.

 @param argumentsRead - the words the command line gave, after the script's path

 @returns the step, the guest's name, the log folder and the two switches

 @example
 ```ts
 options({ argumentsRead: runArguments({ argv: ['green', '--no-build'] }) });
 ```
 */
function options({
  argumentsRead,
}: {
  readonly argumentsRead: RunArguments;
},): Options {
  /**
   The guest's name, which the command line may give.
   */
  const named = argumentsRead.values
    .get('--name',);
  /**
   The log folder, which the command line may give.
   */
  const folder = argumentsRead.values
    .get('--log-dir',);
  return {
    step: argumentsRead.step,
    name: named ?? defaultGuestName,
    logDir: folder
      ?? resolve(
        import.meta.dirname,
        '..',
        defaultLogFolder,
      ),
    keepGuest: argumentsRead.flags
      .has('--keep-guest',),
    noBuild: argumentsRead.flags
      .has('--no-build',),
  };
}

/**
 Create the guest, refusing a name another run holds.

 @param chosen - the run's options

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await create(chosen);
 ```
 */
async function create(chosen: GuestRun,): Promise<void> {
  await recipe({
    chosen,
    step: 'create',
  },);
}

/**
 Install OpenZFS 2.4.4 and the Rust toolchain in the guest.

 @param chosen - the run's options

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await prepare(chosen);
 ```
 */
async function prepare(chosen: GuestRun,): Promise<void> {
  await recipe({
    chosen,
    step: 'prepare',
  },);
}

/**
 Destroy the guest.

 @param chosen - the run's options

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await destroy(chosen);
 ```
 */
async function destroy(chosen: GuestRun,): Promise<void> {
  await recipe({
    chosen,
    step: 'destroy',
  },);
}

/**
 One arm of the run: what it needs is the guest and where its logs go.
 */
type Arm = (armArguments: { readonly chosen: GuestRun; }) => Promise<void>;

/**
 Build the handler in the guest unless this run was told the guest holds it, then
 run one arm.

 @param chosen - the run's options

 @param arm - the arm to run after the build

 @throws Error naming what refused

 @example
 ```ts
 await armed({ chosen, arm: greenArm });
 ```
 */
async function armed({
  chosen,
  arm,
}: {
  readonly chosen: Options;
  readonly arm: Arm;
},): Promise<void> {
  if (!chosen.noBuild) await buildInGuest({ chosen, },);
  await arm({ chosen, },);
}

/**
 Every arm the whole run performs, in the order the design reads them.

 @param chosen - the run's options

 @throws Error naming the arm that refused

 @example
 ```ts
 await everyArm(chosen);
 ```
 */
async function everyArm(chosen: Options,): Promise<void> {
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
 Create, prepare, build, run every arm, and destroy, destroying even when a step
 refused.

 @param chosen - the run's options

 @throws Error naming the step that refused, after the guest it created is gone

 @example
 ```ts
 await all(chosen);
 ```
 */
async function all(chosen: Options,): Promise<void> {
  /**
   The guest this run creates, which `create` refuses to make twice.
   */
  function createThisGuest(): Promise<void> {
    return create(chosen,);
  }

  /**
   Everything the guest was created for: its preparation, the handler's build and
   every arm.
   */
  async function prepareBuildAndRunEveryArm(): Promise<void> {
    await prepare(chosen,);
    await buildInGuest({ chosen, },);
    await everyArm(chosen,);
  }

  /**
   The guest's destruction, which happens whatever the work did.
   */
  function destroyThisGuest(): Promise<void> {
    return destroy(chosen,);
  }

  await runWithAGuestThatIsDestroyed({
    name: chosen.name,
    subject: 'git-policy probe guest',
    keepGuest: chosen.keepGuest,
    create: createThisGuest,
    run: prepareBuildAndRunEveryArm,
    destroy: destroyThisGuest,
  },);
}

/**
 The steps this driver knows, and nothing else.
 */
const steps: ReadonlyMap<string, (chosen: Options,) => Promise<void>> = new Map([
  [
    'create',
    create,
  ],
  [
    'prepare',
    prepare,
  ],
  [
    'red',
    async function redStep(chosen: Options,): Promise<void> {
      await armed({
        chosen,
        arm: redArm,
      },);
    },
  ],
  [
    'probe-dataset',
    async function probeDatasetStep(chosen: Options,): Promise<void> {
      await probeDatasetArm({ chosen, },);
    },
  ],
  [
    'green',
    async function greenStep(chosen: Options,): Promise<void> {
      await armed({
        chosen,
        arm: greenArm,
      },);
    },
  ],
  [
    'no-configuration',
    async function noConfigurationStep(chosen: Options,): Promise<void> {
      await armed({
        chosen,
        arm: noConfigurationArm,
      },);
    },
  ],
  [
    'm1-key',
    async function m1KeyStep(chosen: Options,): Promise<void> {
      await armed({
        chosen,
        arm: m1Key,
      },);
    },
  ],
  [
    'm1-read-write',
    async function m1ReadWriteStep(chosen: Options,): Promise<void> {
      await armed({
        chosen,
        arm: m1ReadWrite,
      },);
    },
  ],
  [
    'm2-controls',
    async function m2ControlsStep(chosen: Options,): Promise<void> {
      await armed({
        chosen,
        arm: m2Controls,
      },);
    },
  ],
  [
    'm3-kill',
    async function m3KillStep(chosen: Options,): Promise<void> {
      await armed({
        chosen,
        arm: m3Kill,
      },);
    },
  ],
  [
    'm7-scan',
    async function m7ScanStep(chosen: Options,): Promise<void> {
      await armed({
        chosen,
        arm: m7Scan,
      },);
    },
  ],
  [
    'm9-busy',
    async function m9BusyStep(chosen: Options,): Promise<void> {
      await armed({
        chosen,
        arm: m9Busy,
      },);
    },
  ],
  [
    'gate-skip',
    async function gateSkipStep(chosen: Options,): Promise<void> {
      await armed({
        chosen,
        arm: gateSkip,
      },);
    },
  ],
  [
    'timings',
    async function timingsStep(chosen: Options,): Promise<void> {
      await armed({
        chosen,
        arm: timings,
      },);
    },
  ],
  [
    'timings-boot-first',
    async function timingsBootFirstStep(chosen: Options,): Promise<void> {
      await timingsBootFirst({ chosen, },);
    },
  ],
  [
    'destroy',
    destroy,
  ],
  [
    'all',
    all,
  ],
],);

/**
 Refuse a pulled arm log that proves nothing, which is what a run that only
 returned leaves behind.

 @param chosen - the run's options

 @throws Error naming the marker the log lacks

 @example
 ```ts
 await proveTheArmRan(chosen);
 ```
 */
async function proveTheArmRan(chosen: Options,): Promise<void> {
  if ((chosen.step === 'no-configuration') || (chosen.step === 'all')) {
    /**
     The no-configuration arm's own log, written where this run's logs go.
     */
    const pulled = await readLog({ path: join(
      chosen.logDir,
      noConfigurationLog,
    ), },);
    requireMarkers({
      pulled,
      markers: noConfigurationMarkers,
      subject: 'the no-configuration log',
      because: 'the guest ran the arm without proving it',
    },);
  }
  if ((chosen.step === 'green') || (chosen.step === 'all')) {
    /**
     The green arm's own log, written where this run's logs go.
     */
    const pulled = await readLog({ path: join(
      chosen.logDir,
      greenLog,
    ), },);
    requireMarkers({
      pulled,
      markers: greenMarkers,
      subject: 'the green log',
      because: 'the guest ran the arm without proving it',
    },);
  }
}

/**
 Run the step the command line names, then prove the arms whose logs it wrote.

 @param chosen - the run's options

 @throws Error naming a step this driver does not have, or what the step refused

 @example
 ```ts
 await runTheStep(chosen);
 ```
 */
async function runTheStep(chosen: Options,): Promise<void> {
  /**
   The step the command line named.
   */
  const step = steps.get(chosen.step,);
  if (step === undefined)
    throw new Error(`unknown step '${chosen.step}'; use ${[...steps.keys(),].join(', ',)}`);
  await mkdir(
    chosen.logDir,
    {
      recursive: true,
      mode: logFolderMode,
    },
  );
  await step(chosen,);
  await proveTheArmRan(chosen,);
  console.log(`git-policy probe guest: step '${chosen.step}' finished; logs in ${chosen.logDir}`);
}

/**
 The words the command line gave, after the script's path.
 */
const words: readonly string[] = process.argv
  .slice(2);
/**
 The step and the values the command line gave it.
 */
const chosen: Options = options({ argumentsRead: runArguments({ argv: words, },), },);
await runTheStep(chosen,);
