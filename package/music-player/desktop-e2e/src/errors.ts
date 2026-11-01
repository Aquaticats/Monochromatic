/**
 Error classes of the end-to-end runner.
 
 @example
 ```ts
 import { CheckFailureError, } from './errors.ts';
 
 throw new CheckFailureError('the ring is off',);
 ```
 
 @packageDocumentation
 */

/**
 A check did not hold. The runner records it as a FAIL outcome and goes on with the next check.
 
 @example
 ```ts
 throw new CheckFailureError('route Settings',);
 ```
 */
export class CheckFailureError extends Error {
  /**
   Names the error so a recorded outcome shows its kind.
   
   @param message - why the check fails
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'CheckFailureError';
  }
}

/**
 A routing fault that ends the whole run at once: nothing may play anywhere but the private sink.
 
 @example
 ```ts
 throw new SafetyStopError('SAFETY STOP: the app stream links elsewhere',);
 ```
 */
export class SafetyStopError extends Error {
  /**
   Names the error so a run ending shows its kind.
   
   @param message - which stream linked where
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'SafetyStopError';
  }
}

/**
 A condition inside a check that must hold, else the check fails.
 
 @param condition - what must hold
 
 @param message - why the check fails when the condition does not hold
 
 @throws CheckFailureError when the condition does not hold
 
 @example
 ```ts
 expect({ condition: state.route === 'Player', message: `route ${state.route}`, },);
 ```
 */
export function expect(
  {
    condition,
    message,
  }: {
    readonly condition: boolean;
    readonly message: string;
  },
): void {
  if (!condition)
    throw new CheckFailureError(message,);
}
