/**
 Emptying a run's scratch folder in a process of its own, at the lowest disk priority the machine gives a process, so a deletion of
 tens of GiB neither holds the disk against another run's build nor asks the kernel for one descriptor per file.

 The process is the command `empty-the-scratch-folder-of-a-run`, which is also what a person runs to empty one folder by hand. Its
 lines are the ones the run's end carries, so the figure a reader sees is read once, after the deletion, rather than guessed at
 before it.

 @module
 */

import { siblingCommandEntry, } from './sibling-command-entry.ts';
import {
  type CommandOutput,
  type RunCommand,
  runCommandWithChildProcess,
} from './run-command.ts';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import { entriesLeftIn, } from './deleting-what-a-run-left-in-its-scratch-folder.ts';
import { scratchFolderOf, } from './scratch-folder-of-a-run.ts';

/**
 Seconds the emptying of one run's scratch folder may take before this step gives up on it and says so.

 Measured on 2026-11-13 in this machine's own scratch folder, through the command this step runs: 2,000 files of 1 MiB and
 20,000 small files, 2.0 GiB and 22,000 entries, were deleted in 1.38 s at full speed, which is about 1.5 GiB a second. The
 largest scratch folder measured on this machine held 63.5 GiB of its own data (`btrfs filesystem du`, of the two days of
 scratch that `du` called 190.5 GB), which is about 42 s at that rate. This bound is forty times that, because the deletion
 runs at the idle disk class and gives way to every other process: a folder holding more than this can delete is named by its
 own line rather than left silent.
 */
export const SCRATCH_EMPTYING_BOUND_SECONDS = 1_800;

/**
 The code a failure carries when the program a command names is not on this machine at all, which is the one failure the fallback
 below exists for.
 */
const NO_SUCH_PROGRAM_CODE = 'ENOENT';

/**
 The program that puts this step's own process at the lowest disk priority, which is util-linux's `ionice`.
 */
const LOWEST_DISK_PRIORITY_PROGRAM = 'ionice';

/**
 The argument that asks `ionice` for the idle class, which is the lowest disk priority the kernel has.
 */
const IDLE_DISK_CLASS = 'idle';

/**
 The program that lowers this step's own processor priority, which is coreutils' `nice`.
 */
const LOWEST_PROCESSOR_PRIORITY_PROGRAM = 'nice';

/**
 The adjustment `nice` takes for the lowest processor priority an unprivileged process may ask for.
 */
const LOWEST_PROCESSOR_PRIORITY = '19';

/**
 The option that names the root a run's scratch folder lives under.
 */
const SCRATCH_ROOT_OPTION = '--scratch-root=';

/**
 The option that names the run whose folder is emptied.
 */
const RUN_NAME_OPTION = '--run-name=';

/**
 Reads the lines a command printed, which is how the emptying's own words reach the run's end.

 @param shown - what the command printed

 @returns the lines it printed, without the empty ones

 @example
 ```ts
 const lines = linesIn({ shown: 'scratch: a: emptied /tmp/a-scratch except keep/\n', },);
 ```
 */
function linesIn({ shown, }: { readonly shown: string },): readonly string[] {
  return shown
    .split('\n',)
    .filter(function isALine(line,) {
      return line.trim() !== '';
    },);
}

/**
 Says why the emptying printed nothing, in the words a reader of the run's end can act on.

 @param output - what the command answered

 @returns the reason

 @example
 ```ts
 const reason = reasonOfSilence({ output, },);
 ```
 */
function reasonOfSilence({ output, }: { readonly output: CommandOutput },): string {
  // A bound kill is named as what it is rather than as the message the program that ran it gives, which is the whole command line.
  if (output.killedByItsOwnBound === true) {
    return `it ran past the ${String(SCRATCH_EMPTYING_BOUND_SECONDS,)} second bound this step gives one deletion`;
  }
  if (output.problem !== undefined) {
    return output.problem;
  }
  /**
   What the command complained about, which names what it could not do.
   */
  const complained = output.stderr
    .trim();
  if (complained !== '') {
    return complained;
  }
  return `it ended with status ${String(output.status,)} and printed nothing`;
}

/**
 Empties one run's scratch folder at the lowest disk priority, and answers with the lines the run's end carries.

 @param run - how one command is run, which a test replaces

 @param scratchRoot - the root every run's scratch folder lives under

 @param runName - the run whose folder is emptied

 @returns the lines to report, which say what was removed, what `keep/` holds, and what was left where it is

 @example
 ```ts
 const lines = await emptyTheScratchFolderAtTheLowestDiskPriority({ run: runCommandWithChildProcess, scratchRoot: '/home/a/temp/agent', runName: 'copy-route', },);
 ```
 */
export async function emptyTheScratchFolderAtTheLowestDiskPriority({
  run,
  scratchRoot,
  runName,
}: {
  readonly run: RunCommand;
  readonly scratchRoot: string;
  readonly runName: string;
},): Promise<readonly string[]> {
  /**
   The entry of the command that does the emptying.
   */
  const entry = await siblingCommandEntry({
    folder: import.meta.dirname,
    command: 'empty-the-scratch-folder-of-a-run',
  },);
  /**
   The command's own words, which name the root and the run and never the folder: the folder is what that command computes from
   them, so a path that is not a run's scratch folder cannot be asked for here.
   */
  const commandWords: readonly string[] = [
    process.execPath,
    entry,
    `${SCRATCH_ROOT_OPTION}${scratchRoot}`,
    `${RUN_NAME_OPTION}${runName}`,
  ];
  /**
   What the emptying printed.
   */
  const output = await run({
    command: LOWEST_DISK_PRIORITY_PROGRAM,
    words: [
      '--class',
      IDLE_DISK_CLASS,
      '--',
      LOWEST_PROCESSOR_PRIORITY_PROGRAM,
      '--adjustment',
      LOWEST_PROCESSOR_PRIORITY,
      ...commandWords,
    ],
    timeoutMilliseconds: SCRATCH_EMPTYING_BOUND_SECONDS * MILLISECONDS_PER_SECOND,
  },);
  /**
   What the command printed, which is empty when it never started.
   */
  const printed = linesIn({ shown: output.stdout, },);
  if (printed.length > 0) {
    return printed;
  }
  // A machine without `ionice` is not a machine where a folder goes unemptied: the emptying is made without the disk class, and
  // the line it answers with is the same line, because the deletion is what the run's end reports.
  //
  // Only a machine without the program is answered that way. A deletion this step's own bound killed is no missing program: it is
  // named with what it left, and the rest is left for the next end of a run of this name or for the command a person runs. Trying
  // it again at full disk priority would take twice the bound this step documents, against every other process on the disk.
  if ((output.status === (-1)) && (output.problemCode === NO_SUCH_PROGRAM_CODE)) {
    /**
     What the emptying printed when it ran without the disk class.
     */
    const plain = await run({
      command: process.execPath,
      words: commandWords.slice(1,),
      timeoutMilliseconds: SCRATCH_EMPTYING_BOUND_SECONDS * MILLISECONDS_PER_SECOND,
    },);
    /**
     The lines that run printed.
     */
    const plainPrinted = linesIn({ shown: plain.stdout, },);
    if (plainPrinted.length > 0) {
      return plainPrinted;
    }
    return [`scratch: ${runName}: the folder could not be emptied: ${reasonOfSilence({ output: plain, },)}`,];
  }
  /**
   The folder the command was asked to empty, which a bound kill leaves part of.
   */
  const folder = scratchFolderOf({
    scratchRoot,
    runName,
  },);
  if (output.killedByItsOwnBound === true) {
    /**
     What the folder still holds, which is how far the deletion got.
     */
    const left = await entriesLeftIn(folder,);
    return [
      `scratch: ${runName}: ${reasonOfSilence({ output, },)}; ${String(left.files,)} `
        + `file${(left.files === 1) ? '' : 's'} and ${String(left.folders,)} `
        + `folder${(left.folders === 1) ? '' : 's'} are still in ${folder}, and the next end of a run of this name, or `
        + `empty-the-scratch-folder-of-a-run, takes them`,
    ];
  }
  return [`scratch: ${runName}: the folder at ${folder} could not be emptied: ${reasonOfSilence({ output, },)}`,];
}
