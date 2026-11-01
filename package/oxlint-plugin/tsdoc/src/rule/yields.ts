import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  CreateOnceRule,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ReadonlyDeep, } from 'type-fest';

import {
  isGeneratorFunction,
  type TsdocParseResult,
} from '../tsdoc-utils.ts';

import {
  commentReportLoc,
  createFunctionTsdocVisitor,
} from './tsdoc-visitors.ts';

/**
 Checks whether a TSDoc comment documents yielded values.
 
 TSDoc standard doesn't define a yields tag natively, so this checks
 the raw comment text for the tag pattern.
 
 @param result - parsed TSDoc result
 
 @returns true when yielded values are documented
 */
function hasYieldsTag(result: ReadonlyDeep<TsdocParseResult>,): boolean {
  return result.comment
    .value
    .includes('@yields',);
}

/**
 Requires yield documentation (per {@link hasYieldsTag}) for generator
 functions (per {@link isGeneratorFunction}).
 
 @example
 ```ts
 // Bad; missing yield documentation for generator
 /\** Generates numbers. *\/
 function* count(): Generator<number> { yield 1; }
 
 // Good
 /\**
  Generates numbers.
 
  @yields sequential integers
 *\/
 function* count(): Generator<number> { yield 1; }
 ```
 */
export const requireYields: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Require @yields tag for generator functions.',
      recommended: true,
    },
    messages: {
      missing: 'Missing @yields tag for generator function.',
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
      includeArrowFunctions: false,
      handler: function requireYieldsHandler(
        node,
        result,
      ): void {
        if (!isGeneratorFunction(node,))
          return;
        if (!hasYieldsTag(result,)) {
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
 Validates yield tag consistency with generator functions, per
 {@link isGeneratorFunction}.
 
 Reports yield documentation (per {@link hasYieldsTag}) on non-generator
 functions.
 */
export const requireYieldsCheck: CreateOnceRule = {
  meta: {
    type: 'problem',
    docs: {
      description: 'Validate @yields tag consistency with generator functions.',
      recommended: true,
    },
    messages: {
      notGenerator: 'Function is not a generator but has @yields tag.',
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
      includeArrowFunctions: false,
      handler: function requireYieldsCheckHandler(
        node,
        result,
      ): void {
        if ((!isGeneratorFunction(node,)) && hasYieldsTag(result,)) {
          context.report({
            loc: commentReportLoc(result.comment,),
            messageId: 'notGenerator',
          },);
        }
      },
    },);
  },
};
