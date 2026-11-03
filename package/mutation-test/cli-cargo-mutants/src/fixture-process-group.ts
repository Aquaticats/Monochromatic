import { spawn } from 'node:child_process';
import { Socket } from 'node:net';

import { mentionsProcessLimit } from './process-limits.ts';

/**
 Tell whether an error says that the process or group is not there.

 @param error - caught value

 @returns true for the error code `ESRCH`

 @example
 ```ts
 isNoSuchProcess(new Error('x'));
 ```
 */
function isNoSuchProcess(error: unknown): boolean {
  return Error.isError(error)
    && ('code' in error)
    && (error.code === 'ESRCH');
}

/**
 Kill every process of a process group.

 @param group - the group's ID, which is the process ID of its leader

 @returns true when a group was there to kill, false when it had already ended

 @example
 ```ts
 killGroup(12345);
 ```
 */
export function killGroup(group: number): boolean {
  try {
    process.kill(
      -group,
      'SIGKILL',
    );
    return true;
  }
  catch (error) {
    // A group that is already gone is the wanted state; any other failure is logged, and the caller sees no effect.
    if (!isNoSuchProcess(error))
      console.error(`Could not kill process group ${String(group)}: ${String(error)}`);
    return false;
  }
}

/**
 The watcher for this test process, kept under one key once the first program has started it.
 */
const watchers = new Map<'reaper', ReturnType<typeof spawn>>();

/**
 Tell the watcher about a process group, starting the watcher if this is the first.

 @param line - `add <id>` or `remove <id>`

 @example
 ```ts
 tellReaper('add 12345');
 ```
 */
function tellReaper(line: string): void {
  /**
   The watcher, started now if it is not running.
   */
  const reaper = watchers.get('reaper') ?? spawn(
    process.execPath,
    [new URL(
      'fixture-reaper.ts',
      import.meta.url,
    ).pathname],
    {
      // A group of its own, so that a signal sent to the test run's group does not end the watcher before it has cleaned up.
      detached: true,
      stdio: [
        'pipe',
        'inherit',
        'inherit',
      ],
    },
  );
  if (!watchers.has('reaper')) {
    watchers.set(
      'reaper',
      reaper,
    );
    // A watcher that cannot start (the process limit is the usual reason) must say so, not crash the test file with a bare
    // `spawn ... EAGAIN`; programs are still killed with their group when they end, only a hard kill of this process is not covered.
    reaper.once(
      'error',
      function reportWatcher(error: unknown): void {
        console.error(`The fixture watcher could not run, so fixture programs are not killed if this process is: ${String(error)}.${mentionsProcessLimit(String(error)) ? ' The process limit was reached: raise the limit (systemd TasksMax=) or lower the number of test files that run at once.' : ''}`);
      },
    );
    /**
     The watcher's input pipe.
     */
    const { stdin } = reaper;
    stdin?.once(
      'error',
      function reportPipe(error: unknown): void {
        console.error(`The fixture watcher's pipe failed: ${String(error)}`);
      },
    );
  }
  // Neither the watcher nor its pipe keeps this process alive; the watcher ends on its own when the pipe closes.
  reaper.unref();
  if (reaper.stdin instanceof Socket) {
    /**
     The pipe, which would otherwise keep this process alive.
     */
    const pipe = reaper.stdin;
    pipe.unref();
  }
  reaper.stdin
    ?.write(`${line}\n`);
}

/**
 Register a process group as belonging to this test process, so that it is killed if this process dies.

 @param group - the group's ID

 @example
 ```ts
 watchGroup(12345);
 ```
 */
export function watchGroup(group: number): void {
  tellReaper(`add ${String(group)}`);
}

/**
 Stop watching a process group that has ended.

 @param group - the group's ID

 @example
 ```ts
 unwatchGroup(12345);
 ```
 */
export function unwatchGroup(group: number): void {
  tellReaper(`remove ${String(group)}`);
}
