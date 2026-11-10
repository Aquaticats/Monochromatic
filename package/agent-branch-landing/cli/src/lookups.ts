/**
 An answer that may be absent, as a shape and never as a nullish union: either the value is there, or the caller is told it is not.

 @module
 */

/**
 Something looked for, found or not.
 */
export type Looked<Value> =
  | {
    /**
     Whether the search found what it looked for.
     */
    readonly found: true;

    /**
     What it found.
     */
    readonly value: Value;
  }
    | {
    /**
     Whether the search found what it looked for, which it did not.
     */
      readonly found: false;

    /**
     Why nothing was found, in the caller's words, which a report may name.
     */
      readonly why: string;
    };

/**
 Something that was found.

 @param value - what was found

 @returns the found shape

 @example
 ```ts
 const answer = found(41,);
 ```
 */
export function found<Value>(value: Value,): Looked<Value> {
  return {
    found: true,
    value,
  };
}

/**
 Something that was not found, and why.

 @param why - what was looked for and why it is not there

 @returns the absent shape

 @example
 ```ts
 const answer = missing('no option named --branch was given',);
 ```
 */
export function missing(why: string,): {
  readonly found: false;
  readonly why: string;
} {
  return {
    found: false,
    why,
  };
}
