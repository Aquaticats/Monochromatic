#!/usr/bin/env node
/**
 Drive plan step 13's managed-Git-lock arms in one throwaway `mvm` guest, on a file-backed pool.

 The arms need a real ZFS clone mounted over a Git worktree slot and a file-backed pool, so nothing here may run on a
 person's machine: the driver borrows the ZFS binding's guest recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`), makes the pool the arms clone on, builds this
 crate's `native_binary` test target inside the guest, runs the lock arm there, and destroys the guest.

 The arm is plan step 13's "VM measurements": item 6's lock arms, extended to installing the lock after pointer loss,
 and measurement `M2`'s mounted arm of the two-fact proof.

 Usage: node e2e/managed-git-lock-guest.ts <create|prepare|pool|run|destroy|all> [--name <guest>]
          [--log-dir <dir>] [--keep-guest]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `lock-arm.log`, pulled back into the log directory, and a run passes only when it carries
 every `LOCK` line below and no `SKIP` line, so an arm that returned without running fails the run instead of
 reporting a green suite.
 */
import {
  mkdir,
} from 'node:fs/promises';
import {
  resolve,
} from 'node:path';

import {
  guestDriverOptions,
  type GuestDriverOptions,
} from '../bin/mvm-guest-options.ts';
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
import {
  guestBackend,
  refuseAStepWhoseArmsCannotRun,
  STEPS_THAT_RUN_NO_ARM,
} from '../bin/mvm-guest-backend.ts';
import type { GuestFacts, } from '../bin/mvm-guest-guest-facts.ts';
import {
  pushWorktreeSnapshot,
  recipe,
} from '../bin/mvm-guest-host-command.ts';
import {
  recipeStepOf,
} from '../bin/mvm-guest-recipe-step.ts';
import {
  guest,
} from '../bin/mvm-guest-script.ts';
import {
  pullGuestLog,
} from '../bin/mvm-guest-detached-script.ts';
import {
  runWholeGuestRun,
} from '../bin/mvm-guest-whole-run.ts';

//region Constants

/**
 Where the crate is unpacked and built inside the guest, which every account may traverse.
 */
const CRATE_DIR = '/opt/git-policy-cli-lock';

/**
 The checkout the arm's Git repository and its worktree live in, under the guest's own root.
 */
const CHECKOUT = '/root/lock-checkout';

/**
 The pool the arm's clone parent and clone live in, and the file it stands on.
 */
const POOL = 'git-policy-cli-store';

/**
 The file the pool's one vdev stands on, inside the guest.
 */
const POOL_IMAGE = '/home/git-policy-cli-lock-fixture/lock.img';

/**
 The `PATH` the guest's OpenZFS 2.4.4 userspace needs, which the recipe installs under `/opt`.
 */
const OPENZFS_PATH = '/opt/openzfs-2.4.4/sbin:/opt/openzfs-2.4.4/bin:/sbin:/usr/sbin:/bin:/usr/bin';

/**
 The variable the arm reads, so it never runs outside the guest.
 */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_LOCK_GUEST';

/**
 Milliseconds the guest's build of the test target may take.
 */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;

/**
 Milliseconds the arm may take.
 */
const ARM_TIMEOUT_MS = 1_800_000;

/**
 The guest name a run takes when the command line names none.
 */
const DEFAULT_GUEST_NAME = 'git-policy-lock';

/**
 The log directory a run takes when the command line names none: this package's build folder.
 */
const DEFAULT_LOG_DIRECTORY_NAME = 'lock-guest-logs';

/**
 The name of the file the arm's whole output is pulled back into.
 */
const ARM_LOG = 'lock-arm.log';

/**
 The word a test binary prints at the start of a line when it returned without running its arm.
 */
const SKIP_WORD = 'SKIP';

/**
 The lines a run must find in the guest's arm log, one per arm it performed.
 */
const EXPECTED_MARKERS = [
  'LOCK mounted slot-identity-answers=no',
  'LOCK mounted classified managed=yes',
  'LOCK mounted clone-guid-answers=yes',
  'LOCK mounted cloned-again refused=yes',
  'LOCK mounted restored-clone managed=yes',
  'LOCK classified managed-nonce=1 pointer=absent',
  'LOCK prune-kept-the-locked-registration=true',
  'LOCK operator class=an-operators-lock',
  'LOCK operator git-removal-refused=yes single-force-refused=yes',
  'LOCK put-back wrote=yes line=yes',
  'LOCK installed-lock-kept-the-registration=true',
  'LOCK release refused-with-another-clones-proof=yes',
  'LOCK released=yes registration-still-stands=true',
  'LOCK unlock-control-lost-the-registration=true',
  'LOCK done',
] as const;

//endregion Constants

//region Arguments

/**
 The log directory a run keeps its logs in, which this package's build folder holds unless the command line names one.

 @returns the directory this package's build folder holds for this driver's logs

 @example
 ```ts
 defaultLogDirectory(); // <package>/target/lock-guest-logs
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
 Make the file-backed pool the arm's datasets live in, and the folder its image stands in.

 The pool is made without a mountpoint so the guest's own root keeps no mount from it; the arm's checkout dataset
 carries its own.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError when the guest could not run the script, or the pool refused it

 @example
 ```ts
 await pool({ chosen });
 ```
 */
async function pool({ chosen, }: { readonly chosen: GuestFacts; },): Promise<void> {
  /**
   The script the guest runs: the pool's image, then the pool itself when it is not already there.
   */
  const script = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}`,
    `mkdir --parents "$(dirname ${POOL_IMAGE})"`,
    `if ! zpool list -H -o name | grep --quiet '^${POOL}$'; then truncate --size 2G ${POOL_IMAGE}; zpool create -f -O mountpoint=none ${POOL} ${POOL_IMAGE}; fi`,
    `zpool list -H -o name ${POOL}`,
  ].join('\n',);
  await guest({
    chosen,
    script,
    log: 'guest-pool.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS,
  },);
}

/**
 Build the test target inside the guest and run the managed-lock arm there.

 Only the target the arm starts is built: the lib test's compile is the guest's largest memory peak and nothing this
 arm runs reads it.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError when a step refused, or when the pulled arm log lacks a line or holds a skip

 @example
 ```ts
 await run({ chosen });
 ```
 */
async function run({ chosen, }: { readonly chosen: GuestFacts; },): Promise<void> {
  await pushWorktreeSnapshot({ chosen, },);
  /**
   The script the guest runs to unpack and build the test target.
   */
  const build = [
    'set -eu',
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'df --human-readable / /mnt/shared',
    'free',
    `if ! swapon --show | grep --quiet /swapfile; then fallocate --length ${SWAP_GIBIBYTES} /swapfile; chmod 600 /swapfile; mkswap /swapfile; swapon /swapfile; fi`,
    'free',
    // One job and a swap file, as the settling-command, removal-lineage and mount-git-worktree drivers do: cargo's
    // default of one job per vCPU ran four rustcs here, the guest's own `AnonPages` reached 7,701,208 kB of its
    // 7,755 MiB with `MemAvailable` at 63,928 kB, and the kernel killed the lib test's rustc at 6,668,028 kB of
    // anonymous memory (measured 2026-11-27 in this driver's own guest, issue 6611). At one job the same reading of
    // the guest's `/proc/meminfo` peaks at 7,019,956 kB, one rustc of this crate's tests, with `MemAvailable` at
    // 625,704 kB at its lowest, so the bound rests on that figure and the swap file on the linker beside it.
    // Only the target the arm runs is built (issue 6612): the lib test's compile was the peak and the arm never reads
    // it.
    'if cargo build --release --test native_binary --locked --features test-only-switches --jobs 1 > /root/lock-build.log 2>&1; then status=0; else status=$?; fi',
    // A build the guest's killer ends leaves no compile error to read, so the log is printed here and the status is
    // the script's own: `set -eu` alone would end the script at the killed build and keep the reason in the guest.
    'grep --extended-regexp "^(error|warning)" --after-context=6 /root/lock-build.log | tail --lines=80 || true',
    'tail --lines=5 /root/lock-build.log',
    'exit $status',
  ].join('\n',);
  await guest({
    chosen,
    script: build,
    log: 'guest-crate-build.log',
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS,
  },);
  await pool({ chosen, },);
  /**
   The script the guest runs to perform the arm and refuse a log that lacks a claim it made.
   */
  const arm = [
    'set -eu',
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export ${GUEST_VARIABLE}=${chosen.name}`,
    `export CLI_GIT_ZFS_SWEEP_POOL=${POOL}`,
    'export CLI_GIT_ZFS_SWEEP_SBIN=/opt/openzfs-2.4.4/sbin',
    `export CLI_GIT_ZFS_SWEEP_CHECKOUT=${CHECKOUT}`,
    // The arm's own layout call destroys the pool's tree and its mounts first, so no stale mount is left to refuse.
    'if cargo nextest run --release --locked --features test-only-switches --test native_binary --no-capture --status-level all ' +
      `--filterset "test(zfs_managed_lock_guest)" > /root/${ARM_LOG} 2>&1; then status=0; else status=$?; fi`,
    `tail --lines=60 /root/${ARM_LOG}`,
    'test "$status" = 0',
    // A returning arm is not a passing arm, which every guest arm of this package holds.
    `if grep --quiet '^${SKIP_WORD}' /root/${ARM_LOG}; then echo 'the lock arm returned without running'; exit 1; fi`,
    ...EXPECTED_MARKERS.map(function markerLine(marker: string,): string {
      return `grep --quiet --fixed-strings ${shellQuote(marker,)} /root/${ARM_LOG}`;
    },),
    `cp /root/${ARM_LOG} /mnt/shared/${ARM_LOG}`,
  ].join('\n',);
  await guest({
    chosen,
    script: arm,
    log: 'guest-lock-arm.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS,
  },);
  /**
   The arm's own output, as the host pulled it back.
   */
  const pulled = await pullGuestLog({
    chosen,
    file: ARM_LOG,
    log: 'host-pull-lock-arm.log',
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
    throw new GuestDriverError('the pulled arm log holds a SKIP line, so the lock arm did not run',);
  }
}

/**
 Take the guest away, or leave it for a person to inspect when the run asked for that.

 A kept guest is named in the line this prints, so whoever reads the run knows the name to destroy.

 @param chosen - the run's options

 @throws GuestDriverError when the recipe's destruction refused

 @example
 ```ts
 await destroy({ chosen });
 ```
 */
async function destroy({
  chosen,
  keepGuest,
}: {
  readonly chosen: GuestFacts;
  readonly keepGuest: boolean;
},): Promise<void> {
  if (keepGuest) {
    console.log(`git-policy lock guest: --keep-guest kept '${chosen.name}' to inspect and destroy`,);
    return;
  }
  await recipe({
    chosen,
    step: 'destroy',
  },);
}

//endregion Steps

/**
 Build the test target in the guest and run the managed-lock arm there.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError when a step refused, or when the pulled arm log lacks a line

 @example
 ```ts
 await runTheArm({ chosen });
 ```
 */
async function runTheArm({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await run({ chosen, },);
}

//region The whole run

/**
 Create, prepare, run the arm and destroy, in that order, destroying even when a step refused.

 @param chosen - the run's options

 @throws GuestDriverError naming the step that refused, after the guest it created is gone

 @example
 ```ts
 await all({ chosen });
 ```
 */
async function all({ chosen, }: { readonly chosen: GuestDriverOptions; },): Promise<void> {
  await runWholeGuestRun({
    label: 'git-policy lock guest',
    chosen,
    keepGuest: chosen.keepGuest,
    steps: {
      create: recipeStepOf({ step: 'create', },),
      prepare: recipeStepOf({ step: 'prepare', },),
      work: runTheArm,
      destroy: async function destroyTheGuest({ chosen: facts }: { readonly chosen: GuestFacts }) {
        await destroy({
          chosen: facts,
          keepGuest: chosen.keepGuest,
        },);
      },
    },
  },);
}

//endregion The whole run

//region The command line

/**
 The steps by name, and the whole run.
 */
const steps: ReadonlyMap<string, (chosen: GuestDriverOptions,) => Promise<void>> = new Map([
  [
    'create',
    async function createStep(chosen: GuestDriverOptions,) {
      await recipe({
        chosen,
        step: 'create',
      },);
    },
  ],
  [
    'prepare',
    async function prepareStep(chosen: GuestDriverOptions,) {
      await recipe({
        chosen,
        step: 'prepare',
      },);
    },
  ],
  [
    'pool',
    async function poolStep(chosen: GuestDriverOptions,) {
      await pool({ chosen, },);
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
      await destroy({
        chosen,
        keepGuest: chosen.keepGuest,
      },);
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
  defaultLogDir: defaultLogDirectory(),
},);

await mkdir(
  chosen.logDir,
  {
    recursive: true,
    mode: FOLDER_MODE,
  },
);

/**
 The step this run performs.
 */
const step = steps.get(chosen.step,);
if (step === undefined) {
  throw new GuestDriverError(`unknown step '${chosen.step}'; use ${[...steps.keys(),].join(', ',)}`,);
}
// The steps that run an arm in the guest refuse on a backend this package's arm scripts cannot reach, before this run
// makes anything. The three that only make, prepare and take away a guest run on either backend.
refuseAStepWhoseArmsCannotRun({
  backend: guestBackend(),
  step: chosen.step,
  stepsThatRunNoArm: STEPS_THAT_RUN_NO_ARM,
});

await step(chosen,);
console.log(`git-policy lock guest: step '${chosen.step}' finished; logs in ${chosen.logDir}`,);

//endregion The command line
