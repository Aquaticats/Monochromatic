/**
 Outcome of looking for a value that may not be there: the value, or the reason it was not found.

 The release helpers model absence this way, never with `undefined` or an empty text, so a caller must check `found`
 before it can reach the value and every miss carries text that explains it.
 */
export type Lookup<Value> =
  | {
    readonly found: true;
    readonly value: Value;
  }
    | {
      readonly found: false;
    /**
     Why there is no value, written for a person reading a diagnostic.
     */
      readonly reason: string;
    };

/**
 Wrap a value that was found.

 @param value - the found value

 @returns a successful lookup

 @example
 ```ts
 foundValue('abc');
 ```
 */
export function foundValue<Value>(value: Value): Lookup<Value> {
  return {
    found: true,
    value
  };
}

/**
 Report that nothing was found.

 @param reason - why there is no value

 @returns a failed lookup that carries the reason

 @example
 ```ts
 missingValue('no such run');
 ```
 */
export function missingValue(reason: string): Lookup<never> {
  return {
    found: false,
    reason
  };
}
