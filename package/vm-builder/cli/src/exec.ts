import spawn from 'nano-spawn';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { l, } from './logger.ts';

/**
 Runs one external command directly (no shell) and resolves with its captured stdout.

 Stdout loses one trailing newline, as `nano-spawn` strips it; stderr is not returned.
 The command logs at debug level before it starts, so a failed probe can be matched to its argument list.

 @param cmd - Executable name or path; looked up on `PATH` when it has no directory part

 @param args - Arguments passed to the command unchanged; no shell expansion applies

 @param timeoutMs - Milliseconds after which the child is sent `SIGTERM`; omitted means no limit

 @returns Stdout of the command, without one trailing newline

 @throws `SubprocessError` from `nano-spawn` when the command exits non-zero:
 message `Command failed with exit code N: cmd args`, with `exitCode`, `stdout` and `stderr` properties

 @throws `SubprocessError` when the command is killed by a signal, including the `timeoutMs` kill:
 message `Command was terminated with SIGTERM: cmd args`, with `signalName`

 @throws `SubprocessError` when the command cannot start, for example a missing executable:
 message `Command failed: cmd args`, the system error as `cause`, and no `exitCode`

 @example
 ```ts
 const state = (await exec({ cmd: 'virsh', args: ['domstate', 'monochromatic-dev',], },)).trim();
 ```
 */
export async function exec(
  {
    cmd,
    args = [],
    timeoutMs,
  }: {
    readonly cmd: string;
    readonly args?: readonly string[];
    readonly timeoutMs?: number;
  },
): Promise<string> {
  /**
   Function-scoped logger tagged with the call site for traceable command logs.
   */
  const rl = tagged({
    tag: exec.name,
    l,
  },);
  rl.debug(`${cmd} ${args.join(' ',)}`,);
  /**
   Spawn result; only `stdout` is forwarded to the caller.
   */
  const { stdout, } = await spawn(
    cmd,
    [...args,],
    timeoutMs === undefined
      ? {}
      : { timeout: timeoutMs, },
  );
  return stdout;
}
