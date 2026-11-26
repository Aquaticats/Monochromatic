/**
 Runs one command asynchronously and collects its output.

 The alert entry starts `journalctl` and `notify-send`, and the install task copies
 files; none of that needs a synchronous API, and the repository bans Node's
 synchronous ones outside tests.

 @module
 */

import { spawn, } from 'node:child_process';
import {
  once,
} from 'node:events';
import { text as readStreamText, } from 'node:stream/consumers';

/**
 How long one command may run before it is stopped, in milliseconds.
 */
export const COMMAND_TIMEOUT_MS = 30_000;

/**
 What one finished command produced.

 `problem` is set when the command could not run or was stopped, and then no exit
 code is meaningful.
 */
export type CommandResult = {
  readonly stdout: string;
  readonly stderr: string;
  readonly exitCode?: number;
  readonly problem?: string;
};

/**
 Reads a child's stream to its end.

 @param stream - the child's stdout or stderr

 @returns everything the stream produced, decoded as UTF-8
 */
async function collectStream(stream: NodeJS.ReadableStream,): Promise<string> {
  return await readStreamText(stream,);
}

/**
 Runs a command to its end, or until the timeout stops it.

 @param command - executable name or absolute path

 @param args - arguments, one array element each

 @param timeoutMs - longest time the command may run

 @returns stdout, stderr, the exit code, or the reason the command did not finish

 @example
 ```ts
 const result = await runCommand({ command: 'notify-send', args: ['hi'], timeoutMs: 30_000, },);
 result.exitCode; // 0 when the notification server accepted it
 ```
 */
export async function runCommand({
  command,
  args,
  timeoutMs = COMMAND_TIMEOUT_MS,
}: Readonly<{
  command: string;
  args: readonly string[];
  timeoutMs?: number;
}>): Promise<CommandResult> {
  /**
   Signal that stops a command that overruns its timeout.
   */
  const timeout = AbortSignal.timeout(timeoutMs,);
  /**
   The started process, or the error that prevented starting it.
   */
  const started = spawn(
    command,
    [...args,],
    {
      signal: timeout,
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  try {
    /**
     Both streams read together; a child that fills one pipe while the other is
     unread would otherwise wait forever.
     */
    const [stdout, stderr,] = await Promise.all([
      collectStream(started.stdout,),
      collectStream(started.stderr,),
    ],);
    /**
     Exit code and signal of the finished process.
     */
    const closed: readonly unknown[] = await once(
      started,
      'close',
    );
    /**
     Exit status of the finished process: a number for an exit, a signal name, or
     nothing when the process never started.
     */
    const exitCode = closed.at(0,);
    return {
      stdout,
      stderr,
      exitCode: (typeof exitCode) === 'number' ? exitCode : -1,
    };
  } catch (error) {
    return {
      stdout: '',
      stderr: '',
      problem: `${command} could not be run or was stopped after ${String(timeoutMs,)} ms: ${String(error,)}`,
    };
  }
}
