import { getProperty as dotPropGet, } from 'dot-prop';
import type { Path, } from '../types.ts';

/**
 Removes duplicate lines from a string, preserving first occurrence order.
 
 @param content - Multiline string to deduplicate
 
 @returns Content with duplicate lines removed
 
 @example
 ```ts
 dedup('a\nb\na\nc');
 // 'a\nb\nc'
 ```
 */
export function dedup(content: string,): string {
  return [...new Set(content.split('\n',),),].join('\n',);
}

/**
 Extracts a nested property from JSON content using a structured path.
 
 Backed by {@link dotPropGet} (`dot-prop`'s `getProperty`), called with array
 segments (no dot-string translation), which means keys containing literal
 dots resolve correctly: `['a.b',]` selects key `"a.b"`, not nested `a` then `b`.
 
 Return type is `string` for the common path; on missing path the underlying
 `JSON.stringify(undefined)` returns `undefined`, so callers can still rely on
 `=== undefined` for missing-value checks.
 
 @param path - Sequence of key segments (and numeric array indices)
 
 @param content - JSON string to parse and extract from
 
 @returns Extracted value as string (`JSON.stringify(value, null, 2)` when not already a string), or `undefined` when missing
 
 @example
 ```ts
 getJsonProperty({ path: ['rules',], content: '{"rules":{"no-var":"error"}}', },);
 // '{\n  "no-var": "error"\n}'
 ```
 */
export function getJsonProperty(
  {
    path,
    content,
  }: {
    readonly path: Path;
    readonly content: string;
  },
): string {
  /**
   Parsed JSON value
   */
  const parsed: unknown = JSON.parse(content,);
  /**
   Extracted value at the array-path; dot-prop v10 accepts arrays natively
   */
  const extracted: unknown = dotPropGet(
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- JSON.parse returns unknown, dot-prop requires Record
    parsed as Record<string, unknown>,
    path,
  );
  return ((typeof extracted) === 'string') ? extracted : JSON.stringify(
    extracted,
    null,
    2,
  );
}
