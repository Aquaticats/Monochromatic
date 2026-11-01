#!/usr/bin/env node

/**
 Make-style dependency checker for mise tasks.
 
 Checks staleness via file modification times and/or shell command probes,
 then runs the given command only when dependencies are stale.
 Output from the command is collapsed (hidden on success, shown on failure).
 
 Both `-s` and `-o` accept file globs or `sh:` prefixed shell commands.
 Shell commands must output a parseable timestamp on stdout:
 unix epoch (seconds or ms), ISO 8601, `Infinity`, or `-Infinity`.
 Non-zero exit codes are treated as errors (not silent staleness signals).
 
 Timestamps are aggregated per-side using configurable strategies
 (`--source-time-strategy`, `--output-time-strategy`), then compared:
 `sourceTime > outputTime` → stale.
 
 @example
 ```bash
 # File-based
 task-depends -s "src/*.ts" -o "dist/*.js"; mise run build
 
 # Command-based output check (gate pattern)
 task-depends -o "sh:podman image exists img && echo Infinity || echo -Infinity"; podman build .
 
 # Timestamp from command
 task-depends -s "sh:git log -1 --format=%ct" -o "dist/*.js"; mise run build
 
 # Catch missing outputs in mixed lists with oldest strategy
 task-depends --output-time-strategy oldest -s "src/**" -o "dist/**" -o "sh:..."; mise run build
 ```
 */

// TODO: deprecate Optique
import { object, } from '@optique/core/constructs';
// TODO: deprecate Optique
import {
  multiple,
  optional,
} from '@optique/core/modifiers';
// TODO: deprecate Optique
import {
  argument,
  option,
} from '@optique/core/primitives';
// TODO: deprecate Optique
import { string, } from '@optique/core/valueparser';
// TODO: deprecate Optique
import { runSync, } from '@optique/run';
import dedent from 'string-dedent';

import { executeWithCollapsedOutput, } from './depends-exec.ts';
import {
  type BuiltinTimeStrategy,
  checkStaleness,
  type TimeStrategy,
} from './depends-staleness.ts';



//region Parser definition

/**
 TODO: deprecate Optique
 Optique parser for the task-depends CLI
 */
const parser = object({
  sources: multiple(
    optional(option(
      '-s',
      '--sources',
      string(),
    ),),
  ),
  outputs: multiple(
    optional(option(
      '-o',
      '--outputs',
      string(),
    ),),
  ),
  sourceTimeStrategy: optional(option(
    '--source-time-strategy',
    string(),
  ),),
  outputTimeStrategy: optional(option(
    '--output-time-strategy',
    string(),
  ),),
  allowFailure: option(
    '-a',
    '--allowFailure',
  ),
  verbose: option(
    '-v',
    '--verbose',
  ),
  rest: multiple(
    argument(string(),),
  ),
},);

//endregion Parser definition

//region Argument validation

/**
 TODO: deprecate Optique
 Parsed CLI arguments from process.argv
 */
const rawArgs = runSync(
  parser,
  {
    programName: 'task-depends',
    help: 'option',
  },
);

// TODO: deprecate Optique
/* oxlint-disable no-restricted-syntax/no-nullish-union -- external boundary: @optique/core `optional()` is typed `Parser<…, TValue | undefined, …>`, so `multiple(optional(...))` yields `(string | undefined)[]` for omitted options; this helper mirrors that upstream type to strip the absent entries */
/**
 Filters absent values produced by `multiple(optional(...))` when an option is omitted.
 
 optique returns `undefined` for omitted optional options inside `multiple`.
 
 @param values - Array that may contain `undefined` from optique parsing
 
 @returns Array with absent values removed
 
 @example
 ```ts
 filterNullish([undefined, 'a', undefined, 'b']) // ['a', 'b']
 ```
 */
function filterNullish(values: readonly (string | undefined)[],): string[] {
  return values.filter(function isString(value,): value is string {
    return value !== undefined;
  },);
}
/* oxlint-enable no-restricted-syntax/no-nullish-union */

/**
 Valid builtin time strategy names
 */
const BUILTIN_STRATEGIES: ReadonlySet<BuiltinTimeStrategy> = new Set([
  'newest',
  'oldest',
  'mean',
  'median',
],);

/**
 Options for {@link validateTimeStrategy}.
 
 @example
 ```ts
 const options: ValidateTimeStrategyOptions = {
   value: 'oldest',
   flagName: '--source-time-strategy',
 };
 ```
 */
type ValidateTimeStrategyOptions = {
  // TODO: deprecate Optique
  /* oxlint-disable no-restricted-syntax/no-nullish-union -- external boundary: @optique/core `optional()` is typed `Parser<…, TValue | undefined, …>`, so an omitted `--*-time-strategy` arrives here as `undefined`; this field mirrors that upstream type */
  /**
   Raw value from optique (`undefined` when option is omitted)
   */
  readonly value: string | undefined;
  /* oxlint-enable no-restricted-syntax/no-nullish-union */
  /**
   Flag name for error messages
   */
  readonly flagName: string;
};

/**
 Validates and defaults a time strategy option.
 
 Accepts builtin strategy names from {@link BUILTIN_STRATEGIES} or `sh:` prefixed shell commands.
 
 @param value - Raw value from optique (possibly nullish when option is omitted)
 
 @param flagName - Flag name for error messages
 
 @returns Validated strategy, defaulting to `'newest'`
 
 @throws When value is not a valid builtin and does not start with `sh:`
 
 @example
 ```ts
 validateTimeStrategy({ value: undefined, flagName: '--source-time-strategy' }) // 'newest'
 validateTimeStrategy({ value: 'oldest', flagName: '--output-time-strategy' }) // 'oldest'
 validateTimeStrategy({ value: 'sh:my-script', flagName: '--source-time-strategy' }) // 'sh:my-script'
 ```
 */
function validateTimeStrategy({
  value,
  flagName,
}: ValidateTimeStrategyOptions,): TimeStrategy {
  if (value === undefined)
    return 'newest';
  /* oxlint-disable typescript/no-unsafe-type-assertion -- value is validated by Set.has / sh: prefix check before each cast below */
  if (BUILTIN_STRATEGIES.has(value as BuiltinTimeStrategy,))
    return value as TimeStrategy;
  if (value.startsWith('sh:',))
    return value as TimeStrategy;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  throw new Error(
    `Invalid ${flagName}: "${value}". Must be a builtin (newest, oldest, mean, median) or sh:command.`,
  );
}

/**
 Cleaned CLI arguments with nullish values filtered out
 */
const args = {
  sources: filterNullish(rawArgs.sources,),
  outputs: filterNullish(rawArgs.outputs,),
  sourceTimeStrategy: validateTimeStrategy({
    value: rawArgs.sourceTimeStrategy,
    flagName: '--source-time-strategy',
  },),
  outputTimeStrategy: validateTimeStrategy({
    value: rawArgs.outputTimeStrategy,
    flagName: '--output-time-strategy',
  },),
  allowFailure: rawArgs.allowFailure,
  verbose: rawArgs.verbose,
  rest: rawArgs.rest,
};

/**
 Destructured command and its arguments from the rest args after `--`
 */
const [command, ...commandArgs] = args.rest;

if ((command === undefined) || (command === '')) {
  throw new Error(
    dedent`
      No command specified after --
      Usage: task-depends -s "src/*" -o "dist/*" -- command args...
      Usage: task-depends -o "sh:podman image exists img" -- command args...
    `,
  );
}

if (args.outputs
  .length
  === 0) {
  throw new Error(
    'At least one -o is required (defines what "done" looks like: file glob or sh:command)',
  );
}

//endregion Argument validation

//region Staleness check and execution

/**
 Whether sources are newer than outputs, indicating the command needs to run
 */
const stale = await checkStaleness({
  sources: args.sources,
  outputs: args.outputs,
  verbose: args.verbose,
  sourceTimeStrategy: args.sourceTimeStrategy,
  outputTimeStrategy: args.outputTimeStrategy,
},);

if (stale) {
  await executeWithCollapsedOutput({
    command,
    commandArgs,
    verbose: args.verbose,
    allowFailure: args.allowFailure,
  },);
}
else if (args.verbose) {
  console.error('[task-depends] all checks passed, skipping command',);
}

//endregion Staleness check and execution
