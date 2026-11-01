/**
 Classification of TypeScript 7 semantic types into the value kinds that decide
 whether a spread or copy is useless, a real conversion, or unknowable.

 @module
 */

import {
  TypeFlags,
  type Checker,
  type Type,
} from 'typescript/sync';

import { TYPED_ARRAY_NAMES, } from './typed-array-names.ts';

/**
 What a value is, as far as a spread or copy rule needs to know.

 - `array`: an array or tuple, so spreading it into a new array only copies it.
 - `typed-array`: spreading converts it to a plain array.
 - `string`: spreading splits it into code points.
 - `other`: a known non-array type (iterator, set, mixed union), so spreading converts.
 - `untyped`: no usable type (`any`, `unknown`, an error type, or no semantic bridge).

 @example
 ```ts
 const kind: ValueKind = 'array';
 ```
 */
export type ValueKind = 'array' | 'typed-array' | 'string' | 'other' | 'untyped';

/**
 Deepest type-parameter constraint chain followed before giving up.
 */
const MAX_CONSTRAINT_DEPTH = 8;

/**
 Combines member kinds of a union: agreement keeps the kind, any `untyped` member
 poisons the union, and disagreement means some member really converts.

 @param kinds - Kind of every union member.

 @returns Union kind.

 @example
 ```ts
 combineUnionKinds(['array', 'array']); // 'array'
 ```
 */
function combineUnionKinds(kinds: readonly ValueKind[],): ValueKind {
  if (kinds.includes('untyped',))
    return 'untyped';
  /**
   First member kind, compared against every other member.
   */
  const [first,] = kinds;
  if (first === undefined)
    return 'untyped';
  return kinds.every(function sameKind(kind,): boolean {
    return kind === first;
  },)
    ? first
    : 'other';
}

/**
 Combines member kinds of an intersection: a branded array is still an array,
 so the first identifying member kind wins.

 @param kinds - Kind of every intersection member.

 @returns Intersection kind.

 @example
 ```ts
 combineIntersectionKinds(['array', 'other']); // 'array'
 ```
 */
function combineIntersectionKinds(kinds: readonly ValueKind[],): ValueKind {
  /**
   Identifying kinds in precedence order.
   */
  const identifying: readonly ValueKind[] = [
    'array',
    'typed-array',
    'string',
  ];
  /**
   First identifying kind carried by any member.
   */
  const found = identifying.find(function carried(kind,): boolean {
    return kinds.includes(kind,);
  },);
  if (found !== undefined)
    return found;
  return kinds.includes('untyped',) ? 'untyped' : 'other';
}

/**
 Classifies one semantic type.

 @param checker - Checker of the project that produced `type`.

 @param type - Type of the expression being spread or copied.

 @param depth - Type-parameter constraints already followed.

 @returns Value kind of `type`.

 @example
 ```ts
 valueKindOfType({ checker, type, depth: 0 });
 ```
 */
export function valueKindOfType(
  {
    checker,
    type,
    depth,
  }: {
    readonly checker: Checker;
    readonly type: Type;
    readonly depth: number;
  },
): ValueKind {
  if (((type.flags & TypeFlags.AnyOrUnknown) !== 0) || type.isErrorType())
    return 'untyped';
  if (type.isTypeParameter()) {
    /**
     Constraint standing in for an unresolved type parameter.
     */
    const constraint = checker.getBaseConstraintOfType(type,);
    if ((constraint === undefined) || (depth >= MAX_CONSTRAINT_DEPTH))
      return 'untyped';
    return valueKindOfType({
      checker,
      type: constraint,
      depth: depth + 1,
    },);
  }
  if (type.isUnionType() || type.isIntersectionType()) {
    /**
     Kind of every member type.
     */
    const kinds = (type.getTypes() ?? []).map(function memberKind(member,): ValueKind {
      return valueKindOfType({
        checker,
        type: member,
        depth,
      },);
    },);
    return type.isUnionType() ? combineUnionKinds(kinds,) : combineIntersectionKinds(kinds,);
  }
  if ((type.flags & TypeFlags.StringLike) !== 0)
    return 'string';
  if (checker.isArrayType(type,) || checker.isTupleType(type,))
    return 'array';
  if (TYPED_ARRAY_NAMES.has(type.getSymbol()
    ?.name
    ?? '',))
    return 'typed-array';
  return 'other';
}
