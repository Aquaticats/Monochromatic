/**
 TSDoc asterisk validation rule.
 
 Extracted from `structural.ts` to keep files under 100 countable lines.
 
 @module
 */

import { isWhitespaceChar, } from '@monochromatic-dev/oxlint-plugin-shared/ts';
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
} from './tsdoc-visitors.ts';

/**
 Disallows doubled canonical asterisk prefixes in TSDoc comment lines.
 
 Lines like ` ** text` are invalid while literal Markdown such as
 ` **Note**` remains content rather than a doubled prefix.
 */
export const noMultiAsterisks: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Disallow doubled canonical asterisk prefixes in TSDoc comment lines.',
      recommended: true,
    },
    messages: {
      extra: 'Extra asterisk at start of TSDoc comment line.',
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
      handler: function noMultiHandler(
        _node,
        comment,
      ): void {
        /**
         Comment body split into lines; opener and closer are sliced off before scanning.
         */
        const lines = getCommentLines(comment,);
        // Skip first line (opening) and last line (closing)
        lines
          .slice(
          1,
          -1,
          )
          .forEach(function checkLine(
            line,
            index,
          ): void {
            /**
             Leading-whitespace-stripped line; needed to detect a `**` that should be a single `*`.
             */
            const trimmed = line.trimStart();
            /**
             Character after doubled stars; whitespace distinguishes a malformed
             prefix from literal-leading Markdown such as `**Note**`.
             */
            const afterDoubledAsterisk = trimmed.charAt(2,);
            if (trimmed.startsWith('**',)
              && ((afterDoubledAsterisk.length === 0)
                || isWhitespaceChar(afterDoubledAsterisk,))) {
              context.report({
                loc: commentLineReportLoc({
                  comment,
                  lineOffset: index + 1,
                },),
                messageId: 'extra',
              },);
            }
          },);
      },
    },);
  },
};
