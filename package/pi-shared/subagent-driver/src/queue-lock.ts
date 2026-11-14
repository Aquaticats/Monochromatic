/**
 One writer at a time to a queue file: the lock a read, change and write of the queue's entries is held under, taken so that exactly
 one caller takes it, and given back when the block it was taken in ends.

 A queue is one JSON array whose order is explicit, and a coordinator reads and edits that file by eye, so its shape stays the file
 it is; what it lacked was one owner for the read-change-write. A change is milliseconds on this machine, the lock included
 (measured: 50 changes to a 51-entry queue took 0.66 ms each on average, 1.7 ms at the 95th percentile and 2.1 ms at worst), so the
 lock is held for a few file operations and no caller waits for work.

 The bounds, and what each is read against:

 - {@link QUEUE_LOCK_WAIT_MILLISECONDS} is how long a caller keeps trying before it refuses and says who holds the queue. Ten seconds
   is about five thousand times the worst measured change, so a waiter that reaches it is waiting on a holder that is not doing the
   work, and naming that holder is more use than waiting longer.
 - {@link QUEUE_LOCK_POLL_MILLISECONDS} is the pause between attempts, longer than a change, so a waiter usually takes the lock on
   its next attempt rather than spinning on the filesystem.
 - {@link QUEUE_LOCK_STALE_MILLISECONDS} is the age past which a lock file cannot belong to a live critical section: ten minutes is
   about three hundred thousand times the worst measured change. It is the last resort for a lock whose holder's process number now
   belongs to another process, or whose holder was stopped and never resumed.

 A holder that was killed leaves its lock file and no live process, and the next caller breaks it at once: a process that is gone
 cannot hold a queue. A lock file that names no process could only have been damaged, because this module writes the holder to a file
 beside the lock and links that file into place whole, so it is broken too.
 */

import { randomUUID, } from 'node:crypto';
import {
  link,
  readFile,
  rename,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import { setTimeout as sleep, } from 'node:timers/promises';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import { errorCodeOf, } from './error-code-of.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The code the operating system gives a path that is there already, which is a lock another caller holds.
 */
const EXISTS_CODE = 'EEXIST';

/**
 The code the operating system gives one process's number when no process of the account has it.
 */
const NO_SUCH_PROCESS_CODE = 'ESRCH';

/**
 The end of the file beside a queue that says which process holds it.
 */
const LOCK_FILE_SUFFIX = '.lock';

/**
 The start of the name of a file written beside another and linked or renamed into place, which is never read as anything.
 */
const BEING_WRITTEN_START = '.being-written-';

/**
 The start of the name a broken lock is renamed to before it is removed, so only one breaker wins the rename.
 */
const BROKEN_LOCK_START = '.broken-';

/**
 How long a caller keeps trying to take a queue's lock before it refuses, in milliseconds.
 */
export const QUEUE_LOCK_WAIT_MILLISECONDS = 10_000;

/**
 The pause between two attempts to take a queue's lock, in milliseconds.
 */
export const QUEUE_LOCK_POLL_MILLISECONDS = 10;

/**
 The age past which a lock file cannot belong to a live critical section, in milliseconds.
 */
export const QUEUE_LOCK_STALE_MILLISECONDS = 600_000;

/**
 The queue is held by another writer, and this caller's wait ran out, so nothing was changed.
 */
export class QueueLockHeldError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'QueueLockHeldError';
}

/**
 A queue's lock while a caller changes the queue: it is given back when the block that took it ends, whether that block answered or
 threw.
 */
export type TheQueueLock = {
  /**
   Removes the lock file, which lets the next writer take the queue.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 What the file beside a queue says about its holder.
 */
type HolderOfAQueue = {
  /**
   The process number the holder wrote, when the file names one.
   */
  readonly pid?: number;

  /**
   When the holder took the lock, when the file names a time.
   */
  readonly at?: string;
};

/**
 What a lock file's text was read as: what it holds, or that it holds nothing a caller can read.
 */
type ReadLock =
  | { readonly parsed: unknown }
    | { readonly unreadable: true };

/**
 What a look at a lock file found.
 */
type StateOfALock = {
  /**
   Whether a lock that holds no live critical section may be broken.
   */
  readonly broken: boolean;

  /**
   Why it may be broken, for a caller's line.
   */
  readonly reason?: string;
};

/**
 Reads the path of the file beside a queue that says which process holds it.

 @param queueFile - the queue's path

 @returns the lock file's path

 @example
 ```ts
 const lockFile = lockFileOfAQueue({ queueFile: '/runs/queue.json' });
 ```
 */
export function lockFileOfAQueue({ queueFile, }: { readonly queueFile: string },): string {
  return `${queueFile}${LOCK_FILE_SUFFIX}`;
}

/**
 Reads a path one operation writes beside the queue and then puts into place, which no other writer of this machine picks.

 The name carries the process number and a random identifier, because two writes of one process once shared a name and one of them
 failed on the rename.

 @param path - the path the operation writes beside

 @returns a path beside it

 @example
 ```ts
 const beside = aPathWrittenBeside({ path: '/runs/queue.json' });
 ```
 */
export function aPathWrittenBeside({ path, }: { readonly path: string },): string {
  return `${path}${BEING_WRITTEN_START}${String(process.pid,)}-${randomUUID()}`;
}

/**
 Links a complete file into a lock's place, which fails when another caller holds the lock.

 @param beside - the complete file written beside the lock

 @param lockFile - the lock's path

 @returns whether this caller took the lock, or another holds it

 @example
 ```ts
 const taken = await linkTheLockIntoPlace({ beside: '/runs/queue.json.lock.being-written-1-2', lockFile: '/runs/queue.json.lock' });
 ```
 */
async function linkTheLockIntoPlace({
  beside,
  lockFile,
}: {
  readonly beside: string;
  readonly lockFile: string;
},): Promise<boolean> {
  try {
    await link(
      beside,
      lockFile,
    );
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === EXISTS_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 Writes a lock file's content beside the lock and links it into place whole, so no reader meets a half-written lock.

 @param lockFile - the lock's path

 @param content - what the lock should say

 @returns whether this caller took the lock, or another holds it

 @example
 ```ts
 const taken = await takeTheLockFile({ lockFile: '/runs/queue.json.lock', content: '{"pid":1}' });
 ```
 */
async function takeTheLockFile({
  lockFile,
  content,
}: {
  readonly lockFile: string;
  readonly content: string;
},): Promise<boolean> {
  /**
   The file written beside the lock and linked into its place.
   */
  const beside = aPathWrittenBeside({ path: lockFile, },);
  await writeFile(
    beside,
    content,
  );
  try {
    /**
     Whether this caller took the lock, or another holds it.
     */
    const taken = await linkTheLockIntoPlace({
      beside,
      lockFile,
    },);
    await rm(
      beside,
      { force: true, },
    );
    return taken;
  }
  catch (error) {
    // The file beside the lock is removed before the failure is carried out, so a lock that could not be taken leaves nothing.
    await rm(
      beside,
      { force: true, },
    );
    throw error;
  }
}

/**
 Reads a lock file's text as JSON, or answers that it holds nothing a caller can read.

 @param lockFile - the lock's path

 @returns what it holds, or that it is gone or damaged

 @example
 ```ts
 const read = await parsedLockIn({ lockFile: '/runs/queue.json.lock' });
 ```
 */
async function parsedLockIn({ lockFile, }: { readonly lockFile: string },): Promise<ReadLock> {
  try {
    return {
      parsed: JSON.parse(await readFile(
        lockFile,
        'utf8',
      ),),
    };
  }
  catch (error) {
    // A lock that is not there is one its holder gave back, and one that is not JSON is damaged: neither names a live holder.
    if ((errorCodeOf(error,) === MISSING_PATH_CODE) || (error instanceof SyntaxError)) {
      return { unreadable: true, };
    }
    throw error;
  }
}

/**
 Reads what the file beside a queue says about its holder.

 @param lockFile - the lock's path

 @returns the process number and the time the file names, and nothing when it names neither

 @example
 ```ts
 const holder = await holderOfAQueue({ lockFile: '/runs/queue.json.lock' });
 ```
 */
async function holderOfAQueue({ lockFile, }: { readonly lockFile: string },): Promise<HolderOfAQueue> {
  /**
   What the lock file holds.
   */
  const read = await parsedLockIn({ lockFile, },);
  if ('unreadable' in read) {
    return {};
  }
  /**
   What the file holds, of any shape.
   */
  const parsed: unknown = read.parsed;
  if (((typeof parsed) !== 'object') || (parsed === null)) {
    return {};
  }
  /**
   The process number the file names, when it names one.
   */
  const pid: unknown = Reflect.get(
    parsed,
    'pid',
  );
  /**
   The time the file names, when it names one.
   */
  const at: unknown = Reflect.get(
    parsed,
    'at',
  );
  return {
    ...(((typeof pid) === 'number') ? { pid, } : {}),
    ...(((typeof at) === 'string') ? { at, } : {}),
  };
}

/**
 Says whether the process a lock names is still there.

 @param pid - the process number the lock names

 @returns whether it is there, which a process this account may not signal counts as

 @example
 ```ts
 const there = processIsThere({ pid: 1 });
 ```
 */
function processIsThere({ pid, }: { readonly pid: number },): boolean {
  try {
    process.kill(
      pid,
      0,
    );
    return true;
  }
  catch (error) {
    // A process number this account may not signal still names a live process, so only "no such process" says the holder is gone.
    return errorCodeOf(error,) !== NO_SUCH_PROCESS_CODE;
  }
}

/**
 Reads how old a lock file is.

 @param lockFile - the lock's path

 @param now - the clock the age is read from

 @returns the age, or that the lock is gone

 @example
 ```ts
 const age = await ageOfALock({ lockFile: '/runs/queue.json.lock', now: Date.now });
 ```
 */
async function ageOfALock({
  lockFile,
  now,
}: {
  readonly lockFile: string;
  readonly now: () => number;
},): Promise<{ readonly ageMilliseconds: number } | { readonly gone: true }> {
  try {
    return {
      ageMilliseconds: now() - (await stat(lockFile,)).mtimeMs,
    };
  }
  catch (error) {
    // A lock that vanished between the read and the look is one its holder gave back, which the next attempt finds free.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return { gone: true, };
    }
    throw error;
  }
}

/**
 Reads whether a lock holds no live critical section, and so may be broken.

 @param lockFile - the lock's path

 @param holder - what the lock file says about its holder

 @param now - the clock the lock's age is read from

 @returns whether the lock may be broken, and the reason a caller's line gives

 @example
 ```ts
 const breaker = await lockMayBeBroken({ lockFile: '/runs/queue.json.lock', holder, now: Date.now });
 ```
 */
async function lockMayBeBroken({
  lockFile,
  holder,
  now,
}: {
  readonly lockFile: string;
  readonly holder: HolderOfAQueue;
  readonly now: () => number;
},): Promise<StateOfALock> {
  if (holder.pid === undefined) {
    return {
      broken: true,
      reason: 'it names no process, which only a damaged lock file does',
    };
  }
  if (!processIsThere({ pid: holder.pid, },)) {
    return {
      broken: true,
      reason: `its process ${String(holder.pid,)} is gone`,
    };
  }
  /**
   How old the lock is, or that it went away.
   */
  const age = await ageOfALock({
    lockFile,
    now,
  },);
  if ('gone' in age) {
    return { broken: false, };
  }
  return (age.ageMilliseconds > QUEUE_LOCK_STALE_MILLISECONDS)
    ? {
      broken: true,
      reason: `it is ${String(Math.round(age.ageMilliseconds / MILLISECONDS_PER_SECOND,),)} s old, older than any change to a queue could be`,
    }
    : { broken: false, };
}

/**
 Breaks a lock that holds no live critical section, by renaming it to a name of this caller's own and removing that.

 @param lockFile - the lock's path

 @returns whether this caller broke it, or another caller did first

 @example
 ```ts
 const broken = await breakTheLockFile({ lockFile: '/runs/queue.json.lock' });
 ```
 */
async function breakTheLockFile({ lockFile, }: { readonly lockFile: string },): Promise<boolean> {
  /**
   The name this caller renames the lock to, which only one caller can win.
   */
  const broken = `${lockFile}${BROKEN_LOCK_START}${String(process.pid,)}-${randomUUID()}`;
  try {
    await rename(
      lockFile,
      broken,
    );
  }
  catch (error) {
    // Another caller renamed it first, so there is nothing left to break.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
  await rm(
    broken,
    { force: true, },
  );
  return true;
}

/**
 Waits, bounded, for one caller to take a queue's lock, and answers the lock it holds.

 @param queueFile - the queue whose lock is taken

 @param now - the clock the wait and the lock's age are read from

 @param wait - the pause between two attempts

 @param waitMilliseconds - how long to keep asking before refusing

 @returns the lock, given back when the block that took it ends

 @throws QueueLockHeldError when another writer holds it for the whole wait, naming that writer and what may be done

 @example
 ```ts
 await using lock = await acquireTheQueueLock({ queueFile: '/runs/queue.json' });
 const entries = await readQueueOfRuns({ queueFile: '/runs/queue.json' });
 ```
 */
export async function acquireTheQueueLock({
  queueFile,
  now = Date.now,
  wait = sleep,
  waitMilliseconds = QUEUE_LOCK_WAIT_MILLISECONDS,
}: {
  readonly queueFile: string;
  readonly now?: () => number;
  readonly wait?: (milliseconds: number) => Promise<unknown>;
  readonly waitMilliseconds?: number;
},): Promise<TheQueueLock> {
  /**
   The lock's path.
   */
  const lockFile = lockFileOfAQueue({ queueFile, },);
  /**
   The moment this caller stops asking.
   */
  const deadline = now() + waitMilliseconds;
  /* oxlint-disable eslint/no-await-in-loop -- one attempt, then a pause, then the next attempt: taking a lock is a sequence of
     attempts by definition, and each reads what the attempt before it left. See doc/lint/sequential-awaits.md. */
  for (;;) {
    if (await takeTheLockFile({
      lockFile,
      content: `${JSON.stringify({
        pid: process.pid,
        at: new Date(now(),).toISOString(),
      },)}\n`,
    },)) {
      return {
        async [Symbol.asyncDispose]() {
          await rm(
            lockFile,
            { force: true, },
          );
        },
      };
    }
    /**
     What the lock says about the caller that holds it.
     */
    const holder = await holderOfAQueue({ lockFile, },);
    /**
     Whether that lock holds no live critical section.
     */
    const breaker = await lockMayBeBroken({
      lockFile,
      holder,
      now,
    },);
    if (breaker.broken) {
      // The holder that would care is gone, so the breaking is silent and the caller simply goes on: the reason is in
      // `lockMayBeBroken` for a reader of this code, and a lock that named no live process cannot hold the queue.
      await breakTheLockFile({ lockFile, },);
      continue;
    }
    if (now() >= deadline) {
      throw new QueueLockHeldError(
        `the queue at ${queueFile} was not changed: another writer holds it, process ${String(holder.pid,)} since ${holder.at ?? 'an unknown time'}. `
          + `Its lock file is ${lockFile}; if that process is gone or stuck, the file can be removed.`,
      );
    }
    await wait(QUEUE_LOCK_POLL_MILLISECONDS,);
  }
  /* oxlint-enable eslint/no-await-in-loop */
}
