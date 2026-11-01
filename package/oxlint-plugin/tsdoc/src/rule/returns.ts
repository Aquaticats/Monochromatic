/**
 TSDoc returns tag validation rules.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  CreateOnceRule,
  VisitorWithHooks,
} from '@oxlint/plugins';

import { functionReturnsValue, } from '../tsdoc-utils.ts';

import {
  commentReportLoc,
  createFunctionTsdocVisitor,
} from './tsdoc-visitors.ts';

/**
 Requires returns tag for functions that return a value.
 
 Skips void/never return types, constructors, and setters, per
 {@link functionReturnsValue}.
 
 @example
 ```ts
 // Bad; missing returns tag
 /\** Adds numbers. *\/
 function add(a: number, b: number): number { return a + b; }
 
 // Good
 /\**
  Adds numbers.
 
  @returns sum of a and b
 *\/
 function add(a: number, b: number): number { return a + b; }
 ```
 */
export const requireReturns: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Require @returns tag for functions that return a value.',
      recommended: true,
    },
    messages: {
      missing: 'Missing @returns tag for function that returns a value.',
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
      handler: function requireReturnsHandler(
        node,
        result,
      ): void {
        if (!functionReturnsValue(node,))
          return;
        if (result.docComment
          .returnsBlock
          === undefined) {
          context.report({
            loc: commentReportLoc(result.comment,),
            messageId: 'missing',
          },);
        }
      },
    },);
  },
};

/**
 Validates returns tag consistency with the function signature.
 
 Reports returns tag on void functions, and missing returns tag on
 functions with non-void return types (when returns tag is present
 but the function doesn't return a value, per {@link functionReturnsValue}).
 */
export const requireReturnsCheck: CreateOnceRule = {
  meta: {
    type: 'problem',
    docs: {
      description: 'Validate @returns tag consistency with function signature.',
      recommended: true,
    },
    messages: {
      voidReturn: 'Function has void/never return type but has @returns tag.',
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
      handler: function requireReturnsCheckHandler(
        node,
        result,
      ): void {
        if ((!functionReturnsValue(node,))
          && (result.docComment
            .returnsBlock
            !== undefined))
          {
          context.report({
            loc: commentReportLoc(result.comment,),
            messageId: 'voidReturn',
          },);
        }
      },
    },);
  },
};

export { requireReturnsDescription, } from './returns-description.ts';
