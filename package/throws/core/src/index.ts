/**
 Expression-position throwing helper.
 
 Use {@link throws}`(error,)` when JavaScript syntax requires an expression but the
 domain model requires throwing a prebuilt `Error`. Prefer statement `throw`
 in ordinary control flow, and prefer `module-or-throw` validators when the
 desired operation is checking a value and returning that same value.
 
 @example
 ```ts
 import { throws, } from '@monochromatic-dev/throws-core';
 
 const token = maybeToken ?? throws(new MissingTokenError(),);
 ```
 
 @packageDocumentation
 */

export { throws, } from './throws.ts';
