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
 - a locked holder, refused before any signal;
 - a locked holder that is a service unit of the guest account's own user manager: the same two refusals before the
   stop, the manager's stop of that unit, and the host's unmount and the head destroy after it;
 - a mount namespace no task holds, found by `listns` and ended by the act's own stages, or refused
   `holder-unendable` on a kernel that answers the listing `ENOSYS`.
 The caller's session's other job stays alive through every phase.

 The `user-manager` step is the measurement this round owed: it gives the guest's account a user manager of its own
 (`loginctl enable-linger`, quoted from the guest's own manual), starts a transient unit that holds a copy of a slot
 mount, and reads what the kernel answers the unmount and the move before `systemctl --user stop` of that unit and
 after it. It makes the state twice, once with `PrivateMounts=yes` alone and once with `PrivateUsers=yes` as well,
 and the arm makes it a third time.

 Usage: node e2e/forced-act-guest.ts <create|prepare|user-manager|run|arm|destroy|all> [--name <guest>] [--log-dir <dir>]
        [--image <mvm template>] [--kernel-release <release>]
 `--image` names a custom `mvm` template, such as step 23's CachyOS one, and the recipe then checks what the template
 already carries instead of installing OpenZFS into an Ubuntu guest; `--kernel-release` names the release that template
 must boot. A run that names neither gets the Ubuntu 24.04 guest this driver always made.
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.
 `--image` names a custom `mvm` template, such as step 23's CachyOS one, and the recipe then checks what the template
 already carries instead of installing OpenZFS into an Ubuntu guest; `--kernel-release` names the release that template
 must boot. A run that names neither gets the Ubuntu 24.04 guest this driver always made.

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
  type GuestDriverOptions,
  type RunArguments,
  flagWord,
  guestDriverOptions,
  runArguments,
} from '../bin/mvm-guest-options.ts';
import {
  FOLDER_MODE,
  HOST_COMMAND_TIMEOUT_MS,
} from '../bin/mvm-guest-bounds.ts';
import {
  holdsLineStartingWith,
} from '../bin/mvm-guest-arm-log.ts';
import {
  awaitDetachedGuestScript,
  pullGuestLog,
  startDetachedGuestScript,
} from '../bin/mvm-guest-detached-script.ts';
import {
  armScript,
  buildScript,
} from './forced-act-guest-arm-scripts.ts';
import {
  measureTheStop,
} from './forced-act-guest-measurement.ts';

/**
 Milliseconds the guest's build of the library and its tests may take.

 The bound is a stop for a build that hangs and not a deadline the machine is expected to meet. Measured in the guest of
 this round while five or more other guests ran on this host: 34m55s on 2026-11-27T22:50, 29m54s twice after that, and
 one run cut at the 39 minutes a 40-minute bound leaves on 2026-11-28T01:38 without finishing.
 */
const CRATE_BUILD_TIMEOUT_MS = 5_400_000;
/**
 Milliseconds the guest arm may take: eight phases, two of them waiting the harness's termination wait and the locked
 holder's stop its own.
 */
const ARM_TIMEOUT_MS = 900_000;
/**
 Milliseconds the host waits past a detached script's own bound before it gives up on the script.

 The script's own `timeout` fires inside the guest first, so this margin only covers the guest agent's own lateness.
 */
const FINISH_MARGIN_MS = 600_000;

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
  'FORCEDACT phase=locked-holder-stopped-through-its-unit',
  // The taskless namespace's one marker covers both kernels: a kernel without `listns` writes
  // `phase=taskless-namespace-unlistable`, which holds this text and states the refusal word.
  'FORCEDACT phase=taskless-namespace',
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
    extraArgumentWords: recipeExtraArguments(),
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
    extraArgumentWords: recipeExtraArguments(),
  },);
}

/**
 The words the recipe's steps take beyond the guest's name and log folder.

 A run that names a custom template hands the recipe the template and the kernel release it must boot; a run that names
 neither leaves the recipe on its own Ubuntu 24.04 image, which it installs OpenZFS into.

 @returns the recipe's extra words, empty for a run that names no template

 @example
 ```ts
 recipeExtraArguments(); // ['--image', 'cachyos-7.3.5-1-zfs-2.4.4', '--kernel-release', '7.3.5-1-cachyos']
 ```
 */
function recipeExtraArguments(): readonly string[] {
  /**
   The words this run's own command line carries.
   */
  const read: RunArguments = runArguments({
    argv: process.argv
      .slice(2,),
  },);
  /**
   The `mvm` template the run names, empty when it names none.
   */
  const image = flagWord({
    read,
    flag: '--image',
    fallback: '',
  },);
  /**
   The kernel release that template boots, empty when the run names none.
   */
  const kernelRelease = flagWord({
    read,
    flag: '--kernel-release',
    fallback: '',
  },);
  /**
   The extra words, in the order the recipe reads them.
   */
  const words: string[] = [];
  if (image !== '')
    words.push(
      '--image',
      image,
    );
  if (kernelRelease !== '')
    words.push(
      '--kernel-release',
      kernelRelease,
    );
  return words;
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
  await removeTheOldSharedLog({
    chosen,
    log: 'guest-crate-build-shell.log',
  },);
  await startDetachedGuestScript({
    chosen,
    script: buildScript(),
    log: 'guest-crate-build-shell.log',
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS,
    nameCheck: 'none',
  },);
  await awaitDetachedGuestScript({
    chosen,
    log: 'guest-crate-build-shell.log',
    finishTimeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS + FINISH_MARGIN_MS,
  },);
  await runTheArm(chosen,);
}

/**
 What:
  Delete one detached script's log in the guest's shared folder before that script starts.
 Why:
  The step file truncates its own log when it starts, and the host asks for that log's last line the moment the launcher
  returns: a log an earlier run left there has a `DONE=` line the host reads as this run's end, and a pull can meet the
  file while the new step is emptying it. A log that is gone before the start is the one answer to both.

 @param chosen - the run's guest and log folder

 @param log - the log's name inside the shared folder

 @throws GuestDriverError when the guest's own command refused

 @example
 ```ts
 await removeTheOldSharedLog({ chosen, log: 'guest-crate-build-shell.log' });
 ```
 */
async function removeTheOldSharedLog({
  chosen,
  log,
}: {
  readonly chosen: GuestDriverOptions;
  readonly log: string;
},): Promise<void> {
  await guest({
    chosen,
    script: `rm --force /mnt/shared/${log}`,
    log: 'guest-remove-the-old-log.log',
    timeoutMilliseconds: HOST_COMMAND_TIMEOUT_MS,
  },);
}

/**
 Start the arm detached behind a build that already ran, read its log, and refuse a run that lacks a phase's line.

 @param chosen - the run's options

 @throws Error naming the marker the pulled log lacks, or the guest script that refused
 */
async function runTheArm(chosen: GuestDriverOptions,): Promise<void> {
  await removeTheOldSharedLog({
    chosen,
    log: 'guest-forced-act-shell.log',
  },);
  await startDetachedGuestScript({
    chosen,
    script: armScript({
      name: chosen.name,
      markers: EXPECTED_MARKERS,
    },),
    log: 'guest-forced-act-shell.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS,
    nameCheck: 'none',
  },);
  await awaitDetachedGuestScript({
    chosen,
    log: 'guest-forced-act-shell.log',
    finishTimeoutMilliseconds: ARM_TIMEOUT_MS + FINISH_MARGIN_MS,
  },);
  /**
   The arm's own log, pulled back out of the guest.
   */
  const pulled: string = await pullGuestLog({
    chosen,
    file: '/forced-act-arm.log',
    log: 'host-pull-arm.log',
  },);
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
    extraArgumentWords: recipeExtraArguments(),
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
  await measureTheStop(chosen,);
  await run(chosen,);
  await destroy(chosen,);
}

/**
 The steps this driver knows, and nothing else.
 */
const steps: Record<string, (chosen: GuestDriverOptions,) => Promise<void>> = {
  create,
  prepare,
  'user-manager': measureTheStop,
  run,
  arm: runTheArm,
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
