/**
 TSDoc access modifier validation rule.
 
 @module
 */

import { isWhitespace, } from '@monochromatic-dev/agent-harness-shared-text-scan/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  CreateOnceRule,
  VisitorWithHooks,
} from '@oxlint/plugins';

import {
  commentReportLoc,
  createTsdocVisitor,
} from './tsdoc-visitors.ts';

/**
 Returns true when `tag` appears in `text` preceded by start-of-string
 or whitespace and followed by whitespace, end-of-string, or `*`.
 
 Replaces the prior `new RegExp((?:^|\s)<tag>(?:\s|$|\*))` test with a
 single linear pass: each candidate position is found by `indexOf`, the
 surrounding characters are checked in constant time, and the cursor
 advances past every match attempt so the worst-case work is bounded by
 the length of `text`.
 
 @param text - haystack searched for the tag
 
 @param tag - literal tag including the leading `@` (e.g. `'@public'`)
 
 @returns whether the bounded tag occurs at least once in `text`
 
 @example
 ```ts
 containsBoundedAccessTag({ text: 'see @public here', tag: '@public', }); // true
 containsBoundedAccessTag({ text: 'mypublic', tag: '@public', }); // false
 ```
 */
export function containsBoundedAccessTag({
  text,
  tag,
}: {
  readonly text: string;
  readonly tag: string;
},): boolean {
  // Linear walk: each occurrence of `tag` is located by `indexOf`; the cursor
  // advances by one past every boundary-rejected candidate, so worst-case work
  // is bounded by the length of `text` and the stack stays flat.
  for (let from = 0; from <= text
    .length;) {
    /**
     Position of the next literal occurrence of `tag`; -1 ends the search.
     */
    const idx = text.indexOf(
      tag,
      from,
    );
    if (idx === (-1))
      return false;
    /**
     Char immediately before the match; start-of-string acts as whitespace.
     */
    const before = idx === 0 ? '' : text.charAt(idx - 1,);
    /**
     Whether the preceding char satisfies the `(?:^|\s)` anchor.
     */
    const beforeOk = (before === '') || isWhitespace(before,);
    if (!beforeOk) {
      from = idx + 1;
      continue;
    }
    /**
     Index immediately after the match; used to inspect the trailing char.
     */
    const afterIdx = idx + tag
      .length;
    /**
     Char immediately after the match; end-of-string acts as a valid terminator.
     */
    const after = afterIdx >= text
      .length ? '' : text.charAt(afterIdx,);
    /**
     Whether the trailing char satisfies the `(?:\s|$|\*)` anchor.
     */
    const afterOk = (after === '') || (after === '*')
      || isWhitespace(after,);
    if (afterOk)
      return true;
    from = idx + 1;
  }
  return false;
}

/**
 Validates access modifier tags in TSDoc comments.
 
 Reports conflicting access modifiers (e.g., public and internal together),
 detected via {@link containsBoundedAccessTag}.
 */
export const checkAccess: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Validate TSDoc access modifier tags.',
      recommended: true,
    },
    messages: {
      conflict: 'Conflicting access modifiers: {{tags}}. Use only one.',
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
     Access-level tags that are mutually exclusive.
     */
    const accessTags = [
      '@public',
      '@internal',
      '@alpha',
      '@beta',
      '@experimental',
    ];

    return createTsdocVisitor({
      context,
      handler: function checkAccessHandler(
        _node,
        comment,
      ): void {
        /**
         Raw comment body searched once per access tag with the boundary-anchored predicate below.
         */
        const text = comment.value;
        /**
         Subset of `accessTags` whose bounded form actually appears; multiple entries are a conflict.
         */
        const found = accessTags.filter(function isPresent(tag,): boolean {
          return containsBoundedAccessTag({
            text,
            tag,
          },);
        },);

        if (found.length
          > 1) {
          context.report({
            loc: commentReportLoc(comment,),
            messageId: 'conflict',
            data: { tags: found.join(', ',), },
          },);
        }
      },
    },);
  },
};

export { checkTagNames, } from './tag-names.ts';

export {
  noTypes,
  validTypes,
} from './tag-types.ts';
