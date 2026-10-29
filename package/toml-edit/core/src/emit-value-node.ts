/**
 Render a {@link ValueNode} to TOML text.
 
 A clean child re-emits through {@link emitContentNode} so its original raw
 spelling (hex integers, quote style, datetime text) survives even when a
 sibling changed. A synthetic child renders canonically from its structure,
 reusing the shared array/inline-table assemblers so layout matches the
 from-scratch encoders.
 
 @module
 */

import type { ValueNode, } from './document.ts';
import {
  assembleArrayParts,
  assembleInlineTableParts,
  emitContentNode,
} from './emit-value.ts';
import { encodeKey, } from './keys.ts';
import type { CanonicalOptions, } from './types.ts';
import { jsValueToTomlText, } from './values.ts';

/**
 Render `value` as TOML text at nesting `depth`.
 
 @returns Computed string.
 
 @example
 ```ts
 renderValueNode({ value: kv.value, options: edit.canonical, depth: 0, },);
 ```
 */
export function renderValueNode(
  {
    value,
    options,
    depth,
  }: {
    readonly value: ValueNode;
    readonly options: CanonicalOptions;
    readonly depth: number;
  },
): string {
  if (value.origin
    .kind
    === 'clean') {
    return emitContentNode({
      node: value.origin
        .astNode,
      options,
      depth,
    },);
  }
  if (value.kind
    === 'scalar') {
    return value.renderText
      ?? jsValueToTomlText({
        input: value.jsValue,
        options,
      },);
  }
  if (value.kind
    === 'array') {
    /**
     Per-element text so the assembler can choose inline or multi-line layout.
     */
    const parts = value.elements
      .map(function each(el,) {
        return renderValueNode({
          value: el,
          options,
          depth: depth + 1,
        },);
      },);
    return assembleArrayParts({
      parts,
      options,
      depth,
    },);
  }
  /**
   Per-entry `key = value` fragments for the inline-table assembler.
   */
  const parts = value.entries
    .map(function each(entry,) {
    /**
     Encoded dotted key so each segment reuses canonical key spelling.
     */
      const keyText = entry.keySegments
        .map(function eachSeg(seg,) {
          return encodeKey({ key: seg, },);
        },)
        .join('.',);
      return `${keyText} = ${
      renderValueNode({
        value: entry.value,
        options,
        depth: depth + 1,
      },)
    }`;
    },);
  return assembleInlineTableParts({ parts, },);
}
