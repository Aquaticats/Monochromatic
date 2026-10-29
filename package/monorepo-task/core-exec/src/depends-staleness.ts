/**
 Unified staleness detection for task-depends.
 
 Both `-s` (source) and `-o` (output) items resolve to timestamps.
 Items can be file globs or `sh:` prefixed shell commands.
 
 Shell commands must output a parseable timestamp on stdout:
 unix epoch (seconds or ms), ISO 8601, `Infinity`, or `-Infinity`.
 Non-zero exit codes and unparseable output are treated as errors,
 preventing silent misinterpretation of unexpected command failures.
 
 File globs resolve to file modification times.
 Empty globs contribute no timestamps (empty array).
 
 Timestamps are aggregated per-side using configurable strategies
 (`newest`/`oldest`), then compared: `sourceTime > outputTime` → stale.
 
 @example
 ```ts
 const stale = await checkStaleness({
   sources: ['src/*.ts'],
   outputs: ['sh:podman image exists img && echo Infinity || echo -Infinity'],
   verbose: false,
   sourceTimeStrategy: 'newest',
   outputTimeStrategy: 'newest',
 });
 ```
 */

import { resolveItems, } from './depends-resolve.ts';
import { aggregateTimestamps, } from './depends-staleness-aggregate.ts';
import type { TimeStrategy, } from './depends-strategy.ts';

export type {
  BuiltinTimeStrategy,
  TimeStrategy,
} from './depends-strategy.ts';

//region Staleness check

/**
 Formats a timestamp for verbose output.
 
 @param t - Timestamp in milliseconds (possibly `Infinity` or `-Infinity`)
 
 @returns ISO 8601 string for finite values, `"Infinity"` or `"-Infinity"` for sentinels
 
 @example
 ```ts
 formatTimestamp(1710000000000) // '2024-03-09T...'
 formatTimestamp(Infinity) // 'Infinity'
 ```
 */
function formatTimestamp(t: number,): string {
  if (!Number.isFinite(t,))
    return String(t,);
  return new Date(t,).toISOString();
}

/**
 Checks whether sources are stale relative to outputs.
 
 Both sources and outputs accept file globs and `sh:` shell commands.
 All items resolve to timestamps via {@link resolveItems} (including `Infinity` and `-Infinity`).
 
 Timestamps are aggregated per-side using the given strategies via {@link aggregateTimestamps},
 then compared: `sourceTime > outputTime` → stale.
 
 When no sources are provided (or all source globs match nothing),
 source time resolves to `-Infinity` ("no information"), making the
 comparison always false (fresh). To trigger rebuilds without file
 sources, use an explicit `sh:` source like `-s "sh:echo Infinity"`.
 
 @param sources - File globs or `sh:` commands for source timestamps
 
 @param outputs - File globs or `sh:` commands for output timestamps
 
 @param verbose - Whether to log diagnostic messages
 
 @param sourceTimeStrategy - Strategy for aggregating source timestamps
 
 @param outputTimeStrategy - Strategy for aggregating output timestamps
 
 @returns `true` when stale (command needs to run)
 
 @throws When a `sh:` command fails or returns unparseable output
 
 @example
 ```ts
 const stale = await checkStaleness({
   sources: ['src/*.ts'],
   outputs: ['dist/*.js'],
   verbose: false,
   sourceTimeStrategy: 'newest',
   outputTimeStrategy: 'newest',
 });
 ```
 */
export async function checkStaleness(
  {
    sources,
    outputs,
    verbose,
    sourceTimeStrategy,
    outputTimeStrategy,
  }: {
    readonly sources: readonly string[];
    readonly outputs: readonly string[];
    readonly verbose: boolean;
    readonly sourceTimeStrategy: TimeStrategy;
    readonly outputTimeStrategy: TimeStrategy;
  },
): Promise<boolean> {
  /**
   Per-source timestamps resolved from globs, files, or shell commands; aggregated below into one source-side time.
   */
  const sourceTimestamps = await resolveItems({
    items: sources,
    position: 'source',
    verbose,
  },);
  /**
   Per-output timestamps resolved from globs, files, or shell commands; aggregated below into one output-side time.
   */
  const outputTimestamps = await resolveItems({
    items: outputs,
    position: 'output',
    verbose,
  },);

  /**
   Single source-side timestamp produced by the configured strategy (max, min, mean, median, ...).
   */
  const sourceTime = await aggregateTimestamps({
    timestamps: sourceTimestamps,
    strategy: sourceTimeStrategy,
    verbose,
  },);
  /**
   Single output-side timestamp produced by the configured strategy; compared against `sourceTime` to decide staleness.
   */
  const outputTime = await aggregateTimestamps({
    timestamps: outputTimestamps,
    strategy: outputTimeStrategy,
    verbose,
  },);

  /**
   True when the source side is newer than the output side, meaning the build is stale and must rerun.
   */
  const stale = sourceTime > outputTime;

  if (verbose) {
    console.error(
      `[task-depends] source: ${
        formatTimestamp(sourceTime,)
      } (${sourceTimeStrategy}), output: ${
        formatTimestamp(outputTime,)
      } (${outputTimeStrategy}) → ${stale ? 'stale' : 'fresh'}`,
    );
  }

  return stale;
}

//endregion Staleness check
