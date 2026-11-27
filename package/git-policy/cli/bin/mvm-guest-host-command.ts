/**
 The host side of a guest run: a command whose whole output lands in its own log, one step of the ZFS binding's guest
 recipe, and the snapshot of this checkout that a guest builds.

 Every driver borrows the ZFS binding's guest recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): the recipe owns the guest's creation, its OpenZFS
 preparation and its destruction, so no driver repeats that work and each one names the step it needs. The snapshot is
 the tree the container gate builds from (`bin/native-wrapper-snapshot.ts`), so a guest's build compiles against the path
 crates the wrapper links rather than against a copy somebody remembered to add.

 The host command and the recipe step were answered best by `e2e/guest-host/host-side-commands.ts` and the bound that
 starts a command in a process group of its own by `bin/mvm-guest-step-command.ts`; the snapshot was carried by
 `guest-host` alone, and its three drivers were the only ones that pushed a checkout without repeating the four commands.
 */
import {
  mkdir,
  rm,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';

import { copyWrapperSnapshot, } from './native-wrapper-snapshot.ts';
import {
  FOLDER_MODE,
  HOST_COMMAND_TIMEOUT_MS,
} from './mvm-guest-bounds.ts';
import type {
  GuestFacts,
  GuestLogFolder,
} from './mvm-guest-guest-facts.ts';
import { runGuestRecipeStep, } from './mvm-guest-recipe-step.ts';
import {
  runRequiredStepCommand,
} from './mvm-guest-step-command.ts';

/**
 One host command whose whole output goes to its own log.

 @param chosen - the run's folder, where the log is written

 @param command - the program to run, found on `PATH`

 @param args - its arguments, each passed whole

 @param log - the log file's name in the log folder

 @param timeoutMilliseconds - milliseconds the command may take, which a guest script's own bound replaces because the
 guest command *is* that script

 @throws GuestDriverError naming the command and its status

 @example
 ```ts
 await host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'list'], log: 'host-mvm-list.log' });
 ```
 */
export async function host({
  chosen,
  command,
  args,
  log,
  timeoutMilliseconds = HOST_COMMAND_TIMEOUT_MS,
}: {
  readonly chosen: GuestLogFolder;
  readonly command: string;
  readonly args: readonly string[];
  readonly log: string;
  readonly timeoutMilliseconds?: number;
},): Promise<void> {
  await runRequiredStepCommand({
    logDirectory: chosen.logDir,
    program: command,
    argumentWords: args,
    log,
    timeoutMilliseconds,
  },);
}

/**
 Run one step of the ZFS binding's guest recipe, which owns the guest's creation, preparation and destruction.

 The recipe's own progress goes to this process's streams, because a person watching a guest being prepared is reading
 it, and to the run's log folder as well through the recipe's own log.

 @param chosen - the run's guest and log folder

 @param step - the recipe's step

 @param extraArgumentWords - further recipe options, such as the guest's image, which the driver's command line gave

 @throws GuestDriverError naming the recipe step and its status

 @example
 ```ts
 await recipe({ chosen, step: 'prepare' });
 ```
 */
export async function recipe({
  chosen,
  step,
  extraArgumentWords = [],
}: {
  readonly chosen: GuestFacts;
  readonly step: string;
  readonly extraArgumentWords?: readonly string[];
},): Promise<void> {
  await runGuestRecipeStep({
    name: chosen.name,
    logDirectory: chosen.logDir,
    step,
    extraArgumentWords,
  },);
}

/**
 Push this checkout into a guest as one archive, ready for the guest to unpack and build.

 The snapshot is taken in the run's own log folder, so a build somebody has to look into afterwards sits beside the logs
 of the commands that made it.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError naming the command that refused

 @example
 ```ts
 await pushWorktreeSnapshot({ chosen });
 ```
 */
export async function pushWorktreeSnapshot({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  /**
   The folder the snapshot is taken in, which this run replaces whatever an earlier run left there.
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
   The archive pushed into the guest, which its build unpacks.
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
}
