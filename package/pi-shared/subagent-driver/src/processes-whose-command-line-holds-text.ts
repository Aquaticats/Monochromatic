/**
 The processes whose command line holds a text, read from the process table the rest of this package already reads: a text that
 names a script, a run folder or a unit finds every process working on it, with how old each is and what it runs.

 The table is the seam: a test hands over what `ps` would have printed and needs no process of its own.

 @module
 */

import type { ProcessUnderARun, } from './processes-under-a-pi-run.ts';
import { SECONDS_PER_MINUTE, } from './overdue-constants.ts';

/**
 How much of a command line the line prints, when the caller names no width. A command line can hold a whole document, and the head
 names the work.
 */
export const COMMAND_LINE_CHARACTERS = 200;

/**
 One process whose command line holds the text, as a line to read.
 */
export type CommandLineMatch = {
  /**
   The process number.
   */
  readonly pid: number;

  /**
   The process number of the process that started it.
   */
  readonly parentPid: number;

  /**
   How long it has been running, in seconds.
   */
  readonly elapsedSeconds: number;

  /**
   Its command line, cut to the width the caller asked for.
   */
  readonly command: string;
};

/**
 Says whether a command line holds a text.

 @param command - the command line, as `ps` printed it

 @param text - the text to look for

 @returns whether the command line holds it

 @example
 ```ts
 const holds = commandHolds({ command: 'node /runs/launcher.ts --name=a', text: '--name=a', },);
 ```
 */
export function commandHolds({
  command,
  text,
}: {
  readonly command: string;
  readonly text: string;
},): boolean {
  return command.includes(text,);
}

/**
 Cuts a command line to the width a line shows.

 @param command - the command line

 @param characters - how much of it to keep

 @returns the command line, with an ellipsis when it was longer

 @example
 ```ts
 const shown = cutCommandLine({ command: 'node /runs/a.ts', characters: 8, },);
 ```
 */
function cutCommandLine({
  command,
  characters,
}: {
  readonly command: string;
  readonly characters: number;
},): string {
  return (command.length <= characters) ? command : `${command.slice(
    0,
    characters,
  )}...`;
}

/**
 Reads every process whose command line holds a text.

 The listing is read once, so a process that ends while it is being read is a process this command does not name.
 The command excludes its own process: a caller that passes text naming this command would otherwise always find itself.

 @param table - the process table, as `ps` printed it

 @param text - the text a command line must hold

 @param excludingPid - the process number not to name, which is this command's own

 @param commandCharacters - how much of each command line to print

 @returns one entry per matching process, in the order the table listed them

 @example
 ```ts
 const found = processesWhoseCommandLineHolds({ table, text: 'launcher.ts', excludingPid: 1, commandCharacters: 200, },);
 ```
 */
export function processesWhoseCommandLineHolds({
  table,
  text,
  excludingPid,
  commandCharacters = COMMAND_LINE_CHARACTERS,
}: {
  readonly table: readonly ProcessUnderARun[];
  readonly text: string;
  readonly excludingPid: number;
  readonly commandCharacters?: number;
},): readonly CommandLineMatch[] {
  return table.filter(function isAnotherProcess(process,) {
    return (process.pid !== excludingPid) && commandHolds({
      command: process.command,
      text,
    },);
  },)
    .map(function asMatch(process,) {
      return {
        pid: process.pid,
        parentPid: process.parentPid,
        elapsedSeconds: process.elapsedSeconds,
        command: cutCommandLine({
          command: process.command,
          characters: commandCharacters,
        },),
      };
    },);
}

/**
 Says one match as the line a reader acts on.

 @param match - the process

 @returns the line: its number, its parent, its age in minutes, and its command line

 @example
 ```ts
 const line = matchLineOf({ pid: 12, parentPid: 1, elapsedSeconds: 120, command: 'node a.ts', },);
 ```
 */
export function matchLineOf(match: CommandLineMatch,): string {
  return `${String(match.pid,)} parent ${String(match.parentPid,)} age ${String(Math.floor(match.elapsedSeconds / SECONDS_PER_MINUTE,),)}m ${match.command}`;
}
