/**
 TypeScript declaration readonly-modifier query.
 
 @module
 */

import { ModifierFlags, } from 'typescript/sync';

/**
 Reads declaration modifier flags without assuming every node supports modifiers.
 
 @param value - TypeScript declaration node.
 
 @returns whether declaration carries readonly modifier.
 
 @example
 ```ts
 declarationIsReadonly(declaration);
 ```
 */
export function declarationIsReadonly(value: object,): boolean {
  if (!('modifierFlags' in value))
    return false;
  /**
   Runtime-narrowed modifier flags on declaration node.
   */
  const { modifierFlags, } = value;
  return ((typeof modifierFlags) === 'number')
    && ((modifierFlags & ModifierFlags.Readonly) !== 0);
}
