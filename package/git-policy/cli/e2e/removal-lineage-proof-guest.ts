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
import { mkdir, } from 'node:fs/promises';
import { resolve, } from 'node:path';

import {
  FOLDER_MODE,
} from '../bin/mvm-guest-bounds.ts';
import {
  holdsLineStartingWith,
  requireMarkers,
} from '../bin/mvm-guest-arm-log.ts';
import {
  GuestDriverError,
} from '../bin/mvm-guest-error.ts';
import type { GuestFacts, } from '../bin/mvm-guest-guest-facts.ts';
import {
  pushWorktreeSnapshot,
  recipe,
} from '../bin/mvm-guest-host-command.ts';
import type { GuestDriverOptions, } from '../bin/mvm-guest-options.ts';
import {
  guestDriverOptions,
} from '../bin/mvm-guest-options.ts';
import {
  awaitDetachedGuestScript,
  pullGuestLog,
  startDetachedGuestScript,
} from '../bin/mvm-guest-detached-script.ts';
import {
  ARM_LOG,
  EXPECTED_MARKERS,
  SKIP_WORD,
  armScript,
  buildScript,
} from './removal-lineage-proof-guest-arm-scripts.ts';
import {
  runWholeGuestRun,
} from '../bin/mvm-guest-whole-run.ts';

//region Constants

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
 Milliseconds the host waits past the build's own bound, so the guest can write the status line the host reads.
 */
const BUILD_FINISH_MARGIN_MS = 180_000;

/**
 Milliseconds the host waits past the arm's own bound, so the guest can write the status line the host reads.
 */
const ARM_FINISH_MARGIN_MS = 120_000;

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
  await recipe({
    chosen,
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
  await recipe({
    chosen,
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
  await pushWorktreeSnapshot({ chosen, },);
  await startDetachedGuestScript({
    chosen,
    script: buildScript(),
    log: 'removal-lineage-build.log',
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS,
    nameCheck: 'none',
  },);
  await awaitDetachedGuestScript({
    chosen,
    log: 'removal-lineage-build.log',
    finishTimeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS + BUILD_FINISH_MARGIN_MS,
  },);
  await startDetachedGuestScript({
    chosen,
    script: armScript({ name: chosen.name, },),
    log: 'removal-lineage-arm-shell.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS,
    nameCheck: 'none',
  },);
  await awaitDetachedGuestScript({
    chosen,
    log: 'removal-lineage-arm-shell.log',
    finishTimeoutMilliseconds: ARM_TIMEOUT_MS + ARM_FINISH_MARGIN_MS,
  },);
  /**
   The arm's own output, as the host pulled it back.
   */
  const pulled = await pullGuestLog({
    chosen,
    file: ARM_LOG,
    log: 'host-pull-arm.log',
  },);
  requireMarkers({
    text: pulled,
    markers: EXPECTED_MARKERS,
    subject: 'the pulled arm log',
    because: 'the guest ran the arm without performing it',
  },);
  if (holdsLineStartingWith({
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
  await recipe({
    chosen,
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
async function all({ chosen, }: { readonly chosen: GuestDriverOptions; },): Promise<void> {
  await runWholeGuestRun({
    label: 'git-policy removal-lineage guest',
    chosen,
    keepGuest: chosen.keepGuest,
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
const steps: ReadonlyMap<string, (chosen: GuestDriverOptions,) => Promise<void>> = new Map([
  [
    'create',
    async function createStep(chosen: GuestDriverOptions,) {
      await create({ chosen, },);
    },
  ],
  [
    'prepare',
    async function prepareStep(chosen: GuestDriverOptions,) {
      await prepare({ chosen, },);
    },
  ],
  [
    'run',
    async function runStep(chosen: GuestDriverOptions,) {
      await run({ chosen, },);
    },
  ],
  [
    'destroy',
    async function destroyStep(chosen: GuestDriverOptions,) {
      await destroy({ chosen, },);
    },
  ],
  [
    'all',
    async function allStep(chosen: GuestDriverOptions,) {
      await all({ chosen, },);
    },
  ],
],);

/**
 The options this run was given.
 */
const chosen = guestDriverOptions({
  argv: process.argv.slice(2,),
  defaultName: DEFAULT_GUEST_NAME,
  defaultLogDir: defaultLogDirectory(),
},);

/**
 The step this run performs.
 */
const step: ((chosen: GuestDriverOptions,) => Promise<void>) | undefined = steps.get(chosen.step,);
if (step === undefined) {
  throw new GuestDriverError(`unknown step '${chosen.step}'; use ${[...steps.keys(),].join(', ',)}`,);
}
await mkdir(
  chosen.logDir,
  {
    recursive: true,
    mode: FOLDER_MODE,
  },
);
await step(chosen,);

//endregion The command line
