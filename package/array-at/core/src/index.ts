/**
 Throwing array accessor with static and runtime diagnostics.
 
 Publication remains blocked while `package.json` has `private: true`.
 See package README for clearance requirements.
 
 @example
 ```ts
 import { arrayAt, } from '@monochromatic-dev/array-at-core';
 
 const last = arrayAt({ array: [10, 20, 30], index: -1, });
 ```
 
 @packageDocumentation
 */

export { arrayAt, } from './array-at.ts';
export { ArrayAtError, } from './array-at-error.ts';
export type { ArrayAtDiagnostic, } from './diagnostic-types.ts';
export type { ArrayAtDiagnostics, } from './array-at-types.ts';
export {
  asSafeInteger,
  assertSafeInteger,
  isSafeInteger,
} from './safe-integer.ts';
export type { SafeInteger, } from './safe-integer.ts';
