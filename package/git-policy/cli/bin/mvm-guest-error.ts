/**
 The one error a guest driver's steps throw for a refusal a person can act on.

 A refusal names the step that refused, the command it ran or the answer the guest gave, and what a person does next. It
 carries no stack of this package's own frames into the log a coordinator reads, because every way a guest driver stops
 is a sentence about one throwaway guest rather than a defect in this package.
 */
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

/**
 Failures of the guest drivers' own steps.
 */
export class GuestDriverError extends Error {
  /**
   Stable class name that tells a refusal of a guest driver's step apart from an unrelated error in a log.
   */
  override name = 'GuestDriverError';
}

/**
 The error to throw for a value caught where a step failed.

 A caught value that is already an error is thrown unchanged, so the class, the cause and the stack the failing program
 wrote reach the caller that reads the log. Anything else is wrapped, because a program that threw a bare string left no
 stack of its own to keep.

 @param caught - what the failed step handed back

 @returns the error to throw, carrying the caught value as its cause when it had to be wrapped

 @example
 ```ts
 throw refusalOf({ caught: new Error('mvm exec exited 1') });
 ```
 */
export function refusalOf({ caught, }: { readonly caught: unknown; },): Error {
  if (Error.isError(caught,))
    return caught;
  return new GuestDriverError(
    caughtValueText(caught,),
    { cause: caught, },
  );
}
