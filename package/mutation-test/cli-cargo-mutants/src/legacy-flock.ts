/**
 A one-time probe of the kernel advisory lock the earlier release of this package took, for as long as a process of that
 release may still run.

 The earlier campaign lock was a `flock` on the campaign's evidence directory, and the earlier gate slot was a `flock` on
 `slot-<n>.lock` held by the process that ran the gated command. Neither is taken any more (the numbered claim files of
 `src/owner-claim.ts` replace them), but a campaign or gate that started before the change may still be running, and two
 runs must never share one item, whichever release they are. The probe asks without waiting and gives the lock straight back
 (`package/git-policy/cli/src/native/worktree/worktree_convert_journal.rs:268`, `legacy_lock_is_held`): a held lock is a live owner,
 and a lock file nobody holds is removed while the probe still holds it, so no earlier command can take the name between the
 probe and the removal and the leftover disappears after the upgrade. A file that is absent costs nothing, and a file that is
 not a plain file is left where it is.

 The lock belongs to the open file description, which this process keeps open while a short-lived `flock` process, lent the
 descriptor, takes the lock on it and ends. That is the handshake the earlier campaign lock used.

 A gate of the earlier release that is still waiting keeps polling its slot file with `flock --nonblock` and never looks at
 a claim file, and a campaign of it locks its evidence directory the same way, so a probe that is released again cannot keep
 such a run out. {@link holdLegacyLock} therefore keeps the lock for as long as the claim is held, and never removes the file,
 because a probe that removes a file races with a run of the earlier release that has just created and locked a new one.
 */
import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { once } from 'node:events';
import {
  constants,
  type Stats,
} from 'node:fs';
import {
  type FileHandle,
  lstat,
  open,
  unlink,
} from 'node:fs/promises';

import {
  flockProgram,
  slotConflictExitCode,
} from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';
import { isMissing } from './fs-guards.ts';

/**
 Number of the descriptor the lent file has in the `flock` process: the first after standard input, output and error.
 */
const lentDescriptor = 3;

/**
 Exit status reported for a `flock` process that ended without one, because a signal or an error ended it.
 */
const noStatus = -1;

/**
 What the probe found: `absent` (no such file, or one that is not a plain file and is left alone), `held` (a process of the
 earlier release holds the lock), `free` (nobody holds it and it was kept), or `retired` (nobody held it and the file was
 removed).
 */
export type LegacyStanding = 'absent' | 'held' | 'free' | 'retired';

/**
 Wait for the `flock` process to end.

 @param locker - the `flock` process

 @returns its exit status, or {@link noStatus} when a signal or an error ended it

 @example
 ```ts
 await exitStatusOf(locker);
 ```
 */
async function exitStatusOf(locker: ChildProcess): Promise<number> {
  try {
    /**
     The arguments of the close event: the exit code, which is not a number when a signal ended it.
     */
    const closed: readonly unknown[] = await once(
      locker,
      'close',
    );
    /**
     The exit code.
     */
    const [code] = closed;
    return (typeof code) === 'number' ? code : noStatus;
  }
  catch (error) {
    console.error(`The flock process could not run: ${String(error)}`);
    return noStatus;
  }
}

/**
 Read a stream to its end.

 @param stream - the process's standard error

 @returns its text

 @example
 ```ts
 await textOf(locker.stderr);
 ```
 */
async function textOf(stream: ChildProcess['stderr']): Promise<string> {
  if (stream === null)
    return '';
  stream.setEncoding('utf8');
  /**
   Chunks as they arrive.
   */
  const chunks: string[] = [];
  for await (const chunk of stream)
    chunks.push(String(chunk));
  return chunks.join('');
}

/**
 Try the lock on a descriptor without waiting.

 @param descriptor - the open file's descriptor, lent to the `flock` process

 @param program - the `flock` program

 @returns true when the lock is held by another, false when this process now holds it

 @throws CargoMutantsToolingError when `flock` cannot be run or ends in any way but success or a conflict, because a lock
 that cannot be probed must stop the caller and never be read as free

 @example
 ```ts
 await lockIsHeld({ descriptor: 7, program: 'flock' });
 ```
 */
async function lockIsHeld({
  descriptor,
  program,
}: {
  readonly descriptor: number;
  readonly program: string;
}): Promise<boolean> {
  /**
   `flock` on the lent descriptor, without waiting: it takes the lock for the open file description and ends.
   */
  const locker = spawn(
    program,
    [
      '--nonblock',
      '--exclusive',
      `--conflict-exit-code=${String(slotConflictExitCode)}`,
      String(lentDescriptor),
    ],
    {
      stdio: [
        'ignore',
        'ignore',
        'pipe',
        descriptor,
      ],
    },
  );
  /**
   Standard error of `flock`, to explain a failure.
   */
  const errors = textOf(locker.stderr);
  /**
   The only thing that counts as evidence about the lock: how `flock` ended. Output, or the lack of it, proves nothing.
   */
  const status = await exitStatusOf(locker);
  if (status === slotConflictExitCode)
    return true;
  if (status !== 0)
    throw new CargoMutantsToolingError(`Could not probe the lock of the earlier release with ${program} (status ${String(status)}): ${(await errors).trim()}`);
  return false;
}

/**
 Open the path for the probe, never following a link.

 @param path - the lock file or directory

 @returns the open file, or the text `absent` when there is no such path

 @throws CargoMutantsToolingError when the path cannot be opened for any other reason

 @example
 ```ts
 await openForProbe('/l/slot-1.lock');
 ```
 */
async function openForProbe(path: string): Promise<FileHandle | 'absent'> {
  try {
    return await open(
      path,
      constants.O_RDONLY | constants.O_NONBLOCK
        | constants.O_NOFOLLOW,
    );
  }
  catch (error) {
    if (isMissing(error))
      return 'absent';
    throw new CargoMutantsToolingError(`Cannot open ${path} to probe the lock of the earlier release (${String(error)}); nothing was changed.`);
  }
}

/**
 Stands for a lock file whose path names another file than the one the probe locked.
 */
const pathChanged: unique symbol = Symbol('legacy-flock.path-changed');

/**
 Most times the probe starts over when the path keeps naming a new file, before it gives up.
 */
const mostProbeAttempts = 3;

/**
 What a path names now, by device and inode.

 @param path - the lock file

 @returns the file's status, or the text `absent` when there is no such path

 @throws CargoMutantsToolingError when the path cannot be examined for any other reason

 @example
 ```ts
 await statOnDisk('/l/slot-1.lock');
 ```
 */
async function statOnDisk(path: string): Promise<Stats | 'absent'> {
  try {
    return await lstat(path);
  }
  catch (error) {
    if (isMissing(error))
      return 'absent';
    throw new CargoMutantsToolingError(`Cannot examine ${path}, the lock file the earlier release left (${String(error)}).`);
  }
}

/**
 One probe: open the path, try the lock, and retire the file when asked and nobody holds it.

 @param path - the lock file or evidence directory
 
 @param retireWhenFree - true to remove a plain lock file nobody holds
 
 @param program - the `flock` program
 
 @param afterLocking - runs once the probe holds the lock, before the file is removed; for tests

 @returns what the probe found, or {@link pathChanged} when the path no longer names the file that was locked

 @example
 ```ts
 await probeOnce({ path: '/l/slot-1.lock', retireWhenFree: true, program: 'flock' });
 ```
 */
async function probeOnce({
  path,
  retireWhenFree,
  program,
  afterLocking,
}: {
  readonly path: string;
  readonly retireWhenFree: boolean;
  readonly program: string;
  readonly afterLocking?: () => Promise<void>;
}): Promise<LegacyStanding | typeof pathChanged> {
  /**
   The path, opened here: this descriptor, not the helper, holds the lock once it is taken.
   */
  const opened = await openForProbe(path);
  if (opened === 'absent')
    return 'absent';
  /**
   The opened file, closed when the probe ends, which gives the lock back.
   */
  await using file = opened;
  /**
   What the opened file is.
   */
  const status = await file.stat();
  // A path that is not a plain file is never a lock file the earlier release made, and is left where it is.
  if (retireWhenFree && (!status.isFile()))
    return 'absent';
  if (await lockIsHeld({
    descriptor: file.fd,
    program,
  }))
    return 'held';
  if (!retireWhenFree)
    return 'free';
  if (afterLocking !== undefined)
    await afterLocking();
  // Holding a lock does not protect the name: another probe may have retired this file and a gate of the earlier release
  // may have created and locked a new one at the same path, which must not be removed here.
  /**
   What the path names now.
   */
  const onDisk = await statOnDisk(path);
  if (onDisk === 'absent')
    return 'absent';
  if ((onDisk.dev !== status.dev) || (onDisk.ino !== status.ino))
    return pathChanged;
  try {
    // Removed while the probe still holds the lock, so no earlier command can take the name in between.
    await unlink(path);
  }
  catch (error) {
    if (!isMissing(error))
      throw new CargoMutantsToolingError(`Cannot remove ${path}, the lock file the earlier release left (${String(error)}).`);
  }
  return 'retired';
}

/**
 What {@link holdLegacyLock} found: `busy` (a process holds the lock), `absent` (no such path where none was asked to be
 created), or `kept` (this process now holds the lock, which it gives back when `lock` is disposed).
 */
export type LegacyHold =
  | { readonly kind: 'busy' }
    | { readonly kind: 'absent' }
    | {
      readonly kind: 'kept';
    /**
     Gives the kernel lock back when disposed.
     */
      readonly lock: AsyncDisposable;
    };

/**
 Permission bits of a lock file this process creates, before the process mask: what `flock` gives a file it creates.
 */
const lockFileMode = 0o666;

/**
 Open the path for a lock that is kept.

 @param path - the lock file or evidence directory

 @param create - true to create a missing plain lock file, as a poll of the earlier release does

 @returns the open file, or the text `absent` when there is no such path

 @throws CargoMutantsToolingError when the path cannot be opened for any other reason

 @example
 ```ts
 await openForHold({ path: '/l/slot-1.lock', create: true });
 ```
 */
async function openForHold({
  path,
  create,
}: {
  readonly path: string;
  readonly create: boolean;
}): Promise<FileHandle | 'absent'> {
  if (!create)
    return await openForProbe(path);
  try {
    return await open(
      path,
      constants.O_RDONLY | constants.O_NONBLOCK
        | constants.O_NOFOLLOW
        | constants.O_CREAT,
      lockFileMode,
    );
  }
  catch (error) {
    throw new CargoMutantsToolingError(`Cannot open or create ${path} to keep the lock of the earlier release (${String(error)}); nothing was changed.`);
  }
}

/**
 Take the kernel lock a run of the earlier release would take on a path, without waiting, and keep it.

 The file is opened (and created when asked), the lock is taken on the open file description, and the path is checked to still
 name that file, which it does because nothing here ever removes it. A run of the earlier release that polls the path then finds
 the lock held and keeps waiting, or refuses, as it does for any holder of its own release.

 @param path - the lock file (`slot-<n>.lock`) or the evidence directory

 @param create - true to create a missing plain lock file (a slot); false to lock only what exists (an evidence directory)

 @param program - the `flock` program; absent to use the one on the search path

 @param attemptsLeft - how many more times this may start over when the path names a new file; absent in use

 @returns `busy` when a process holds the lock, `absent` when there is nothing to lock, or `kept` with the lock to dispose

 @throws CargoMutantsToolingError when the path cannot be opened, or is not a plain file where a plain file was asked for, or
 `flock` cannot be run or ends unexpectedly, or the path keeps naming new files

 @example
 ```ts
 const hold = await holdLegacyLock({ path: '/l/slot-1.lock', create: true });
 ```
 */
export async function holdLegacyLock({
  path,
  create,
  program = flockProgram,
  attemptsLeft = mostProbeAttempts,
}: {
  readonly path: string;
  readonly create: boolean;
  readonly program?: string;
  readonly attemptsLeft?: number;
}): Promise<LegacyHold> {
  if (attemptsLeft < 1)
    throw new CargoMutantsToolingError(`${path} keeps being replaced by another lock file while its lock is taken; nothing was changed. Run the command again.`);
  /**
   The path, opened here: this descriptor holds the lock once it is taken.
   */
  const opened = await openForHold({
    path,
    create,
  });
  if (opened === 'absent')
    return { kind: 'absent' };
  /**
   Closes the file, which gives the lock back, unless the lock is kept and the stack moved to the caller.
   */
  await using stack = new AsyncDisposableStack();
  /**
   The opened file, in the stack.
   */
  const file = stack.use(opened);
  /**
   What the opened file is.
   */
  const status = await file.stat();
  // A path that is not a plain file is never a lock file the earlier release made (it makes only plain ones), so where a plain
  // file is wanted it is an error to fix, never a holder to wait for; a directory is wanted only where asked not to create.
  if (create && (!status.isFile()))
    throw new CargoMutantsToolingError(`${path} is not a plain file, and no run of the earlier release makes anything else at a slot file, so it cannot hold the slot and nothing was changed. Move it away or remove it, then run the command again.`);
  if (await lockIsHeld({
    descriptor: file.fd,
    program,
  }))
    return { kind: 'busy' };
  /**
   What the path names now.
   */
  const onDisk = await statOnDisk(path);
  if ((onDisk === 'absent') || (onDisk.dev !== status.dev)
    || (onDisk.ino !== status.ino))
    return await holdLegacyLock({
      path,
      create,
      program,
      attemptsLeft: attemptsLeft - 1,
    });
  return {
    kind: 'kept',
    lock: stack.move(),
  };
}

/**
 Ask once, without waiting, whether a process of the earlier release holds the kernel lock on a path.

 @param path - the lock file (`slot-<n>.lock`) or the evidence directory the earlier release locked

 @param retireWhenFree - true to remove a plain lock file nobody holds, false to keep whatever the path is

 @param program - the `flock` program; absent to use the one on the search path

 @param afterLocking - runs once the probe holds the lock, before the file is removed; a test lets another command act there

 @param attemptsLeft - how many more times the probe may start over when the path names a new file; absent in use

 @returns what the probe found

 @throws CargoMutantsToolingError when the path cannot be opened or `flock` cannot be run or ends unexpectedly, or the path
 keeps naming new files; nothing is removed then

 @example
 ```ts
 const standing = await probeLegacyLock({ path: '/l/slot-1.lock', retireWhenFree: true });
 ```
 */
export async function probeLegacyLock({
  path,
  retireWhenFree,
  program = flockProgram,
  afterLocking,
  attemptsLeft = mostProbeAttempts,
}: {
  readonly path: string;
  readonly retireWhenFree: boolean;
  readonly program?: string;
  readonly afterLocking?: () => Promise<void>;
  readonly attemptsLeft?: number;
}): Promise<LegacyStanding> {
  if (attemptsLeft < 1)
    throw new CargoMutantsToolingError(`${path} keeps being replaced by another lock file while it is probed; nothing was removed. Run the command again.`);
  /**
   What one probe found.
   */
  const found = await probeOnce({
    path,
    retireWhenFree,
    program,
    ...(afterLocking === undefined ? {} : { afterLocking }),
  });
  if (found !== pathChanged)
    return found;
  return await probeLegacyLock({
    path,
    retireWhenFree,
    program,
    ...(afterLocking === undefined ? {} : { afterLocking }),
    attemptsLeft: attemptsLeft - 1,
  });
}
