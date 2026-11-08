/**
 The part of a run after the prompt is sent: watches the limits, delivers the inbox, and decides when the run is over.

 @module
 */

import {
  readdir,
  readFile,
  rename,
  unlink,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { setInterval as everyInterval, } from 'node:timers/promises';
import type { ConnectionToPi, } from './connection-to-pi.ts';
import {
  FEED_PREVIEW_CHARACTERS,
  MILLISECONDS_PER_SECOND,
  POLL_MILLISECONDS,
  STOP_FILE_NAME,
} from './driver-constants.ts';
import type {
  DriverOptions,
  EndReason,
  RunProgress,
} from './driver-types.ts';
import {
  preview,
  type RecordOfPiRun,
} from './record-of-pi-run.ts';

/**
 The polling interval stopped, which it never does by itself.
 */
export class WatchEndedWithoutReasonError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'WatchEndedWithoutReasonError';
}

/**
 What watching a run needs.
 */
export type WatchRequest = {
  readonly connection: ConnectionToPi;
  readonly record: RecordOfPiRun;
  readonly options: DriverOptions;
  readonly inbox: string;
  readonly deadline: number;
};

/**
 Delivers one inbox message. The file goes only once Pi has it; a refused or unanswered one stays readable under a dot name,
 which the watch skips, so it is neither lost nor sent again.

 @param watch - watch request

 @param name - message's file name
 */
async function deliverMessage({ watch, name, }: { readonly watch: WatchRequest; readonly name: string; },): Promise<void> {
  /**
   The message file.
   */
  const path = join(watch.inbox, name,);
  /**
   The message.
   */
  const text = await readFile(path, 'utf8',);
  /**
   How the message reaches Pi: a steer while it works, a new prompt while it rests.
   */
  const sentAs = watch.record.isRunning() ? 'steer' : 'prompt';
  /**
   Pi's answer.
   */
  const answer = await watch.connection.send({ command: { type: sentAs, message: text, }, },);
  await (answer.success === true ? unlink(path,) : rename(path, join(watch.inbox, `.failed-${name}`,),));
  watch.record.note({
    kind: 'inbox',
    file: name,
    sentAs,
    success: answer.success,
    error: answer.error ?? null,
    preview: preview({ value: text, length: FEED_PREVIEW_CHARACTERS.inboxMessage, },),
  },);
}

/**
 Delivers one inbox entry, or sets it aside when it cannot be read, such as a folder dropped there by mistake.

 @param watch - watch request

 @param name - entry's name
 */
async function deliverOrSetAside({ watch, name, }: { readonly watch: WatchRequest; readonly name: string; },): Promise<void> {
  try {
    await deliverMessage({ watch, name, },);
  }
  catch (error) {
    watch.record.note({ kind: 'problem', problem: `could not deliver ${name}: ${String(error,)}`, },);
    await rename(join(watch.inbox, name,), join(watch.inbox, `.failed-${name}`,),);
  }
}

/**
 Looks once at every limit that could end a run.

 @param watch - watch request

 @param wallDeadline - when the run must be over

 @returns why the run is over, or that it goes on
 */
async function progressAgainstLimits(
  { watch, wallDeadline, }: { readonly watch: WatchRequest; readonly wallDeadline: number; },
): Promise<RunProgress> {
  if (watch.connection.hasExited()) {
    return 'pi-exited';
  }
  if (Date.now() >= wallDeadline) {
    return 'wall-clock';
  }
  if ((await readdir(watch.options.runFolder,)).includes(STOP_FILE_NAME,)) {
    return 'stop-file';
  }
  return watch.record.turnCount() >= watch.options.turnCap ? 'turn-cap' : 'still-running';
}

/**
 Lists the messages ready to read. A name starting with a dot is still being written, or was refused.

 @param inbox - inbox folder

 @returns message file names, in order
 */
async function readyMessages(inbox: string,): Promise<readonly string[]> {
  return (await readdir(inbox,)).filter(function isReady(name,) {
    return !name.startsWith('.',);
  },).toSorted();
}

/**
 Tells whether Pi has rested for long enough that the run is over.

 @param watch - watch request

 @returns whether the idle wait has passed
 */
function idleLongEnough(watch: WatchRequest,): boolean {
  return !watch.record.isRunning()
    && (watch.record.settledCount() >= 1)
    && (watch.record.silentMilliseconds() > (watch.options.idleSeconds * MILLISECONDS_PER_SECOND));
}

/**
 Checks the limits and delivers the next inbox message, on every tick, until something ends the run. One message per tick keeps
 them in order, each after Pi has answered the one before.

 @param watch - watch request

 @returns why the run ended

 @throws WatchEndedWithoutReasonError when the interval stops, which it never does by itself

 @example
 ```ts
 const endReason = await watchUntilEnded(watch,);
 ```
 */
export async function watchUntilEnded(watch: WatchRequest,): Promise<EndReason> {
  for await (const wallDeadline of everyInterval(POLL_MILLISECONDS, watch.deadline,)) {
    /**
     Where the run stands against its limits.
     */
    const progress = await progressAgainstLimits({ watch, wallDeadline, },);
    if (progress !== 'still-running') {
      return progress;
    }
    /**
     The next message to deliver, if any.
     */
    const [nextMessage,] = await readyMessages(watch.inbox,);
    if (nextMessage !== undefined) {
      await deliverOrSetAside({ watch, name: nextMessage, },);
    }
    else if (idleLongEnough(watch,)) {
      return 'idle';
    }
  }
  throw new WatchEndedWithoutReasonError('The polling interval stopped before the run ended.',);
}
