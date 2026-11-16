/**
 The last step of a start: what a run left, its summary, and the status the launcher exits with. Split out of the launcher so that
 file stays one screen long.

 @module
 */

import {
  finishTheRun,
  type FinishRequest,
} from './finish-a-run.ts';

/**
 What finishing one start needs, which is everything but where a line is printed.
 */
export type FinishOfAStartRequest = Omit<FinishRequest, 'report'>;

/**
 Finishes one start and answers the status the launcher exits with.

 @param request - the run, its folders, what the wait answered, and the environment

 @returns the status this command exits with

 @example
 ```ts
 const status = await finishThisStart({ ...request, },);
 ```
 */
export async function finishThisStart(request: FinishOfAStartRequest,): Promise<number> {
  /**
   What finishing the run answered.
   */
  const finished = await finishTheRun({
    ...request,
    report: console.log,
  },);
  return finished.status;
}
