#!/usr/bin/env node
/**
 Drive the management grammar's two ZFS route words in a throwaway `mvm` guest: the cancel command's refusals and
 its reversal of a removal a kill left, and the settling command's refusal and its two settlements, each through the
 built wrapper.

 Both commands read `/dev/zfs` and this process's mount table, so no control on a person's machine may run them
 against a pool it did not build. This driver borrows the ZFS binding's guest recipe for everything that is not the
 wrapper's (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and
 prepares it with OpenZFS 2.4.4, the Rust toolchain and `cargo-nextest`, then builds this crate inside it, runs the
 arms one at a time through the `native_binary` target, and destroys the guest.

 Usage: node e2e/cancel-and-settle-commands-guest.ts <create|prepare|run|destroy|all> [--name <guest>] [--log-dir <dir>]

 The evidence of one run is `grammar-commands-arm.log`, pulled back into the log directory, and a run passes only
 when it carries every `GRAMMAR` line below and no `SKIPPED` line, so an arm that returned without running fails the
 run instead of reporting a green suite.
 */
import {
  mkdir,
} from 'node:fs/promises';
import {
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
  type GuestDriverOptions,
  flagWord,
  runArguments,
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
const CRATE_DIR = '/opt/grammar-commands';

/**
 Milliseconds the guest's build of the library and its tests may take, which is the removal-lineage driver's bound.
 */
const CRATE_BUILD_TIMEOUT_MS = 5_400_000;

/**
 Milliseconds one arm may take.
 */
const ARM_TIMEOUT_MS = 900_000;

/**
 Milliseconds the host waits past the build's own bound, so the guest can write the status line the host reads.
 */
const BUILD_FINISH_MARGIN_MS = 180_000;

/**
 Milliseconds the host waits past an arm's own bound, so the guest can write the status line the host reads.
 */
const ARM_FINISH_MARGIN_MS = 120_000;

/**
 The log the arms' own output is kept in, inside the guest's shared folder and beside the host's logs.
 */
const ARM_LOG = 'grammar-commands-arm.log';

/**
 The word a test binary prints at the start of a line when it returned without running its arm.
 */
const SKIP_WORD = 'SKIPPED';

/**
 The three arms, one nextest filterset each, so an arm that rebuilds the fixture never meets another's.
 */
const ARMS = [
  'the_cancel_word_refuses_a_slot_no_removal_has_touched',
  'the_settling_word_settles_a_slot_whose_clone_is_gone',
  'the_cancel_word_reverses_a_removal_a_kill_left',
] as const;

/**
 The lines a run must find in the guest's arm log, one per arm it performed.
 */
const EXPECTED_MARKERS = [
  'GRAMMAR cancel-untouched=refused status=2',
  'GRAMMAR settling-stands=refused status=1',
  'GRAMMAR settling-gone=settled status=0',
  'GRAMMAR cancel-killed-removal=reversed status=0',
  'GRAMMAR done',
] as const;

/**
 The pool every arm builds and destroys.
 */
const POOL = 'zfsgrammarcommands';

/**
 The folder the pool's image and the design's checkout live under.
 */
const FIXTURE_ROOT = '/home/zfs-grammar-commands-fixture';

/**
 The checkout the design's layout mounts.
 */
const CHECKOUT = `${FIXTURE_ROOT}/checkout`;

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
const DEFAULT_GUEST_NAME = 'zfs-r2-grammar-commands';

/**
 The log directory a run takes when the command line names none: this package's build folder.
 */
const DEFAULT_LOG_DIRECTORY_NAME = 'grammar-commands-guest-logs';

//endregion Constants

//region Arguments

/**
 The four flags every driver shares, and the keep decision this driver adds.
 */
type Options = GuestDriverOptions & {
  /**
   Whether a failed run keeps its guest for a person to look at.
   */
  readonly keepGuest: boolean;
};

/**
 The log directory a run keeps its logs in, which this package's build folder holds unless the command line names one.

 @returns the directory this package's build folder holds for this driver's logs

 @example
 ```ts
 defaultLogDirectory(); // <package>/target/grammar-commands-guest-logs
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

 @returns the step, the guest's name, the log directory and the keep decision

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
  /**
   The four flags every driver shares.
   */
  const read = runArguments({
    argv: argumentLine,
  },);

  return {
    step: read.step,
    name: flagWord({
      read,
      flag: '--name',
      fallback: DEFAULT_GUEST_NAME,
    },),
    logDir: flagWord({
      read,
      flag: '--log-dir',
      fallback: defaultLogDirectory(),
    },),
    keepGuest: read.flags
      .has('--keep-guest',),
  };
}

//endregion Arguments

//region Steps

/**
 The lines the guest's arm script runs to build the library and its test binaries.

 @returns the script, its lines in the order the guest runs them

 @example
 ```ts
 buildScript();
 ```
 */
function buildScript(): string {
  return [
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
}

/**
 The lines the guest's arm script runs to perform every arm and refuse a log that lacks a claim it made.

 @param name - the guest's name, which every guest variable answers

 @returns the script, its lines in the order the guest runs them

 @example
 ```ts
 armScript({ name: 'zfs-r2-grammar-commands' });
 ```
 */
function armScript({ name, }: { readonly name: string; },): string {
  /**
   The three arms, each its own nextest run so one arm's fixture never meets another's, and a failure that is kept
   rather than ending the script before the log is copied.
   */
  const runs = ARMS.map(function armRun(arm) {
    return `if ! cargo nextest run --release --locked --features test-only-switches --test native_binary --no-capture --status-level fail --filterset ${shellQuote(`test(${arm})`,)} >> /root/${ARM_LOG} 2>&1; then echo ${shellQuote(`the arm ${arm} failed`,)}; status=1; fi`;
  },);
  return [
    'set -eu',
    `export PATH=${OPENZFS_PATH}:/root/.cargo/bin:$PATH`,
    `export CLI_GIT_ZFS_GRAMMAR_GUEST=${name}`,
    `export CLI_GIT_ZFS_GRAMMAR_POOL=${POOL}`,
    `export CLI_GIT_ZFS_GRAMMAR_SBIN=${OPENZFS_PREFIX}/sbin`,
    `export CLI_GIT_ZFS_GRAMMAR_CHECKOUT=${CHECKOUT}`,
    `mkdir --parents ${FIXTURE_ROOT}`,
    `if ! zpool list -H -o name ${POOL} > /dev/null 2>&1; then`,
    `  truncate --size 1G ${FIXTURE_ROOT}/pool.img`,
    `  zpool create -f -O mountpoint=none ${POOL} ${FIXTURE_ROOT}/pool.img`,
    'fi',
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    `: > /root/${ARM_LOG}`,
    'status=0',
    ...runs,
    'test "$status" = 0',
    `cp /root/${ARM_LOG} /mnt/shared/${ARM_LOG} || true`,
    `tail --lines=120 /root/${ARM_LOG}`,
    `if grep --quiet '^${SKIP_WORD}' /root/${ARM_LOG}; then echo 'an arm returned without running'; exit 1; fi`,
    ...EXPECTED_MARKERS.map(function markerLine(marker) {
      return `grep --quiet --fixed-strings ${shellQuote(marker,)} /root/${ARM_LOG}`;
    },),
    `cp /root/${ARM_LOG} /mnt/shared/${ARM_LOG}`,
    `zpool destroy -f ${POOL} 2>/dev/null || true`,
    `rm --force ${FIXTURE_ROOT}/pool.img 2>/dev/null || true`,
  ].join('\n',);
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
  await pushWorktreeSnapshot({
    chosen,
  },);
  await startDetachedGuestScript({
    chosen,
    script: buildScript(),
    log: 'grammar-commands-build.log',
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS,
    nameCheck: 'none',
  },);
  await awaitDetachedGuestScript({
    chosen,
    log: 'grammar-commands-build.log',
    finishTimeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS + BUILD_FINISH_MARGIN_MS,
  },);
  await startDetachedGuestScript({
    chosen,
    script: armScript({
      name: chosen.name,
    },),
    log: 'grammar-commands-arm-shell.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS,
    nameCheck: 'none',
  },);
  try {
    await awaitDetachedGuestScript({
      chosen,
      log: 'grammar-commands-arm-shell.log',
      finishTimeoutMilliseconds: ARM_TIMEOUT_MS + ARM_FINISH_MARGIN_MS,
    },);
  } catch (error) {
    // A failed arm's own log is the evidence a person needs, so it is pulled before the failure is reported, and the
    // guest is kept so the script's log can be read in it.
    await pullGuestLog({
      chosen,
      file: ARM_LOG,
      log: 'host-pull-arm.log',
    },);
    throw error;
  }
  /**
   The arms' own output, as the host pulled it back.
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
    because: 'the guest ran an arm without performing it',
  },);
  if (holdsLineStartingWith({
    text: pulled,
    word: SKIP_WORD,
  },)) {
    throw new GuestDriverError('the pulled arm log holds a SKIPPED line, so an arm did not run',);
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
    label: 'git-policy grammar-commands guest',
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
  },),
  prepare: recipeStepOf({
    step: 'prepare',
  },),
  destroy: recipeStepOf({
    step: 'destroy',
  },),
};

/**
 The steps this driver knows, and nothing else.
 */
const steps: ReadonlyMap<string, (chosen: GuestDriverOptions,) => Promise<void>> = new Map([
  [
    'create',
    async function createStep(chosen: GuestDriverOptions,) {
      await recipeSteps.create({
        chosen,
      },);
    },
  ],
  [
    'prepare',
    async function prepareStep(chosen: GuestDriverOptions,) {
      await recipeSteps.prepare({
        chosen,
      },);
    },
  ],
  [
    'run',
    async function runStep(chosen: GuestDriverOptions,) {
      await run({
        chosen,
      },);
    },
  ],
  [
    'destroy',
    async function destroyStep(chosen: GuestDriverOptions,) {
      await recipeSteps.destroy({
        chosen,
      },);
    },
  ],
  [
    'all',
    async function allStep(chosen: GuestDriverOptions,) {
      await all({
        chosen,
      },);
    },
  ],
],);

/**
 The step this run performs.
 */
const step = steps.get(runOptions.step,);
if (step === undefined) {
  throw new GuestDriverError(`unknown step '${runOptions.step}'; use ${[...steps.keys(),].join(', ')}`,);
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
