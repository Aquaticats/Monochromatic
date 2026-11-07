/**
 Errors of the move tools.

 @module
 */

/**
 A step that stops the run on purpose, with every earlier step recorded. The message names the cause and what to do.
 */
export class MoveStopError extends Error {
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
