import type { StringJsonc, } from './brand.ts';
import { jsToNode, } from './edit-convert.ts';
import { parseJsonc, } from './parse-jsonc.ts';
import { emitJsoncValue, } from './stringify.ts';
import type { JsoncValue, } from './value.ts';

//region State

/**
 Immutable handle on a parsed JSONC document. Every edit function returns a
 fresh state; the underlying node tree is shared by reference where unchanged.
 */
export type JsoncEditState = {
  readonly root: JsoncValue;
};

//endregion State

//region Normalization

/**
 Expands a fast-path `plainJson` leaf into a structured node so the edit API can
 navigate and rebuild it uniformly. Structured nodes pass through unchanged.
 
 @param node - Node to normalize.
 
 @returns Structured node carrying the original top-level comment.
 
 @example
 ```ts
 normalizeNode({ node: { kind: 'plainJson', json: { a: 1 } } });
 // => { kind: 'record', entries: [...] }
 ```
 */
function normalizeNode({
  node,
}: {
  readonly node: JsoncValue;
},): JsoncValue {
  if (node.kind !== 'plainJson')
    return node;
  /**
   Structured expansion of the plain JSON value.
   */
  const expanded = jsToNode({ value: node.json, },);
  if (node.comment === undefined)
    return expanded;
  return {
    ...expanded,
    comment: node.comment,
  };
}

//endregion Normalization

//region Entry points

/**
 Parses a JSONC document into an editable state. The fast-path `plainJson` leaf
 is expanded to structured nodes so edits operate uniformly.
 
 @param source - Branded JSONC source string.
 
 @returns Editable state.
 
 @throws JsoncParseError on malformed input or a non-container top level.
 
 @example
 ```ts
 const state = parseJsoncEdit({ source: '{ "a": 1 } // n' as StringJsonc });
 ```
 */
export function parseJsoncEdit({
  source,
}: {
  readonly source: StringJsonc;
},): JsoncEditState {
  return {
    root: normalizeNode({ node: parseJsonc({ source, },), },),
  };
}

/**
 Serializes an edit state back to canonical JSONC text, preserving all
 comments. Canonical mode: formatting is normalized, comments and unedited
 scalar tokens are kept.
 
 @param state - Edit state to serialize.
 
 @returns Canonical JSONC text.
 
 @mutates state - `JSON.stringify` may invoke hooks on embedded plain JSON values.
 
 @example
 ```ts
 jsoncStringify({ state: parseJsoncEdit({ source: '{"a":1}' as StringJsonc }) });
 // => '{\n  "a": 1,\n}'
 ```
 */
export function jsoncStringify({
  state,
}: {
  state: JsoncEditState;
},): string {
  return emitJsoncValue({ value: state.root, },);
}

//endregion Entry points
