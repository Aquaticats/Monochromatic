/**
 TSDoc parameter name and description validation rules.
 
 Extracted from `params.ts` to keep files under 100 countable lines.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  CreateOnceRule,
  VisitorWithHooks,
} from '@oxlint/plugins';

import {
  commentReportLoc,
  createFunctionTsdocVisitor,
} from './tsdoc-visitors.ts';

/**
 Requires that every `\@param` tag has a parameter name.
 
 Reports `\@param - description` (missing name before the hyphen).
 */
export const requireParamName: CreateOnceRule = {
  meta: {
    type: 'problem',
    docs: {
      description: 'Require @param tags to specify a parameter name.',
      recommended: true,
    },
    messages: {
      missingName: '@param tag is missing a parameter name.',
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
    return createFunctionTsdocVisitor({
      context,
      handler: function requireParamNameHandler(
        _node,
        result,
      ): void {
        result.docComment
          .params
          .blocks
          .forEach(function checkBlock(block,): void {
            if (block.parameterName
              .trim()
              .length
              === 0) {
              context.report({
                loc: commentReportLoc(result.comment,),
                messageId: 'missingName',
              },);
            }
          },);
      },
    },);
  },
};

/**
 Requires that every `\@param` tag has a description after the parameter name.
 
 Relies on the scanner's precomputed `hasDescription`, which is true when the
 block has any non-whitespace text after the parameter name and optional
 hyphen separator.
 
 @example
 ```ts
 // Bad; no description
 /\** \@param name *\/
 function foo(name: string): void {}
 
 // Good
 /\** \@param name - user name to display *\/
 function foo(name: string): void {}
 ```
 */
export const requireParamDescription: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Require descriptions for @param tags.',
      recommended: true,
    },
    messages: {
      missingDescription: '@param "{{paramName}}" is missing a description.',
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
    return createFunctionTsdocVisitor({
      context,
      handler: function requireParamDescHandler(
        _node,
        result,
      ): void {
        result.docComment
          .params
          .blocks
          .forEach(function checkBlock(block,): void {
            if (!block.hasDescription) {
              context.report({
                loc: commentReportLoc(result.comment,),
                messageId: 'missingDescription',
                data: { paramName: block.parameterName, },
              },);
            }
          },);
      },
    },);
  },
};
