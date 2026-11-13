/**
 When a run may end as one that came back: Pi has rested, nothing it settled on is unanswered, and no message the driver sent is
 owed a reply.

 Kept apart from the watch that calls it, because the rest a run has had and the messages still owed a reply are two questions a
 reader asks separately, and the watch file is at the length one file is allowed.

 @module
 */

import type { WatchRequest, } from './watch-request.ts';
import { answeredItsWake, } from './kept-while-work-runs.ts';

/**
 Milliseconds in a second, so a limit the options state in seconds is compared against the record's own clock.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 Tells whether Pi has rested for long enough that the run is over.

 @param watch - watch request

 @returns whether the idle wait has passed
 */
function idleLongEnough(watch: WatchRequest,): boolean {
  return (!watch.record
    .isRunning())
    && (watch.record
      .settledCount()
      >= 1)
    && (watch.record
      .silentMilliseconds()
      > (watch.options
        .idleSeconds
        * MILLISECONDS_PER_SECOND));
}

/**
 Tells whether the run may end as one that came back with nothing to report: Pi has rested long enough, no failure it settled on
 is still unanswered, and no message the driver sent it is still owed a reply.

 A failure stays until a reply succeeds, so a prompt Pi only handled leaves the run ending on that failure rather than making it
 look like a run that finished its task with nothing to say. A run the driver sent back to work it left is up until the reply to
 that message settles: the message went a moment ago, and the silence before it is not the silence of a run with nothing left.

 @param watch - watch request

 @param wokenAtSettle - how many settles Pi had when the driver last woke the agent, from the work it left running

 @returns whether nothing stands between the run and its end but the rest it has had

 @example
 ```ts
 const ready = mayEndIdle({ watch, wokenAtSettle: undefined, },);
 ```
 */
export function mayEndIdle({
  watch,
  wokenAtSettle,
}: {
  readonly watch: WatchRequest;
  readonly wokenAtSettle?: number
},): boolean {
  /**
   The failure the run settled on, when nothing after it recovered.
   */
  const failure = watch.record
    .unrecoveredModelError();
  return (!failure.failed) && idleLongEnough(watch,)
    && answeredItsWake({
      ...(wokenAtSettle === undefined ? {} : { wokenAtSettle, }),
      settledNow: watch.record
        .settledCount(),
    },);
}
