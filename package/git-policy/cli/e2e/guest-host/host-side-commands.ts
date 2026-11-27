/**
 The host side of a guest run: a command whose output lands in its own log, one
 step of the ZFS binding's guest recipe, and the snapshot of this checkout that a
 guest builds.

 Every driver here borrows the ZFS binding's guest recipe for the parts that are
 not the wrapper's (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`):
 the recipe owns the guest's creation, its OpenZFS preparation and its
 destruction, so no driver repeats that work and each one names the step it needs.
 The snapshot is the tree the container gate builds from
 (`bin/native-wrapper-snapshot.ts`), so a guest's build compiles against the path
 crates the wrapper links rather than against a copy somebody remembered to add.

 @example
 ```ts
 await recipe({ chosen, step: 'create' });
 ```
 */

import {
  mkdir,
  rm,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { copyWrapperSnapshot, } from '../../bin/native-wrapper-snapshot.ts';
import {
  hostCommandTimeoutMilliseconds,
  recipeTimeoutMilliseconds,
} from './guest-host-bounds.ts';
import type {
  GuestRun,
  GuestRunFolder,
} from './guest-run.ts';
import {
  runInheriting,
  runToLog,
} from './logged-command.ts';

/**
 One host command whose whole output goes to its own log.

 @param chosen - the run's folder, where the log is written

 @param command - the program to run

 @param args - its arguments

 @param log - the log file's name in the log folder

 @param timeout - milliseconds the command may take, which a guest script's own
   bound replaces because the guest command *is* that script

 @throws Error naming the command and its status

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
  timeout = hostCommandTimeoutMilliseconds,
}: {
  readonly chosen: GuestRunFolder;
  readonly command: string;
  readonly args: readonly string[];
  readonly log: string;
  readonly timeout?: number;
},): Promise<void> {
  await runToLog({
    logDir: chosen.logDir,
    subject: `${command} ${args.join(' ',)}`,
    command,
    args,
    log,
    timeout,
  },);
}

/**
 Run one step of the ZFS binding's guest recipe, which owns the guest's creation,
 preparation and destruction.

 The recipe's own progress goes to this process's streams, because a person
 watching a guest being prepared is reading it, and to the run's log folder as
 well through the recipe's own log.

 @param chosen - the run's guest and log folder

 @param step - the recipe's step

 @param extraArguments - further recipe options, such as the guest's image

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await recipe({ chosen, step: 'prepare' });
 ```
 */
export async function recipe({
  chosen,
  step,
  extraArguments = [],
}: {
  readonly chosen: GuestRun;
  readonly step: string;
  readonly extraArguments?: readonly string[];
},): Promise<void> {
  /**
   The recipe's own entry point, in the sibling package that owns the guest:
   this folder is four levels below `package/`, which is where that package sits.
   */
  const script = resolve(
    import.meta.dirname,
    '..',
    '..',
    '..',
    '..',
    'zfs',
    'request-to-openzfs-kernel-device',
    'e2e',
    'run-fixture.ts',
  );
  await runInheriting({
    subject: `the guest recipe's ${step} step`,
    command: 'node',
    args: [
      script,
      step,
      '--name',
      chosen.name,
      '--log-dir',
      chosen.logDir,
      ...extraArguments,
    ],
    timeout: recipeTimeoutMilliseconds,
  },);
}

/**
 Push this checkout into a guest as one archive, ready for the guest to unpack
 and build.

 The snapshot is taken in the run's own log folder, so a build somebody has to
 look into afterwards sits beside the logs of the commands that made it.

 @param chosen - the run's guest and log folder

 @throws Error naming the command that refused

 @example
 ```ts
 await pushWorktreeSnapshot({ chosen });
 ```
 */
export async function pushWorktreeSnapshot({
  chosen,
}: {
  readonly chosen: GuestRun;
},): Promise<void> {
  /**
   The folder the snapshot is taken in.
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
      mode: 0o700,
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
