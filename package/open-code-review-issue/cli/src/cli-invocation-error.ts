/**
 Command invocation error vocabulary.
 
 @module
 */

/**
 Reports command invocation misuse mapped to exit status two.
 */
export class CliInvocationError extends Error {
  /**
   Creates invocation failure.
   
   @param message - User-facing flag or argument diagnostic.
   
   @example
   ```ts
   const error = new CliInvocationError('exactly one mode is required');
   ```
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'CliInvocationError';
  }
}

/**
 Identifies omitted positional OCR input requiring generation guidance.
 */
export class MissingCliInputError extends CliInvocationError {
  /**
   Creates missing-input invocation failure.
   
   @example
   ```ts
   const error = new MissingCliInputError();
   ```
   */
  public constructor() {
    super('every mode requires one positional input',);
    this.name = 'MissingCliInputError';
  }
}
