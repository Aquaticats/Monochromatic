#!/usr/bin/env node
/**
 Drive plan step 18's `doctor` checks for steps 2 to 9 and its removal-side lines in a throwaway `mvm` guest.

 The checks read `/dev/zfs`, a real pool's dataset properties and the kernel's delegation reply, so only a guest with a
 file-backed pool can show them. This driver owns the guest through the ZFS binding's own recipe
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest, prepares it with
 OpenZFS 2.4.4, Rust and cargo-nextest, builds this crate's binaries and tests inside it, runs the arm, and destroys the
 guest. The arm itself creates the design's layout, the store, the installed files and the records, asserts every group
 passes, then breaks one fact at a time and asserts the finding.

 Every line a run must find is printed by the arm as `DOCTOR-GUEST <claim>`. `create` refuses a name `mvm list` already
 reports, and `destroy` removes only a guest this driver created.

 Usage: node e2e/doctor-zfs-route-guest.ts <create|prepare|build|arm|destroy|all> [--name <guest>] [--log-dir <dir>]
          [--keep-guest]
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
  guestBackend,
} from '../bin/mvm-guest-backend.ts';
import { ensureTheGuestSharedFolder, } from '../bin/mvm-guest-shared-folder.ts';
import { copyWrapperSnapshot, } from '../bin/native-wrapper-snapshot.ts';
import {
  host,
  recipe,
} from '../bin/mvm-guest-host-command.ts';
import {
  guest,
} from '../bin/mvm-guest-script.ts';
import {
  guestDriverOptions,
  type GuestDriverOptions,
} from '../bin/mvm-guest-options.ts';
import {
  FOLDER_MODE,
  HOST_COMMAND_TIMEOUT_MS,
} from '../bin/mvm-guest-bounds.ts';
import {
  holdsLineStartingWith,
  missingMarkers,
} from '../bin/mvm-guest-arm-log.ts';
import {
  holdsGuestName,
} from '../bin/mvm-guest-listing.ts';
import {
  captureHostCommand,
} from '../bin/mvm-guest-step-command.ts';

/**
 The backend this run drives, read from the one place every guest step reads it from.
 */
const backend = guestBackend();

//region Constants

/**
 Where the crate is unpacked and built inside the guest.
 */
const CRATE_DIR = '/opt/git-policy-cli-doctor-route';
/**
 The file-backed pool the arm's layout sits on, which only the guest ever sees.
 */
const POOL = 'zpcachyos';
/**
 The variable the arm reads: its value is the guest's name, so a run outside the guest prints SKIP.
 */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_DOCTOR_GUEST';
/**
 The variable the arm reads for the pool's name.
 */
const POOL_VARIABLE = 'CLI_GIT_ZFS_DOCTOR_POOL';
/**
 The variable the removal-side arm reads for its guest.
 */
const REMOVAL_GUEST_VARIABLE = 'CLI_GIT_ZFS_DOCTOR_REMOVAL_GUEST';
/**
 The variable the removal-side arm reads for its pool.
 */
const REMOVAL_POOL_VARIABLE = 'CLI_GIT_ZFS_DOCTOR_REMOVAL_POOL';
/**
 The variable the removal-side arm reads for its OpenZFS prefix.
 */
const REMOVAL_PREFIX_VARIABLE = 'CLI_GIT_ZFS_DOCTOR_REMOVAL_PREFIX';
/**
 The variable the removal-side arm reads for its fixture root.
 */
const REMOVAL_ROOT_VARIABLE = 'CLI_GIT_ZFS_DOCTOR_REMOVAL_ROOT';
/**
 The pool the removal-side arm builds.
 */
const REMOVAL_POOL = 'zfsdoctorremoval';
/**
 The OpenZFS prefix the removal-side arm's commands come from.
 */
const REMOVAL_PREFIX = '/opt/openzfs-2.4.4';
/**
 The folder the removal-side arm's fixture lives under.
 */
const REMOVAL_ROOT = '/home/zfs-doctor-removal-fixture';
/**
 The variable the gatherer's arm reads for its guest.
 */
const CLONE_FACTS_GUEST_VARIABLE = 'CLI_GIT_ZFS_CLONE_FACTS_GUEST';
/**
 The variable the gatherer's arm reads for its pool.
 */
const CLONE_FACTS_POOL_VARIABLE = 'CLI_GIT_ZFS_CLONE_FACTS_POOL';
/**
 The variable the gatherer's arm reads for its OpenZFS prefix.
 */
const CLONE_FACTS_PREFIX_VARIABLE = 'CLI_GIT_ZFS_CLONE_FACTS_PREFIX';
/**
 The variable the gatherer's arm reads for its fixture root.
 */
const CLONE_FACTS_ROOT_VARIABLE = 'CLI_GIT_ZFS_CLONE_FACTS_ROOT';
/**
 The pool the gatherer's six clones are built in.
 */
const CLONE_FACTS_POOL = 'zfsclonefacts';
/**
 The folder the gatherer's arm lives under.
 */
const CLONE_FACTS_ROOT = '/home/zfs-clone-facts-fixture';
/**
 Milliseconds the guest's crate build may take.
 */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;
/**
 Milliseconds one arm may take.
 */
const ARM_TIMEOUT_MS = 900_000;
/**
 The lines a green arm must print, one per claim it makes.
 */
const GREEN_MARKERS: readonly string[] = [
  'DOCTOR-GUEST group-one-passes',
  'DOCTOR-GUEST group-two-passes',
  'DOCTOR-GUEST group-three-passes',
  'DOCTOR-GUEST unloaded-key-is-a-finding',
  'DOCTOR-GUEST container-canmount-is-a-finding',
  'DOCTOR-GUEST nested-row-is-a-finding',
  'DOCTOR-GUEST misfiled-record-is-a-finding',
  'DOCTOR-GUEST staging-forms-are-removed-and-nothing-else-is',
  'DOCTOR-GUEST rename-grant-is-a-finding',
  'DOCTOR-GUEST readiness-from-another-boot-is-a-finding',
  'DOCTOR-GUEST unread-step-facts-are-unknown',
  'DOCTOR-GUEST snapdir-visible-is-a-finding',
  'DOCTOR-GUEST snapdir-hidden-is-a-finding',
  'DOCTOR-GUEST snapdir-default-is-a-finding',
  'DOCTOR-GUEST snapdir-follows-the-container',
  'DOCTOR-GUEST-REMOVAL healthy-lines-pass',
  'DOCTOR-GUEST-REMOVAL lost-lock-is-a-finding',
  'DOCTOR-GUEST-REMOVAL written-lock-passes-and-the-line-wrote-nothing',
  'DOCTOR-GUEST-REMOVAL foreign-lock-is-a-finding',
  'DOCTOR-GUEST-REMOVAL interrupt-state-is-a-finding',
  'DOCTOR-GUEST-REMOVAL blocked-removal-names-the-holder',
  'DOCTOR-GUEST-REMOVAL lost-snapshot-is-a-finding',
  'DOCTOR-GUEST-REMOVAL origin-mark-is-a-finding',
  'DOCTOR-GUEST-REMOVAL pruned-registration-is-a-finding',
  'DOCTOR-GUEST-REMOVAL retention-names-its-fields',
  'CLONE-FACTS-GUEST mounted-clone-passes',
  'CLONE-FACTS-GUEST unmounted-clone-passes',
  'CLONE-FACTS-GUEST staged-clone-is-a-staging-finding',
  'CLONE-FACTS-GUEST stopped-removal-stands-with-its-plan',
  'CLONE-FACTS-GUEST marked-origin-is-a-finding',
  'CLONE-FACTS-GUEST held-clone-names-its-namespace-and-task',
  'CLONE-FACTS-GUEST gather-writes-nothing',
];

//endregion Constants

//region Steps

/**
 Refuse a name another guest already holds.

 @param chosen - the run's options

 @throws Error naming the guest a listing already holds
 */
async function create(chosen: GuestDriverOptions,): Promise<void> {
  /**
   What `mvm list` reported before this run asked for a guest.
   */
  const listed = await captureHostCommand({
    command: 'mvm',
    args: [...backend.mvmArgumentWords, 'list',],
    timeoutMilliseconds: HOST_COMMAND_TIMEOUT_MS,
  },);
  if (holdsGuestName({
    listing: listed,
    name: chosen.name,
  },))
    throw new Error(`a guest named ${chosen.name} is already listed; name another with --name`,);
  await recipe({
    chosen,
    step: 'create',
  },);
}

/**
 Install the guest's tools: OpenZFS 2.4.4, Rust and cargo-nextest.

 @param chosen - the run's options
 */
async function prepare(chosen: GuestDriverOptions,): Promise<void> {
  await recipe({
    chosen,
    step: 'prepare',
  },);
}

/**
 Push a snapshot of this worktree and build this crate's binaries and tests inside the guest.

 @param chosen - the run's options

 @throws Error naming the step that refused
 */
async function build(chosen: GuestDriverOptions,): Promise<void> {
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
  await host({
    chosen,
    command: 'tar',
    args: [
      '--create',
      '--gzip',
      '--file',
      tarball,
      '--exclude=target',
      '--exclude=node_modules',
      '--directory',
      context,
      '.',
    ],
    log: 'host-package.log',
  },);
  // A backend that mounts no share has no folder for the archive to land in until this makes one.
  await ensureTheGuestSharedFolder({ chosen, },);
  await host({
    chosen,
    command: 'mvm',
    args: [
      ...backend.mvmArgumentWords,
      'push',
      chosen.name,
      tarball,
      backend.pushDestinationOf('subject.tar.gz'),
    ],
    log: 'guest-push.log',
  },);
  /**
   The guest's own script: unpack the archive and build the executable, the binary tests and the library tests.
   */
  const script = [
    'set -eu',
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    // The lib test binary is built without optimisation: one release-mode rustc compiling the whole crate's tests was
    // killed with SIGKILL inside this guest's 7.75 GiB, and the removal lines' arm needs that harness, not its speed.
    'cargo build --release --locked --features test-only-switches --bins > /root/doctor-route-build.log 2>&1',
    'cargo build --release --locked --features test-only-switches --test native_binary >> /root/doctor-route-build.log 2>&1',
    'cargo test --locked --features test-only-switches --lib --no-run >> /root/doctor-route-build.log 2>&1',
    'tail --lines=3 /root/doctor-route-build.log',
  ]
    .join('\n',);
  await guest({
    chosen,
    script,
    log: 'guest-crate-build.log',
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS,
  },);
}

/**
 Run the arm and refuse unless every claim it prints was printed.

 @param chosen - the run's options

 @throws Error naming the skip the arm printed, or the claims it did not print
 */
async function arm(chosen: GuestDriverOptions,): Promise<void> {
  /**
   The guest's own script: both arms, each with its log on the guest's standard output.
   */
  const script = [
    'set -eu',
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/opt/openzfs-2.4.4/sbin:/opt/openzfs-2.4.4/bin:/root/.cargo/bin:$PATH',
    'export LD_LIBRARY_PATH=/opt/openzfs-2.4.4/lib',
    `export ${GUEST_VARIABLE}=${backend.hostnameOf(chosen.name)}`,
    `export ${POOL_VARIABLE}=${POOL}`,
    `export ${REMOVAL_GUEST_VARIABLE}=${backend.hostnameOf(chosen.name)}`,
    `export ${REMOVAL_POOL_VARIABLE}=${REMOVAL_POOL}`,
    `export ${REMOVAL_PREFIX_VARIABLE}=${REMOVAL_PREFIX}`,
    `export ${REMOVAL_ROOT_VARIABLE}=${REMOVAL_ROOT}`,
    `export ${CLONE_FACTS_GUEST_VARIABLE}=${backend.hostnameOf(chosen.name)}`,
    `export ${CLONE_FACTS_POOL_VARIABLE}=${CLONE_FACTS_POOL}`,
    `export ${CLONE_FACTS_PREFIX_VARIABLE}=${REMOVAL_PREFIX}`,
    `export ${CLONE_FACTS_ROOT_VARIABLE}=${CLONE_FACTS_ROOT}`,
    'if /root/.cargo/bin/cargo nextest run --release --locked --features test-only-switches --test native_binary --no-capture --status-level all --filterset "test(zfs_doctor_route_guest)" > /root/doctor-route-arm.log 2>&1; then status=0; else status=$?; fi',
    'cat /root/doctor-route-arm.log',
    'if /root/.cargo/bin/cargo test --locked --lib --no-fail-fast -- --nocapture zfs_doctor_removal_lines_guest > /root/doctor-removal-arm.log 2>&1; then removal=0; else removal=$?; fi',
    'cat /root/doctor-removal-arm.log',
    'if /root/.cargo/bin/cargo test --locked --lib --no-fail-fast -- --nocapture zfs_doctor_clone_facts_guest > /root/doctor-clone-facts-arm.log 2>&1; then clonefacts=0; else clonefacts=$?; fi',
    'cat /root/doctor-clone-facts-arm.log',
    'if test $status -ne 0; then exit $status; fi',
    'if test $removal -ne 0; then exit $removal; fi',
    'exit $clonefacts',
  ]
    .join('\n',);
  /**
   Everything the arm printed, read back from the guest's log.
   */
  const pulled = await guest({
    chosen,
    script,
    log: 'guest-arm.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS,
  },);
  if (holdsLineStartingWith({
    text: pulled,
    word: 'SKIP',
  },))
    throw new Error('the arm printed SKIP inside the guest: its variables did not reach it',);
  /**
   The claims the arm did not print.
   */
  const missing = missingMarkers({
    text: pulled,
    markers: GREEN_MARKERS,
  },);
  if (missing.length > 0)
    throw new Error(`the arm did not print: ${missing
      .join(', ',)}`,);
  process.stdout
    .write(`doctor route arm passed ${String(GREEN_MARKERS.length)} claim(s)\n`,);
}

/**
 Destroy the guest this driver made, unless the run asked to keep it.

 @param chosen - the run's options
 */
async function destroy(chosen: GuestDriverOptions,): Promise<void> {
  if (chosen.keepGuest) {
    process.stdout
      .write(`kept ${chosen.name}\n`,);
    return;
  }
  await recipe({
    chosen,
    step: 'destroy',
  },);
}

/**
 The guest this run made, destroyed however the run ends.

 The step's own `finally` destroyed the guest after the build and the arm; a disposable does the same and leaves the
 run one shape, whether the arm passed, threw, or was never reached.

 @param chosen - the run's options

 @returns the guest, whose disposal destroys it unless the run keeps it

 @example
 ```ts
 await using thisGuest = madeGuest(chosen);
 ```
 */
function madeGuest(chosen: GuestDriverOptions,): { readonly [Symbol.asyncDispose]: () => Promise<void>; } {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await destroy(chosen,);
    },
  };
}

/**
 Create, prepare, build, arm and destroy, in that order.

 @param chosen - the run's options

 @throws Error naming the step or the claim that refused
 */
async function all(chosen: GuestDriverOptions,): Promise<void> {
  await create(chosen,);
  /**
   The guest this run made, destroyed however this run ends.
   */
  await using thisGuest = madeGuest(chosen,);
  await prepare(chosen,);
  await build(chosen,);
  await arm(chosen,);
}

//endregion Steps

//region The run

/**
 The steps this driver knows, and nothing else.
 */
const steps: Record<string, (chosen: GuestDriverOptions,) => Promise<void>> = {
  create,
  prepare,
  build,
  arm,
  destroy,
  all,
};

/**
 The step and the values the command line gave this run.
 */
const chosen = guestDriverOptions({
  argv: process.argv
    .slice(2,),
  defaultName: 'git-policy-doctor-route',
  defaultLogDir: resolve(
    import.meta.dirname,
    '..',
    'target',
    'doctor-route-guest-logs',
  ),
},);
await mkdir(
  chosen.logDir,
  {
    recursive: true,
    mode: FOLDER_MODE,
  },
);

/**
 The step the command line names.
 */
const step = steps[chosen.step];

if (step === undefined)
  throw new Error(`unknown step ${chosen.step}; give create, prepare, build, arm, destroy or all`,);
await step(chosen,);

//endregion The run
