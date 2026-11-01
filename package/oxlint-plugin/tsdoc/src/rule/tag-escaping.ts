/**
 TSDoc inline tag escaping rule.
 
 Extracted from `structural-tags.ts` to keep files under 100 countable lines.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  CreateOnceRule,
  VisitorWithHooks,
} from '@oxlint/plugins';

import {
  commentLineReportLoc,
  createTsdocVisitor,
  getCommentLines,
  stripCommentLineMarker,
} from './tsdoc-visitors.ts';

/**
 Returns true when `trimmed` contains an unescaped `*\/` sequence
 (i.e. one whose preceding character is not a backslash).
 
 Linear scan: each candidate `*\/` is found by `indexOf`, the previous
 character is checked, and the cursor advances past the match before the
 next `indexOf` call, so worst-case work is bounded by the length of `s`.
 
 @param s - line content (with the leading `*` already stripped)
 
 @returns whether an unescaped comment closer appears in `s`
 
 @example
 ```ts
 hasUnescapedCommentClose('plain content'); // false
 ```
 */
export function hasUnescapedCommentClose(s: string,): boolean {
  // Linear walk: each `*\/` is located by `indexOf`, the cursor advances past
  // every escaped match, so total work is bounded by the length of `s` and the
  // stack stays flat regardless of how many escaped closers appear.
  for (let from = 0; from <= s
    .length;) {
    /**
     Position of the next `*\/`; -1 means the rest of the string is safe.
     */
    const idx = s.indexOf(
      '*/',
      from,
    );
    if (idx === (-1))
      return false;
    /**
     Char immediately before the match; backslash means the closer is escaped.
     */
    const prev = idx === 0 ? '' : s.charAt(idx - 1,);
    if (prev !== '\\')
      return true;
    from = idx + 2;
  }
  return false;
}

/**
 Enforces that `*\/` inside TSDoc content is escaped as `*\\/`, detected via
 {@link hasUnescapedCommentClose}.
 
 An unescaped `*\/` would prematurely close the comment block.
 */
export const escapeInlineTags: CreateOnceRule = {
  meta: {
    type: 'problem',
    docs: {
      description: 'Enforce escaping of `*/` inside TSDoc comments.',
      recommended: true,
    },
    messages: {
      unescaped: String.raw`Unescaped '*/' inside TSDoc content. Use '*\/' instead.`,
    },
  },
  /**
   Handles effectful plugin callback.
   
   @param context - Foreign callback value carrying diagnostic capability.
   
   @mutates context - Emits Oxlint diagnostics through foreign rule context.
   
   @example
   ```ts
   createOnce(context);
   ```
   */
  createOnce(context: ForeignBorrowed<Context>,): VisitorWithHooks {
    return createTsdocVisitor({
      context,
      handler: function escapeHandler(
        _node,
        comment,
      ): void {
        /**
         Comment body split into lines; the last (legitimate closer) is dropped before scanning.
         */
        const lines = getCommentLines(comment,);
        // Skip the last line which is the legitimate closing `*/`
        lines
          .slice(
          0,
          -1,
          )
          .forEach(function checkLine(
            line,
            index,
          ): void {
            // Skip the first line opener
            if ((index === 0)
              && line.trimEnd()
                .endsWith('*',))
              return;
            /**
             Line stripped of indent and `*` so an embedded `*\/` becomes detectable in content.
             */
            const trimmed = stripCommentLineMarker(line.trimStart(),);
            // Look for `*/` not preceded by backslash inside content
            if (hasUnescapedCommentClose(trimmed,)) {
              context.report({
                loc: commentLineReportLoc({
                  comment,
                  lineOffset: index,
                },),
                messageId: 'unescaped',
              },);
            }
          },);
      },
    },);
  },
};
