/**
 Whole-container replacements for {@link tomlSet}: replace a standard table's
 body, the top-level body, or an implicit dotted-key parent.
 
 Each requires a plain object value (mirroring table-body semantics); anything
 else throws {@link TomlTypeError}. The implicit-parent case deletes the
 constituent entries under the path and inserts the object's entries as dotted
 keys (issue #252 defect: set over an implicit parent).
 
 @module
 */

import type {
  Block,
  KeyValueNode,
  TableNode,
} from './document.ts';
import { TomlTypeError, } from './errors.ts';
import { formatPath, } from './path.ts';
import { makeKeyValue, } from './set-create.ts';
import type {
  TomlEditState,
  TomlPath,
} from './types.ts';
import { isPlainObject, } from './value-encoders.ts';

/**
 Require `value` to be a plain object, else throw a table-replace type error.
 
 @returns The value narrowed to a record.
 
 @throws {@link TomlTypeError} when `value` is not a plain object.
 
 @mutates value - Plain-object validation can invoke caller-owned proxy prototype hooks.
 */
function requireObject(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: TomlPath;
  },
): Record<string, unknown> {
  if (!isPlainObject(value,)) {
    /**
     Empty path names the document itself rather than an unnamed table.
     */
    const target = path.length === 0 ? 'document root' : formatPath({ path, },);
    /**
     Container description agreeing with the addressed target.
     */
    const body = path.length === 0 ? 'document body' : 'a table body';
    throw new TomlTypeError(
      `tomlSet at ${target} requires a plain object to replace ${body}`,
    );
  }
  return value;
}

/**
 New synthetic key-value blocks for each entry of `value`.
 
 @returns Computed blocks.
 
 @mutates value - `Object.entries` and recursive value building can invoke caller-owned proxy and accessor hooks.
 */
function entryBlocks(
  {
    value,
    prefix,
    options,
  }: {
    readonly value: Record<string, unknown>;
    readonly prefix: readonly string[];
    readonly options: TomlEditState['canonical'];
  },
): readonly KeyValueNode[] {
  return Object.entries(value,)
    .map(function each([key, v,],) {
      return makeKeyValue({
        segments: [
          ...prefix,
          key,
        ],
        value: v,
        options,
      },);
    },);
}

/**
 Replace the body of the standard table at `tableIndex` with `value`'s entries.
 
 @returns Fresh {@link TomlEditState}.
 
 @throws {@link TomlTypeError} when `value` is not a plain object.
 
 @mutates value - Validation and entry building can invoke caller-owned proxy and accessor hooks.
 
 @example
 ```ts
 doTableReplace({ edit, tableIndex: 0, table, path: ['t'], value: { a: 1, }, },);
 ```
 */
export function doTableReplace(
  {
    edit,
    tableIndex,
    table,
    path,
    value,
  }: {
    readonly edit: TomlEditState;
    readonly tableIndex: number;
    readonly table: TableNode;
    readonly path: TomlPath;
    readonly value: unknown;
  },
): TomlEditState {
  /**
   Object body entries so the table's direct key-values can be rebuilt.
   */
  const object = requireObject({
    value,
    path,
  },);
  return {
    ...edit,
    blocks: edit.blocks
      .with(
      tableIndex,
      {
        ...table,
        body: entryBlocks({
          value: object,
          prefix: [],
          options: edit.canonical,
        },),
      },
      ),
  };
}

/**
 Replace the top-level body: swap all top-level key-values for `value`'s
 entries, keeping table sections and fillers.
 
 @returns Fresh {@link TomlEditState}.
 
 @throws {@link TomlTypeError} when `value` is not a plain object.
 
 @mutates value - Validation and entry building can invoke caller-owned proxy and accessor hooks.
 
 @example
 ```ts
 doTopLevelReplace({ edit, value: { title: 'x', }, },);
 ```
 */
export function doTopLevelReplace(
  {
    edit,
    value,
  }: {
    readonly edit: TomlEditState;
    readonly value: unknown;
  },
): TomlEditState {
  /**
   Object whose entries become the new top-level key-values.
   */
  const object = requireObject({
    value,
    path: [],
  },);
  /**
   Table sections and fillers kept as-is (only bare key-values are replaced).
   */
  const kept = edit.blocks
    .filter(function keep(b,) {
      return b.kind
        !== 'keyvalue';
    },);
  return {
    ...edit,
    blocks: [
      ...entryBlocks({
        value: object,
        prefix: [],
        options: edit.canonical,
      },),
      ...kept,
    ],
  };
}

/**
 True when `block` is a constituent of the implicit parent at `path` (a
 top-level key-value or table whose absolute key strictly extends `path`).
 
 @returns Resulting boolean.
 
 @example
 ```ts
 isImplicitConstituent({ block, path: ['a'], },);
 ```
 */
export function isImplicitConstituent(
  {
    block,
    path,
  }: {
    readonly block: Block;
    readonly path: TomlPath;
  },
): boolean {
  /**
   Absolute key segments of the block (key-value key, or table header).
   */
  const segs = block.kind
    === 'keyvalue'
    ? block.keySegments
    : block.kind
      === 'table'
      ? block.headerSegments
      : null;
  if (segs === null)
    return false;
  return (segs.length
    > path.length)
    && path.every(function eq(
      seg,
      i,
    ) {
      return seg === segs[i];
    },);
}

/**
 Replace an implicit dotted-key parent: delete its constituent entries and
 insert `value`'s entries as dotted keys under `path`.
 
 @returns Fresh {@link TomlEditState}.
 
 @throws {@link TomlTypeError} when `value` is not a plain object.
 
 @mutates value - Validation and entry building can invoke caller-owned proxy and accessor hooks.
 
 @example
 ```ts
 doImplicitReplace({ edit, path: ['a'], value: { b: 1, }, },);
 ```
 */
export function doImplicitReplace(
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
   Object whose entries become dotted keys under the implicit parent path.
   */
  const object = requireObject({
    value,
    path,
  },);
  /**
   String-form prefix so the new dotted keys extend the parent path.
   */
  const prefix = path.map(function stringifySegment(segment,) {
    return (typeof segment) === 'number' ? String(segment,) : segment;
  },);
  /**
   Blocks with the implicit parent's constituents removed.
   */
  const kept = edit.blocks
    .filter(function keep(b,) {
      return !isImplicitConstituent({
        block: b,
        path,
      },);
    },);
  /**
   First table header in the kept blocks; new keys must precede it.
   */
  const firstTable = kept.findIndex(function isTable(b,) {
    return b.kind
      === 'table';
  },);
  /**
   New dotted key-value blocks for the object entries.
   */
  const inserts = entryBlocks({
    value: object,
    prefix,
    options: edit.canonical,
  },);
  if (firstTable === (-1))
    return {
      ...edit,
      blocks: [
        ...kept,
        ...inserts,
      ],
    };
  return {
    ...edit,
    blocks: [
      ...kept.slice(
        0,
        firstTable,
      ),
      ...inserts,
      ...kept.slice(firstTable,),
    ],
  };
}
