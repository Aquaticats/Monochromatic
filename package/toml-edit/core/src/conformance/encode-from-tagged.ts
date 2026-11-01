/**
 Convert a toml-test tagged tree into a {@link tomlSet} value input.
 
 The encoder adapter receives the same tagged JSON dialect the decoder emits
 and must rebuild a JS value that {@link tomlSet} serializes back to TOML. Integers
 become `bigint` wrappers so 64-bit values survive, offset datetimes become a
 JS `Date` (emitted as an RFC 3339 instant), and the three local datetime
 kinds become tagged wrappers since a `Date` cannot distinguish them.
 
 @module
 */

import type { TomlWrappedInput, } from '../index.ts';

import type { TaggedType, } from './tagged-types.ts';

/**
 Recognized tagged-scalar type names, for distinguishing scalars from tables.
 */
const TAG_NAMES: ReadonlySet<string> = new Set<TaggedType>([
  'string',
  'integer',
  'float',
  'bool',
  'datetime',
  'datetime-local',
  'date-local',
  'time-local',
],);

/**
 Test whether `value` is a plain object record (not an array, not null).
 
 @param value - Arbitrary parsed JSON node.
 
 @returns True when `value` is a non-array object.
 
 @example
 ```ts
 isRecord({ a: 1, }); // true
 ```
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Test whether a record is a tagged scalar rather than a table.
 
 A tagged scalar carries a string `type` drawn from the toml-test vocabulary;
 a table whose source had a `type` key holds a nested node there instead, so
 the string check is decisive.
 
 @param value - Record to classify.
 
 @returns True when `value` is a tagged scalar.
 
 @example
 ```ts
 isTaggedScalar({ type: 'bool', value: 'true', }); // true
 ```
 */
function isTaggedScalar(
  value: Readonly<Record<string, unknown>>,
): value is Record<string, unknown> & {
  type: TaggedType;
  value: string;
} {
  return ((typeof value.type) === 'string')
    && ((typeof value.value) === 'string')
    && TAG_NAMES.has(value.type,);
}

/**
 Parse a tagged float payload, including the special spellings.
 
 @param value - Float payload from the tagged scalar.
 
 @returns Numeric value, with `inf` / `-inf` / `nan` mapped to JS specials.
 
 @example
 ```ts
 floatFromTag('-inf'); // Number.NEGATIVE_INFINITY
 ```
 */
function floatFromTag(value: string,): number {
  /**
   Lower-cased payload so the sign-and-word spellings collapse to a few cases.
   */
  const lower = value.toLowerCase();
  if ((lower === 'nan') || (lower === '+nan')
    || (lower === '-nan'))
    return Number.NaN;
  if ((lower === 'inf') || (lower === '+inf'))
    return Number.POSITIVE_INFINITY;
  if (lower === '-inf')
    return Number.NEGATIVE_INFINITY;
  return Number(value,);
}

/**
 Convert one tagged scalar into its {@link tomlSet} value input.
 
 @param type - Tagged scalar type name.
 
 @param value - Tagged scalar payload string.
 
 @returns Value input: a primitive, a `Date`, or a tagged wrapper object.
 
 @throws SyntaxError when an integer payload is not a valid `bigint` literal,
         so malformed encoder input rejects rather than corrupting output.
 
 @example
 ```ts
 scalarToInput({ type: 'integer', value: '255', }); // { tomlKind: 'integer', value: 255n }
 ```
 */
function scalarToInput(
  {
    type,
    value,
  }: {
    readonly type: TaggedType;
    readonly value: string
  },
): unknown {
  if (type === 'string')
    return value;
  if (type === 'integer')
    return {
      tomlKind: 'integer',
      value: BigInt(value,),
    } satisfies TomlWrappedInput;
  if (type === 'float')
    return {
      tomlKind: 'float',
      value: floatFromTag(value,),
    } satisfies TomlWrappedInput;
  if (type === 'bool')
    return value === 'true';
  if (type === 'datetime')
    return new Date(value,);
  if (type === 'datetime-local')
    return {
      tomlKind: 'local-date-time',
      value,
    } satisfies TomlWrappedInput;
  if (type === 'date-local')
    return {
      tomlKind: 'local-date',
      value,
    } satisfies TomlWrappedInput;
  return {
    tomlKind: 'local-time',
    value,
  } satisfies TomlWrappedInput;
}

/**
 Convert a tagged tree (scalar, array, or table) into a {@link tomlSet} value input.
 
 @param tree - Parsed tagged JSON node.
 
 @returns Value input understood by {@link tomlSet}.
 
 @throws Error when a node is neither a tagged scalar, an array, nor a table.
 
 @mutates tree - Traversal can invoke caller-owned array, proxy, and property-access hooks recursively.
 
 @example
 ```ts
 taggedToInput({ tree: { a: { type: 'bool', value: 'true', } }, }); // { a: true }
 ```
 */
export function taggedToInput({ tree, }: { readonly tree: unknown; },): unknown {
  if (Array.isArray(tree,))
    return tree.map(
      /**
       Converts one tagged array child recursively.
       
       @param child - Tagged child value.
       
       @returns converted TOML input.
       
       @mutates child - Recursive traversal can invoke caller-owned array, proxy, and accessor hooks.
       */
      function element(child,) {
        return taggedToInput({ tree: child, },);
      },
    );
  if (isRecord(tree,)) {
    if (isTaggedScalar(tree,))
      return scalarToInput({
        type: tree.type,
        value: tree.value,
      },);
    return Object.fromEntries(
      Object.entries(tree,)
        .map(
          /**
           Converts one tagged table entry recursively.
           
           @param taggedEntry - Tagged table key and child pair.
           
           @returns converted key and TOML input pair.
           
           @mutates taggedEntry - Recursive child traversal can invoke caller-owned array, proxy, and accessor hooks.
           */
        function entry(taggedEntry,) {
          /**
           Tagged key and child selected by current entry.
           */
          const [key, child,] = taggedEntry;
          return [
            key,
            taggedToInput({ tree: child, },),
          ];
        },),
    );
  }
  throw new Error('conformance encode: malformed tagged JSON node',);
}
