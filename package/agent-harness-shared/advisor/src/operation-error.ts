/**
 Failed operations preserve their accounting snapshot alongside a readable diagnostic. @module
 */
import type { AdvisorOperationSnapshot, } from './operation-types.ts';

/**
 Error carrying locally finalized operation evidence for tool and slash-command persistence.
 
 @example
 ```ts
 throw new AdvisorOperationError(snapshot);
 ```
 */
export class AdvisorOperationError extends Error {
  /**
   Sealed operation snapshot, including available failed-attempt usage.
   */
  readonly operation: AdvisorOperationSnapshot;

  /**
   Build a diagnostic without pretending missing provider usage is zero billing.
   
   @param operation - locally finalized operation
   
   @example
   ```ts
   new AdvisorOperationError(operation);
   ```
   */
  constructor(operation: AdvisorOperationSnapshot,) {
    /**
     Attempt summaries survive hosts which discard thrown structured details.
     */
    const attempts = operation.attempts
      .map(function summary(attempt,): string {
        return `${attempt.model} attempt ${String(attempt.attempt,)}: ${attempt.diagnostic ?? attempt.state}`;
      },)
      .join('; ',);
    super(`advisor: ${operation.end === 'caller' ? 'call cancelled' : operation.end === 'deadline' ? 'operation deadline expired' : 'no usable review'}${attempts === '' ? '' : `; ${attempts}`}`,);
    this.name = 'AdvisorOperationError';
    this.operation = operation;
  }
}
