/**
 Wraps callback so only first two positional arguments reach wrapped function.
 
 Useful when host APIs pass value, index, and collection but callback logic
 must be protected from collection argument. `Array.prototype.map` remains a
 natural consumer because mapped value and index stay available while source
 array does not leak into callback.
 
 @param fn - callback that can run from first two positional arguments alone
 
 @returns wrapper forwarding only first two positional arguments
 
 @example
 ```ts
 import { binary, } from '\@monochromatic-dev/function-arity-core';
 
 const rendered = ['a', 'b'].map(binary(function render(
   value: string,
   index: number,
 ): string {
   return `${index}:${value}`;
 },),);
 // ['0:a', '1:b']
 ```
 */
export function binary<const TFirstArgument, const TSecondArgument, const TReturn,>(
  fn: (
    firstArgument: TFirstArgument,
    secondArgument: TSecondArgument,
  ) => TReturn,
): (
  firstArgument: TFirstArgument,
  secondArgument: TSecondArgument,
) => TReturn {
  return (function binaryWrapper(
    firstArgument: TFirstArgument,
    secondArgument: TSecondArgument,
  ): TReturn {
    return fn(
      firstArgument,
      secondArgument,
    );
  });
}
