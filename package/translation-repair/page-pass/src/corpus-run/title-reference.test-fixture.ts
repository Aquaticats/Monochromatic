/**
 Slices and page rows the title reference tests build their pages from.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import type { SliceReplacement, } from '../../dist/final/node/index.mjs';

/**
 One slice.

 @param sliceIndex - where the slice stands

 @param source - original text

 @param target - archive text, empty where the archive never translated it

 @returns Prepared pair

 @example
 ```ts
 const slice = pair({ sliceIndex: 0, source: '### 窗边猫', target: '', },);
 ```
 */
export function pair(
  {
    sliceIndex,
    source,
    target,
  }: {
    readonly sliceIndex: number;
    readonly source: string;
    readonly target: string;
  },
): ChunkPair {
  return {
    source: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: source.length,
      text: source,
    },
    target: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: target.length,
      text: target,
    },
  };
}

/**
 Text of every row, in order.

 @param rows - rows a pass returned

 @returns Each row's replacement text, to compare whole pages at once

 @example
 ```ts
 const texts = textsOf({ rows: unified.replacements, },);
 ```
 */
export function textsOf({ rows, }: { readonly rows: readonly SliceReplacement[]; },): readonly string[] {
  return rows.map(function textOf(row,): string {
    return row.replacementText;
  },);
}
