/**
 Narrowing helpers for materialized TOML values.

 @module
 */

/**
 Sentinel for a TOML value that is not a number.
 */
export const NOT_A_NUMBER: unique symbol = Symbol('TOML value is neither an integer nor a float',);

/**
 Whether `value` is a plain string-keyed table.

 @param value - materialized TOML value

 @returns whether it can be read as a table

 @example
 ```ts
 isTable({ version: '>=1', }); // true
 ```
 */
export function isTable(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Reads a materialized TOML integer or float as a JavaScript number.

 `module-toml-edit` materializes the minute counts this catalog accepts as
 plain numbers; anything else is reported as not numeric.

 @param value - materialized TOML value

 @returns number, or {@link NOT_A_NUMBER} when the value is not numeric

 @example
 ```ts
 toNumber(1440); // 1440
 ```
 */
export function toNumber(value: unknown,): number | typeof NOT_A_NUMBER {
  if ((typeof value) === 'number')
    return value;
  return NOT_A_NUMBER;
}

/**
 Whether `value` is a non-empty string.

 @param value - materialized TOML value

 @returns whether it is a string with at least one character

 @example
 ```ts
 isNonEmptyString('abc'); // true
 ```
 */
export function isNonEmptyString(value: unknown,): value is string {
  return ((typeof value) === 'string') && (value.length > 0);
}
