/**
 {@link tomlInsertCommentBefore}: add a comment block immediately before a path.
 
 @module
 */

import type { Block, } from './document.ts';
import { TomlPathNotFoundError, } from './errors.ts';
import { formatPath, } from './path.ts';
import {
  isStrictPrefix,
  segmentsEqual,
} from './path-prefix.ts';
import type {
  TomlEditState,
  TomlPath,
} from './types.ts';

/**
 Sentinel for "the target block was not found".
 */
const NOT_FOUND: unique symbol = Symbol('toml-edit/insert-before-not-found',);

/**
 Insert `#`-prefixed comment lines as a filler block just before the entry
 at `path`.
 
 @returns A fresh {@link TomlEditState} reflecting the change.
 
 @throws {@link TomlPathNotFoundError} when `path` does not exist.
 
 @example
 ```ts
 tomlInsertCommentBefore({ edit, path: ['version'], comment: ' bumped', },);
 ```
 */
export function tomlInsertCommentBefore(
  {
    edit,
    path,
    comment,
  }: {
    readonly edit: TomlEditState;
    readonly path: TomlPath;
    readonly comment: string | readonly string[];
  },
): TomlEditState {
  /**
   Comment lines rendered as their own physical lines.
   */
  const text = ((typeof comment) === 'string' ? [comment,] : comment)
    .map(function withHash(line,) {
      return `# ${line}\n`;
    },)
    .join('',);
  /**
   Blocks with a filler spliced before the target, or the not-found sentinel.
   */
  const inserted = insertFillerBefore({
    blocks: edit.blocks,
    path,
    text,
  },);
  if (inserted === NOT_FOUND) {
    throw new TomlPathNotFoundError(
      `Path ${formatPath({ path, },)} not found`,
    );
  }
  return {
    ...edit,
    blocks: inserted,
  };
}

/**
 Splice a filler block before the entry named by `path`.
 
 @returns Fresh blocks, or {@link NOT_FOUND}.
 */
function insertFillerBefore(
  {
    blocks,
    path,
    text,
  }: {
    readonly blocks: readonly Block[];
    readonly path: TomlPath;
    readonly text: string;
  },
): readonly Block[] | typeof NOT_FOUND {
  for (const [index, block,] of blocks.entries()) {
    if (matchesExact({
      block,
      path,
    },))
      return [
        ...blocks.slice(
          0,
          index,
        ),
        {
          kind: 'filler',
          text,
        },
        ...blocks.slice(index,),
      ];
    if ((block.kind
      === 'table')
      && (block.tableKind
        === 'standard')
      && isStrictPrefix({
        candidate: block.headerSegments,
        path,
      },)) {
      /**
       Body after recursing with the header-relative path.
       */
      const newBody = insertFillerBefore({
        blocks: block.body,
        path: path.slice(block.headerSegments
          .length,),
        text,
      },);
      if (newBody !== NOT_FOUND)
        return blocks.with(
          index,
          {
            ...block,
            body: newBody,
          },
        );
    }
  }
  return NOT_FOUND;
}

/**
 True when a block's key/header exactly names `path`.
 
 @returns Resulting boolean.
 */
function matchesExact(
  {
    block,
    path,
  }: {
    readonly block: Block;
    readonly path: TomlPath;
  },
): boolean {
  /**
   The block's own key segments, or `null` for a filler.
   */
  const segs = block.kind
    === 'keyvalue'
    ? block.keySegments
    : block.kind
      === 'table'
      ? block.headerSegments
      : null;
  return (segs !== null)
    && segmentsEqual({
      left: segs,
      right: path,
    },);
}
