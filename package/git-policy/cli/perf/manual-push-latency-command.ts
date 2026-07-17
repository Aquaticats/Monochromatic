/**
 * Shell-free command execution for manual-push latency benchmarking.
 *
 * @module
 */

import spawn from 'nano-spawn';

import {
  COMMAND_ENV,
  NANOSECONDS_PER_MILLISECOND,
  type ExecuteOptions,
} from './manual-push-latency-contracts.ts';

/**
 * Execute command with benchmark environment and optional output suppression.
 *
 * @param command - Executable path or name.
 *
 * @param args - Literal command arguments.
 *
 * @param options - Working directory and output behavior.
 *
 * @returns Trimmed standard output.
 *
 * @throws Error from `nano-spawn` when command fails.
 *
 * @example
 * ```ts
 * await execute({ command: '/usr/bin/git', args: ['--version'] });
 * ```
 */
export async function execute({
  command,
  args,
  options = {},
}: Readonly<{
  command: string;
  args: readonly string[];
  options?: ExecuteOptions;
}>): Promise<string> {
  /**
   * Completed subprocess result containing captured output.
   */
  const result = await spawn(
    command,
    [...args,],
    {
      cwd: options.cwd,
    env: COMMAND_ENV,
    stdin: 'ignore',
    stdout: options.discardOutput === true ? 'ignore' : 'pipe',
      stderr: options.discardOutput === true ? 'ignore' : 'pipe',
    },
  );
  return result.stdout
    .trim();
}

/**
 * Measure successful command wall time.
 *
 * @param command - Executable path or name.
 *
 * @param args - Literal command arguments.
 *
 * @param cwd - Repository where command executes.
 *
 * @returns Elapsed wall time in milliseconds.
 *
 * @throws Error from `nano-spawn` when command fails.
 *
 * @example
 * ```ts
 * await measure({ command: '/usr/bin/git', args: ['status'], cwd: '/work/direct' });
 * ```
 */
export async function measure({
  command,
  args,
  cwd,
}: Readonly<{
  command: string;
  args: readonly string[];
  cwd: string;
}>): Promise<number> {
  /**
   * Monotonic nanosecond timestamp before command execution.
   */
  const started = process.hrtime
    .bigint();
  await execute({
    command,
    args,
    options: {
      cwd,
      discardOutput: true
    }
  });
  /**
   * Monotonic elapsed nanoseconds after command completion.
   */
  const elapsedNanoseconds = process.hrtime
    .bigint()
    - started;
  return Number(elapsedNanoseconds) / NANOSECONDS_PER_MILLISECOND;
}
