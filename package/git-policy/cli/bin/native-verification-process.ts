/**
 Shell-free command execution and container cleanup shared by the native wrapper's container verification runners.

 Every command runs from a complete argument array, so no shell ever interprets an argument.
 A command that cannot start, exceeds the capture limit, or exits nonzero without `allowFailure`
 throws an error naming the command and its first argument, so a failed step always fails the task.
 */
import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';
import type { Readable, } from 'node:stream';

import {
  type CommandDeadline,
  stopAtDeadline,
} from './command-deadline.ts';

/**
 Largest stdout or stderr a captured command may produce: 64 MiB.
 The former synchronous runners passed this bound as `maxBuffer`; overflow stops the command and fails the step.
 */
const captureLimitBytes = 67_108_864;

/**
 Failed setup or execution cannot masquerade as native verification.
 */
export class NativeVerificationError extends Error {
  /**
   Stable class name that tells a failed verification step apart from an unrelated error in a log.
   */
  override name = 'NativeVerificationError';
}

/**
 One captured output stream and whether it passed the capture limit.
 */
export type CapturedStream = {
  readonly text: string;
  readonly overflowed: boolean;
};

/**
 Exit state of one finished command plus its captured output, empty when output went to the terminal.
 `status` is absent when a signal ended the command, and `signal` is absent when the command exited by itself.
 */
export type CommandOutcome = {
  readonly status?: number;
  readonly signal?: NodeJS.Signals;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Read one captured stream to text, stopping the command once the stream passes the capture limit.

 @param stream - stream to drain
 
 @param child - command to stop on overflow, so a runaway command cannot exhaust memory

 @returns decoded text and overflow flag, kept apart so the caller names the step
 */
async function readCapturedStream({
  stream,
  child,
}: {
  readonly stream: Readable;
  readonly child: ChildProcess;
}): Promise<CapturedStream> {
  /**
   Chunks kept until the stream ends or passes the capture limit.
   */
  const chunks: Buffer[] = [];
  /**
   Bytes seen so far, counted before a chunk is kept so the overflowing chunk is never buffered.
   */
  let receivedBytes = 0;
  for await (const chunk of stream) {
    if (!Buffer.isBuffer(chunk))
      throw new NativeVerificationError('A captured command produced a non-byte output chunk.');
    receivedBytes += chunk.length;
    if (receivedBytes > captureLimitBytes) {
      child.kill();
      break;
    }
    chunks.push(chunk);
  }
  /**
   Text of the kept chunks with the overflow verdict.
   */
  const captured = {
    text: Buffer.concat(chunks)
      .toString('utf8'),
    overflowed: receivedBytes > captureLimitBytes,
  };
  return captured;
}

/**
 How waiting for one command, its streams and its close ended.
 */
type CommandWait =
  | { readonly kind: 'ended', }
    | {
      readonly kind: 'refused',
      readonly cause: unknown,
    };

/**
 Wait for one command, its streams and its close, and report how it ended instead of raising it.

 The wait is kept whole because the deadline a caller gave the command is read before this wait's own failure: a command
 stopped at its deadline reports the deadline, never the broken pipe the stop left behind.

 @param stdout - the command's standard output being read

 @param stderr - the command's standard error being read

 @param closed - the close event of the command

 @returns that all three ended, or what refused them

 @example
 ```ts
 const waited = await waitForCommand({ stdout, stderr, closed });
 ```
 */
async function waitForCommand({
  stdout,
  stderr,
  closed,
}: {
  readonly stdout: Promise<CapturedStream>;
  readonly stderr: Promise<CapturedStream>;
  readonly closed: Promise<unknown>;
},): Promise<CommandWait> {
  try {
    await Promise.all([
      stdout,
      stderr,
      closed,
    ],);
    return { kind: 'ended', };
  }
  catch (cause) {
    return {
      kind: 'refused',
      cause,
    };
  }
}

/**
 Wait for one command under its deadline, and leave the wait at the grace even when something the stop cannot end is
 still there.

 A process the kernel cannot end, or a child that escaped the stopped process group while holding the command's output,
 keeps the streams and the close of the command open, and nothing the stop sends can free them. The grace releases the
 caller anyway, because "nothing should ever wait forever or indefinitely".

 @param stdout - the command's standard output being read

 @param stderr - the command's standard error being read

 @param closed - the close event of the command

 @param deadline - the stop this command's deadline makes, whose grace releases this wait

 @returns that all three ended, or what refused them

 @example
 ```ts
 const waited = await waitForStop({ stdout, stderr, closed, deadline });
 ```
 */
async function waitForStop({
  stdout,
  stderr,
  closed,
  deadline,
}: {
  readonly stdout: Promise<CapturedStream>;
  readonly stderr: Promise<CapturedStream>;
  readonly closed: Promise<unknown>;
  readonly deadline: CommandDeadline;
},): Promise<CommandWait> {
  try {
    await Promise.race([
      Promise.all([
        stdout,
        stderr,
        closed,
      ],),
      deadline.released(),
    ],);
    return { kind: 'ended', };
  }
  catch (cause) {
    return {
      kind: 'refused',
      cause,
    };
  }
}

/**
 Run one complete argument array with no shell interpretation.

 @param command - executable to start
 
 @param args - its arguments, each passed as one argument with no shell interpretation
 
 @param capture - whether to capture output instead of streaming it to the terminal
 
 @param allowFailure - whether a nonzero exit is evidence the caller inspects rather than a failed step
 
 @param cwd - working directory, which defaults to this process's
 
 @param env - environment, which defaults to this process's

 @param deadlineMilliseconds - how long the command may run before it and everything it started are stopped, absent
   when the caller has no bound for it; a bound also releases the wait at the stop's grace, for output that something
   the stop cannot end still holds

 @returns exit state and captured output for steps that read either

 @throws NativeVerificationError when the command could not run, when it ran past a deadline it was given, or when it
   exited nonzero without `allowFailure`

 @example
 ```ts
 const outcome = await runCommand({ command: 'podman', args: ['--version'], capture: true });
 ```
 */
export async function runCommand({
  command,
  args,
  capture = false,
  allowFailure = false,
  cwd = process.cwd(),
  env = process.env,
  deadlineMilliseconds,
}: {
  readonly command: string;
  readonly args: readonly string[];
  readonly capture?: boolean;
  readonly allowFailure?: boolean;
  readonly cwd?: string;
  readonly env?: NodeJS.ProcessEnv;
  readonly deadlineMilliseconds?: number;
}): Promise<CommandOutcome> {
  /**
   Command and first argument, enough to name the step in an error without echoing every argument.
   */
  const step = args[0] === undefined
    ? command
    : `${command} ${args[0]}`;
  /**
   The bound this command is given, absent when it has none, so one narrowed value decides the stop below.
   */
  const bound = deadlineMilliseconds;
  /**
   The started command; a command that cannot start surfaces through its streams and its close event.
   A bounded command leads a process group of its own, so its deadline stops everything it started and not only the
   program named here: a `podman run` client stopped alone leaves its container running.
   */
  const child = spawn(
    command,
    [...args],
    {
      cwd,
      env,
      detached: bound !== undefined,
      stdio: capture
        ? [
          'ignore',
          'pipe',
          'pipe',
        ]
        : 'inherit',
    },
  );
  /**
   Stand-in for a stream that was not piped, so streamed output counts as empty captured output.
   */
  const emptyStream = Promise.resolve({
    text: '',
    overflowed: false,
  });
  /**
   Standard output being read, or empty when it went to the terminal.
   */
  const stdout = child.stdout === null
    ? emptyStream
    : readCapturedStream({
      stream: child.stdout,
      child,
    });
  /**
   Standard error being read, or empty when it went to the terminal.
   */
  const stderr = child.stderr === null
    ? emptyStream
    : readCapturedStream({
      stream: child.stderr,
      child,
    });
  /**
   Settles when the command and its streams have closed, or rejects when it could not start.
   */
  const closed = once(
    child,
    'close',
  );
  /**
   The stop this command's deadline makes, absent when the caller gave it no bound.
   */
  const deadline = bound === undefined
    ? undefined
    : stopAtDeadline({
      child,
      deadlineMilliseconds: bound,
      refusal: `${step} was still running after ${String(bound)} ms, the bound it is given, so it and everything it started were stopped.`,
    },);
  /**
   How the wait for the command ended, kept whole so the deadline is read before the wait's own failure is reported.
   */
  const waited = deadline === undefined
    ? await waitForCommand({
      stdout,
      stderr,
      closed,
    },)
    : await waitForStop({
      stdout,
      stderr,
      closed,
      deadline,
    },);
  if (deadline !== undefined)
    deadline.end();
  if ((deadline !== undefined) && deadline.stoppedTheCommand())
    throw new NativeVerificationError(deadline.refusal);
  if (waited.kind === 'refused') {
    throw new NativeVerificationError(
      `${step} could not start or its output could not be read.`,
      { cause: waited.cause, },
    );
  }
  /**
   Finished standard output.
   */
  const capturedStdout = await stdout;
  /**
   Finished standard error.
   */
  const capturedStderr = await stderr;
  if (capturedStdout.overflowed || capturedStderr.overflowed)
    throw new NativeVerificationError(`${step} produced more than ${String(captureLimitBytes)} bytes on one output stream.`);
  /**
   Exit state with the captured output; a code or signal the command did not end with stays absent.
   */
  const outcome: CommandOutcome = {
    ...(child.exitCode === null ? {} : { status: child.exitCode, }),
    ...(child.signalCode === null ? {} : { signal: child.signalCode, }),
    stdout: capturedStdout.text,
    stderr: capturedStderr.text,
  };
  if ((!allowFailure) && (outcome.status !== 0))
    throw new NativeVerificationError(
      `${step} failed (status ${String(outcome.status ?? 'none')}, signal ${outcome.signal ?? 'none'}). ${outcome.stderr}`,
    );
  return outcome;
}

/**
 Create one container and remove it when its scope ends, whether verification passed or failed.

 @param args - complete `podman create` arguments

 @returns container ID
   and its forced removal, bound before the caller validates the ID so even a malformed result is cleaned up

 @example
 ```ts
 await using container = await createContainer({ args: ['create', 'localhost/image:tag'] });
 ```
 */
export async function createContainer({ args, }: {
  readonly args: readonly string[];
},): Promise<{
  readonly id: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
}> {
  /**
   Result of `podman create`, whose standard output is the new container's ID.
   */
  const created = await runCommand({
    command: 'podman',
    args,
    capture: true,
  });
  /**
   Container ID without the trailing newline.
   */
  const id = created.stdout
    .trim();
  return {
    id,
    async [Symbol.asyncDispose](): Promise<void> {
      await runCommand({
        command: 'podman',
        args: [
          'rm',
          '--force',
          id,
        ],
        capture: true,
      });
    },
  };
}
