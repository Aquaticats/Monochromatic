/**
 One queue command, run as a program, killed at the instruction a control names: the stand-in counts the file-system steps that change the
 queue and kills itself at the numbered one, so a control can walk every instruction a command takes and ask what the next command finds.

 @internal

 @module
 */

import { mkdir, } from 'node:fs/promises';
import { argv, } from 'node:process';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  countTheSteps,
  stepsTaken,
} from './counting-the-file-steps-of-a-queue-command-for-tests.ts';
import { whatTheCommandAnswered, } from './stand-in-a-queue-commands-for-tests.ts';

/**
 The words the program was given: the command, the instruction to stop at, the queue's folder and its runs folder.
 */
const [commandText, killAtText, queueFolderText, runsFolderText,] = argv.slice(2,);

/**
 The command to run, which the control always names.
 */
const command: string = commandText ?? '';

/**
 The queue's folder, which the control always names.
 */
const queueFolder: string = queueFolderText ?? '';

/**
 The runs folder the command writes its record into, which the control names so that what it writes and what the control reads are one
 folder.
 */
const runsFolder: string = runsFolderText ?? `${queueFolder}-runs`;

/**
 The instruction the program is killed at, which is past every instruction when it is larger than the command has.
 */
const killAt = Number(killAtText,);

// The counting is installed before the command's own modules are imported, so every step it takes is counted.
countTheSteps({ killAt, },);

try {
  // The runs folder this stand-in's commands write their records into, which the control's queue does not make.
  await mkdir(
    runsFolder,
    { recursive: true, },
  );
  /**
   What the command answered.
   */
  const answer = await whatTheCommandAnswered({
    command,
    queueFolder,
    runsFolder,
  },);
  process.stdout
    .write(`${JSON.stringify({
      command,
      steps: stepsTaken(),
      answer: (answer === undefined) ? null : answer,
    },)}\n`,);
}
catch (error) {
  process.stderr
    .write(`${caughtValueText(error,)}\n`,);
  process.exitCode = 1;
}
