//region Naturalness completeness refusal

/**
 Refuses slice whose bounded final polish did not meet absolute quality floor.

 Message names only slice position. Model findings and corpus wording remain
 in internal settlement and never cross user-facing error boundary.

 @example
 ```ts
 throw new NaturalnessCompletenessError({ sliceIndex: 2, });
 ```
 */
export class NaturalnessCompletenessError extends Error {
  /**
   Declares message safe to forward because it names only slice index.
   */
  readonly messageNamesOnly: true = true;

  /**
   Prepared slice refused.
   */
  readonly sliceIndex: number;

  /**
   @param sliceIndex - prepared slice without absolute naturalness approval

   @param cause - what made the review unreadable, where the refusal stands for an unreadable review; the message still
   names only the slice
   */
  public constructor(
    {
      sliceIndex,
      cause,
    }: {
      readonly sliceIndex: number;
      readonly cause?: unknown;
    },
  ) {
    super(
      `slice ${String(sliceIndex,)} did not meet absolute naturalness floor`,
      (cause === undefined) ? {} : { cause, },
    );
    this.name = 'NaturalnessCompletenessError';
    this.sliceIndex = sliceIndex;
  }
}

//endregion Naturalness completeness refusal
