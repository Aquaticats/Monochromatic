/**
 Definite runtime callability classification for callback positions.
 
 @module
 */

import {
  type Checker,
  SignatureKind,
  type Type,
  TypeFlags,
} from 'typescript/sync';

/**
 Tests whether semantic type is definitely callable at runtime.
 
 @param checker - TypeScript checker resolving constraints and signatures.
 
 @param type - Candidate callback type.
 
 @returns whether every runtime constituent is callable.
 
 @example
 ```ts
 typeDefinitelyCallable({ checker, type });
 ```
 */
export function typeDefinitelyCallable({
  checker,
  type,
}: {
  readonly checker: Checker;
  readonly type: Type;
}): boolean {
  if ((type.flags & TypeFlags.AnyOrUnknown) !== 0)
    return false;
  if (type.isUnionType()) {
    return type.getTypes()
      .every(function unionConstituentCallable(constituent,): boolean {
        return typeDefinitelyCallable({
          checker,
          type: constituent,
        },);
      },);
  }
  if (type.isTypeParameter()) {
    /**
     Constraint proving callable shape when present.
     */
    const constraint = checker.getBaseConstraintOfType(type,);
    return (constraint !== undefined)
      && typeDefinitelyCallable({
        checker,
        type: constraint,
      },);
  }
  /**
   Callable signatures exposed by nonunion candidate type.
   */
  const signatures = checker.getSignaturesOfType(
    type,
    SignatureKind.Call,
  );
  return signatures.length > 0;
}
