/**
 The stops of the guided runner: the exit statuses and the error that carries one.

 @module
 */

/**
 Exit status when an assertion was missing or an answer was no.
 */
export const EXIT_STOPPED_AT_ANSWER = 2;

/**
 Exit status when the window ran past its stated maximum.
 */
export const EXIT_WINDOW_EXCEEDED = 3;

/**
 Exit status when a stage failed or its check failed.
 */
export const EXIT_STAGE_FAILED = 1;

/**
 Exit status when a start-up condition is not met.
 */
export const EXIT_PRECONDITION = 64;

/**
 What a stop carries: the exit status, the headline and the failed stage's rollback text.
 */
export type GuidedStopDetails = {
  readonly status: number;
  readonly message: string;
  readonly rollback?: string;
};

/**
 A stop with a status and a headline; the runner prints the progress and exits.
 */
export class GuidedStopError extends Error {
  /**
   The exit status of the stop.
   */
  public readonly status: number;

  /**
   The rollback of the stage that failed, or an empty string.
   */
  public readonly rollback: string;

  /**
   Creates the stop.

   @param status - the exit status
   
   @param message - the headline
   
   @param rollback - the rollback of the failed stage
   */
  public constructor({
    status,
    message,
    rollback = '',
  }: GuidedStopDetails,) {
    super(message,);
    this.name = 'GuidedStopError';
    this.status = status;
    this.rollback = rollback;
  }
}
