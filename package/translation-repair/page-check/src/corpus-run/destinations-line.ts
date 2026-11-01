import {
  lifecycleStamp,
  wallClockNow,
} from '@monochromatic-dev/translation-repair-markup/ts';
import type { DestinationCheck, } from './dropped-destinations.ts';

//region Destinations line
// THE PER-ENTRY LINE THE PASS PRINTS BESIDE ITS TALLY, counts only. The
// addresses themselves are corpus content and stay in the run log, where the
// publisher writes them; stdout carries what a grep over a pass can total.

/**
 Renders the `DESTINATIONS` line for one published entry.

 @param entryId - person entry the line is about

 @param destinations - what the publisher found on both sides

 @param attempt - number of the entry's attempt the page was published in, as the pass's attempts file holds it

 @param wallClock - clock the line's time is read on; the wall clock unless a test fixes it

 @returns One line, `DESTINATIONS <id> at=<time> attempt=<n> source=N page=M dropped=K`, followed by any finding

 @throws `LifecycleAttemptError` (markup package) when the attempt is not a whole number from 1

 @example
 ```ts
 console.log(destinationsLine({ entryId: 'BookshopCat', destinations, attempt: 2, },),);
 ```
 */
export function destinationsLine(
  {
    entryId,
    destinations,
    attempt,
    wallClock = wallClockNow,
  }: {
    readonly entryId: string;
    readonly destinations: DestinationCheck;
    readonly attempt: number;
    readonly wallClock?: () => Date;
  },
): string {
  /**
   Destinations the source carries.
   */
  const sourceCount = destinations
    .source
    .length;

  /**
   Destinations the page carries.
   */
  const pageCount = destinations
    .page
    .length;

  /**
   Source destinations the page lacks.
   */
  const droppedCount = destinations
    .dropped
    .length;

  /**
   Findings, each set off by a space, empty when there are none.
   */
  const noted = destinations
    .findings
    .map(function spaced(finding,): string {
      return ` ${finding}`;
    },)
    .join('',);

  return `DESTINATIONS ${entryId} ${
    lifecycleStamp({
      attempt,
      wallClock,
    },)
  } source=${String(sourceCount,)} page=${String(pageCount,)} dropped=${String(droppedCount,)}${noted}`;
}

//endregion Destinations line
