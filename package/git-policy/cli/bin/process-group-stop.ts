/**
 Stops one started command and everything it started, so a deadline releases whoever waits for it and leaves nothing of
 the command running.

 The command must lead a process group of its own, which is what `detached: true` gives it when it is spawned: a signal
 sent to a group reaches the helpers the command started, and one sent to the command alone leaves them running. Two
 callers rely on that, and both spawn their commands detached for this reason
 (`bin/native-release-host-control-run.ts`, `bin/native-verification-process.ts`).
 */
import type { ChildProcess, } from 'node:child_process';

/**
 Whether a caught value is a system error, which carries the code that says why a call was refused.

 @param value - the caught value

 @returns true for an error with a `code` field

 @example
 ```ts
 isSystemError(new Error('not a system error'));
 ```
 */
function isSystemError(value: unknown,): value is NodeJS.ErrnoException {
  return ((typeof value) === 'object') && (value !== null)
    && ('code' in value);
}

/**
 Stop a command and everything it started.

 @param child - the started command, made a process group leader when it was spawned

 @example
 ```ts
 stopProcessGroup({ child });
 ```
 */
export function stopProcessGroup({ child, }: {
  readonly child: ChildProcess;
},): void {
  /**
   The command's own handle, whose process ID is also its group's because it was spawned detached.
   */
  const { pid, } = child;
  if (pid === undefined) {
    child.kill('SIGKILL',);
    return;
  }
  try {
    process.kill(
      -pid,
      'SIGKILL',
    );
  }
  catch (error) {
    // A group the kernel no longer has answers ESRCH; the command itself may still run, so it is stopped as well.
    if ((!isSystemError(error,)) || (error.code !== 'ESRCH'))
      throw error;
    child.kill('SIGKILL',);
  }
}
