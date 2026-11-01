/**
 {@link tomlGetCommentAfter}: trailing inline comment on the same source line.
 
 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { trailingCommentAt, } from './comments.ts';
import type { TableNode, } from './document.ts';
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
 The same-line trailing comment for the entry at `path`.
 
 @returns Object whose `comment` field is the trailing comment, or absent.
 
 @throws {@link TomlPathNotFoundError} when `path` does not exist.
 
 @example
 ```toml
 key = "value"  # trailing
 ```
 */
export function tomlGetCommentAfter(
  {
    edit,
    path,
  }: {
    readonly edit: TomlEditState;
    readonly path: TomlPath;
  },
): { readonly comment?: TomlComment; } {
  /**
   Entry block at the path; its clean end offset anchors the trailing scan.
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
   Clean end offset after which a same-line comment attaches, or `-1`.
   */
  const from = endOf({ located, },);
  if (from === (-1))
    return {};
  return trailingCommentAt({
    comments: edit.comments,
    source: edit.source,
    from,
  },);
}

/**
 End of a clean table's closing bracket, before any same-line comment.

 The stored header span includes the entire line and newline, so scanning
 from its end would miss a comment on the header itself.

 @param table - Table whose retained parse-time key anchors comment scanning.

 @returns Key end offset, or `-1` for a synthetic or mismatched header.

 @example
 ```ts
 cleanTableKeyEnd(table); // bracket end before trailing comment
 ```
 */
function cleanTableKeyEnd(table: TableNode,): number {
  if (table.headerOrigin
    .kind
    !== 'clean')
    return -1;
  if (table.headerOrigin
    .astNode
    .type
    !== 'TOMLTable')
    return -1;
  return table.headerOrigin
    .astNode
    .key
    .range[1];
}

/**
 Clean end offset for a located entry, or `-1` when synthetic.

 Char offsets are non-negative, so `-1` unambiguously signals "no clean source
 position" without a nullish union.

 @param located - Located block whose clean end offset anchors the scan.

 @returns Clean end offset, or `-1` when synthetic.

 @example
 ```ts
 endOf({ located: locateBlock({ blocks, path, },), },);
 ```
 */
function endOf(
  { located, }: { readonly located: ReturnType<typeof locateBlock>; },
): number {
  if (located === NOT_LOCATED)
    return -1;
  if (located.kind
    === 'kv')
    return located.kv
      .valueRange
      === undefined ? -1 : located.kv
        .valueRange[1];
  if (located.kind
    === 'table')
    return cleanTableKeyEnd(located.table,);
  /**
   Last array-of-tables instance is where a trailing comment attaches.
   */
  const last = nonNullishOrThrow(located.tables
    .at(-1,),);
  return cleanTableKeyEnd(last,);
}
