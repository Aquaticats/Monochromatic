#!/usr/bin/env node
/**
 Drive the checkout-propagation step's guest arm in a throwaway `mvm` guest: a real checkout dataset made shared, shared
 and slave, private, slave and unbindable, and one that is not the checkout's, each driven through the step as root, and
 then the accepted design's measurement of the provisional systemd trigger.

 The step changes a mount and reads the kernel's own answer about it, and the trigger half installs a unit and asks a
 manager for a reload, so no control on a person's machine may run either. This driver borrows the ZFS binding's guest
 recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and prepares it
 with OpenZFS 2.4.4 and the Rust toolchain, then builds this crate's library and its tests inside it, runs the arm, and
 destroys the guest.

 The arm itself is `src/native/checkout_propagation/propagation_guest_arms_tests.rs`, and it calls the step directly
 because no root mode reaches it in this release: its call at the head of `boot-reconcile` and of the default sweep is
 plan step 19's wiring, which is not built. The gate the trigger asks to reload is a stand-in for the same reason, and
 its `ExecReload=` appends one line, which is how the arm counts the reloads.

 Usage: node e2e/propagation-step-guest.ts <create|prepare|build|run|destroy|all> [--name <guest>] [--log-dir <dir>]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `propagation-arm.log`, pulled back into the log directory, and a run passes only when it
 carries every `PROPAGATION` line below and no `SKIPPED` line, so an arm that returned without running fails the run
 instead of reporting a green suite.
 */
import { mkdir, } from 'node:fs/promises';
import { resolve, } from 'node:path';

import { detachedStep, } from './adoption-guest-arms/detached-step.ts';
import { requireMarkers, } from '../bin/mvm-guest-arm-log.ts';
import {
  crateBuildTimeoutMilliseconds,
  buildFinishMarginMilliseconds,
  armTimeoutMilliseconds,
  finishMarginMilliseconds,
} from './adoption-guest-arms/adoption-fixture.ts';
import type {
  GuestFacts,
} from '../bin/mvm-guest-guest-facts.ts';
import {
  guestDriverOptions,
  type GuestDriverOptions,
} from '../bin/mvm-guest-options.ts';
import {
  FOLDER_MODE,
  HOST_COMMAND_TIMEOUT_MS,
} from '../bin/mvm-guest-bounds.ts';
import {
  pushWorktreeSnapshot,
  recipe,
} from '../bin/mvm-guest-host-command.ts';
import { pullGuestLog, } from '../bin/mvm-guest-detached-script.ts';
import { runQuietStepCommand, } from '../bin/mvm-guest-step-command.ts';
import { runWholeGuestRun, } from '../bin/mvm-guest-whole-run.ts';

/**
 Where the crate is unpacked and built inside the guest, which every account may traverse.
 */
const CRATE_DIR = '/opt/git-policy-cli-propagation';
/**
 The `PATH` the guest's OpenZFS 2.4.4 userspace needs.
 */
const OPENZFS_PATH = '/opt/openzfs-2.4.4/sbin:/opt/openzfs-2.4.4/bin:/sbin:/usr/sbin:/bin:/usr/bin';
/**
 The environment the arm reads, so it never runs anywhere but this guest.
 */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_PROPAGATION_GUEST';
/**
 The pool the arm makes and destroys.
 */
const POOL = 'propagation';
/**
 The checkout path the arm mounts its dataset at.
 */
const CHECKOUT = '/root/propagation-checkout';
/**
 The OpenZFS prefix the guest's recipe installs.
 */
const OPENZFS_PREFIX = '/opt/openzfs-2.4.4';
/**
 The name of the arm's own log inside the guest's shared folder.
 */
const ARM_LOG = 'propagation-arm.log';

/**
 Remove one log an earlier run left in the guest's shared folder.

 A detached script writes its status line into a log that outlives it, and the host reads the last line to learn that
 the script ended: a stale line from an earlier attempt ends the wait for a run that has just started, which happened in
 this round on a guest kept across four build attempts.

 @param chosen - the run's guest

 @param log - the log's name inside the guest's shared folder

 @example
 ```ts
 await clearTheOldLog({ chosen, log: 'guest-crate-build.log' });
 ```
 */
async function clearTheOldLog({
  chosen,
  log,
}: {
  readonly chosen: GuestFacts;
  readonly log: string;
},): Promise<void> {
  await runQuietStepCommand({
    program: 'mvm',
    argumentWords: [
      '--backend',
      'libvirt',
      'exec',
      chosen.name,
      '--',
      'rm',
      '--force',
      `/mnt/shared/${log}`,
    ],
    timeoutMilliseconds: HOST_COMMAND_TIMEOUT_MS,
  },);
}

/**
 How many times the package's own build bound this driver's build gets.

 A fresh extract changes every path dependency's timestamp, so cargo recompiles the workspace crates too: one build took
 40m21s in a guest on a host running ten guests, and this round's third attempt was still compiling when the doubled
 bound ended it.

 @example
 ```ts
 const bound = crateBuildTimeoutMilliseconds * BUILD_BOUNDS;
 ```
 */
const BUILD_BOUNDS = 3;

/**
 The lines a run must find in the guest's arm log, one per phase it performed.

 The five states and the refusal are the step's own; the `trigger` line carries the measurement's readings, whose two
 rules are read in the round's report.
 */
const EXPECTED_MARKERS = [
  'PROPAGATION fixture guest=',
  'PROPAGATION state=shared',
  'PROPAGATION state=private',
  'PROPAGATION state=unbindable',
  'PROPAGATION state=shared+slave',
  'PROPAGATION refused=source',
  'PROPAGATION trigger arming=',
  'PROPAGATION done',
] as const;

/**
 Create the guest, refusing a name another run holds.

 @param chosen - the run's options
 */
async function create(chosen: GuestFacts,): Promise<void> {
  await recipe({
    chosen,
    step: 'create',
  },);
}

/**
 Install OpenZFS 2.4.4, the Rust toolchain and `cargo-nextest` in the guest.

 @param chosen - the run's options
 */
async function prepare(chosen: GuestFacts,): Promise<void> {
  await recipe({
    chosen,
    step: 'prepare',
  },);
}

/**
 Push this checkout into the guest and build the crate's library and its tests there.

 @param chosen - the run's options

 @throws Error naming the step that refused
 */
async function build(chosen: GuestFacts,): Promise<void> {
  await pushWorktreeSnapshot({ chosen, },);
  /**
   The guest's whole build: unpack the archive, add the swap the guest's recipes give a build, and compile the tests.
   */
  const script = [
    'set -eu',
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    // The dependency build is the guest's heaviest work and outlives one run, so it is kept out of the tree this run
    // replaces: a second run then compiles this crate's own test binary alone.
    `if test -d ${CRATE_DIR}/package/git-policy/cli/target; then rm --recursive --force /root/target-kept; mv ${CRATE_DIR}/package/git-policy/cli/target /root/target-kept; fi`,
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `if test -d /root/target-kept; then mv /root/target-kept ${CRATE_DIR}/package/git-policy/cli/target; fi`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'if ! swapon --show | grep --quiet /swapfile; then fallocate --length 6G /swapfile; chmod 600 /swapfile; mkswap /swapfile; swapon /swapfile; fi',
    // Two jobs: the guest's 8 GiB and its 6 GiB swap hold two `rustc` processes, and one job outlasted the 2,400 s
    // bound this package's other drivers give a build on a host running ten guests (measured in this round).
    'CARGO_BUILD_JOBS=2 cargo build --release --tests --locked --features test-only-switches > /root/propagation-build.log 2>&1',
    'tail --lines=3 /root/propagation-build.log',
  ].join('\n',);
  await clearTheOldLog({
    chosen,
    log: 'guest-crate-build.log',
  },);
  await detachedStep({
    chosen,
    script,
    log: 'guest-crate-build.log',
    // Three times the bound the package's other drivers give a build: measured in this round, a fresh extract changes
    // every path-dependency's timestamp, so cargo recompiles the workspace crates too, and one build took 40m21s in a
    // guest on a host running ten guests.
    timeoutMilliseconds: crateBuildTimeoutMilliseconds * BUILD_BOUNDS,
    finishTimeoutMilliseconds: (crateBuildTimeoutMilliseconds * BUILD_BOUNDS) + buildFinishMarginMilliseconds,
  },);
}

/**
 Run the arm in the guest, refuse a log that only returned, and pull its evidence back.

 @param chosen - the run's options

 @throws Error naming the marker the pulled log lacks
 */
async function run(chosen: GuestFacts,): Promise<void> {
  /**
   The arm itself, which needs the pool, the checkout and the OpenZFS tools on its `PATH`.
   */
  const script = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}:/root/.cargo/bin:$PATH`,
    `export ${GUEST_VARIABLE}=${chosen.name}`,
    `export CLI_GIT_ZFS_PROPAGATION_POOL=${POOL}`,
    `export CLI_GIT_ZFS_PROPAGATION_CHECKOUT=${CHECKOUT}`,
    `export CLI_GIT_ZFS_PROPAGATION_PREFIX=${OPENZFS_PREFIX}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    // The arm makes its own pool, so a run that met one left by an earlier run takes it down first.
    `zpool destroy -f ${POOL} 2>/dev/null || true`,
    'if cargo nextest run --release --locked --features test-only-switches --lib --run-ignored all --no-capture --status-level all '
      + `--filterset "test(the_propagation_guest_arm_tests)" > /root/${ARM_LOG} 2>&1; then status=0; else status=$?; fi`,
    `tail --lines=80 /root/${ARM_LOG}`,
    'test "$status" = 0',
    // A returning arm is not a passing arm, which the package's other live recipes also hold.
    `if grep --quiet '^SKIPPED' /root/${ARM_LOG}; then echo 'the arm returned without running'; exit 1; fi`,
    `cp /root/${ARM_LOG} /mnt/shared/${ARM_LOG}`,
    'cp /root/propagation-gate.log /mnt/shared/propagation-gate.log 2>/dev/null || true',
  ].join('\n',);
  await clearTheOldLog({
    chosen,
    log: 'guest-propagation-arm.log',
  },);
  await detachedStep({
    chosen,
    script,
    log: 'guest-propagation-arm.log',
    timeoutMilliseconds: armTimeoutMilliseconds,
    finishTimeoutMilliseconds: armTimeoutMilliseconds + finishMarginMilliseconds,
  },);
  /**
   The arm's own log, which the step copied into the guest's shared folder.
   */
  const pulled = await pullGuestLog({
    chosen,
    file: ARM_LOG,
    log: 'host-pull-propagation-arm.log',
  },);
  requireMarkers({
    text: pulled,
    markers: EXPECTED_MARKERS,
    subject: 'the pulled propagation-arm log',
    because: 'the guest ran the arm without performing it',
  },);
}

/**
 Destroy the guest.

 @param chosen - the run's options
 */
async function destroy(chosen: GuestFacts,): Promise<void> {
  await recipe({
    chosen,
    step: 'destroy',
  },);
}

/**
 Create, prepare, build, run and destroy, destroying even when a step refused.

 @param chosen - the run's options

 @throws Error naming the step that refused, after the guest it created is gone
 */
async function all(chosen: GuestDriverOptions,): Promise<void> {
  await runWholeGuestRun({
    label: 'git-policy propagation guest',
    chosen,
    keepGuest: chosen.keepGuest,
    steps: {
      create: createTheGuest,
      prepare: prepareTheGuest,
      work: buildAndRun,
      destroy: destroyTheGuest,
    },
  },);
}

/**
 The guest's creation, as `all` hands it to the shared whole-run helper.

 @param chosen - the guest's facts

 @throws Error naming the recipe step and its status
 */
async function createTheGuest({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await create(chosen,);
}

/**
 The guest's preparation, as `all` hands it to the shared whole-run helper.

 @param chosen - the guest's facts

 @throws Error naming the recipe step and its status
 */
async function prepareTheGuest({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await prepare(chosen,);
}

/**
 The guest's destruction, as `all` hands it to the shared whole-run helper.

 @param chosen - the guest's facts

 @throws Error naming the recipe step and its status
 */
async function destroyTheGuest({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await destroy(chosen,);
}

/**
 Build the crate in the guest and run the arm, which is what `all` does between the guest's own steps.

 @param chosen - the guest's facts

 @throws Error naming the step that refused, or the marker the pulled log lacks
 */
async function buildAndRun({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await build(chosen,);
  await run(chosen,);
}

/**
 The steps this driver knows, and nothing else.
 */
const steps: Record<string, (chosen: GuestDriverOptions,) => Promise<void>> = {
  create,
  prepare,
  build,
  run,
  destroy,
  all,
};

/**
 The step and the values the command line gave this run.
 */
const chosen = guestDriverOptions({
  argv: process.argv
    .slice(2,),
  defaultName: 'zfs-r6-propagation',
  defaultLogDir: resolve(
    import.meta.dirname,
    '..',
    'target',
    'propagation-guest-logs',
  ),
},);

/**
 The step the command line names.
 */
const step = steps[chosen.step];

if (step === undefined)
  throw new Error(`unknown step '${chosen.step}'; use create, prepare, build, run, destroy or all`,);
await mkdir(
  chosen.logDir,
  {
    recursive: true,
    mode: FOLDER_MODE,
  },
);
await step(chosen,);
