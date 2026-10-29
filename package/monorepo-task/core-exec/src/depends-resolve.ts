/**
 Item resolution pipeline for task-depends staleness detection.
 
 Resolves source/output items (file globs and `sh:` shell commands)
 to arrays of timestamps. File globs resolve to file modification times;
 shell commands must output a parseable timestamp on stdout.
 
 @module
 */

import { stat, } from 'node:fs/promises';

import spawn from 'nano-spawn';
import dedent from 'string-dedent';

import {
  extractCommand,
  isShellCommand,
  parseTimestamp,
  UNPARSEABLE_TIMESTAMP,
} from './depends-parse.ts';
import { resolveGlobFiles, } from './depends-resolve-glob.ts';

//region Item resolution: resolve individual items to timestamps

/**
 Position label distinguishing source items (cause staleness) from output items (compared against source).
 
 @example
 ```ts
 const pos: ItemPosition = 'source';
 ```
 */
type ItemPosition = 'source' | 'output';

/**
 Options for {@link resolveGlob}.
 
 @example
 ```ts
 const options: ResolveGlobOptions = {
   pattern: 'src/*.ts',
   position: 'source',
   verbose: false,
 };
 ```
 */
type ResolveGlobOptions = {
  /**
   File glob pattern
   */
  readonly pattern: string;
  /**
   Whether this is a source or output item (for logging)
   */
  readonly position: ItemPosition;
  /**
   Whether to log diagnostic messages
   */
  readonly verbose: boolean;
};

/**
 Resolves a file glob to an array of file mtimes via {@link resolveGlobFiles}.
 
 When the glob matches no files, returns an empty array.
 Empty arrays contribute no timestamps to the strategy aggregation,
 which returns `-Infinity` ("no information") for empty input.
 
 @param pattern - File glob pattern
 
 @param position - Whether this is a source or output item (for logging)
 
 @param verbose - Whether to log diagnostic messages
 
 @returns Array of file mtimes in milliseconds, or empty when no files match
 
 @example
 ```ts
 const mtimes = await resolveGlob({ pattern: 'src/*.ts', position: 'source', verbose: false });
 ```
 */
async function resolveGlob({
  pattern,
  position,
  verbose,
}: ResolveGlobOptions,): Promise<number[]> {
  /**
   Files matched by the glob; an empty result yields an empty timestamp set rather than an error.
   */
  const files = await resolveGlobFiles(pattern,);

  if (files.length
    === 0) {
    if (verbose)
      console.error(`[task-depends] ${position} glob "${pattern}" matched no files`,);
    return [];
  }

  /**
   `fs.stat` results for every matched file; awaited concurrently so I/O latency overlaps.
   */
  const stats = await Promise.all(files.map(function statFile(file,) {
    return stat(file,);
  },),);
  /**
   Modification times in milliseconds, projected from the stat results so the caller does not need stat metadata.
   */
  const mtimes = stats.map(function extractMtime(fileStat: Readonly<{ mtimeMs: number; }>,): number {
    return fileStat.mtimeMs;
  },);

  if (verbose) {
    console.error(
      `[task-depends] ${position} glob "${pattern}" matched ${files.length} files`,
    );
  }

  return mtimes;
}

/**
 Options for {@link resolveShellCommand}.
 
 @example
 ```ts
 const options: ResolveShellCommandOptions = {
   command: 'git log -1 --format=%ct',
   position: 'source',
   verbose: false,
 };
 ```
 */
type ResolveShellCommandOptions = {
  /**
   Shell command to execute (without `sh:` prefix)
   */
  readonly command: string;
  /**
   Whether this is a source or output item (for error messages)
   */
  readonly position: ItemPosition;
  /**
   Whether to log diagnostic messages
   */
  readonly verbose: boolean;
};

/**
 Resolves a shell command to a single timestamp.
 
 Commands must output a parseable timestamp on stdout:
 unix epoch (seconds or ms), ISO 8601, `Infinity`, or `-Infinity`.
 
 Non-zero exit codes throw an error rather than being silently
 interpreted, preventing subtle bugs when commands fail unexpectedly.
 
 @param command - Shell command to execute (without `sh:` prefix)
 
 @param position - Whether this is a source or output item (for error messages)
 
 @param verbose - Whether to log diagnostic messages
 
 @returns Resolved timestamp in milliseconds (possibly `Infinity` or `-Infinity`)
 
 @throws When command exits with non-zero code or {@link parseTimestamp} cannot parse stdout (see {@link UNPARSEABLE_TIMESTAMP})
 
 @example
 ```ts
 // Timestamp from stdout
 await resolveShellCommand({ command: 'git log -1 --format=%ct', position: 'source', verbose: false })
 // Gate pattern: explicit Infinity/-Infinity
 await resolveShellCommand({ command: 'podman image exists img && echo Infinity || echo -Infinity', position: 'output', verbose: false })
 ```
 */
async function resolveShellCommand({
  command,
  position,
  verbose,
}: ResolveShellCommandOptions,): Promise<number> {
  /**
   Raw stdout from the command
   */
  let stdout = '';

  try {
    /**
     Captured subprocess result; only `stdout` is consumed because the command contract returns its timestamp there.
     */
    const result = await spawn(
      command,
      { shell: true, },
    );
    stdout = result.stdout
      .trim();
  }
  catch (error) {
    throw new Error(
      dedent`
        ${position} sh: "${command}" failed with non-zero exit code
        Commands must succeed and output a timestamp (unix epoch, ISO 8601, Infinity, or -Infinity)
        Example: sh:${command} && echo Infinity || echo -Infinity
      `,
      { cause: error, },
    );
  }

  /**
   Numeric timestamp parsed from the command's stdout; `UNPARSEABLE_TIMESTAMP` triggers the unparseable-output error path.
   */
  const parsed = parseTimestamp(stdout,);
  if (parsed === UNPARSEABLE_TIMESTAMP) {
    throw new Error(
      dedent`
        ${position} sh: "${command}" returned unparseable output: "${stdout}"
        Commands must output a timestamp (unix epoch, ISO 8601, Infinity, or -Infinity)
      `,
    );
  }

  if (verbose) {
    /**
     ISO timestamp for finite values, raw `Infinity`/`-Infinity` string otherwise; only used for human-readable logging.
     */
    const display = Number.isFinite(parsed,)
      ? new Date(parsed,).toISOString()
      : String(parsed,);
    console.error(
      `[task-depends] ${position} sh: "${command}" → ${display}`,
    );
  }

  return parsed;
}

/**
 Options for {@link resolveItems}.
 
 @example
 ```ts
 const options: ResolveItemsOptions = {
   items: ['src/*.ts', 'sh:git log -1 --format=%ct'],
   position: 'source',
   verbose: false,
 };
 ```
 */
export type ResolveItemsOptions = {
  /**
   Array of glob patterns and/or `sh:` commands
   */
  readonly items: readonly string[];
  /**
   Whether these are source or output items
   */
  readonly position: ItemPosition;
  /**
   Whether to log diagnostic messages
   */
  readonly verbose: boolean;
};

/**
 Resolves an array of source or output items to timestamps.
 
 Each item is either a file glob (resolved via {@link resolveGlob} to file mtimes) or a `sh:` prefixed
 shell command (resolved via {@link resolveShellCommand} to a single timestamp from stdout).
 
 @param items - Array of glob patterns and/or `sh:` commands
 
 @param position - Whether these are source or output items
 
 @param verbose - Whether to log diagnostic messages
 
 @returns Flat array of all resolved timestamps
 
 @example
 ```ts
 const ts = await resolveItems({ items: ['src/*.ts', 'sh:git log -1 --format=%ct'], position: 'source', verbose: false });
 ```
 */
export async function resolveItems({
  items,
  position,
  verbose,
}: ResolveItemsOptions,): Promise<number[]> {
  /**
   Per-item timestamp arrays awaited concurrently; flattened below into a single timestamp set.
   */
  const results = await Promise.all(
    items.map(async function resolveItem(item,): Promise<number[]> {
      if (isShellCommand(item,)) {
        /**
         Single timestamp produced by a `sh:` command; wrapped in an array so the outer `flat()` call sees a uniform shape.
         */
        const ts = await resolveShellCommand({
          command: extractCommand(item,),
          position,
          verbose,
        },);
        return [ts,];
      }
      return resolveGlob({
        pattern: item,
        position,
        verbose,
      },);
    },),
  );

  return results.flat();
}

//endregion Item resolution
