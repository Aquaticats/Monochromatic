/**
 Tuple and string type helpers behind the promisified result computation.
 
 They mirror upstream `pify`'s `index.d.ts` helpers (`LastArrayElement`,
 `DropLastArrayElement`, `StringEndsWith`) so migrating callers see the same
 result-type inference.
 
 Derived from [`pify`](https://github.com/sindresorhus/pify) by Sindre Sorhus
 (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution.
 
 @module
 */

//region Empty tuple

/* oxlint-disable no-restricted-syntax/no-optional-escape -- External-boundary mirror: upstream `pify` 6.1.0's `index.d.ts` pins single-function calls to `Options<[], [], MultiArgs, ErrorFirst>` and models the no-caller-argument fallback as `[]`; the empty tuple is a concrete zero-element selection in both roles, not encoded absence, and the pin is what steers single-function calls to the first `pify` overload. */
/**
 Concrete empty tuple: the zero-key selection pin and the zero-argument
 fallback upstream `pify`'s `index.d.ts` expresses as `[]`.
 
 @example
 ```ts
 type None = EmptyTuple;
 // => []
 ```
 */
export type EmptyTuple = [];
/* oxlint-enable no-restricted-syntax/no-optional-escape */

//endregion Empty tuple

//region Tuple helpers

/**
 Last element type of a tuple, or `never` for the empty tuple: the trailing
 callback slot of a wrapped function's parameter tuple.
 
 Mirrors upstream `pify`'s `LastArrayElement`.
 
 @typeParam Tuple - tuple to read the last element of
 
 @example
 ```ts
 type Callback = LastArrayElement<[string, (error: unknown) => void]>;
 // => (error: unknown) => void
 ```
 */
export type LastArrayElement<Tuple extends readonly unknown[]> = Tuple extends [
  ...unknown[],
  infer Last,
] ? Last
  : never;

/**
 Everything before a tuple's last element: the argument tuple callers pass
 once the trailing callback slot is dropped.
 
 Mirrors upstream `pify`'s `DropLastArrayElement`.
 
 @typeParam Tuple - tuple to drop the last element of
 
 @example
 ```ts
 type Args = DropLastArrayElement<[string, (error: unknown) => void]>;
 // => [string]
 ```
 */
export type DropLastArrayElement<Tuple extends readonly unknown[]> = Tuple extends [
  ...infer Prefix,
  unknown,
] ? Prefix
  : EmptyTuple;

//endregion Tuple helpers

//region String helpers

/**
 Whether a value ends with one of a union of suffixes: the membership test
 behind the default `Sync` / `Stream` exclusion.
 
 Mirrors upstream `pify`'s `StringEndsWith`, candidate left unconstrained on
 purpose: non-string member keys (symbols, numbers) must classify as `false`
 the way upstream's template-literal check classifies them.
 
 @typeParam Candidate - value to test
 
 @typeParam Suffixes - suffix union to test against
 
 @example
 ```ts
 StringEndsWith<'readFileSync', 'Sync' | 'Stream'>; // => true
 StringEndsWith<'readFile', 'Sync' | 'Stream'>; // => false
 ```
 */
export type StringEndsWith<
  Candidate,
  Suffixes extends string,
  > = Candidate extends `${infer _Prefix}${Suffixes}` ? true : false;

//endregion String helpers
