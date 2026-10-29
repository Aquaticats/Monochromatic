import { setTimeout as sleepFor, } from 'node:timers/promises';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  isWholeNumberText,
  monotonicMs,
  WHOLE_NUMBER_RULE,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Request pace
// A sliding-window pacer that lets at most `perWindow` requests START in any
// `windowMs`, queueing the rest, so a provider's request-rate limit is met by
// waiting rather than by being refused.
//
// WHY. Hyper limits this account to 1,000 requests in a rolling hour (the
// owner's figure, recorded in `hyper-client.ts`) and refuses the excess with
// HTTP 429 ("You've hit your hourly rate limit. Please try again in 1s").
// Measured on XIEPT2, 2026-09-03 00:55 to 01:39 UTC, Hyper the only provider
// left: 612 successful requests in the whole run (SPEND lines), refusals from
// 01:00:33 UTC once the hour's thousand was spent (257 of them in this run, the
// rest in the run before it), then a trickle of 100 to 140 successes per ten
// minutes, which is the rate at which requests from an hour earlier left the
// window; 6,441 retry attempts and 1,487 calls refused five times over, every
// seat losing about half its calls, and the run ended provider-unavailable. A
// provider-wide hold on the refusal herded every call into one burst; no hold
// at all saturated the window outright. Neither is pacing. This is.
//
// A FIRST READING OF THE SAME LOG counted "completed streams" and reached
// 9,628 successes in 43 minutes; a refused exchange completes its stream too,
// so that count was successes plus refusals. The SPEND lines are the count.
//
// CLASS ONE HUNDRED FORTY-NINE (hulicaijia30, 2026-09-26). The pass spent the
// hour's thousand starts between 07:31 and 08:04 UTC, and at 08:04:20 UTC one take
// slept 1,640,012 ms until the first start left the window. Takes then ran
// through one promise chain and the sleep did not hear an abort, so every Hyper
// call behind it waited too, and the 360 s call deadlines that fired at about
// 08:10 UTC were only read at 08:31:40 UTC, when the sleep ended: eight translate
// slices stalled 27 to 34 minutes and the entry took 100.6 minutes. Each take
// now RESERVES its start at once, in arrival order, and waits for it on its
// own abortable timer: a caller that gives up leaves at once and hands its
// place back. `waitMs` says how long a take would wait, which the router reads
// to send the call to a provider that can start it now.
//
// LIMITS. The window starts empty: requests an earlier process made in the
// last hour are not in it, so a launch right after a heavy run can be refused
// until they leave the window; the retry ladder honours the refusal's own
// wait for that. A place handed back does not move later reservations
// earlier; they start on their own time, which never exceeds the rate.

/**
 Logger root for the pacer.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Length of Hyper's window: a rolling hour, in milliseconds.
 */
export const HYPER_PACE_WINDOW_MS = 3_600_000;

/**
 Environment variable overriding how many Hyper requests may start in any
 rolling hour.
 */
export const HYPER_REQUESTS_PER_HOUR_VAR = 'TRANSLATION_REPAIR_HYPER_REQUESTS_PER_HOUR';

/**
 Requests in any rolling hour the pacer allows Hyper by default: the account's
 limit as the owner stated it and as the 2026-09-03 refusals bear out.
 */
export const HYPER_REQUESTS_PER_HOUR = 1_000;

/**
 What a pacer offers: a turn to start one request, granted when the window
 has room, and a reading of how long that turn is away.

 @example
 ```ts
 const pace: RequestPace = createRequestPace({ perWindow: 1_000, windowMs: 3_600_000, },);
 await pace.take({ signal, },);
 ```
 */
export type RequestPace = {
  /**
   Reserves one start inside the window and waits until it comes.

   @throws The signal's reason when the caller aborts while waiting
   */
  readonly take: (input: { readonly signal: AbortSignal; }) => Promise<void>;

  /**
   How many starts the window holds that have already happened.
   */
  readonly inWindow: () => number;

  /**
   How long a take made now would wait, zero while the window has room.
   */
  readonly waitMs: () => number;
};

/**
 Sleeps the given milliseconds unless the caller aborts first.

 @param ms - how long to sleep

 @param signal - caller's abort, which ends the sleep at once

 @throws The timer's abort error when the caller gives up

 @example
 ```ts
 await abortableSleep({ ms: 1_000, signal, },);
 ```
 */
async function abortableSleep(
  {
    ms,
    signal,
  }: {
    readonly ms: number;
    readonly signal: AbortSignal;
  },
): Promise<void> {
  await sleepFor(
    ms,
    undefined,
    { signal, },
  );
}

/**
 Builds a pacer over a sliding window.

 A TAKE RESERVES ITS START WITHOUT AWAITING ANYTHING, so two calls arriving
 together cannot both read a window with one free place and both start, and
 no caller ever waits behind another caller's wait.

 @param perWindow - starts allowed in any window, a whole number of one or
 more: a window holds whole requests, and the start `perWindow` places back is
 read by index, which a fraction would miss

 @param windowMs - window length

 @param now - clock, injectable for tests; `monotonicMs` by default, since a
 window measured on the system clock moved an hour when it was set
 (ledger B78)

 @param wait - sleeper that must end when the signal aborts, injectable for
 tests

 @returns Pacer

 @throws {@link RangeError} when `perWindow` is not a whole number of one or
 more, whatever the caller built it from

 @example
 ```ts
 const pace = createRequestPace({ perWindow: 1_000, windowMs: 3_600_000, },);
 ```
 */
export function createRequestPace(
  {
    perWindow,
    windowMs,
    now = monotonicMs,
    wait = abortableSleep,
  }: {
    readonly perWindow: number;
    readonly windowMs: number;
    readonly now?: () => number;
    readonly wait?: (
      args: {
        readonly ms: number;
        readonly signal: AbortSignal;
      },
    ) => Promise<void>;
  },
): RequestPace {
  // A PLACE COUNT THE WINDOW CANNOT HOLD IS REFUSED HERE, so a pacer built by
  // hand cannot reach `nextFreeAt` with a fractional index: 1.5 places read
  // `starts[1.5]` as missing and made every take wait a whole window.
  if ((!Number.isSafeInteger(perWindow,)) || (perWindow < 1))
    throw new RangeError(`perWindow must be a whole number of one or more, and ${String(perWindow,)} is not`,);

  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: createRequestPace.name,
    l,
  },);
  /**
   Start times inside the window, past and reserved, oldest first.
   */
  const starts: number[] = [];

  /**
   Drops starts that have left the window.
   */
  function prune(): void {
    /**
     Oldest moment still inside the window.
     */
    const edge = now() - windowMs;
    while ((starts.length > 0) && (nonNullishOrThrow(starts[0],) <= edge))
      starts.shift();
  }

  /**
   When the next start may happen: now while the window has room, else when
   the start `perWindow` places back leaves the window.

   @returns Moment of the next free place
   */
  function nextFreeAt(): number {
    prune();
    if (starts.length < perWindow)
      return now();
    return Math.max(
      now(),
      // A PLAIN INDEX READ: `perWindow` is a whole number of one or more and
      // the window holds at least that many starts, so the index is in range.
      nonNullishOrThrow(starts[starts.length - perWindow],) + windowMs,
    );
  }

  /**
   Hands a reserved place back when its caller gave up.

   @param at - place's start time
   */
  function release(at: number,): void {
    /**
     Where the place sits; reservations at one moment are interchangeable.
     */
    const index = starts.lastIndexOf(at,);
    if (index !== (-1))
      starts.splice(
        index,
        1,
      );
  }

  return {
    take: async function take({ signal, },): Promise<void> {
      signal.throwIfAborted();
      /**
       Start this take reserves.
       */
      const at = nextFreeAt();
      /**
       Starts the window holds, reservations included, before this one.
       */
      const held = starts.length;
      starts.push(at,);
      /**
       How long until the reserved start.
       */
      const ms = at - now();
      if (ms <= 0)
        return;
      rl.info(`window full (${String(held,)} starts in ${String(windowMs,)}ms); waiting ${String(ms,)}ms`,);
      // SLEPT AGAIN FOR WHAT IS LEFT whenever a sleep ends before the reserved
      // start: a timer ends by libuv's coarser loop clock, which can fall a
      // millisecond short of this one, and a take that returned there let its
      // request start before its place opened (ledger B78).
      for (let left = ms; left > 0; left = at - now()) {
        try {
          // oxlint-disable-next-line no-await-in-loop -- the loop IS the wait: each sleep is for what the last one left
          await wait({
            ms: left,
            signal,
          },);
        } catch (error) {
          release(at,);
          signal.throwIfAborted();
          throw error;
        }
        if (signal.aborted) {
          release(at,);
          signal.throwIfAborted();
        }
      }
    },
    inWindow: function inWindow(): number {
      prune();
      /**
       Moment the count is read at.
       */
      const moment = now();
      return starts
        .filter(function started(start,): boolean {
          return start <= moment;
        },)
        .length;
    },
    waitMs: function waitMs(): number {
      return Math.max(
        0,
        nextFreeAt() - now(),
      );
    },
  };
}

/**
 Requests per rolling hour the environment asks for, or the default.

 @param env - environment to read

 @returns Whole number of one or more from the variable, or the default when
 it is unset, empty or blank

 @throws {@link StatedRefusalError} when the variable is set and is not a
 whole number of one or more, as every other dial refuses (ledger D14): a
 mistyped rate that fell back to the account limit ran a launch at a pace
 nobody asked for, and a fraction such as 1.5 is a rate the pacer cannot keep
 since its window holds whole requests

 @example
 ```ts
 const perHour = hyperRequestsPerHour({ env: process.env, },);
 ```
 */
export function hyperRequestsPerHour(
  { env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },
): number {
  /**
   Raw value when set.
   */
  const raw = env[HYPER_REQUESTS_PER_HOUR_VAR] ?? '';
  // BLANKS SAY NOTHING, as for every numeric dial in the package: `Number`
  // read them as a rate of zero, so this refused a variable set to spaces
  // that the grace, cap, spend ceiling and credit dials read as unset
  // (ledger B73).
  if (raw.trim() === '')
    return HYPER_REQUESTS_PER_HOUR;
  /**
   Parsed value. `Number` rather than `parseFloat`, which would read a leading
   number out of a typo such as `300/h`.
   */
  const parsed = Number(raw,);
  // A WHOLE NUMBER IN DIGITS, as an operator writes a count of requests:
  // `Number` also read `0x10`, `1e1`, `+15`, ` 15` and `.5` as rates nobody
  // wrote that way (ledger B73), and a fraction is no rate a window of whole
  // requests can keep.
  if ((!isWholeNumberText({ text: raw, },)) || (parsed < 1)) {
    throw new StatedRefusalError({
      says: `${HYPER_REQUESTS_PER_HOUR_VAR} must be one or more requests per hour, as ${WHOLE_NUMBER_RULE}, and `
        + `${JSON.stringify(raw,)} is not; leave it unset for the account limit of ${String(HYPER_REQUESTS_PER_HOUR,)}`,
    },);
  }
  return parsed;
}

//endregion Request pace
