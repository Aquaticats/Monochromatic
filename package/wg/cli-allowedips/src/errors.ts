/**
 Error raised when command-line arguments do not satisfy the command contract.
 
 @example
 ```ts
 throw new CliUsageError('Missing required option: --allowed');
 ```
 */
export class CliUsageError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'CliUsageError';
}
