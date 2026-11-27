/**
 The line a start that ended without a run leaves: which wait ended it, the bound that wait is given, how long it had waited, and
 what to do next.

 A start can end without a run in ways the summary cannot name, and in one way the summary cannot even be written: the driver reads
 this machine's secret environment before it starts Pi, and a store that cannot be read stops the run there. On 2026-11-26 four
 starts ended that way in 32 seconds each, leaving one feed line, no summary and no standard error, and the unit's status was all a
 reader had (issue 6556). The same silence covered the three answers a run waits for while Pi starts, which ended three starts
 without naming the ask, its bound, or the time it had taken.

 So every such ending writes this line under its own name in the run folder, prints it where the ending is printed, and carries its
 facts in the summary when a summary can be written. A launcher prints the line from the folder, which is what makes a start that
 wrote no summary still tell a reader how it ended.

 @module
 */

import {
  lstat,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import {
  MASKING_NOTHING,
  type TextMasking,
} from './secret-value-masking.ts';

/**
 The name the line takes in a run folder.
 */
export const START_WITHOUT_A_RUN_FILE_NAME = 'start-ended-without-a-run.txt';

/**
 The code the operating system gives a name that is already taken.
 */
const ALREADY_THERE_CODE = 'EEXIST';

/**
 One wait that ended a start, named for a reader who will follow it to the call that ran it.
 */
export type WaitThatEndedAStart = {
  /**
   The wait, in words that name the call or the file: `get_state`, the first prompt, the sops decrypt of the checkout's store.
   */
  readonly wait: string;

  /**
   How long that wait is given, in words that hold its unit, such as `120000 ms` or `3 starts`; absent for a wait that has no bound
   of its own because the thing it waited on answered.
   */
  readonly bound?: string;

  /**
   How long it had waited when it ended, in milliseconds.
   */
  readonly waitedMilliseconds: number;
};

/**
 What a start that ended without a run has to say, and all a reader needs to answer it.
 */
export type StartThatEndedWithoutARun = {
  /**
   The waits that ended it, in the order they ended: one for a single bound running out, and two when Pi left and the bound on the
   whole sequence of starts then refused another.
   */
  readonly waits: readonly WaitThatEndedAStart[];

  /**
   Why no run happened, in words that say what the end leaves behind.
   */
  readonly what: string;

  /**
   What to do next, naming every way this ending can be answered.
   */
  readonly next: string;
};

/**
 Says one wait in the words the line carries: what it waited for, how long it is given, and how long it had waited.

 @param wait - the wait

 @returns the wait as one clause
 */
function waitInWords({
  wait,
  bound,
  waitedMilliseconds,
}: WaitThatEndedAStart,): string {
  /**
   How long it had waited, in words that hold the unit.
   */
  const waited = `${String(waitedMilliseconds,)} ms`;
  return (bound === undefined)
    ? `${wait}: no bound of its own, and it had waited ${waited}`
    : `${wait}: given ${bound}, and it had waited ${waited}`;
}

/**
 Says the waits that ended a start, in the order they ended, as one sentence a line can carry beside what else it says.

 @param start - the start

 @returns the sentence, which names every wait
 */
function waitsSentence({
  waits,
}: {
  readonly waits: readonly WaitThatEndedAStart[]
},): string {
  /**
   The waits, each in its own words.
   */
  const named = waits.map(waitInWords,);
  if (named.length === 1) {
    return `The wait that ended it: ${named[0] ?? ''}.`;
  }
  return `The waits that ended it, in the order they ended: ${named.join('; then ',)}.`;
}

/**
 Says what a start that ended without a run has to say, without the folder or the fact of the ending, so a line that already names
 both can carry it.

 @param start - the start

 @returns the sentence, which names the waits, why no run happened, and what to do next
 */
export function sentenceOfAStartThatEndedWithoutARun({
  start,
}: {
  readonly start: StartThatEndedWithoutARun
},): string {
  return `${start.what}. ${waitsSentence(start,)} What to do next: ${start.next}`;
}

/**
 Says the whole line a start that ended without a run leaves, for a reader who has the folder but not the summary.

 @param runFolder - the run's own folder

 @param start - the start

 @returns the line, without its line ending
 */
export function lineOfAStartThatEndedWithoutARun({
  runFolder,
  start,
}: {
  readonly runFolder: string;
  readonly start: StartThatEndedWithoutARun;
},): string {
  return `The start in ${runFolder} ended without a run. ${sentenceOfAStartThatEndedWithoutARun({ start, },)}`;
}

/**
 Writes the line a start that ended without a run leaves, under its own name in the run folder.

 The name is created rather than opened, so a line an earlier attempt left in the same folder is never written over: that folder is
 another attempt's record, and a refusal says so instead of replacing it.

 @param runFolder - the run's own folder, which the driver made before it asked anything

 @param start - the start

 @param mask - the masking every value of this machine's secret environment goes through, which this line is written through as
 every other file of a run is

 @returns the line as written, or what kept it from being written
 */
export async function writeAStartThatEndedWithoutARun({
  runFolder,
  start,
  mask = MASKING_NOTHING,
}: {
  readonly runFolder: string;
  readonly start: StartThatEndedWithoutARun;
  readonly mask?: TextMasking;
},): Promise<string> {
  /**
   The line, through the masking: a path or a bound can hold a value this machine masks, and no file of a run carries one.
   */
  const line = mask.maskText(lineOfAStartThatEndedWithoutARun({ runFolder, start, }),);
  try {
    await writeFile(
      join(
        runFolder,
        START_WITHOUT_A_RUN_FILE_NAME,
      ),
      `${line}\n`,
      { flag: 'wx', },
    );
    return line;
  }
  catch (error) {
    if (!Error.isError(error,)) {
      throw error;
    }
    /**
     The code the operating system answered, which tells a taken name from a folder that cannot be written.
     */
    const code = errorCodeOf(error,);
    if (code === ALREADY_THERE_CODE) {
      return `The start in ${runFolder} ended without a run, and its line was not written: ${join(
        runFolder,
        START_WITHOUT_A_RUN_FILE_NAME,
      )} is already there, left by an earlier attempt. Read that file, or the summary beside it, for how that attempt ended.`;
    }
    throw error;
  }
}

/**
 Reads the line a start that ended without a run left, for a launcher that prints what the start said.

 A file that cannot be read is reported as the one line it is, rather than left out: a launcher that printed nothing for it would
 read as one whose run ended with a run.

 @param runFolder - the run's own folder

 @returns the file's lines, or none when the start left none

 @throws Error when the file is there and cannot be read
 */
export async function linesOfAStartThatEndedWithoutARun({
  runFolder,
}: {
  readonly runFolder: string;
},): Promise<readonly string[]> {
  /**
   The file a start that ended without a run leaves.
   */
  const file = join(
    runFolder,
    START_WITHOUT_A_RUN_FILE_NAME,
  );
  try {
    /**
     What that name is, read without following anything: a folder or a pipe under it is not the line a start left.
     */
    const kept = await lstat(file,);
    if (!kept.isFile()) {
      throw new Error(`${file} is not an ordinary file, so the line this start left cannot be read`,);
    }
    return (await readFile(
      file,
      'utf8',
    )).split('\n',)
      .filter((line,) => line !== '');
  }
  catch (error) {
    if (!Error.isError(error,)) {
      throw error;
    }
    if (errorCodeOf(error,) === 'ENOENT') {
      return [];
    }
    throw error;
  }
}

/**
 Prints the line a start that ended without a run left, so a launcher's own output names the wait and what to do next.

 @param runFolder - the run's own folder

 @param report - where the launcher prints a line
 */
export async function reportAStartThatEndedWithoutARun({
  runFolder,
  report,
}: {
  readonly runFolder: string;
  readonly report: (line: string) => void;
},): Promise<void> {
  try {
    for (const line of await linesOfAStartThatEndedWithoutARun({ runFolder, },)) {
      report(line,);
    }
  }
  catch (error) {
    if (!Error.isError(error,)) {
      throw error;
    }
    // A line that is there and cannot be read is named, so a start never reads as one that ended with a run.
    report(
      `${runFolder} holds a line saying how a start ended without a run, and it could not be read: ${error.message}`,
    );
  }
}
