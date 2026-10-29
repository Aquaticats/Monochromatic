import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { JsoncComment, } from './comment.ts';
import type { JsoncEditState, } from './edit-state.ts';
import {
  findNode,
  type JsoncPath,
  NODE_ABSENT,
} from './edit-navigate.ts';
import {
  JsoncPathNotFoundError,
  JsoncTypeError,
} from './errors.ts';
import type { JsoncValue, } from './value.ts';

//region Sentinel

/**
 Sentinel returned by the comment getters when the addressed node or key has no
 comment. A `Symbol` rather than `undefined` so absence is an explicit value,
 not a nullish union.
 */
export const COMMENT_ABSENT: unique symbol = Symbol(
  'jsonc-edit/the queried JSONC value or key carries no attached comment',
);

//endregion Sentinel

//region Generic transform

/**
 Rebuilds `node` with `transform` applied to the node the path addresses,
 descending the tree structurally. The target must exist.
 
 @param node - Current node being descended.
 
 @param path - Path to the node to transform.
 
 @param pathIndex - Index of the segment to resolve at this level.
 
 @param transform - Pure mapping applied to the addressed node.
 
 @returns Rebuilt node.
 
 @throws JsoncPathNotFoundError when a segment is missing.
 
 @throws JsoncTypeError when a segment cannot index the node's kind.
 
 @example
 ```ts
 transformAtPath({ node, path: ['a'], pathIndex: 0, transform: keepNode });
 ```
 */
function transformAtPath({
  node,
  path,
  pathIndex,
  transform,
}: {
  readonly node: JsoncValue;
  readonly path: JsoncPath;
  readonly pathIndex: number;
  readonly transform: (target: JsoncValue) => JsoncValue;
},): JsoncValue {
  if (pathIndex >= path.length)
    return transform(node,);
  /**
   Segment resolved at this level.
   */
  const segment = nonNullishOrThrow(path.at(pathIndex,),);
  if ((node.kind === 'record') && ((typeof segment) === 'string')) {
    /**
     Index of the last entry matching the key, or -1 when absent.
     */
    const matchIndex = node.entries
      .findLastIndex(function matchesKey(entry,): boolean {
        return entry.key
          .value
          === segment;
      },);
    if (matchIndex === (-1))
      throw new JsoncPathNotFoundError({ path, },);
    /**
     Matched entry retained while its value is rebuilt.
     */
    const entry = nonNullishOrThrow(node.entries[matchIndex],);
    return {
      ...node,
      entries: node.entries
        .with(
        matchIndex,
        {
          key: entry.key,
          value: transformAtPath({
            node: entry.value,
            path,
            pathIndex: pathIndex + 1,
            transform,
          },),
        },
        ),
    };
  }
  if ((node.kind === 'array') && ((typeof segment) === 'number')) {
    if (!Number.isInteger(segment,))
      throw new JsoncPathNotFoundError({ path, },);
    if ((segment < 0) || (segment
      >= node.elements
        .length))
      throw new JsoncPathNotFoundError({ path, },);
    /**
     Matched element rebuilt while siblings retain identity.
     */
    const element = nonNullishOrThrow(node.elements[segment],);
    return {
      ...node,
      elements: node.elements
        .with(
        segment,
        transformAtPath({
          node: element,
          path,
          pathIndex: pathIndex + 1,
          transform,
        },),
        ),
    };
  }
  throw new JsoncTypeError({
    message: `jsonc comment edit: cannot index ${node.kind} with ${JSON.stringify(segment,)}`,
  },);
}

//endregion Generic transform

//region Value comment

/**
 Reads the comment attached to the value at a path.
 
 @param state - Edit state.
 
 @param path - Path to the value.
 
 @returns The value's comment, or {@link COMMENT_ABSENT}.
 
 @throws JsoncPathNotFoundError when the path does not resolve.
 
 @example
 ```ts
 jsoncGetComment({ state, path: ['a'] });
 ```
 */
export function jsoncGetComment({
  state,
  path,
}: {
  readonly state: JsoncEditState;
  readonly path: JsoncPath;
},): JsoncComment | typeof COMMENT_ABSENT {
  /**
   Node addressed by the path.
   */
  const node = findNode({
    root: state.root,
    path,
  },);
  if (node === NODE_ABSENT)
    throw new JsoncPathNotFoundError({ path, },);
  return node.comment ?? COMMENT_ABSENT;
}

/**
 Sets the comment attached to the value at a path, returning a fresh state.
 
 @param state - Edit state.
 
 @param path - Path to the value.
 
 @param comment - Comment to attach.
 
 @returns Fresh state with the comment applied.
 
 @throws JsoncPathNotFoundError when the path does not resolve.
 
 @example
 ```ts
 jsoncSetComment({ state, path: ['a'], comment: { type: 'inline', text: ' n' } });
 ```
 */
export function jsoncSetComment({
  state,
  path,
  comment,
}: {
  readonly state: JsoncEditState;
  readonly path: JsoncPath;
  readonly comment: JsoncComment;
},): JsoncEditState {
  return {
    root: transformAtPath({
      node: state.root,
      path,
      pathIndex: 0,
      transform: function applyComment(target: JsoncValue,): JsoncValue {
        return {
          ...target,
          comment,
        };
      },
    },),
  };
}

//endregion Value comment

//region Key comment

/**
 Reads the comment attached to the record key at a path. The final segment must
 be a string key.
 
 @param state - Edit state.
 
 @param path - Path whose final segment is the key.
 
 @returns The key's comment, or {@link COMMENT_ABSENT}.
 
 @throws JsoncTypeError when the path is empty or its final segment is not a key.
 
 @throws JsoncPathNotFoundError when the key does not resolve.
 
 @example
 ```ts
 jsoncGetKeyComment({ state, path: ['a'] });
 ```
 */
export function jsoncGetKeyComment({
  state,
  path,
}: {
  readonly state: JsoncEditState;
  readonly path: JsoncPath;
},): JsoncComment | typeof COMMENT_ABSENT {
  /**
   Final segment, which must address a record key.
   */
  const key = lastKeySegment({ path, },);
  /**
   Parent record holding the keyed entry.
   */
  const parent = findNode({
    root: state.root,
    path: path.slice(
      0,
      -1,
    ),
  },);
  if ((parent === NODE_ABSENT) || (parent.kind !== 'record'))
    throw new JsoncPathNotFoundError({ path, },);
  /**
   Last entry matching the key.
   */
  const entry = parent.entries
    .findLast(function matchesKey(candidate,): boolean {
      return candidate.key
        .value
        === key;
    },);
  if (entry === undefined)
    throw new JsoncPathNotFoundError({ path, },);
  return entry.key
    .comment
    ?? COMMENT_ABSENT;
}

/**
 Sets the comment attached to the record key at a path, returning a fresh state.
 
 @param state - Edit state.
 
 @param path - Path whose final segment is the key.
 
 @param comment - Comment to attach to the key.
 
 @returns Fresh state with the key comment applied.
 
 @throws JsoncTypeError when the path is empty or its final segment is not a key.
 
 @throws JsoncPathNotFoundError when the key does not resolve.
 
 @example
 ```ts
 jsoncSetKeyComment({ state, path: ['a'], comment: { type: 'inline', text: ' k' } });
 ```
 */
export function jsoncSetKeyComment({
  state,
  path,
  comment,
}: {
  readonly state: JsoncEditState;
  readonly path: JsoncPath;
  readonly comment: JsoncComment;
},): JsoncEditState {
  /**
   Final segment, which must address a record key.
   */
  const key = lastKeySegment({ path, },);
  return {
    root: transformAtPath({
      node: state.root,
      path: path.slice(
        0,
        -1,
      ),
      pathIndex: 0,
      transform: function applyKeyComment(parent: JsoncValue,): JsoncValue {
        if (parent.kind !== 'record')
          throw new JsoncPathNotFoundError({ path, },);
        /**
         Index of the last entry matching the key.
         */
        const matchIndex = parent.entries
          .findLastIndex(function matchesKey(entry,): boolean {
            return entry.key
              .value
              === key;
          },);
        if (matchIndex === (-1))
          throw new JsoncPathNotFoundError({ path, },);
        /**
         Matched entry rebuilt while siblings retain identity.
         */
        const entry = nonNullishOrThrow(parent.entries[matchIndex],);
        return {
          ...parent,
          entries: parent.entries
            .with(
            matchIndex,
            {
              key: {
                ...entry.key,
                comment,
              },
              value: entry.value,
            },
            ),
        };
      },
    },),
  };
}

/**
 Extracts and validates the final path segment as a record key.
 
 @param path - Path whose final segment addresses a key.
 
 @returns The key string.
 
 @throws JsoncTypeError when the path is empty or its final segment is not a string.
 
 @example
 ```ts
 lastKeySegment({ path: ['a', 'b'] }); // => 'b'
 ```
 */
function lastKeySegment({
  path,
}: {
  readonly path: JsoncPath;
},): string {
  if (path.length === 0)
    throw new JsoncTypeError({ message: 'jsonc key comment: empty path has no key', },);
  /**
   Final path segment.
   */
  const key = nonNullishOrThrow(path.at(-1,),);
  if ((typeof key) !== 'string')
    throw new JsoncTypeError({ message: 'jsonc key comment: final segment is not a key', },);
  return key;
}

//endregion Key comment
