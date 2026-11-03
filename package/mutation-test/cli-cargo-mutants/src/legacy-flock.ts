/**
 A one-time probe of the kernel advisory lock the earlier release of this package took, for as long as a process of that
 release may still run.

 The earlier campaign lock was a `flock` on the campaign's evidence directory, and the earlier gate slot was a `flock` on
 `slot-<n>.lock` held by the process that ran the gated command. Neither is taken any more (the numbered claim files of
 `src/owner-claim.ts` replace them), but a campaign or gate that started before the change may still be running, and two
 runs must never share one item, whichever release they are. The probe asks without waiting and gives the lock straight back
 (`package/git-policy/cli/src/native/worktree_convert_journal.rs:268`, `legacy_lock_is_held`): a held lock is a live owner,
 and a lock file nobody holds is removed while the probe still holds it, so no earlier command can take the name between the
 probe and the removal and the leftover disappears after the upgrade. A file that is absent costs nothing, and a file that is
 not a plain file is left where it is.

 The lock belongs to the open file description, which this process keeps open while a short-lived `flock` process, lent the
 descriptor, takes the lock on it and ends. That is the handshake the earlier campaign lock used.
 */
import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { once } from 'node:events';
import { constants } from 'node:fs';
import {
  type FileHandle,
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
 Ask once, without waiting, whether a process of the earlier release holds the kernel lock on a path.

 @param path - the lock file (`slot-<n>.lock`) or the evidence directory the earlier release locked

 @param retireWhenFree - true to remove a plain lock file nobody holds, false to keep whatever the path is

 @param program - the `flock` program; absent to use the one on the search path

 @returns what the probe found

 @throws CargoMutantsToolingError when the path cannot be opened or `flock` cannot be run or ends unexpectedly; nothing is
 removed then

 @example
 ```ts
 const standing = await probeLegacyLock({ path: '/l/slot-1.lock', retireWhenFree: true });
 ```
 */
export async function probeLegacyLock({
  path,
  retireWhenFree,
  program = flockProgram,
}: {
  readonly path: string;
  readonly retireWhenFree: boolean;
  readonly program?: string;
}): Promise<LegacyStanding> {
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
   What the path is.
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
