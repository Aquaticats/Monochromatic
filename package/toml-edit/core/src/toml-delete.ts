/**
 {@link tomlDelete}: remove a key, table, array element, or implicit parent at
 `path`, returning a fresh {@link TomlEditState}.
 
 Dispatch over the current block tree: remove an existing value or entry;
 remove the standard/array table section(s) named exactly by the path; or
 remove every constituent of an implicit dotted-key parent (issue #252). A
 path that resolves to nothing is a no-op.
 
 @module
 */

import type { Block, } from './document.ts';
import {
  NOT_REMOVED,
  removeAtPath,
} from './delete-value.ts';
import { segmentsEqual, } from './path-prefix.ts';
import { isImplicitConstituent, } from './set-replace.ts';
import type {
  TomlEditState,
  TomlPath,
} from './types.ts';

/**
 True when `block` is a table section whose header exactly names `path`.
 
 @returns Resulting boolean.
 */
function isExactTable(
  {
    block,
    path,
  }: {
    readonly block: Block;
    readonly path: TomlPath;
  },
): boolean {
  return (block.kind
    === 'table')
    && segmentsEqual({
      left: block.headerSegments,
      right: path,
    },);
}

/**
 Delete the entry at `path`.
 
 @returns Fresh {@link TomlEditState} (unchanged when the path is absent).
 
 @example
 ```ts
 tomlDelete({ edit, path: ['old'], },);
 tomlDelete({ edit, path: ['fruits'], },);   // every [[fruits]] instance
 tomlDelete({ edit, path: ['arr', 1], },);   // element at index 1
 ```
 */
export function tomlDelete(
  {
    edit,
    path,
  }: {
    readonly edit: TomlEditState;
    readonly path: TomlPath;
  },
): TomlEditState {
  if (path.length
    === 0)
    return edit;

  /**
   Fresh blocks when an existing value/entry was removed; else the sentinel.
   */
  const removed = removeAtPath({
    blocks: edit.blocks,
    path,
  },);
  if (removed !== NOT_REMOVED)
    return {
      ...edit,
      blocks: removed,
    };

  if (edit.blocks
    .some(function isExact(b,) {
      return isExactTable({
        block: b,
        path,
      },);
    },))
    return {
      ...edit,
      blocks: edit.blocks
        .filter(function keep(b,) {
          return !isExactTable({
            block: b,
            path,
          },);
        },),
    };

  if (edit.blocks
    .some(function isConst(b,) {
      return isImplicitConstituent({
        block: b,
        path,
      },);
    },))
    return {
      ...edit,
      blocks: edit.blocks
        .filter(function keep(b,) {
          return !isImplicitConstituent({
            block: b,
            path,
          },);
        },),
    };

  return edit;
}
