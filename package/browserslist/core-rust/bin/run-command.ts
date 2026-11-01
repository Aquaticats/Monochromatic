/**
 Shell-free command runner shared by the conformance scripts.
 */
import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';

/**
 Raised when a started command ends with a nonzero status or a signal.
 */
export class CommandFailedError extends Error {
  /**
   Names the failing command in the message.

   @param message - the command line and how it ended, so the failing step is clear from the log alone

   @param options - the underlying failure, as `cause`
   */
  public constructor(
    message: string,
    options?: ErrorOptions,
  ) {
    super(
      message,
      options,
    );
    this.name = 'CommandFailedError';
  }
}

/**
 One command to run.
 */
export type CommandRun = {
  /**
   Executable name or path.
   */
  readonly command: string;
  /**
   Arguments, each passed as exactly one argument (no shell is involved).
   */
  readonly args: readonly string[];
  /**
   Working directory; the caller's when absent.
   */
  readonly cwd?: string;
  /**
   True to collect standard output and return it instead of passing it through to the terminal.
   */
  readonly captureOutput?: boolean;
};

/**
 Wait for a started command to close.

 @param child - the running command

 @returns the exit code and the signal the close event reports; each is `null` when the command ended the other way

 @throws CommandFailedError when the command could not be started

 @example
 ```ts
 const [code] = await closeOf(spawn('true'));
 ```
 */
async function closeOf(child: ChildProcess,): Promise<readonly unknown[]> {
  try {
    /**
     Arguments of the close event.
     */
    const closed: readonly unknown[] = await once(
      child,
      'close',
    );
    return closed;
  } catch (error) {
    throw new CommandFailedError(
      'the command could not be started',
      { cause: error, },
    );
  }
}

/**
 Run a command to completion without a shell.

 @param command - executable name or path

 @param args - arguments, each passed as one argument

 @param cwd - working directory, or nothing for the caller's

 @param captureOutput - true to return standard output instead of passing it through

 @returns the standard output bytes when captured, otherwise an empty buffer

 @throws CommandFailedError when the command cannot start or does not exit with status zero

 @example
 ```ts
 const bytes = await runCommand({ command: 'tar', args: ['--list', '--file', 'a.tgz'], captureOutput: true });
 ```
 */
export async function runCommand({
  command,
  args,
  cwd,
  captureOutput = false,
}: CommandRun,): Promise<Buffer> {
  /**
   The running command.
   */
  const child = spawn(
    command,
    [...args,],
    {
      ...(cwd === undefined ? {} : { cwd, }),
      stdio: captureOutput ? [
        'ignore',
        'pipe',
        'inherit',
      ] : 'inherit',
    },
  );
  /**
   Standard output chunks as they arrive.
   */
  const chunks: Buffer[] = [];
  child.stdout
    ?.on(
    'data',
    function collectChunk(chunk: Buffer,) {
      chunks.push(chunk,);
    },
    );
  /**
   Exit code and signal; each is `null` when the command ended the other way.
   */
  const [code, signal,] = await closeOf(child,);
  if (code !== 0)
    throw new CommandFailedError(
      `${command} ${args.join(' ',)} ended with code ${String(code,)} and signal ${String(signal,)}`,
    );
  return Buffer.concat(chunks,);
}
