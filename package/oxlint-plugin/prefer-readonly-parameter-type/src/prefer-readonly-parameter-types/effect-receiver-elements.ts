/**
 Whether a member call's receiver holds strictly primitive elements.
 
 @module
 */

import type { CallExpression, } from 'typescript/ast';
import { isPropertyAccessExpression, } from 'typescript/ast/is';
import {
  type Checker,
  type Type,
  TypeFlags,
} from 'typescript/sync';

/**
 Tests whether every type this call's receiver is instantiated over is primitive.
 
 Asked for the one channel that is narrow conditionally. A member coercing what it read
 reaches user code exactly when an element can supply a `toString` or a `valueOf`.
 
 Strictly primitive rather than "carries no mutable state", and the difference is the whole
 correctness of this. `{ readonly label: string; }` carries nothing writable and is still an
 object, so coercing it reaches whatever `toString` the runtime value actually has.
 `elementCoercionEffect` in `readonly-member-channel-invalid.ts` is the fixture that says so,
 and it caught the looser reading immediately. `receiverElementsArePrimitive` in
 `effect-primitive-origin.ts` does not express this either, for the same reason.
 
 Fails closed on every unresolved step, which is the opposite direction from most neighbours
 and is deliberate: this answer grants a discharge, so not knowing must withhold it.
 
 @param checker - TypeScript checker resolving the receiver's type arguments.
 
 @param call - Member call whose receiver is inspected.
 
 @returns whether coercing any element provably runs nothing.
 
 @example
 ```ts
 receiverElementsArePrimitiveHere({ checker, call });
 ```
 */
export function receiverElementsArePrimitiveHere({
  checker,
  call,
}: {
  readonly checker: Checker;
  readonly call: CallExpression;
},): boolean {
  /**
   Property access selecting the member, whose own expression is the receiver.
   */
  const selector = call.expression;
  if (!isPropertyAccessExpression(selector,))
    return false;
  /**
   Receiver type, whose arguments name what it holds.
   */
  const receiverType = checker.getTypeAtLocation(selector.expression,);
  if ((receiverType === undefined) || (!receiverType.isTypeReference()))
    return false;
  /**
   Types the receiver is instantiated over.
   */
  const elementTypes = checker.getTypeArguments(receiverType,);
  if (elementTypes.length === 0)
    return false;
  return elementTypes
    .every(function isPrimitive(elementType,): boolean {
      return typeIsWhollyPrimitive(elementType,);
    },);
}

/**
 Tests whether a type is a primitive, or a union of nothing but primitives.
 
 @param type - Element type being classified.
 
 @returns whether coercing a value of this type can run no user code.
 
 @example
 ```ts
 typeIsWhollyPrimitive(elementType);
 ```
 */
function typeIsWhollyPrimitive(type: Type,): boolean {
  if (type.isUnionType())
    return type.getTypes()
      .every(function constituentIsPrimitive(constituent,): boolean {
        return typeIsWhollyPrimitive(constituent,);
      },);
  return (type.flags & TypeFlags.Primitive) !== 0;
}
