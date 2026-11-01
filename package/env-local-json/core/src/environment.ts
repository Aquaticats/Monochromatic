/**
 Environment inspection helpers shared by the reader and writer.

 @module
 */

import type { EnvLocalJsonEnvironment, } from './types.ts';

//region Values

/**
 Whether a value is a non-blank string.

 @param value - candidate value of unknown type

 @returns `true` when value is a string with non-whitespace content

 @example
 ```ts
 isNonBlank('x'); // true
 isNonBlank('   '); // false
 ```
 */
export function isNonBlank(value: unknown,): value is string {
  return ((typeof value) === 'string') && (value.trim() !== '');
}

/**
 Whether every requested key already has a non-blank environment value.

 @param keys - requested environment variable names

 @param env - environment to inspect

 @returns `true` when the store can be skipped entirely

 @example
 ```ts
 hasEveryRequestedKey({ keys: ['A'], env: { A: 'x' } }); // true
 ```
 */
export function hasEveryRequestedKey({
  keys,
  env,
}: {
  readonly keys: readonly string[];
  readonly env: EnvLocalJsonEnvironment;
},): boolean {
  return keys.every(function isPresent(key: string,): boolean {
    return isNonBlank(env[key],);
  },);
}

/**
 Selects requested keys worth returning from the store: those absent or blank
 in the environment and present in the decrypted store.

 @param keys - requested environment variable names

 @param env - environment whose values take precedence

 @param store - validated decrypted store values

 @returns fill-in values keyed by requested name

 @example
 ```ts
 selectFillInValues({ keys: ['A'], env: {}, store: { A: 'x' } }); // { A: 'x' }
 ```
 */
export function selectFillInValues({
  keys,
  env,
  store,
}: {
  readonly keys: readonly string[];
  readonly env: EnvLocalJsonEnvironment;
  readonly store: Readonly<Record<string, string>>;
},): Readonly<Record<string, string>> {
  /**
   Fill-in values being collected.
   */
  const values: Record<string, string> = {};
  for (const key of keys) {
    if (isNonBlank(env[key],))
      continue;
    if (!Object.hasOwn(
      store,
      key,
    ))
      continue;
    /**
     Store value for this requested key.
     */
    const value = store[key];
    if (isNonBlank(value,))
      values[key] = value;
  }
  return values;
}

//endregion Values
