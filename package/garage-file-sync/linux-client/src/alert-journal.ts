/**
 Reads the journal of the run an alert describes, and pulls its causes out of the log.

 The previous script read the unit's latest invocation when systemd passed no id, and
 fell back to a thirty-day look-back when that run logged no error of its own. Five-
 minute runs make a look-back both expensive and misleading, so the read is always the
 failed run's own invocation.

 @module
 */

import {
  type CommandRunner,
  runCommand,
} from './run-command.ts';

/**
 How long the journal read may take, in milliseconds.

 GLib waits up to 25 s for a notification server reply and the alert unit allows 90 s
 in all, so the journal gets the same 30 s the notification does.
 */
export const JOURNAL_TIMEOUT_MS = 30_000;

/**
 Escape character that starts a terminal colour sequence.
 */
const ESCAPE = '\u001B';

/**
 Prefix of a terminal colour sequence.
 */
const ESCAPE_BRACKET = `${ESCAPE}[`;

/**
 Prefix of a bisync critical-error line.
 */
const CRITICAL_ERROR_PREFIX = 'Bisync critical error: ';

/**
 Markers of lines that name no single cause, so the alert does not quote them as one.
 */
const GENERIC_MARKERS: readonly string[] = [
  'not deleting files as there were IO errors',
  'not deleting directories as there were IO errors',
  'cannot find prior Path1 or Path2 listings',
  'Bisync aborted',
];

/**
 Marker that starts a per-file failure in an error line.
 */
const FILE_FAILURE_MARKER = ': Failed to ';

/**
 How many causes one alert quotes.
 */
const MAX_CAUSES = 3;

/**
 What one journal read produced.

 `problem` is set when the read failed or timed out; the alert then says the cause is
 unknown instead of quoting a partial history.
 */
export type JournalRead = {
  readonly lines: readonly string[];
  readonly problem?: string;
};

/**
 Which run's journal is read.

 `invocationId` is empty when systemd passed none, and the read then asks for the
 unit's latest invocation.
 */
export type JournalRequest = {
  readonly unit: string;
  readonly invocationId: string;
};

/**
 Reads the journal of one run and reports its lines, or why it could not use one.
 */
export type JournalReader = (request: Readonly<JournalRequest>,) => Promise<JournalRead>;

/**
 Arguments for the journal read of one run.

 @param unit - the unit whose run is read

 @param invocationId - systemd invocation id of that run, empty for the latest one

 @returns the arguments after the command name

 @example
 ```ts
 journalArguments({ unit: 'a.service', invocationId: '', },); // [..., '--invocation=0']
 ```
 */
export function journalArguments({
  unit,
  invocationId,
}: Readonly<JournalRequest>,): readonly string[] {
  return [
    '--user',
    `--unit=${unit}`,
    '--output=cat',
    '--no-pager',
    `--invocation=${invocationId === '' ? '0' : invocationId}`,
  ];
}

/**
 Whether a character may sit between a colour sequence's bracket and its final letter.

 @param character - one character of the sequence

 @returns true for a digit or a semicolon
 */
function isSequenceParameter(character: string,): boolean {
  return ((character >= '0') && (character <= '9')) || (character === ';');
}

/**
 Finds the end of the colour sequence that starts at `start`.

 @param text - line holding the sequence

 @param start - index of the escape character

 @returns the index just after the sequence's final `m`, or -1 when the sequence is
   not a colour sequence
 */
function sequenceEnd({
  text,
  start,
}: Readonly<{
  text: string;
  start: number;
}>): number {
  for (let cursor = start + ESCAPE_BRACKET.length; cursor < text.length; cursor += 1) {
    /**
     Character at the cursor.
     */
    const character = text[cursor];
    if (character === 'm') return cursor + 1;
    if ((character === undefined) || (!isSequenceParameter(character,))) return -1;
  }
  return -1;
}

/**
 Removes terminal colour sequences from a line.

 @param text - line as journalctl printed it

 @returns the line without colour sequences

 @example
 ```ts
 stripAnsi('\u001B[0;32mBisync successful\u001B[0m',); // 'Bisync successful'
 ```
 */
export function stripAnsi(text: string,): string {
  /**
   Kept pieces of the line, in order.
   */
  const pieces: string[] = [];
  for (let cursor = 0; cursor < text.length;) {
    /**
     Index of the next escape character, or -1 when none is left.
     */
    const start = text.indexOf(
      ESCAPE_BRACKET,
      cursor,
    );
    if (start === (-1)) {
      pieces.push(text.slice(cursor,),);
      break;
    }
    /**
     Index just after the sequence, or -1 when it is not a colour sequence.
     */
    const end = sequenceEnd({
      text,
      start,
    },);
    if (end === (-1)) {
      pieces.push(text.slice(cursor,),);
      break;
    }
    pieces.push(text.slice(
      cursor,
      start,
    ),);
    cursor = end;
  }
  return pieces.join('',);
}

/**
 Builds the journal reader over one command runner.

 The runner is a parameter so a test can drive a readable journal, a refused one and a
 journalctl that never ran, without a user journal on the machine that runs the tests.

 @param run - runs one command to its end

 @returns a reader that reports the run's lines, or the reason the read failed

 @example
 ```ts
 const readThroughRunner = journalReader(runCommand,);
 const read = await readThroughRunner({ unit: 'a.service', invocationId: 'abc', },);
 ```
 */
export function journalReader(run: CommandRunner,): JournalReader {
  return async function readRunJournal({
    unit,
    invocationId,
  },) {
    /**
     Finished journalctl run.
     */
    const result = await run({
      command: 'journalctl',
      args: journalArguments({
        unit,
        invocationId,
      },),
      timeoutMs: JOURNAL_TIMEOUT_MS,
    },);
    if (result.problem !== undefined) {
      return {
        lines: [],
        problem: result.problem,
      };
    }
    if (result.exitCode !== 0) {
      return {
        lines: [],
        problem: `journalctl exited ${String(result.exitCode,)}: ${result.stderr
          .trim()}`,
      };
    }
    return {
      lines: result.stdout
        .split('\n',)
        .map(function tidy(line,) {
          return stripAnsi(line,)
            .trim();
        },),
    };
  };
}

/**
 Reads the journal of one run with the real journalctl.

 @param unit - the unit whose run is read

 @param invocationId - systemd invocation id of that run, empty for the latest one

 @returns the run's lines, trimmed and without colour, or the reason the read failed

 @example
 ```ts
 const read = await readRunJournal({ unit: 'a.service', invocationId: 'abc', },);
 read.lines; // ['ERROR : ...'] when the run logged one
 ```
 */
export const readRunJournal: JournalReader = journalReader(runCommand,);

/**
 The text after the `ERROR :` marker of a journal line.

 @param line - one trimmed journal line

 @returns the error text, or an empty string when the line is not an error line
 */
function errorText(line: string,): string {
  /**
   Marker every bisync error line starts with.
   */
  const marker = 'ERROR';
  if (!line.startsWith(marker,)) return '';
  /**
   Line after the marker, with its spacing.
   */
  const afterSpaces = line.slice(marker.length,)
    .trimStart();
  if (!afterSpaces.startsWith(':',)) return '';
  return afterSpaces.slice(1,)
    .trimStart();
}

/**
 The messages of the lines that carry the `ERROR` marker.

 @param lines - journal lines of one run

 @returns one message per error line, in journal order

 @example
 ```ts
 errorLines(['ERROR : disk full', 'Bisync successful',],); // ['disk full']
 ```
 */
export function errorLines(lines: readonly string[],): readonly string[] {
  /**
   Messages of the error lines, in journal order.
   */
  const errors: string[] = [];
  for (const line of lines) {
    /**
     Message of this line, empty when it is not an error line.
     */
    const text = errorText(line,);
    if (text !== '') errors.push(text,);
  }
  return errors;
}

/**
 Reads one per-file failure out of an error line.

 The live rclone line looks like `<path>: Failed to <verb>: <detail>: <reason>`, and the
 reason is what a person can act on.

 @param line - one error message

 @returns `<path>: <reason>`, or an empty string when the line is not a per-file failure
 */
function fileFailureText(line: string,): string {
  /**
   Index where the per-file marker starts, or -1 when the line has none.
   */
  const markerStart = line.indexOf(FILE_FAILURE_MARKER,);
  if (markerStart === (-1)) return '';
  /**
   Index of the reason's colon, the last one in the line.
   */
  const reasonColon = line.lastIndexOf(':',);
  if (reasonColon === (-1)) return '';
  /**
   Character after the reason's colon, which the line writes as a space.
   */
  const spaceAfterReason = line[reasonColon + 1];
  if (spaceAfterReason !== ' ') return '';
  /**
   Reason the file failed.
   */
  const reason = line.slice(reasonColon + 1,)
    .trim();
  if (reason === '') return '';
  return `${line.slice(
    0,
    markerStart,
  )}: ${reason}`;
}

/**
 Removes the repeated parts of a bisync critical error line.

 @param line - one error message

 @param localRoot - the local bisync root with a trailing separator

 @returns the message without its prefix and without the local root path
 */
function shortenCriticalError({
  line,
  localRoot,
}: Readonly<{
  line: string;
  localRoot: string;
}>): string {
  /**
   Message without the constant prefix.
   */
  const withoutPrefix = line.startsWith(CRITICAL_ERROR_PREFIX,)
    ? line.slice(CRITICAL_ERROR_PREFIX.length,)
    : line;
  return withoutPrefix.replaceAll(
    localRoot,
    ''
  );
}

/**
 Picks the causes worth quoting in an alert.

 Every per-file failure becomes `<path>: <reason>`; when none exists, the critical
 errors are quoted with the local root shortened. At most three distinct causes.

 @param errors - error messages of the run

 @param localRoot - the local bisync root with a trailing separator

 @returns up to three distinct causes, in journal order

 @example
 ```ts
 causeLines({ errors: ['Plain/a.txt: Failed to copy: x: disk full',], localRoot: '/home/me/Plain/', },);
 // ['Plain/a.txt: disk full']
 ```
 */
export function causeLines({
  errors,
  localRoot,
}: Readonly<{
  errors: readonly string[];
  localRoot: string;
}>): readonly string[] {
  /**
   Error messages that name no single cause.
   */
  const specific = errors.filter(function isSpecific(errorLine,) {
    return !GENERIC_MARKERS.some(function matches(marker,) {
      return errorLine.includes(marker,);
    },);
  },);
  /**
   Per-file failures among the specific messages.
   */
  const fileFailures: string[] = [];
  for (const line of specific) {
    /**
     Readable form of this line, empty when it names no single file.
     */
    const fileFailure = fileFailureText(line,);
    if (fileFailure !== '') fileFailures.push(fileFailure,);
  }
  /**
   Causes to quote: per-file failures when there are any, the critical errors otherwise.
   */
  const chosen = fileFailures.length > 0
    ? fileFailures
    : specific.map(function shorten(line,) {
      return shortenCriticalError({
        line,
        localRoot,
      },);
    },);
  return [...new Set(chosen,),].slice(
    0,
    MAX_CAUSES,
  );
}

/**
 The last non-empty lines of a journal.

 @param lines - journal lines of one run

 @param count - how many lines to keep

 @returns at most `count` lines, oldest first

 @example
 ```ts
 tailLines({ lines: ['a', '', 'b',], count: 1, },); // ['b']
 ```
 */
export function tailLines({
  lines,
  count,
}: Readonly<{
  lines: readonly string[];
  count: number;
}>): readonly string[] {
  /**
   Lines that hold text, in journal order.
   */
  const filled = lines.filter(function hasText(line,) {
    return line !== '';
  },);
  return filled.slice(-count,);
}
