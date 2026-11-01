/**
 Wraps callback so only first positional argument reaches wrapped function.
 
 Useful when host APIs pass extra arguments that have different meanings for
 existing functions. `Array.prototype.map(Number.parseInt)` treats index as
 radix; `array.map(unary(Number.parseInt))` always calls `Number.parseInt`
 with just value.
 
 @param fn - callback that can run from first positional argument alone
 
 @returns wrapper forwarding only first positional argument
 
 @example
 ```ts
 import { unary, } from '\@monochromatic-dev/function-arity-core';
 
 const parsed = ['10', '10', '10'].map(unary(Number.parseInt,));
 // [10, 10, 10]
 ```
 */
export function unary<const TArgument, const TReturn,>(
  fn: (argument: TArgument,) => TReturn,
): (argument: TArgument,) => TReturn {
  return (function unaryWrapper(argument: TArgument,): TReturn {
    return fn(argument,);
  });
}
