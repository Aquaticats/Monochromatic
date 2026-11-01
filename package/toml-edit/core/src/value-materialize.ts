/**
 Materialize a {@link ValueNode} (and dotted-key chains) into plain JS.
 
 Reads (`tomlGetValue` / `tomlHas` / `tomlKeys`) walk the current tree and
 materialize the addressed node, so they always agree with what `tomlStringify`
 would emit.
 
 Materialized objects are built immutably and prototype-safely: every key is
 written through a computed object property (never plain assignment) and every
 read goes through {@link Object.hasOwn}, so a `__proto__` key becomes a normal
 own property instead of mutating a prototype.
 
 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { ValueNode, } from './document.ts';

/**
 Whether `value` is a plain (non-array) object record.
 
 @param value - Candidate whose object-ness decides whether descent continues
   into it or replaces it with a fresh table.
 
 @returns Whether `value` is a non-null, non-array object.
 
 @example
 ```ts
 isRecord({},); // true
 isRecord([],); // false
 ```
 */
export function isRecord(value: unknown,): value is Record<string, unknown> {
  return (value !== null) && ((typeof value) === 'object')
    && (!Array.isArray(value,));
}

/**
 Whether `value` is an array, narrowed to `readonly unknown[]` so element
 access stays typed rather than collapsing to `any`.
 
 @param value - Candidate tested for array-of-tables descent.
 
 @returns Whether `value` is an array.
 
 @example
 ```ts
 isUnknownArray([1],); // true
 ```
 */
export function isUnknownArray(value: unknown,): value is readonly unknown[] {
  return Array.isArray(value,);
}

/**
 Return a shallow clone of `source` with `key` set to `value` as an own data
 property.
 
 Uses a computed object property rather than assignment so a `__proto__` key
 becomes an own property instead of invoking the prototype setter. Object
 spread likewise copies an existing own `__proto__` through data-property
 creation, keeping the clone prototype-safe.
 
 @param source - Object cloned so the update stays immutable.
 
 @param key - Property name written, safe for `__proto__` and friends.
 
 @param value - Value stored at `key`.
 
 @returns Fresh object equal to `source` but with `key` set to `value`.
 
 @example
 ```ts
 assocOwn({ source: { a: 1, }, key: 'b', value: 2, },); // { a: 1, b: 2 }
 ```
 */
function assocOwn(
  {
    source,
    key,
    value,
  }: {
    readonly source: Readonly<Record<string, unknown>>;
    readonly key: string;
    readonly value: unknown;
  },
): Record<string, unknown> {
  return {
    ...source,
    [key]: value,
  };
}

/**
 Descent frame recording, per intermediate segment, the parent object and key
 to rebuild through, plus the crossed array when descent stepped into an
 array-of-tables instance.
 */
type DescentFrame = {
  readonly parent: Readonly<Record<string, unknown>>;
  readonly key: string;
  readonly array?: readonly unknown[];
};

/**
 Immutably transform the value at nested `path` within `container`.
 
 Descends the intermediate segments, stepping into the last element of any
 array crossed (array-of-tables sub-table semantics) and creating records for
 absent segments, then applies `update` to the value at the final segment and
 rebuilds every crossed container bottom-up. Reads use {@link Object.hasOwn}
 and writes use {@link assocOwn}, so a `__proto__` segment is handled as a
 normal own property. Iterative (reduce / reduceRight) rather than recursive
 over the flat `path` spine.
 
 @param container - Root object the update is threaded through.
 
 @param path - Segment chain addressing the value to transform.
 
 @param update - Maps the current value at `path` (or `undefined`) to its
   replacement, letting one primitive serve replace, ensure, and append.
 
 @returns Fresh root object with the addressed value transformed.
 
 @mutates container - `Object.hasOwn` can invoke caller-owned proxy descriptor hooks while descending.
 
 @mutates update - Invoking caller-supplied updater can change captured or otherwise reachable state.
 
 @example
 ```ts
 updateDeep({ container: {}, path: ['a', 'b'], update: () => 1, },); // { a: { b: 1 } }
 ```
 */
export function updateDeep(
  {
    container,
    path,
    update,
  }: {
    readonly container: Record<string, unknown>;
    readonly path: readonly (string | number)[];
    readonly update: (existing: unknown) => unknown;
  },
): Record<string, unknown> {
  /**
   Final segment key; the intermediates are everything before it.
   */
  const lastSeg = path.at(-1,);
  if (lastSeg === undefined)
    return { ...container, };
  /**
   Descent state: cursor at the current depth and the frames to rebuild.
   */
  const descent = path
    .slice(
    0,
    -1,
    )
    .reduce<{
      readonly cursor: Record<string, unknown>;
      readonly frames: readonly DescentFrame[];
    }>(
    /**
     Descends one path segment through own properties.
     
     @param acc - Cursor and immutable rebuild frames.
     
     @param seg - Path segment being resolved.
     
     @returns Next cursor and appended rebuild frame.
     
     @mutates acc - `Object.hasOwn` can invoke proxy descriptor hooks reachable through cursor.
     */
    function step(
      acc: {
        readonly cursor: Record<string, unknown>;
        readonly frames: readonly DescentFrame[];
      },
      seg,
    ) {
      /**
       String key so numeric header slots address by string like the parser.
       */
      const key = String(seg,);
      /**
       Existing child, read own-only so a `__proto__` segment cannot reach the prototype.
       */
      const existing = Object.hasOwn(
        acc.cursor,
        key,
      )
        ? acc.cursor[key]
        : undefined;
      if (isUnknownArray(existing,)) {
        /**
         Last array-of-tables instance the sub-table descends into.
         */
        const last = nonNullishOrThrow(existing.at(-1,),);
        return {
          cursor: isRecord(last,) ? last : {},
          frames: [
            ...acc.frames,
            {
              parent: acc.cursor,
              key,
              array: existing,
            },
          ],
        };
      }
      return {
        cursor: isRecord(existing,) ? existing : {},
        frames: [
          ...acc.frames,
          {
            parent: acc.cursor,
            key,
          },
        ],
      };
    },
    {
      cursor: container,
      frames: [],
    },
    );
  /**
   Final key so the leaf value is read and rewritten own-only.
   */
  const finalKey = String(lastSeg,);
  /**
   Deepest container rebuilt with the transformed leaf value.
   */
  const rebuiltDeepest = assocOwn({
    source: descent.cursor,
    key: finalKey,
    value: update(
      Object.hasOwn(
        descent.cursor,
        finalKey,
      )
        ? descent.cursor[finalKey]
        : undefined,
    ),
  },);
  return descent.frames
    .reduceRight<Record<string, unknown>>(
    function rebuild(
      child: Readonly<Record<string, unknown>>,
      frame,
    ) {
      return assocOwn({
        source: frame.parent,
        key: frame.key,
        value: frame.array === undefined
          ? child
          : frame.array
            .with(
            frame.array
              .length
              - 1,
            child,
            ),
      },);
    },
    rebuiltDeepest,
    );
}

/**
 Immutably set `value` at nested `path` within `container`.
 
 Thin wrapper over {@link updateDeep} whose leaf op replaces the addressed
 value outright.
 
 @param container - Root object the write is threaded through.
 
 @param path - Segment chain addressing the slot to set.
 
 @param value - Value written at `path`.
 
 @returns Fresh root object with `value` set at `path`.
 
 @example
 ```ts
 setDeep({ container: {}, path: ['a', 'b'], value: 1, },); // { a: { b: 1 } }
 ```
 */
export function setDeep(
  {
    container,
    path,
    value,
  }: {
    readonly container: Readonly<Record<string, unknown>>;
    readonly path: readonly (string | number)[];
    readonly value: unknown;
  },
): Record<string, unknown> {
  return updateDeep({
    container,
    path,
    update: function replaceLeaf() {
      return value;
    },
  },);
}

/**
 Materialize a value node to its plain JS value.
 
 @param value - Value node whose materialized JS value reads return.
 
 @returns Computed JS value.
 
 @example
 ```ts
 materializeValue({ value: kv.value, },); // 42 | 'x' | [1,2] | { a: 1 }
 ```
 */
export function materializeValue({ value, }: { readonly value: ValueNode; },): unknown {
  if (value.kind
    === 'scalar')
    return value.jsValue;
  if (value.kind
    === 'array') {
    return value.elements
      .map(function each(el,) {
        return materializeValue({ value: el, },);
      },);
  }
  return value.entries
    .reduce<Record<string, unknown>>(
    function step(
      acc: Readonly<Record<string, unknown>>,
      entry,
    ) {
      return setDeep({
        container: acc,
        path: entry.keySegments,
        value: materializeValue({ value: entry.value, },),
      },);
    },
    {},
    );
}
