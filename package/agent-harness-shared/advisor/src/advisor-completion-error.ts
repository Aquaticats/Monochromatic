/**
 Advisor completion error type.
 
 @module
 */

/**
 Advisor completion failure carrying a user-visible provider diagnostic.
 
 @example
 ```typescript
 throw new AdvisorCompletionError('advisor: provider call failed');
 ```
 */
export class AdvisorCompletionError extends Error {
  /**
   Build an Advisor completion failure.
   
   @param message - actionable failure diagnostic
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'AdvisorCompletionError';
  }
}
