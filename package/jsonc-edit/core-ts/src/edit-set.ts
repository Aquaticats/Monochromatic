import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type {
  JsonValue,
  ReadonlyDeep,
} from 'type-fest';
import { jsToNode, } from './edit-convert.ts';
import type { JsoncPath, } from './edit-navigate.ts';
import type { JsoncEditState, } from './edit-state.ts';
import {
  JsoncPathNotFoundError,
  JsoncTypeError,
} from './errors.ts';
import type { JsoncValue, } from './value.ts';

//region Set

/**
 Rebuilds `node` with the value at the remaining path replaced by `newNode`,
 descending the tree structurally one segment per level. A missing final key is
 appended; a missing final array index equal to the length appends; any other
 gap throws. The replaced value keeps its existing comment.
 
 @param node - Current node being descended.
 
 @param path - Full path being set.
 
 @param pathIndex - Index of the segment to resolve at this level.
 
 @param newNode - Replacement node for the path target.
 
 @returns Rebuilt node.
 
 @throws JsoncPathNotFoundError when an intermediate segment is missing.
 
 @throws JsoncTypeError when a segment cannot index the node's kind.
 
 @example
 ```ts
 setAtPath({ node, path: ['a'], pathIndex: 0, newNode });
 ```
 */
function setAtPath({
  node,
  path,
  pathIndex,
  newNode,
}: {
  readonly node: JsoncValue;
  readonly path: JsoncPath;
  readonly pathIndex: number;
  readonly newNode: JsoncValue;
},): JsoncValue {
  if (pathIndex >= path.length)
    return (node.comment === undefined)
      ? newNode
      : {
        ...newNode,
        comment: node.comment,
      };
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
    if (matchIndex !== (-1)) {
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
            value: setAtPath({
              node: entry.value,
              path,
              pathIndex: pathIndex + 1,
              newNode,
            },),
          },
          ),
      };
    }
    if (pathIndex === (path.length - 1))
      return {
        ...node,
        entries: [
          ...node.entries,
          {
            key: { value: segment, },
            value: newNode,
          },
        ],
      };
    throw new JsoncPathNotFoundError({ path, },);
  }
  if ((node.kind === 'array') && ((typeof segment) === 'number')) {
    if (!Number.isInteger(segment,))
      throw new JsoncPathNotFoundError({ path, },);
    if ((segment >= 0) && (segment
      < node.elements
        .length)) {
      /**
       Matched element rebuilt while siblings retain identity.
       */
      const element = nonNullishOrThrow(node.elements[segment],);
      return {
        ...node,
        elements: node.elements
          .with(
          segment,
          setAtPath({
            node: element,
            path,
            pathIndex: pathIndex + 1,
            newNode,
          },),
          ),
      };
    }
    if ((segment
      === node.elements
        .length) && (pathIndex === (path.length - 1)))
      return {
        ...node,
        elements: [
          ...node.elements,
          newNode,
        ],
      };
    throw new JsoncPathNotFoundError({ path, },);
  }
  throw new JsoncTypeError({
    message: `jsoncSet: cannot index ${node.kind} with ${JSON.stringify(segment,)}`,
  },);
}

/**
 Sets the value at a path, returning a fresh state. Missing trailing keys (or an
 array index equal to the length) are created; missing intermediate segments
 throw. The target's comment is preserved.
 
 @param state - Edit state.
 
 @param path - Path to set; empty replaces the whole document.
 
 @param value - Plain JSON value to set.
 
 @returns Fresh state with the value applied.
 
 @throws JsoncPathNotFoundError when an intermediate segment is missing.
 
 @throws JsoncTypeError when a segment cannot index a node.
 
 @example
 ```ts
 jsoncSet({ state, path: ['a'], value: 2 });
 ```
 */
export function jsoncSet({
  state,
  path,
  value,
}: {
  readonly state: JsoncEditState;
  readonly path: JsoncPath;
  readonly value: ReadonlyDeep<JsonValue>;
},): JsoncEditState {
  // The document contract is a record or array root. Accepting a scalar here would return a state
  // whose canonical output this package's own parser rejects, so the root keeps its shape. Fuzzing
  // the Rust port found the same hole there as an emission of `null`.
  if ((path.length === 0) && (((typeof value) !== 'object') || (value === null)))
    throw new JsoncTypeError({ message: 'jsoncSet: the document root must stay an object or array', },);
  return {
    root: setAtPath({
      node: state.root,
      path,
      pathIndex: 0,
      newNode: jsToNode({ value, },),
    },),
  };
}

//endregion Set

//region Delete

/**
 Rebuilds `node` with the path target removed. At the final segment the matching
 record entries (all, since duplicates are undefined behavior) or the array
 element are dropped; earlier segments are descended.
 
 @param node - Current node being descended.
 
 @param path - Full path being deleted.
 
 @param pathIndex - Index of the segment to resolve at this level.
 
 @returns Rebuilt node.
 
 @throws JsoncPathNotFoundError when an intermediate segment is missing.
 
 @throws JsoncTypeError when a segment cannot index the node's kind.
 
 @example
 ```ts
 deleteAtPath({ node, path: ['a'], pathIndex: 0 });
 ```
 */
function deleteAtPath({
  node,
  path,
  pathIndex,
}: {
  readonly node: JsoncValue;
  readonly path: JsoncPath;
  readonly pathIndex: number;
},): JsoncValue {
  /**
   Segment resolved at this level.
   */
  const segment = nonNullishOrThrow(path.at(pathIndex,),);
  /**
   Whether this segment addresses the value to remove.
   */
  const isLast = pathIndex === (path.length - 1);
  if ((node.kind === 'record') && ((typeof segment) === 'string')) {
    if (isLast)
      return {
        ...node,
        entries: node.entries
          .filter(function keepOther(entry,): boolean {
            return entry.key
              .value
              !== segment;
          },),
      };
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
          value: deleteAtPath({
            node: entry.value,
            path,
            pathIndex: pathIndex + 1,
          },),
        },
        ),
    };
  }
  if ((node.kind === 'array') && ((typeof segment) === 'number')) {
    if (!Number.isInteger(segment,))
      throw new JsoncPathNotFoundError({ path, },);
    if (isLast)
      return {
        ...node,
        elements: node.elements
          .filter(function keepOther(
            _element: JsoncValue,
            index: number,
          ): boolean {
            return index !== segment;
          },),
      };
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
        deleteAtPath({
          node: element,
          path,
          pathIndex: pathIndex + 1,
        },),
        ),
    };
  }
  throw new JsoncTypeError({
    message: `jsoncDelete: cannot index ${node.kind} with ${JSON.stringify(segment,)}`,
  },);
}

/**
 Deletes the value at a path, returning a fresh state.
 
 @param state - Edit state.
 
 @param path - Path to delete; must be non-empty.
 
 @returns Fresh state with the value removed.
 
 @throws JsoncTypeError when the path is empty or a segment cannot index a node.
 
 @throws JsoncPathNotFoundError when an intermediate segment is missing.
 
 @example
 ```ts
 jsoncDelete({ state, path: ['a'] });
 ```
 */
export function jsoncDelete({
  state,
  path,
}: {
  readonly state: JsoncEditState;
  readonly path: JsoncPath;
},): JsoncEditState {
  if (path.length === 0)
    throw new JsoncTypeError({ message: 'jsoncDelete: cannot delete the document root', },);
  return {
    root: deleteAtPath({
      node: state.root,
      path,
      pathIndex: 0,
    },),
  };
}

//endregion Delete
