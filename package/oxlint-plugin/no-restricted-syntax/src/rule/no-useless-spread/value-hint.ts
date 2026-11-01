/**
 Syntax-only evidence about what a spread argument evaluates to.

 Mirrors the `const_eval` classification of oxlint's `unicorn/no-useless-spread`
 with two corrections: method names shared by arrays, strings, typed arrays, and
 iterators yield `ambiguous` instead of `array`, and `Object.create` is not
 treated as a plain object because spreading it drops the prototype.

 @module
 */

import type { ESTree, } from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  getStaticCallMemberName,
  NO_STATIC_MEMBER_NAME,
} from '../ast-shared.ts';
import { unwrapExpression, } from '../no-immediate-mutation.syntax.ts';
import { TYPED_ARRAY_NAMES, } from '../spread-evidence/typed-array-names.ts';

/**
 What syntax alone proves about an expression.

 - `array`: certainly a fresh array; `needsFill` marks `new Array(length)`,
   whose spread densifies holes.
 - `object`: certainly a fresh plain object.
 - `typed-array`: certainly a typed array, so spreading converts it.
 - `promise-array`: a promise that resolves to a fresh array.
 - `ambiguous`: result of a method whose return kind depends on the receiver type.
 - `unknown`: nothing provable.

 @example
 ```ts
 const hint: SpreadValueHint = { kind: 'ambiguous', method: 'slice' };
 ```
 */
export type SpreadValueHint =
  | {
    readonly kind: 'array';
    readonly needsFill: boolean
  }
    | { readonly kind: 'object'; }
    | { readonly kind: 'typed-array'; }
    | { readonly kind: 'promise-array'; }
    | {
      readonly kind: 'ambiguous';
      readonly method: string
    }
    | { readonly kind: 'unknown'; };

/**
 Decides whether an identifier reads an unshadowed global.
 */
export type IsGlobalIdentifier = (identifier: ESTree.IdentifierReference) => boolean;

/**
 Recursive syntax-only classifier signature shared by the mutually recursive helpers.
 */
type ValueHintClassifier = (
  options: {
    readonly expression: ESTree.Expression;
    readonly isGlobal: IsGlobalIdentifier;
  },
) => SpreadValueHint;

/**
 Methods returning a fresh value of the receiver's own kind (array, string, typed
 array, or iterator), so syntax cannot tell a useless copy from a conversion.
 */
export const AMBIGUOUS_FRESH_METHODS: ReadonlySet<string> = new Set([
  'concat',
  'copyWithin',
  'filter',
  'flat',
  'flatMap',
  'map',
  'slice',
  'splice',
  'toReversed',
  'toSorted',
  'toSpliced',
  'with',
  'split',
],);

/**
 Hint for anything syntax cannot classify.
 */
const UNKNOWN: SpreadValueHint = { kind: 'unknown', };

/**
 Sentinel for a member call whose receiver is not an unshadowed global identifier.
 */
const NO_GLOBAL_RECEIVER: unique symbol = Symbol('method call receiver is not an unshadowed global identifier',);

/**
 Returns the unshadowed global object name a static member call is made on.

 @param call - Call to inspect.

 @param isGlobal - Global-reference predicate from the rule context.

 @returns Global receiver name, or {@link NO_GLOBAL_RECEIVER}.

 @example
 ```ts
 globalReceiverName({ call, isGlobal }); // 'Array' for Array.from(x)
 ```
 */
function globalReceiverName(
  {
    call,
    isGlobal,
  }: ForeignBorrowed<{
    readonly call: ESTree.CallExpression;
    readonly isGlobal: IsGlobalIdentifier;
  }>,
): string | typeof NO_GLOBAL_RECEIVER {
  if (call.callee
    .type
    !== 'MemberExpression')
    return NO_GLOBAL_RECEIVER;
  /**
   Receiver of the member call.
   */
  const { object, } = call.callee;
  if ((object.type !== 'Identifier') || (!isGlobal(object,)))
    return NO_GLOBAL_RECEIVER;
  return object.name;
}

/**
 Classifies calls on global constructors such as `Array.from(x)`.

 @param call - Call to inspect.

 @param receiver - Global receiver name.

 @param method - Static method name.

 @returns Hint for recognized global factories, `unknown` otherwise.

 @example
 ```ts
 globalFactoryHint({ call, receiver: 'Object', method: 'keys' }); // array
 ```
 */
function globalFactoryHint(
  {
    call,
    receiver,
    method,
  }: ForeignBorrowed<{
    readonly call: ESTree.CallExpression;
    readonly receiver: string;
    readonly method: string;
  }>,
): SpreadValueHint {
  /**
   Whether the call has exactly one argument.
   */
  const unary = call.arguments
    .length
    === 1;
  if (TYPED_ARRAY_NAMES.has(receiver,) && (method === 'from')
    && unary)
    return { kind: 'typed-array', };
  if ((receiver === 'Array') && ((method === 'from') || (method === 'of')))
    return {
      kind: 'array',
      needsFill: false,
    };
  if ((receiver === 'Object') && [
    'keys',
    'values',
    'entries',
  ].includes(method,))
    return {
      kind: 'array',
      needsFill: false,
    };
  if ((receiver === 'Object') && (method === 'fromEntries')
    && unary)
    return { kind: 'object', };
  if ((receiver === 'Promise') && ((method === 'all') || (method === 'allSettled'))
    && unary)
    return { kind: 'promise-array', };
  return UNKNOWN;
}

/**
 Classifies `new X(...)` expressions.

 @param expression - Constructor call.

 @param isGlobal - Global-reference predicate.

 @returns Hint for global `Array`, typed array, and argument-free `Object` constructors.

 @example
 ```ts
 newExpressionHint({ expression, isGlobal }); // new Array(3) -> array needing fill
 ```
 */
function newExpressionHint(
  {
    expression,
    isGlobal,
  }: ForeignBorrowed<{
    readonly expression: ESTree.NewExpression;
    readonly isGlobal: IsGlobalIdentifier;
  }>,
): SpreadValueHint {
  /**
   Constructor being invoked.
   */
  const { callee, } = expression;
  if ((callee.type !== 'Identifier') || (!isGlobal(callee,)))
    return UNKNOWN;
  if (callee.name === 'Array')
    return {
      kind: 'array',
      needsFill: (expression.arguments
        .length
        === 1) && (expression.arguments[0]
          ?.type
          !== 'SpreadElement'),
    };
  if (TYPED_ARRAY_NAMES.has(callee.name,) && (expression.arguments
    .length
    > 0))
    return { kind: 'typed-array', };
  if ((callee.name === 'Object') && (expression.arguments
    .length
    === 0))
    return { kind: 'object', };
  return UNKNOWN;
}

/**
 Merges the hints of both branches of a conditional: only agreeing certain kinds
 survive, because a spread useless on one branch may convert on the other.

 @param left - Consequent hint.

 @param right - Alternate hint.

 @returns Shared certain hint, or `unknown`.

 @example
 ```ts
 combineBranchHints({ left: arrayHint, right: arrayHint }); // array
 ```
 */
function combineBranchHints(
  {
    left,
    right,
  }: {
    readonly left: SpreadValueHint;
    readonly right: SpreadValueHint;
  },
): SpreadValueHint {
  if ((left.kind === 'array') && (right.kind === 'array'))
    return {
      kind: 'array',
      needsFill: false,
    };
  if (((left.kind === 'object') && (right.kind === 'object'))
    || ((left.kind === 'typed-array') && (right.kind === 'typed-array'))
    || ((left.kind === 'promise-array') && (right.kind === 'promise-array')))
    return left;
  return UNKNOWN;
}

/**
 Classifies a call: global factories are certain, fresh-value methods inherit a
 certain receiver kind and are otherwise ambiguous.

 @param call - Call being classified.

 @param isGlobal - Global-reference predicate.

 @param classify - Recursive classifier for receivers and accumulators, passed in
 because {@link valueHint} and this function recurse into each other.

 @returns Syntax-only hint of the call result.

 @example
 ```ts
 callExpressionHint({ call, isGlobal, classify: valueHint }); // text.slice(0, 3) -> ambiguous
 ```
 */
function callExpressionHint(
  {
    call,
    isGlobal,
    classify,
  }: ForeignBorrowed<{
    readonly call: ESTree.CallExpression;
    readonly isGlobal: IsGlobalIdentifier;
    readonly classify: ValueHintClassifier;
  }>,
): SpreadValueHint {
  if (call.optional)
    return UNKNOWN;
  /**
   Static method name, or sentinel for computed and non-member callees.
   */
  const method = getStaticCallMemberName({ call, },);
  if ((method === NO_STATIC_MEMBER_NAME) || (call.callee
    .type
    !== 'MemberExpression'))
    return UNKNOWN;
  /**
   Global receiver name for factory calls such as `Array.from`.
   */
  const receiver = globalReceiverName({
    call,
    isGlobal,
  },);
  if (receiver !== NO_GLOBAL_RECEIVER)
    return globalFactoryHint({
      call,
      receiver,
      method,
    },);
  if (method === 'reduce') {
    /**
     Initial accumulator of `reduce(callback, initial)`.
     */
    const [, initial,] = call.arguments;
    if ((call.arguments
      .length
      !== 2) || (initial === undefined)
      || (initial.type === 'SpreadElement'))
      return UNKNOWN;
    return classify({
      expression: initial,
      isGlobal,
    },)
      .kind
      === 'array'
      ? {
        kind: 'ambiguous',
        method,
      }
      : UNKNOWN;
  }
  if (!AMBIGUOUS_FRESH_METHODS.has(method,))
    return UNKNOWN;
  if (call.callee
    .object
    .type
    === 'Super')
    return UNKNOWN;
  /**
   Syntax-only hint of the receiver.
   */
  const receiverHint = classify({
    expression: call.callee
      .object,
    isGlobal,
  },);
  if (receiverHint.kind === 'typed-array')
    return receiverHint;
  if ((receiverHint.kind === 'array') && (method !== 'split'))
    return {
      kind: 'array',
      needsFill: false,
    };
  return {
    kind: 'ambiguous',
    method,
  };
}

/**
 Classifies what a spread argument evaluates to from syntax alone.

 @param expression - Spread argument or nested sub-expression.

 @param isGlobal - Global-reference predicate, so shadowed `Array` or `Object` never count.

 @returns Syntax-only hint.

 @example
 ```ts
 valueHint({ expression: spread.argument, isGlobal });
 ```
 */
export function valueHint(
  {
    expression,
    isGlobal,
  }: ForeignBorrowed<{
    readonly expression: ESTree.Expression;
    readonly isGlobal: IsGlobalIdentifier;
  }>,
): SpreadValueHint {
  /**
   Expression with parentheses and type-only wrappers removed.
   */
  const inner = unwrapExpression({ expression, },);
  if (inner.type === 'ArrayExpression')
    return {
      kind: 'array',
      needsFill: false,
    };
  if (inner.type === 'ObjectExpression')
    return { kind: 'object', };
  if (inner.type === 'AwaitExpression') {
    /**
     Hint of the awaited operand.
     */
    const awaited = valueHint({
      expression: inner.argument,
      isGlobal,
    },);
    return awaited.kind === 'promise-array'
      ? {
        kind: 'array',
        needsFill: false,
      }
      : UNKNOWN;
  }
  if (inner.type === 'SequenceExpression') {
    /**
     Value-producing last operand of the sequence.
     */
    const last = inner.expressions
      .at(-1,);
    return last === undefined ? UNKNOWN : valueHint({
      expression: last,
      isGlobal,
    },);
  }
  if (inner.type === 'ConditionalExpression')
    return combineBranchHints({
      left: valueHint({
        expression: inner.consequent,
        isGlobal,
      },),
      right: valueHint({
        expression: inner.alternate,
        isGlobal,
      },),
    },);
  if (inner.type === 'NewExpression')
    return newExpressionHint({
      expression: inner,
      isGlobal,
    },);
  if (inner.type === 'CallExpression')
    return callExpressionHint({
      call: inner,
      isGlobal,
      classify: valueHint,
    },);
  return UNKNOWN;
}
