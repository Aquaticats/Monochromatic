/**
 Syntax recognition for duplicated caught-value formatters.
 
 @module
 */

import type {
  Context,
  ESTree,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { findVariable, } from './no-sync.syntax.ts';

/**
 Sentinel returned when an expression is not a supported identifier.
 */
export const NO_IDENTIFIER: unique symbol = Symbol('prefer-caught-value-text identifier not found',);

/**
 Reads an identifier name from an expression.
 
 @param expression - Expression candidate.
 
 @returns identifier name or sentinel.
 
 @example
 ```ts
 identifierName({ expression });
 ```
 */
function identifierName(
  { expression, }: ForeignBorrowed<{ readonly expression: ESTree.Expression; }>,
): string | typeof NO_IDENTIFIER {
  return expression.type === 'Identifier'
    ? expression.name
    : NO_IDENTIFIER;
}

/**
 Tests whether identifier resolves to global binding rather than local shadow.
 
 @param context - Oxlint context providing lexical scope metadata.
 
 @param identifier - Identifier reference to resolve.
 
 @returns whether identifier is unresolved or global without local definitions.
 
 @example
 ```ts
 isUnshadowedGlobalIdentifier({ context, identifier });
 ```
 */
function isUnshadowedGlobalIdentifier(
  {
    context,
    identifier,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly identifier: ESTree.IdentifierReference;
  }>,
): boolean {
  /**
   Scope variable behind identifier when metadata includes one.
   */
  const variable = findVariable({
    context,
    node: identifier,
    name: identifier.name,
  },);
  if ((typeof variable) === 'symbol')
    return true;
  /**
   Local definitions proving identifier does not resolve to unshadowed global.
   */
  const { defs, } = variable;
  return defs.length === 0;
}

/**
 Reads identifier tested by direct `Error.isError` call.
 
 @param context - Oxlint context resolving global Error binding.
 
 @param expression - Potential detector call.
 
 @returns tested identifier name or sentinel.
 
 @example
 ```ts
 errorDetectorIdentifier({ context, expression });
 ```
 */
export function errorDetectorIdentifier(
  {
    context,
    expression,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly expression: ESTree.Expression;
  }>,
): string | typeof NO_IDENTIFIER {
  if (expression.type !== 'CallExpression')
    return NO_IDENTIFIER;
  /**
   Potential Error.isError member call target.
   */
  const { callee, } = expression;
  if (callee.type !== 'MemberExpression')
    return NO_IDENTIFIER;
  if (callee.computed)
    return NO_IDENTIFIER;
  /**
   Potential Error constructor and isError property.
   */
  const {
    object,
    property,
  } = callee;
  if (object.type !== 'Identifier')
    return NO_IDENTIFIER;
  if (object.name !== 'Error')
    return NO_IDENTIFIER;
  if (!isUnshadowedGlobalIdentifier({
    context,
    identifier: object,
  },))
    return NO_IDENTIFIER;
  if (property.type !== 'Identifier')
    return NO_IDENTIFIER;
  if (property.name !== 'isError')
    return NO_IDENTIFIER;
  /**
   Value tested by Error.isError.
   */
  const [argument,] = expression.arguments;
  if ((argument === undefined) || (argument.type === 'SpreadElement'))
    return NO_IDENTIFIER;
  return identifierName({ expression: argument, },);
}

/**
 Tests whether expression reads named Error field from detector input.
 
 @param expression - Potential member read.
 
 @param identifier - Detector input name.
 
 @param property - Error field name.
 
 @returns whether expression is exact field read.
 
 @example
 ```ts
 readsErrorField({ expression, identifier: 'error', property: 'message' });
 ```
 */
function readsErrorField(
  {
    expression,
    identifier,
    property,
  }: ForeignBorrowed<{
    readonly expression: ESTree.Expression;
    readonly identifier: string;
    readonly property: 'message' | 'stack';
  }>,
): boolean {
  if (expression.type !== 'MemberExpression')
    return false;
  if (expression.computed)
    return false;
  /**
   Potential Error value and field name.
   */
  const {
    object,
    property: memberProperty,
  } = expression;
  if (object.type !== 'Identifier')
    return false;
  if (object.name !== identifier)
    return false;
  return (memberProperty.type === 'Identifier')
    && (memberProperty.name === property);
}

/**
 Tests whether expression yields Error message or stack diagnostics.
 
 @param expression - Branch result expression.
 
 @param identifier - Detector input name.
 
 @returns whether branch reproduces shared caught-value formatting.
 
 @example
 ```ts
 readsErrorDiagnostic({ expression, identifier: 'error' });
 ```
 */
export function readsErrorDiagnostic(
  {
    expression,
    identifier,
  }: ForeignBorrowed<{
    readonly expression: ESTree.Expression;
    readonly identifier: string;
  }>,
): boolean {
  if (readsErrorField({
    expression,
    identifier,
    property: 'message',
  },))
    return true;
  if (expression.type !== 'LogicalExpression')
    return false;
  if (expression.operator !== '??')
    return false;
  return readsErrorField({
    expression: expression.left,
    identifier,
    property: 'stack',
  },) && readsErrorField({
    expression: expression.right,
    identifier,
    property: 'message',
  },);
}

/**
 Tests whether expression is duplicated non-Error diagnostic fallback.
 
 @param context - Oxlint context resolving global String binding.
 
 @param expression - Alternate branch expression.
 
 @param identifier - Detector input name.
 
 @returns whether branch stringifies or categorizes detector input.
 
 @example
 ```ts
 isDuplicateFallback({ context, expression, identifier: 'error' });
 ```
 */
export function isDuplicateFallback(
  {
    context,
    expression,
    identifier,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly expression: ESTree.Expression;
    readonly identifier: string;
  }>,
): boolean {
  if (expression.type === 'Identifier')
    return expression.name === identifier;
  if (expression.type === 'Literal')
    return (typeof expression.value) === 'string';
  if (expression.type === 'UnaryExpression') {
    return (expression.operator === 'typeof')
      && (identifierName({ expression: expression.argument, },) === identifier);
  }
  if (expression.type === 'TemplateLiteral') {
    /**
     Dynamic template expressions inspected for typeof categorization.
     */
    const { expressions, } = expression;
    // Element binding rather than a callback parameter: provenance from the
    // ForeignBorrowed boundary on `expression` reaches `for...of` bindings but is
    // lost through callback parameters of a method invoked on a marked value, so
    // the callback form makes this parameter read as an unowned inferred type.
    // Tracked upstream of this package as #427.
    for (const templateExpression of expressions) {
      if (
        (templateExpression.type === 'UnaryExpression')
          && (templateExpression.operator === 'typeof')
          && (identifierName({ expression: templateExpression.argument, },) === identifier)
      ) {
        return true;
      }
    }
    return false;
  }
  if (expression.type !== 'CallExpression')
    return false;
  /**
   Potential direct global String call target.
   */
  const { callee, } = expression;
  if (callee.type !== 'Identifier')
    return false;
  if (callee.name !== 'String')
    return false;
  if (!isUnshadowedGlobalIdentifier({
    context,
    identifier: callee,
  },))
    return false;
  /**
   Value passed to direct String conversion.
   */
  const [argument, ...remainingArguments] = expression.arguments;
  return (remainingArguments.length === 0)
    && (argument !== undefined)
    && (argument.type !== 'SpreadElement')
    && (identifierName({ expression: argument, },) === identifier);
}

/**
 Tests whether statement returns duplicated fallback.
 
 @param context - Oxlint context resolving fallback globals.
 
 @param statement - Potential fallback statement.
 
 @param identifier - Detector input name.
 
 @returns whether statement returns categorized detector input.
 
 @example
 ```ts
 returnsDuplicateFallback({ context, statement, identifier: 'error' });
 ```
 */
function returnsDuplicateFallback(
  {
    context,
    statement,
    identifier,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly statement: ESTree.Statement;
    readonly identifier: string;
  }>,
): boolean {
  if (statement.type === 'ReturnStatement') {
    return (statement.argument !== null)
      && isDuplicateFallback({
        context,
        expression: statement.argument,
        identifier,
      },);
  }
  if (statement.type === 'IfStatement') {
    return returnsDuplicateFallback({
      context,
      statement: statement.consequent,
      identifier,
    },) || ((statement.alternate !== null)
      && returnsDuplicateFallback({
        context,
        statement: statement.alternate,
        identifier,
      },));
  }
  if (statement.type !== 'BlockStatement')
    return false;
  /**
   Nested block statements searched for fallback returns.
   */
  const { body, } = statement;
  return body.some(function hasDuplicateFallback(childStatement,): boolean {
    return returnsDuplicateFallback({
      context,
      statement: childStatement,
      identifier,
    },);
  },);
}

/**
 Tests whether statement returns Error diagnostic field.
 
 @param statement - Consequent statement.
 
 @param identifier - Detector input name.
 
 @returns whether statement returns matching diagnostic.
 
 @example
 ```ts
 returnsErrorDiagnostic({ statement, identifier: 'error' });
 ```
 */
function returnsErrorDiagnostic(
  {
    statement,
    identifier,
  }: ForeignBorrowed<{
    readonly statement: ESTree.Statement;
    readonly identifier: string;
  }>,
): boolean {
  if (statement.type === 'ReturnStatement') {
    return (statement.argument !== null)
      && readsErrorDiagnostic({
        expression: statement.argument,
        identifier,
      },);
  }
  if (statement.type !== 'BlockStatement')
    return false;
  /**
   First branch statement, sufficient only when branch contains one return.
   */
  const [onlyStatement, ...remainingStatements] = statement.body;
  return (remainingStatements.length === 0)
    && (onlyStatement !== undefined)
    && returnsErrorDiagnostic({
      statement: onlyStatement,
      identifier,
    },);
}

/**
 Tests whether function starts local caught-value formatter implementation.
 
 @param context - Oxlint context resolving detector and fallback globals.
 
 @param node - Function declaration or expression.
 
 @returns whether body duplicates shared formatter behavior.
 
 @example
 ```ts
 duplicatesCaughtValueFormatter({ context, node });
 ```
 */
export function duplicatesCaughtValueFormatter(
  {
    context,
    node,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly node: ESTree.Function | ESTree.ArrowFunctionExpression;
  }>,
): boolean {
  /**
   Function body before block-only formatter recognition.
   */
  const { body: nodeBody, } = node;
  if (nodeBody === null)
    return false;
  if (nodeBody.type !== 'BlockStatement')
    return false;
  /**
   Body statements where compact formatter starts with Error branch.
   */
  const { body, } = nodeBody;
  /**
   Leading function statement.
   */
  const [firstStatement,] = body;
  if ((firstStatement === undefined) || (firstStatement.type !== 'IfStatement'))
    return false;
  /**
   Identifier tested by leading Error branch.
   */
  const identifier = errorDetectorIdentifier({
    context,
    expression: firstStatement.test,
  },);
  if ((typeof identifier) === 'symbol')
    return false;
  if (!returnsErrorDiagnostic({
    statement: firstStatement.consequent,
    identifier,
  },))
    return false;
  if (firstStatement.alternate !== null) {
    return returnsDuplicateFallback({
      context,
      statement: firstStatement.alternate,
      identifier,
    },);
  }
  return body
    .slice(1,)
    .some(function hasDuplicateFallback(statement,): boolean {
      return returnsDuplicateFallback({
        context,
        statement,
        identifier,
      },);
    },);
}
