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

 No age breaks a lock. Age says nothing about whether a holder is alive: a review's brief or a slow volume can hold a change for
 longer than any number a reader picks, and taking the lock from a live holder is what let one holder write the list it had read
 over the change that replaced it, losing that change while both callers answered success (reproduced on 2026-11-15 against the
 queue of that day). What is read instead is identity: the lock names its holder's process and the start time of that process, and
 a process number that now belongs to a different process is a number reused after its holder died.

 A holder that was killed leaves its lock file and no live process, and the next caller breaks it at once: a process that is gone
 cannot hold a queue. A lock file that names no process could only have been damaged, because this module writes the holder to a
 file beside the lock and links that file into place whole, so it is broken too. A lock naming a live process whose start time is
 the recorded one is never taken, however old it is: the wait runs out and the refusal names that process, and removing the file in
 hand is the remedy for a holder that is stopped and never resumed.

 A lock whose holder is broken while it still lives is harmless, because the lock carries a token: a caller's release removes the
 lock only while it carries that token, and a write of the queue under the lock first reads the lock and refuses when the token is
 not this caller's, so the replaced caller's change is refused rather than written over the taker's.
 */

import { randomUUID, } from 'node:crypto';
import {
  link,
  readFile,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import { setTimeout as sleep, } from 'node:timers/promises';
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
 The folder a Linux machine keeps one folder per process in, which is where the start time of a process is read.
 */
const PROCESS_FOLDER = '/proc';

/**
 Which field of a process's own statistics holds when it started, counted in clock ticks since the machine came up.
 */
const PROCESS_START_TIME_FIELD = 22;

/**
 The column a process's statistics separate its name from its other fields with, which the name itself may hold.
 */
const PROCESS_STAT_NAME_END = ')';

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
 The lock that was taken is no longer this caller's, so the queue this caller read is not the queue it would write.
 */
export class QueueLockLostError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'QueueLockLostError';
}

/**
 A queue's lock while a caller changes the queue: it is given back when the block that took it ends, whether that block answered or
 threw.
 */
export type TheQueueLock = {
  /**
   The lock file's path.
   */
  readonly lockFile: string;

  /**
   The token this caller's lock carries, which no other caller's lock carries.
   */
  readonly token: string;

  /**
   Removes the lock file while it still carries this caller's token, which lets the next writer take the queue.
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

  /**
   When the holder's process started, when the file names it.
   */
  readonly startedAt?: string;

  /**
   The token the holder wrote, when the file names one.
   */
  readonly token?: string;
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
  /**
   When the holder's process started, when the file names it.
   */
  const startedAt: unknown = Reflect.get(
    parsed,
    'startedAt',
  );
  /**
   The token the holder wrote, when the file names one.
   */
  const token: unknown = Reflect.get(
    parsed,
    'token',
  );
  return {
    ...(((typeof pid) === 'number') ? { pid, } : {}),
    ...(((typeof at) === 'string') ? { at, } : {}),
    ...(((typeof startedAt) === 'string') ? { startedAt, } : {}),
    ...(((typeof token) === 'string') ? { token, } : {}),
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
export function processIsThere({ pid, }: { readonly pid: number },): boolean {
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
 Reads when one process started, which tells a process from a process number reused after its holder died.

 @param pid - the process number

 @returns the start time the kernel counted, or nothing when no process of that number is there or its statistics cannot be read

 @example
 ```ts
 const startedAt = await processStartedAtOf({ pid: process.pid });
 ```
 */
export async function processStartedAtOf({ pid, }: { readonly pid: number },): Promise<string | undefined> {
  /**
   The process's statistics.
   */
  let shown: string;
  try {
    shown = await readFile(
      `${PROCESS_FOLDER}/${String(pid,)}/stat`,
      'utf8',
    );
  }
  catch (error) {
    // A process that is gone, or one this account may not read, answers nothing: the caller treats an unknown start time as a
    // holder it cannot judge, which is a lock it must not take.
    if ((errorCodeOf(error,) === MISSING_PATH_CODE) || (errorCodeOf(error,) === 'EACCES')) {
      return undefined;
    }
    throw error;
  }
  /**
   Where the process's name ends, which is the last closing bracket: the name itself may hold brackets and spaces.
   */
  const nameEnd = shown.lastIndexOf(PROCESS_STAT_NAME_END,);
  if (nameEnd === (-1)) {
    return undefined;
  }
  /**
   The fields after the name, of which the first is the state and the twenty-second from one is the start time.
   */
  const afterName = shown.slice(nameEnd + 1,)
    .trim()
    .split(/\s+/u,);
  /**
   The start time, which is at the field the kernel counts it in.
   */
  const startedAt = afterName[PROCESS_START_TIME_FIELD - 3];
  return ((startedAt === undefined) || (startedAt === '')) ? undefined : startedAt;
}

/**
 Reads whether a lock holds no live critical section, and so may be broken.

 @param holder - what the lock file says about its holder

 @returns whether the lock may be broken, and the reason a caller's line gives

 @example
 ```ts
 const breaker = lockMayBeBroken({ holder });
 ```
 */
async function lockMayBeBroken({ holder, }: { readonly holder: HolderOfAQueue },): Promise<StateOfALock> {
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
  if (holder.startedAt === undefined) {
    // A lock written by hand, or by a version of this module that recorded no start time, names a process this caller cannot
    // tell from a process number reused after its holder died: it is a holder, and a live holder is never taken from.
    return { broken: false, };
  }
  /**
   When the process now holding that number started.
   */
  const startedAt = await processStartedAtOf({ pid: holder.pid, },);
  if ((startedAt !== undefined) && (startedAt !== holder.startedAt)) {
    return {
      broken: true,
      reason: `the number of its process ${String(holder.pid,)} now belongs to a process started at ${startedAt}, not at the ${holder.startedAt} it names`,
    };
  }
  return { broken: false, };
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
 Removes a lock file while it still carries this caller's token, and leaves another caller's lock alone.

 @param lockFile - the lock's path

 @param token - the token this caller's lock carries

 @example
 ```ts
 await removeTheLockWhileItIsOurs({ lockFile: '/runs/queue.json.lock', token });
 ```
 */
async function removeTheLockWhileItIsOurs({
  lockFile,
  token,
}: {
  readonly lockFile: string;
  readonly token: string;
},): Promise<void> {
  /**
   What the lock says now, which is another caller's lock once this caller's was broken and taken.
   */
  const holder = await holderOfAQueue({ lockFile, },);
  if (holder.token !== token) {
    return;
  }
  await rm(
    lockFile,
    { force: true, },
  );
}

/**
 Refuses a change when the lock under which the queue was read is no longer this caller's.

 A lock is only ever broken while its holder is gone or while the number it names belongs to another process, so this answers
 about a caller whose lock was taken by someone who judged it wrongly, or about a change made after its own lock was given back.
 Either way the list this caller read is not the list it would write, so nothing is written.

 @param lockFile - the lock's path

 @param token - the token this caller's lock carries

 @param queueFile - the queue's path, for the message

 @throws QueueLockLostError when the lock is there and carries another token, or is gone

 @example
 ```ts
 await assertTheLockIsStillOurs({ lockFile: '/runs/queue.json.lock', token, queueFile: '/runs/queue.json' });
 ```
 */
export async function assertTheLockIsStillOurs({
  lockFile,
  token,
  queueFile,
}: {
  readonly lockFile: string;
  readonly token: string;
  readonly queueFile: string;
},): Promise<void> {
  /**
   What the lock says now.
   */
  const holder = await holderOfAQueue({ lockFile, },);
  if (holder.token === token) {
    return;
  }
  throw new QueueLockLostError(
    `the queue at ${queueFile} was not changed: the lock this caller took is gone, and ${lockFile} now holds `
      + `${((holder.pid === undefined) ? 'nothing this caller can read' : `process ${String(holder.pid,)} since ${holder.at ?? 'an unknown time'}`)}. `
      + 'The queue was read under the old lock, so writing it would write over whatever the new holder changed.',
  );
}

/**
 Waits, bounded, for one caller to take a queue's lock, and answers the lock it holds.

 @param queueFile - the queue whose lock is taken

 @param now - the clock the wait and the lock's time are read from

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
   The token this caller's lock carries, which tells it from a lock taken after its own was broken.
   */
  const token = randomUUID();
  /**
   When this caller's process started, which tells a later holder from this process number reused after this one dies.
   */
  const startedAt = await processStartedAtOf({ pid: process.pid, },);
  /**
   The moment this caller stops asking.
   */
  const deadline = now() + waitMilliseconds;
  /* oxlint-disable eslint/no-unused-expressions -- one attempt, then a pause, then the next attempt: taking a lock is a sequence
     of attempts by definition, and each reads what the attempt before it left. See doc/lint/sequential-awaits.md. */
  /* oxlint-disable eslint/no-await-in-loop -- one attempt, then a pause, then the next attempt: taking a lock is a sequence of
     attempts by definition, and each reads what the attempt before it left. See doc/lint/sequential-awaits.md. */
  for (;;) {
    if (await takeTheLockFile({
      lockFile,
      content: `${JSON.stringify({
        pid: process.pid,
        ...((startedAt === undefined) ? {} : { startedAt, }),
        at: new Date(now(),).toISOString(),
        token,
      },)}\n`,
    },)) {
      return {
        lockFile,
        token,
        async [Symbol.asyncDispose]() {
          // Only this caller's own lock goes: a lock that was broken and taken by another caller carries that caller's token, and
          // removing it would take a live holder's queue from it.
          await removeTheLockWhileItIsOurs({
            lockFile,
            token,
          },);
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
    const breaker = await lockMayBeBroken({ holder, },);
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
