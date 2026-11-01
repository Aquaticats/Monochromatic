/**
 Matcher option parsing and validation.
 
 Accepts upstream `matcher`'s options shapes (absent, `null`, or an options
 object) and normalizes them into one resolved record, so the matcher itself
 never branches on its input shape.
 
 @module
 */

//region Types

/**
 Matching options accepted by `matcher` and `isMatch`.
 
 Mirrors upstream `matcher`'s `Options` with the same defaults.
 
 @example
 ```ts
 const options: MatcherOptions = {
   caseSensitive: true,
   allPatterns: true,
 };
 ```
 */
export type MatcherOptions = {
  /**
   Make matching case-sensitive. When absent or falsy, uppercase and
   lowercase characters match each other.
   
   @defaultValue false
   */
  readonly caseSensitive?: unknown;
  /**
   Require every negated pattern to not match and every normal pattern to
   match at least once.
   
   @defaultValue false
   */
  readonly allPatterns?: unknown;
};

/**
 Validated matching options with every flag present as a boolean.
 
 @example
 ```ts
 const resolved: ResolvedMatcherOptions = {
   caseSensitive: false,
   allPatterns: true,
 };
 ```
 */
export type ResolvedMatcherOptions = {
  /**
   Validated case-sensitivity flag.
   */
  readonly caseSensitive: boolean;
  /**
   Validated all-patterns flag.
   */
  readonly allPatterns: boolean;
};

//endregion Types

//region Resolution

/**
 Own-property key for the case-sensitivity flag.
 */
const caseSensitiveKey = 'caseSensitive';

/**
 Own-property key for the all-patterns flag.
 */
const allPatternsKey = 'allPatterns';

/**
 Normalizes the options argument accepted by `matcher` and `isMatch` into
 one validated record.
 
 `undefined` and `null` both mean defaults. Any other value contributes
 only its own flag properties (prototype pollution never enables a flag,
 matching upstream `matcher`'s spread plus its pollution regression test),
 then each flag is coerced with the same truthiness test: only truthy flag
 values enable the flag.
 
 @param options - Options value of unknown runtime type.
 
 @returns Validated options with every flag present as a boolean.
 
 @example
 ```ts
 resolveMatcherOptions({ caseSensitive: 1, });
 // => { caseSensitive: true, allPatterns: false }
 ```
 */
export function resolveMatcherOptions(options: unknown,): ResolvedMatcherOptions {
  if ((options === undefined) || (options === null))
    return {
      caseSensitive: false,
      allPatterns: false,
    };

  /**
   Own flag values only, matching upstream `matcher`'s `{...defaults,
   ...options}` spread. The names are read through `Object.hasOwn` so
   prototype pollution never enables a flag (upstream's own test pins
   `SECRET.txt` matching `secret*` under a polluted prototype).
   */
  /**
   Options record whose own flag properties decide the verdict. A string-
   keyed index read keeps the lookup own-only without member syntax.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- options is an external untyped value; the record view only gates own-property reads
  const record = options as Record<string, unknown>;
  /**
   Own `caseSensitive` flag value, ignoring the prototype chain.
   */
  const caseSensitive = Object.hasOwn(
    record,
    caseSensitiveKey,
  )
    ? record[caseSensitiveKey]
    : undefined;
  /**
   Own `allPatterns` flag value, ignoring the prototype chain.
   */
  const allPatterns = Object.hasOwn(
    record,
    allPatternsKey,
  )
    ? record[allPatternsKey]
    : undefined;

  return {
    caseSensitive: Boolean(caseSensitive,),
    allPatterns: Boolean(allPatterns,),
  };
}

//endregion Resolution
