/**
 Generic input validation and parsing helpers for API handlers.
 
 Pure functions with no task-domain coupling. Task-specific parsing
 lives in task-validation-update.ts.
 */

/**
 Narrows `unknown` to a plain object for property access.
 
 @param value - Value to check
 
 @returns True when value is a non-null object
 
 @example
 ```ts
 if (!isRecord(body)) return null;
 ```
 */
export function isRecord(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 Sentinel returned by the parsers below when untrusted input fails validation.
 
 A unique `Symbol` keeps "invalid / not the expected shape" out of a nullish
 union (banned by `no-nullish-union`); callers narrow with `=== INVALID`.
 */
export const INVALID: unique symbol = Symbol('untrusted request input failed shape validation',);

/**
 Extracts a trimmed, non-empty string array from untrusted input.
 
 @param value - Raw input that may be an array
 
 @returns Parsed array, or {@link INVALID} when the input is not an array
 
 @example
 ```ts
 const tags = parseStringArray(body.tags); // ['shopping', 'errands'] or INVALID
 ```
 */
export function parseStringArray(value: unknown,): string[] | typeof INVALID {
  if (!Array.isArray(value,))
    return INVALID;

  return value
    .filter(function isString(entry,): entry is string {
      return (typeof entry) === 'string';
    },)
    .map(function trimEntry(entry,) {
      return entry.trim();
    },)
    .filter(function isNonEmpty(entry,) {
      return entry.length
        > 0;
    },);
}

/**
 Parses an enum field from untrusted input against a set of recognised values.
 
 Callers handle an explicit `null` (treated as "field not provided") before
 calling, so a `null` reaching here is just another non-string and maps to
 {@link INVALID}.
 
 @param value - Raw input value
 
 @param validValues - Set of recognized enum strings
 
 @returns Validated string, or {@link INVALID} when absent or unrecognized
 
 @example
 ```ts
 const priority = parseEnumValue({ value: body.priority, validValues: validPriorities });
 ```
 */
export function parseEnumValue(
  {
    value,
    validValues,
  }: {
    readonly value: unknown;
    readonly validValues: ReadonlySet<string>;
  },
): string | typeof INVALID {
  if ((typeof value) !== 'string')
    return INVALID;
  return validValues.has(value,) ? value : INVALID;
}
