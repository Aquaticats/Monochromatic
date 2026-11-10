/**
 A run that could not be prepared, so no unit of it was started.

 @module
 */

/**
 The run could not be prepared.
 */
export class PiRunNotStartedError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'PiRunNotStartedError';
}
