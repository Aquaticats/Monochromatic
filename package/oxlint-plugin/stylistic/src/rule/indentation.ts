import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  CreateOnceRule,
  Fixer,
  VisitorWithHooks,
} from '@oxlint/plugins';

import {
  computeIndentExpectations,
} from '../utility/indent-expectation.ts';

/**
 Requires two-space indentation from each line's structural anchor.
 
 A checked line's first token anchors at the nearest earlier-line AST
 ancestor's line plus two columns. Closers (`}`, `]`, `)`) return to their
 opener's base line. Suffix keywords (`else`, `catch`, `finally`, do-while
 `while`) align with their statement line. Template-literal content,
 comment-led lines, and blank lines stay exempt.
 
 The autofix rewrites only the leading whitespace of deviating lines. Expected
 widths build on earlier lines' model widths, so one pass normalizes whole
 files to this fixed point, converging with the per-line, chain, and
 invocation-depth fixers that emit the same base-plus-two indentation.
 
 This replaces the TypeScript indentation dprint used to own; dprint stays
 disabled for TypeScript because its rewraps touch far more than indentation.
 
 @example
 ```ts
 // Bad
 function f() {
 return 1;
 }

 // Good
 function f() {
   return 1;
 }
 ```
 */
export const indentation: CreateOnceRule = {
  meta: {
    type: 'layout',
    fixable: 'code',
    schema: [],
    docs: {
      description: 'Require two-space indentation from each line\'s structural anchor.',
      recommended: true,
    },
    messages: {
      unexpectedIndentation: 'Expected indentation of {{expected}} spaces but found {{actual}}.',
    },
  },
  createOnce(context: ForeignBorrowed<Context>,): VisitorWithHooks {
    /**
     Program visitor runs the whole-file indentation model once per file.
     
     Every deviation reports with its own whitespace-only fix; the fixes
     never overlap, so one pass applies all of them together.
     */
    function checkProgram(): void {
      /**
       Source text, AST, and visitor keys the indentation model walks.
       */
      const {
        ast,
        text,
        visitorKeys,
      } = context.sourceCode;
      const expectations = computeIndentExpectations({
        sourceText: text,
        root: ast,
        tokens: ast.tokens,
        visitorKeys,
      },);
      for (const expectation of expectations) {
        if (expectation.expectedWidth === expectation.actualWidth)
          continue;
        context.report({
          loc: context.sourceCode
            .getLocFromIndex(expectation.contentStart,),
          messageId: 'unexpectedIndentation',
          data: {
            expected: expectation.expectedWidth,
            actual: expectation.actualWidth,
          },
          fix(fixer: ForeignBorrowed<Fixer>,): ReturnType<Fixer['replaceTextRange']> {
            return fixer.replaceTextRange(
              [
                expectation.lineStart,
                expectation.contentStart,
              ],
              ' '.repeat(expectation.expectedWidth,),
            );
          },
        },);
      }
    }

    return {
      Program: checkProgram,
    };
  },
};
