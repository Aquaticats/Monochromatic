/**
 Build {@link ValueNode}s from parsed `toml-eslint-parser` content nodes.
 
 Every built node is `clean`: it retains the parse-time AST node and its
 source range so a fully-clean document emits verbatim. Inline-table entries
 are built as {@link KeyValueNode}s without line structure (no comments, no
 trailing newline), since they live inside `{ }`.
 
 @module
 */

import {
  type AST,
  getStaticTOMLValue,
} from 'toml-eslint-parser';

import type {
  KeyValueNode,
  ScalarKind,
  ValueNode,
} from './document.ts';
import { keysOf, } from './path.ts';

/**
 Build a {@link ValueNode} from a parsed content node.
 
 @returns Computed {@link ValueNode}.
 
 @mutates node - `getStaticTOMLValue` can invoke caller-owned AST hooks while reading scalar value
 
 @example
 ```ts
 buildValue({ node: keyValue.value, },);
 ```
 */
export function buildValue(
  { node, }: { readonly node: AST.TOMLContentNode; },
): ValueNode {
  if (node.type
    === 'TOMLArray') {
    return {
      kind: 'array',
      elements: node.elements
        .map(
        /**
         Builds one array element recursively.
         
         @param el - Parser-owned TOML content node.
         
         @returns Built value node.
         
         @mutates el - Recursive build can invoke caller-owned AST hooks through `getStaticTOMLValue`.
         */
        function each(el: AST.TOMLContentNode,) {
          return buildValue({ node: el, },);
        },),
      origin: {
        kind: 'clean',
        range: node.range,
        astNode: node,
      },
    };
  }
  if (node.type
    === 'TOMLInlineTable') {
    return {
      kind: 'inline-table',
      entries: node.body
        .map(function each(kv: AST.TOMLKeyValue,) {
          return buildInlineKeyValue({ kv, },);
        },),
      origin: {
        kind: 'clean',
        range: node.range,
        astNode: node,
      },
    };
  }
  return {
    kind: 'scalar',
    tomlKind: node.kind as ScalarKind,
    jsValue: getStaticTOMLValue(node,),
    origin: {
      kind: 'clean',
      range: node.range,
      astNode: node,
    },
  };
}

/**
 Build a {@link KeyValueNode} for an inline-table entry (no line, no comments).
 
 @returns Computed {@link KeyValueNode}.
 
 @mutates kv - Recursive value build can invoke caller-owned AST hooks through `getStaticTOMLValue`.
 
 @example
 ```ts
 buildInlineKeyValue({ kv: inlineTable.body[0], },);
 ```
 */
export function buildInlineKeyValue(
  { kv, }: { readonly kv: AST.TOMLKeyValue; },
): KeyValueNode {
  return {
    kind: 'keyvalue',
    keySegments: keysOf({ key: kv.key, },),
    value: buildValue({ node: kv.value, },),
    origin: {
      kind: 'clean',
      range: kv.range,
      astNode: kv,
    },
    valueRange: kv.value
      .range,
    commentsBefore: [],
  };
}
