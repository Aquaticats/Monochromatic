export { caughtValueText as caughtErrorMessage, } from '@monochromatic-dev/caught-value-core/ts';

export { caughtErrorHasCode, } from './error.ts';

//region Error type and unknown-error helpers

/**
 Error thrown when manifest cache persistence cannot safely proceed.
 
 @example
 ```ts
 throw new StalenessManifestPersistenceError('Invalid staleness manifest');
 ```
 */
export class StalenessManifestPersistenceError extends Error {
  /**
   Creates a manifest persistence error.
   
   @param message - Human-readable persistence failure.
   
   @param options - Optional error cause.
   
   @mutates options through super options.cause getter or proxy effects
   
   @example
   ```ts
   new StalenessManifestPersistenceError('Invalid staleness manifest');
   ```
   */
  constructor(
    message: string,
    options?: ErrorOptions,
  ) {
    super(
      message,
      options,
    );
    this.name = 'StalenessManifestPersistenceError';
  }
}

//endregion Error type and unknown-error helpers
