/**
 One queue command, run as a program, killed at the instruction a control names: the stand-in counts the file-system steps that change the
 queue and kills itself at the numbered one, so a control can walk every instruction a command has and ask what the next command finds.

 The counting is a patch of `node:fs/promises` before the command's own modules are imported, which is the one place this repository's rules
 allow a module-level patch: a control cannot otherwise stop a program between two of its file-system calls. Each patch names its own
 parameters, and the two casts that install them are suppressed with the reason here.

 @internal

 @module
 */

import fs from 'node:fs';
import type { PathLike, } from 'node:fs';
import { mkdir, } from 'node:fs/promises';
import { syncBuiltinESMExports, } from 'node:module';
import { argv, } from 'node:process';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { whatTheCommandAnswered, } from './stand-in-a-queue-commands-for-tests.ts';

/**
 The words the program was given: the command, the instruction to stop at, the queue's folder and its runs folder.
 */
const [command, killAtText, queueFolderText, runsFolderText,] = argv.slice(2,);

/**
 The queue's folder, which the control always names.
 */
const queueFolder = queueFolderText ?? '';

/**
 The runs folder the command writes its record into, which the control names so that what it writes and what the control reads are one
 folder.
 */
const runsFolder = runsFolderText ?? `${queueFolder}-runs`;

/**
 The instruction the program is killed at, which is past every instruction when it is larger than the command has.
 */
const killAt = Number(killAtText,);

/**
 What the program counts, which is one number of the steps taken and no other state.
 */
const counting = { steps: 0, };

/**
 The functions this program counts, kept before they are replaced.
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
};

/**
 Counts one changing step and kills the program when it is the one asked for.
 */
function aStep(): void {
  counting.steps += 1;
  if (counting.steps === killAt) {
    // The program kills itself: the only signal this probe sends, to the only process it signals.
    process.kill(
      process.pid,
      'SIGKILL',
    );
  }
}

/**
 Writes a file, counting the step.
 
 @param path - where the file goes
 
 @param data - what it holds
 
 @returns nothing
 */
async function countedWriteFile({
  path,
  data,
}: {
  readonly path: Parameters<typeof fs.promises.writeFile>[0];
  readonly data: Parameters<typeof fs.promises.writeFile>[1];
},): Promise<void> {
  aStep();
  await original.writeFile(
    path,
    data,
  );
}

/**
 Links a file into a name, counting the step.
 
 @param existing - the file made already
 
 @param created - the name it is linked to
 
 @returns nothing
 */
async function countedLink({
  existing,
  created,
}: {
  readonly existing: PathLike;
  readonly created: PathLike;
},): Promise<void> {
  aStep();
  await original.link(
    existing,
    created,
  );
}

/**
 Renames a file, counting the step.
 
 @param from - the name it has
 
 @param to - the name it takes
 
 @returns nothing
 */
async function countedRename({
  from,
  to,
}: {
  readonly from: PathLike;
  readonly to: PathLike;
},): Promise<void> {
  aStep();
  await original.rename(
    from,
    to,
  );
}

/**
 Removes a file, counting the step.
 
 @param path - the file's path
 
 @returns nothing
 */
async function countedRemove({ path, }: { readonly path: PathLike },): Promise<void> {
  aStep();
  await original.rm(
    path,
    { force: true, },
  );
}

// the same arguments and returns the same thing; TypeScript cannot prove the overloads match, which is what the patch is for.
fs.promises
  .writeFile = countedWriteFile as unknown as typeof fs.promises.writeFile;
fs.promises
  .link = countedLink as unknown as typeof fs.promises.link;
fs.promises
  .rename = countedRename as unknown as typeof fs.promises.rename;
fs.promises
  .rm = countedRemove as unknown as typeof fs.promises.rm;
syncBuiltinESMExports();

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
      steps: counting.steps,
      answer: (answer === undefined) ? null : answer,
    },)}\n`,);
}
catch (error) {
  process.stderr
    .write(`${caughtValueText(error,)}\n`,);
  process.exitCode = 1;
}
