/**
 Structured input validation error.
 
 @module
 */

/**
 Reports an unsupported envelope or malformed finding before policy work.
 
 @example
 ```ts
 throw new InputValidationError('input must be JSON');
 ```
 */
export class InputValidationError extends Error {
  /**
   Creates a structured-input validation failure.
   
   @param message - User-facing evidence identifying rejected input.
   
   @example
   ```ts
   const error = new InputValidationError('comment path is missing');
   ```
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'InputValidationError';
  }
}
