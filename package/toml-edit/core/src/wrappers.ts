/**
 Factory functions producing tagged inputs for {@link tomlSet}.
 
 Tagged objects are how a caller disambiguates JS-to-TOML conversions
 where JS values are ambiguous (e.g. a JS `Date` cannot represent
 `local-date-time` distinctly from `offset-date-time`).
 
 @module
 */

import type { TomlWrappedInput, } from './types.ts';

/**
 Wrap an integer for {@link tomlSet}. Forces integer emission even when the
 inferred kind from a bare JS `number` would be float.
 
 @param value - Integer to emit; accepts `number` and `bigint`.
 
 @returns Tagged wrapper that {@link tomlSet} recognises as TOML integer.
 
 @example
 ```ts
 tomlSet({ edit, path: ['count',], value: tomlInteger(42,), },);
 ```
 */
export function tomlInteger(value: number | bigint,): TomlWrappedInput {
  return {
    tomlKind: 'integer',
    value,
  };
}

/**
 Wrap a float for {@link tomlSet}. Forces float emission even when the value is
 an integer in JS.
 
 @param value - Number to emit as a TOML float.
 
 @returns Tagged wrapper that {@link tomlSet} recognises as TOML float.
 
 @example
 ```ts
 tomlSet({ edit, path: ['ratio',], value: tomlFloat(1,), },);  // emits `1.0`
 ```
 */
export function tomlFloat(value: number,): TomlWrappedInput {
  return {
    tomlKind: 'float',
    value,
  };
}

/**
 Wrap a TOML local-date (e.g. `'2026-05-14'`).
 
 @param value - ISO-style local-date string (no time component).
 
 @returns Tagged wrapper that {@link tomlSet} recognises as TOML local-date.
 
 @example
 ```ts
 tomlSet({ edit, path: ['birthday',], value: tomlLocalDate('2026-05-14',), },);
 ```
 */
export function tomlLocalDate(value: string,): TomlWrappedInput {
  return {
    tomlKind: 'local-date',
    value,
  };
}

/**
 Wrap a TOML local-date-time (e.g. `'2026-05-14T10:00:00'`).
 
 @param value - ISO-style local-date-time string (no offset).
 
 @returns Tagged wrapper that {@link tomlSet} recognises as TOML local-date-time.
 
 @example
 ```ts
 tomlSet({ edit, path: ['meeting',], value: tomlLocalDateTime('2026-05-14T10:00:00',), },);
 ```
 */
export function tomlLocalDateTime(value: string,): TomlWrappedInput {
  return {
    tomlKind: 'local-date-time',
    value,
  };
}

/**
 Wrap a TOML local-time (e.g. `'10:00:00'`).
 
 @param value - ISO-style local-time string (no date component).
 
 @returns Tagged wrapper that {@link tomlSet} recognises as TOML local-time.
 
 @example
 ```ts
 tomlSet({ edit, path: ['alarm',], value: tomlLocalTime('07:30:00',), },);
 ```
 */
export function tomlLocalTime(value: string,): TomlWrappedInput {
  return {
    tomlKind: 'local-time',
    value,
  };
}
