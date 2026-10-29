/**
 Error raised when input text cannot represent an address set accepted by the CLI.
 
 @example
 ```ts
 throw new InputValidationError('Allowed input must contain at least one address.');
 ```
 */
export class InputValidationError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'InputValidationError';
}
