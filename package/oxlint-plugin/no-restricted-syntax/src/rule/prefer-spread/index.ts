/**
 Project replacement for oxlint's `unicorn/prefer-spread`.

 Upstream rewrites `x.slice()` to `[...x]` under `--fix`, guessing from the
 identifier's spelling whether `x` is an array, which turns a typed-array copy into
 a plain array (repo issue 565). This rule fixes only receivers TypeScript proves
 are arrays, leaves proven typed arrays and strings alone, and reports untyped
 receivers without a fix so the author states the conversion.

 @module
 */

import type {
  Context,
  CreateOnceRule,
  ESTree,
  Fix,
  Fixer,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  getSingleNonSpreadArgument,
  getStaticCallMemberName,
  NO_SINGLE_ARGUMENT,
} from '../ast-shared.ts';
import { unwrapExpression, } from '../no-immediate-mutation.syntax.ts';
import { expressionValueKind, } from '../spread-evidence/expression-kind.ts';
import { isUndeclaredReference, } from '../spread-evidence/undeclared-reference.ts';
import { copyCall, } from './receiver.ts';

//region Spread text
// Rendering an expression as a spread argument, parenthesized only where it
// binds looser than spread.

/**
 Spread text of an expression, parenthesized unless it binds tighter than spread.

 @param context - Rule context supplying source text.

 @param expression - Expression to spread.

 @returns Array literal text spreading the expression.

 @example
 ```ts
 spreadText({ context, expression }); // '[...rows]'
 ```
 */
function spreadText(
  {
    context,
    expression,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly expression: ESTree.Expression;
  }>,
): string {
  /**
   Verbatim source of the expression.
   */
  const text = context.sourceCode
    .getText(expression,);
  return expression.type === 'SequenceExpression' ? `[...(${text})]` : `[...${text}]`;
}

//endregion Spread text

//region Array.from
// Reporting `Array.from(x)` where TypeScript proves `x` is already an array or
// typed array, so `[...x]` builds the same plain array.

/**
 Reports `Array.from(x)` when TypeScript proves `x` is an array or typed array,
 where `[...x]` builds the same plain array.

 `Array.from` also accepts array-likes and splits strings without tripping
 `typescript/no-misused-spread`, so any other or unknown `x` keeps the explicit form.

 @param context - Rule context receiving the diagnostic.

 @param call - `Array.from(x)` call.

 @mutates context - Emits diagnostics through the foreign rule context.

 @example
 ```ts
 reportArrayFrom({ context, call });
 ```
 */
function reportArrayFrom(
  {
    context,
    call,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly call: ESTree.CallExpression;
  }>,
): void {
  if (call.optional || (call.callee
    .type
    !== 'MemberExpression'))
    return;
  /**
   Receiver of the static call.
   */
  const { object, } = call.callee;
  if ((object.type !== 'Identifier') || (object.name !== 'Array')
    || (!isUndeclaredReference({
      context,
      identifier: object,
    },)))
    return;
  /**
   Sole ordinary argument, or sentinel.
   */
  const argument = getSingleNonSpreadArgument({ call, },);
  if ((argument === NO_SINGLE_ARGUMENT) || (unwrapExpression({ expression: argument, },)
    .type
    === 'ObjectExpression'))
    return;
  /**
   Semantic kind of the converted value.
   */
  const kind = expressionValueKind({
    context,
    node: argument,
  },);
  if ((kind !== 'array') && (kind !== 'typed-array'))
    return;
  /**
   Replacement array literal.
   */
  const replacement = spreadText({
    context,
    expression: argument,
  },);
  context.report({
    node: call,
    messageId: 'preferSpreadOverArrayFrom',
    fix(fixer: ForeignBorrowed<Fixer>,): Fix {
      return fixer.replaceText(
        call,
        replacement,
      );
    },
  },);
}

//endregion Array.from

//region Copy calls
// Reporting the copy methods `x.slice()`, `x.slice(0)`, `x.toSpliced()`, and
// `x.concat(...)` by the semantic kind of their receiver.

/**
 Reports `x.slice()`, `x.slice(0)`, `x.toSpliced()`, and `x.concat(...)` by the
 receiver's semantic kind.

 @param context - Rule context receiving the diagnostic.

 @param call - Candidate copy call.

 @mutates context - Emits diagnostics through the foreign rule context.

 @example
 ```ts
 reportCopyCall({ context, call });
 ```
 */
function reportCopyCall(
  {
    context,
    call,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly call: ESTree.CallExpression;
  }>,
): void {
  /**
   Copy method and receiver, or `none`.
   */
  const candidate = copyCall({
    call,
    isGlobal: function isGlobal(identifier,): boolean {
      return isUndeclaredReference({
        context,
        identifier,
      },);
    },
  },);
  if (candidate.method === 'none')
    return;
  /**
   Semantic kind of the receiver.
   */
  const kind = expressionValueKind({
    context,
    node: candidate.receiver,
  },);
  if (kind === 'untyped') {
    context.report({
      node: call,
      messageId: candidate.method === 'concat' ? 'ambiguousConcat' : 'ambiguousCopy',
      data: { method: candidate.method, },
    },);
    return;
  }
  if (kind !== 'array')
    return;
  if (candidate.method === 'concat') {
    context.report({
      node: call,
      messageId: 'preferSpreadOverConcat',
    },);
    return;
  }
  /**
   Replacement array literal.
   */
  const replacement = spreadText({
    context,
    expression: candidate.receiver,
  },);
  context.report({
    node: call,
    messageId: 'preferSpreadOverCopy',
    data: { method: candidate.method, },
    fix(fixer: ForeignBorrowed<Fixer>,): Fix {
      return fixer.replaceText(
        call,
        replacement,
      );
    },
  },);
}

//endregion Copy calls

//region The rule
// The exported rule, which fixes proven array copies and asks untyped code to
// state the conversion it means.

/**
 Prefers `[...x]` for array copies and conversions where TypeScript proves the
 spread builds the same array, and asks untyped code to state which copy it means.

 @example
 ```ts
 const copy = rows.slice(); // fixed to [...rows] when rows is an array
 const view = bytes.slice(); // left alone when bytes is a Uint8Array
 ```
 */
export const preferSpread: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    fixable: 'code',
    docs: {
      description:
        'Prefer spread for array copies TypeScript proves, and require untyped code to state which copy it means.',
      recommended: true,
    },
    messages: {
      preferSpreadOverArrayFrom: 'This value is an array or typed array, so `Array.from()` and a spread build the same array; write `[...value]`.',
      preferSpreadOverCopy: 'This receiver is an array, so `{{method}}()` only copies it; write `[...receiver]`.',
      preferSpreadOverConcat: 'This receiver is an array; write `[...receiver, ...other]` instead of `concat()`, spreading only the arguments that are arrays.',
      ambiguousCopy:
        'No type information is available here, so `{{method}}()` is ambiguous: it copies an array into an array, but a typed array stays a typed array and a string stays a string. Say which: `[...receiver]` for an array, the matching constructor such as `new Uint8Array(receiver)` for a typed array, and the string itself, which is immutable.',
      ambiguousConcat:
        'No type information is available here, so `concat()` is ambiguous: it joins arrays, but concatenates text on a string. Say which: `[...receiver, ...other]` for arrays, and a template literal for strings.',
    },
  },
  /**
   Creates the call visitor.

   @param context - Foreign rule context receiving diagnostics.

   @mutates context - Emits diagnostics through the foreign rule context.

   @example
   ```ts
   preferSpread.createOnce(context);
   ```
   */
  createOnce(context: ForeignBorrowed<Context>,): VisitorWithHooks {
    return {
      CallExpression(node: ForeignBorrowed<ESTree.CallExpression>,): void {
        if (getStaticCallMemberName({ call: node, },) === 'from') {
          reportArrayFrom({
            context,
            call: node,
          },);
          return;
        }
        reportCopyCall({
          context,
          call: node,
        },);
      },
    };
  },
};

//endregion The rule
