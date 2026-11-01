/**
 Direct callable and constructable capability classification.
 
 @module
 */

import {
  type Checker,
  SignatureKind,
  type Type,
} from 'typescript/sync';

/**
 Classification for direct callable and constructable inputs.
 */
export const CALLABLE_CAPABILITY = {
  kind: 'opaque-capability',
  reason: 'callable or constructable input can execute caller-defined behavior',
} as const;

/**
 Tests whether type directly exposes call or construct signatures.
 
 @param checker - TypeScript checker resolving signatures.
 
 @param type - Semantic type under classification.
 
 @returns whether type carries direct callable capability.
 
 @example
 ```ts
 typeHasCallableCapability({ checker, type });
 ```
 */
export function typeHasCallableCapability({
  checker,
  type,
}: {
  readonly checker: Checker;
  readonly type: Type;
}): boolean {
  /**
   Call signatures directly exposed by type.
   */
  const callSignatures = checker.getSignaturesOfType(
    type,
    SignatureKind.Call,
  );
  if (callSignatures.length > 0)
    return true;
  /**
   Construct signatures directly exposed by type.
   */
  const constructSignatures = checker.getSignaturesOfType(
    type,
    SignatureKind.Construct,
  );
  return constructSignatures.length > 0;
}
