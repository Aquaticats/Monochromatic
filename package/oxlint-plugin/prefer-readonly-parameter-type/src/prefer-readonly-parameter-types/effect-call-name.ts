import type { Expression, } from 'typescript/ast';
import {
  isIdentifier,
  isPropertyAccessExpression,
} from 'typescript/ast/is';

/**
 Builds stable authored name for unresolved callable expression.
 
 Property-access recursion removes formatting whitespace around dots without
 rewriting string literals or other computed expressions.
 
 @param expression - Unresolved callable expression.
 
 @returns Plain call name suitable for diagnostics and contract matching.
 
 @example
 ```ts
 effectCallName(propertyAccess);
 // 'context.git.candidates'
 ```
 */
export function effectCallName(expression: Expression,): string {
  if (isIdentifier(expression,))
    return expression.text;
  if (isPropertyAccessExpression(expression,)) {
    /**
     Canonical receiver name for nested property access.
     */
    const receiverName = effectCallName(expression.expression,);
    /**
     Final authored member identifier.
     */
    const memberName = expression.name
      .text;
    return `${receiverName}.${memberName}`;
  }
  return expression.getText()
    .trim();
}
