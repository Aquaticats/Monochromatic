/**
 Remove an existing value or entry from the block tree for {@link tomlDelete}.
 
 Descends through standard-table bodies, array-of-tables instances, and value
 nodes (arrays, inline tables) to the target. Removing a whole key-value drops
 its block (its physical span already includes the trailing comment and
 newline); removing an array element or inline entry rebuilds the enclosing
 value as synthetic so it re-renders canonically. Returns {@link NOT_REMOVED}
 when the path has no existing target.
 
 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type {
  Block,
  KeyValueNode,
  ValueNode,
} from './document.ts';
import { isPrefix, } from './path-prefix.ts';
import type { TomlPath, } from './types.ts';

/**
 Sentinel for "nothing removed at this path".
 */
export const NOT_REMOVED: unique symbol = Symbol('toml-edit/delete-no-existing-value-target',);

/**
 Remove the value or entry at `path` within `blocks`.
 
 @returns Fresh blocks, or {@link NOT_REMOVED}.
 
 @example
 ```ts
 removeAtPath({ blocks, path: ['arr', 1], },);
 ```
 */
export function removeAtPath(
  {
    blocks,
    path,
  }: {
    readonly blocks: readonly Block[];
    readonly path: TomlPath;
  },
): readonly Block[] | typeof NOT_REMOVED {
  for (const [index, block,] of blocks.entries()) {
    if (block.kind
      === 'keyvalue') {
      /**
       Removal outcome for this key-value: drop it, edit its value, or skip.
       */
      const outcome = removeFromKeyValue({
        kv: block,
        path,
      },);
      if (outcome === 'drop')
        return [
          ...blocks.slice(
            0,
            index,
          ),
          ...blocks.slice(index + 1,),
        ];
      if (outcome !== NOT_REMOVED)
        return blocks.with(
          index,
          outcome,
        );
      continue;
    }
    if (block.kind
      !== 'table')
      continue;
    /**
     Body result after descending into this table, or the not-removed sentinel.
     */
    const newBody = descendTableBody({
      table: block,
      path,
    },);
    if (newBody !== NOT_REMOVED)
      return blocks.with(
        index,
        {
          ...block,
          body: newBody,
        },
      );
  }
  return NOT_REMOVED;
}

/**
 Descend into a table's body to remove the target under `path`.
 
 @returns Fresh body blocks, or {@link NOT_REMOVED}.
 */
function descendTableBody(
  {
    table,
    path,
  }: {
    readonly table: Extract<Block, { kind: 'table'; }>;
    readonly path: TomlPath;
  },
): readonly Block[] | typeof NOT_REMOVED {
  /**
   Header length; where the body-relative path begins.
   */
  const headerLen = table.headerSegments
    .length;
  /**
   Array instances consume one extra segment (the matching index) first.
   */
  const bodyStart = table.tableKind
    === 'array' ? headerLen + 1 : headerLen;
  if ((table.tableKind
    === 'array') && (path[headerLen] !== table.aotIndex))
    return NOT_REMOVED;
  if ((!isPrefix({
    candidate: table.headerSegments,
    path,
  },)) || (path.length
    <= bodyStart))
    return NOT_REMOVED;
  return removeAtPath({
    blocks: table.body,
    path: path.slice(bodyStart,),
  },);
}

/**
 Decide what happens to a key-value for a deletion at `path`.
 
 @returns `'drop'` to remove the block, a replacement key-value when a nested
          value changed, or {@link NOT_REMOVED}.
 */
function removeFromKeyValue(
  {
    kv,
    path,
  }: {
    readonly kv: KeyValueNode;
    readonly path: TomlPath;
  },
): KeyValueNode | 'drop' | typeof NOT_REMOVED {
  if (!isPrefix({
    candidate: kv.keySegments,
    path,
  },))
    return NOT_REMOVED;
  if (kv.keySegments
    .length
    === path.length)
    return 'drop';
  /**
   Value after removing the addressed nested element/entry.
   */
  const newValue = removeInValue({
    value: kv.value,
    rest: path.slice(kv.keySegments
      .length,),
  },);
  if (newValue === NOT_REMOVED)
    return NOT_REMOVED;
  return {
    ...kv,
    value: newValue,
  };
}

/**
 Remove the element/entry addressed by `rest` inside a value node.
 
 @returns Fresh value node, or {@link NOT_REMOVED}.
 */
function removeInValue(
  {
    value,
    rest,
  }: {
    readonly value: ValueNode;
    readonly rest: TomlPath;
  },
): ValueNode | typeof NOT_REMOVED {
  /**
   Leading segment selecting the element or entry to descend into or drop.
   */
  const [head, ...tail] = rest;
  if ((value.kind
    === 'array') && ((typeof head) === 'number')) {
    if ((head < 0) || (head
      >= value.elements
        .length))
      return NOT_REMOVED;
    if (tail.length
      === 0) {
      return {
        kind: 'array',
        elements: value.elements
          .filter(function notDropped(
            _el,
            i,
          ) {
            return i !== head;
          },),
        origin: { kind: 'synthetic', },
      };
    }
    /**
     Element after removing the deeper target; the bounds check above proves
     the index is in range, so the slot is a present value node.
     */
    const newEl = removeInValue({
      value: nonNullishOrThrow(value.elements[head],),
      rest: tail,
    },);
    if (newEl === NOT_REMOVED)
      return NOT_REMOVED;
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
    return removeFromInlineTable({
      value,
      rest,
    },);
  return NOT_REMOVED;
}

/**
 Remove (or descend to remove within) a matching inline-table entry.
 
 @returns Fresh inline-table node, or {@link NOT_REMOVED}.
 */
function removeFromInlineTable(
  {
    value,
    rest,
  }: {
    readonly value: Extract<ValueNode, { kind: 'inline-table'; }>;
    readonly rest: TomlPath;
  },
): ValueNode | typeof NOT_REMOVED {
  for (const [j, entry,] of value.entries
    .entries()) {
    if (!isPrefix({
      candidate: entry.keySegments,
      path: rest,
    },))
      continue;
    if (entry.keySegments
      .length
      === rest.length) {
      return {
        kind: 'inline-table',
        entries: value.entries
          .filter(function notDropped(
            _e,
            i,
          ) {
            return i !== j;
          },),
        origin: { kind: 'synthetic', },
      };
    }
    /**
     Entry value after removing the deeper target.
     */
    const nested = removeInValue({
      value: entry.value,
      rest: rest.slice(entry.keySegments
        .length,),
    },);
    if (nested === NOT_REMOVED)
      return NOT_REMOVED;
    return {
      kind: 'inline-table',
      entries: value.entries
        .with(
        j,
        {
          ...entry,
          value: nested,
        },
        ),
      origin: { kind: 'synthetic', },
    };
  }
  return NOT_REMOVED;
}
