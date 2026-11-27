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
} from '../bin/mvm-guest-arm-log.ts';
import {
  flagGiven,
  flagValue,
  shellQuote,
} from '../bin/mvm-guest-command-line.ts';
import { GuestDriverError, } from '../bin/mvm-guest-error.ts';
import { runGuestScript, } from '../bin/mvm-guest-script.ts';
import { runGuestRecipeStep, } from '../bin/mvm-guest-recipe-step.ts';
import {
  readStepLog,
  runRequiredStepCommand,
} from '../bin/mvm-guest-step-command.ts';
import { runWholeGuestRun, } from '../bin/mvm-guest-whole-run.ts';

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
 Milliseconds one host command may take.
 */
const HOST_COMMAND_TIMEOUT_MS = 120_000;

/**
 Milliseconds the guest's build of the test target may take.
 */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;

/**
 Milliseconds the arm may take.
 */
const ARM_TIMEOUT_MS = 1_800_000;

/**
 Seconds the guest's own `timeout` fires before `mvm`'s kill, so the guest reports a status rather than dying with it.
 */
const GUEST_TIMEOUT_MARGIN_SECONDS = 60;

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
 The step, the guest name, the log directory and whether `all` keeps the guest.
 */
type Options = {
  readonly step: string;
  readonly name: string;
  readonly logDirectory: string;
  readonly keepGuest: boolean;
};

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

/**
 The step and the values the command line gave it.

 @returns the step, the guest's name, the log directory and whether the guest is kept

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
   The step this run performs, which the first word names.
   */
  const step = argumentLine[0] ?? '';
  /**
   The words after the step, which name the guest, its log directory and whether it is kept.
   */
  const argumentWords = argumentLine.slice(1,);
  return {
    step,
    name: flagValue({
      argumentWords,
      flag: '--name',
      fallback: DEFAULT_GUEST_NAME,
    },),
    logDirectory: flagValue({
      argumentWords,
      flag: '--log-dir',
      fallback: defaultLogDirectory(),
    },),
    keepGuest: flagGiven({
      argumentWords,
      flag: '--keep-guest',
    },),
  };
}

//endregion Arguments

//region Steps

/**
 Make the file-backed pool the arm's datasets live in, and the folder its image stands in.

 The pool is made without a mountpoint so the guest's own root keeps no mount from it; the arm's checkout dataset
 carries its own.

 @param chosen - the run's options

 @throws GuestDriverError when the guest could not run the script, or the pool refused it

 @example
 ```ts
 await pool({ chosen });
 ```
 */
async function pool({ chosen, }: { readonly chosen: Options; },): Promise<void> {
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
  await runGuestScript({
    logDirectory: chosen.logDirectory,
    name: chosen.name,
    script,
    log: 'guest-pool.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS,
    guestTimeoutMarginSeconds: GUEST_TIMEOUT_MARGIN_SECONDS,
  },);
}

/**
 Build the test target inside the guest and run the managed-lock arm there.

 @param chosen - the run's options

 @throws GuestDriverError when a step refused, or when the pulled arm log lacks a line or holds a skip

 @example
 ```ts
 await run({ chosen });
 ```
 */
async function run({ chosen, }: { readonly chosen: Options; },): Promise<void> {
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
    'cargo build --release --tests --locked --features test-only-switches > /root/lock-build.log 2>&1',
    'tail --lines=3 /root/lock-build.log',
  ].join('\n',);
  await runGuestScript({
    logDirectory: chosen.logDirectory,
    name: chosen.name,
    script: build,
    log: 'guest-crate-build.log',
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS,
    guestTimeoutMarginSeconds: GUEST_TIMEOUT_MARGIN_SECONDS,
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
    ...EXPECTED_MARKERS.map(function markerLine(marker) {
      return `grep --quiet --fixed-strings ${shellQuote({ script: marker, },)} /root/${ARM_LOG}`;
    },),
    `cp /root/${ARM_LOG} /mnt/shared/${ARM_LOG}`,
  ].join('\n',);
  await runGuestScript({
    logDirectory: chosen.logDirectory,
    name: chosen.name,
    script: arm,
    log: 'guest-lock-arm.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS,
    guestTimeoutMarginSeconds: GUEST_TIMEOUT_MARGIN_SECONDS,
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
    log: 'host-pull-lock-arm.log',
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
async function destroy({ chosen, }: { readonly chosen: Options; },): Promise<void> {
  if (chosen.keepGuest) {
    console.log(`git-policy lock guest: --keep-guest kept '${chosen.name}' to inspect and destroy`,);
    return;
  }
  await runGuestRecipeStep({
    name: chosen.name,
    logDirectory: chosen.logDirectory,
    step: 'destroy',
  },);
}

//endregion Steps

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
async function all({ chosen, }: { readonly chosen: Options; },): Promise<void> {
  await runWholeGuestRun({
    label: 'git-policy lock guest',
    chosen,
    steps: {
      create: async function createTheGuest() {
        await recipeStepOf({ step: 'create', },)(chosen,);
      },
      prepare: async function prepareTheGuest() {
        await recipeStepOf({ step: 'prepare', },)(chosen,);
      },
      work: async function runTheArm() {
        await run({ chosen, },);
      },
      destroy: async function destroyTheGuest() {
        await destroy({ chosen, },);
      },
    },
  },);
}

//endregion The whole run

//region The command line

/**
 One step of the ZFS binding's guest recipe, as a step of this driver's own command line.

 @param step - the recipe's step

 @returns a step that hands the recipe the guest's name and log directory

 @example
 ```ts
 const steps = { create: recipeStepOf({ step: 'create' }) };
 ```
 */
function recipeStepOf({ step, }: { readonly step: string; },): (chosen: Options) => Promise<void> {
  return async function recipeStep(chosen) {
    await runGuestRecipeStep({
      name: chosen.name,
      logDirectory: chosen.logDirectory,
      step,
    },);
  };
}

/**
 The steps by name, and the whole run.
 */
const steps: Record<string, (chosen: Options) => Promise<void>> = {
  create: recipeStepOf({ step: 'create', },),
  prepare: recipeStepOf({ step: 'prepare', },),
  pool: async function poolStep(chosen) {
    await pool({ chosen, },);
  },
  run: async function runStep(chosen) {
    await run({ chosen, },);
  },
  destroy: async function destroyStep(chosen) {
    await destroy({ chosen, },);
  },
  all: async function allStep(chosen) {
    await all({ chosen, },);
  },
};

/**
 The options this run was given.
 */
const chosen = options();

await mkdir(
  chosen.logDirectory,
  {
    recursive: true,
    mode: 0o700,
  },
);

/**
 The step this run performs, or nothing when the command line named one this driver does not know.
 */
const step = steps[chosen.step];
if (step === undefined) {
  throw new GuestDriverError(`unknown step '${chosen.step}'; use ${Object.keys(steps,)
    .join(', ',)}`,);
}
await step(chosen,);
console.log(`git-policy lock guest: step '${chosen.step}' finished; logs in ${chosen.logDirectory}`,);

//endregion The command line
