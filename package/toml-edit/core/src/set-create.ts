/**
 Path-create for {@link tomlSet}: insert a fresh dotted key-value when the path
 does not resolve to an existing value.
 
 A new key-value is placed inside the deepest standard table whose header is a
 prefix of the path, else at top level before the first table header (the TOML
 grammar requires bare keys to precede section headers). The inserted block is
 synthetic and renders canonically.
 
 @module
 */

import { buildValueFromInput, } from './build-input.ts';
import type {
  Block,
  KeyValueNode,
} from './document.ts';
import { TomlImmutableNodeError, } from './errors.ts';
import { formatPath, } from './path.ts';
import type {
  CanonicalOptions,
  TomlEditState,
  TomlPath,
} from './types.ts';

/**
 Build a synthetic key-value block for `segments = value`.
 
 @returns Computed {@link KeyValueNode}.
 
 @example
 ```ts
 makeKeyValue({ segments: ['a','b'], value: 1, options, },);
 ```
 */
export function makeKeyValue(
  {
    segments,
    value,
    options,
  }: {
    readonly segments: readonly string[];
    readonly value: unknown;
    readonly options: CanonicalOptions;
  },
): KeyValueNode {
  return {
    kind: 'keyvalue',
    keySegments: segments,
    value: buildValueFromInput({
      input: value,
      options,
    },),
    origin: { kind: 'synthetic', },
    commentsBefore: [],
  };
}

/**
 Create a new key-value at a currently-absent `path`.
 
 @returns Fresh {@link TomlEditState}.
 
 @throws {@link TomlImmutableNodeError} when the remaining tail has a numeric segment.
 
 @example
 ```ts
 doCreate({ edit, path: ['a','b','c'], value: 42, },);
 ```
 */
export function doCreate(
  {
    edit,
    path,
    value,
  }: {
    readonly edit: TomlEditState;
    readonly path: TomlPath;
    readonly value: unknown;
  },
): TomlEditState {
  assertNoScalarPrefix({
    blocks: edit.blocks,
    path,
    basePath: [],
  },);
  /**
   Index of the deepest standard table whose header is a strict prefix of path.
   */
  const tableIndex = deepestTableIndex({
    blocks: edit.blocks,
    path,
  },);
  if (tableIndex === (-1))
    return createAtTopLevel({
      edit,
      path,
      value,
    },);
  /**
   The owning table block; its body gains the new dotted key-value.
   */
  const table = edit.blocks[tableIndex];
  if ((table === undefined) || (table.kind
    !== 'table'))
    return createAtTopLevel({
      edit,
      path,
      value,
    },);
  /**
   Dotted tail relative to the table header.
   */
  const segments = dottedTail({
    path,
    consumed: table.headerSegments
      .length,
  },);
  return {
    ...edit,
    blocks: edit.blocks
      .with(
      tableIndex,
      {
        ...table,
        body: [
          ...table.body,
          makeKeyValue({
            segments,
            value,
            options: edit.canonical,
          },),
        ],
      },
      ),
  };
}

/**
 Insert a new top-level dotted key-value before the first table header.
 
 @returns Fresh {@link TomlEditState}.
 */
function createAtTopLevel(
  {
    edit,
    path,
    value,
  }: {
    readonly edit: TomlEditState;
    readonly path: TomlPath;
    readonly value: unknown;
  },
): TomlEditState {
  /**
   New key-value block for the full path (all segments are top-level here).
   */
  const kv = makeKeyValue({
    segments: dottedTail({
      path,
      consumed: 0,
    },),
    value,
    options: edit.canonical,
  },);
  /**
   First table header index; the key-value must land before it.
   */
  const firstTable = edit.blocks
    .findIndex(function isTable(b,) {
      return b.kind
        === 'table';
    },);
  /**
   Leading newline filler when the source does not already end on a line break.
   */
  const needsNewline = (firstTable === (-1))
    && (edit.source !== '')
    && (!edit.source
      .endsWith('\n',));
  /**
   Blocks to splice in: the key-value, preceded by a newline filler if needed.
   */
  const inserted: readonly Block[] = needsNewline
    ? [
      {
        kind: 'filler',
        text: '\n',
      },
      kv,
    ]
    : [kv,];
  if (firstTable === (-1))
    return {
      ...edit,
      blocks: [
        ...edit.blocks,
        ...inserted,
      ],
    };
  return {
    ...edit,
    blocks: [
      ...edit.blocks
        .slice(
        0,
        firstTable,
        ),
      kv,
      ...edit.blocks
        .slice(firstTable,),
    ],
  };
}

/**
 Throw when `path` would be created under an existing scalar or array leaf
 (a key-value whose absolute key strictly prefixes `path` and whose value is
 not a table shape). Path-create cannot descend through such a value.
 
 @throws {@link TomlImmutableNodeError} on a scalar/array prefix conflict.
 */
function assertNoScalarPrefix(
  {
    blocks,
    path,
    basePath,
  }: {
    readonly blocks: readonly Block[];
    readonly path: TomlPath;
    readonly basePath: readonly (string | number)[];
  },
): void {
  for (const block of blocks) {
    if (block.kind
      === 'keyvalue') {
      /**
       Absolute key of this entry so the prefix test spans table context.
       */
      const abs = [
        ...basePath,
        ...block.keySegments,
      ];
      if ((abs.length
        < path.length)
        && abs.every(function eq(
          seg,
          i,
        ) {
          return seg === path[i];
        },)
        && (block.value
          .kind
          !== 'inline-table')) {
        throw new TomlImmutableNodeError(
          `Cannot path-create through ${block.value
            .kind} at path ${formatPath({ path, },)}`,
        );
      }
      continue;
    }
    if (block.kind
      === 'table')
      assertNoScalarPrefix({
        blocks: block.body,
        path,
        basePath: block.headerSegments,
      },);
  }
}

/**
 Index of the deepest standard table whose header strictly prefixes `path`.
 
 @returns Block index, or `-1` when none.
 */
function deepestTableIndex(
  {
    blocks,
    path,
  }: {
    readonly blocks: readonly Block[];
    readonly path: TomlPath;
  },
): number {
  return blocks.reduce(
    function step(
      best,
      block,
      index,
    ) {
      if ((block.kind
        !== 'table')
        || (block.tableKind
          !== 'standard'))
        return best;
      /**
       Header length; a strict prefix qualifies this table as a container.
       */
      const len = block.headerSegments
        .length;
      if ((len
        >= path.length)
        || (!block.headerSegments
          .every(function eq(
            seg,
            i,
          ) {
            return seg === path[i];
          },)))
        return best;
      /**
       Current best header length so the longest (deepest) prefix wins.
       */
      const bestLen = (best === (-1)) || (blocks[best] === undefined)
        || (blocks[best]
          ?.kind
          !== 'table')
        ? -1
        : (blocks[best] as Extract<Block, { kind: 'table'; }>).headerSegments
          .length;
      return len
        > bestLen ? index : best;
    },
    -1,
  );
}

/**
 Remaining path segments after `consumed`, verified all-string.
 
 @returns Computed dotted segments.
 
 @throws {@link TomlImmutableNodeError} when a remaining segment is numeric.
 */
function dottedTail(
  {
    path,
    consumed,
  }: {
    readonly path: TomlPath;
    readonly consumed: number;
  },
): readonly string[] {
  return path.slice(consumed,)
    .map(function asString(seg,) {
      if ((typeof seg) !== 'string') {
        throw new TomlImmutableNodeError(
          `Cannot path-create at numeric segment ${String(seg,)} in ${formatPath({ path, },)}`,
        );
      }
      return seg;
    },);
}
