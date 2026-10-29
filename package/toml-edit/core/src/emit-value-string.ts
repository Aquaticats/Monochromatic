/**
 String leaf emission helpers for parsed TOML values.
 
 Basic-string escaping lives in the shared `basic-escape.ts` so this emitter
 and the from-scratch value and key encoders cannot drift apart; here it is
 applied per the parsed node's style and `multiline` flag.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { AST, } from 'toml-eslint-parser';

import {
  escapeBasicMultiline,
  escapeBasicSingleLine,
} from './basic-escape.ts';

//region String emission

/**
 Emit a `TOMLStringValue` per its style and `multiline` flag.
 
 @param node - Parsed TOML string value.
 
 @returns Computed string.
 
 @example
 ```ts
 emitStringValue({ node: tomlStringNode, },);
 ```
 */
export function emitStringValue({
  node,
}: {
  readonly node: ForeignBorrowed<AST.TOMLStringValue>;
}): string {
  if (node.style === 'literal') {
    if (node.multiline)
      return `'''${node.value}'''`;
    return `'${node.value}'`;
  }
  if (node.multiline)
    return `"""${escapeBasicMultiline({ value: node.value, },)}"""`;
  return `"${escapeBasicSingleLine({ value: node.value, },)}"`;
}

//endregion String emission
