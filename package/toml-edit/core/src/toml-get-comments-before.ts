/**
 {@link tomlGetCommentsBefore}: attached comment block immediately before a path.
 
 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { attachedCommentsAt, } from './comments.ts';
import { TomlPathNotFoundError, } from './errors.ts';
import { formatPath, } from './path.ts';
import { locateBlock, } from './resolve-block.ts';
import { NOT_LOCATED, } from './resolve-document.ts';
import type {
  TomlComment,
  TomlEditState,
  TomlPath,
} from './types.ts';

/**
 The contiguous block of comments immediately preceding the entry at `path`,
 with no blank line between any pair. A comment separated by a blank line is
 not attached; a created (synthetic) entry has no source comments.
 
 @returns Computed result (`readonly TomlComment[]`).
 
 @throws {@link TomlPathNotFoundError} when `path` does not exist.
 
 @example
 ```toml
 # attached
 key = 1
 ```
 */
export function tomlGetCommentsBefore(
  {
    edit,
    path,
  }: {
    readonly edit: TomlEditState;
    readonly path: TomlPath;
  },
): readonly TomlComment[] {
  /**
   Entry block at the path; its clean start offset anchors the comment scan.
   */
  const located = locateBlock({
    blocks: edit.blocks,
    path,
  },);
  if (located === NOT_LOCATED) {
    throw new TomlPathNotFoundError(
      `Path ${formatPath({ path, },)} not found`,
    );
  }
  /**
   Clean start offset of the entry, or `-1` when the entry is synthetic.
   */
  const start = startOf({ located, },);
  if (start === (-1))
    return [];
  return attachedCommentsAt({
    comments: edit.comments,
    source: edit.source,
    at: start,
  },);
}

/**
 Clean start offset for a located entry, or `-1` when synthetic.
 
 Char offsets are non-negative, so `-1` unambiguously signals "no clean source
 position" without a nullish union.
 
 @param located - Located block whose clean start offset anchors the scan.
 
 @returns Clean start offset, or `-1` when synthetic.
 
 @example
 ```ts
 startOf({ located: locateBlock({ blocks, path, },), },);
 ```
 */
function startOf(
  { located, }: { readonly located: ReturnType<typeof locateBlock>; },
): number {
  if (located === NOT_LOCATED)
    return -1;
  if (located.kind
    === 'kv')
    return located.kv
      .origin
      .kind
      === 'clean' ? located.kv
        .origin
        .range[0] : -1;
  if (located.kind
    === 'table')
    return located.table
      .headerOrigin
      .kind
      === 'clean' ? located.table
        .headerOrigin
        .range[0] : -1;
  /**
   First array-of-tables instance is where a preceding block attaches.
   */
  const first = nonNullishOrThrow(located.tables[0],);
  return first.headerOrigin
    .kind
    === 'clean' ? first.headerOrigin
      .range[0] : -1;
}
