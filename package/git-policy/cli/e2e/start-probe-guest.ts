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

import {
  requireMarkers,
} from '../bin/mvm-guest-arm-log.ts';
import {
  type RunArguments,
  runArguments,
} from '../bin/mvm-guest-options.ts';
import {
  readLog,
} from '../bin/mvm-guest-step-command.ts';


import {
  guestBackend,
  refuseAStepWhoseArmsCannotRun,
  STEPS_THAT_RUN_NO_ARM,
} from '../bin/mvm-guest-backend.ts';
import { greenLog, } from './start-probe-arms/green-arm.ts';
import { noConfigurationLog, } from './start-probe-arms/no-configuration-arm.ts';
import {
  greenMarkers,
  noConfigurationMarkers,
} from './start-probe-arms/probe-fixture.ts';
import { stepsOf, } from './start-probe-arms/whole-run.ts';

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
      text: pulled,
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
      text: pulled,
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
  await step({
    chosen,
    noBuild: chosen.noBuild,
    keepGuest: chosen.keepGuest,
  },);
  await proveTheArmRan(chosen,);
  console.log(`git-policy probe guest: step '${chosen.step}' finished; logs in ${chosen.logDir}`);
}

/**
 The steps this driver's command line knows.
 */
const steps = stepsOf();

/**
 The words the command line gave, after the script's path.
 */
const words: readonly string[] = process.argv
  .slice(2);
/**
 The step and the values the command line gave it.
 */
const chosen: Options = options({ argumentsRead: runArguments({ argv: words, },), },);
// The steps that run an arm in the guest refuse on a backend this package's arm scripts cannot reach, before this run
// makes anything. The three that only make, prepare and take away a guest run on either backend.
refuseAStepWhoseArmsCannotRun({
  backend: guestBackend(),
  step: chosen.step,
  stepsThatRunNoArm: STEPS_THAT_RUN_NO_ARM,
});
await runTheStep(chosen,);
