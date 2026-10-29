/**
 * Low-level container execution: runs a command in the container runtime and
 * returns structured output including stdout, stderr, exit code, and timeout status.
 */
import { MS_PER_SECOND, } from '@monochromatic-dev/const-core/ts';

import { execBun, } from './container-base.ts';
import {
  CONTAINER_RUNTIME,
  CONTAINER_TIMEOUT_SECONDS,
  HOST_TIMEOUT_BUFFER_SECONDS,
} from './container-runtime.ts';
import {
  l,
  tagged,
} from './log.ts';

/**
 * Result of running generated code in a container
 */
export type ContainerResult = {
  /**
   * Combined stdout output
   */
  readonly stdout: string;
  /**
   * Combined stderr output
   */
  readonly stderr: string;
  /**
   * Process exit code (0 = success)
   */
  readonly exitCode: number;
  /**
   * Whether the container was killed due to timeout
   */
  readonly timedOut: boolean;
};

/**
 * Options for {@link execContainer}.
 *
 * @example
 * ```ts
 * const opts: ExecContainerOptions = {
 *   containerArgs: ['run', '--rm', 'oven/bun:1.3', 'echo', 'hi'],
 *   signal: undefined,
 * };
 * ```
 */
type ExecContainerOptions = {
  /**
   * Fully-formed arguments for the container runtime binary
   */
  readonly containerArgs: readonly string[];
  /**
   * Abort signal; kills the container process immediately on abort, or absent to disable
   */
  readonly signal?: AbortSignal;
};

/**
 * Executes the container command and captures stdout/stderr/exit-code.
 *
 * Resolves on any exit code (including non-zero): callers check exitCode/timedOut.
 *
 * @param containerArgs - fully-formed arguments for the container runtime binary
 *
 * @param signal - optional abort signal; kills the container process immediately on abort
 *
 * @returns container execution result
 *
 * @example
 * ```ts
 * const result = await execContainer({ containerArgs: ['run', '--rm', 'oven/bun:1.3', 'echo', 'hi'] });
 * result.stdout; // "hi\n"
 * ```
 */
export async function execContainer({
  containerArgs,
  signal,
}: ExecContainerOptions,): Promise<ContainerResult> {
  /**
   * Maximum stderr characters to include in error log
   */
  const STDERR_PREVIEW_LENGTH = 200;
  /**
   * Total host-side timeout: container limit plus a buffer for startup/teardown
   */
  const timeoutMs = (CONTAINER_TIMEOUT_SECONDS + HOST_TIMEOUT_BUFFER_SECONDS)
    * MS_PER_SECOND;
  /**
   * Raw spawn result before the kill-vs-exit fields are renamed for the public {@link ContainerResult} shape.
   */
  const result = await execBun({
    command: CONTAINER_RUNTIME,
    args: containerArgs,
    options: {
      timeout: timeoutMs,
      ...((signal !== undefined) ? { signal, } : {}),
    },
  },);

  if ((result.exitCode
    !== 0) || result
    .killed) {
    /**
     * Container-specific logger for execution failure messages.
     */
    const rl = tagged({
      tag: 'container',
      l,
    },);
    rl.error(
      `exit=${String(result.exitCode,)} timedOut=${String(result.killed,)}`,
    );
    if (result.stderr
      .length
      > 0) {
      rl.error(
        `stderr: ${
          result.stderr
            .slice(
            0,
            STDERR_PREVIEW_LENGTH,
          )
        }`,
      );
    }
  }

  return {
    stdout: result.stdout,
    stderr: result.stderr,
    exitCode: result.exitCode,
    timedOut: result.killed,
  };
}
