/**
 The right to run one campaign: a kernel advisory lock on the campaign's evidence directory.

 The lock is held by a `flock` process that lives exactly as long as the campaign holds it. The kernel grants it to one
 holder at a time and drops it when that process ends, whatever ended it, so there is no lock file to go stale, no
 judgement about whether a holder is still alive, and no takeover to race: a run killed by `systemctl stop`, a reboot or
 `SIGKILL` simply no longer holds it. The holder reads its standard input and ends when that is closed, so a campaign that
 dies, or releases the lock, ends the holder too.

 It is needed even though the parallel launcher runs each campaign as a uniquely named systemd unit: a run started by hand,
 or by a second launcher under another label for the same file, names the same evidence directory, and the runner removes
 an orphaned container of the campaign's name before it creates its own, which would kill a live run's container.
 */
import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { once } from 'node:events';
import { stat } from 'node:fs/promises';

import {
  flockProgram,
  slotConflictExitCode,
} from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';

/**
 Shell program the holder runs once it has the lock: it says so, then keeps the lock until its input closes.
 */
const holderProgram = 'echo held; exec cat';

/**
 What the holder wrote first, when it has the lock.
 */
const heldSignal = 'held';

/**
 Exit status reported for a holder that ended without one, because a signal or an error ended it.
 */
const noStatus = -1;

/**
 Wait for the first thing the holder writes.

 @param holder - the `flock` process

 @returns the signal that it holds the lock, or no signal when its output ended without any

 @example
 ```ts
 await firstOutput(holder);
 ```
 */
async function firstOutput(holder: ChildProcess): Promise<readonly string[]> {
  try {
    await once(
      holder.stdout ?? holder,
      'data',
    );
    return [heldSignal];
  }
  catch (error) {
    console.error(`The lock holder's output failed: ${String(error)}`);
    return [];
  }
}

/**
 Wait for the holder to end.

 @param holder - the `flock` process

 @returns its exit status, or {@link noStatus} when a signal or an error ended it

 @example
 ```ts
 await exitStatusOf(holder);
 ```
 */
async function exitStatusOf(holder: ChildProcess): Promise<number> {
  try {
    /**
     The arguments of the close event: the exit code, which is not a number when a signal ended the holder.
     */
    const closed: readonly unknown[] = await once(
      holder,
      'close',
    );
    /**
     The exit code.
     */
    const [code] = closed;
    return (typeof code) === 'number' ? code : noStatus;
  }
  catch (error) {
    console.error(`The lock holder could not run: ${String(error)}`);
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

 @returns the held lock; declare it with `await using` so it is released when the campaign ends

 @throws CargoMutantsToolingError when another run of the same campaign holds the lock, the directory does not exist, or
 `flock` cannot be run

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
   The holder: `flock` on the directory, without waiting, running a program that waits for its input to close.
   */
  const holder = spawn(
    flockProgram,
    [
      '--nonblock',
      `--conflict-exit-code=${String(slotConflictExitCode)}`,
      evidence,
      'sh',
      '-c',
      holderProgram,
    ],
    {
      stdio: [
        'pipe',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Settle when the holder ends, whatever the reason, and when it writes; both are attached before anything is awaited.
   */
  const ended = exitStatusOf(holder);
  /**
   Standard error of the holder, to explain a refusal.
   */
  const errors = textOf(holder.stderr);
  /**
   The first thing that happens: the holder says it holds the lock, or it ends.
   */
  const first = await Promise.race([
    firstOutput(holder),
    ended,
  ]);
  if ((typeof first) === 'number') {
    throw new CargoMutantsToolingError(first === slotConflictExitCode
      ? `Another run of this campaign is alive (it holds the lock on ${evidence}); wait for it or stop it, then run this again.`
      : `Could not take the lock on ${evidence} with ${flockProgram} (status ${String(first)}): ${(await errors).trim()}`);
  }
  holder.stdout
    ?.resume();
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      holder.stdin
        ?.end();
      await ended;
    },
  };
}
