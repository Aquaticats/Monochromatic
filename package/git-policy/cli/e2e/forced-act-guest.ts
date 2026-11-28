#!/usr/bin/env node
/**
 Drive the forced act's guest arm in a throwaway `mvm` guest: every kernel call the act makes, run by the route's own
 code against a file-backed pool.

 The act enters holder mount namespaces, unmounts by a recorded handle, makes a mount a slave, moves a mount and
 removes a directory, so no control on a person's machine may run it. This driver borrows the ZFS binding's guest
 recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and prepares it
 with OpenZFS 2.4.4 and the Rust toolchain, then builds this crate inside it, runs the arm, and destroys the guest.

 The phases, as the design's "Measurements owed" and the kind table ask
 (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`):
 - a path holder in a session of its own, asked out, then the host's unmount and the head destroy;
 - the same for a holder that ignores the ask and is killed;
 - way 8, in place by the add's handle, in the shape systemd leaves a unit's parent;
 - the peer shape, where the closure names the host and the act falls to way 7;
 - way 7 in both shapes, with the directory made, the copy unmoved, and the directory removed;
 - a locked holder, refused before any signal.
 The caller's session's other job stays alive through every phase.

 Usage: node e2e/forced-act-guest.ts <create|prepare|run|destroy|all> [--name <guest>] [--log-dir <dir>]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `forced-act-arm.log`, pulled back into the log directory, and a run passes only when it
 carries every `FORCEDACT` line below and no `SKIPPED` line, so an arm that returned without running fails the run
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
} from '../bin/mvm-guest-arm-log.ts';
import {
  shellQuote,
} from '../bin/mvm-guest-command-line.ts';

/**
 Where the wrapper's tree is unpacked and built inside the guest, which every account may traverse.
 */
const CRATE_DIR = '/opt/forced-act';
/**
 Milliseconds the guest's build of the library and its tests may take.
 */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;
/**
 Milliseconds the guest arm may take: seven phases, two of them waiting the harness's termination wait.
 */
const ARM_TIMEOUT_MS = 900_000;

/**
 The environment the arm reads, so it never runs anywhere but this guest.
 */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_FORCED_ACT_GUEST';
/**
 The pool the arm builds and destroys.
 */
const POOL = 'forcedact';
/**
 The folder the pool's image lives under.
 */
const FIXTURE_ROOT = '/home/forced-act-fixture';
/**
 The OpenZFS 2.4.4 prefix the guest's recipe installs.
 */
const OPENZFS_PREFIX = '/opt/openzfs-2.4.4';
/**
 The `PATH` the guest's OpenZFS userspace needs.
 */
const OPENZFS_PATH = `${OPENZFS_PREFIX}/sbin:${OPENZFS_PREFIX}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;

/**
 The lines a run must find in the guest's arm log, one per phase it performed.
 */
const EXPECTED_MARKERS = [
  'FORCEDACT phase=path-holder-asked',
  'FORCEDACT phase=path-holder-killed',
  'FORCEDACT phase=namespace-way8-systemd-shape',
  'FORCEDACT phase=namespace-way8-peer-sent-to-way7',
  'FORCEDACT phase=namespace-way7-repaired',
  'FORCEDACT phase=namespace-way7-shared',
  'FORCEDACT phase=locked-holder-refused',
  'FORCEDACT phase=arm-finished',
] as const;

/**
 Create the guest, refusing a name another run holds.

 @param chosen - the run's options
 */
async function create(chosen: GuestDriverOptions,): Promise<void> {
  await recipe({
    chosen,
    step: 'create',
  },);
}

/**
 Install OpenZFS 2.4.4, the Rust toolchain and `cargo-nextest` in the guest.

 @param chosen - the run's options
 */
async function prepare(chosen: GuestDriverOptions,): Promise<void> {
  await recipe({
    chosen,
    step: 'prepare',
  },);
}

/**
 Push a repository-shaped snapshot of the wrapper, build it in the guest, and run the arm.

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
   The guest's own script: unpack the archive and build the crate's library and its tests.
   */
  const build = [
    'set -eu',
    // `mvm push` lands the tarball on the shared folder, so the guest copies it where the build reads it, over whatever
    // an earlier run left there: a kept copy from an earlier run is a stale tree, not this run's.
    'cp --force /mnt/shared/subject.tar.gz /root/subject.tar.gz',
    // The dependency build is the guest's heaviest work and outlives one run, so it is kept out of the tree this run
    // replaces: a second run then compiles this crate's own test binary alone.
    `if test -d ${CRATE_DIR}/package/git-policy/cli/target; then rm --recursive --force /root/target-kept; mv ${CRATE_DIR}/package/git-policy/cli/target /root/target-kept; fi`,
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `if test -d /root/target-kept; then mv /root/target-kept ${CRATE_DIR}/package/git-policy/cli/target; fi`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    // One job, for the same reason the deferred-origin driver runs one: this build is the library's test binary, and
    // one `rustc` of it takes the guest's whole memory. Measured 2026-11-27 in the guest of issue 6612's round at two
    // jobs, the build's own samples read `AnonPages` 7,249,568 kB of the guest's 8,131,780 kB with one `rustc` at
    // 7,418,336 kB resident and `MemAvailable` at 398,408 kB at its lowest.
    'cargo test --release --lib --no-run --locked -j 1 > /root/forced-act-build.log 2>&1',
    'tail --lines=3 /root/forced-act-build.log',
  ]
    .join('\n',);
  await guest({
    chosen,
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
    `export CLI_GIT_ZFS_FORCED_ACT_POOL=${POOL}`,
    `export CLI_GIT_ZFS_FORCED_ACT_PREFIX=${OPENZFS_PREFIX}`,
    `export CLI_GIT_ZFS_FORCED_ACT_ROOT=${FIXTURE_ROOT}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'if cargo nextest run --release --locked --lib --run-ignored all --no-capture --status-level all --filterset "test(the_forced_act_guest_arm_tests)" > /root/forced-act-arm.log 2>&1; then status=0; else status=$?; fi',
    'tail --lines=60 /root/forced-act-arm.log',
    'test "$status" = 0',
    // A returning arm is not a passing arm, which the binding's own live recipe also holds.
    `if grep --quiet '^SKIPPED' /root/forced-act-arm.log; then echo 'the arm returned without running'; exit 1; fi`,
    ...EXPECTED_MARKERS
      .map(function markerLine(marker: string,): string {
        return `grep --quiet --fixed-strings ${shellQuote(marker,)} /root/forced-act-arm.log`;
      },),
    'cp /root/forced-act-arm.log /mnt/shared/forced-act-arm.log',
    // The pool goes even when the arm left it behind, so the guest is ready for another run.
    `zpool destroy -f ${POOL} 2>/dev/null || true`,
    `rm --force ${FIXTURE_ROOT}/pool.img 2>/dev/null || true`,
  ]
    .join('\n',);
  await guest({
    chosen,
    script: arm,
    log: 'guest-forced-act.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS,
  },);
  /**
   The arm's own log, pulled back out of the guest.
   */
  const pulledPath = join(
    chosen.logDir,
    'forced-act-arm.log',
  );
  await host({
    chosen,
    command: 'mvm',
    args: [
      '--backend',
      'libvirt',
      'pull',
      chosen.name,
      '/mnt/shared/forced-act-arm.log',
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
  if (holdsLineStartingWith({
    text: pulled,
    word: 'SKIPPED',
  },))
    throw new Error('the pulled arm log holds a SKIPPED line, so the arm did not run',);
}

/**
 Destroy the guest.

 @param chosen - the run's options
 */
async function destroy(chosen: GuestDriverOptions,): Promise<void> {
  await recipe({
    chosen,
    step: 'destroy',
  },);
}

/**
 Create, prepare, run and destroy, in that order.

 @param chosen - the run's options

 @throws Error naming the step that refused
 */
async function all(chosen: GuestDriverOptions,): Promise<void> {
  await create(chosen,);
  await prepare(chosen,);
  await run(chosen,);
  await destroy(chosen,);
}

/**
 The steps this driver knows, and nothing else.
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
  defaultName: 'forced-act',
  defaultLogDir: resolve(
    import.meta.dirname,
    '..',
    'target',
    'forced-act-guest-logs',
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
