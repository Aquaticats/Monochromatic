#!/usr/bin/env node
/**
 Drive plan step 12's slot classes, its move and removal refusals, its prune pointer and its repair report end to
 end in a throwaway `mvm` guest.

 The arm needs a real pool, a real Git registration and a real clone: it reads one slot's class from the binding
 record, the mount intent and the mount table while the clone is mounted and again while it is unmounted, and it
 shows the wrapper's refusal beside Git's own answer, which is a busy mount while mounted and success while
 unmounted. The same run drives the wrapper's own command path: `git worktree move` is refused, the removal of a slot
 a whole clone is bound to runs the wrapper's own order and destroys the clone, its records and its registration,
 the prune pass writes the recorded pointer back before Git would drop the registration, and `git worktree repair`
 prints what it can and cannot restore. Every one of those is a ZFS or Git fact no host control can make, so the arm
 runs only inside the guest the ZFS binding's recipe creates
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`).

 Usage: node e2e/slot-state-and-move-refusal-guest.ts <create|prepare|run|destroy|all> [--name <guest>]
          [--log-dir <dir>] [--keep-guest]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `slot-state-arm.log`, pulled back into the log directory, and a run passes only when it
 carries every `SLOT-STATE`, `MOVE-REFUSAL`, `REMOVAL`, `PRUNE-POINTER` and `REPAIR` line below and no `SKIP`
 line, so an arm that returned without running fails the run instead of reporting a green suite.
 */
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
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
  shellQuote,
} from '../bin/mvm-guest-command-line.ts';
import {
  awaitDetachedGuestScript,
  startDetachedGuestScript,
} from '../bin/mvm-guest-detached-script.ts';

/**
 Where the crate is unpacked and built inside the guest, which every account may traverse.
 */
const CRATE_DIR = '/opt/git-policy-cli-slot-state';
/**
 The log the guest's own build writes into the shared folder, which the host reads its status from.
 */
const BUILD_LOG = 'slot-state-crate-build.log';
/**
 Milliseconds the host waits past the build's own bound, so the guest can write the status line the host reads.
 */
const BUILD_FINISH_MARGIN_MS = 180_000;
/**
 Milliseconds the guest's build of the test target may take.
 */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;
/**
 Milliseconds the guest arm may take.
 */
const ARM_TIMEOUT_MS = 900_000;

/**
 The environment variable the arm reads, so it never writes a guest fixture on a machine somebody uses.
 */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_SLOT_STATE_GUEST';
/**
 The checkout the arm registers, clones and moves, under the guest's own root.
 */
const CHECKOUT = '/root/slot-state-checkout';
/**
 The OpenZFS userspace the guest built, which every `zfs` the arm runs comes from.
 */
const SBIN = '/opt/openzfs-2.4.4/sbin';
/**
 The pool this driver makes for the arm, and the file-backed vdev it stands on.
 */
const POOL = 'git-policy-cli-slot-state';
/**
 The file-backed vdev the pool stands on.
 */
const POOL_IMAGE = '/home/git-policy-cli-slot-state-fixture/pool.img';

/**
 The folder the pool's vdev stands in, which the guest makes before it truncates the vdev.
 */
const POOL_IMAGE_FOLDER = POOL_IMAGE
  .slice(
  0,
  POOL_IMAGE
    .lastIndexOf('/',),
  );

/**
 The lines a run must find in the guest's arm log, one per class and per move arm plan step 12 runs.
 */
const EXPECTED_MARKERS = [
  'SLOT-STATE mounted class=mounted managed clone dataset=',
  'SLOT-STATE record-behind-mount class=cannot be proved',
  'SLOT-STATE unmounted class=known unmounted clone intent=mount-requested',
  'SLOT-STATE scene slot=removal class=known unmounted clone',
  'SLOT-STATE scene slot=aliased-removal class=known unmounted clone',
  'SLOT-STATE pending-add class=pending add',
  'SLOT-STATE pending-removal class=pending removal',
  'MOVE-REFUSAL mounted cause=bound',
  'MOVE-REFUSAL mounted git=refused-busy',
  'MOVE-REFUSAL wrapper-mounted cause=bound status=refused',
  'MOVE-REFUSAL unmounted cause=bound',
  'MOVE-REFUSAL wrapper-unmounted cause=bound status=refused',
  'MOVE-REFUSAL unmounted git=succeeded',
  'REMOVAL wrapper class=known-unmounted-clone status=removed clone=gone records=gone',
  'REMOVAL wrapper-through-a-shell-alias class=known-unmounted-clone status=removed clone=gone records=gone',
  'PRUNE-POINTER intact registration=kept',
  'PRUNE-POINTER restored registration=kept',
  'REPAIR damaged registration=stands',
  'REPAIR pruned registration=gone',
  'SLOT-STATE done',
] as const;

/**
 How the guest work of one `all` run ended: by itself, or with the failure the run reports.
 */
type GuestWorkEnding =
  | { readonly kind: 'ended', }
    | {
      readonly kind: 'failed',
      readonly error: Error
    };

/**
 Create the guest the arm runs in.

 @param chosen - the run's options
 */
async function create(chosen: GuestDriverOptions,): Promise<void> {
  await recipe({
    chosen,
    step: 'create',
  },);
}

/**
 Install the guest's tools: OpenZFS 2.4.4, the Rust toolchain and `cargo-nextest`.

 @param chosen - the run's options
 */
async function prepare(chosen: GuestDriverOptions,): Promise<void> {
  await recipe({
    chosen,
    step: 'prepare',
  },);
}

/**
 Destroy the guest this run made.

 @param chosen - the run's options
 */
async function destroy(chosen: GuestDriverOptions,): Promise<void> {
  await recipe({
    chosen,
    step: 'destroy',
  },);
}

/**
 Build the test target inside the guest and run plan step 12's arm there.

 The snapshot is the same repository-shaped tree the container gate builds from (`bin/native-wrapper-snapshot.ts`), so
 the arm compiles against the path crates the wrapper links, the ZFS binding among them.

 @param chosen - the run's options

 @throws Error naming the step that refused, or the marker the pulled log lacks
 */
async function run(chosen: GuestDriverOptions,): Promise<void> {
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
      '--directory',
      context,
      '.',
    ],
    log: 'host-package.log',
  },);
  await host({
    chosen,
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
   The guest's own script: unpack the archive, make the pool, and build the test target.
   */
  const prepareBuild = [
    'set -eu',
    // `mvm push` lands the tarball in the guest's shared folder, not at the path it was given.
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    // The arm needs a pool of its own; the checkout dataset and its clone are made on it by the arm.
    `mkdir --parents ${POOL_IMAGE_FOLDER}`,
    `if ! ${SBIN}/zpool list -H -o name ${POOL} > /dev/null 2>&1; then`,
    `  truncate --size 1G ${POOL_IMAGE}`,
    `  ${SBIN}/zpool create -f -O mountpoint=none ${POOL} ${POOL_IMAGE}`,
    'fi',
    `mkdir --parents ${CHECKOUT}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    // The one target the arm starts, two jobs at a time: a release rustc of this crate's tests took 6.9 GiB of this
    // guest's 8 GiB, and the kernel killed it at four jobs on 2026-11-27 (issue 6581; cargo exited 101). The lib test
    // harness the arm never runs is not built here.
    'cargo build --release --test native_binary --locked --features test-only-switches -j 2 > /root/slot-state-build.log 2>&1',
    'tail --lines=3 /root/slot-state-build.log',
  ]
    .join('\n',);
  // The build takes minutes (6m 07s on 2026-11-27), and a guest agent can lose a long command's result
  // (`doc/troubleshooting/mvm-guest-agent-forgets-a-long-commands-result.md`), so it runs detached and writes the line
  // that says how it ended into its own log, which the host waits for and pulls.
  await startDetachedGuestScript({
    chosen,
    script: prepareBuild,
    log: BUILD_LOG,
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS,
    nameCheck: 'launcher',
  },);
  await awaitDetachedGuestScript({
    chosen,
    log: BUILD_LOG,
    finishTimeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS + BUILD_FINISH_MARGIN_MS,
  },);
  /**
   The guest's own script: run the arm, refuse a returned one, and copy its log to the shared folder.
   */
  const arm = [
    'set -eu',
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export ${GUEST_VARIABLE}=${chosen.name}`,
    `export CLI_GIT_ZFS_SWEEP_POOL=${POOL}`,
    `export CLI_GIT_ZFS_SWEEP_SBIN=${SBIN}`,
    `export CLI_GIT_ZFS_SWEEP_CHECKOUT=${CHECKOUT}`,
    'if cargo nextest run --release --locked --features test-only-switches --test native_binary --no-capture --status-level all --filterset "test(zfs_slot_state_guest)" > /root/slot-state-arm.log 2>&1; then status=0; else status=$?; fi',
    'tail --lines=60 /root/slot-state-arm.log',
    'test "$status" = 0',
    `if grep --quiet '^SKIPPED' /root/slot-state-arm.log; then echo 'the slot-state arm returned without running'; exit 1; fi`,
    ...EXPECTED_MARKERS
      .map(function markerLine(marker: string,): string {
        return `grep --quiet --fixed-strings ${shellQuote(marker,)} /root/slot-state-arm.log`;
      },),
    'cp /root/slot-state-arm.log /mnt/shared/slot-state-arm.log',
  ]
    .join('\n',);
  await guest({
    chosen,
    script: arm,
    log: 'guest-slot-state.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS * 2,
  },);
  /**
   The arm's own log, pulled back out of the guest.
   */
  const pulledPath = join(
    chosen.logDir,
    'slot-state-arm.log',
  );
  await host({
    chosen,
    command: 'mvm',
    args: [
      '--backend',
      'libvirt',
      'pull',
      chosen.name,
      '/mnt/shared/slot-state-arm.log',
      pulledPath,
    ],
    log: 'host-pull-slot-state.log',
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
      throw new Error(`the pulled slot-state log lacks ${marker}; the guest ran the arm without performing it`,);
  }
}

/**
 The guest work of one `all` run, which reports how it ended instead of ending the run there, so the guest it made is
 dealt with first.

 @param chosen - the run's options

 @param made - whether this run has to deal with a guest, which the work sets once `create` returned

 @returns how the guest work ended, with the failure the run reports
 */
async function guestWork({
  chosen,
  made,
}: {
  readonly chosen: GuestDriverOptions;
  readonly made: { created: boolean };
},): Promise<GuestWorkEnding> {
  try {
    await create(chosen,);
    made.created = true;
    await prepare(chosen,);
    await run(chosen,);
    return { kind: 'ended', };
  }
  catch (thrown) {
    /**
     The failure this run reports once the guest it made is dealt with.
     */
    const error = Error
      .isError(thrown,)
      ? thrown
      : new Error(
        caughtValueText(thrown,),
        { cause: thrown, },
      );
    return {
      kind: 'failed',
      error,
    };
  }
}

/**
 Destroy the guest this run made, reporting a destruction that failed beside a failure the run already holds.

 The first failure is the one the run reports: a guest that could not be destroyed is named on standard error beside
 it, because both are a person's to deal with and only one of them can be thrown.

 @param chosen - the run's options

 @param holdsAFailure - whether the run already holds the failure it reports, which a destruction must not replace

 @throws Error naming the destruction that failed, when the guest work itself ended
 */
async function destroyedAfter({
  chosen,
  holdsAFailure,
}: {
  readonly chosen: GuestDriverOptions;
  readonly holdsAFailure: boolean;
},): Promise<void> {
  try {
    await destroy(chosen,);
  }
  catch (destruction) {
    if (!holdsAFailure)
      throw destruction;
    console.error(`git-policy slot-state guest: destroying '${chosen.name}' also failed: ${String(destruction)}`,);
  }
}

/**
 Create, prepare, run and destroy, keeping the guest when the run asked for it and reporting the first failure.

 @param chosen - the run's options

 @throws Error naming the first failure of the run
 */
async function all(chosen: GuestDriverOptions,): Promise<void> {
  /**
   Whether this run made a guest of its own, which decides whether it may destroy one.
   */
  const made = { created: false, };
  /**
   How the guest work ended.
   */
  const ending = await guestWork({
    chosen,
    made,
  },);
  if (chosen.keepGuest && made.created)
    console.log(`git-policy slot-state guest: --keep-guest kept '${chosen.name}' to inspect and destroy`,);
  else if ((!chosen.keepGuest) && made.created)
    await destroyedAfter({
      chosen,
      holdsAFailure: ending.kind === 'failed',
    },);
  else if (!chosen.keepGuest)
    console.error(`git-policy slot-state guest: no guest was destroyed, because this run did not create '${chosen.name}'`,);
  if (ending.kind === 'failed')
    throw ending.error;
}

/**
 The steps by name, and the whole run.
 */
const steps: Record<string, (chosen: GuestDriverOptions,) => Promise<void>> = {
  create,
  prepare,
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
  defaultName: 'git-policy-cli-slot-state',
  defaultLogDir: resolve(
    import.meta.dirname,
    '..',
    'target',
    'slot-state-guest-logs',
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
  throw new Error(`unknown step '${chosen.step}'; use ${Object
    .keys(steps,)
    .join(', ',)}`,);
await step(chosen,);
console.log(`git-policy slot-state guest: step '${chosen.step}' finished; logs in ${chosen.logDir}`,);
