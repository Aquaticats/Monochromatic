//region Artifact version 2 consolidation fault
// A SETTLEMENT THE STAGE COULD NOT HAVE WRITTEN, named so a caller can match on it. Among the classes of this package
// that refuse a record handed in, only `ArtifactPreparationMismatchError` (a ledger that does not describe the
// preparation it is filed under) names the same kind of fault: a defect in the record itself. The others name
// something else, and this class does not belong with them: `RoundsNotRecordedError` an earlier shape,
// `OffRosterModelError` an earlier seating that says nothing bad about the record, and
// `UnknownArtifactGenerationError` a generation its key table does not cover.

/**
 Reports a settlement that marks the translate lane winner as shipped and names no reason for it.

 A NAMED CLASS, NOT `nonNullishOrThrow`, because the error rule asks for a class a caller can match on (a plain
 `Error` is matched only by its text) and because the fault is a defect in the record handed in, a settlement the
 stage could not have written, as `ArtifactPreparationMismatchError` names for a ledger that does not describe its
 preparation. `nonNullishOrThrow` would say only that a value was absent; this says which slice, which mark, and what
 the artifact needs the reason for.

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
