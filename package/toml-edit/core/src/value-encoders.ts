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
import { TomlTypeError, } from './errors.ts';
import type { TomlWrappedInput, } from './types.ts';
import {
  isLocalDateText,
  isLocalDateTimeText,
  isLocalTimeText,
} from './wrapped-scalar-grammar.ts';

/**
 Wrapper shape as it can arrive from a caller that never passed through the
 factory functions, whose signatures alone reject a mismatched payload.
 */
type UnvalidatedWrappedInput = {
  readonly tomlKind: string;
  readonly value: unknown;
};

/**
 Tagged kinds the encoder can represent, named once so the unknown-kind
 diagnostic can list them.
 */
const REPRESENTABLE_KINDS = [
  'integer',
  'float',
  'local-date',
  'local-date-time',
  'local-time',
] as const;

/**
 Bits in one half of TOML's signed 64-bit integer range.
 */
const INTEGER_MAGNITUDE_BITS = 63;

/**
 Lowest TOML integer, as a bigint.
 */
const INTEGER_MIN_BIGINT = -(1n << BigInt(INTEGER_MAGNITUDE_BITS,));

/**
 Lowest integer just past TOML's range, as a bigint.
 */
const INTEGER_EXCLUSIVE_MAX_BIGINT = 1n << BigInt(INTEGER_MAGNITUDE_BITS,);

/**
 Negative zero's digits, which `String(-0)` cannot produce because it returns
 `'0'`; TOML keeps the sign in `-0.0`.
 */
const NEGATIVE_ZERO_TEXT = '-0';

/**
 Encode a tagged wrapper input ({@link tomlInteger}, {@link tomlFloat}, date kinds).
 
 The payload is validated against the kind it claims before any of it is
 interpolated, because the payload reaches the document as bare text and a
 stray newline, quote, or comment marker would change the document's meaning.
 
 @param wrapped - Tagged wrapper whose kind and payload are checked here.
 
 @returns Computed TOML text for the wrapped value.
 
 @throws {@link TomlTypeError} when the payload cannot be represented as the
         kind it claims, or when the kind is not one the encoder knows.
 
 @example
 ```ts
 encodeWrapped({ wrapped: { tomlKind: 'integer', value: 7n, }, },); // '7'
 encodeWrapped({ wrapped: { tomlKind: 'integer', value: '1\nbad = 1', }, },); // throws
 ```
 */
export function encodeWrapped(
  { wrapped, }: { readonly wrapped: UnvalidatedWrappedInput; },
): string {
  if (wrapped.tomlKind
    === 'integer')
    return encodeIntegerPayload({ value: wrapped.value, },);
  if (wrapped.tomlKind
    === 'float')
    return encodeFloatPayload({ value: wrapped.value, },);
  if (wrapped.tomlKind
    === 'local-date') {
    return encodeDateKindPayload({
      kind: 'local-date',
      value: wrapped.value,
      isValid: isLocalDateText,
      expected: 'a YYYY-MM-DD date naming a real day, such as 2026-05-14',
    },);
  }
  if (wrapped.tomlKind
    === 'local-date-time') {
    return encodeDateKindPayload({
      kind: 'local-date-time',
      value: wrapped.value,
      isValid: isLocalDateTimeText,
      expected:
        'a date, then a T, t, or space, then a time without an offset, as in 2026-05-14T10:00:00',
    },);
  }
  if (wrapped.tomlKind
    === 'local-time') {
    return encodeDateKindPayload({
      kind: 'local-time',
      value: wrapped.value,
      isValid: isLocalTimeText,
      expected: 'a time such as 10:00:00, with no date and no offset',
    },);
  }
  throw new TomlTypeError(
    `Cannot encode a wrapped value with unknown kind ${
      describePayload({ value: wrapped.tomlKind, },)
    }; expected one of ${REPRESENTABLE_KINDS.join(', ',)}. Build the value with tomlInteger, tomlFloat, tomlLocalDate, tomlLocalDateTime, or tomlLocalTime.`,
  );
}

/**
 Encode an integer payload, which becomes bare digits.
 
 @param value - Payload to check and format.
 
 @returns Decimal digits for the integer.
 
 @throws {@link TomlTypeError} when the payload is not a bigint inside TOML's
         signed 64-bit range, or a number that a double spells exactly.
 
 @example
 ```ts
 encodeIntegerPayload({ value: 42n, }); // '42'
 ```
 */
function encodeIntegerPayload({ value, }: { readonly value: unknown; },): string {
  if (((typeof value) === 'bigint')
    && (value
      >= INTEGER_MIN_BIGINT)
    && (value
      < INTEGER_EXCLUSIVE_MAX_BIGINT)) {
    return value.toString();
  }
  /**
   A safe integer is the only number whose decimal spelling survives the round
   trip: above it, `String` emits the shortest form that parses back to the same
   double, which is a different integer.
   */
  if (((typeof value) === 'number') && Number.isSafeInteger(value,))
    return String(value,);
  throw rejectWrappedPayload({
    kind: 'integer',
    value,
    expected:
      'an integer number no larger than 9007199254740991 in magnitude, or a bigint in the signed 64-bit range TOML allows, -9223372036854775808 to 9223372036854775807',
  },);
}

/**
 Encode a float payload, which becomes bare digits plus a float marker.
 
 @param value - Payload to check and format.
 
 @returns TOML float text, with the special spellings for nonfinite values.
 
 @throws {@link TomlTypeError} when the payload is not a JS number.
 
 @example
 ```ts
 encodeFloatPayload({ value: 1, }); // '1.0'
 ```
 */
function encodeFloatPayload({ value, }: { readonly value: unknown; },): string {
  if ((typeof value) !== 'number') {
    throw rejectWrappedPayload({
      kind: 'float',
      value,
      expected: 'a number, including NaN and the two infinities',
    },);
  }
  if (!Number.isFinite(value,)) {
    if (Number.isNaN(value,))
      return 'nan';
    return value > 0 ? 'inf' : '-inf';
  }
  /**
   String form so the float-marker check can scan once; negative zero keeps its
   sign here, because `String(-0)` drops the sign TOML expresses as `-0.0`.
   */
  const text = Object.is(
    value,
    -0,
  )
    ? NEGATIVE_ZERO_TEXT
    : String(value,);
  return text.includes('.',)
    || text
      .includes('e',)
    || text
      .includes('E',) ? text : `${text}.0`;
}

/**
 Encode a date or time payload, which becomes the payload text itself.
 
 @param kind - Wrapper kind being encoded, named in a rejection.
 
 @param value - Payload to check and interpolate.
 
 @param isValid - Grammar check for the kind's complete lexeme.
 
 @param expected - Plain-language description of the accepted form.
 
 @returns The validated payload text, safe to place after `=`.
 
 @throws {@link TomlTypeError} when the payload is not a string, or is a
         string the kind's grammar does not accept.
 
 @example
 ```ts
 encodeDateKindPayload({ kind: 'local-date', value: '2026-05-14', isValid: isLocalDateText, expected: 'a date', }); // '2026-05-14'
 ```
 */
function encodeDateKindPayload(
  {
    kind,
    value,
    isValid,
    expected,
  }: {
    readonly kind: string;
    readonly value: unknown;
    readonly isValid: (input: { readonly text: string; }) => boolean;
    readonly expected: string;
  },
): string {
  if (((typeof value) === 'string') && isValid({ text: value, },))
    return value;
  throw rejectWrappedPayload({
    kind,
    value,
    expected,
  },);
}

/**
 Render a payload for a diagnostic without letting it break the message.
 
 @param value - Payload that failed its kind's grammar.
 
 @returns String payloads as JSON text, bigints with their suffix, and
          everything else through `String`.
 
 @example
 ```ts
 describePayload({ value: 'a\nb', }); // '"a\\nb"'
 ```
 */
function describePayload({ value, }: { readonly value: unknown; },): string {
  if ((typeof value) === 'string')
    return JSON.stringify(value,);
  if ((typeof value) === 'bigint')
    return `${value.toString()}n`;
  return String(value,);
}

/**
 Build the rejection for a wrapper payload the kind cannot represent.
 
 @param kind - Wrapper kind that rejected the payload.
 
 @param value - Payload that could not be represented.
 
 @param expected - Plain-language description of an acceptable payload.
 
 @returns Error naming the kind, the payload, and the accepted form.
 
 @example
 ```ts
 rejectWrappedPayload({ kind: 'integer', value: 1.5, expected: 'an integer', });
 ```
 */
function rejectWrappedPayload(
  {
    kind,
    value,
    expected,
  }: {
    readonly kind: string;
    readonly value: unknown;
    readonly expected: string;
  },
): TomlTypeError {
  return new TomlTypeError(
    `Cannot encode ${
      describePayload({ value, },)
    } as TOML ${kind}: expected ${expected}. Pass a payload the ${kind} wrapper can represent, or encode the text as a TOML string.`,
  );
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
 
 Both keys are required: a table that merely happens to hold a string under
 `tomlKind` keeps the ordinary inline-table path, and only a value carrying
 both `tomlKind` and `value` is treated as a claim about a TOML scalar kind.
 
 @param value - Arbitrary JS value to test.
 
 @returns True when `value` is an object carrying a string `tomlKind` and a
          `value` property.
 
 @example
 ```ts
 isWrappedInput({ tomlKind: 'float', value: 1, },); // true
 isWrappedInput({ tomlKind: 'float', },);          // false
 ```
 */
export function isWrappedInput(value: unknown,): value is TomlWrappedInput {
  return (
    ((typeof value) === 'object')
      && (value !== null)
      && ('tomlKind' in value)
      && ('value' in value)
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
