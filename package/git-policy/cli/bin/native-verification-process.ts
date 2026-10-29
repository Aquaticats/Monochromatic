/**
 Shell-free command execution and container cleanup shared by the native wrapper's container verification runners.

 Every command runs from a complete argument array, so no shell ever interprets an argument.
 A command that cannot start, exceeds the capture limit, or exits nonzero without `allowFailure`
 throws an error naming the command and its first argument, so a failed step always fails the task.
 */
// The bin scripts sit outside the package tsconfig's include list, so type-aware lint checks them in an inferred
// program that loads no ambient Node types; this reference loads them, so `process` and `node:` imports stay typed
// in this file on its own instead of depending on a sibling module that carries the same reference.
/// <reference types="node" />
import { spawn } from 'node:child_process';
import { once } from 'node:events';

/**
 Largest stdout or stderr a captured command may produce: 64 MiB.
 The former synchronous runners passed this bound as `maxBuffer`; overflow stops the command and fails the step.
 */
const captureLimitBytes = 67_108_864;

/**
 Failed setup or execution cannot masquerade as native verification.
 */
export class NativeVerificationError extends Error {
  name = 'NativeVerificationError';
}

/**
 One captured output stream and whether it passed the capture limit.

 @typedef {{ text: string, overflowed: boolean }} CapturedStream
 */

/**
 Exit state of one finished command plus its captured output, empty when output went to the terminal.

 @typedef {{ status: number | null, signal: NodeJS.Signals | null, stdout: string, stderr: string }} CommandOutcome
 */

/**
 Read one captured stream to text, stopping the command once the stream passes the capture limit.

 @param {{ stream: import('node:stream').Readable, child: import('node:child_process').ChildProcess }} request -
   stream to drain and the command to stop on overflow, so a runaway command cannot exhaust memory
 @returns {Promise<CapturedStream>} decoded text and overflow flag, kept apart so the caller names the step
 */
async function readCapturedStream({
  stream,
  child,
}) {
  /** @type {Buffer[]} */
  const chunks = [];
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
  const captured = {
    text: Buffer.concat(chunks)
      .toString('utf8'),
    overflowed: receivedBytes > captureLimitBytes,
  };
  return captured;
}

/**
 Run one complete argument array with no shell interpretation.

 @param {{ command: string, args: readonly string[], capture?: boolean, allowFailure?: boolean, cwd?: string,
   env?: NodeJS.ProcessEnv }} request -
   executable, its arguments, whether to capture output instead of streaming it to the terminal,
   whether a nonzero exit is evidence the caller inspects rather than a failed step,
   and the working directory and environment, which default to this process's
 @returns {Promise<CommandOutcome>} exit state and captured output for steps that read either
 */
export async function runCommand({
  command,
  args,
  capture = false,
  allowFailure = false,
  cwd = process.cwd(),
  env = process.env,
}) {
  const step = args[0] === undefined
    ? command
    : `${command} ${args[0]}`;
  const child = spawn(
    command,
    [...args],
    {
      cwd,
      env,
      stdio: capture
        ? [
          'ignore',
          'pipe',
          'pipe',
        ]
        : 'inherit',
    },
  );
  const emptyStream = Promise.resolve({
    text: '',
    overflowed: false,
  });
  const stdout = child.stdout === null
    ? emptyStream
    : readCapturedStream({
      stream: child.stdout,
      child,
    });
  const stderr = child.stderr === null
    ? emptyStream
    : readCapturedStream({
      stream: child.stderr,
      child,
    });
  const closed = once(
    child,
    'close',
  );
  try {
    await Promise.all([
      stdout,
      stderr,
      closed,
    ]);
  }
  catch (error) {
    throw new NativeVerificationError(
      `${step} could not start or its output could not be read.`,
      { cause: error },
    );
  }
  const capturedStdout = await stdout;
  const capturedStderr = await stderr;
  if (capturedStdout.overflowed || capturedStderr.overflowed)
    throw new NativeVerificationError(`${step} produced more than ${String(captureLimitBytes)} bytes on one output stream.`);
  const outcome = {
    status: child.exitCode,
    signal: child.signalCode,
    stdout: capturedStdout.text,
    stderr: capturedStderr.text,
  };
  if ((!allowFailure) && (outcome.status !== 0))
    throw new NativeVerificationError(
      `${step} failed (status ${String(outcome.status)}, signal ${String(outcome.signal)}). ${outcome.stderr}`,
    );
  return outcome;
}

/**
 Create one container and remove it when its scope ends, whether verification passed or failed.

 @param {{ args: readonly string[] }} request - complete `podman create` arguments
 @returns {Promise<{ id: string, [Symbol.asyncDispose]: () => Promise<void> }>} container ID
   and its forced removal, bound before the caller validates the ID so even a malformed result is cleaned up
 */
export async function createContainer({ args }) {
  const created = await runCommand({
    command: 'podman',
    args,
    capture: true,
  });
  const id = created.stdout
    .trim();
  return {
    id,
    async [Symbol.asyncDispose]() {
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
