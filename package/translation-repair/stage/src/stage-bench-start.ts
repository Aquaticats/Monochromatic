//region Stage bench start
// A PINNED START OF THE BENCH IS A SEAT OF IT. `StageSettings.editorBenchStart` and the `benchStart` a gather takes name the seat the
// bench's rotation starts at (`rotatedBench`, `stage-fanout-window.ts`), so that the seats a fixture seats move with no sheet
// text. Handed to `Array.prototype.slice` unchecked, a negative number counted from the end, a NaN or a number at or past the size
// of the bench left the bench unrotated and a fraction was truncated, and a fixture that pinned a seat got another seating than it
// named with nothing to say so. The one place the value and the size of the bench are both in hand is the rotation, so it asks.

/**
 Thrown when a pinned start of the bench is not a whole number from zero to below the size of the bench.

 @example
 ```ts
 throw new StageBenchStartError({ start: -1, benchSize: 7, },);
 ```
 */
export class StageBenchStartError extends Error {
  /**
   Declares this message safe to forward: it names a number and a count of seats, and never anything a model wrote.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the report from the start that was pinned and the size of the bench it was pinned on.

   @param start - the pinned start, as it was given

   @param benchSize - seats on the bench the rotation was asked for
   */
  constructor(
    {
      start: pinnedStart,
      benchSize,
    }: {
      readonly start: number;
      readonly benchSize: number;
    },
  ) {
    /**
     The last seat a start may name.
     */
    const lastSeat = benchSize - 1;
    super(
      `the pinned start of the bench is ${String(pinnedStart,)}, which is not a whole number from 0 to ${String(lastSeat,)}; `
        + `the size of the bench is ${String(benchSize,)}, and a start that is negative, not a whole number or not below that size `
        + `would seat the rotation somewhere other than the seat the caller named`,
    );
    this.name = 'StageBenchStartError';
  }
}

/**
 Refuses a pinned start that names no seat of the bench.

 @param start - the pinned start

 @param benchSize - seats on the bench

 @throws {@link StageBenchStartError} when the start is not a whole number from zero to below the size of the bench

 @example
 ```ts
 assertBenchStart({ start: 3, benchSize: 7, },);
 ```
 */
export function assertBenchStart(
  {
    start,
    benchSize,
  }: {
    readonly start: number;
    readonly benchSize: number;
  },
): void {
  /**
   Whether the start is a seat of the bench.
   */
  const isSeat = Number.isInteger(start,)
    && (start >= 0)
    && (start < benchSize);
  if (!isSeat) {
    throw new StageBenchStartError({
      start,
      benchSize,
    },);
  }
}

//endregion Stage bench start
