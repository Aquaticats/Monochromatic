import { spawn } from 'node:child_process';

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
    if (!(Error.isError(error) && ('code' in error) && (error.code === 'ESRCH')))
      console.error(`Could not kill process group ${String(group)}: ${String(error)}`);
    return false;
  }
}

/**
 The watcher for this test process, started the first time a program starts.
 */
let reaper: ReturnType<typeof spawn> | undefined = undefined;

/**
 Tell the watcher about a process group, starting the watcher if this is the first.

 @param line - `add <id>` or `remove <id>`

 @example
 ```ts
 tellReaper('add 12345');
 ```
 */
function tellReaper(line: string): void {
  reaper ??= spawn(
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
  // Neither the watcher nor its pipe keeps this process alive; the watcher ends on its own when the pipe closes.
  reaper.unref();
  reaper.stdin
    ?.unref();
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
