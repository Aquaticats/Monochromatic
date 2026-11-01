/**
 CLI argv helpers for reconstructing pi's `--models` scope.
 
 @module
 */

/**
 Sentinel returned by {@link parseArgvModelPatterns} when no `--models` flag
 is present in argv. A `unique symbol`; callers narrow with
 `=== NO_ARGV_MODELS`. Exported because `scope-resolver` consumes it across
 the module seam. An empty `--models` value yields `[]`, not this sentinel.
 */
export const NO_ARGV_MODELS: unique symbol = Symbol('model-selection/no-argv-models',);

//region Public API

/**
 Options for parsing `--models` from argv.
 */
export type ParseArgvModelsOptions = {
  /**
   Process arguments excluding or including executable prefix.
   */
  readonly argv: readonly string[];
};

/**
 Parse pi's `--models` argument from an argv array.
 
 Supports `--models value` and `--models=value`, matching pi's documented
 comma-separated format.
 
 @param options - argv to inspect
 
 @returns cleaned patterns, or {@link NO_ARGV_MODELS} when `--models` is absent
 
 @example
 ```typescript
 parseArgvModelPatterns({ argv: ['--models', 'claude-*,gpt-*'] });
 ```
 */
export function parseArgvModelPatterns(
  options: ParseArgvModelsOptions,
): string[] | typeof NO_ARGV_MODELS {
  /**
   Inline `--models=value` argument, if present.
   */
  const inline = options.argv
    .find(function isInlineModelsArg(arg,) {
      return arg.startsWith('--models=',);
    },);
  if (inline !== undefined)
    return splitPatterns(inline.slice('--models='.length,),);

  /**
   Index of `--models` when supplied as a separate argument.
   */
  const modelsIndex = options.argv
    .indexOf('--models',);
  if (modelsIndex === (-1))
    return NO_ARGV_MODELS;

  /**
   Value following `--models`.
   */
  const value = options.argv[modelsIndex + 1];
  if (value === undefined)
    return [];
  return splitPatterns(value,);
}

//endregion Public API

//region Internal helpers

/**
 Split a comma-separated model pattern string.
 
 @param value - comma-separated model pattern string
 
 @returns cleaned model patterns
 */
function splitPatterns(
  value: string,
): string[] {
  return value
    .split(',',)
    .map(function trimPattern(pattern,) {
      return pattern.trim();
    },)
    .filter(function keepPattern(pattern,) {
      return pattern !== '';
    },);
}

//endregion Internal helpers
