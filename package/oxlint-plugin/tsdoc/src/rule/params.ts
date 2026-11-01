/**
 TSDoc parameter name matching and presence rules.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  CreateOnceRule,
  VisitorWithHooks,
} from '@oxlint/plugins';

import {
  extractDestructuredParamNames,
  extractDocParamNames,
  extractParamNames,
} from '../tsdoc-utils.ts';

import {
  commentReportLoc,
  createFunctionTsdocVisitor,
} from './tsdoc-visitors.ts';

/**
 Validates that `\@param` tag names match the function's actual parameter names.
 
 Reports mismatches, incorrect order, and `\@param` tags for nonexistent parameters.
 Compares parameter names from {@link extractParamNames} against documented
 tag names from {@link extractDocParamNames}. Allows `\@param` tags that match
 property names from destructured parameters (ObjectPattern/ArrayPattern, via
 {@link extractDestructuredParamNames}), since documenting destructured
 properties by name is a common TSDoc convention.
 
 @example
 ```ts
 // Bad; parameter name doesn't match
 /\** \@param x - description *\/
 function foo(name: string): void {}
 
 // Good
 /\** \@param name - description *\/
 function foo(name: string): void {}
 
 // Good; destructured property names are allowed
 /\** \@param value - item to process *\/
 function foo({ value }: Options): void {}
 ```
 */
export const checkParamNames: CreateOnceRule = {
  meta: {
    type: 'problem',
    docs: {
      description: 'Validate that @param names match function parameter names.',
      recommended: true,
    },
    messages: {
      mismatch:
        '@param "{{docName}}" does not match parameter "{{paramName}}". For destructured parameters, document each destructured parameter.',
      extra:
        '@param "{{docName}}" does not match any function parameter. For destructured parameters, document each destructured parameter.',
      order:
        '@param tags are not in the same order as the function parameters. For destructured parameters, document each destructured parameter.',
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
      handler: function checkParamNamesHandler(
        node,
        result,
      ): void {
        /**
         Ordered list of declared parameter names; positions in this array drive the index match.
         */
        const paramNames = extractParamNames(node,);
        /**
         Ordered list of `\@param` names extracted from the TSDoc comment.
         */
        const docParamNames = extractDocParamNames(result.docComment,);
        /**
         Names from destructured patterns; `\@param` tags matching these are accepted as-is.
         */
        const destructuredNames = extractDestructuredParamNames(node,);

        // Check each documented param exists in the function signature
        docParamNames.forEach(function checkDocParam(
          docName,
          index,
        ): void {
          // Allow @param tags that match destructured property names
          if (destructuredNames.has(docName,))
            return;

          /**
           Parameter at the same index as the current doc tag; basis for the name comparison.
           */
          const correspondingParam = paramNames[index];
          if (correspondingParam === undefined) {
            // Extra @param with no matching parameter
            if (!paramNames.includes(docName,)) {
              context.report({
                loc: commentReportLoc(result.comment,),
                messageId: 'extra',
                data: { docName, },
              },);
            }
            else {
              context.report({
                loc: commentReportLoc(result.comment,),
                messageId: 'order',
              },);
            }
          }
          else if (docName !== correspondingParam) {
            context.report({
              loc: commentReportLoc(result.comment,),
              messageId: 'mismatch',
              data: {
                docName,
                paramName: correspondingParam,
              },
            },);
          }
        },);
      },
    },);
  },
};

/**
 Requires `\@param` tags for all function parameters, comparing
 {@link extractParamNames} against {@link extractDocParamNames}.
 
 @example
 ```ts
 // Bad; missing \@param for `count`
 /\** \@param name - user name *\/
 function greet(name: string, count: number): void {}
 
 // Good
 /\**
  \@param name - user name
  \@param count - greeting count
 *\/
 function greet(name: string, count: number): void {}
 ```
 */
export const requireParam: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Require @param tags for all function parameters.',
      recommended: true,
    },
    messages: {
      missing: 'Missing @param tag for "{{paramName}}".',
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
      handler: function requireParamHandler(
        node,
        result,
      ): void {
        /**
         Declared parameter names; each must appear among the documented tags.
         */
        const paramNames = extractParamNames(node,);
        /**
         Documented param names indexed in a Set for O(1) presence checks.
         */
        const docParamNames = new Set(extractDocParamNames(result.docComment,),);

        paramNames.forEach(function checkParam(paramName,): void {
          if (!docParamNames.has(paramName,)) {
            context.report({
              loc: commentReportLoc(result.comment,),
              messageId: 'missing',
              data: { paramName, },
            },);
          }
        },);
      },
    },);
  },
};

export {
  requireParamDescription,
  requireParamName,
} from './param-validation.ts';
