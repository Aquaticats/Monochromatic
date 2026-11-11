/**
 The stop one command's deadline makes: it ends the command and everything the command started, and it releases the
 caller even from a stream a process the kernel cannot end holds open.

 A command that never answers is the one wait a caller cannot bound by itself, so every bound here starts a timer that
 ends the command instead of waiting for it. Both timers exist only while a command has a deadline, and the scope that
 made them ends them, so a command that answered leaves nothing behind.
 */
import type { ChildProcess, } from 'node:child_process';

import { stopProcessGroup, } from './process-group-stop.ts';

/**
 Milliseconds a command stopped at its deadline is given to end before the caller is released anyway.

 `SIGKILL` ends a command and its process group at once, and the kernel wakes every pipe that read its output, so this
 grace is reached only by a process the kernel cannot end: one waiting on a device that stopped answering. The caller is
 released at the grace regardless, because "nothing should ever wait forever or indefinitely".
 */
export const stoppedGraceMilliseconds = 5_000;

/**
 What a caller reads after waiting under a deadline.
 */
export type CommandDeadline = {
  /**
   Whether the deadline stopped the command, which is the refusal the caller reports instead of an exit status.
   */
  readonly stoppedTheCommand: () => boolean;
  /**
   The refusal a stopped command reports, which names the command and the bound it was given.
   */
  readonly refusal: string;
  /**
   Ends the deadline, so a command that ended by itself leaves no timer behind.
   */
  readonly end: () => void;
  /**
   Settles once the grace after the stop has passed, so a caller waiting on something the kernel cannot end is released.
   */
  readonly released: () => Promise<void>;
};

/**
 Stop one command and everything it started when its deadline passes.

 @param child - the started command, made a process group leader when it was spawned, so the stop reaches what it started

 @param deadlineMilliseconds - how long it may run before it and everything it started are stopped

 @param refusal - what the caller reports when the deadline stopped it, which names the bound

 @returns the stop, whether it happened, and the way to end it

 @example
 ```ts
 const deadline = stopAtDeadline({ child, deadlineMilliseconds: 600_000, refusal: 'podman run was still running' });
 ```
 */
export function stopAtDeadline({
  child,
  deadlineMilliseconds,
  refusal,
}: {
  readonly child: ChildProcess;
  readonly deadlineMilliseconds: number;
  readonly refusal: string;
},): CommandDeadline {
  /**
   Whether the deadline has stopped this command, which the caller reads once it is released.
   */
  const state = { stopped: false, };
  /**
   The promise the grace settles, which is what releases a caller still waiting after it.
   */
  const released = Promise.withResolvers<void>();
  /**
   The deadline's timers: the stop, and the grace it rearms once the command is stopped.
   */
  const timers: {
    stop: NodeJS.Timeout;
    grace?: NodeJS.Timeout;
  } = {
    stop: setTimeout(
      function stopTheCommand() {
        state.stopped = true;
        stopProcessGroup({ child, },);
        timers.grace = setTimeout(
          function releaseTheCaller() {
            // A process the kernel cannot end holds the pipes open, so the streams are ended here and the caller's wait
            // returns; its refusal names the deadline either way.
            child
              .stdout
              ?.destroy();
            child
              .stderr
              ?.destroy();
            released.resolve();
          },
          stoppedGraceMilliseconds,
        );
      },
      deadlineMilliseconds,
    ),
  };
  return {
    stoppedTheCommand(): boolean {
      return state.stopped;
    },
    refusal,
    end(): void {
      clearTimeout(timers.stop,);
      if (timers.grace !== undefined)
        clearTimeout(timers.grace,);
    },
    released(): Promise<void> {
      return released.promise;
    },
  };
}
