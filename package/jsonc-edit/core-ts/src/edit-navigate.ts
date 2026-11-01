import type { JsonValue, } from 'type-fest';
import { toJsValue, } from './edit-convert.ts';
import type { JsoncEditState, } from './edit-state.ts';
import {
  JsoncPathNotFoundError,
  JsoncTypeError,
} from './errors.ts';
import type { JsoncValue, } from './value.ts';

//region Path

/**
 Address into a JSONC document: string segments index record keys, number
 segments index array elements.
 
 @example
 ```ts
 const path: JsoncPath = ['server', 'ports', 0];
 ```
 */
export type JsoncPath = readonly (string | number)[];

/**
 Sentinel returned by {@link findNode} when a path does not resolve. A `Symbol`
 rather than `undefined` so absence is an explicit value, not a nullish union.
 */
export const NODE_ABSENT: unique symbol = Symbol('jsonc-edit/node absent at path',);

//endregion Path

//region Lookup

/**
 Resolves one path segment against a node, returning the child node or
 {@link NODE_ABSENT}. Record keys match by string segment (last occurrence
 wins, since duplicates are undefined behavior); array elements match by number
 segment.
 
 @param node - Node to descend into.
 
 @param segment - Path segment.
 
 @returns Child node, or the absence sentinel.
 
 @example
 ```ts
 childAt({ node: arrayNode, segment: 0 });
 ```
 */
function childAt({
  node,
  segment,
}: {
  readonly node: JsoncValue;
  readonly segment: string | number;
},): JsoncValue | typeof NODE_ABSENT {
  if ((node.kind === 'record') && ((typeof segment) === 'string')) {
    /**
     Last entry whose key matches the segment.
     */
    const entry = node.entries
      .findLast(function matchesKey(candidate,): boolean {
        return candidate.key
          .value
          === segment;
      },);
    return (entry === undefined)
      ? NODE_ABSENT
      : entry.value;
  }
  if ((node.kind === 'array') && ((typeof segment) === 'number')) {
    /**
     Element at the segment index.
     */
    const element = node.elements
      .at(segment,);
    return element ?? NODE_ABSENT;
  }
  return NODE_ABSENT;
}

/**
 Walks a path from the root, returning the node it addresses or
 {@link NODE_ABSENT}.
 
 @param root - Document root node.
 
 @param path - Path to resolve.
 
 @returns Addressed node, or the absence sentinel.
 
 @example
 ```ts
 findNode({ root, path: ['a', 0] });
 ```
 */
export function findNode({
  root,
  path,
}: {
  readonly root: JsoncValue;
  readonly path: JsoncPath;
},): JsoncValue | typeof NODE_ABSENT {
  return path.reduce<JsoncValue | typeof NODE_ABSENT>(
    function step(
      current,
      segment,
    ): JsoncValue | typeof NODE_ABSENT {
      return (current === NODE_ABSENT)
        ? NODE_ABSENT
        : childAt({
          node: current,
          segment,
        },);
    },
    root,
  );
}

//endregion Lookup

//region Read API

/**
 Reads the plain JSON value at a path. Comments and raw tokens are discarded.
 
 @param state - Edit state.
 
 @param path - Path to read.
 
 @returns Plain JSON value at the path.
 
 @throws JsoncPathNotFoundError when the path does not resolve.
 
 @example
 ```ts
 jsoncGetValue({ state, path: ['a'] }); // => 1
 ```
 */
export function jsoncGetValue({
  state,
  path,
}: {
  readonly state: JsoncEditState;
  readonly path: JsoncPath;
},): JsonValue {
  /**
   Node addressed by the path.
   */
  const node = findNode({
    root: state.root,
    path,
  },);
  if (node === NODE_ABSENT)
    throw new JsoncPathNotFoundError({ path, },);
  return toJsValue({ node, },);
}

/**
 Tests whether a path resolves to a node.
 
 @param state - Edit state.
 
 @param path - Path to test.
 
 @returns `true` when the path resolves.
 
 @example
 ```ts
 jsoncHas({ state, path: ['a'] }); // => true
 ```
 */
export function jsoncHas({
  state,
  path,
}: {
  readonly state: JsoncEditState;
  readonly path: JsoncPath;
},): boolean {
  return findNode({
    root: state.root,
    path,
  },) !== NODE_ABSENT;
}

/**
 Returns the keys of the record at a path, in document order.
 
 @param state - Edit state.
 
 @param path - Path to a record.
 
 @returns Key strings.
 
 @throws JsoncPathNotFoundError when the path does not resolve.
 
 @throws JsoncTypeError when the target is not a record.
 
 @example
 ```ts
 jsoncKeys({ state, path: [] }); // => ['a', 'b']
 ```
 */
export function jsoncKeys({
  state,
  path,
}: {
  readonly state: JsoncEditState;
  readonly path: JsoncPath;
},): readonly string[] {
  /**
   Node addressed by the path.
   */
  const node = findNode({
    root: state.root,
    path,
  },);
  if (node === NODE_ABSENT)
    throw new JsoncPathNotFoundError({ path, },);
  if (node.kind !== 'record')
    throw new JsoncTypeError({ message: 'jsoncKeys: target is not an object', },);
  return node.entries
    .map(function keyOf(entry,): string {
      return entry.key
        .value;
    },);
}

//endregion Read API
