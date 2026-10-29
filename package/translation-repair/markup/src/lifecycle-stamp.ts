//region Lifecycle stamp
// THE CLOCK TIME AND THE ATTEMPT A LIFECYCLE LINE CARRIES, written in one place so every line the pass prints for an
// entry spells them alike (ledger B960).
//
// WHY A LINE NEEDS BOTH. A reader looking for what happened inside a time window cannot place a `TALLY`, `REATTEMPT` or
// `OVERLAP` line that carries no time, and a hard cap that ends an attempt (not an entry) is invisible when the line
// does not say which attempt it closed. On 2026-10-22 a fired 420-minute cap on `XingZ60` read as a `TALLY` with
// `error=Timeout`, then a `REATTEMPT` line, then a second attempt that began under a fresh cap; none of the three said
// when it was printed or which attempt it belonged to.
//
// THE SPELLING. `at=` is the instant as `Date.prototype.toISOString` writes it, which is how the logger stamps its own
// lines (`iso-stamp-text.ts`), so `isIsoStampText` reads it back by the same rule. `attempt=` is the number the pass
// persists for the entry in its attempts file after it counts the attempt, never a count a line keeps for itself.
// A line of the pass, or of an entry no attempt has begun for, carries `at=` alone (`lifecycleClockStamp`).
//
// THE CLOCK IS INJECTED. The wall clock is read only as a stamp for a moment that outlives the process (ledger B78);
// durations stay on the monotonic clock.

/**
 Reads the wall clock, for the lines that stamp a moment and are given no clock of their own.

 @returns Instant now

 @example
 ```ts
 const printedAt = wallClockNow();
 ```
 */
export function wallClockNow(): Date {
  return new Date();
}

/**
 An attempt number that is not a whole number of at least one.

 @example
 ```ts
 throw new LifecycleAttemptError({ attempt: 0, },);
 ```
 */
export class LifecycleAttemptError extends Error {
  /**
   Declares the message safe because it holds fixed words and the number it names, a count.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names the number a line was asked to carry.

   @param attempt - number that is not an attempt count
   */
  public constructor({ attempt, }: { readonly attempt: number; },) {
    super(
      `a lifecycle line carries the attempt it belongs to as a whole number from 1 (the pass counts an attempt before `
        + `it starts), and ${String(attempt,)} is not one`,
    );
    this.name = 'LifecycleAttemptError';
  }
}

/**
 The clock time alone, for a line that belongs to the pass or to no attempt: the stop notes, the START and DONE lines
 and the INCOMPLETE line of an entry whose page is absent at the pin, which no attempt has begun for.

 A LINE READ BY TIME NEEDS ITS TIME EVEN WHERE NO ATTEMPT EXISTS, and must not print an attempt that does not (ledger
 B960): `lifecycleStamp` refuses an attempt under 1, and the absence of `attempt=` is what tells the page-absent
 INCOMPLETE line from the INCOMPLETE line of an attempt.

 @param wallClock - clock the time is read on; the wall clock unless a test fixes it

 @returns `at=<ISO 8601 UTC with milliseconds>`, with no leading or trailing space

 @example
 ```ts
 const stamp = lifecycleClockStamp({},);
 // => 'at=2026-10-22T08:10:11.123Z'
 ```
 */
export function lifecycleClockStamp(
  { wallClock = wallClockNow, }: { readonly wallClock?: () => Date; },
): string {
  return `at=${
    wallClock()
      .toISOString()
  }`;
}

/**
 The two fields a lifecycle line carries after its opening words: the clock time it was printed at and the attempt it
 belongs to.

 @param attempt - number of the entry's attempt the line belongs to, as the attempts file holds it once the attempt is
 counted

 @param wallClock - clock the time is read on; the wall clock unless a test fixes it

 @returns `at=<ISO 8601 UTC with milliseconds> attempt=<n>`, with no leading or trailing space

 @throws {@link LifecycleAttemptError} when the attempt is not a whole number from 1

 @example
 ```ts
 const stamp = lifecycleStamp({ attempt: 2, },);
 // => 'at=2026-10-22T08:10:11.123Z attempt=2'
 ```
 */
export function lifecycleStamp(
  {
    attempt,
    wallClock = wallClockNow,
  }: {
    readonly attempt: number;
    readonly wallClock?: () => Date;
  },
): string {
  if ((!Number.isSafeInteger(attempt,)) || (attempt < 1))
    throw new LifecycleAttemptError({ attempt, },);
  return `${lifecycleClockStamp({ wallClock, },)} attempt=${String(attempt,)}`;
}

//endregion Lifecycle stamp
