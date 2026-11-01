/**
 Structural TSDoc rules for comment alignment and multiline format.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Comment,
  Context,
  CreateOnceRule,
  Fix,
  Fixer,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ReadonlyDeep, } from 'type-fest';

import { isTsdocBlock, } from '../tsdoc-comments.ts';
import {
  commentLineReportLoc,
  commentReportLoc,
  createTsdocVisitor,
  getCommentLines,
  shouldSkipIgnoredFile,
  stripCommentLineMarker,
} from './tsdoc-visitors.ts';

/**
 Inputs for extracting source indentation before a byte offset.
 */
type LinePrefixBeforeOffsetParams = {
  /**
   Complete file source text containing offset.
   */
  readonly sourceText: string;

  /**
   Byte offset whose line prefix should be copied.
   */
  readonly offset: number;
};

/**
 Inputs for building autofix text for single-line TSDoc blocks.
 */
type MultilineTsdocReplacementParams = {
  /**
   Rule context whose source text supplies indentation.
   */
  readonly context: Context;

  /**
   TSDoc block comment to rewrite.
   */
  readonly comment: ReadonlyDeep<Comment>;
};

/**
 Extracts whitespace before offset on its source line.
 
 @param params - source text and offset to inspect
 
 @returns indentation prefix from line start through offset
 
 @example
 ```ts
 linePrefixBeforeOffset({ sourceText: '  /** x *\/', offset: 2 });
 ```
 */
function linePrefixBeforeOffset(params: Readonly<LinePrefixBeforeOffsetParams>,): string {
  /**
   Complete file text searched for preceding line break.
   */
  const { sourceText, } = params;

  /**
   Target byte offset where comment begins.
   */
  const { offset, } = params;

  /**
   Previous line-break index, or negative one when offset is on first line.
   */
  const previousLineBreakIndex = sourceText.lastIndexOf(
    '\n',
    offset - 1,
  );

  /**
   First byte of line containing offset.
   */
  const lineStartIndex = previousLineBreakIndex + 1;

  return sourceText.slice(
    lineStartIndex,
    offset,
  );
}

/**
 Extracts content lines from TSDoc block body text.
 
 @param comment - TSDoc block comment whose body should be rendered
 
 @returns non-empty content lines without leading TSDoc asterisks
 
 @example
 ```ts
 const lines = tsdocContentLines(commentNode);
 ```
 */
function tsdocContentLines(comment: ReadonlyDeep<Comment>,): readonly string[] {
  return getCommentLines(comment,)
    .map(function stripTsdocMarker(line,): string {
      return stripCommentLineMarker(line.trimStart(),)
        .trim();
    },)
    .filter(function keepNonEmptyLine(line,): boolean {
      return line.length > 0;
    },);
}

/**
 Renders a single-line TSDoc block as canonical multiline text, combining
 indentation from {@link linePrefixBeforeOffset} with body lines from
 {@link tsdocContentLines}.
 
 @param params - rule context and comment to rewrite
 
 @returns replacement text preserving comment indentation
 
 @example
 ```ts
 const replacement = multilineTsdocReplacement({ context, comment });
 ```
 */
function multilineTsdocReplacement(
  params: ForeignBorrowed<Readonly<MultilineTsdocReplacementParams>>,
): string {
  /**
   Rule context whose source text provides indentation.
   */
  const { context, } = params;

  /**
   TSDoc comment selected for replacement.
   */
  const { comment, } = params;

  /**
   Complete source text for the file being linted.
   */
  const sourceText = context.sourceCode
    .getText();

  /**
   Indentation prefix before the opening comment delimiter.
   */
  const indent = linePrefixBeforeOffset({
    sourceText,
    offset: comment.range[0],
  },);

  /**
   Meaningful TSDoc content lines without leading asterisk markers.
   */
  const contentLines = tsdocContentLines(comment,);

  if (contentLines.length === 0)
    return `/**\n${indent} \n${indent} */`;

  /**
   Rendered body with each content line carrying star-less block indentation.
   */
  const body = contentLines
    .map(function renderContentLine(line,): string {
      return `${indent} ${line}`;
    },)
    .join('\n',);

  return `/**\n${body}\n${indent} */`;
}

/**
 Enforces consistent alignment of asterisks in TSDoc block comments.
 
 Each line of a TSDoc comment (except the opener) must have its leading `*`
 aligned with the first `*` of the opening `/**`.
 */
export const checkAlignment: CreateOnceRule = {
  meta: {
    type: 'layout',
    docs: {
      description: 'Enforce consistent alignment of leading asterisks in TSDoc comments.',
      recommended: true,
    },
    messages: {
      misaligned:
        'TSDoc asterisk misaligned: expected {{expected}} spaces of indent, found {{actual}}.',
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
      handler: function checkAlignmentHandler(
        _node,
        comment,
      ): void {
        /**
         Comment body split into lines; each continuation line is checked against `expectedIndent`.
         */
        const lines = getCommentLines(comment,);
        if (lines.length
          < 2)
          return;

        // The opening line `* ...` sets the expected indent
        // comment.loc.start.column is the column of `/*`, so `*` alignment is column + 1
        /**
         Expected column for the leading `*` on continuation lines, derived from the opener.
         */
        const expectedIndent = comment.loc
          .start
          .column
          + 1;

        lines
          .slice(1,)
          .forEach(function checkLine(
            line,
            index,
          ): void {
          /**
           Continuation line with leading whitespace removed; used to test for the `*` marker.
           */
            const trimmed = line.trimStart();
            if (trimmed.length
              === 0)
              return;
            if (!trimmed.startsWith('*',))
              return;
          /**
           Number of leading whitespace columns on the actual line; compared with `expectedIndent`.
           */
            const actualIndent = line.length
              - trimmed
                .length;
            if (actualIndent !== expectedIndent) {
              context.report({
                loc: commentLineReportLoc({
                  comment,
                  lineOffset: index + 1,
                  column: actualIndent,
                },),
                messageId: 'misaligned',
                data: {
                  expected: String(expectedIndent,),
                  actual: String(actualIndent,),
                },
              },);
            }
          },);
      },
    },);
  },
};

/**
 Enforces that TSDoc comments use multiline block style.
 
 Single-line `/** comment *\/`, detected via {@link isTsdocBlock}, is
 reported and auto-fixed to canonical star-less multiline format using
 {@link multilineTsdocReplacement}.
 */
export const multilineBlocks: CreateOnceRule = {
  meta: {
    type: 'layout',
    fixable: 'whitespace',
    docs: {
      description: 'Enforce multiline format for TSDoc block comments.',
      recommended: true,
    },
    messages: {
      singleLine: 'TSDoc comments must use multiline format.',
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
    return {
      before() {
        if (shouldSkipIgnoredFile({ context, }))
          return false;
        return undefined;
      },
      Program(): void {
        context.sourceCode
          .getAllComments()
          .filter(function keepTsdocBlock(comment: ForeignBorrowed<Comment>,): boolean {
            return isTsdocBlock(comment,);
          },)
          .forEach(function checkComment(comment: ForeignBorrowed<Comment>,): void {
            /**
             Comment body split into lines; one body line means opener and closer share a physical line.
             */
            const lines = getCommentLines(comment,);
            /**
             Line count for a single-line block comment body.
             */
            const singleLineCount = 1;
            if (lines.length !== singleLineCount)
              return;
            context.report({
              loc: commentReportLoc(comment,),
              messageId: 'singleLine',
              fix(fixer: ForeignBorrowed<Fixer>,): Fix | Fix[] {
                /**
                 Complete source text used to detect whether comment starts mid-line after code.
                 */
                const sourceText = context.sourceCode
                  .getText();
                /**
                 Text before comment on its line; non-whitespace would make a multiline fix unsafe.
                 */
                const prefix = linePrefixBeforeOffset({
                  sourceText,
                  offset: comment.range[0],
                },);
                /**
                 Prefix without whitespace; non-empty means comment follows code on the same line.
                 */
                const trimmedPrefix = prefix.trim();
                if (trimmedPrefix.length > 0)
                  return [];
                return fixer.replaceTextRange(
                  [
                    comment.range[0],
                    comment.range[1],
                  ],
                  multilineTsdocReplacement({
                    context,
                    comment,
                  },),
                );
              },
            },);
          },);
      },
    };
  },
};

export { noMultiAsterisks, } from './asterisk-validation.ts';

export {
  emptyTags,
  escapeInlineTags,
  tagLines,
} from './structural-tags.ts';
