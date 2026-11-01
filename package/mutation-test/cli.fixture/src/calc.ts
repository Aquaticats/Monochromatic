/**
 Half-tested arithmetic helpers for mutation-test integration runs.
 
 `clampedSum` is fully tested, so its mutants must be killed;
 `describeSign`'s zero branch is deliberately untested, so conditional
 mutants there must survive.
 
 @example
 ```ts
 clampedSum({ a: 2, b: 3, max: 4 });
 // 4
 ```
 */

/**
 Adds two numbers, clamping the result to a maximum.
 
 @param options - Operands and inclusive maximum.
 
 @returns Clamped sum.
 
 @example
 ```ts
 clampedSum({ a: 1, b: 2, max: 10 });
 // 3
 ```
 */
export function clampedSum(options: {
  readonly a: number;
  readonly b: number;
  readonly max: number;
},): number {
  /**
   Unclamped sum of both operands.
   */
  const sum = options.a + options.b;
  return sum > options.max ? options.max : sum;
}

/**
 Describes a number's sign as text.
 
 Zero branch is deliberately untested so its mutants survive.
 
 @param value - Number under inspection.
 
 @returns Sign description.
 
 @example
 ```ts
 describeSign(-2);
 // 'negative'
 ```
 */
export function describeSign(value: number,): string {
  if (value < 0)
    return 'negative';

  if (value === 0)
    return 'zero';

  return 'positive';
}
