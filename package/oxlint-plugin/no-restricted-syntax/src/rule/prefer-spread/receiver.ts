/**
 Syntax-only receiver checks for the prefer-spread replacement: which receivers
 are provably not arrays, and which calls are candidates at all.

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
 Buffer constructors whose instances cannot be spread at all.
 */
const BUFFER_CONSTRUCTORS: ReadonlySet<string> = new Set([
  'ArrayBuffer',
  'SharedArrayBuffer',
],);

/**
 Whether syntax proves a receiver is not an array: literals, template literals,
 arithmetic or concatenation results, `join()` results, and fresh typed arrays or buffers.

 Unlike upstream, identifier capitalization is not treated as evidence.

 @param receiver - Receiver of `slice`, `concat`, or `toSpliced`.

 @returns Whether the receiver is provably not an array.

 @example
 ```ts
 isProvablyNotArray({ receiver }); // true for `'abc'.slice()`
 ```
 */
export function isProvablyNotArray(
  { receiver, }: ForeignBorrowed<{ readonly receiver: ESTree.Expression; }>,
): boolean {
  /**
   Receiver without parentheses and type-only wrappers.
   */
  const inner = unwrapExpression({ expression: receiver, },);
  if ((inner.type === 'Literal') || (inner.type === 'TemplateLiteral')
    || (inner.type === 'BinaryExpression')
    || (inner.type === 'ThisExpression'))
    return true;
  if (inner.type === 'NewExpression')
    return (inner.callee
      .type
      === 'Identifier')
      && (TYPED_ARRAY_NAMES.has(inner.callee
        .name,) || BUFFER_CONSTRUCTORS.has(inner.callee
          .name,));
  if (inner.type === 'CallExpression')
    return (getStaticCallMemberName({ call: inner, },) === 'join') && (inner.arguments
      .length
      < 2);
  return false;
}

/**
 Copy-method calls the rule considers, with their receiver.

 - `slice`: `x.slice()` or `x.slice(0)`.
 - `toSpliced`: `x.toSpliced()`.
 - `concat`: `x.concat(...)`.
 */
export type CopyCall =
  | {
    readonly method: 'concat' | 'slice' | 'toSpliced';
    readonly receiver: ESTree.Expression
  }
    | { readonly method: 'none'; };

/**
 Whether an argument list is empty or a single literal `0`.

 @param args - Call arguments.

 @returns Whether `slice` with these arguments copies the whole receiver.

 @example
 ```ts
 copiesWholeReceiver([zeroLiteral]); // true
 ```
 */
function copiesWholeReceiver(args: ForeignBorrowed<readonly ESTree.Argument[]>,): boolean {
  if (args.length === 0)
    return true;
  /**
   Sole argument, if any.
   */
  const [only,] = args;
  return (args.length === 1) && (only !== undefined)
    && (only.type === 'Literal')
    && (only.value === 0);
}

/**
 Classifies a call as one of the copy methods the rule considers.

 @param call - Call to inspect.

 @param isGlobal - Global-reference predicate; a global receiver such as `Buffer` in
 `Buffer.concat(chunks)` is a static-method namespace, never an array.

 @returns Method and receiver, or `none`.

 @example
 ```ts
 copyCall({ call, isGlobal }); // { method: 'slice', receiver } for rows.slice()
 ```
 */
export function copyCall(
  {
    call,
    isGlobal,
  }: ForeignBorrowed<{
    readonly call: ESTree.CallExpression;
    readonly isGlobal: (identifier: ESTree.IdentifierReference) => boolean;
  }>,
): CopyCall {
  if (call.optional || (call.callee
    .type
    !== 'MemberExpression')
    || (call.callee
      .object
      .type
      === 'Super'))
    return { method: 'none', };
  /**
   Static method name, or sentinel.
   */
  const method = getStaticCallMemberName({ call, },);
  if (method === NO_STATIC_MEMBER_NAME)
    return { method: 'none', };
  /**
   Receiver of the copy method.
   */
  const receiver = call.callee
    .object;
  /**
   Receiver without parentheses and type-only wrappers.
   */
  const bareReceiver = unwrapExpression({ expression: receiver, },);
  if ((bareReceiver.type === 'ArrayExpression') || isProvablyNotArray({ receiver, },))
    return { method: 'none', };
  if ((bareReceiver.type === 'Identifier') && isGlobal(bareReceiver,))
    return { method: 'none', };
  if ((method === 'slice') && copiesWholeReceiver(call.arguments,))
    return {
      method,
      receiver,
    };
  if ((method === 'toSpliced') && (call.arguments
    .length
    === 0))
    return {
      method,
      receiver,
    };
  if (method === 'concat')
    return {
      method,
      receiver,
    };
  return { method: 'none', };
}
