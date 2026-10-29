/**
 Path utilities.
 
 @module
 */

import type { TomlPath, } from './types.ts';

/**
 Decoded key segment fields needed by path readers, without parser ownership.

 @example
 ```ts
 const segment: TomlKeySegmentView = { type: 'TOMLBare', name: 'title', };
 ```
 */
type TomlKeySegmentView =
  | {
    readonly type: 'TOMLBare';
    readonly name: string;
  }
    | {
      readonly type: 'TOMLQuoted';
      readonly value: string;
    };

/**
 Dotted key segments viewed as read-only path data.

 @example
 ```ts
 const key: TomlKeyView = { keys: [{ type: 'TOMLBare', name: 'title', },], };
 ```
 */
type TomlKeyView = { readonly keys: readonly TomlKeySegmentView[]; };

/**
 Surface the string form of a key fragment.
 
 `TOMLBare` carries the identifier as `name`; `TOMLQuoted` carries the
 decoded string as `value`. Both are surfaced as the same string here so
 path comparisons treat `key = 1` and `"key" = 1` identically.
 
 @returns Computed string.
 
 @example
 ```ts
 keyNameOf({ key: { type: 'TOMLBare', name: 'foo' } as never, },); // 'foo'
 ```
 */
export function keyNameOf({ key, }: { readonly key: TomlKeySegmentView; },): string {
  return key.type
    === 'TOMLBare' ? key.name : key.value;
}

/**
 The full list of segments that a `TOMLKey` spells.
 
 Dotted-key forms like `a.b.c = 1` produce three segments.
 
 @returns Computed result (`readonly string[]`).
 
 @example
 ```ts
 keysOf({ key: tomlKeyForABC, },); // ['a', 'b', 'c']
 ```
 */
export function keysOf({ key, }: { readonly key: TomlKeyView; },): readonly string[] {
  return key.keys
    .map(function nameOf(k: TomlKeySegmentView,) {
      return keyNameOf({ key: k, },);
    },);
}

/**
 Render a `TomlPath` as a human-readable string for error messages.
 
 @returns Computed string.
 
 @example
 ```ts
 formatPath({ path: ['fruits', 0, 'name'] as const, },); // 'fruits[0].name'
 ```
 */
export function formatPath({ path, }: { readonly path: TomlPath; },): string {
  return path
    .map(function fmt(
      seg,
      i,
    ) {
      if ((typeof seg) === 'number')
        return `[${String(seg,)}]`;
      if (i === 0)
        return seg;
      return `.${seg}`;
    },)
    .join('',);
}
