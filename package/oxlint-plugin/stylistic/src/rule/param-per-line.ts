import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  CreateOnceRule,
  Span,
  VisitorWithHooks,
} from '@oxlint/plugins';

import { checkItemsPerLine, } from '../utility/item-per-line.ts';
import {
  at,
  rangeOf,
} from '../utility/range.ts';

/**
 Function-like node shape carrying parameters for this rule.
 */
type FunctionParamListNode = Span & {
  /**
   Function parameters in source order.
   */
  readonly params?: readonly Span[];
};

/**
 Enforces one parameter per line in function declarations, function
 expressions, arrow functions, and the full TypeScript function-like
 node set: `TSFunctionType`, `TSDeclareFunction`, `TSMethodSignature`,
 `TSCallSignatureDeclaration`, `TSConstructSignatureDeclaration`,
 `TSConstructorType`, and `TSEmptyBodyFunctionExpression`.
 
 In ESTree (used by oxlint JS plugins), `node.params` is a plain array
 of parameter nodes without a wrapper container. This rule locates the
 surrounding parentheses, then delegates to {@link checkItemsPerLine} with those
 explicit delimiter offsets.
 
 The autofix replaces only the source range `[openParen, closeParen + 1]`,
 so syntactic prefixes (`new` for constructor types and construct signatures)
 and tails (`=> void`, `: void;`) of every covered shape are preserved.
 
 @example
 ```ts
 // Bad
 function create(name: string, age: number): void {}
 type F = (a: string, b: number) => void;
 declare function ambient(a: string, b: number): void;
 
 // Good
 function create(
   name: string,
   age: number,
 ): void {}
 type F = (
   a: string,
   b: number,
 ) => void;
 declare function ambient(
   a: string,
   b: number,
 ): void;
 ```
 */
export const paramPerLine: CreateOnceRule = {
  meta: {
    type: 'layout',
    fixable: 'whitespace',
    docs: {
      description:
        'Require each function parameter to be on its own line when there are 2 or more parameters.',
      recommended: true,
    },
    messages: {
      paramPerLine: 'Each function parameter must be on its own line.',
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
     Checks function parameters and reports if they share lines.
     
     @param node - function declaration or expression AST node
     */
    function checkFunction(node: ForeignBorrowed<Span>,): void {
      /**
       Narrowed function-like visitor node used for parameter access.
       */
      const fnNode = node as FunctionParamListNode;
      /**
       Extract params from the function-like node.
       */
      const { params, } = fnNode;
      if ((params === undefined)
        || (params.length
          < 2))
        {
        return;
      }

      /**
       Source text is needed for boundary-paren lookup and the fixer call below.
       */
      const sourceText = context.sourceCode
        .getText();
      /**
       Range of the first param; used to find the `(` to its left.
       */
      const firstRange = rangeOf(at({
        arr: params,
        index: 0,
      },),);
      /**
       Range of the last param; used to find the `)` to its right.
       */
      const lastRange = rangeOf(at({
        arr: params,
        index: params.length
          - 1,
      },),);

      /**
       Find the `(` before the first param.
       */
      const openParen = sourceText.lastIndexOf(
        '(',
        firstRange[0],
      );
      /**
       Find the `)` after the last param.
       */
      const closeParen = sourceText.indexOf(
        ')',
        lastRange[1],
      );

      if ((openParen === (-1)) || (closeParen === (-1)))
        return;

      checkItemsPerLine({
        context,
        container: node,
        items: params,
        messageId: 'paramPerLine',
        bracketPair: {
          open: '(',
          close: ')',
        },
        boundary: {
          openOffset: openParen,
          closeOffset: closeParen,
        },
      },);
    }

    return {
      FunctionDeclaration: checkFunction,
      FunctionExpression: checkFunction,
      ArrowFunctionExpression: checkFunction,
      TSFunctionType: checkFunction,
      TSDeclareFunction: checkFunction,
      TSMethodSignature: checkFunction,
      TSCallSignatureDeclaration: checkFunction,
      TSConstructSignatureDeclaration: checkFunction,
      TSConstructorType: checkFunction,
      TSEmptyBodyFunctionExpression: checkFunction,
    };
  },
};
