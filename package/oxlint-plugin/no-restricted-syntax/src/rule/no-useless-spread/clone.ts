/**
 Single spreads that copy a value already fresh: `[...Array.from(x)]`,
 `[...await Promise.all(x)]`, `{ ...Object.fromEntries(x) }`.

 When syntax cannot tell whether the spread value is an array (`[...x.slice()]`),
 the TypeScript 7 bridge decides; without a type the spread is reported as an
 ambiguous conversion with no fix, because code without types must state its
 conversions explicitly.

 @module
 */

import type {
  Context,
  ESTree,
  Fix,
  Fixer,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { expressionValueKind, } from '../spread-evidence/expression-kind.ts';
import { isUndeclaredReference, } from '../spread-evidence/undeclared-reference.ts';
import { valueHint, } from './value-hint.ts';

//region Rewrite vocabulary
// The expression types that bind tighter than any operator, so their text can
// stand where the enclosing literal stood, and the longest argument text a
// diagnostic quotes.

/**
 Expression types that bind tighter than any operator, so their text can stand
 anywhere the enclosing literal stood.
 */
const PRIMARY_TYPES: ReadonlySet<string> = new Set([
  'ArrayExpression',
  'CallExpression',
  'Identifier',
  'MemberExpression',
  'NewExpression',
  'ObjectExpression',
  'ParenthesizedExpression',
],);

/**
 Longest argument text quoted in a diagnostic.
 */
const MAX_QUOTED_LENGTH = 50;

//endregion Rewrite vocabulary

//region Replacement and diagnostic text
// The spread argument's source text with parentheses only where precedence
// needs them, and the short quoted form diagnostics use.

/**
 Source text of a spread argument, parenthesized unless it is a primary expression.

 @param context - Rule context supplying source text.

 @param argument - Spread argument.

 @returns Replacement text safe in the literal's position.

 @example
 ```ts
 replacementText({ context, argument }); // '(await Promise.all(tasks))'
 ```
 */
function replacementText(
  {
    context,
    argument,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly argument: ESTree.Expression;
  }>,
): string {
  /**
   Verbatim argument source.
   */
  const text = context.sourceCode
    .getText(argument,);
  return PRIMARY_TYPES.has(argument.type,) ? text : `(${text})`;
}

/**
 Short argument text for diagnostics; long or multi-line arguments get a generic label.

 @param text - Argument source text.

 @returns Quotable label.

 @example
 ```ts
 quotedExpression('rows.slice()'); // 'rows.slice()'
 ```
 */
function quotedExpression(text: string,): string {
  return (text.length > MAX_QUOTED_LENGTH) || text.includes('\n',) ? 'this expression' : text;
}

//endregion Replacement and diagnostic text

//region Reporting
// Reporting a fresh array copy and a fresh array or object spread, each with the
// fix that drops the spread.

/**
 Reports the array copy `[...x]` when `x` is proven fresh, fixing it to `x`.

 @param context - Rule context receiving the diagnostic.

 @param array - Single-spread array literal.

 @param spread - Its spread element.

 @param needsFill - Whether `x` is `new Array(length)`, whose copy densifies holes.

 @example
 ```ts
 reportUselessArrayCopy({ context, array, spread, needsFill: false });
 ```

 @mutates context - Emits diagnostics through the foreign rule context.
 */
function reportUselessArrayCopy(
  {
    context,
    array,
    spread,
    needsFill,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly array: ESTree.ArrayExpression;
    readonly spread: ESTree.SpreadElement;
    readonly needsFill: boolean;
  }>,
): void {
  /**
   Replacement for the whole array literal.
   */
  const replacement = needsFill
    ? `${replacementText({
      context,
      argument: spread.argument,
    },)}.fill()`
    : replacementText({
      context,
      argument: spread.argument,
    },);
  context.report({
    node: spread,
    messageId: 'cloneArray',
    data: { expression: quotedExpression(context.sourceCode
      .getText(spread.argument,),), },
    fix(fixer: ForeignBorrowed<Fixer>,): Fix {
      return fixer.replaceText(
        array,
        replacement,
      );
    },
  },);
}

/**
 Reports `[...x]` or `{ ...x }` when `x` is already a fresh array or object.

 @param context - Rule context receiving the diagnostic.

 @param literal - Array or object literal whose only item is `spread`.

 @param spread - The literal's single spread element.

 @example
 ```ts
 reportUselessClone({ context, literal: node, spread });
 ```

 @mutates context - Emits diagnostics through the foreign rule context.
 */
export function reportUselessClone(
  {
    context,
    literal,
    spread,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly literal: ESTree.ArrayExpression | ESTree.ObjectExpression;
    readonly spread: ESTree.SpreadElement;
  }>,
): void {
  /**
   Syntax-only evidence about the spread value.
   */
  const hint = valueHint({
    expression: spread.argument,
    isGlobal: function isGlobal(identifier,): boolean {
      return isUndeclaredReference({
        context,
        identifier,
      },);
    },
  },);
  if (literal.type === 'ObjectExpression') {
    if (hint.kind !== 'object')
      return;
    /**
     Replacement for the whole object literal.
     */
    const replacement = replacementText({
      context,
      argument: spread.argument,
    },);
    context.report({
      node: spread,
      messageId: 'cloneObject',
      data: { expression: quotedExpression(context.sourceCode
        .getText(spread.argument,),), },
      fix(fixer: ForeignBorrowed<Fixer>,): Fix {
        return fixer.replaceText(
          literal,
          replacement,
        );
      },
    },);
    return;
  }
  if (hint.kind === 'array') {
    reportUselessArrayCopy({
      context,
      array: literal,
      spread,
      needsFill: hint.needsFill,
    },);
    return;
  }
  if (hint.kind !== 'ambiguous')
    return;
  /**
   Semantic value kind, or `untyped` when the bridge cannot answer.
   */
  const kind = expressionValueKind({
    context,
    node: spread.argument,
  },);
  if (kind === 'array') {
    reportUselessArrayCopy({
      context,
      array: literal,
      spread,
      needsFill: false,
    },);
    return;
  }
  if (kind !== 'untyped')
    return;
  context.report({
    node: spread,
    messageId: 'ambiguousConversion',
    data: {
      expression: quotedExpression(context.sourceCode
        .getText(spread.argument,),),
      method: hint.method,
    },
  },);
}

//endregion Reporting
