/**
 Errors of the move tools.

 @module
 */

/**
 A step that stops the run on purpose, with every earlier step recorded. The message names the cause and what to do.
 */
export class MoveStopError extends Error {
  /**
   Creates the stop.

   @param message - the cause and what to do
   
   @param options - the error options, for a cause
   */
  public constructor(
    message: string,
    options?: ErrorOptions,
  ) {
    super(
      message,
      options,
    );
    this.name = 'MoveStopError';
  }
}
