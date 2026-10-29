/**
 Positional comment attribution over the flat parse-time comment list.
 
 Comments are unattached in `toml-eslint-parser`; a contiguous block of
 comments immediately above an offset (single-line gaps only) is "attached",
 and a comment on the same line after an offset is "trailing". Used by the
 comment readers against the state's retained comment list and source.
 
 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { isAttachedGap, } from './build-comments.ts';
import type { TomlComment, } from './types.ts';

/**
 Comments attached immediately above `at`, in source order.
 
 @returns Computed result (`readonly TomlComment[]`).
 
 @example
 ```ts
 attachedCommentsAt({ comments, source, at: node.range[0], },);
 ```
 */
export function attachedCommentsAt(
  {
    comments,
    source,
    at,
  }: {
    readonly comments: readonly TomlComment[];
    readonly source: string;
    readonly at: number;
  },
): readonly TomlComment[] {
  /**
   Last comment ending before `at`; the backward walk starts here.
   */
  const initialIdx = comments.reduce(
    function step(
      acc,
      c,
      i,
    ) {
      return c.range[1]
        < at ? i : acc;
    },
    -1,
  );
  /**
   Cursor retreating to each attached comment's start as the block grows.
   */
  let cursorPos = at;
  /**
   Collected nearest-first; reversed to source order on return.
   */
  const collected: TomlComment[] = [];
  for (let idx = initialIdx; idx >= 0; idx--) {
    /**
     Candidate comment whose gap to the cursor decides attachment.
     */
    const comment = nonNullishOrThrow(comments[idx],);
    if (comment.range[1]
      >= cursorPos)
      continue;
    if (!isAttachedGap(source.slice(
      comment.range[1],
      cursorPos,
    ),))
      break;
    /**
     Start offset of this attached comment; the cursor retreats to it next.
     */
    const [commentStart,] = comment.range;
    collected.push(comment,);
    cursorPos = commentStart;
  }
  /**
   Nearest-first collection reversed to source order; returned as the binding
   so this build-via-mutation helper satisfies the no-function-root-let shape.
   */
  const ordered = collected.toReversed();
  return ordered;
}

/**
 Same-line trailing comment after `from`, if any.
 
 @returns Object whose `comment` field is the trailing comment, or absent.
 
 @example
 ```ts
 trailingCommentAt({ comments, source, from: kv.value.range[1], },);
 ```
 */
export function trailingCommentAt(
  {
    comments,
    source,
    from,
  }: {
    readonly comments: readonly TomlComment[];
    readonly source: string;
    readonly from: number;
  },
): { readonly comment?: TomlComment; } {
  /**
   Line end; the trailing comment must start before it.
   */
  const newlineAfter = source.indexOf(
    '\n',
    from,
  );
  /**
   Upper bound for a same-line match (EOF when the line runs to end).
   */
  const limit = newlineAfter === (-1) ? source.length : newlineAfter;
  /**
   First comment starting at or after `from` and before the line break.
   */
  const match = comments.find(function inLine(c,) {
    return (c.range[0]
      >= from) && (c.range[0]
        < limit);
  },);
  return match === undefined ? {} : { comment: match, };
}
