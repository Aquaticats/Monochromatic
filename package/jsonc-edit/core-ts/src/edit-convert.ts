import type {
  JsonValue,
  ReadonlyDeep,
} from 'type-fest';
import type { JsoncValue, } from './value.ts';

//region JS value to node

/**
 Converts a plain JSON value into a structured node tree, with no `raw` tokens
 and no comments. Used to expand the fast-path `plainJson` leaf for editing and
 to build nodes for values set through the edit API.
 
 @param value - Plain JSON value.
 
 @returns Equivalent structured node.
 
 @example
 ```ts
 jsToNode({ value: { a: 1 } });
 // => { kind: 'record', entries: [{ key: { value: 'a' }, value: { kind: 'number', value: 1 } }] }
 ```
 */
export function jsToNode({
  value,
}: {
  readonly value: ReadonlyDeep<JsonValue>;
},): JsoncValue {
  if (value === null)
    return { kind: 'null', };
  if ((typeof value) === 'boolean')
    return {
      kind: 'boolean',
      value,
    };
  if ((typeof value) === 'number')
    return {
      kind: 'number',
      value,
    };
  if ((typeof value) === 'string')
    return {
      kind: 'string',
      value,
    };
  if (Array.isArray(value,))
    return {
      kind: 'array',
      elements: value.map(function elementToNode(
        element: ReadonlyDeep<JsonValue>,
      ): JsoncValue {
        return jsToNode({ value: element, },);
      },),
    };
  return {
    kind: 'record',
    entries: Object.entries(value,)
      .map(function pairToEntry([key, child]: readonly [
        string,
        JsonValue
      ],) {
        return {
          key: { value: key, },
          value: jsToNode({ value: child, },),
        };
      },),
  };
}

//endregion JS value to node

//region Node to JS value

/**
 Converts a structured node back to a plain JSON value, discarding comments and
 raw tokens. Duplicate record keys are malformed input, so last-wins applies
 with no preservation guarantee.
 
 @param node - Node to flatten.
 
 @returns Plain JSON value.
 
 @example
 ```ts
 toJsValue({ node: { kind: 'number', value: 1 } }); // => 1
 ```
 */
export function toJsValue({
  node,
}: {
  readonly node: JsoncValue;
},): JsonValue {
  if (node.kind === 'string')
    return node.value;
  if (node.kind === 'number')
    return node.value;
  if (node.kind === 'boolean')
    return node.value;
  if (node.kind === 'null')
    return null;
  if (node.kind === 'plainJson')
    return node.json as JsonValue;
  if (node.kind === 'array')
    return node.elements
      .map(function elementToJs(element: JsoncValue,): JsonValue {
        return toJsValue({ node: element, },);
      },);
  return Object.fromEntries(
    node.entries
      .map(function entryToPair(entry,): readonly [
        string,
        JsonValue
      ] {
        return [
          entry.key
            .value,
          toJsValue({ node: entry.value, },),
        ];
      },),
  );
}

//endregion Node to JS value
