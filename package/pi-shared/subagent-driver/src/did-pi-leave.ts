/**
 Whether Pi is known to have left: a status or a signal it ended with, never a start the operating system refused and never the
 moment this driver heard of it.

 A run records what it knows of Pi's leaving, so that question is asked in more than one place: by the channel to Pi, by the reason
 a run ends with, and by the ending's own account. One answer keeps them from disagreeing.

 @module
 */

import type { PiExit, } from './driver-types.ts';

/**
 Tells whether Pi's exit was seen, which a status or a signal it left with proves and a refused start does not.

 @param exit - how Pi ended, of any of the shapes {@link PiExit} takes

 @returns whether Pi is known to have gone

 @example
 ```ts
 const left = didPiLeave({ code: 0 });
 ```
 */
export function didPiLeave(exit: PiExit,): boolean {
  return (exit.code !== undefined) || (exit.signal !== undefined);
}
