/**
 One queue command, run as a program, killed at the instruction a control names: the stand-in counts the file-system steps that change the
 queue and sends itself `SIGKILL` at the numbered one, so a control can walk every instruction a command has and ask what the next command
 finds.

 It prints what it did as one JSON line, and answers a refusal with a failing status and the reason, which is what the commands do.

 @internal

 @module
 */

import fs from 'node:fs';
import { mkdir, } from 'node:fs/promises';
import { syncBuiltinESMExports, } from 'node:module';
import { argv, } from 'node:process';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
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
import type { QueueRecord, } from './queue-of-runs.ts';
import {
  readQueueOfRuns,
  readTheEntryNamed,
} from './reading-and-writing-the-folder-a-queue-is.ts';
import { setAsideAnEntryOfAQueue, } from './set-aside-entry.ts';
import { settleTheQueueAfterAKill, } from './settling-the-queue-after-a-kill.ts';

/**
 The words the program was given: the command, the instruction to stop at, the queue's folder, and what the command needs.
 */
const [command, killAtText, queueFolderText,] = argv.slice(2,);

/**
 The queue's folder, which the control always names.
 */
const queueFolder = queueFolderText ?? '';

/**
 The instruction the program is killed at, which is past every instruction when it is larger than the command has.
 */
const killAt = Number(killAtText,);

/**
 How many changing steps the program has taken.
 */
let steps = 0;

/**
 The stand-in's record of each changing step, so a control can see how many a command has.
 */
const original = {
  writeFile: fs.promises
    .writeFile,
  link: fs.promises
    .link,
  rename: fs.promises
    .rename,
  rm: fs.promises
    .rm,
  mkdir: fs.promises
    .mkdir,
};

/**
 Counts one changing step and kills the program when it is the one asked for.

 @param run - the step itself

 @returns what the step answered

 @example
 ```ts
 const written = await aStep(() => original.writeFile(file, 'x'));
 ```
 */
async function aStep<Answer>(run: () => Promise<Answer>,): Promise<Answer> {
  steps += 1;
  if (steps === killAt) {
    // The program kills itself, which is the only signal this probe sends and the only process it signals.
    process.kill(
      process.pid,
      'SIGKILL',
    );
  }
  return await run();
}

fs.promises
  .writeFile = async function countedWriteFile(...words) {
    await aStep(() => original.writeFile(...words));
  };
fs.promises
  .link = async function countedLink(...words) {
    await aStep(() => original.link(...words));
  };
fs.promises
  .rename = async function countedRename(...words) {
    await aStep(() => original.rename(...words));
  };
fs.promises
  .rm = async function countedRemove(...words) {
    await aStep(() => original.rm(...words));
  };
fs.promises
  .mkdir = (async function countedMkdir(...words: Parameters<typeof original.mkdir>) {
    return await aStep(() => original.mkdir(...words));
  }) as typeof original.mkdir;
syncBuiltinESMExports();

/**
 The runs folder a command writes a record into, which is the queue's own folder's sibling.
 */
const runsFolder = `${queueFolder}-runs`;

/**
 The entry every command of this stand-in works on, which the control wrote into the queue.
 */
const entry: QueueRecord = {
  name: 'a',
  branch: 'feat/a',
  prompt: '/runs/a-prompt.md',
  wall: 28_800,
};

try {
  // The runs folder this stand-in's commands write their records into, which the control's queue does not make.
  await mkdir(
    runsFolder,
    { recursive: true, },
  );
  /**
   What the command answered.
   */
  let answer: unknown;
  if (command === 'put') {
    answer = await putAQueueEntryIn({
      queueFolder,
      entry,
      position: 0,
    },);
  }
  else if (command === 'take') {
    answer = await takeAQueueEntryOut({
      queueFolder,
      name: entry.name,
    },);
  }
  else if (command === 'move') {
    answer = await moveAQueueEntryTo({
      queueFolder,
      name: entry.name,
      position: 0,
    },);
  }
  else if (command === 'claim') {
    /**
     The entry the queue holds, which is the one the claim is taken on.
     */
    const held = await readTheEntryNamed({
      queueFolder,
      name: entry.name,
    },);
    answer = await claimAnEntryOfAQueue({
      queueFolder,
      record: (held as QueueRecord),
      now: Date.now,
    },);
  }
  else if (command === 'complete') {
    /**
     The entry the queue holds, whose identity the claim was taken on.
     */
    const held = (await readQueueOfRuns({ queueFolder, },))[0] as QueueRecord;
    answer = await completeTheClaim({
      queueFolder,
      name: entry.name,
      queueId: String(held.queueId,),
    },);
  }
  else if (command === 'giveBack') {
    answer = await giveBackTheClaim({
      queueFolder,
      name: entry.name,
    },);
  }
  else if (command === 'setAside') {
    /**
     The entry the queue holds, which the record and the removal are written for.
     */
    const held = (await readQueueOfRuns({ queueFolder, },))[0] as QueueRecord;
    answer = await setAsideAnEntryOfAQueue({
      queueFolder,
      runsFolder,
      record: held,
      queueId: String(held.queueId,),
      reason: 'the entry failed its start',
      now: Date.now,
    },);
  }
  else if (command === 'putBack') {
    answer = await putASetAsideEntryBack({
      queueFolder,
      runsFolder,
      name: entry.name,
      position: 0,
    },);
  }
  else if (command === 'settle') {
    answer = await settleTheQueueAfterAKill({
      queueFolder,
      runsFolder,
      report: function report() {
        return undefined;
      },
    },);
  }
  else if (command === 'convert') {
    answer = await convertAQueueFileIntoAFolder({
      queueFile: `${queueFolder}.json`,
      queueFolder,
    },);
  }
  else {
    throw new Error(`no command of the stand-in is named ${String(command,)}`,);
  }
  process.stdout
    .write(`${JSON.stringify({
      command,
      steps,
      answer: (answer === undefined) ? null : answer,
    },)}\n`,);
}
catch (error) {
  process.stderr
    .write(`${caughtValueText(error,)}\n`,);
  process.exitCode = 1;
}
