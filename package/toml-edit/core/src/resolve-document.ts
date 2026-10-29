/**
 Structural resolution over the current block tree.
 
 {@link locateValueNode} finds the value node a path addresses (a key-value's
 value, an element inside an array/inline table, or a whole table section),
 used by the parse-time-view readers {@link tomlGetNode} / {@link tomlGetRaw}.
 Because it walks the current tree, only clean nodes carry a source range.
 
 @module
 */

import type {
  Block,
  KeyValueNode,
  TableNode,
  ValueNode,
} from './document.ts';
import {
  isPrefix,
  isStrictPrefix,
  segmentsEqual,
} from './path-prefix.ts';
import type { TomlPath, } from './types.ts';

/**
 A table section a path names directly.
 
 `table`: the path names a standard `[foo]` section or one indexed `[[foo]]` instance.
 `aot`: the path names the whole array-of-tables collection.
 */
export type TableSectionHit =
  | {
    readonly kind: 'table';
    readonly table: TableNode
  }
    | {
      readonly kind: 'aot';
      readonly tables: readonly TableNode[]
    };

/**
 Instruction to resume resolution inside a parent section's body.
 */
export type TableSectionDescent = {
  readonly kind: 'descend';
  readonly blocks: readonly Block[];
  readonly path: TomlPath;
};

/**
 A resolved structural location.
 
 `value`: the path addresses a key-value's value or a nested element.
 */
export type Located =
  | {
    readonly kind: 'value';
    readonly value: ValueNode
  }
    | TableSectionHit;

/**
 Sentinel for "no structural location at this path".
 */
export const NOT_LOCATED: unique symbol = Symbol('toml-edit/resolve-block-not-located',);

/**
 Locate the value node (or table) that `path` addresses in `blocks`.
 
 @returns A {@link Located} result, or {@link NOT_LOCATED}.
 
 @example
 ```ts
 locateValueNode({ blocks: edit.blocks, path: ['tools', 'bun'], },);
 ```
 */
export function locateValueNode(
  {
    blocks,
    path,
  }: {
    readonly blocks: readonly Block[];
    readonly path: TomlPath;
  },
): Located | typeof NOT_LOCATED {
  /**
   Key-value whose key chain is a prefix of `path`, resolved first so a direct
   or dotted hit short-circuits the table scan.
   */
  const kvHit = matchKeyValue({
    blocks,
    path,
  },);
  if (kvHit !== NOT_LOCATED) {
    if (kvHit.matched
      === path.length)
      return {
        kind: 'value',
        value: kvHit.kv
          .value,
      };
    return descendValue({
      value: kvHit.kv
        .value,
      rest: path.slice(kvHit.matched,),
    },);
  }
  /**
   Section scan shared with {@link locateBlock}; only the descent target differs.
   */
  const section = matchTableSection({
    blocks,
    path,
  },);
  if (section === NOT_LOCATED)
    return NOT_LOCATED;
  if (section.kind
    === 'descend')
    return locateValueNode(section,);
  return section;
}

/**
 Find a key-value block whose key segments prefix `path`.
 
 @returns Match plus its matched length, or {@link NOT_LOCATED}.
 */
function matchKeyValue(
  {
    blocks,
    path,
  }: {
    readonly blocks: readonly Block[];
    readonly path: TomlPath;
  },
): {
  readonly kv: KeyValueNode;
  readonly matched: number
} | typeof NOT_LOCATED {
  for (const block of blocks) {
    if (block.kind
      !== 'keyvalue')
      continue;
    if (isPrefix({
      candidate: block.keySegments,
      path,
    },))
      return {
        kv: block,
        matched: block.keySegments
          .length,
      };
  }
  return NOT_LOCATED;
}

/**
 Resolve `path` against the standard and array table sections in `blocks`.
 
 Shared by both resolvers: an exact header yields a {@link TableSectionHit},
 a strict-prefix standard header yields a {@link TableSectionDescent} that the
 caller replays against its own entry point.
 
 @returns Section hit, descent instruction, or {@link NOT_LOCATED}.
 
 @example
 ```ts
 matchTableSection({ blocks: edit.blocks, path: ['tools'], },);
 ```
 */
export function matchTableSection(
  {
    blocks,
    path,
  }: {
    readonly blocks: readonly Block[];
    readonly path: TomlPath;
  },
): TableSectionHit | TableSectionDescent | typeof NOT_LOCATED {
  /**
   Table sections whose header exactly names `path`.
   */
  const exact = blocks.filter(function isExact(b,): b is TableNode {
    return (b.kind
      === 'table')
      && segmentsEqual({
        left: b.headerSegments,
        right: path,
      },);
  },);
  if (exact.length
    > 0) {
    /**
     First exact table so a standard header resolves to that single section.
     */
    const [first,] = exact;
    if ((first !== undefined) && (first.tableKind
      === 'standard'))
      return {
        kind: 'table',
        table: first,
      };
    return {
      kind: 'aot',
      tables: exact,
    };
  }
  /**
   Array-of-tables instance selected by the numeric segment after its header.
   */
  const indexed = blocks.find(function isIndexedAot(b,): b is TableNode {
    if (b.kind !== 'table')
      return false;
    if (b.tableKind !== 'array')
      return false;
    if (!isStrictPrefix({
      candidate: b.headerSegments,
      path,
    },))
      return false;
    return path[b.headerSegments
      .length] === b.aotIndex;
  },);
  if (indexed !== undefined) {
    /**
     Segment count of the selected instance header.
     */
    const headerLength = indexed.headerSegments
      .length;
    /**
     Body path starts after the header and numeric instance segment.
     */
    const bodyStart = headerLength + 1;
    if (path.length === bodyStart)
      return {
        kind: 'table',
        table: indexed,
      };
    return {
      kind: 'descend',
      blocks: indexed.body,
      path: path.slice(bodyStart,),
    };
  }
  /**
   Standard table whose header is a strict prefix of `path`; descend its body.
   */
  const parent = blocks.find(function isPrefixTable(b,): b is TableNode {
    return (b.kind
      === 'table')
      && (b.tableKind
        === 'standard')
      && isStrictPrefix({
        candidate: b.headerSegments,
        path,
      },);
  },);
  if (parent !== undefined)
    return {
      kind: 'descend',
      blocks: parent.body,
      path: path.slice(parent.headerSegments
        .length,),
    };
  return NOT_LOCATED;
}

/**
 Descend into a value node with the remaining path segments.
 
 @returns A {@link Located} value, or {@link NOT_LOCATED}.
 */
function descendValue(
  {
    value,
    rest,
  }: {
    readonly value: ValueNode;
    readonly rest: TomlPath;
  },
): Located | typeof NOT_LOCATED {
  if (rest.length
    === 0)
    return {
      kind: 'value',
      value,
    };
  /**
   Leading segment selecting the next child.
   */
  const [head, ...tail] = rest;
  if ((value.kind
    === 'array') && ((typeof head) === 'number')) {
    /**
     Selected element, or `undefined` for an out-of-range index.
     */
    const element = value.elements[head];
    if (element === undefined)
      return NOT_LOCATED;
    return descendValue({
      value: element,
      rest: tail,
    },);
  }
  if (value.kind
    === 'inline-table') {
    /**
     Inline entry whose key chain prefixes the remaining segments.
     */
    const hit = matchKeyValue({
      blocks: value.entries,
      path: rest,
    },);
    if (hit === NOT_LOCATED)
      return NOT_LOCATED;
    if (hit.matched
      === rest.length)
      return {
        kind: 'value',
        value: hit.kv
          .value,
      };
    return descendValue({
      value: hit.kv
        .value,
      rest: rest.slice(hit.matched,),
    },);
  }
  return NOT_LOCATED;
}
