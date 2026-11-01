/**
 Value-node descent for {@link tomlSet}, split out of `set-value.ts` to keep
 each module under the max-lines budget.
 
 Given the current value at a key-value (an array or inline table) and the
 remaining path, descend and replace (or, for a missing inline key, append)
 the addressed leaf, returning a fresh synthetic value node. Also hosts the
 shared {@link NOT_SET} sentinel and {@link existingArg} carrier so the
 block-level module (`set-value.ts`) can depend on this one without either a
 dependency cycle or a duplicated helper.
 
 @module
 */

import { buildValueFromInput, } from './build-input.ts';
import type {
  KeyValueNode,
  ValueNode,
} from './document.ts';
import { TomlImmutableNodeError, } from './errors.ts';
import { formatPath, } from './path.ts';
import { isPrefix, } from './path-prefix.ts';
import type {
  CanonicalOptions,
  TomlPath,
} from './types.ts';
import type { ExistingNode, } from './values.ts';

/**
 Sentinel for "no existing value at this path".
 */
export const NOT_SET: unique symbol = Symbol('toml-edit/set-no-existing-value-target',);

/**
 Existing-node argument for {@link buildValueFromInput}: present only when the
 current value is clean, so an equal re-set preserves its raw spelling.
 
 @param value - Current value whose retained parse-time node, when clean, is
   threaded onward so an equal re-set keeps the original raw spelling.
 
 @returns Object carrying the `existing` node, or empty when the value is dirty.
 
 @example
 ```ts
 existingArg(kv.value,); // { existing: { node } } | {}
 ```
 */
export function existingArg(value: ValueNode,): { readonly existing?: ExistingNode; } {
  /**
   Retained parse-time node when clean; narrowed to a const so the spread type is exact.
   */
  const node = value.origin
    .kind
    === 'clean' ? value.origin
      .astNode : undefined;
  return node === undefined ? {} : { existing: { node, }, };
}

/**
 Descend `rest` into an array or inline-table value and replace (or, for a
 missing inline key, append) the leaf.
 
 @param value - Current value node being descended into.
 
 @param rest - Remaining path segments addressing the leaf within `value`.
 
 @param input - New JS value written at the leaf.
 
 @param options - Canonical formatting options for any synthesized node.
 
 @param path - Full original path, retained for error messages.
 
 @returns Replacement value node, or {@link NOT_SET}.
 
 @throws {@link TomlImmutableNodeError} when an inline extension collides with an
         existing key chain.
 
 @example
 ```ts
 replaceInValue({ value: kv.value, rest: ['x'], input: 2, options, path, },);
 ```
 */
export function replaceInValue(
  {
    value,
    rest,
    input,
    options,
    path,
  }: {
    readonly value: ValueNode;
    readonly rest: TomlPath;
    readonly input: unknown;
    readonly options: CanonicalOptions;
    readonly path: TomlPath;
  },
): ValueNode | typeof NOT_SET {
  /**
   Leading segment selecting the child to descend into or replace.
   */
  const [head, ...tail] = rest;
  if ((value.kind
    === 'array') && ((typeof head) === 'number')) {
    /**
     Targeted element, or `undefined` when the index is out of range.
     */
    const element = value.elements[head];
    if (element === undefined)
      return NOT_SET;
    /**
     New element: a direct replace at the tail, or a deeper descent.
     */
    const newEl = tail.length
      === 0
      ? buildValueFromInput({
        input,
        options,
        ...existingArg(element,),
      },)
      : replaceInValue({
        value: element,
        rest: tail,
        input,
        options,
        path,
      },);
    if (newEl === NOT_SET)
      return NOT_SET;
    return {
      kind: 'array',
      elements: value.elements
        .with(
        head,
        newEl,
        ),
      origin: { kind: 'synthetic', },
    };
  }
  if (value.kind
    === 'inline-table')
    return replaceInInlineTable({
      value,
      rest,
      input,
      options,
      path,
    },);
  return NOT_SET;
}

/**
 Replace a matching entry in an inline table, or append a new dotted entry.
 
 @param value - Inline-table value the descent replaces or extends.
 
 @param rest - Remaining path segments addressing the entry within `value`.
 
 @param input - New JS value written at the addressed entry.
 
 @param options - Canonical formatting options for any synthesized node.
 
 @param path - Full original path, retained for error messages.
 
 @returns Replacement inline-table node.
 
 @throws {@link TomlImmutableNodeError} when the new key chain overlaps an existing one.
 
 @example
 ```ts
 replaceInInlineTable({ value: inline, rest: ['b'], input: 1, options, path, },);
 ```
 */
function replaceInInlineTable(
  {
    value,
    rest,
    input,
    options,
    path,
  }: {
    readonly value: Extract<ValueNode, { kind: 'inline-table'; }>;
    readonly rest: TomlPath;
    readonly input: unknown;
    readonly options: CanonicalOptions;
    readonly path: TomlPath;
  },
): ValueNode {
  for (const [j, entry,] of value.entries
    .entries()) {
    if (!isPrefix({
      candidate: entry.keySegments,
      path: rest,
    },))
      continue;
    /**
     New entry: replace whole value at an exact key hit, else descend deeper.
     */
    const newEntry = entry.keySegments
      .length
      === rest.length
      ? {
        ...entry,
        value: buildValueFromInput({
          input,
          options,
          ...existingArg(entry.value,),
        },),
      }
      : descendInlineEntry({
        entry,
        rest,
        input,
        options,
        path,
      },);
    if (newEntry === NOT_SET)
      continue;
    return {
      kind: 'inline-table',
      entries: value.entries
        .with(
        j,
        newEntry,
        ),
      origin: { kind: 'synthetic', },
    };
  }
  return appendInlineEntry({
    value,
    rest,
    input,
    options,
    path,
  },);
}

/**
 Descend into an inline entry's value for a deeper replace.
 
 @param entry - Matched inline entry whose value is descended into.
 
 @param rest - Remaining path segments, still including this entry's key.
 
 @param input - New JS value written at the deeper leaf.
 
 @param options - Canonical formatting options for any synthesized node.
 
 @param path - Full original path, retained for error messages.
 
 @returns Replacement key-value, or {@link NOT_SET}.
 
 @example
 ```ts
 descendInlineEntry({ entry, rest: ['a', 'b'], input: 1, options, path, },);
 ```
 */
function descendInlineEntry(
  {
    entry,
    rest,
    input,
    options,
    path,
  }: {
    readonly entry: KeyValueNode;
    readonly rest: TomlPath;
    readonly input: unknown;
    readonly options: CanonicalOptions;
    readonly path: TomlPath;
  },
): KeyValueNode | typeof NOT_SET {
  /**
   Value after descending past this entry's matched key segments.
   */
  const nested = replaceInValue({
    value: entry.value,
    rest: rest.slice(entry.keySegments
      .length,),
    input,
    options,
    path,
  },);
  if (nested === NOT_SET)
    return NOT_SET;
  return {
    ...entry,
    value: nested,
  };
}

/**
 Append a new dotted entry to an inline table (extension), after a collision
 check against existing entries.
 
 @param value - Inline-table value being extended.
 
 @param rest - Remaining path segments forming the new dotted key chain.
 
 @param input - New JS value written at the appended entry.
 
 @param options - Canonical formatting options for the synthesized node.
 
 @param path - Full original path, retained for error messages.
 
 @returns Extended inline-table node.
 
 @throws {@link TomlImmutableNodeError} on an overlapping key chain, or when a
         segment of `rest` is numeric.
 
 @example
 ```ts
 appendInlineEntry({ value: inline, rest: ['b'], input: 1, options, path, },);
 ```
 */
function appendInlineEntry(
  {
    value,
    rest,
    input,
    options,
    path,
  }: {
    readonly value: Extract<ValueNode, { kind: 'inline-table'; }>;
    readonly rest: TomlPath;
    readonly input: unknown;
    readonly options: CanonicalOptions;
    readonly path: TomlPath;
  },
): ValueNode {
  /**
   String-only new key chain; a numeric segment cannot address an inline key.
   */
  const segs = rest.map(function asString(seg,) {
    if ((typeof seg) !== 'string') {
      throw new TomlImmutableNodeError(
        `Cannot set numeric segment ${String(seg,)} in ${formatPath({ path, },)}`,
      );
    }
    return seg;
  },);
  for (const entry of value.entries) {
    if (isPrefix({
      candidate: entry.keySegments,
      path: rest,
    },) || isPrefix({
      candidate: segs,
      path: entry.keySegments,
    },)) {
      throw new TomlImmutableNodeError(
        `tomlSet at ${
          formatPath({ path, },)
        } would create invalid TOML: inline-table key ${
          entry.keySegments
            .join('.',)
        } overlaps ${segs.join('.',)}`,
      );
    }
  }
  return {
    kind: 'inline-table',
    entries: [
      ...value.entries,
      {
        kind: 'keyvalue',
        keySegments: segs,
        value: buildValueFromInput({
          input,
          options,
        },),
        origin: { kind: 'synthetic', },
        commentsBefore: [],
      },
    ],
    origin: { kind: 'synthetic', },
  };
}
