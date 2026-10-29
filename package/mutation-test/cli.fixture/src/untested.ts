/**
 File with no tests at all: every mutant here must short-circuit to
 survived without any container run.
 
 @example
 ```ts
 doubled(2);
 // 4
 ```
 */

/**
 Doubles a number.
 
 @param value - Number to double.
 
 @returns Twice the value.
 
 @example
 ```ts
 doubled(3);
 // 6
 ```
 */
export function doubled(value: number,): number {
  return value * 2;
}
