/**
 What a guest's pulled log must prove, and what one line of it means.

 An arm that returns without running is the failure these drivers exist to
 catch: a `SKIPPED` line, or a log without the `DONE=` line its detached script
 ends with, is a run that did nothing, and it has to fail the host's run rather
 than read as a green suite. The marker check is what every arm states its own
 lines against.

 @example
 ```ts
 requireMarkers({ pulled, markers: ['ADOPTION done'], subject: 'the pulled arm log', because: 'the guest ran the arm without performing it' });
 ```
 */

import { readFile, } from 'node:fs/promises';

/**
 The prefix a detached guest script writes its own status behind.
 */
const donePrefix = 'DONE=';

/**
 Whether one detached guest script said how it ended.
 */
export type DoneStatus = {
  /**
   True for a log whose `DONE=` line names a status.
   */
  readonly found: true;
  /**
   The digits that line holds, as text, so a status of zero is not `0` alone.
   */
  readonly status: string;
} | {
  /**
   True for a log with no `DONE=` line naming a status.
   */
  readonly found: false;
};

/**
 Read one pulled log.

 @param path - where the pulled log sits on the host

 @returns its whole text

 @example
 ```ts
 await readLog({ path: join(logDir, 'adoption-arm.log') });
 ```
 */
export function readLog({
  path,
}: {
  readonly path: string;
},): Promise<string> {
  return readFile(
    path,
    'utf8',
  );
}

/**
 Refuse a pulled log that lacks any of the lines an arm performs.

 @param pulled - the log's whole text

 @param markers - the lines the arm prints, one per claim it makes

 @param subject - what the log is called in the refusal

 @param because - why a missing line is a failure, after the semicolon

 @throws Error naming the first marker the log lacks

 @example
 ```ts
 requireMarkers({ pulled, markers: ['SWEEP done'], subject: 'the pulled sweep-arm log', because: 'the guest ran the arm without performing it' });
 ```
 */
export function requireMarkers({
  pulled,
  markers,
  subject,
  because,
}: {
  readonly pulled: string;
  readonly markers: readonly string[];
  readonly subject: string;
  readonly because: string;
},): void {
  for (const marker of markers)
    if (!pulled.includes(marker))
      throw new Error(`${subject} lacks ${marker}; ${because}`);
}

/**
 Whether one of a log's lines starts with a word, which is how a skipped arm
 announces itself.

 @param pulled - the log's whole text

 @param word - the word a line has to start with

 @returns true when any line starts with that word

 @example
 ```ts
 holdsLineStartingWith({ pulled, word: 'SKIPPED' });
 ```
 */
export function holdsLineStartingWith({
  pulled,
  word,
}: {
  readonly pulled: string;
  readonly word: string;
},): boolean {
  return pulled
    .split('\n')
    .some(function startsWithTheWord(line,) {
      return line.startsWith(word,);
    },);
}

/**
 The status a detached guest script ended with, read from the `DONE=` line it
 appends to its own log.

 The line is the script's last word on how it ended, so the parser takes the
 first line that is exactly `DONE=` and digits: anything else, including no line
 at all, is a script that ended without saying.

 @param pulled - the pulled log's whole text

 @returns whether the log names a status, and the digits it names

 @example
 ```ts
 doneStatusOf({ pulled: 'arm output\nDONE=0\n' });
 ```
 */
export function doneStatusOf({
  pulled,
}: {
  readonly pulled: string;
},): DoneStatus {
  /**
   The digits of every `DONE=` line that names a whole number, in the order the
   log holds them.
   */
  const named = pulled
    .split('\n')
    .filter(function isDoneLine(line,) {
      return line.startsWith(donePrefix,);
    },)
    .map(function digitsOfDoneLine(line,) {
      return line.slice(donePrefix.length,);
    },)
    .filter(function namesAStatus(text,) {
      return isWholeNumber({ text, },);
    },);
  /**
   The first status the log names, absent when it names none.
   */
  const [status,] = named;
  if (status === undefined) return { found: false, };
  return {
    found: true,
    status,
  };
}

/**
 Whether one text is a whole number written in digits.

 The scan counts characters rather than code points on purpose: a status is
 ASCII digits, and anything else, including a character an emoji is built from,
 is a status this does not name.

 @param text - the text after a `DONE=` line's prefix

 @returns true when the text is one or more digits and nothing else

 @example
 ```ts
 isWholeNumber({ text: '17' });
 ```
 */
function isWholeNumber({
  text,
}: {
  readonly text: string;
},): boolean {
  if (text.length === 0) return false;
  for (let at = 0; at < text.length; at += 1) {
    /**
     The character at this position, which is one UTF-16 unit and not one word.
     */
    const character = text.charAt(at,);
    if ((character < '0') || (character > '9')) return false;
  }
  return true;
}
