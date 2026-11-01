//region Reviewed-reference diagnostics
// Refusals identify the reviewed input boundary, never reproduce corpus passages.

/**
 Closed verification boundaries of the calibration reference contract.

 @example
 ```ts
 const operation: FidelityReferenceOperation = 'archive';
 ```
 */
export type FidelityReferenceOperation =
  | 'pin'
    | 'source'
    | 'archive'
    | 'edit'
    | 'reference'
    | 'donor'
    | 'damage'
    | 'request';

/**
 Refuses a calibration input that no longer matches its reviewed evidence.

 @example
 ```ts
 throw new FidelityReferenceError({ referenceId: spec.id, operation: 'reference' });
 ```
 */
export class FidelityReferenceError extends Error {
  /**
   Only an input identifier and a closed operation name enter the message.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds an actionable refusal without repeating the disputed text.

   @param referenceId - reviewed reference identifier supplied by the caller

   @param operation - closed boundary requiring verification

   @example
   ```ts
   new FidelityReferenceError({ referenceId: 'portrait-reference', operation: 'source' });
   ```
   */
  public constructor({
    referenceId,
    operation,
  }: {
    readonly referenceId: string;
    readonly operation: FidelityReferenceOperation;
  },) {
    super(`reviewed fidelity reference ${referenceId} failed ${operation} verification;`
      + ' use its pinned inputs and reviewed manifest, or source-review a replacement before calibration.',);
    this.name = 'FidelityReferenceError';
  }
}

//endregion Reviewed-reference diagnostics
