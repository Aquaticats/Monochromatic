/**
 Arithmetic and Bash test visitors for `unbash` shell command analysis.
 
 @module
 */

import type {
  ArithmeticExpression as UnbashArithmeticExpression,
  TestExpression as UnbashTestExpression,
} from 'unbash';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  EMPTY_VISIT_RESULT,
  type VisitResult,
} from './internal-types.ts';
import { visitExpansion, } from './nested.ts';
import type { ShellCommandContext, } from './types.ts';
import { wordWorkItems, } from './work-items.ts';

/**
 Build child work from arithmetic expression.
 
 @param expression - arithmetic expression to inspect
 
 @param context - execution context inherited by nested expansions
 
 @returns nested command expansion work
 
 @example
 ```ts
 visitArithmetic({ expression, context });
 ```
 */
function visitArithmetic(
  {
    expression,
    context,
  }: {
    readonly expression: ForeignBorrowed<UnbashArithmeticExpression>;
    readonly context: ShellCommandContext;
  },
): VisitResult {
  if (expression.type === 'ArithmeticCommandExpansion') {
    return visitExpansion({
      expansion: expression,
      context,
    },);
  }
  if (expression.type === 'ArithmeticBinary') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        {
          kind: 'arithmetic',
          expression: expression.left,
          context,
        },
        {
          kind: 'arithmetic',
          expression: expression.right,
          context,
        },
      ],
    };
  }
  if (expression.type === 'ArithmeticUnary') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [{
        kind: 'arithmetic',
        expression: expression.operand,
        context,
      },],
    };
  }
  if (expression.type === 'ArithmeticTernary') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        {
          kind: 'arithmetic',
          expression: expression.test,
          context,
        },
        {
          kind: 'arithmetic',
          expression: expression.consequent,
          context,
        },
        {
          kind: 'arithmetic',
          expression: expression.alternate,
          context,
        },
      ],
    };
  }
  if (expression.type === 'ArithmeticGroup') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [{
        kind: 'arithmetic',
        expression: expression.expression,
        context,
      },],
    };
  }
  return EMPTY_VISIT_RESULT;
}

/**
 Build child work from Bash test expression.
 
 @param expression - Bash test expression to inspect
 
 @param context - execution context inherited by nested expansions
 
 @returns word or nested test work
 
 @example
 ```ts
 visitTest({ expression, context });
 ```
 */
function visitTest(
  {
    expression,
    context,
  }: {
    readonly expression: ForeignBorrowed<UnbashTestExpression>;
    readonly context: ShellCommandContext;
  },
): VisitResult {
  if (expression.type === 'TestUnary') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: wordWorkItems({
        word: expression.operand,
        context,
      },),
    };
  }
  if (expression.type === 'TestBinary') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...wordWorkItems({
          word: expression.left,
          context,
        },),
        ...wordWorkItems({
          word: expression.right,
          context,
        },),
      ],
    };
  }
  if (expression.type === 'TestLogical') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: expression.operands
        .map(function testWorkItem(operand,) {
          return {
            kind: 'test' as const,
            expression: operand,
            context,
          };
        },),
    };
  }
  if (expression.type === 'TestNot') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [{
        kind: 'test',
        expression: expression.operand,
        context,
      },],
    };
  }
  return {
    ...EMPTY_VISIT_RESULT,
    workItems: [{
      kind: 'test',
      expression: expression.expression,
      context,
    },],
  };
}

export {
  visitArithmetic,
  visitTest,
};
