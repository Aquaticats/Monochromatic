/**
 Strategy aggregation for task-depends staleness detection.
 
 Dispatches timestamp arrays to builtin strategy functions or custom
 shell commands, producing a single representative timestamp per side.
 
 @module
 */

import spawn from 'nano-spawn';
import dedent from 'string-dedent';

import {
  builtinStrategies,
  type BuiltinTimeStrategy,
  type TimeStrategy,
} from './depends-strategy.ts';

//region Shell strategy execution

/**
 Options for {@link runStrategyCommand}.
 
 @example
 ```ts
 const options: RunStrategyCommandOptions = {
   command: 'sort -rn | head -1',
   timestamps: [1, 2, 3],
   verbose: false,
 };
 ```
 */
type RunStrategyCommandOptions = {
  /**
   Shell command (without `sh:` prefix)
   */
  readonly command: string;
  /**
   Resolved timestamps to pipe via stdin
   */
  readonly timestamps: readonly number[];
  /**
   Whether to log diagnostic messages
   */
  readonly verbose: boolean;
};

/**
 Runs a custom shell strategy command with timestamps piped via stdin.
 
 Timestamps are piped to the command as newline-separated values on stdin
 using `printf`. The command must output a single parseable timestamp
 on stdout. Common Unix patterns work naturally:
 - `sort -n | head -1` for minimum (oldest)
 - `sort -rn | head -1` for maximum (newest)
 
 @param command - Shell command (without `sh:` prefix)
 
 @param timestamps - Resolved timestamps to pipe via stdin
 
 @param verbose - Whether to log diagnostic messages
 
 @returns Aggregated timestamp from command stdout
 
 @throws When command fails or returns unparseable output
 
 @example
 ```ts
 await runStrategyCommand({ command: 'sort -rn | head -1', timestamps: [1, 2, 3], verbose: false }) // 3
 ```
 */
async function runStrategyCommand({
  command,
  timestamps,
  verbose,
}: RunStrategyCommandOptions,): Promise<number> {
  /**
   Space-separated stringified timestamps suitable for embedding in a `printf` argument list.
   */
  const formattedValues = timestamps.map(function stringifyTimestamp(timestamp,): string {
    return `${timestamp}`;
  },)
    .join(' ',);
  // Use printf to pipe timestamps (one per line) into the strategy command via stdin
  /**
   Full pipeline executed via `sh`, piping one timestamp per line into the user's strategy command.
   */
  const fullCommand = `printf '%s\\n' ${formattedValues} | ${command}`;

  if (verbose)
    console.error(`[task-depends] running strategy command: ${fullCommand}`,);

  /**
   Raw stdout from the command
   */
  let stdout = '';
  try {
    /**
     Captured subprocess result; only `stdout` is consumed because the strategy contract returns its answer there.
     */
    const result = await spawn(
      fullCommand,
      { shell: true, },
    );
    stdout = result.stdout
      .trim();
  }
  catch (error) {
    throw new Error(
      dedent`
        Strategy sh: "${command}" failed with non-zero exit code
        Strategy commands must succeed and output a single timestamp
      `,
      { cause: error, },
    );
  }

  // Strategy commands receive and return millisecond timestamps.
  // Parse as raw number (no seconds/ms heuristic) or Infinity/-Infinity.
  /**
   Aggregated timestamp parsed from stdout
   */
  let result = 0;
  if (stdout === 'Infinity')
    result = Infinity;
  else if (stdout === '-Infinity')
    result = -Infinity;
  else {
    /**
     Numeric coercion of the strategy's stdout; rejected as unparseable when NaN.
     */
    const num = Number(stdout,);
    if (Number.isNaN(num,)) {
      throw new Error(
        dedent`
          Strategy sh: "${command}" returned unparseable output: "${stdout}"
          Strategy commands receive millisecond timestamps and must return a millisecond timestamp, Infinity, or -Infinity
        `,
      );
    }
    result = num;
  }

  if (verbose) {
    /**
     ISO timestamp for finite values, raw `Infinity`/`-Infinity` string otherwise; only used for human-readable logging.
     */
    const display = Number.isFinite(result,)
      ? new Date(result,).toISOString()
      : String(result,);
    console.error(`[task-depends] strategy sh: "${command}" → ${display}`,);
  }

  return result;
}

//endregion Shell strategy execution

//region Aggregation dispatch

/**
 Options for {@link aggregateTimestamps}.
 
 @example
 ```ts
 const options: AggregateTimestampsOptions = {
   timestamps: [1, 2, 3],
   strategy: 'newest',
   verbose: false,
 };
 ```
 */
export type AggregateTimestampsOptions = {
  /**
   Resolved timestamps to aggregate
   */
  readonly timestamps: readonly number[];
  /**
   Builtin strategy name or `sh:` command
   */
  readonly strategy: TimeStrategy;
  /**
   Whether to log diagnostic messages
   */
  readonly verbose: boolean;
};

/**
 Aggregates timestamps using the given strategy.
 
 Dispatches to a {@link builtinStrategies} function or runs a custom shell command via {@link runStrategyCommand}.
 
 @param timestamps - Resolved timestamps to aggregate
 
 @param strategy - Builtin strategy name or `sh:` command
 
 @param verbose - Whether to log diagnostic messages
 
 @returns Single aggregated timestamp
 
 @example
 ```ts
 await aggregateTimestamps({ timestamps: [1, 2, 3], strategy: 'newest', verbose: false }) // 3
 ```
 */
export async function aggregateTimestamps({
  timestamps,
  strategy,
  verbose,
}: AggregateTimestampsOptions,): Promise<number> {
  if (strategy in builtinStrategies) {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- 'in' check above narrows strategy to BuiltinTimeStrategy
    return builtinStrategies[strategy as BuiltinTimeStrategy](timestamps,);
  }

  /**
   Prefix that identifies a shell command strategy
   */
  const shPrefix = 'sh:';
  if (strategy.startsWith(shPrefix,)) {
    if (timestamps.length
      === 0)
      return -Infinity;
    return await runStrategyCommand({
      command: strategy.slice(shPrefix.length,),
      timestamps,
      verbose,
    },);
  }

  throw new Error(`Unknown time strategy: "${strategy}"`,);
}

//endregion Aggregation dispatch
