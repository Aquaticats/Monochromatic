/**
 Reads one gate's verdict out of its log: the lines a person needs, and a bounded wait for them to arrive.

 A gate is read from its log and never from its unit, because the unit is collected when it ends and `systemctl show` then
 reports the defaults of a unit that does not exist
 (`doc/troubleshooting/systemd-run-collect-reports-success-for-a-failed-unit.md`). What matters in the log is the exit
 status the unit wrote as its last line, nextest's summary, and every line that names a failure, which is what this reads.
 */
import { readFile } from 'node:fs/promises';

import { wait as waitUntil } from '@monochromatic-dev/async-time-core/ts';

import { NativeGateError } from './native-gate-error.ts';
import { gateExitStatusMarker } from './native-gate-unit.ts';

/**
 Milliseconds between two looks at a gate's log when the caller names no interval.

 A full gate writes megabytes over the better part of an hour, so a look every twenty seconds costs nothing and still
 finds a verdict within one interval of the unit ending.
 */
export const defaultVerdictPollMilliseconds = 20_000;

/**
 How many of the log's interesting lines a verdict prints: the tail of the log is where the failures and the summary are,
 and forty lines hold a run's failures when they are few.
 */
export const verdictLineLimit = 40;

/**
 Text nextest prints in front of its summary of the whole run.
 */
const summaryMarker = 'Summary [';

/**
 The text every line that names a failure starts with: nextest's own tag for a test that failed or was stopped, and the
 word a nextest or compiler error line begins with.
 */
const failureStarts = [
  'FAIL',
  'SIGABRT',
  'SIGKILL',
  'TIMEOUT',
] as const;

/**
 The text an error line begins with.
 */
const errorStarts = 'error';

/**
 Tells whether one character continues a word, which is what decides that `FAIL [` starts a status line and `FAILURE`
 does not.

 @param character - the character after a failure's tag

 @returns true for a letter, a digit or an underscore

 @example
 ```ts
 continuesWord('a');
 ```
 */
function continuesWord(character: string): boolean {
  return ((character >= 'a') && (character <= 'z'))
    || ((character >= 'A') && (character <= 'Z'))
    || ((character >= '0') && (character <= '9'))
    || (character === '_');
}

/**
 Tells whether one line of a gate log answers what happened: the exit status the unit wrote, nextest's summary, a tag for
 a failed or stopped test, or an error.

 @param line - one line of the log

 @returns true when a verdict prints this line

 @example
 ```ts
 isInterestingLine({ line: '        FAIL [   1.204s] (2/2) monochromatic-git-policy-cli c::d' });
 ```
 */
export function isInterestingLine({ line }: { readonly line: string; }): boolean {
  if (line.includes(summaryMarker) || line.includes(gateExitStatusMarker)
    || line.startsWith(errorStarts))
    return true;
  /**
   The line without its progress indent, which nextest prints in front of a status line.
   */
  const trimmed = line.trimStart();
  return failureStarts.some(function startsWithTheTag(tag) {
    return trimmed.startsWith(tag) && (!continuesWord(trimmed.charAt(tag.length)));
  });
}

/**
 The lines of a gate log that answer what happened, without duplicates and at most {@link verdictLineLimit} of them.

 @param text - the log's text

 @returns the interesting lines, in the order the log has them

 @example
 ```ts
 verdictLinesOf({ text: 'Summary [ 1000s] 6900 tests run: 6900 passed\ngate-exit-status=0\n' });
 ```
 */
export function verdictLinesOf({ text }: { readonly text: string; }): readonly string[] {
  return [
    ...new Set(text.split('\n')
      .filter(function isInteresting(line) {
        return isInterestingLine({ line, });
      })),
  ].slice(-verdictLineLimit);
}

/**
 Reads one gate log, answering with empty text while the unit has not written it yet.

 @param logPath - absolute path of the gate's log

 @returns the log's text, or empty text when the file is not there yet

 @throws NativeGateError when the file is there and cannot be read

 @example
 ```ts
 await readGateLog('/l/gpc-gate-a-20261110T180203-0500.log');
 ```
 */
export async function readGateLog(logPath: string): Promise<string> {
  try {
    return await readFile(
      logPath,
      'utf8',
    );
  }
  catch (error) {
    // A gate that has not started writing yet has no log, which is not a failure; anything else is.
    if (((typeof error) === 'object') && (error !== null)
      && (Reflect.get(
        error,
        'code',
      ) === 'ENOENT'))
      return '';
    throw new NativeGateError(
      `the gate log ${logPath} is there and could not be read, so no verdict can be told from it: ${String(error)}`,
      { cause: error, }
    );
  }
}

/**
 How many of a log's last lines a person sees when no verdict arrived, which is what the scratch wait scripts printed: enough
 for one failure and the line under it.
 */
export const noVerdictTailLines = 30;

/**
 What one wait for a verdict found.
 */
export type VerdictOutcome =
  | {
    /**
     Marks a verdict the log already holds.
     */
    readonly kind: 'verdict';
    /**
     The interesting lines of the log, whose last one is the exit status the unit wrote.
     */
    readonly lines: readonly string[]
  }
    | {
    /**
     Marks a wait that ended without a verdict.
     */
      readonly kind: 'none';
    /**
     What ended the wait: the deadline, or a unit that stopped without writing one.
     */
      readonly reason: 'deadline' | 'unit-ended';
    /**
     The log's last lines, which say how far the run got.
     */
      readonly tail: readonly string[]
    };

/**
 The last lines of one log.

 @param text - the log's text

 @param count - how many of its last lines to keep

 @returns the last `count` lines, without the empty text a trailing newline leaves

 @example
 ```ts
 tailLinesOf({ text: 'a\nb\n', count: 1 });
 ```
 */
export function tailLinesOf({
  text,
  count,
}: {
  readonly text: string;
  readonly count: number;
}): readonly string[] {
  /**
   The log's lines, without the empty text a trailing newline leaves behind.
   */
  const lines = text.split('\n');
  return ((lines.at(-1) === '') ? lines.slice(
    0,
    -1
  ) : lines).slice(-count);
}

/**
 Waits, bounded, for a gate log to hold a verdict.

 The log is read once before the first interval passes, so a verdict that is already there answers at once, and the wait
 never outlasts the deadline: the last interval is shortened to what is left, and a deadline that has already passed is
 one look and no sleep. A unit the caller names ends the wait as well, so a gate that died before writing its verdict is
 reported as soon as systemd sees it stop rather than at the end of the deadline.

 @param logPath - absolute path of the gate's log

 @param marker - the text the log has to carry, which is the gate's exit-status line by default

 @param deadlineMilliseconds - how long this may wait, counted from its own start

 @param pollMilliseconds - milliseconds between two looks, which the deadline shortens

 @param stillRunning - asks whether the gate unit still runs; absent, only the marker or the deadline ends the wait

 @param readLog - reads the log, which a test replaces with its own stand-in

 @param wait - waits one interval, which a test replaces with a stand-in that records the interval

 @param now - reads the clock, which a test replaces with its own

 @returns the verdict lines, or the reason no verdict came and the log's last lines

 @example
 ```ts
 await waitForGateVerdict({ logPath: '/l/gate.log', marker: 'gate-exit-status=', deadlineMilliseconds: 1_800_000, pollMilliseconds: 20_000 });
 ```
 */
export async function waitForGateVerdict({
  logPath,
  marker = gateExitStatusMarker,
  deadlineMilliseconds,
  pollMilliseconds,
  stillRunning,
  readLog = readGateLog,
  wait = waitUntil,
  now = Date.now,
}: {
  readonly logPath: string;
  readonly marker?: string;
  readonly deadlineMilliseconds: number;
  readonly pollMilliseconds: number;
  readonly stillRunning?: () => Promise<boolean>;
  readonly readLog?: (logPath: string) => Promise<string>;
  readonly wait?: (milliseconds: number) => Promise<undefined>;
  readonly now?: () => number;
}): Promise<VerdictOutcome> {
  /**
   When this wait must be over.
   */
  const deadline = now() + deadlineMilliseconds;
  for (;;) {
    /**
     The log as it stands now.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- sequential polling must read the log before it sleeps again
    const text = await readLog(logPath);
    if (text.includes(marker))
      return {
        kind: 'verdict',
        lines: verdictLinesOf({ text, }),
      };
    /**
     How long is left before this wait must be over.
     */
    const left = deadline - now();
    if (left <= 0)
      return {
        kind: 'none',
        reason: 'deadline',
        tail: tailLinesOf({
          text,
          count: noVerdictTailLines,
        }),
      };
    /**
     Whether the gate unit still runs, which is true when the caller named none.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- the unit has to be asked between two reads, and its answer decides the next step
    const running = (stillRunning === undefined) ? true : await stillRunning();
    if (!running)
      return {
        kind: 'none',
        reason: 'unit-ended',
        tail: tailLinesOf({
          text,
          count: noVerdictTailLines,
        }),
      };
    // oxlint-disable-next-line eslint/no-await-in-loop -- sequential polling must delay between two reads
    await wait(Math.min(
      pollMilliseconds,
      left,
    ));
  }
}
