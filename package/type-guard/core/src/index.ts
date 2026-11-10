/**
 Runtime type guards for values that arrive as `unknown`.

 Every guard here answers one question the type system cannot: whether a value
 read from JSON, a network response or a caught throw matches a shape the
 caller may then use. `@sindresorhus/is` supplies the primitive predicates;
 this package owns only the guards the library does not hold, so the shapes
 this repository narrows on stay one spelling each instead of one per module.

 The folder's files do not depend on each other: `is-record.ts` and
 `is-errno-exception.ts` each import only from `@sindresorhus/is`, and this
 module re-exports both.

 @module
 */

export { isErrnoException, } from './is-errno-exception.ts';
export { isRecord, } from './is-record.ts';
