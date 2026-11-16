/**
 One claimer, run as a program beside another claimer, so a race test has two real processes and not two promises: it claims the entry named
 `a` at the moment the test gives it and prints what it was answered.

 A test gives both processes the same moment, so neither can be later than the other by reading the clock, and the claim's own time is the
 moment the test named.

 @internal

 @module
 */

import { argv, } from 'node:process';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { claimAnEntryOfAQueue, } from './claiming-an-entry-of-a-queue.ts';
import { readTheEntryNamed, } from './reading-and-writing-the-folder-a-queue-is.ts';

/**
 The queue's folder and the moment to claim at, which the test gives.
 */
const [queueFolder, at,] = argv.slice(2,);

try {
  /**
   The entry the queue holds under the name `a`.
   */
  const held = await readTheEntryNamed({
    queueFolder: queueFolder ?? '',
    name: 'a',
  },);
  if (((typeof held) !== 'object') || (held === null)) {
    process.stdout
      .write('no-entry\n',);
  }
  else {
    /**
     What taking a claim on the entry answered.
     */
    const claimed = await claimAnEntryOfAQueue({
      queueFolder: queueFolder ?? '',
      record: held,
      now: function theMomentTheTestGave() {
        return Date.parse(at ?? '',);
      },
    },);
    process.stdout
      .write(`${claimed.kind}\n`,);
  }
}
catch (error) {
  process.stdout
    .write(`threw ${caughtValueText(error,)}\n`,);
  process.exitCode = 1;
}
