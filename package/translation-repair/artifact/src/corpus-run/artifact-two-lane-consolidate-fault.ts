//region Artifact version 2 consolidation fault
// A SETTLEMENT THE STAGE COULD NOT HAVE WRITTEN, named so a caller can match on it. This sits with the other classes of
// this package that refuse a record no stage of this repository builds (`UnknownArtifactGenerationError`,
// `ArtifactPreparationMismatchError`, `RoundsNotRecordedError`, `OffRosterModelError`): the fault is in the record
// handed in, and nothing a reader of the artifact could do repairs it.

/**
 Reports a settlement that marks the translate lane winner as shipped and names no reason for it.

 A NAMED CLASS, NOT `nonNullishOrThrow`, because the error rule asks for a class a caller can match on (a plain
 `Error` is matched only by its text) and because the fault is a record the stage could not have written, which is the
 family the classes above belong to. `nonNullishOrThrow` would say only that a value was absent; this says which
 slice, which mark, and what the artifact needs the reason for.

 @example
 ```ts
 throw new TranslateWinnerReasonMissingError({ sliceIndex: 7, },);
 ```
 */
export class TranslateWinnerReasonMissingError extends Error {
  /**
   Declares this message safe to forward: a slice index and a sentence written here.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names the slice and says why the record cannot be written.

   @param sliceIndex - slice whose settlement carried the mark without a reason

   @example
   ```ts
   new TranslateWinnerReasonMissingError({ sliceIndex: 7, },);
   ```
   */
  public constructor({ sliceIndex, }: { readonly sliceIndex: number; },) {
    super(
      `slice ${String(sliceIndex,)}: the settlement marks the translate lane winner as shipped but names no reason, `
        + 'and the artifact records why a winner shipped (a standing the rule refused, or an unendorsed standing the '
        + 'tense detector flagged); the stage sets the reason with the mark, so a settlement without one did not come '
        + 'from the stage',
    );
    this.name = 'TranslateWinnerReasonMissingError';
  }
}

//endregion Artifact version 2 consolidation fault
