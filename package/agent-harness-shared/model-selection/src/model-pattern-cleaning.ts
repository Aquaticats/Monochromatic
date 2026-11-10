/**
 Model-pattern cleanup shared by argv and settings scope input.

 The argv `--models` list and the settings `enabledModels` array reach scope
 resolution through this one pipeline, so a policy change such as also
 dropping patterns that are only whitespace lands in one place.

 @module
 */

//region Cleaning

/**
 Trim every configured model pattern and drop the empty results.

 @param patterns - raw model patterns from argv or settings

 @returns trimmed, non-empty model patterns in input order

 @example
 ```typescript
 cleanModelPatterns([' cheap/* ', '', '   ']); // ['cheap/*']
 ```

 @internal
 */
export function cleanModelPatterns(
  patterns: readonly string[],
): string[] {
  return patterns
    .map(function trimPattern(pattern,) {
      return pattern.trim();
    },)
    .filter(function keepPattern(pattern,) {
      return pattern !== '';
    },);
}

//endregion Cleaning
