#!/usr/bin/env node
/**
 Drive plan step 19's shutdown pass in a throwaway `mvm` guest: an idle clone unmounted, a busy one kept and named,
 this boot's readiness withdrawn before either, and no durable intent changed.

 The pass reads this boot's readiness, this process's mount table, the route's recorded `.claude/worktrees` identity and
 one dataset's GUID, and it unmounts a real mount, so no control on a person's machine may run it. This driver borrows
 the ZFS binding's guest recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and prepares it
 with OpenZFS 2.4.4, the Rust toolchain and `cargo-nextest`, then pushes this worktree, builds the crate's test targets
 inside it, runs the arm, and destroys the guest.

 The mode that will call the pass is still the stub that refuses (`zfs-root shutdown-unmount`), so the arm calls the pass
 itself: that is what this round proves, and the round that wires the mode adds the mode's own arm.

 Usage: node e2e/shutdown-unmount-guest.ts <create|prepare|run|destroy|all> [--name <guest>] [--log-dir <dir>]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `shutdown-unmount-arm.log`, pulled back into the log directory, and a run passes only when
 it carries every `SHUTDOWN-UNMOUNT` line below and no `SKIPPED` line, so an arm that returned without running fails the
 run instead of reporting a green suite.
 */
import { mkdir, } from 'node:fs/promises';
import { resolve, } from 'node:path';

import { guestBackend, } from '../bin/mvm-guest-backend.ts';
import {
  FOLDER_MODE,
  HOST_COMMAND_TIMEOUT_MS,
} from '../bin/mvm-guest-bounds.ts';
import {
  holdsGuestName,
} from '../bin/mvm-guest-listing.ts';
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
import {
  guest,
} from '../bin/mvm-guest-script.ts';
import {
  type GuestDriverOptions,
  guestDriverOptions,
} from '../bin/mvm-guest-options.ts';
import {
  awaitDetachedGuestScript,
  pullGuestLog,
  startDetachedGuestScript,
} from '../bin/mvm-guest-detached-script.ts';
import {
  captureHostCommand,
} from '../bin/mvm-guest-step-command.ts';
import {
  runWholeGuestRun,
} from '../bin/mvm-guest-whole-run.ts';
import {
  ARM_LOG,
  BUILD_LOG,
  EXPECTED_MARKERS,
  OPENZFS_PREFIX,
  POOL,
  POOL_IMAGE_FOLDER,
  SKIP_WORD,
  armScript,
  buildScript,
} from './shutdown-unmount-guest-scripts.ts';

/**
 The backend this run drives, read from the one place every guest step reads it from.
 */
const backend = guestBackend();

//region Constants

/**
 Milliseconds the guest's build of the library and its tests may take.

 The removal driver measured a whole-crate test build at 24m55s and 38m16s at one job on this host, so this bound is a
 stop for a build that hangs rather than a deadline the machine is expected to meet.
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
 Milliseconds the guest's own pool step may take.
 */
const POOL_TIMEOUT_MS = 600_000;

/**
 The guest name a run takes when the command line names none, which begins the way every guest of this round does.
 */
const DEFAULT_GUEST_NAME = 'zfs-r7-shutdown-unmount';

/**
 The log directory a run takes when the command line names none: this package's build folder.
 */
const DEFAULT_LOG_DIRECTORY_NAME = 'shutdown-unmount-guest-logs';

/**
 The ready clones the arm builds, and the mounts it takes down, which the guest keeps between the arm and the destroy.
 */
/**
 The one guest step that prepares the pool the arm's datasets live on.

 The arm makes its own checkout dataset, its container and its clones, so this step only makes the pool: a file-backed
 one, because nothing of the guest's root disk may be a vdev.

 @returns the script, its lines in the order the guest runs them

 @example
 ```ts
 const script = poolScript();
 ```
 */
function poolScript(): string {
  return [
    'set -eu',
    `mkdir --parents ${POOL_IMAGE_FOLDER}`,
    `if ! ${OPENZFS_PREFIX}/sbin/zpool list -H -o name ${POOL} > /dev/null 2>&1; then`,
    `  truncate --size 1G ${POOL_IMAGE_FOLDER}/pool.img`,
    `  ${OPENZFS_PREFIX}/sbin/zpool create -f -O mountpoint=none ${POOL} ${POOL_IMAGE_FOLDER}/pool.img`,
    'fi',
  ].join('\n',);
}

//endregion Constants

//region Steps

/**
 Create the guest, refusing a name another run holds.

 @param chosen - the run's options

 @throws GuestDriverError when the name is taken or the recipe's creation refused

 @example
 ```ts
 await create({ chosen });
 ```
 */
async function create({ chosen, }: { readonly chosen: GuestFacts; },): Promise<void> {
  /**
   What `mvm list` reported before this run asked for a guest.
   */
  const listed = await captureHostCommand({
    command: 'mvm',
    args: [
      ...backend.mvmArgumentWords,
      'list',
    ],
    timeoutMilliseconds: HOST_COMMAND_TIMEOUT_MS,
  },);
  if (holdsGuestName({
    listing: listed,
    name: chosen.name,
  },)) {
    throw new GuestDriverError(`a guest named ${chosen.name} is already listed; name another with --name`,);
  }
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
 Push this worktree, make the guest's pool, build the crate's test targets and run the arm.

 @param chosen - the run's options

 @throws GuestDriverError naming the step that refused, or the marker the pulled log lacks

 @example
 ```ts
 await run({ chosen });
 ```
 */
async function run({ chosen, }: { readonly chosen: GuestFacts; },): Promise<void> {
  await pushWorktreeSnapshot({ chosen, },);
  await guest({
    chosen,
    script: poolScript(),
    log: 'guest-pool.log',
    timeoutMilliseconds: POOL_TIMEOUT_MS,
  },);
  await startDetachedGuestScript({
    chosen,
    script: buildScript(),
    log: BUILD_LOG,
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS,
    nameCheck: 'none',
  },);
  await awaitDetachedGuestScript({
    chosen,
    log: BUILD_LOG,
    finishTimeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS + BUILD_FINISH_MARGIN_MS,
  },);
  await startDetachedGuestScript({
    chosen,
    script: armScript({ name: chosen.name, },),
    log: 'shutdown-unmount-arm-shell.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS,
    nameCheck: 'none',
  },);
  await awaitDetachedGuestScript({
    chosen,
    log: 'shutdown-unmount-arm-shell.log',
    finishTimeoutMilliseconds: ARM_TIMEOUT_MS + ARM_FINISH_MARGIN_MS,
  },);
  /**
   The arm's own output, as the host pulled it back.
   */
  const pulled: string = await pullGuestLog({
    chosen,
    file: ARM_LOG,
    log: 'host-pull-arm.log',
  },);
  if (holdsLineStartingWith({
    text: pulled,
    word: SKIP_WORD,
  },)) {
    throw new GuestDriverError('the pulled arm log holds a SKIPPED line, so the arm did not run',);
  }
  requireMarkers({
    text: pulled,
    markers: EXPECTED_MARKERS,
    subject: 'the pulled arm log',
    because: 'the guest ran the arm without performing it',
  },);
}

/**
 Destroy the guest this run made.

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
    label: 'git-policy shutdown-unmount guest',
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
  argv: process.argv
    .slice(2,),
  defaultName: DEFAULT_GUEST_NAME,
  defaultLogDir: resolve(
    import.meta.dirname,
    '..',
    'target',
    DEFAULT_LOG_DIRECTORY_NAME,
  ),
},);

/**
 The step this run performs.
 */
const step = steps.get(chosen.step,);
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
