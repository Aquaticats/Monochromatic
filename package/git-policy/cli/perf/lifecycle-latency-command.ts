/**
 * Shell-free process and statistics helpers for lifecycle latency benchmarks.
 *
 * @module
 */

import nanoSpawn from 'nano-spawn';
import {
  LifecycleBenchmarkError,
  NANOSECONDS_PER_MILLISECOND,
  type CommandRequest,
} from './lifecycle-latency-contracts.ts';

/**
 * Numerator for ninety-fifth percentile.
 */
const PERCENTILE_NUMERATOR = 95;
/**
 * Percentage denominator.
 */
const PERCENTAGE_DENOMINATOR = 100;

/**
 * Executes one successful benchmark command.
 *
 * @param request - literal command invocation
 *
 * @returns captured standard output
 *
 * @throws {@link LifecycleBenchmarkError} when command fails
 *
 * @example
 * ```ts
 * await execute({ command: '/usr/bin/git', args: ['--version'], cwd: '/work' });
 * ```
 */
export async function execute(request: CommandRequest,): Promise<string> {
  try {
    /**
     * Completed shell-free process result.
     */
    const result = await nanoSpawn(
      request.command,
      [...request.args,],
      {
        cwd: request.cwd,
        env: {
          ...process.env,
          ...request.env,
        },
        stdin: 'ignore',
        stdout: 'pipe',
        stderr: 'pipe',
      },
    );
    return result.stdout
      .trim();
  }
  catch (error: unknown) {
    throw new LifecycleBenchmarkError(
      `Command failed: ${request.command} ${request.args
        .join(' ')}`,
      { cause: error, },
    );
  }
}

/**
 * Measures one successful command with a monotonic clock.
 *
 * @param request - literal command invocation
 *
 * @returns elapsed wall time in milliseconds
 *
 * @example
 * ```ts
 * await measure({ command: '/usr/bin/git', args: ['status'], cwd: '/work' });
 * ```
 */
export async function measure(request: CommandRequest,): Promise<number> {
  /**
   * Monotonic timestamp before command.
   */
  const started = process.hrtime
    .bigint();
  await execute(request,);
  return Number(process.hrtime
    .bigint()
    - started) / NANOSECONDS_PER_MILLISECOND;
}

/**
 * Returns sorted numeric copy.
 *
 * @param values - measured values
 *
 * @returns ascending values
 */
function sorted(values: readonly number[],): readonly number[] {
  return values.toSorted(function compare(
    left,
    right,
  ) {
    return left - right;
  },);
}

/**
 * Reads required sample.
 *
 * @param values - non-empty values
 *
 * @param index - required position
 *
 * @returns sample at position
 */
function requiredSample({
  values,
  index,
}: Readonly<{
  values: readonly number[];
  index: number;
}>,): number {
  /**
   * Sample required by summary operation.
   */
  const value = values[index];
  if (value === undefined)
    throw new LifecycleBenchmarkError(`Missing latency sample at index ${String(index,)}.`,);
  return value;
}

/**
 * Calculates median.
 *
 * @param values - non-empty measured values
 *
 * @returns median value
 *
 * @example
 * ```ts
 * median([1, 2, 3]);
 * ```
 */
export function median(values: readonly number[],): number {
  if (values.length === 0)
    throw new LifecycleBenchmarkError('Cannot summarize empty latency samples.',);
  /**
   * Ascending sample copy.
   */
  const ordered = sorted(values,);
  /**
   * Integer midpoint.
   */
  const midpoint = Math.floor(ordered.length / 2,);
  if ((ordered.length % 2) === 1)
    return requiredSample({
      values: ordered,
      index: midpoint,
    },);
  return (
    requiredSample({
      values: ordered,
      index: midpoint - 1,
    },)
    + requiredSample({
      values: ordered,
      index: midpoint,
    },)
  ) / 2;
}

/**
 * Calculates median absolute deviation.
 *
 * @param values - non-empty measured values
 *
 * @returns median distance from median
 *
 * @example
 * ```ts
 * medianAbsoluteDeviation([1, 2, 3]);
 * ```
 */
export function medianAbsoluteDeviation(values: readonly number[],): number {
  /**
   * Median used as robust distribution center.
   */
  const center = median(values,);
  return median(values.map(function distance(value,) {
    return Math.abs(value - center,);
  },),);
}

/**
 * Calculates nearest-rank ninety-fifth percentile.
 *
 * @param values - non-empty measured values
 *
 * @returns percentile value
 *
 * @example
 * ```ts
 * p95([1, 2, 3]);
 * ```
 */
export function p95(values: readonly number[],): number {
  if (values.length === 0)
    throw new LifecycleBenchmarkError('Cannot summarize empty latency samples.',);
  /**
   * Ascending sample copy.
   */
  const ordered = sorted(values,);
  /**
   * Fractional percentile rank.
   */
  const percentile = PERCENTILE_NUMERATOR / PERCENTAGE_DENOMINATOR;
  return requiredSample({
    values: ordered,
    index: Math.ceil(ordered.length * percentile,) - 1,
  },);
}
