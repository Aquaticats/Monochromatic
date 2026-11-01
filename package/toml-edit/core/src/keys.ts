/**
 TOML key encoding helpers.
 
 @module
 */

import { escapeBasicSingleLine, } from './basic-escape.ts';

//region Key encoding

/**
 Tests whether `c` is permitted inside a TOML bare key (`[A-Za-z0-9_-]`).
 
 @param c - single character
 
 @returns whether `c` is a TOML bare-key character
 */
function isBareKeyChar(c: string,): boolean {
  return ((c >= 'a') && (c <= 'z'))
    || ((c >= 'A') && (c <= 'Z'))
    || ((c >= '0') && (c <= '9'))
    || (c === '_')
    || (c === '-');
}

/**
 Tests whether every char of `key` is a TOML bare-key char (and `key`
 is non-empty). Mirrors the shape of `/^[A-Za-z0-9_-]+$/` with a
 linear scan.
 
 @param key - candidate TOML key
 
 @returns whether `key` is a valid TOML bare key
 */
function isBareKey(key: string,): boolean {
  if (key.length
    === 0)
    return false;
  for (const char of key) {
    if (!isBareKeyChar(char,))
      return false;
  }
  return true;
}

/**
 Encode a key string. Bare when every char is `[A-Za-z0-9_-]`, basic-quoted otherwise.
 
 @param key - Raw key segment.
 
 @returns TOML key segment text.
 
 @example
 ```ts
 encodeKey({ key: 'tools', },);  // 'tools'
 encodeKey({ key: 'my key', },); // '"my key"'
 ```
 */
export function encodeKey({ key, }: { readonly key: string; },): string {
  if (isBareKey(key,))
    return key;
  // A quoted key is a basic string, so it must escape control scalars the same
  // way a basic-string value does; emitting a raw control character (newline,
  // NUL, backspace) here produces invalid TOML.
  return `"${escapeBasicSingleLine({ value: key, },)}"`;
}

//endregion Key encoding
