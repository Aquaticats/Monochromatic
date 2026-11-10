/**
 TSDoc type annotation disallow rule.
 
 Extracted from `tag-types.ts` to keep files under 100 countable lines.
 
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
  stripCommentLineMarker,
} from './tsdoc-visitors.ts';

//region Inline tag recognition
// The character after `{` that makes a brace a TSDoc inline tag, and the check
// that separates one from a JSDoc type annotation.

/**
 Character that follows `{` for TSDoc inline tags such as `{@link Target}`.
 */
const INLINE_TAG_MARKER = '@';

/**
 Checks whether a brace opens a TSDoc inline tag rather than a JSDoc type.
 
 @param params - source text and opening-brace index
 
 @returns true when the brace is immediately followed by an inline tag marker
 
 @example
 ```ts
 isInlineTagOpener({ text: '{@link Target}', braceIndex: 0 }); // true
 ```
 */
function isInlineTagOpener(params: {
  /**
   Source text containing the candidate brace.
   */
  readonly text: string;
  /**
   Index of the candidate opening brace.
   */
  readonly braceIndex: number;
},): boolean {
  /**
   Source text and brace index used to test the character after `{`.
   */
  const {
    text,
    braceIndex,
  } = params;
  return text.charAt(braceIndex + 1,) === INLINE_TAG_MARKER;
}

//endregion Inline tag recognition

//region Annotation collection
// The linear scan that collects every `{Type}` body following a `@word`, which
// is what the rule reports on.

/**
 Collects every JSDoc-style `{Type}` body that follows a `@word` and at
 least one whitespace char in `line`. Matches the legacy `/@\w+\s+\{([^}]+)\}/g`
 shape except for TSDoc inline tags such as `{@link Target}`, detected via
 {@link isInlineTagOpener}.
 
 Strictly linear: each `@` candidate is found by `indexOf`, surrounding
 checks are constant-time, and the cursor advances past every consumed
 span so the total work is bounded by the length of `line`.
 
 @param line - line content (already stripped of comment markers and trim)
 
 @returns ordered list of captured type bodies (without the braces)
 
 @example
 ```ts
 // built by concatenation so this example does not itself trip the no-types rule
 findTypeAnnotations('@param ' + '{string}'); // ['string']
 ```
 */
export function findTypeAnnotations(line: string,): readonly string[] {
  /**
   Captured type bodies in source order; each omits the surrounding braces.
   */
  const out: string[] = [];
  // Linear walk: each `@` is located by `indexOf`; the tag run, whitespace gap,
  // and `{...}` body are each measured once and the cursor jumps past every
  // consumed span, so total work is bounded by the length of `line` and the stack
  // stays flat. An unterminated `{` halts the scan, matching the prior recursion.
  for (let from = 0; from < line
    .length;) {
    /**
     Position of the next at-sign; -1 ends the scan.
     */
    const atIdx = line.indexOf(
      '@',
      from,
    );
    if (atIdx === (-1))
      break;
    /**
     Exclusive end of the tag-name run; cursor starts at `atIdx + 1` to skip the at-sign.
     */
    const tagEnd = wordRunEnd({
      text: line,
      start: atIdx + 1,
    },);
    if (tagEnd === (atIdx + 1)) {
      from = atIdx + 1;
      continue;
    }
    /**
     First index past the inter-token whitespace; `{` must live here for a match.
     */
    const afterWs = whitespaceRunEnd({
      text: line,
      start: tagEnd,
    },);
    if ((afterWs === tagEnd) || (line.charAt(afterWs,)
      !== '{')) {
      from = tagEnd;
      continue;
    }
    /**
     Position of the matching `}`; -1 means the type body is unterminated.
     */
    const closeIdx = line.indexOf(
      '}',
      afterWs + 1,
    );
    if (closeIdx === (-1))
      break;
    if (isInlineTagOpener({
      text: line,
      braceIndex: afterWs,
    },)) {
      from = closeIdx + 1;
      continue;
    }
    /**
     Captured body between `{` and `}` exclusive; must be non-empty per `[^}]+`.
     */
    const body = line.slice(
      afterWs + 1,
      closeIdx,
    );
    if (body.length
      === 0) {
      from = closeIdx + 1;
      continue;
    }
    out.push(body,);
    from = closeIdx + 1;
  }
  return out;
}

//endregion Annotation collection

//region The rule
// The exported rule, reporting one diagnostic per collected annotation.

/**
 Disallows type annotations in TSDoc tags, detected via
 {@link findTypeAnnotations}.
 
 In TypeScript projects, types are expressed via type annotations, not JSDoc-style
 `{Type}` syntax. Reports param/returns with `{Type}` syntax.
 */
export const noTypes: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Disallow JSDoc-style type annotations in TSDoc comments.',
      recommended: true,
    },
    messages: {
      noType:
        'Type annotations in TSDoc are not allowed in TypeScript. Remove the "{{{type}}}" type.',
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
      handler: function noTypesHandler(
        _node,
        comment,
      ): void {
        /**
         Comment body split into lines; each is scanned for the disallowed JSDoc-style
         curly-braced type annotation that follows a tag.
         */
        const lines = comment.value
          .split('\n',);
        lines.forEach(function checkLine(
          line,
          index,
        ): void {
          /**
           Line stripped of indent and leading `*` so the scan matches at the start of the content.
           */
          const trimmed = stripCommentLineMarker(line.trimStart(),)
            .trimStart();
          for (const body of findTypeAnnotations(trimmed,)) {
            context.report({
              loc: commentLineReportLoc({
                comment,
                lineOffset: index,
              },),
              messageId: 'noType',
              data: { type: body, },
            },);
          }
        },);
      },
    },);
  },
};

//endregion The rule
