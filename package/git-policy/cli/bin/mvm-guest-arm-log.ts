/**
 What a guest driver reads back out of an arm's pulled log: whether a line begins with a word, which of the lines an arm
 prints a log lacks, and the refusal a log that lacks one earns.

 An arm that returns is not an arm that ran, which every guest driver of this package holds: the test binary prints its
 `SKIP` line and exits zero when its variables did not reach it, so a green exit status alone would report a suite that
 measured nothing. A driver therefore judges the pulled log itself, and the readings it makes live here. Three sets of
 guest helpers each held a copy (`bin/guest-route-log-lines.ts`, `bin/mvm-guest-arm-log.ts` and
 `e2e/guest-host/guest-host-rules-fixture.ts`), and the surviving scan is one pass over the text with no regular
 expression, because a pulled log is the guest's own output and has no bound.
 */

import { GuestDriverError, } from './mvm-guest-error.ts';

/**
 The markers a pulled log does not hold, in the order the driver listed them.

 Every marker is one claim the arm printed, so a log that lacks one ran without performing what it is named for. The
 markers are compared as whole strings and never as patterns, because they are the arm's own printed lines.

 @param text - the whole log the guest wrote

 @param markers - the lines the run must find, one per claim

 @returns the markers the log lacks, empty when it holds every one

 @example
 ```ts
 missingMarkers({ text: 'LOCK done\n', markers: ['LOCK done', 'LOCK released=yes'] }); // ['LOCK released=yes']
 ```
 */
export function missingMarkers({
  text,
  markers,
}: {
  readonly text: string;
  readonly markers: readonly string[];
},): readonly string[] {
  /**
   The markers the log does not hold, collected in the order they were asked for.
   */
  const missing: string[] = [];
  for (const marker of markers) {
    if (!text.includes(marker,))
      missing.push(marker,);
  }
  return missing;
}

/**
 Refuse a pulled log that lacks any of the lines an arm performs.

 The refusal names the log the run kept, the line the arm never printed, and why a missing line is a failure, so a run
 that only returned is a failed run rather than a green suite.

 @param text - the log's whole text

 @param markers - the lines the arm prints, one per claim it makes

 @param subject - what the log is called in the refusal, such as `the pulled adoption-arm log`

 @param because - why a missing line is a failure, after the semicolon

 @throws GuestDriverError naming the first marker the log lacks

 @example
 ```ts
 requireMarkers({ text: pulled, markers: ['SWEEP done',], subject: 'the pulled sweep-arm log', because: 'the guest ran the arm without performing it' });
 ```
 */
export function requireMarkers({
  text,
  markers,
  subject,
  because,
}: {
  readonly text: string;
  readonly markers: readonly string[];
  readonly subject: string;
  readonly because: string;
},): void {
  /**
   The markers the log lacks, of which the first one is the refusal.
   */
  const [missing,] = missingMarkers({
    text,
    markers,
  },);
  if (missing !== undefined)
    throw new GuestDriverError(`${subject} lacks ${missing}; ${because}`,);
}

/**
 Whether a log holds a line that begins with one word.

 The word is the arm's own: a test binary that skipped prints `SKIP` or `SKIPPED` at the start of a line and exits zero,
 so a driver reads the start of every line rather than any occurrence anywhere in the log. One pass over the text, with
 no regular expression, because a pulled log is the guest's own output and has no bound.

 @param text - the whole log the guest wrote

 @param word - the word a line must begin with, such as `SKIP`

 @returns true when one line begins with the word

 @example
 ```ts
 holdsLineStartingWith({ text: 'building\nSKIP the arm did not run\n', word: 'SKIP' }); // true
 ```
 */
export function holdsLineStartingWith({
  text,
  word,
}: {
  readonly text: string;
  readonly word: string;
},): boolean {
  for (const line of text.split('\n',)) {
    if (line.startsWith(word,))
      return true;
  }
  return false;
}
