#!/usr/bin/env node
/**
 Drive plan step 15's guest arm in a throwaway `mvm` guest: the deferred-origin check against a real pool.

 The check reads `/dev/zfs`, so no control on a person's machine may run it against a pool it did not build. This
 driver borrows the ZFS binding's guest recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and prepares it
 with OpenZFS 2.4.4 and the Rust toolchain, then builds this crate inside it, runs the arm, and destroys the guest.

 The four arms, as the design's "Measurements owed" asks (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
 M1 and control 28):
 - the planned case: an unmarked origin goes on, and the mark reads `0` through the same read the wrapper uses;
 - a marked origin: `zfs destroy -d` marks the clone's origin snapshot, the mark reads `1`, and the check refuses;
 - the reap: destroying the clone's own snapshots and then the head takes the marked snapshot in the same transaction,
   which is the loss the refusal avoids, so an arm that cannot see it cannot show the refusal matters;
 - a moved origin: `zfs promote` moves the lineage under the tuple the removal recorded, and the check refuses on an
   identity.

 Usage: node e2e/deferred-origin-check-guest.ts <create|prepare|run|destroy|all> [--name <guest>] [--log-dir <dir>]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `deferred-origin-arm.log`, pulled back into the log directory, and a run passes only when it
 carries every `DEFERRED` line below and no `SKIPPED` line, so an arm that returned without running fails the run
 instead of reporting a green suite.
 */
import {
  mkdir,
  readFile,
  rm,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { copyWrapperSnapshot, } from '../bin/native-wrapper-snapshot.ts';
import {
  runGuestScript,
  runHostCommand,
  runRecipeStep,
} from '../bin/guest-route-commands.ts';
import {
  type GuestRouteOptions,
  guestRouteOptions,
} from '../bin/guest-route-options.ts';
import { FOLDER_MODE, } from '../bin/guest-route-permissions.ts';
import { hasLineStartingWith, } from '../bin/guest-route-log-lines.ts';
import { shellQuote, } from '../bin/guest-route-shell-quote.ts';

/**
 Where the wrapper's tree is unpacked and built inside the guest, which every account may traverse.
 */
const CRATE_DIR = '/opt/deferred-origin-check';
/**
 Milliseconds the guest's build of the library and its tests may take.
 */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;
/**
 Milliseconds the guest arm may take.
 */
const ARM_TIMEOUT_MS = 900_000;

/**
 The environment the arm reads, so it never opens the host's device.
 */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_DEFERRED_ORIGIN_GUEST';
/**
 The pool the arm builds and destroys.
 */
const POOL = 'zfsstep15';
/**
 The folder the pool's image and the fixtures live under.
 */
const FIXTURE_ROOT = '/home/zfs-step-15-fixture';
/**
 The OpenZFS 2.4.4 prefix the guest's recipe installs.
 */
const OPENZFS_PREFIX = '/opt/openzfs-2.4.4';
/**
 The `PATH` the guest's OpenZFS userspace needs.
 */
const OPENZFS_PATH = `${OPENZFS_PREFIX}/sbin:${OPENZFS_PREFIX}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;

/**
 The lines a run must find in the guest's arm log, one per arm it performed.
 */
const EXPECTED_MARKERS = [
  'DEFERRED planned=proceed mark=0',
  'DEFERRED marked counts clones=1 holds=Counted(0)',
  'DEFERRED marked=refused step=origin-mark',
  'DEFERRED reaped=gone origin=',
  'DEFERRED promoted=refused',
  'DEFERRED done',
] as const;

/**
 Create the guest, refusing a name another run holds.

 @param chosen - the run's options
 */
async function create(chosen: GuestRouteOptions,): Promise<void> {
  await runRecipeStep({
    logDir: chosen.logDir,
    name: chosen.name,
    step: 'create',
  },);
}

/**
 Install OpenZFS 2.4.4, the Rust toolchain and `cargo-nextest` in the guest.

 @param chosen - the run's options
 */
async function prepare(chosen: GuestRouteOptions,): Promise<void> {
  await runRecipeStep({
    logDir: chosen.logDir,
    name: chosen.name,
    step: 'prepare',
  },);
}

/**
 Push a repository-shaped snapshot of the wrapper, build it in the guest, and run the arm.

 @param chosen - the run's options

 @throws Error naming the step that refused, or the marker the pulled log lacks
 */
async function run(chosen: GuestRouteOptions,): Promise<void> {
  /**
   Where the snapshot of this worktree is unpacked before it is pushed.
   */
  const context = join(
    chosen.logDir,
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
      mode: FOLDER_MODE,
    },
  );
  await copyWrapperSnapshot({ context, },);
  /**
   The archive the guest unpacks, which nothing this run builds stays in.
   */
  const tarball = join(
    chosen.logDir,
    'subject.tar.gz',
  );
  await runHostCommand({
    logDir: chosen.logDir,
    command: 'tar',
    args: [
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
  },);
  await runHostCommand({
    logDir: chosen.logDir,
    command: 'mvm',
    args: [
      '--backend',
      'libvirt',
      'push',
      chosen.name,
      tarball,
      '/root/subject.tar.gz',
    ],
    log: 'guest-push.log',
  },);
  /**
   The guest's own script: unpack the archive and build the crate's library and tests.
   */
  const build = [
    'set -eu',
    'if test -f /root/subject.tar.gz; then :; else cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'cargo build --release --tests --locked --features test-only-switches > /root/deferred-origin-build.log 2>&1',
    'tail --lines=3 /root/deferred-origin-build.log',
  ]
    .join('\n',);
  await runGuestScript({
    logDir: chosen.logDir,
    name: chosen.name,
    script: build,
    log: 'guest-crate-build.log',
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS,
  },);
  /**
   The guest's own script: run the arm, refuse a returned one, and copy its log to the shared folder.
   */
  const arm = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}:/root/.cargo/bin:$PATH`,
    `export ${GUEST_VARIABLE}=${chosen.name}`,
    `export CLI_GIT_ZFS_DEFERRED_ORIGIN_POOL=${POOL}`,
    `export CLI_GIT_ZFS_DEFERRED_ORIGIN_PREFIX=${OPENZFS_PREFIX}`,
    `export CLI_GIT_ZFS_DEFERRED_ORIGIN_ROOT=${FIXTURE_ROOT}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'if cargo nextest run --release --locked --features test-only-switches --lib --run-ignored all --no-capture --status-level all --filterset "test(the_deferred_origin_guest_arms)" > /root/deferred-origin-arm.log 2>&1; then status=0; else status=$?; fi',
    'tail --lines=40 /root/deferred-origin-arm.log',
    'test "$status" = 0',
    // A returning arm is not a passing arm, which the binding's own live recipe also holds.
    `if grep --quiet '^SKIPPED' /root/deferred-origin-arm.log; then echo 'the arm returned without running'; exit 1; fi`,
    ...EXPECTED_MARKERS
      .map(function markerLine(marker: string,): string {
        return `grep --quiet --fixed-strings ${shellQuote(marker,)} /root/deferred-origin-arm.log`;
      },),
    'cp /root/deferred-origin-arm.log /mnt/shared/deferred-origin-arm.log',
    // The pool goes even when the arm left it behind, so the guest is ready for another run.
    `zpool destroy -f ${POOL} 2>/dev/null || true`,
    `rm --force ${FIXTURE_ROOT}/pool.img 2>/dev/null || true`,
  ]
    .join('\n',);
  await runGuestScript({
    logDir: chosen.logDir,
    name: chosen.name,
    script: arm,
    log: 'guest-deferred-origin.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS,
  },);
  /**
   The arm's own log, pulled back out of the guest.
   */
  const pulledPath = join(
    chosen.logDir,
    'deferred-origin-arm.log',
  );
  await runHostCommand({
    logDir: chosen.logDir,
    command: 'mvm',
    args: [
      '--backend',
      'libvirt',
      'pull',
      chosen.name,
      '/mnt/shared/deferred-origin-arm.log',
      pulledPath,
    ],
    log: 'host-pull-arm.log',
  },);
  /**
   Everything the arm printed once it was pulled back.
   */
  const pulled = await readFile(
    pulledPath,
    'utf8',
  );
  for (const marker of EXPECTED_MARKERS) {
    if (!pulled
      .includes(marker,))
      throw new Error(`the pulled arm log lacks ${marker}; the guest ran the arm without performing it`,);
  }
  if (hasLineStartingWith({
    text: pulled,
    prefix: 'SKIPPED',
  },))
    throw new Error('the pulled arm log holds a SKIPPED line, so the arm did not run',);
}

/**
 Destroy the guest and keep the list of what is left.

 @param chosen - the run's options
 */
async function destroy(chosen: GuestRouteOptions,): Promise<void> {
  await runRecipeStep({
    logDir: chosen.logDir,
    name: chosen.name,
    step: 'destroy',
  },);
}

/**
 Create, prepare, run and destroy, in that order.

 @param chosen - the run's options

 @throws Error naming the step that refused
 */
async function all(chosen: GuestRouteOptions,): Promise<void> {
  await create(chosen,);
  await prepare(chosen,);
  await run(chosen,);
  await destroy(chosen,);
}

/**
 The steps this driver knows, and nothing else.
 */
const steps: Record<string, (chosen: GuestRouteOptions,) => Promise<void>> = {
  create,
  prepare,
  run,
  destroy,
  all,
};

/**
 The step and the values the command line gave this run.
 */
const chosen = guestRouteOptions({
  argv: process.argv,
  defaultName: 'deferred-origin-check',
  defaultLogDir: resolve(
    import.meta.dirname,
    '..',
    'target',
    'deferred-origin-guest-logs',
  ),
},);

/**
 The step the command line names.
 */
const step = steps[chosen.step];

if (step === undefined)
  throw new Error(`unknown step '${chosen.step}'; use create, prepare, run, destroy or all`,);
await mkdir(
  chosen.logDir,
  {
    recursive: true,
    mode: FOLDER_MODE,
  },
);
await step(chosen,);
