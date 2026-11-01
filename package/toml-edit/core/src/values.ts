/**
 JS-to-TOML value coercion.
 
 Produces canonical TOML text for an arbitrary JS value. Optionally
 preserves style and raw spelling from an existing AST node when the new
 numeric / string value equals the parse-time value.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { AST, } from 'toml-eslint-parser';

import {
  assembleArrayParts,
  assembleInlineTableParts,
} from './emit-value.ts';
import { TomlTypeError, } from './errors.ts';
import { encodeKey, } from './keys.ts';
import type { CanonicalOptions, } from './types.ts';
import {
  encodeStringWithStyle,
  encodeWrapped,
  isPlainObject,
  isWrappedInput,
} from './value-encoders.ts';

export { encodeKey, } from './keys.ts';
export { isPlainObject, } from './value-encoders.ts';

/**
 Optional existing AST node carrier.
 
 Marks parser-owned AST ingress once so observation helpers preserve foreign
 ownership provenance without repeating markers on descendants.
 */
export type ExistingNode = {
  readonly node: ForeignBorrowed<AST.TOMLNode>;
};

/**
 Encode a JS value as TOML text suitable for assignment after `=`.
 
 When `existing` is provided and the new value matches its parsed value,
 `style` / `multiline` / raw `number` / `datetime` spelling are preserved.
 
 @returns Computed string.
 
 @throws {@link TomlTypeError} for `null`, `undefined`, symbols, functions, or
         circular structures.
 
 @mutates input - Recursive encoding can invoke caller-owned proxy, getter, prototype, and serialization hooks.
 
 @example
 ```ts
 jsValueToTomlText({ input: 42, options, },);              // '42'
 jsValueToTomlText({ input: 'hi', options, },);            // '"hi"'
 jsValueToTomlText({ input: { a: 1, }, options, },);       // '{ a = 1, }'
 ```
 */
export function jsValueToTomlText(
  {
    input,
    options,
    existing,
  }: {
    readonly input: unknown;
    readonly options: CanonicalOptions;
    readonly existing?: ExistingNode;
  },
): string {
  /**
   Carrier re-passed only when present, so `undefined` never enters the optional `existing` slot under `exactOptionalPropertyTypes`.
   */
  const existingArg = existing === undefined ? {} : { existing, };
  return encodeValue({
    input,
    options,
    depth: 0,
    ...existingArg,
  },);
}

/**
 Encode an arbitrary JS value as TOML text, recursively.
 
 @returns Computed string.
 
 @throws {@link TomlTypeError} for `null`, `undefined`, or any other value
         that is not a wrapped input, string, boolean, bigint, number,
         `Date`, array, or plain object.
 
 @mutates input - Recursive encoding can invoke caller-owned proxy, getter, prototype, and serialization hooks.
 */
function encodeValue(
  {
    input,
    options,
    existing,
    depth,
  }: {
    readonly input: unknown;
    readonly options: CanonicalOptions;
    readonly existing?: ExistingNode;
    readonly depth: number;
  },
): string {
  if ((input === null) || (input === undefined)) {
    throw new TomlTypeError(
      `Cannot encode ${String(input,)} as TOML; use tomlDelete to remove a key`,
    );
  }

  /**
   Carrier re-passed only when present, so `undefined` never enters the optional `existing` slot under `exactOptionalPropertyTypes`.
   */
  const existingArg = existing === undefined ? {} : { existing, };

  if (isWrappedInput(input,))
    return encodeWrapped({ wrapped: input, },);

  if ((typeof input) === 'string') {
    return encodeString({
      value: input,
      ...existingArg,
    },);
  }

  if ((typeof input) === 'boolean')
    return input ? 'true' : 'false';

  if ((typeof input) === 'bigint')
    return input.toString();

  if ((typeof input) === 'number') {
    return encodeNumber({
      value: input,
      ...existingArg,
    },);
  }

  if (input instanceof Date)
    return input.toISOString();

  if (Array.isArray(input,)) {
    return encodeArray({
      input,
      options,
      depth,
      ...existingArg,
    },);
  }

  if (isPlainObject(input,)) {
    return encodeInlineTable({
      input,
      options,
      depth,
    },);
  }

  throw new TomlTypeError(`Cannot encode ${typeof input} as TOML`,);
}

/**
 Encode a JS string, preserving existing quote style if equal-valued.
 
 @returns Computed string.
 */
function encodeString(
  {
    value,
    existing,
  }: {
    readonly value: string;
    readonly existing?: ExistingNode;
  },
): string {
  if (
    (existing !== undefined)
      && (existing.node
        .type
        === 'TOMLValue')
      && (existing.node
        .kind
        === 'string')
      && (existing.node
        .value
        === value)
  ) {
    return encodeStringWithStyle({
      value,
      style: existing.node
        .style,
      multiline: existing.node
        .multiline,
    },);
  }
  /**
   Multi-line content selects triple-quoted output to avoid splitting.
   */
  const hasNewline = value.includes('\n',);
  if (hasNewline) {
    return encodeStringWithStyle({
      value,
      style: 'basic',
      multiline: true,
    },);
  }
  return encodeStringWithStyle({
    value,
    style: 'basic',
    multiline: false,
  },);
}

/**
 Encode a JS number, preserving existing raw spelling when value is unchanged.
 
 @returns Computed string.
 */
function encodeNumber(
  {
    value,
    existing,
  }: {
    readonly value: number;
    readonly existing?: ExistingNode;
  },
): string {
  if (
    (existing !== undefined)
      && (existing.node
        .type
        === 'TOMLValue')
      && ((existing.node
        .kind
        === 'integer') || (existing.node
          .kind
          === 'float'))
      && (existing.node
        .value
        === value)
  ) {
    return existing.node
      .number;
  }
  if (!Number.isFinite(value,)) {
    if (Number.isNaN(value,))
      return 'nan';
    return value > 0 ? 'inf' : '-inf';
  }
  return String(value,);
}

/**
 Encode a JS array, recursing on elements; inline or multiline per options.
 
 @returns Computed string.
 */
function encodeArray(
  {
    input,
    options,
    depth,
    existing,
  }: {
    readonly input: readonly unknown[];
    readonly options: CanonicalOptions;
    readonly depth: number;
    readonly existing?: ExistingNode;
  },
): string {
  /**
   Existing per-element AST so encoder can reuse spelling when value matches.
   */
  const elementExistings = (existing !== undefined) && (existing.node
    .type
    === 'TOMLArray')
    ? existing.node
      .elements
    : null;
  /**
   Encoded element strings so the result can be both inline-tested and multi-line-emitted.
   */
  const encoded = input.map(function each(
    el,
    i,
  ) {
    /**
     Existing element node at this index, if the parent array carried one.
     */
    const elementExisting = elementExistings === null ? undefined : elementExistings[i];
    /**
     Carrier re-passed only when present, so `undefined` never enters the optional `existing` slot.
     */
    const elementExistingArg = elementExisting === undefined ? {} : { existing: { node: elementExisting, }, };
    return encodeValue({
      input: el,
      options,
      depth: depth + 1,
      ...elementExistingArg,
    },);
  },);
  return assembleArrayParts({
    parts: encoded,
    options,
    depth,
  },);
}

/**
 Encode a JS plain object as a TOML inline table `{ k = v, ... }`.
 
 @returns Computed string.
 
 @mutates input - `Object.entries` and recursive encoding can invoke caller-owned proxy and accessor hooks.
 */
function encodeInlineTable(
  {
    input,
    options,
    depth,
  }: {
    readonly input: Readonly<Record<string, unknown>>;
    readonly options: CanonicalOptions;
    readonly depth: number;
  },
): string {
  /**
   Entries so iteration is keyed and ordering is deterministic.
   */
  const entries = Object.entries(input,);
  /**
   Each entry becomes its own `k = v` fragment so the joiner can comma-separate.
   */
  const parts = entries.map(function each([k, v,],) {
    return `${encodeKey({ key: k, },)} = ${
      encodeValue({
        input: v,
        options,
        depth: depth + 1,
      },)
    }`;
  },);
  return assembleInlineTableParts({ parts, },);
}
