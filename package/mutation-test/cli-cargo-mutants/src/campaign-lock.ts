/**
 The right to run one campaign: a kernel advisory lock on the campaign's evidence directory.

 The campaign process itself holds the lock. It opens the directory and lends the open descriptor to a short-lived `flock`
 process, which takes the lock on it and ends; a lock belongs to the open file description, which the campaign process
 still has open, so the lock stays held exactly as long as that descriptor, and with it the process, is open. The kernel
 grants it to one holder at a time and drops it when the process ends, whatever ended it, so there is no lock file to go
 stale, no judgement about whether a holder is still alive, no takeover to race, and no helper whose death could release the
 lock while the campaign goes on: a run killed by `systemctl stop`, a reboot or `SIGKILL` simply no longer holds it.

 It is needed even though the parallel launcher runs each campaign as a uniquely named systemd unit: a run started by hand,
 or by a second launcher under another label for the same file, names the same evidence directory, and the runner removes
 an orphaned container of the campaign's name before it creates its own, which would kill a live run's container.
 */
import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { once } from 'node:events';
import {
  type FileHandle,
  open,
  stat,
} from 'node:fs/promises';

import {
  flockProgram,
  slotConflictExitCode,
} from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';

/**
 Number of the descriptor the lent directory has in the `flock` process: the first after standard input, output and error.
 */
const lentDescriptor = 3;

/**
 Exit status reported for a `flock` process that ended without one, because a signal or an error ended it.
 */
const noStatus = -1;

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

 @param stream - the holder's standard error

 @returns its text

 @example
 ```ts
 await textOf(holder.stderr);
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
 Tell whether a path is an existing directory.

 @param path - the path

 @returns false for a path that does not exist or is not a directory

 @example
 ```ts
 await isDirectory('/e/native-mutation-abc');
 ```
 */
async function isDirectory(path: string): Promise<boolean> {
  try {
    return (await stat(path)).isDirectory();
  }
  catch (error) {
    console.error(`Cannot examine ${path}: ${String(error)}`);
    return false;
  }
}

/**
 Take the right to run one campaign, so that no second run writes the same evidence directory.

 @param evidence - the campaign's evidence directory, which must exist

 @returns the held lock; declare it with `await using` so it is released when the campaign ends, and keep it for as long as
 the campaign runs, because the lock is the open descriptor it holds

 @throws CargoMutantsToolingError when another run of the same campaign holds the lock, the directory does not exist, or
 `flock` cannot be run or ends in any way but success or a conflict

 @example
 ```ts
 await using lock = await acquireCampaignLock('/e/native-mutation-0123456789ab');
 ```
 */
export async function acquireCampaignLock(evidence: string): Promise<AsyncDisposable> {
  // `flock` creates a file that is not there, and a lock on that would guard nothing.
  if (!(await isDirectory(evidence)))
    throw new CargoMutantsToolingError(`${evidence} is not a directory, so there is nothing to lock.`);
  /**
   The directory, opened here: this descriptor, not a helper, is what holds the lock once it is taken.
   */
  const directory: FileHandle = await open(
    evidence,
    'r',
  );
  try {
    /**
     `flock` on the lent descriptor, without waiting: it takes the lock for the open file description and ends.
     */
    const locker = spawn(
      flockProgram,
      [
        '--nonblock',
        `--conflict-exit-code=${String(slotConflictExitCode)}`,
        String(lentDescriptor),
      ],
      {
        stdio: [
          'ignore',
          'ignore',
          'pipe',
          directory.fd,
        ],
      },
    );
    /**
     Standard error of `flock`, to explain a refusal.
     */
    const errors = textOf(locker.stderr);
    /**
     The only thing that counts as evidence of the lock: `flock` ended with success. Output, or the lack of it, proves nothing.
     */
    const status = await exitStatusOf(locker);
    if (status === slotConflictExitCode)
      throw new CargoMutantsToolingError(`Another run of this campaign is alive (it holds the lock on ${evidence}); wait for it or stop it, then run this again.`);
    if (status !== 0)
      throw new CargoMutantsToolingError(`Could not take the lock on ${evidence} with ${flockProgram} (status ${String(status)}): ${(await errors).trim()}`);
  }
  catch (error) {
    // Nothing was granted, or it cannot be known: give the descriptor back, so a lock that was half taken is not kept.
    await directory.close();
    throw error;
  }
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await directory.close();
    },
  };
}
