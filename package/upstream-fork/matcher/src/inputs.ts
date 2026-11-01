/**
 Matcher input and pattern list shapes.
 
 Accepts upstream `matcher`'s call shapes (single string or readonly string
 list for both inputs and patterns) and normalizes them into one readonly
 string list, so the matcher itself never branches on its input shape.
 
 @module
 */

import {
  InvalidInputsError,
  InvalidPatternsError,
} from './errors.ts';

//region Types

/**
 Validated matcher input list.
 */
export type MatcherInputs = {
  /**
   Input strings to filter, with holes and `undefined` entries removed.
   */
  readonly inputs: readonly string[];
};

/**
 Validated matcher pattern list.
 */
export type MatcherPatterns = {
  /**
   Pattern strings to match against, with holes and `undefined` entries removed.
   */
  readonly patterns: readonly string[];
};

//endregion Types

//region Validation

/**
 Normalizes one `string | readonly string[] | undefined` argument into a
 readonly string list.
 
 `undefined` means the argument was omitted and normalizes to an empty list.
 Array holes and explicit `undefined` entries are dropped, matching upstream
 `matcher`'s filter. Any other non-string value (including `null`) throws
 with upstream `matcher`'s message text.
 
 @param value - Candidate inputs or patterns value of unknown runtime type.
 
 @param name - Argument name used in the error message (`inputs` or `patterns`).
 
 @param element - Error message noun for a bad array element (`inputs` or `patterns`).
 
 @returns Normalized readonly string list.
 
 @throws InvalidInputsError when `name` is `inputs` and the value is invalid.
 
 @throws InvalidPatternsError when `name` is `patterns` and the value is invalid.
 
 @example
 ```ts
 sanitizeList('foo', 'inputs', 'inputs'); // => ['foo']
 ```
 */
export function sanitizeList(
  {
    value,
    name,
    element,
  }: {
    readonly value: unknown;
    readonly name: 'inputs' | 'patterns';
    readonly element: 'inputs' | 'patterns';
  },
): readonly string[] {
  if (value === undefined)
    return [];

  if ((typeof value) === 'string')
    return [value];

  if (!Array.isArray(value,)) {
    if (name === 'inputs')
      throw new InvalidInputsError({ value, },);
    throw new InvalidPatternsError({ value, },);
  }

  /**
   Normalized entries with holes and `undefined` entries removed.
   */
  const entries: string[] = [];

  /**
   Own enumerable indices of the array, so holes are skipped exactly as
   upstream `matcher`'s `Array.prototype.filter` skips them.
   */
  const indices = Object.keys(value,);

  for (const key of indices) {
    /**
     Entry at this own index.
     */
    const entry: unknown = value[Number(key,)];
    if (entry === undefined)
      continue;
    if ((typeof entry) !== 'string') {
      if (element === 'inputs')
        throw new InvalidInputsError({ element: entry, },);
      throw new InvalidPatternsError({ element: entry, },);
    }
    entries.push(entry,);
  }

  return entries;
}

/**
 Normalizes the `inputs` argument of `matcher` and `isMatch`.
 
 @param value - Candidate inputs value of unknown runtime type.
 
 @returns Validated inputs record.
 
 @throws InvalidInputsError when the value is invalid.
 
 @example
 ```ts
 sanitizeInputs('foo'); // => { inputs: ['foo'] }
 ```
 */
export function sanitizeInputs(value: unknown,): MatcherInputs {
  return {
    inputs: sanitizeList({
      value,
      name: 'inputs',
      element: 'inputs',
    },),
  };
}

/**
 Normalizes the `patterns` argument of `matcher` and `isMatch`.
 
 @param value - Candidate patterns value of unknown runtime type.
 
 @returns Validated patterns record.
 
 @throws InvalidPatternsError when the value is invalid.
 
 @example
 ```ts
 sanitizePatterns(['a*', '!b']); // => { patterns: ['a*', '!b'] }
 ```
 */
export function sanitizePatterns(value: unknown,): MatcherPatterns {
  return {
    patterns: sanitizeList({
      value,
      name: 'patterns',
      element: 'patterns',
    },),
  };
}

//endregion Validation
