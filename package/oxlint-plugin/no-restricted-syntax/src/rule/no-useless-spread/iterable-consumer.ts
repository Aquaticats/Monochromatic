/**
 Array spreads handed to something that already accepts any iterable:
 `for (… of [...x])`, `yield* [...x]`, `new Set([...x])`, `Promise.all([...x])`,
 `Array.from([...x])`, and `Object.fromEntries([...x])`.

 @module
 */

import type {
  Context,
  ESTree,
  Fix,
  Fixer,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  getStaticCallMemberName,
  NO_STATIC_MEMBER_NAME,
} from '../ast-shared.ts';
import { TYPED_ARRAY_NAMES, } from '../spread-evidence/typed-array-names.ts';
import { isUndeclaredReference, } from '../spread-evidence/undeclared-reference.ts';

//region Consumer vocabulary
// The sentinel for an array literal whose parent accepts no arbitrary iterable,
// and the constructors and static methods whose single argument may be one.

/**
 Sentinel for an array literal whose parent does not accept an arbitrary iterable in its place.
 */
const NOT_ITERABLE_CONSUMER: unique symbol = Symbol('array literal parent does not accept an arbitrary iterable in its place',);

/**
 Collection constructors whose single argument may be any iterable.
 */
const ITERABLE_CONSTRUCTORS: ReadonlySet<string> = new Set([
  'Map',
  'WeakMap',
  'Set',
  'WeakSet',
],);

/**
 Static methods, keyed by global receiver, whose single argument may be any iterable.
 */
const ITERABLE_STATIC_METHODS: Readonly<Record<string, readonly string[]>> = {
  Promise: [
    'all',
    'allSettled',
    'any',
    'race',
  ],
  Array: ['from',],
  Object: ['fromEntries',],
  ...Object.fromEntries(
    TYPED_ARRAY_NAMES.values()
      .map(function typedFrom(name,): readonly [
        string,
        readonly string[],
      ] {
        return [
          name,
          ['from',],
        ];
      },),
  ),
};

//endregion Consumer vocabulary

//region Naming the consumer
// Matching the array literal's parent against that vocabulary, so a diagnostic
// can say which consumer accepts an iterable.

/**
 Names the consumer that accepts an iterable when `array` is its sole argument.

 @param context - Rule context supplying global-reference checks.

 @param array - Single-spread array literal.

 @returns Consumer name for the diagnostic, or {@link NOT_ITERABLE_CONSUMER}.

 @example
 ```ts
 iterableConsumerName({ context, array }); // 'Set' for new Set([...x])
 ```
 */
function iterableConsumerName(
  {
    context,
    array,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly array: ESTree.ArrayExpression;
  }>,
): string | typeof NOT_ITERABLE_CONSUMER {
  /**
   Call or constructor receiving the array.
   */
  const { parent, } = array;
  if ((parent.type !== 'NewExpression') && (parent.type !== 'CallExpression'))
    return NOT_ITERABLE_CONSUMER;
  if ((parent.arguments
    .length
    !== 1) || (parent.arguments[0] !== array))
    return NOT_ITERABLE_CONSUMER;
  if (parent.type === 'NewExpression') {
    /**
     Constructor being invoked.
     */
    const { callee, } = parent;
    if ((callee.type !== 'Identifier') || (!isUndeclaredReference({
      context,
      identifier: callee,
    },)))
      return NOT_ITERABLE_CONSUMER;
    return ITERABLE_CONSTRUCTORS.has(callee.name,) || TYPED_ARRAY_NAMES.has(callee.name,) ? callee.name : NOT_ITERABLE_CONSUMER;
  }
  if (parent.optional || (parent.callee
    .type
    !== 'MemberExpression'))
    return NOT_ITERABLE_CONSUMER;
  /**
   Receiver of the static method call.
   */
  const { object, } = parent.callee;
  if ((object.type !== 'Identifier') || (!isUndeclaredReference({
    context,
    identifier: object,
  },)))
    return NOT_ITERABLE_CONSUMER;
  /**
   Static method being called.
   */
  const method = getStaticCallMemberName({ call: parent, },);
  if (method === NO_STATIC_MEMBER_NAME)
    return NOT_ITERABLE_CONSUMER;
  return (ITERABLE_STATIC_METHODS[object.name] ?? []).includes(method,) ? `${object.name}.${method}` : NOT_ITERABLE_CONSUMER;
}

//endregion Naming the consumer

//region Reporting
// The exported report, which fixes the spread away except where a loop needs
// the snapshot the spread takes.

/**
 Reports `[...x]` passed where any iterable is accepted.

 `for…of` and `yield*` get no fix: the spread snapshots the source, which a loop
 that mutates the source may rely on.

 @param context - Rule context receiving the diagnostic.

 @param array - Array literal whose only element is `spread`.

 @param spread - The array's single spread element.

 @returns Whether a diagnostic was reported.

 @example
 ```ts
 if (reportIterableConsumer({ context, array, spread })) return;
 ```

 @mutates context - Emits diagnostics through the foreign rule context.
 */
export function reportIterableConsumer(
  {
    context,
    array,
    spread,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly array: ESTree.ArrayExpression;
    readonly spread: ESTree.SpreadElement;
  }>,
): boolean {
  /**
   Syntax holding the array.
   */
  const { parent, } = array;
  if ((parent.type === 'ForOfStatement') && (parent.right === array)) {
    context.report({
      node: spread,
      messageId: 'iterableInForOf',
    },);
    return true;
  }
  if ((parent.type === 'YieldExpression') && parent.delegate
    && (parent.argument === array)) {
    context.report({
      node: spread,
      messageId: 'iterableInYieldStar',
    },);
    return true;
  }
  /**
   Iterable-accepting consumer of the array, if any.
   */
  const consumer = iterableConsumerName({
    context,
    array,
  },);
  if (consumer === NOT_ITERABLE_CONSUMER)
    return false;
  /**
   Source text of the spread argument, placed where the array was; a comma
   sequence keeps parentheses so it stays one argument.
   */
  const argumentText = spread.argument
    .type
    === 'SequenceExpression'
    ? `(${context.sourceCode
      .getText(spread.argument,)})`
    : context.sourceCode
      .getText(spread.argument,);
  context.report({
    node: spread,
    messageId: 'iterableToArray',
    data: { consumer, },
    fix(fixer: ForeignBorrowed<Fixer>,): Fix {
      return fixer.replaceText(
        array,
        argumentText,
      );
    },
  },);
  return true;
}

//endregion Reporting
