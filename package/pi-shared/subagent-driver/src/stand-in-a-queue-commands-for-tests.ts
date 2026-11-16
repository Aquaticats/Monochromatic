/**
 The commands the stand-in of a queue folder can run, one function each, so the program that counts and kills its instructions stays small.

 @internal

 @module
 */

import {
  claimAnEntryOfAQueue,
  completeTheClaim,
  giveBackTheClaim,
} from './claiming-an-entry-of-a-queue.ts';
import { convertAQueueFileIntoAFolder, } from './converting-a-queue-file-into-a-folder.ts';
import {
  moveAQueueEntryTo,
  putAQueueEntryIn,
  takeAQueueEntryOut,
} from './putting-and-taking-entries-of-a-queue.ts';
import { putASetAsideEntryBack, } from './putting-a-set-aside-entry-back.ts';
import {
  isQueueRecord,
  type QueueRecord,
} from './queue-of-runs.ts';
import {
  readQueueOfRuns,
  readTheEntryNamed,
} from './reading-and-writing-the-folder-a-queue-is.ts';
import { setAsideAnEntryOfAQueue, } from './set-aside-entry.ts';
import { settleTheQueueAfterAKill, } from './settling-the-queue-after-a-kill.ts';

/**
 Says nothing, which is the report the settlement needs and a control does not read.

 @returns nothing
 */
function reportNothing(): undefined {
  return undefined;
}

/**
 The entry every command of this stand-in works on, which the control wrote into the queue.
 */
const entry: QueueRecord = {
  name: 'a',
  branch: 'feat/a',
  prompt: '/runs/a-prompt.md',
  wall: 28_800,
};

/**
 Runs the one command the program was named.

 @param command - the command to run, by the name the control gave it

 @param queueFolder - the queue's folder

 @param runsFolder - the runs folder the command writes its record into

 @returns what the command answered

 @example
 ```ts
 const answer = await whatTheCommandAnswered({ command: 'put', queueFolder: '/runs/q', runsFolder: '/runs' });
 ```
 */
export async function whatTheCommandAnswered({
  command,
  queueFolder,
  runsFolder,
}: {
  readonly command: string;
  readonly queueFolder: string;
  readonly runsFolder: string;
},): Promise<unknown> {
  if (command === 'put') {
    return await putAQueueEntryIn({
      queueFolder,
      entry,
      position: 0,
    },);
  }
  if (command === 'take') {
    return await takeAQueueEntryOut({
      queueFolder,
      name: entry.name,
    },);
  }
  if (command === 'move') {
    return await moveAQueueEntryTo({
      queueFolder,
      name: entry.name,
      position: 0,
    },);
  }
  if (command === 'claim') {
    /**
     The entry the queue holds, which is the one the claim is taken on.
     */
    const held = await readTheEntryNamed({
      queueFolder,
      name: entry.name,
    },);
    if (!isQueueRecord(held,)) {
      throw new Error(`the queue holds no entry named ${entry.name}`,);
    }
    return await claimAnEntryOfAQueue({
      queueFolder,
      record: held,
      now: Date.now,
    },);
  }
  if (command === 'complete') {
    /**
     The entry the queue holds, whose identity the claim was taken on.
     */
    const [held,] = await readQueueOfRuns({ queueFolder, },);
    if (held === undefined) {
      throw new Error(`the queue holds no entry named ${entry.name}`,);
    }
    return await completeTheClaim({
      queueFolder,
      name: entry.name,
      queueId: ((typeof held.queueId) === 'string') ? held.queueId : '',
    },);
  }
  if (command === 'giveBack') {
    return await giveBackTheClaim({
      queueFolder,
      name: entry.name,
    },);
  }
  if (command === 'setAside') {
    /**
     The entry the queue holds, which the record and the removal are written for.
     */
    const [held,] = await readQueueOfRuns({ queueFolder, },);
    if (held === undefined) {
      throw new Error(`the queue holds no entry named ${entry.name}`,);
    }
    return await setAsideAnEntryOfAQueue({
      queueFolder,
      runsFolder,
      record: held,
      queueId: ((typeof held.queueId) === 'string') ? held.queueId : '',
      reason: 'the entry failed its start',
      now: Date.now,
    },);
  }
  if (command === 'putBack') {
    return await putASetAsideEntryBack({
      queueFolder,
      runsFolder,
      name: entry.name,
      position: 0,
    },);
  }
  if (command === 'settle') {
    return await settleTheQueueAfterAKill({
      queueFolder,
      runsFolder,
      report: reportNothing,
    },);
  }
  if (command === 'convert') {
    return await convertAQueueFileIntoAFolder({
      queueFile: `${queueFolder}.json`,
      queueFolder,
    },);
  }
  throw new Error(`no command of the stand-in is named ${command}`,);
}
