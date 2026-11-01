/**
 {@link tomlInsertCommentAfter}: add a same-line trailing inline comment.
 
 @module
 */

import type {
  Block,
  KeyValueNode,
} from './document.ts';
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
 Sentinel for "the target key-value was not found".
 */
const NOT_FOUND: unique symbol = Symbol('toml-edit/insert-after-not-found',);

/**
 Append a same-line inline `# <comment>` after the value at `path`.
 
 @returns A fresh {@link TomlEditState} reflecting the change.
 
 @throws {@link TomlPathNotFoundError} when `path` does not name a key-value.
 
 @example
 ```ts
 tomlInsertCommentAfter({ edit, path: ['version'], comment: ' bumped', },);
 ```
 */
export function tomlInsertCommentAfter(
  {
    edit,
    path,
    comment,
  }: {
    readonly edit: TomlEditState;
    readonly path: TomlPath;
    readonly comment: string;
  },
): TomlEditState {
  /**
   Blocks with the trailing comment set on the target key-value.
   */
  const updated = setTrailing({
    blocks: edit.blocks,
    path,
    append: `  # ${comment}`,
  },);
  if (updated === NOT_FOUND) {
    throw new TomlPathNotFoundError(
      `Path ${formatPath({ path, },)} not found`,
    );
  }
  return {
    ...edit,
    blocks: updated,
  };
}

/**
 Set `append` as the trailing comment on the key-value named by `path`.
 
 @returns Fresh blocks, or {@link NOT_FOUND}.
 */
function setTrailing(
  {
    blocks,
    path,
    append,
  }: {
    readonly blocks: readonly Block[];
    readonly path: TomlPath;
    readonly append: string;
  },
): readonly Block[] | typeof NOT_FOUND {
  for (const [index, block,] of blocks.entries()) {
    if ((block.kind
      === 'keyvalue')
      && segmentsEqual({
        left: block.keySegments,
        right: path,
      },)) {
      /**
       Key-value carrying the inserted trailing comment.
       */
      const updated: KeyValueNode = {
        ...block,
        trailingCommentAppend: append,
      };
      return blocks.with(
        index,
        updated,
      );
    }
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
      const newBody = setTrailing({
        blocks: block.body,
        path: path.slice(block.headerSegments
          .length,),
        append,
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
