//region Consolidation ledger gap
// The refusal for a contested slice the repair ledger never wrote, an invariant
// of the two-lane driver rather than a state a run can reach on its own.

/**
 Raised when a slice the contest names has no repair-ledger row.

 @example
 ```ts
 throw new ConsolidationLedgerGapError({ sliceIndex: 3, },);
 ```
 */
export class ConsolidationLedgerGapError extends Error {
  /**
   Declares this message safe to print whole at a boundary: one slice index
   and nothing from any text.
   */
  readonly messageNamesOnly: true = true;

  /**
   @param sliceIndex - slice the contest names
   */
  constructor({ sliceIndex, }: { readonly sliceIndex: number; },) {
    super(
      `consolidation: slice ${String(sliceIndex,)} was contested and does not appear in the repair ledger`,
    );
    this.name = 'ConsolidationLedgerGapError';
  }
}

//endregion Consolidation ledger gap
