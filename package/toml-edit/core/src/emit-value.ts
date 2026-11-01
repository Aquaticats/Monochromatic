/**
 Re-emit an existing AST `TOMLContentNode` as TOML text.
 
 Preserves style and raw spelling: `TOMLStringValue.style` and `multiline`,
 `TOMLIntegerValue.number`, `TOMLFloatValue.number`, `TOMLDateTimeValue.datetime`.
 
 Used by splice and canonical emitters when the same AST node should be
 round-tripped without going through JS-value coercion.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { AST, } from 'toml-eslint-parser';

import { emitStringValue, } from './emit-value-string.ts';
import { TomlImmutableNodeError, } from './errors.ts';
import { encodeKey, } from './keys.ts';
import type { CanonicalOptions, } from './types.ts';

/**
 Emit `node` as TOML text per its parse-time fields.
 
 @returns Computed string.
 
 @example
 ```ts
 emitContentNode({ node: someTOMLValue, options: edit.canonical, },);
 ```
 */
export function emitContentNode(
  {
    node,
    options,
    depth = 0,
  }: {
    readonly node: ForeignBorrowed<AST.TOMLNode>;
    readonly options: CanonicalOptions;
    readonly depth?: number;
  },
): string {
  if (node.type
    === 'TOMLValue')
    return emitValueLeaf({ node, },);
  if (node.type
    === 'TOMLArray') {
    /**
     Per-element text so the assembler can join into inline or multi-line form.
     */
    const parts = node.elements
      .map(function each(el,) {
        return emitContentNode({
          node: el,
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
  return emitInlineTable({
    node,
    options,
    depth,
  },);
}

/**
 Emit a primitive leaf (`string` / `integer` / `float` / `boolean` / date kinds).
 
 @returns Computed string.
 */
function emitValueLeaf({ node, }: { readonly node: AST.TOMLValue; },): string {
  if (node.kind
    === 'string')
    return emitStringValue({ node, },);
  if ((node.kind
    === 'integer') || (node.kind
      === 'float'))
    return node.number;
  if (node.kind
    === 'boolean')
    return node.value ? 'true' : 'false';
  return node.datetime;
}

/**
 Shared array-text assembly for parsed and synthetic value rendering.
 
 @returns Computed string.
 
 @example
 ```ts
 assembleArrayParts({ parts: ['1', '2'], options, depth: 0, },); // '[ 1, 2, ]'
 ```
 */
export function assembleArrayParts(
  {
    parts,
    options,
    depth,
  }: {
    readonly parts: readonly string[];
    readonly options: CanonicalOptions;
    readonly depth: number;
  },
): string {
  /**
   Speculative inline form so the column budget check can decide the layout.
   */
  const inlineCandidate = `[ ${parts.join(', ',)}${parts.length
    === 0 ? '' : ', '}]`;
  if (
    (parts.length
      <= options
        .arrayInlineThreshold)
      && (inlineCandidate.length
        <= options
          .arrayInlineMaxColumns)
  ) {
    return inlineCandidate;
  }
  /**
   Indent for each element when the array goes multi-line.
   */
  const indent = ' '.repeat(options.indent
    * (depth + 1),);
  /**
   Closing bracket sits at the parent's indent level.
   */
  const closingIndent = ' '.repeat(options.indent
    * depth,);
  return `[\n${
    parts
      .map(function withIndent(p,) {
        return `${indent}${p},`;
      },)
      .join('\n',)
  }\n${closingIndent}]`;
}

/**
 Emit a `TOMLInlineTable` as `{ k = v, ... }`.
 
 @returns Computed string.
 
 @throws {@link TomlImmutableNodeError} when `node` is not a `TOMLInlineTable`.
 */
function emitInlineTable(
  {
    node,
    options,
    depth,
  }: {
    readonly node: AST.TOMLNode;
    readonly options: CanonicalOptions;
    readonly depth: number;
  },
): string {
  if (node.type
    !== 'TOMLInlineTable') {
    throw new TomlImmutableNodeError(
      `emitInlineTable: expected TOMLInlineTable, got ${node.type}`,
    );
  }
  /**
   Body entries rendered as `k = v` fragments for the assembler.
   */
  const parts = emitInlineTableBodyParts({
    body: node.body,
    options,
    depth,
  },);
  return assembleInlineTableParts({ parts, },);
}

/**
 Render each `TOMLKeyValue` in an inline-table body as `key = value` text.
 
 @returns Computed result (`readonly string[]`).
 */
function emitInlineTableBodyParts(
  {
    body,
    options,
    depth,
  }: {
    readonly body: readonly AST.TOMLKeyValue[];
    readonly options: CanonicalOptions;
    readonly depth: number;
  },
): readonly string[] {
  return body.map(function each(kv: AST.TOMLKeyValue,) {
    /**
     Encoded key chain joined with `.` so dotted keys reuse their original spelling.
     */
    const keyText = kv
      .key
      .keys
      .map(function each2(k: AST.TOMLBare | AST.TOMLQuoted,) {
        return encodeKey({ key: k.type
          === 'TOMLBare' ? k.name : k.value, },);
      },)
      .join('.',);
    /**
     Encoded value text so the entry can be composed as `k = v`.
     */
    const valueText = emitContentNode({
      node: kv.value,
      options,
      depth: depth + 1,
    },);
    return `${keyText} = ${valueText}`;
  },);
}

/**
 Wrap the rendered parts in `{ ... }` with the canonical comma layout.
 
 @returns Computed string.
 
 @example
 ```ts
 assembleInlineTableParts({ parts: ['a = 1'], },); // '{ a = 1, }'
 ```
 */
export function assembleInlineTableParts(
  {
    parts,
  }: {
    readonly parts: readonly string[];
  },
): string {
  return `{ ${parts.join(', ',)}${parts.length
    === 0 ? '' : ', '}}`;
}
