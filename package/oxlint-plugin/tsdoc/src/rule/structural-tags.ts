/**
 Structural TSDoc rule for tag spacing.
 
 Enforces blank lines before block tags in TSDoc comments.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  CreateOnceRule,
  Fixer,
  VisitorWithHooks,
} from '@oxlint/plugins';

import { wordRunEnd, } from '../comment-text.ts';
import {
  commentLineReportLoc,
  createTsdocVisitor,
  getCommentLines,
  stripCommentLineMarker,
} from './tsdoc-visitors.ts';

/**
 Absence marker for {@link extractLeadingTag} meaning "line does not begin
 with `@word`"; never a captured tag.
 
 @example
 ```ts
 const tag = extractLeadingTag('plain text',);
 if (tag === NO_LEADING_TAG)
   return;
 ```
 */
export const NO_LEADING_TAG: unique symbol = Symbol('tsdoc/no-leading-tag',);

/**
 Extracts the leading `@word` token from `s`, including the `@`.
 
 Linear scan: walks word characters starting at index 1 (just past `@`)
 and stops at the first non-word char. Mirrors `/^(@\w+)/`.
 
 @param s - line content (with the leading `*` already stripped and trimmed)
 
 @returns captured tag (e.g. `'@param'`) or {@link NO_LEADING_TAG} when `s` does not begin with `@word`
 
 @example
 ```ts
 extractLeadingTag('@param foo'); // '@param'
 extractLeadingTag('plain text'); // NO_LEADING_TAG
 ```
 */
export function extractLeadingTag(s: string,): string | typeof NO_LEADING_TAG {
  if (!s.startsWith('@',))
    return NO_LEADING_TAG;
  /**
   Exclusive end of the tag-name run; cursor starts at 1 to skip the leading at-sign.
   */
  const end = wordRunEnd({
    text: s,
    start: 1,
  },);
  if (end === 1)
    return NO_LEADING_TAG;
  return s.slice(
    0,
    end,
  );
}

/**
 Enforces consistent spacing between TSDoc tags.
 
 Requires a blank comment line before block tags identified via
 {@link extractLeadingTag} (configurable).
 */
export const tagLines: CreateOnceRule = {
  meta: {
    type: 'layout',
    fixable: 'whitespace',
    docs: {
      description: 'Enforce consistent spacing between TSDoc tags.',
      recommended: true,
    },
    messages: {
      noBlankBefore: 'Expected a blank line before "{{tag}}".',
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
      handler: function tagLinesHandler(
        _node,
        comment,
      ): void {
        /**
         Raw lines of the comment body, including opener `/**` and closer `*\/`.
         */
        const lines = getCommentLines(comment,);
        /**
         Minimum line count for a comment that can contain tag spacing issues.
         */
        const minContentLines = 3;
        if (lines.length
          < minContentLines)
          return;

        /**
         Indentation for blank comment lines: spaces + `*`.
         Matches the opener's `*` column (column of `/*` + 1).
         */
        const blankLineIndent = ' '.repeat(comment.loc
          .start
          .column
          + 1,);
        /**
         Pre-built ` * ` blank-line text inserted by the autofixer when spacing is missing.
         */
        const blankCommentLine = `${blankLineIndent}*`;

        // Check each content line (skip opener and closer)
        /**
         Comment body without opener and closer; scanned for tag-spacing violations.
         */
        const contentLines = lines.slice(
          1,
          -1,
        );
        contentLines.forEach(function checkTagLine(
          line,
          index,
        ): void {
          if (index === 0)
            return;
          /**
           Current line stripped of indent and the leading `*`, ready for tag matching.
           */
          const trimmed = stripCommentLineMarker(line.trimStart(),)
            .trimStart();
          if (!trimmed.startsWith('@',))
            return;
          // Check if previous line is blank
          /**
           Previous content line; required to determine whether the tag is preceded by a blank.
           */
          const prevLine = contentLines[index - 1];
          if (prevLine === undefined)
            return;
          /**
           Previous line stripped of indent and `*`; empty string means the tag is preceded by blank.
           */
          const prevTrimmed = stripCommentLineMarker(prevLine.trimStart(),)
            .trimStart();
          if (prevTrimmed.length
            > 0) {
            /**
             Tag matched on the line; absent only if the line somehow lost its leading `@`.
             */
            const matched = extractLeadingTag(trimmed,);
            /**
             Resolved tag string for the error message, with `\@unknown` for the impossible-absent case.
             */
            const tag = matched === NO_LEADING_TAG ? '@unknown' : matched;

            /**
             Line number of the tag line in the source file (1-based).
             `index` is 0-based within contentLines; +1 for the opener line, +1 for 1-based.
             */
            const tagLineNumber = comment.loc
              .start
              .line
              + index
              + 1;

            context.report({
              loc: commentLineReportLoc({
                comment,
                lineOffset: index + 1,
              },),
              messageId: 'noBlankBefore',
              data: { tag, },
              fix(fixer: ForeignBorrowed<Fixer>,) {
                /**
                 Insert a blank comment line (`\n *`) just before the tag line.
                 Use `getIndexFromLoc` to find the byte offset of the tag line start,
                 then insert the blank line text ending with a newline.
                 */
                const insertOffset = context.sourceCode
                  .getIndexFromLoc({
                    line: tagLineNumber,
                    column: 0,
                  },);
                return fixer.insertTextBeforeRange(
                  [
                    insertOffset,
                    insertOffset,
                  ],
                  `${blankCommentLine}\n`,
                );
              },
            },);
          }
        },);
      },
    },);
  },
};

export { emptyTags, } from './empty-tags.ts';

export { escapeInlineTags, } from './tag-escaping.ts';
