/**
 Runs one command asynchronously and collects its output.

 The alert entry starts `journalctl` and `notify-send`, and the install task copies
 files; none of that needs a synchronous API, and the repository bans Node's
 synchronous ones outside tests.

 @module
 */

import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
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
 What one command is asked to do.
 */
export type CommandRequest = {
  readonly command: string;
  readonly args: readonly string[];
  readonly timeoutMs?: number;
};

/**
 Runs one command to its end, or until its timeout stops it.

 Injected into the modules that start a program, so a test can drive what a command
 reported without a notification server or a journal.
 */
export type CommandRunner = (request: Readonly<CommandRequest>,) => Promise<CommandResult>;

/**
 Reads a child's stream to its end.

 @param stream - the child's stdout or stderr

 @returns everything the stream produced, decoded as UTF-8
 */
async function collectStream(stream: NodeJS.ReadableStream,): Promise<string> {
  return await readStreamText(stream,);
}

/**
 What stopping a command produced.

 `problem` is set when waiting for the stopped command's close failed.
 */
type StopResult = {
  readonly problem?: string;
};

/**
 Stops a command the runner no longer waits for, and releases its pipes.

 The timeout's abort sends SIGTERM, which a program may ignore or handle slowly; its
 pipes would then keep this process open until whoever started it gives up, so the
 command is killed hard instead. SIGKILL cannot be ignored, and the pipes are dropped
 first in case a grandchild inherited them.

 @param child - the started process

 @returns an empty result once the command is gone, or the reason it is not
 */
async function stopCommand(child: ChildProcess,): Promise<StopResult> {
  /**
   The child's pipes, dropped so a grandchild that inherited them cannot keep this process
   open.
   */
  const {
    stderr,
    stdout,
  } = child;
  stdout?.destroy();
  stderr?.destroy();
  /**
   Whether the command has ended already: Node records an exit code, or the signal that
   ended it, before it emits close, and a command that never started records -2.
   */
  if ((child.exitCode !== null) || (child.signalCode !== null)) return {};
  /**
   Close of the child, taken before the kill so a fast exit is not missed.
   */
  const closed = once(
    child,
    'close',
  );
  child.kill('SIGKILL',);
  try {
    await closed;
    return {};
  } catch (error) {
    return { problem: `stopping it failed: ${String(error,)}`, };
  }
}

/**
 Drops a child's pipes, whatever its state.

 The timeout's abort uses this because Node's own abort emits its error only when the
 child could still be killed. A process the child started can inherit its stdout and
 stderr and hold them open after the child itself exited, and then the abort reports
 nothing and a pending stream read would wait for the descendant forever.

 @param child - the started process whose pipes are dropped
 */
function releasePipes(child: ChildProcess,): void {
  child.stdout
    ?.destroy();
  child.stderr
    ?.destroy();
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
}: Readonly<CommandRequest>,): Promise<CommandResult> {
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
  /**
   Drops the child's pipes when the timeout fires, so a descendant that inherited them
   cannot keep this run waiting.
   */
  timeout.addEventListener(
    'abort',
    function releasePipesOnTimeout() {
      releasePipes(started,);
    },
    { once: true, },
  );
  try {
    /**
     Both streams read together; a child that fills one pipe while the other is
     unread would otherwise wait forever.

     The close event is awaited in the same batch, because `once` also turns the
     `error` a command that cannot start or a timeout stop raises into a rejection:
     a child with no `error` listener would otherwise crash the whole entry.
     */
    const [stdout, stderr, closed,] = await Promise.all([
      collectStream(started.stdout,),
      collectStream(started.stderr,),
      once(
        started,
        'close',
      ),
    ],);
    /**
     Exit status of the finished process: its number for an exit, or -1 when a signal
     ended it. It is absent only from the result a problem path returns.
     */
    const exitCode: unknown = closed.at(0,);
    return {
      stdout,
      stderr,
      exitCode: (typeof exitCode) === 'number' ? exitCode : -1,
    };
  } catch (error) {
    /**
     Stopping the command, which the timeout may have left alive because SIGTERM can be
     ignored.
     */
    const stopped = await stopCommand(started,);
    /**
     Why the command did not finish, with any refusal to stop it.
     */
    const reason = `${command} could not be run or was stopped after ${String(timeoutMs,)} ms: ${String(error,)}`;
    return {
      stdout: '',
      stderr: '',
      problem: stopped.problem === undefined ? reason : `${reason}; ${stopped.problem}`,
    };
  }
}
