//region Unprepared slice
// The refusal for an outcome naming a slice the preparation never produced.
//
// ONE CLASS FOR BOTH READERS. The refine phase used to tolerate the state, read
// an empty original and the outcome's own text for the archive wording, and buy
// rewriter calls against them, after which `repair-refine-step.ts` refused the
// same outcome with a bare Error; the phase now refuses before any call, and
// the step's refusal after it is the same fault said the same way.

/**
 Raised when an outcome names a slice this preparation never produced.

 @example
 ```ts
 throw new UnpreparedSliceError({ sliceIndex: 7, },);
 ```
 */
export class UnpreparedSliceError extends Error {
  /**
   Declares this message safe to print whole at a boundary: it carries one
   slice index and nothing from any text.
   */
  readonly messageNamesOnly: true = true;

  /**
   @param sliceIndex - slice the outcome names
   */
  constructor({ sliceIndex, }: { readonly sliceIndex: number; },) {
    super(
      `slice ${
        String(sliceIndex,)
      } is not one this preparation produced, so the outcome naming it has no original and no archive wording to be held to`,
    );
    this.name = 'UnpreparedSliceError';
  }
}

//endregion Unprepared slice
