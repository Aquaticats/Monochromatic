/**
 The notice a filling owes when it has started nothing for longer than a bound while entries stand and places are free.

 A filling that walks one unstartable entry every round says so on its output, and its unit's journal holds it, but nobody reads a
 journal to find out that no agent is running. This puts the same fact on the desktop, where the machine's owner sees it.

 The bound is twelve rounds of the filling's own 150-second round. The record it is set from: the filling unit's journal of
 2026-11-14 holds one refusal every about 153 seconds from 04:00:14 to 08:40:02, and the pass that started the next entry did so
 within two minutes of the blocked entry being taken out by hand. Nothing legitimate goes twelve rounds without a start while places
 are free, so the notice speaks after half an hour of that and not before.

 @module
 */

import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import type {
  DesktopNotice,
  DesktopNoticeOutcome,
} from './desktop-notice.ts';

/**
 Seconds in one round of the filling, which is the wait the coordinator's own loop takes between passes.
 */
export const FILLING_ROUND_SECONDS = 150;

/**
 Rounds of the filling that may pass without a start before the notice, which is the twelve the record above was read against.
 */
const ROUNDS_WITHOUT_A_START = 12;

/**
 How long a filling may start nothing while entries stand and places are free, in seconds: twelve rounds.
 */
export const STALLED_FILLING_BOUND_SECONDS: number = ROUNDS_WITHOUT_A_START * FILLING_ROUND_SECONDS;

/**
 How long that bound is in milliseconds.
 */
export const STALLED_FILLING_BOUND_MILLISECONDS: number = STALLED_FILLING_BOUND_SECONDS * MILLISECONDS_PER_SECOND;

/**
 What one look for a stalled filling reads and where its notice goes.
 */
export type StalledFillingLook = {
  /**
   When the filling last started an entry, as milliseconds since the epoch.
   */
  readonly lastStartedAt: number;

  /**
   When it last said so on the desktop, which keeps the notice to one a bound rather than one a round.
   */
  readonly lastNoticedAt?: number;

  /**
   When this look reads the clock, as milliseconds since the epoch.
   */
  readonly now: number;

  /**
   How many entries stand in the queue.
   */
  readonly entriesStanding: number;

  /**
   How many places the level leaves free.
   */
  readonly placesFree: number;

  /**
   The queue's path, which the notice names.
   */
  readonly queueFile: string;

  /**
   Why the last pass stopped with places free, when it did.
   */
  readonly stoppedBecause?: string;

  /**
   Shows one notification.
   */
  readonly notify: (notice: DesktopNotice) => Promise<DesktopNoticeOutcome>;

  /**
   Says one line on the filling's own output.
   */
  readonly report: (line: string) => void;
};

/**
 Builds the notice a stalled filling owes.

 @param look - what the look read

 @returns the title and body to show

 @example
 ```ts
 const notice = stalledFillingNoticeOf({ look });
 ```
 */
export function stalledFillingNoticeOf({ look, }: { readonly look: StalledFillingLook },): DesktopNotice {
  /**
   When the filling last started something, as ISO 8601.
   */
  const lastStarted = new Date(look.lastStartedAt,).toISOString();
  return {
    title: 'Pi runs are not being started',
    body: `${String(look.entriesStanding,)} entries stand and ${String(look.placesFree,)} places are free, and nothing started since ${lastStarted}. Queue: ${look.queueFile}.${(look.stoppedBecause === undefined) ? '' : ` The last pass stopped because ${look.stoppedBecause}.`}`,
  };
}

/**
 Says on the desktop that the filling has stalled, once a bound.

 @param look - what the look read

 @returns whether a notice was owed, and when one was shown

 @example
 ```ts
 const { noticed } = await noticeIfTheFillingStalls({ look });
 ```
 */
export async function noticeIfTheFillingStalls({ look, }: { readonly look: StalledFillingLook },): Promise<{
  readonly noticed: boolean;
  readonly lastNoticedAt?: number;
}> {
  if ((look.entriesStanding <= 0) || (look.placesFree <= 0)) {
    return { noticed: false, };
  }
  if ((look.now - look.lastStartedAt) < STALLED_FILLING_BOUND_MILLISECONDS) {
    return { noticed: false, };
  }
  if ((look.lastNoticedAt !== undefined) && ((look.now - look.lastNoticedAt) < STALLED_FILLING_BOUND_MILLISECONDS)) {
    return {
      noticed: false,
      lastNoticedAt: look.lastNoticedAt,
    };
  }
  /**
   What showing the notice answered.
   */
  const shown = await look.notify(stalledFillingNoticeOf({ look, },),);
  look.report(
    `${new Date(look.now,).toISOString()} nothing has started for ${String(Math.floor((look.now - look.lastStartedAt) / MILLISECONDS_PER_SECOND,),)} s while ${String(look.entriesStanding,)} entries stand and ${String(look.placesFree,)} places are free; ${shown.shown ? 'a desktop notification was shown' : `the desktop notification was not shown: ${shown.problem ?? 'no reason given'}`
       }\n`,
  );
  return {
    noticed: true,
    lastNoticedAt: look.now,
  };
}
