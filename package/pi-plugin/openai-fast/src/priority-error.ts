/**
 Priority request validation diagnostics without request data. @module
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';

//region Diagnostics: keep rejected payloads and credentials out of errors and logs.

/**
 Module logger for priority request validation.
 */
const l = tagged({ tag: 'openai-fast/priority-error', },);

/**
 Priority request cannot be prepared without changing its required semantics.
 */
export class PriorityRequestError extends Error {
  /**
   Construct a diagnostic describing the rejected operation, not its payload.

   @param message - actionable description without credentials or request content

   @example
   ```ts
   throw new PriorityRequestError({ message: 'Priority request payload must be an object.' });
   ```
   */
  constructor({ message, }: { readonly message: string; },) {
    super(message,);
    this.name = 'PriorityRequestError';
    /**
     Constructor logger records only the validation boundary.
     */
    const innerL = tagged({
      tag: PriorityRequestError.name,
      l,
    },);
    innerL.debug('created priority request diagnostic',);
  }
}

//endregion Diagnostics
