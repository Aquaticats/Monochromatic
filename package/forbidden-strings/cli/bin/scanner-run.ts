/**
 Shell-free management commands of the scanner's container verification.
 */
import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { once } from 'node:events';
import {
  open,
  readFile,
} from 'node:fs/promises';
import { join } from 'node:path';

/**
 Failed setup or incomplete evidence never counts as a verification pass.
 */
export class ScannerVerificationError extends Error {
  /**
   Names the failing step in the message.

   @param message - what failed and where the evidence is

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
    this.name = 'ScannerVerificationError';
  }
}

/**
 Options of one shell-free management command.
 */
export type RunOptions = {
  readonly command: string;
  readonly args: readonly string[];
  readonly capture?: boolean;
  readonly allowFailure?: boolean;
  /**
   Evidence directory that receives `stdout.log` and `stderr.log`; the last 16000 characters of each print to the
   terminal.
   */
  readonly transcript?: string;
};

/**
 What a finished command left behind; a status or signal is absent when the command did not end that way.
 */
export type RunResult = {
  readonly status?: number;
  readonly signal?: string;
  /**
   Captured standard output; empty unless the command was captured.
   */
  readonly stdout: string;
  /**
   Captured standard error; empty unless the command was captured.
   */
  readonly stderr: string;
};

/**
 Number of trailing characters of each transcript log that reach the terminal.
 */
const previewLength = 16_000;

/**
 Wait for a started command to close.

 @param child - the running command

 @returns the exit code and the signal that the close event reports

 @throws ScannerVerificationError when the command could not be started

 @example
 ```ts
 const [code] = await closeOf(spawn('true'));
 ```
 */
async function closeOf(child: ChildProcess): Promise<readonly unknown[]> {
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
    throw new ScannerVerificationError(
      'Verification command could not start.',
      { cause: error },
    );
  }
}

/**
 Start a command, wait for it to close and collect what it was asked to capture.

 @param command - executable name or path

 @param args - arguments, each passed as one argument

 @param capture - true to collect standard output and error

 @param transcript - evidence directory that receives the output logs, or nothing

 @returns how the command ended, and its captured output

 @throws ScannerVerificationError when the command cannot be started

 @example
 ```ts
 await spawnAndWait({ command: 'true', args: [], capture: false });
 ```
 */
async function spawnAndWait({
  command,
  args,
  capture,
  transcript,
}: {
  readonly command: string;
  readonly args: readonly string[];
  readonly capture: boolean;
  readonly transcript?: string;
}): Promise<RunResult> {
  /**
   Log that receives standard output, when there is a transcript.
   */
  await using stdoutLog = transcript === undefined ? undefined : await open(
    join(
      transcript,
      'stdout.log',
    ),
    'w',
  );
  /**
   Log that receives standard error, when there is a transcript.
   */
  await using stderrLog = transcript === undefined ? undefined : await open(
    join(
      transcript,
      'stderr.log',
    ),
    'w',
  );
  /**
   The running command.
   */
  const child = spawn(
    command,
    [...args],
    {
      stdio: (stdoutLog !== undefined) && (stderrLog !== undefined) ? [
        'ignore',
        stdoutLog.fd,
        stderrLog.fd,
      ] : capture ? [
        'ignore',
        'pipe',
        'pipe',
      ] : 'inherit',
    },
  );
  /**
   Standard output chunks as they arrive.
   */
  const outChunks: Buffer[] = [];
  /**
   Standard error chunks as they arrive.
   */
  const errorChunks: Buffer[] = [];
  child.stdout
    ?.on(
    'data',
    function collectOut(chunk: Buffer) {
      outChunks.push(chunk);
    },
    );
  child.stderr
    ?.on(
    'data',
    function collectError(chunk: Buffer) {
      errorChunks.push(chunk);
    },
    );
  /**
   Exit code and signal; each is not what its name says when the command ended the other way.
   */
  const [code, signal] = await closeOf(child);
  return {
    ...((typeof code) === 'number' ? { status: code } : {}),
    ...((typeof signal) === 'string' ? { signal } : {}),
    stdout: Buffer.concat(outChunks)
      .toString('utf8'),
    stderr: Buffer.concat(errorChunks)
      .toString('utf8'),
  };
}

/**
 Print the end of one transcript log; the full bytes stay in the evidence directory.

 @param transcript - evidence directory holding the log

 @param filename - the log to preview

 @example
 ```ts
 await previewLog({ transcript, filename: 'stdout.log' });
 ```
 */
async function previewLog({
  transcript,
  filename,
}: {
  readonly transcript: string;
  readonly filename: string;
}): Promise<void> {
  console.log((await readFile(
    join(
      transcript,
      filename,
    ),
    'utf8',
  ))
    .slice(-previewLength));
}

/**
 Run a shell-free management command; allow failures only when their evidence is retained.

 @param command - executable name or path

 @param args - arguments, each passed as one argument

 @param capture - true to collect standard output and error instead of passing them through

 @param allowFailure - true when a nonzero exit is a result to inspect

 @param transcript - evidence directory that receives the output logs, or nothing

 @returns the finished command

 @throws ScannerVerificationError when the command cannot start, or exits nonzero while failures are not allowed

 @example
 ```ts
 const { stdout } = await run({ command: 'podman', args: ['--version'], capture: true });
 ```
 */
export async function run({
  command,
  args,
  capture = false,
  allowFailure = false,
  transcript,
}: RunOptions): Promise<RunResult> {
  /**
   How the command ended, with its captured output.
   */
  const result = await spawnAndWait({
    command,
    args,
    capture,
    ...(transcript === undefined ? {} : { transcript }),
  });
  if (transcript !== undefined) {
    await previewLog({
      transcript,
      filename: 'stdout.log',
    });
    await previewLog({
      transcript,
      filename: 'stderr.log',
    });
  }
  if ((!allowFailure) && (result.status !== 0))
    throw new ScannerVerificationError(`${command} ${args[0] ?? ''} exited ${result.status ?? 'without a status'}: ${result.stderr}`);
  return result;
}
