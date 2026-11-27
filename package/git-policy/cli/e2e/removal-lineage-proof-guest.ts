#!/usr/bin/env node
/**
 Drive plan step 14's guest arm in a throwaway `mvm` guest: the removal's lineage proof, bookmark rule, intent and
 plan against a real pool.

 The proof reads `/dev/zfs`, so no control on a person's machine may run it against a pool it did not build. This driver
 borrows the ZFS binding's guest recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and prepares it
 with OpenZFS 2.4.4, the Rust toolchain and `cargo-nextest`, then builds this crate inside it, runs the arm, and destroys
 the guest.

 The arms, as the plan's steps 14 and 15 ask
 (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, "Step 14" and "Step 15"):
 step 14's:
 - the healthy removal enters, writes its plan and prints the one own snapshot it loses;
 - VM item 11's promote: the proof refuses a promoted clone before the first delete, every protected snapshot and
   bookmark tuple survives, and reverse promotion restores the recorded lineage;
 - VM item 13's bookmarks: a bookmark a live snapshot explains passes, and one whose snapshot went refuses and is named;
 - VM item 12's replacement and grants: the decided grant set carries no lineage-changing permission, and a same-name
   replacement clone refuses on its GUID;

 and step 15's, through the library's writing path, because the removal's command path stays cut until step 16:
 - a healthy removal destroys the clone's own snapshots, records every answer and stops before the head destroy;
 - the look inside the entry refuses a mark that is already there, with nothing written;
 - the first check refuses a mark that appears after the look, with no request sent;
 - the owed measurement's second half: the mark appears after the batch and the head check refuses, with the plan
   keeping what the batch destroyed and the Git registration surviving;
 - a replacement clone whose different origin is marked refuses at the entry's own proof, with no request sent.

 Usage: node e2e/removal-lineage-proof-guest.ts <create|prepare|run|destroy|all> [--name <guest>] [--log-dir <dir>]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `removal-lineage-arm.log`, pulled back into the log directory, and a run passes only when
 it carries every `REMOVAL` and `DESTRUCTION` line below and no `SKIPPED` line, so an arm that returned without running
 fails the run instead of reporting a green suite.
 */
import {
  mkdir,
  rm,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { copyWrapperSnapshot, } from '../bin/native-wrapper-snapshot.ts';
import {
  holdsWordLine,
  missingMarkerRefusal,
  missingMarkers,
} from './guest-driver-arm-log.ts';
import { flagValue, } from './guest-driver-command-line.ts';
import { GuestDriverError, } from './guest-driver-error.ts';
import {
  type GuestFacts,
  runGuestRecipeStep,
} from './guest-driver-recipe-step.ts';
import {
  readStepLog,
  runRequiredStepCommand,
} from './guest-driver-step-command.ts';
import {
  awaitDetachedGuestScript,
  startDetachedGuestScript,
} from './guest-driver-detached-script.ts';
import {
  ARM_LOG,
  EXPECTED_MARKERS,
  SKIP_WORD,
  armScript,
  buildScript,
} from './removal-lineage-proof-guest-arm-scripts.ts';
import {
  type GuestRunStep,
  runWholeGuestRun,
} from './guest-driver-whole-run.ts';

//region Constants

/**
 Milliseconds one host command may take.
 */
const HOST_COMMAND_TIMEOUT_MS = 120_000;

/**
 Milliseconds the guest's build of the library and its tests may take.

 Measured on this host at one job: 24m55s on 2026-11-25T14:45 and 38m16s at 15:33, both while other rounds built on the
 same machine, and a third run was killed at the 40 minutes this bound was. The bound is a stop for a build that hangs,
 not a deadline the machine is expected to meet, so it carries room for a slower build under the load this host's
 rounds make.
 */
const CRATE_BUILD_TIMEOUT_MS = 5_400_000;

/**
 Milliseconds the guest arm may take.
 */
const ARM_TIMEOUT_MS = 1_200_000;

/**
 Seconds the guest's own `timeout` fires before `mvm`'s kill, so the guest reports a status rather than dying with it.
 */
const GUEST_TIMEOUT_MARGIN_SECONDS = 60;

/**
 Milliseconds the host waits past the build's own bound, so the guest can write the status line the host reads.
 */
const BUILD_FINISH_MARGIN_MS = 180_000;

/**
 Milliseconds the host waits past the arm's own bound, so the guest can write the status line the host reads.
 */
const ARM_FINISH_MARGIN_MS = 120_000;

/**
 How often the host asks the guest whether a detached script has finished, in milliseconds.
 */
const POLL_INTERVAL_MS = 15_000;

/**
 The guest name a run takes when the command line names none.
 */
const DEFAULT_GUEST_NAME = 'removal-lineage-proof';

/**
 The log directory a run takes when the command line names none: this package's build folder.
 */
const DEFAULT_LOG_DIRECTORY_NAME = 'removal-lineage-guest-logs';

//region Arguments

/**
 The step, the guest name and the log directory.
 */
type Options = {
  readonly step: string;
  readonly name: string;
  readonly logDirectory: string;
};

/**
 The log directory a run keeps its logs in, which this package's build folder holds unless the command line names one.

 @returns the directory this package's build folder holds for this driver's logs

 @example
 ```ts
 defaultLogDirectory(); // <package>/target/removal-lineage-guest-logs
 ```
 */
function defaultLogDirectory(): string {
  return resolve(
    import.meta.dirname,
    '..',
    'target',
    DEFAULT_LOG_DIRECTORY_NAME,
  );
}

/**
 The step and the values the command line gave it.

 @returns the step, the guest's name and the log directory

 @example
 ```ts
 const chosen = options();
 ```
 */
function options(): Options {
  /**
   The words this run was started with, after the program and this file's own path.
   */
  const argumentLine = process.argv
    .slice(2,);
  return {
    step: argumentLine[0] ?? '',
    name: flagValue({
      argumentWords: argumentLine,
      flag: '--name',
      fallback: DEFAULT_GUEST_NAME,
    },),
    logDirectory: flagValue({
      argumentWords: argumentLine,
      flag: '--log-dir',
      fallback: defaultLogDirectory(),
    },),
  };
}


//endregion Arguments

//region Steps

/**
 Create the guest, refusing a name another run holds.

 @param chosen - the run's options

 @throws GuestDriverError when the recipe's creation refused

 @example
 ```ts
 await create({ chosen });
 ```
 */
async function create({ chosen, }: { readonly chosen: GuestFacts; },): Promise<void> {
  await runGuestRecipeStep({
    name: chosen.name,
    logDirectory: chosen.logDirectory,
    step: 'create',
  },);
}

/**
 Install OpenZFS 2.4.4, the Rust toolchain and `cargo-nextest` in the guest.

 @param chosen - the run's options

 @throws GuestDriverError when the recipe's preparation refused

 @example
 ```ts
 await prepare({ chosen });
 ```
 */
async function prepare({ chosen, }: { readonly chosen: GuestFacts; },): Promise<void> {
  await runGuestRecipeStep({
    name: chosen.name,
    logDirectory: chosen.logDirectory,
    step: 'prepare',
  },);
}



/**
 Push a repository-shaped snapshot of the wrapper, build it in the guest, and run every arm.

 @param chosen - the run's options

 @throws GuestDriverError naming the step that refused, or the marker the pulled log lacks

 @example
 ```ts
 await run({ chosen });
 ```
 */
async function run({ chosen, }: { readonly chosen: GuestFacts; },): Promise<void> {
  /**
   The build context the wrapper's tree is copied into before it is packed.
   */
  const context = join(
    chosen.logDirectory,
    'snapshot',
  );
  await rm(
    context,
    {
      recursive: true,
      force: true,
    },
  );
  await mkdir(
    context,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  await copyWrapperSnapshot({ context, },);
  /**
   The packed tree this run hands the guest.
   */
  const tarball = join(
    chosen.logDirectory,
    'subject.tar.gz',
  );
  await runRequiredStepCommand({
    logDirectory: chosen.logDirectory,
    program: 'tar',
    argumentWords: [
      '--create',
      '--gzip',
      '--file',
      tarball,
      '--exclude=target',
      '--directory',
      context,
      '.',
    ],
    log: 'host-package.log',
    timeoutMilliseconds: HOST_COMMAND_TIMEOUT_MS,
  },);
  await runRequiredStepCommand({
    logDirectory: chosen.logDirectory,
    program: 'mvm',
    argumentWords: [
      '--backend',
      'libvirt',
      'push',
      chosen.name,
      tarball,
      '/root/subject.tar.gz',
    ],
    log: 'guest-push.log',
    timeoutMilliseconds: HOST_COMMAND_TIMEOUT_MS,
  },);
  await startDetachedGuestScript({
    logDirectory: chosen.logDirectory,
    name: chosen.name,
    script: buildScript(),
    log: 'removal-lineage-build.log',
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS,
    guestTimeoutMarginSeconds: GUEST_TIMEOUT_MARGIN_SECONDS,
  },);
  await awaitDetachedGuestScript({
    logDirectory: chosen.logDirectory,
    name: chosen.name,
    log: 'removal-lineage-build.log',
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS + BUILD_FINISH_MARGIN_MS,
    pollIntervalMilliseconds: POLL_INTERVAL_MS,
  },);
  await startDetachedGuestScript({
    logDirectory: chosen.logDirectory,
    name: chosen.name,
    script: armScript({ name: chosen.name, },),
    log: 'removal-lineage-arm-shell.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS,
    guestTimeoutMarginSeconds: GUEST_TIMEOUT_MARGIN_SECONDS,
  },);
  await awaitDetachedGuestScript({
    logDirectory: chosen.logDirectory,
    name: chosen.name,
    log: 'removal-lineage-arm-shell.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS + ARM_FINISH_MARGIN_MS,
    pollIntervalMilliseconds: POLL_INTERVAL_MS,
  },);
  await runRequiredStepCommand({
    logDirectory: chosen.logDirectory,
    program: 'mvm',
    argumentWords: [
      '--backend',
      'libvirt',
      'pull',
      chosen.name,
      `/mnt/shared/${ARM_LOG}`,
      join(
        chosen.logDirectory,
        ARM_LOG,
      ),
    ],
    log: 'host-pull-arm.log',
    timeoutMilliseconds: HOST_COMMAND_TIMEOUT_MS,
  },);
  /**
   The arm's own output, as the host pulled it back.
   */
  const pulled = await readStepLog({
    logDirectory: chosen.logDirectory,
    log: ARM_LOG,
  },);
  /**
   The first claim the pulled log lacks a line for, which the run refuses on.
   */
  const [firstMissing] = missingMarkers({
    text: pulled,
    markers: EXPECTED_MARKERS,
  },);
  if (firstMissing !== undefined)
    throw missingMarkerRefusal({ marker: firstMissing, },);
  if (holdsWordLine({
    text: pulled,
    word: SKIP_WORD,
  },)) {
    throw new GuestDriverError('the pulled arm log holds a SKIPPED line, so the arm did not run',);
  }
}

/**
 Destroy the guest and keep the list of what is left.

 @param chosen - the run's options

 @throws GuestDriverError when the recipe's destruction refused

 @example
 ```ts
 await destroy({ chosen });
 ```
 */
async function destroy({ chosen, }: { readonly chosen: GuestFacts; },): Promise<void> {
  await runGuestRecipeStep({
    name: chosen.name,
    logDirectory: chosen.logDirectory,
    step: 'destroy',
  },);
}

/**
 Create, prepare, run and destroy, in that order, destroying even when a step refused.

 @param chosen - the run's options

 @throws GuestDriverError naming the step that refused, after the guest it created is gone

 @example
 ```ts
 await all({ chosen });
 ```
 */
async function all({ chosen, }: { readonly chosen: GuestFacts; },): Promise<void> {
  await runWholeGuestRun({
    label: 'git-policy removal-lineage guest',
    chosen,
    steps: {
      create,
      prepare,
      work: run,
      destroy,
    },
  },);
}

//endregion Steps

//region The command line

/**
 The steps this driver knows, and nothing else.
 */
const steps: Record<string, GuestRunStep> = {
  create,
  prepare,
  run,
  destroy,
  all,
};

/**
 The options this run was given.
 */
const chosen = options();

/**
 The step this run performs, or nothing when the command line named one this driver does not know.
 */
const step = steps[chosen.step];
if (step === undefined) {
  throw new GuestDriverError(`unknown step '${chosen.step}'; use create, prepare, run, destroy or all`,);
}
await mkdir(
  chosen.logDirectory,
  {
    recursive: true,
    mode: 0o700,
  },
);
await step({ chosen, },);

//endregion The command line
