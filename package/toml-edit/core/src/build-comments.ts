/**
 Build-time helpers over the flat `program.comments` list and source: compute
 an entry's physical line end, and extract attached / trailing comment text.
 
 Comments in `toml-eslint-parser` are unattached (a sorted `program.comments`
 array); attribution is positional. A contiguous block of comments
 immediately above a node with no blank line between is "attached"; a comment
 on the same line after the node's value is the "trailing" comment. Extracted
 text is the comment value (the bytes after `#`), used for reads and synthetic
 rendering; clean nodes still emit the real bytes verbatim.
 
 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { TomlComment, } from './types.ts';

/**
 Offset just past the line terminator at or after `from` (EOF when none).
 
 A same-line trailing comment sits before that newline, so this span end
 absorbs it and the newline for free.
 
 @returns Computed offset.
 
 @example
 ```ts
 lineEndAfter({ source: 'a = 1\n', from: 5, },); // 6
 ```
 */
export function lineEndAfter(
  {
    source,
    from,
  }: {
    readonly source: string;
    readonly from: number;
  },
): number {
  /**
   First newline at or after `from`; `-1` means the line runs to EOF.
   */
  const nl = source.indexOf(
    '\n',
    from,
  );
  return nl === (-1) ? source.length : nl + 1;
}

/**
 Whitespace-only gap containing exactly one newline (single line break, no
 blank line), the shape that keeps a preceding comment "attached".
 
 @param gap - Source slice between a comment's end and the following node;
   a lone newline (and nothing else non-blank) keeps the comment attached.
 
 @returns Whether the gap attaches.
 
 @example
 ```ts
 isAttachedGap('\n',);   // true
 isAttachedGap('\n\n',); // false
 ```
 */
export function isAttachedGap(gap: string,): boolean {
  /**
   Newline count via split, avoiding a string spread; a single one keeps
   attachment (a blank line, i.e. two newlines, breaks it).
   */
  const newlineCount = gap.split('\n',)
    .length
    - 1;
  /**
   Gap with every whitespace char stripped; empty means whitespace-only, so
   any leftover char (a non-blank token between the comment and the node)
   breaks attachment.
   */
  const withoutWhitespace = gap
    .replaceAll(
    ' ',
    '',
    )
    .replaceAll(
    '\t',
    '',
    )
    .replaceAll(
    '\n',
    '',
    );
  return (withoutWhitespace === '')
    && (newlineCount === 1);
}

/**
 Comment values (bytes after `#`) attached immediately above `at`, in source
 order.
 
 @returns Computed comment-value strings.
 
 @example
 ```ts
 attachedCommentValues({ comments, source, at: kv.range[0], },); // [' header']
 ```
 */
export function attachedCommentValues(
  {
    comments,
    source,
    at,
  }: {
    readonly comments: readonly TomlComment[];
    readonly source: string;
    readonly at: number;
  },
): readonly string[] {
  /**
   Index of the last comment ending before `at`; the backward walk starts here.
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
  const collected: string[] = [];
  for (let idx = initialIdx; idx >= 0; idx--) {
    /**
     Candidate comment so the gap check can decide whether it stays attached.
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
    collected.push(comment.value,);
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
 Trailing same-line comment value after `from`, if any.
 
 @returns Object whose `value` field is the trailing comment value, or absent
          when there is no same-line comment.
 
 @example
 ```ts
 trailingCommentValue({ comments, source, from: kv.value.range[1], },); // { value: ' note' }
 ```
 */
export function trailingCommentValue(
  {
    comments,
    source,
    from,
  }: {
    readonly comments: readonly TomlComment[];
    readonly source: string;
    readonly from: number;
  },
): { readonly value?: string; } {
  /**
   End of the current line; the trailing comment must start before it.
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
  return match === undefined ? {} : { value: match.value, };
}
