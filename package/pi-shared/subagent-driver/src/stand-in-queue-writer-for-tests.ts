/**
 One queue command's own change, run as a program: it puts or takes an entry the way `put-an-entry-into-a-queue` and
 `take-an-entry-out-of-a-queue` do, so a race control can run two of them as two real processes against one queue file.

 It prints what it did, and answers a refusal with a failing status and the reason, which is what the commands do.

 @internal

 @module
 */

import { argv, } from 'node:process';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  putAQueueEntryIn,
  takeAQueueEntryOut,
} from './queue-of-runs.ts';

/**
 The words the program was given.
 */
const words = argv.slice(2,);

/**
 What to do, the queue, the entry's name and, for a put, the position.
 */
const [what, queueFile, name, positionText,] = words;

/**
 A command line the stand-in cannot use.
 */
class StandInQueueWriterError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'StandInQueueWriterError';
}

if (what === undefined) {
  throw new StandInQueueWriterError('usage: <put|take> <queue-file> <name> [position]',);
}
if (queueFile === undefined) {
  throw new StandInQueueWriterError('usage: <put|take> <queue-file> <name> [position]',);
}
if (name === undefined) {
  throw new StandInQueueWriterError('usage: <put|take> <queue-file> <name> [position]',);
}

try {
  if (what === 'put') {
    /**
     What queueing the entry answered.
     */
    const queued = await putAQueueEntryIn({
      queueFile,
      entry: { name, },
      position: Number(positionText ?? '0',),
    },);
    /**
     Where the entry landed, and how many entries the queue holds.
     */
    const {
      position,
      count,
    } = queued;
    process.stdout
      .write(`queued ${name} at position ${String(position,)} of ${String(count,)}\n`,);
  }
  else if (what === 'take') {
    /**
     What taking the entry out answered.
     */
    const taken = await takeAQueueEntryOut({
      queueFile,
      name,
    },);
    /**
     How many entries were taken out, and how many the queue holds now.
     */
    const {
      taken: took,
      entries,
    } = taken;
    process.stdout
      .write(`took out ${String(took.length,)}; ${String(entries.length,)} left\n`,);
  }
  else {
    throw new StandInQueueWriterError(`unknown change ${what}: usage: <put|take> <queue-file> <name> [position]`,);
  }
}
catch (error) {
  // A refusal is answered the way the commands answer one: the reason on standard error and a failing status.
  process.stderr
    .write(`${caughtValueText(error,)}\n`,);
  process.exitCode = 1;
}
