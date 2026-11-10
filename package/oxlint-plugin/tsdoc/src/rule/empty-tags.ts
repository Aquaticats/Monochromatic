/**
 TSDoc empty modifier tag validation rule.
 
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
  whitespaceRunEnd,
  wordRunEnd,
} from '../comment-text.ts';
import {
  commentLineReportLoc,
  createTsdocVisitor,
  getCommentLines,
  stripCommentLineMarker,
} from './tsdoc-visitors.ts';

//region Tagged line shape
// The absence marker for a line that is not `@tag <text>` form, and the parsed
// shape a tagged line reduces to.

/**
 Absence marker for {@link parseTaggedLine} meaning "line is not `@tag <text>`
 form"; never a parsed tagged line.
 
 @example
 ```ts
 const parsed = parseTaggedLine('plain text',);
 if (parsed === UNTAGGED_LINE)
   return;
 ```
 */
export const UNTAGGED_LINE: unique symbol = Symbol('tsdoc parser found line without tag',);

/**
 Parsed shape of a tagged TSDoc line; {@link UNTAGGED_LINE} when the line is not
 `@tag <text>` form.
 */
type TaggedLine = {
  /**
   Captured tag including the leading at-sign.
   */
  tag: string;
  /**
   Content following the whitespace gap.
   */
  rest: string;
};

//endregion Tagged line shape

//region Parsing one line
// The linear scan over the leading `@word` run, the whitespace gap, and the
// rest of the line.

/**
 Parses a TSDoc line of the form `@tag <whitespace> <rest>`.
 
 Linear scan over the leading `@word` run, the whitespace gap, and the
 remainder; no backtracking. Mirrors `/^(@\w+)\s+(.+)/`.
 
 @param s - line content (with the leading `*` already stripped)
 
 @returns parsed tag + rest, or {@link UNTAGGED_LINE} when the shape does not match
 
 @example
 ```ts
 parseTaggedLine('@param foo'); // { tag: '@param', rest: 'foo' }
 parseTaggedLine('@param'); // UNTAGGED_LINE
 ```
 */
export function parseTaggedLine(s: string,): TaggedLine | typeof UNTAGGED_LINE {
  if (!s.startsWith('@',))
    return UNTAGGED_LINE;
  /**
   Exclusive end of the tag-name run; cursor starts at 1 to skip the leading at-sign.
   */
  const tagEnd = wordRunEnd({
    text: s,
    start: 1,
  },);
  if (tagEnd === 1)
    return UNTAGGED_LINE;
  /**
   First index past the inter-token whitespace; rest starts here.
   */
  const restStart = whitespaceRunEnd({
    text: s,
    start: tagEnd,
  },);
  if (restStart === tagEnd)
    return UNTAGGED_LINE;
  /**
   Remaining content; must be non-empty to match `(.+)`.
   */
  const rest = s.slice(restStart,);
  if (rest.length
    === 0)
    return UNTAGGED_LINE;
  return {
    tag: s.slice(
      0,
      tagEnd,
    ),
    rest,
  };
}

//endregion Parsing one line

//region The rule
// The exported rule, which reports the modifier tags that should carry no
// content but do.

/**
 Enforces that TSDoc tags which should not have content are empty.
 
 Modifier tags like `\@public`, `\@readonly`, `\@override`, `\@sealed`,
 `\@virtual`, `\@alpha`, `\@beta`, `\@internal`, `\@experimental`,
 `\@eventProperty`, and `\@packageDocumentation` must not have content,
 parsed via {@link parseTaggedLine}.
 */
export const emptyTags: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Enforce that TSDoc modifier tags have no content.',
      recommended: true,
    },
    messages: {
      nonEmpty: 'TSDoc modifier tag "{{tag}}" must not have content.',
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
    /**
     Tags that must be standalone (no content after them).
     */
    const modifierTags = new Set([
      '@public',
      '@readonly',
      '@override',
      '@sealed',
      '@virtual',
      '@alpha',
      '@beta',
      '@internal',
      '@experimental',
      '@eventProperty',
      '@packageDocumentation',
    ],);

    return createTsdocVisitor({
      context,
      handler: function emptyTagsHandler(
        _node,
        comment,
      ): void {
        /**
         Comment body split into lines; each is inspected independently for modifier-tag misuse.
         */
        const lines = getCommentLines(comment,);
        lines.forEach(function checkLine(
          line,
          index,
        ): void {
          /**
           Line stripped of indent and `*` so the leading `\@tag` (if any) is at column 0.
           */
          const trimmed = stripCommentLineMarker(line.trimStart(),)
            .trimStart();
          /**
           Parsed `\@tag <text>` shape; absent when the line carries no tag with trailing text.
           */
          const tagMatch = parseTaggedLine(trimmed,);
          if (tagMatch === UNTAGGED_LINE)
            return;
          /**
           Captured tag name and remainder; both populate the diagnostic and gate the report.
           */
          const {
            tag,
            rest,
          } = tagMatch;
          if (modifierTags.has(tag,)
            && (rest.trim()
              .length
              > 0)) {
            context.report({
              loc: commentLineReportLoc({
                comment,
                lineOffset: index,
              },),
              messageId: 'nonEmpty',
              data: { tag, },
            },);
          }
        },);
      },
    },);
  },
};

//endregion The rule
