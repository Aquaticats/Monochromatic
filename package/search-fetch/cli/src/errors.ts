/**
 Error classes carrying search-fetch CLI failures.

 Every failure this CLI maps to a usage exit travels as one of these classes
 or as the shared core's `BlockedUrlError`, so the top-level handler classifies
 by error class and never by error code.

 @module
 */

//region Public API

/**
 Thrown when the command line does not fit the documented grammar.

 Reaches the caller as usage text on stderr and exit status 2.
 */
export class SearchFetchCliUsageError extends Error {
  /**
   Build a usage failure naming the refused input.

   @param message - refused input and the grammar that refuses it
   */
  constructor({ message, }: { readonly message: string; },) {
    super(message,);
    this.name = 'SearchFetchCliUsageError';
  }
}

/**
 Thrown when the search-fetch configuration cannot be loaded.

 Reaches the caller as a diagnostic on stderr and exit status 2.
 */
export class SearchFetchCliConfigurationError extends Error {
  /**
   Build a configuration failure naming the load that failed.

   @param message - load step and the observed failure

   @param cause - underlying load failure, when one was caught
   */
  constructor(
    {
      message,
      cause,
    }: {
      readonly message: string;
      readonly cause?: unknown;
    },
  ) {
    super(
      message,
      cause === undefined ? undefined : { cause, },
    );
    this.name = 'SearchFetchCliConfigurationError';
  }
}

//endregion Public API
