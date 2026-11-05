import { spawn, } from 'node:child_process';
import { once, } from 'node:events';

//region Await task processes with inherited terminal streams

/** A task command did not complete successfully. */
export class TaskCommandFailedError extends Error {
  /**
   Identifies the command, directory and terminal status.

   @param command - Executable selected by the task.
   @param cwd - Working directory where the command failed.
   @param status - Exit status, signal or invalid command diagnostic.
   */
  constructor({ command, cwd, status, }: {
    readonly command: string;
    readonly cwd: string;
    readonly status: string;
  },) {
    super(`Task command ${command} in ${cwd} failed: ${status}`,);
    this.name = 'TaskCommandFailedError';
  }
}

/**
 Starts a command with literal arguments and waits until its process closes.

 @param command - Executable resolved through the task environment.
 @param args - Literal argument vector, never shell-interpolated.
 @param cwd - Explicit working directory owned by the task.
 @param env - Snapshot inherited by the process.
 @throws TaskCommandFailedError when the command closes unsuccessfully.
 @example
 ```ts
 await runTaskProcess({ command: 'node', args: ['build.ts'], cwd, env });
 ```
 */
export async function runTaskProcess({ command, args, cwd, env, }: {
  readonly command: string;
  readonly args: readonly string[];
  readonly cwd: string;
  readonly env: Readonly<NodeJS.ProcessEnv>;
},): Promise<void> {
  /** Inherited streams preserve terminal output without accumulating build logs in memory. */
  const child = spawn(command, args, { cwd, env, stdio: 'inherit', },);
  /** Event payloads cross an untyped Node boundary and are kept unknown until inspection. */
  const result: readonly unknown[] = await once(child, 'close',);
  /** Close reports either an exit code or a terminating signal. */
  const [code, signal,] = result;
  if (code !== 0)
    throw new TaskCommandFailedError({
      command,
      cwd,
      status: signal === null ? `exit ${String(code,)}` : `signal ${String(signal,)}`,
    },);
}

//endregion Await task processes with inherited terminal streams
