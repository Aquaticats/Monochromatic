/**
 `end-a-program-after-a-grace`: waits a stated grace, then kills a program a bound ended, so the forced kill outlives the caller that
 set the bound.

 A bound that sends the polite signal and then relies on a timer inside its own caller is no bound at all: on 2026-11-16 the review of
 `main...35d724bdbe` showed a caller that answered at its bound and exited, after which the unreferenced kill timer never ran and the
 program survived both its caller and the grace. This program is started detached, so it is nobody's child and its wait is its own.

 The number it signals is the one the caller got back when it started the program. A kernel may reuse a number for another program in
 the seconds since, so this program holds the moment the program began against the moment the number names when the grace is over: a
 number whose program began at another moment is not the one the caller was given, and is left alone.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { argv, } from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import { errorCodeOf, } from './error-code-of.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The program that says a signal was sent to a process that is not there.
 */
const MISSING_PROCESS_CODE = 'ESRCH';

/**
 The status this program ends with when its command line is not one it reads.
 */
const REFUSED_USAGE_STATUS = 2;

/**
 The word that names the program to end.
 */
const PID_WORD = '--pid=';

/**
 The word that names how long to wait before the forced kill.
 */
const GRACE_WORD = '--grace-milliseconds=';

/**
 The command line this program answers for.
 */
const USAGE = 'end-a-program-after-a-grace --pid=<number> --grace-milliseconds=<number>';

/**
 Reads one word of the command line.

 @param option - the word as it is written, with its `--` and its `=`

 @returns its value, and nothing when the command line names it not at all

 @example
 ```ts
 const pid = wordValueOf({ option: '--pid=', },);
 ```
 */
function wordValueOf({ option, }: { readonly option: string },): { readonly value?: string } {
  /**
   The word that names the option.
   */
  const written = argv.slice(2,)
    .find(function isTheOption(word,) {
      return word.startsWith(option,);
    },);
  return (written === undefined) ? {} : { value: written.slice(option.length,), };
}

/**
 Reads when the program of one process number began, from the kernel's own record.

 @param pid - the process number

 @returns when it began, in the kernel's ticks since boot, and nothing when no process of that number runs

 @example
 ```ts
 const { began } = await startTicksOf({ pid: 12, },);
 ```
 */
async function startTicksOf({ pid, }: { readonly pid: number },): Promise<{ readonly began?: number }> {
  /**
   The kernel's record of that process, and nothing when it names none.
   */
  const { recorded, } = await recordOf({ pid, },);
  if (recorded === undefined) {
    return {};
  }
  /**
   Where the program's own name ends, after which the fields this reads begin.
   */
  const afterTheName = recorded.lastIndexOf(')',) + 1;
  if (afterTheName === 0) {
    return {};
  }
  /**
   The fields that follow it, of which the state is the first and the start time the twentieth.
   */
  const fields = recorded.slice(afterTheName,)
    .trim()
    .split(' ',);
  /**
   Where the start time sits among those fields: the kernel's twenty-second field, counted from one.
   */
  const START_TIME_FIELD = 22;
  /**
   The state's own field number, counted from one, which is the first of those fields.
   */
  const STATE_FIELD = 3;
  /**
   The moment it began, in the kernel's ticks.
   */
  const began = Number(fields[START_TIME_FIELD - STATE_FIELD],);
  return Number.isSafeInteger(began,) && (began > 0) ? { began, } : {};
}

/**
 Reads the kernel record of one process number.

 @param pid - the process number

 @returns its record, and nothing when no process of that number runs or the kernel keeps none

 @example
 ```ts
 const { recorded } = await recordOf({ pid: 12, },);
 ```
 */
async function recordOf({ pid, }: { readonly pid: number },): Promise<{ readonly recorded?: string }> {
  try {
    return { recorded: await readFile(
      `/proc/${String(pid,)}/stat`,
      'utf8',
    ), };
  }
  catch (error) {
    // A number the kernel no longer knows runs no program, and a kernel without this record cannot answer either.
    if ((errorCodeOf(error,) === MISSING_PROCESS_CODE) || (errorCodeOf(error,) === MISSING_PATH_CODE)) {
      return {};
    }
    throw error;
  }
}

/**
 Kills one program by its number, and answers nothing when it is already gone.

 @param pid - the process number

 @example
 ```ts
 await killNow({ pid: 12, },);
 ```
 */
function killNow({ pid, }: { readonly pid: number },): void {
  try {
    process.kill(
      pid,
      'SIGKILL',
    );
  }
  catch (error) {
    // A program that ended between the read and the signal is exactly what this program was for.
    if (errorCodeOf(error,) !== MISSING_PROCESS_CODE) {
      throw error;
    }
  }
}

/**
 Waits the grace and kills the program, when the number still names the same one.

 @param pid - the process number the caller got back when it started the program

 @param graceMilliseconds - how long to wait after the polite signal

 @example
 ```ts
 await endTheProgramOfTheBound({ pid: 12, graceMilliseconds: 5_000, },);
 ```
 */
async function endTheProgramOfTheBound({
  pid,
  graceMilliseconds,
}: {
  readonly pid: number;
  readonly graceMilliseconds: number;
},): Promise<void> {
  /**
   When that number's program began, held until the grace is over.
   */
  const { began, } = await startTicksOf({ pid, },);

  await wait(graceMilliseconds,);

  /**
   When that number's program begins to exist when the grace is over.
   */
  const { began: beganNow, } = await startTicksOf({ pid, },);

  // A number whose program began at another moment, or names no program now, is not the one this program was given.
  if ((began === undefined) || (beganNow === undefined)
    || (beganNow !== began)) {
    return;
  }
  killNow({ pid, },);
}

/**
 The process number to end.
 */
const pidWord = wordValueOf({ option: PID_WORD, },);

/**
 How long to wait before the forced kill.
 */
const graceWord = wordValueOf({ option: GRACE_WORD, },);

/**
 The number, as a safe whole number above zero, or nothing when the word is no such number.
 */
const pid = Number(pidWord.value,);

/**
 How long to wait, as a safe whole number of milliseconds, or nothing when the word is no such number.
 */
const graceMilliseconds = Number(graceWord.value,);

if (
  (!Number.isSafeInteger(pid,)) || (pid <= 0)
    || (!Number.isSafeInteger(graceMilliseconds,))
    || (graceMilliseconds < 0)
) {
  process.stderr
    .write(`${USAGE}\n`,);
  process.exitCode = REFUSED_USAGE_STATUS;
}
else {
  await endTheProgramOfTheBound({
    pid,
    graceMilliseconds,
  },);
}
