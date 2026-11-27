#!/usr/bin/env node
/**
 Drive the settling command's guest arm in a throwaway `mvm` guest: the absence proof and the two removal orders
 against a real file-backed pool.

 The command reads `/dev/zfs`, so no control on a person's machine may run it against a pool it did not build. This
 driver borrows the ZFS binding's guest recipe for everything that is not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and prepares it
 with OpenZFS 2.4.4, the Rust toolchain and `cargo-nextest`, then builds this crate inside it, runs the arm, and
 destroys the guest.

 The arm, as plan step 16's "VM measurements" ask (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
 "Step 16"):
 - a clone that stands refuses the settling command and nothing is removed;
 - a clone a person destroyed outside the wrapper is settled: the registration, the lock and the three records go;
 - a slot whose checkout has begun keeps its registration, its pointer and the person's file, and only the records go.

 Usage: node e2e/settling-command-guest.ts <create|prepare|run|destroy|all> [--name <guest>] [--log-dir <dir>]

 The evidence of one run is `settling-command-arm.log`, pulled back into the log directory, and a run passes only when
 it carries every `SETTLING` line below and no `SKIPPED` line, so an arm that returned without running fails the run
 instead of reporting a green suite.
 */
import {
  mkdir,
  rm,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  FOLDER_MODE,
  SWAP_GIBIBYTES,
} from '../bin/mvm-guest-bounds.ts';
import {
  holdsLineStartingWith,
  requireMarkers,
} from '../bin/mvm-guest-arm-log.ts';
import {
  shellQuote,
} from '../bin/mvm-guest-command-line.ts';
import {
  GuestDriverError,
} from '../bin/mvm-guest-error.ts';
import type { GuestFacts, } from '../bin/mvm-guest-guest-facts.ts';
import {
  pushWorktreeSnapshot,
  recipe,
} from '../bin/mvm-guest-host-command.ts';
import {
  runArguments,
  type GuestDriverOptions,
  type RunArguments,
} from '../bin/mvm-guest-options.ts';
import {
  recipeStepOf,
} from '../bin/mvm-guest-recipe-step.ts';
import {
  awaitDetachedGuestScript,
  pullGuestLog,
  startDetachedGuestScript,
} from '../bin/mvm-guest-detached-script.ts';
import {
  runWholeGuestRun,
} from '../bin/mvm-guest-whole-run.ts';

//region Constants

/**
 Where the wrapper's tree is unpacked and built inside the guest.
 */
const CRATE_DIR = '/opt/settling-command';

/**
 Milliseconds the guest's build of the library and its tests may take.
 */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;

/**
 Milliseconds the guest arm may take.
 */
const ARM_TIMEOUT_MS = 900_000;

/**
 Milliseconds the host waits past the build's own bound, so the guest can write the status line the host reads.
 */
const BUILD_FINISH_MARGIN_MS = 180_000;

/**
 Milliseconds the host waits past the arm's own bound, so the guest can write the status line the host reads.
 */
const ARM_FINISH_MARGIN_MS = 120_000;

/**
 The environment the arm reads, so it never opens the host's device.
 */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_SETTLING_GUEST';

/**
 The pool the arm builds and destroys.
 */
const POOL = 'zfsstep16settling';

/**
 The folder the pool's image and the fixture repository live under.
 */
const FIXTURE_ROOT = '/home/zfs-step-16-settling-fixture';

/**
 The OpenZFS 2.4.4 prefix the guest's recipe installs.
 */
const OPENZFS_PREFIX = '/opt/openzfs-2.4.4';

/**
 The `PATH` the guest's OpenZFS userspace needs.
 */
const OPENZFS_PATH = `${OPENZFS_PREFIX}/sbin:${OPENZFS_PREFIX}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;

/**
 The guest name a run takes when the command line names none.
 */
const DEFAULT_GUEST_NAME = 'settling-command-arm';

/**
 The log directory a run takes when the command line names none: this package's build folder.
 */
const DEFAULT_LOG_DIRECTORY_NAME = 'settling-command-guest-logs';

/**
 The log the arm's own test output is kept in, inside the guest's shared folder and beside the host's logs.
 */
const ARM_LOG = 'settling-command-arm.log';

/**
 The word a test binary prints at the start of a line when it returned without running its arm.
 */
const SKIP_WORD = 'SKIPPED';

/**
 The lines a run must find in the guest's arm log, one per arm it performed.
 */
const EXPECTED_MARKERS = [
  'SETTLING stands=refused',
  'SETTLING gone=settled',
  'SETTLING checkout=records-only',
  'SETTLING done',
] as const;

//endregion Constants

//region Arguments

/**
 The four flags every driver shares, and the custom template a run of this one may name.
 */
type Options = GuestDriverOptions & {
  /**
   The `mvm` custom template the guest is made from, or absent for the Ubuntu 24.04 image the recipe prepares itself.
   */
  readonly image?: string;
  /**
   The kernel release that template must boot, which the recipe checks instead of installing a module.
   */
  readonly kernelRelease?: string;
};

/**
 The log directory a run keeps its logs in, which this package's build folder holds unless the command line names one.

 @returns the directory this package's build folder holds for this driver's logs

 @example
 ```ts
 defaultLogDirectory(); // <package>/target/settling-command-guest-logs
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

 @returns the step, the guest's name, the log directory, the keep decision and the template a run names

 @example
 ```ts
 const runOptions = options();
 ```
 */
function options(): Options {
  /**
   The words this run was started with, after the program and this file's own path.
   */
  const argumentLine = process.argv
    .slice(2,);
  /**
   The four flags every driver shares, and the words after the step that only this driver reads.
   */
  const read: RunArguments = runArguments({ argv: argumentLine, },);
  /**
   The template the command line names, or absent when it names none.
   */
  const image = read.values
    .get('--image',);
  /**
   The kernel release the command line names, or absent when it names none.
   */
  const kernelRelease = read.values
    .get('--kernel-release',);
  return {
    step: read.step,
    name: read.values
      .get('--name',) ?? DEFAULT_GUEST_NAME,
    logDir: read.values
      .get('--log-dir',) ?? defaultLogDirectory(),
    keepGuest: read.flags
      .has('--keep-guest',),
    ...(image === undefined ? {} : { image, }),
    ...(kernelRelease === undefined ? {} : { kernelRelease, }),
  };
}

/**
 The words the recipe's steps take beyond the guest's name and log directory.

 A run that names a custom template hands the recipe the template and the kernel release it must boot; a run that
 names neither leaves the recipe on its own Ubuntu 24.04 image, which it installs OpenZFS into.

 @param chosen - the run's options

 @returns the recipe's extra words, empty for a run that names no template

 @example
 ```ts
 recipeExtraArguments({ chosen }); // ['--image', 'cachyos-template', '--kernel-release', '7.3.5-1-cachyos']
 ```
 */
function recipeExtraArguments({ chosen, }: { readonly chosen: Options; },): readonly string[] {
  /**
   The extra words, in the order the recipe reads them.
   */
  const words: string[] = [];
  if (chosen.image !== undefined)
    words.push(
      '--image',
      chosen.image,
    );
  if (chosen.kernelRelease !== undefined)
    words.push(
      '--kernel-release',
      chosen.kernelRelease,
    );
  return words;
}

//endregion Arguments

//region Steps



/**
 Push a repository-shaped snapshot of the wrapper, build it in the guest, and run the arm.

 @param chosen - the run's options

 @throws GuestDriverError naming the step that refused, or the marker the pulled log lacks

 @example
 ```ts
 await run({ chosen });
 ```
 */
async function run({ chosen, }: { readonly chosen: GuestFacts; },): Promise<void> {
  await pushWorktreeSnapshot({ chosen, },);
  /**
   The script the guest runs to unpack and build the library and its tests.
   */
  const build = [
    'set -eu',
    'test -f /mnt/shared/subject.tar.gz',
    'cp /mnt/shared/subject.tar.gz /root/subject.tar.gz',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'df --human-readable / /mnt/shared',
    'free',
    `if ! swapon --show | grep --quiet /swapfile; then fallocate --length ${SWAP_GIBIBYTES} /swapfile; chmod 600 /swapfile; mkswap /swapfile; swapon /swapfile; fi`,
    'free',
    'cargo build --release --tests --locked --features test-only-switches --jobs 1',
  ].join('\n',);
  await startDetachedGuestScript({
    chosen,
    script: build,
    log: 'settling-command-build.log',
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS,
    nameCheck: 'none',
  },);
  await awaitDetachedGuestScript({
    chosen,
    log: 'settling-command-build.log',
    finishTimeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS + BUILD_FINISH_MARGIN_MS,
  },);
  /**
   The script the guest runs to perform the arm, refuse a log that lacks a claim it made, and leave nothing behind.
   */
  const arm = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}:/root/.cargo/bin:$PATH`,
    `export ${GUEST_VARIABLE}=${chosen.name}`,
    `export CLI_GIT_ZFS_SETTLING_POOL=${POOL}`,
    `export CLI_GIT_ZFS_SETTLING_PREFIX=${OPENZFS_PREFIX}`,
    `export CLI_GIT_ZFS_SETTLING_ROOT=${FIXTURE_ROOT}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'if cargo nextest run --release --locked --features test-only-switches --lib --run-ignored all --no-capture --status-level fail ' +
      '--filterset "test(the_settling_command_guest_arms)" '
      +
      `> /root/${ARM_LOG} 2>&1; then status=0; else status=$?; fi`,
    `cp /root/${ARM_LOG} /mnt/shared/${ARM_LOG} || true`,
    `tail --lines=120 /root/${ARM_LOG}`,
    'test "$status" = 0',
    `if grep --quiet '^${SKIP_WORD}' /root/${ARM_LOG}; then echo 'the arm returned without running'; exit 1; fi`,
    ...EXPECTED_MARKERS.map(function markerLine(marker) {
      return `grep --quiet --fixed-strings ${shellQuote(marker,)} /root/${ARM_LOG}`;
    },),
    `cp /root/${ARM_LOG} /mnt/shared/${ARM_LOG}`,
    `zpool destroy -f ${POOL} 2>/dev/null || true`,
    `rm --force ${FIXTURE_ROOT}/pool.img 2>/dev/null || true`,
  ].join('\n',);
  await startDetachedGuestScript({
    chosen,
    script: arm,
    log: 'settling-command-arm-shell.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS,
    nameCheck: 'none',
  },);
  await awaitDetachedGuestScript({
    chosen,
    log: 'settling-command-arm-shell.log',
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
 Create, prepare, run and destroy, in that order, destroying even when a step refused.

 @param chosen - the run's options

 @throws GuestDriverError naming the step that refused, after the guest it created is gone

 @example
 ```ts
 await all({ chosen });
 ```
 */
async function all({ chosen, }: { readonly chosen: Options; },): Promise<void> {
  await runWholeGuestRun({
    label: 'git-policy settling-command guest',
    chosen,
    keepGuest: chosen.keepGuest,
    steps: {
      ...recipeSteps,
      work: run,
    },
  },);
}

//endregion Steps

//region The command line

/**
 The options this run was given.
 */
const runOptions = options();

/**
 The recipe's three steps, which this driver's command line and its whole run both perform.
 */
const recipeSteps = {
  create: recipeStepOf({
    step: 'create',
    extraArgumentWords: recipeExtraArguments({ chosen: runOptions, },),
  },),
  prepare: recipeStepOf({
    step: 'prepare',
    extraArgumentWords: recipeExtraArguments({ chosen: runOptions, },),
  },),
  destroy: recipeStepOf({
    step: 'destroy',
    extraArgumentWords: recipeExtraArguments({ chosen: runOptions, },),
  },),
};

/**
 The steps this driver knows, and nothing else.
 */
const steps: ReadonlyMap<string, (chosen: GuestDriverOptions,) => Promise<void>> = new Map([
  [
    'create',
    async function createStep(chosen: GuestDriverOptions,) {
      await recipeSteps.create({ chosen, },);
    },
  ],
  [
    'prepare',
    async function prepareStep(chosen: GuestDriverOptions,) {
      await recipeSteps.prepare({ chosen, },);
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
      await recipeSteps.destroy({ chosen, },);
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
 The step this run performs.
 */
const step: ((chosen: GuestDriverOptions,) => Promise<void>) | undefined = steps.get(runOptions.step,);
if (step === undefined) {
  throw new GuestDriverError(`unknown step '${runOptions.step}'; use ${[...steps.keys(),].join(', ',)}`,);
}
await mkdir(
  runOptions.logDir,
  {
    recursive: true,
    mode: FOLDER_MODE,
  },
);
await step(runOptions,);

//endregion The command line
