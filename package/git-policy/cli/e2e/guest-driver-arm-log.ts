/**
 What a guest driver reads back out of an arm's pulled log, kept where a unit test can hold it.

 An arm that returns is not an arm that ran, which every guest driver of this package holds: the test binary prints its
 `SKIP` line and exits zero when its variables did not reach it, so a green exit status alone would report a suite that
 measured nothing. A driver therefore judges the pulled log itself, and the two readings it makes live here.
 */

import { GuestDriverError, } from './guest-driver-error.ts';

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
 The refusal for an arm log that lacks one of the lines it must hold.

 The arm prints one line per claim it proved, so a log that lacks one ran without performing it: the driver refuses
 before it reports anything green, and names the claim the arm never made.

 @param marker - the line the log lacks

 @returns the refusal to throw

 @example
 ```ts
 throw missingMarkerRefusal({ marker: 'LOCK done' });
 ```
 */
export function missingMarkerRefusal({ marker, }: { readonly marker: string; },): GuestDriverError {
  return new GuestDriverError(`the pulled arm log lacks ${marker}; the guest ran the arm without performing it`,);
}

/**
 Whether a log holds a line that begins with one word.

 The word is the arm's own: a test binary that skipped prints `SKIP` or `SKIPPED` at the start of a line and exits zero,
 so a driver reads the start of every line rather than any occurrence anywhere in the log. One pass over the text,
 with no regular expression, because a pulled log is the guest's own output and has no bound.

 @param text - the whole log the guest wrote

 @param word - the word a line must begin with, such as `SKIP`

 @returns true when one line begins with the word

 @example
 ```ts
 holdsWordLine({ text: 'building\nSKIP the arm did not run\n', word: 'SKIP' }); // true
 ```
 */
export function holdsWordLine({
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
