/**
 {@link tomlSet}: write a value at a path, returning a fresh state.
 
 Resolves the path against the current block tree and dispatches:
 replace an existing value; wholesale-replace an array-of-tables; replace a
 standard table's or the top-level body; replace an implicit dotted-key parent
 (issue #252); or create a fresh dotted key-value. All paths are pure: the new
 state shares unchanged nodes by reference.
 
 @module
 */

import type { Block, } from './document.ts';
import {
  TomlImmutableNodeError,
  TomlTypeError,
} from './errors.ts';
import { formatPath, } from './path.ts';
import { segmentsEqual, } from './path-prefix.ts';
import { doAotReplace, } from './set-aot.ts';
import { doCreate, } from './set-create.ts';
import {
  doImplicitReplace,
  doTableReplace,
  doTopLevelReplace,
  isImplicitConstituent,
} from './set-replace.ts';
import { NOT_SET, } from './set-value-inline.ts';
import { replaceExistingValue, } from './set-value.ts';
import type {
  TomlEditState,
  TomlPath,
} from './types.ts';

/**
 True when `block` is a table of `tableKind` whose header exactly names `path`.
 
 @returns Resulting boolean.
 */
function headerEquals(
  {
    block,
    path,
    tableKind,
  }: {
    readonly block: Block;
    readonly path: TomlPath;
    readonly tableKind: 'standard' | 'array';
  },
): boolean {
  return (block.kind
    === 'table')
    && (block.tableKind
      === tableKind)
    && segmentsEqual({
      left: block.headerSegments,
      right: path,
    },);
}

/**
 Set `value` at `path`, returning a fresh {@link TomlEditState}.
 
 @returns Fresh {@link TomlEditState}.
 
 @throws {@link TomlTypeError} when `value` is `null`/`undefined`, or when an
         object is required (table/implicit replace) but not supplied.
 
 @throws {@link TomlImmutableNodeError} when the path names multiple sibling
         standard tables under an implicit parent (set per sub-table instead).
 
 @example
 ```ts
 const e1 = tomlSet({ edit: e0, path: ['tools', 'bun'], value: 'latest', },);
 ```
 */
export function tomlSet(
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
  if ((value === null) || (value === undefined)) {
    throw new TomlTypeError(
      `Cannot set ${formatPath({ path, },)} to ${String(value,)}; use tomlDelete`,
    );
  }

  /**
   Fresh blocks when the path addressed an existing value; else the sentinel.
   */
  const replaced = replaceExistingValue({
    blocks: edit.blocks,
    path,
    value,
    options: edit.canonical,
  },);
  if (replaced !== NOT_SET)
    return {
      ...edit,
      blocks: replaced,
    };

  if (edit.blocks
    .some(function isAot(b,) {
      return headerEquals({
        block: b,
        path,
        tableKind: 'array',
      },);
    },))
    return doAotReplace({
      edit,
      path,
      value,
    },);

  /**
   Index of a standard table whose header exactly names the path.
   */
  const stdIndex = edit.blocks
    .findIndex(function isStd(b,) {
      return headerEquals({
        block: b,
        path,
        tableKind: 'standard',
      },);
    },);
  if (stdIndex !== (-1)) {
    /**
     The standard table whose body is replaced.
     */
    const table = edit.blocks[stdIndex];
    if ((table !== undefined) && (table.kind
      === 'table'))
      return doTableReplace({
        edit,
        tableIndex: stdIndex,
        table,
        path,
        value,
      },);
  }

  if (path.length
    === 0)
    return doTopLevelReplace({
      edit,
      value,
    },);

  /**
   Constituent entries of an implicit parent at this path, if any.
   */
  const constituents = edit.blocks
    .filter(function isConst(b,) {
      return isImplicitConstituent({
        block: b,
        path,
      },);
    },);
  if (constituents.length
    > 0) {
    if (constituents.some(function isTable(b,) {
      return b.kind
        === 'table';
    },)) {
      throw new TomlImmutableNodeError(
        `tomlSet on the sibling tables at ${
          formatPath({ path, },)
        } is not supported; set per sub-table instead`,
      );
    }
    return doImplicitReplace({
      edit,
      path,
      value,
    },);
  }

  return doCreate({
    edit,
    path,
    value,
  },);
}
