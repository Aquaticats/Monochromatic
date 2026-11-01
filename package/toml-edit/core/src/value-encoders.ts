/**
 Leaf encoders and type guards for JS-to-TOML coercion.
 
 Split out of `values.ts` to keep each file under the 300-LOC cap. None of
 these helpers recurse back into {@link encodeValue}; they are pure leaves.
 
 @module
 */

import {
  escapeBasicMultiline,
  escapeBasicSingleLine,
} from './basic-escape.ts';
import type { TomlWrappedInput, } from './types.ts';

/**
 Encode a tagged wrapper input ({@link tomlInteger}, {@link tomlFloat}, date kinds).
 
 @returns Computed string.
 
 @example
 ```ts
 encodeWrapped({ wrapped: { tomlKind: 'integer', value: 7n, }, },); // '7'
 ```
 */
export function encodeWrapped(
  { wrapped, }: { readonly wrapped: TomlWrappedInput; },
): string {
  if (wrapped.tomlKind
    === 'float') {
    /**
     Numeric form so finiteness and NaN can be checked.
     */
    const n = Number(wrapped.value,);
    if (!Number.isFinite(n,)) {
      if (Number.isNaN(n,))
        return 'nan';
      return n > 0 ? 'inf' : '-inf';
    }
    /**
     String form so the float-marker check can scan once.
     */
    const s = String(n,);
    return s.includes('.',)
      || s
        .includes('e',)
      || s
        .includes('E',) ? s : `${s}.0`;
  }
  return String(wrapped.value,);
}

/**
 Encode a string with explicit `style` and `multiline` choices.
 
 @returns Computed string.
 
 @example
 ```ts
 encodeStringWithStyle({ value: 'hi', style: 'basic', multiline: false, },); // '"hi"'
 ```
 */
export function encodeStringWithStyle(
  {
    value,
    style,
    multiline,
  }: {
    readonly value: string;
    readonly style: 'basic' | 'literal';
    readonly multiline: boolean;
  },
): string {
  if (style === 'literal') {
    if (multiline)
      return `'''\n${value}'''`;
    return `'${value}'`;
  }
  if (multiline)
    return `"""\n${escapeBasicMultiline({ value, },)}"""`;
  return `"${escapeBasicSingleLine({ value, },)}"`;
}

/**
 Type guard for tagged wrapper inputs produced by `wrappers.ts`.
 
 @param value - Arbitrary JS value to test.
 
 @returns True when `value` is an object carrying a string `tomlKind` discriminant.
 
 @example
 ```ts
 isWrappedInput({ tomlKind: 'float', value: 1, },); // true
 ```
 */
export function isWrappedInput(value: unknown,): value is TomlWrappedInput {
  return (
    ((typeof value) === 'object')
      && (value !== null)
      && ('tomlKind' in value)
      && ((typeof (value as { tomlKind: unknown; }).tomlKind) === 'string')
  );
}

/**
 Type guard for plain object literals (proto is `Object.prototype` or `null`).
 
 @param value - Arbitrary JS value to test.
 
 @returns True when `value` is a plain object literal (excludes `Date`, class
          instances, `Map`, `Set`, etc.).
 
 @mutates value - `Object.getPrototypeOf` can invoke caller-owned proxy prototype hooks.
 
 @example
 ```ts
 isPlainObject({ a: 1, },);    // true
 isPlainObject(new Date(),);   // false
 isPlainObject([1, 2, 3,],);   // false (Array.prototype)
 ```
 */
export function isPlainObject(value: unknown,): value is Record<string, unknown> {
  if (((typeof value) !== 'object') || (value === null))
    return false;
  /**
   Prototype lookup so class instances and built-ins are rejected.
   */
  const proto: unknown = Object.getPrototypeOf(value,);
  return (proto === Object
    .prototype) || (proto === null);
}
