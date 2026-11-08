/**
 How much time a bound leaves, for waits that must not outlast it.

 @module
 */

import { COMMAND_ANSWER_MILLISECONDS, } from './driver-constants.ts';

/**
 How long is left before a deadline.

 @param deadline - when the wait must be over, as milliseconds since the epoch

 @returns the milliseconds left, and zero once the deadline has passed

 @example
 ```ts
 const left = millisecondsLeftUntil(Date.now() + 500,);
 ```
 */
export function millisecondsLeftUntil(deadline: number,): number {
  return Math.max(
    0,
    deadline - Date.now(),
  );
}

/**
 How long a command can be waited for: its own ordinary bound, or what the run has left, whichever ends first. One silent
 command cannot spend the whole run, and no command can outlast it.

 @param deadline - when the run must be over, as milliseconds since the epoch

 @returns the milliseconds this command may take

 @example
 ```ts
 const bound = answerMillisecondsBefore(deadline,);
 ```
 */
export function answerMillisecondsBefore(deadline: number,): number {
  return Math.min(
    COMMAND_ANSWER_MILLISECONDS,
    millisecondsLeftUntil(deadline,),
  );
}
