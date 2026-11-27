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

import { copyWrapperSnapshot, } from '../bin/native-wrapper-snapshot.ts';
import {
  holdsWordLine,
  missingMarkerRefusal,
  missingMarkers,
} from './guest-driver-arm-log.ts';
import {
  flagValue,
  shellQuote,
} from './guest-driver-command-line.ts';
import { GuestDriverError, } from './guest-driver-error.ts';
import {
  type GuestFacts,
  recipeStepOf,
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
  type GuestRunStep,
  runWholeGuestRun,
} from './guest-driver-whole-run.ts';

//region Constants

/**
 Where the wrapper's tree is unpacked and built inside the guest.
 */
const CRATE_DIR = '/opt/settling-command';

/**
 Milliseconds one host command may take.
 */
const HOST_COMMAND_TIMEOUT_MS = 120_000;

/**
 Milliseconds the guest's build of the library and its tests may take.
 */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;

/**
 Milliseconds the guest arm may take.
 */
const ARM_TIMEOUT_MS = 900_000;

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
 The swap file the guest adds, in gibibytes, so one rustc does not exhaust its memory.
 */
const SWAP_GIB = '6G';

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
 The step, the guest name, the log directory and, when a run names them, the custom template the guest is made from.
 */
type Options = {
  readonly step: string;
  readonly name: string;
  readonly logDirectory: string;
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

 @returns the step, the guest's name, the log directory and the template a run names

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
   The template the command line names, or empty text when it names none.
   */
  const image = flagValue({
    argumentWords: argumentLine,
    flag: '--image',
    fallback: '',
  },);
  /**
   The kernel release the command line names, or empty text when it names none.
   */
  const kernelRelease = flagValue({
    argumentWords: argumentLine,
    flag: '--kernel-release',
    fallback: '',
  },);
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
    ...(image === '' ? {} : { image, }),
    ...(kernelRelease === '' ? {} : { kernelRelease, }),
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
    `if ! swapon --show | grep --quiet /swapfile; then fallocate --length ${SWAP_GIB} /swapfile; chmod 600 /swapfile; mkswap /swapfile; swapon /swapfile; fi`,
    'free',
    'cargo build --release --tests --locked --features test-only-switches --jobs 1',
  ].join('\n',);
  await startDetachedGuestScript({
    logDirectory: chosen.logDirectory,
    name: chosen.name,
    script: build,
    log: 'settling-command-build.log',
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS,
    guestTimeoutMarginSeconds: GUEST_TIMEOUT_MARGIN_SECONDS,
  },);
  await awaitDetachedGuestScript({
    logDirectory: chosen.logDirectory,
    name: chosen.name,
    log: 'settling-command-build.log',
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS + BUILD_FINISH_MARGIN_MS,
    pollIntervalMilliseconds: POLL_INTERVAL_MS,
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
      return `grep --quiet --fixed-strings ${shellQuote({ script: marker, },)} /root/${ARM_LOG}`;
    },),
    `cp /root/${ARM_LOG} /mnt/shared/${ARM_LOG}`,
    `zpool destroy -f ${POOL} 2>/dev/null || true`,
    `rm --force ${FIXTURE_ROOT}/pool.img 2>/dev/null || true`,
  ].join('\n',);
  await startDetachedGuestScript({
    logDirectory: chosen.logDirectory,
    name: chosen.name,
    script: arm,
    log: 'settling-command-arm-shell.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS,
    guestTimeoutMarginSeconds: GUEST_TIMEOUT_MARGIN_SECONDS,
  },);
  await awaitDetachedGuestScript({
    logDirectory: chosen.logDirectory,
    name: chosen.name,
    log: 'settling-command-arm-shell.log',
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
    label: 'git-policy settling-command guest',
    chosen,
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
const steps: Record<string, GuestRunStep> = {
  create: recipeSteps.create,
  prepare: recipeSteps.prepare,
  run,
  destroy: recipeSteps.destroy,
  all,
};

/**
 The step this run performs, or nothing when the command line named one this driver does not know.
 */
const step = steps[runOptions.step];
if (step === undefined) {
  throw new GuestDriverError(`unknown step '${runOptions.step}'; use create, prepare, run, destroy or all`,);
}
await mkdir(
  runOptions.logDirectory,
  {
    recursive: true,
    mode: 0o700,
  },
);
await step({ chosen: runOptions, },);

//endregion The command line
