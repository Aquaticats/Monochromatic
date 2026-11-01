/**
 Replace (or extend into) an existing value in the block tree for {@link tomlSet}.
 
 Descends through standard-table bodies and array-of-tables instances to the
 addressed key-value, then delegates value-node descent (arrays, inline tables)
 to {@link replaceInValue} in `set-value-inline.ts`, returning a fresh block
 list with that value replaced. An edited array/inline table is marked
 synthetic so it re-renders canonically; the enclosing key-value's key line
 stays clean. Returns {@link NOT_SET} when the path is not an existing value
 (the caller then creates or table-replaces).
 
 @module
 */

import { buildValueFromInput, } from './build-input.ts';
import type {
  Block,
  KeyValueNode,
} from './document.ts';
import { isPrefix, } from './path-prefix.ts';
import {
  existingArg,
  NOT_SET,
  replaceInValue,
} from './set-value-inline.ts';
import type {
  CanonicalOptions,
  TomlPath,
} from './types.ts';

/**
 Replace the existing value at `path` within `blocks`.
 
 @returns Fresh blocks, or {@link NOT_SET} when the path has no existing value.
 
 @example
 ```ts
 replaceExistingValue({ blocks, path: ['a','x'], value: 2, options, },);
 ```
 */
export function replaceExistingValue(
  {
    blocks,
    path,
    value,
    options,
  }: {
    readonly blocks: readonly Block[];
    readonly path: TomlPath;
    readonly value: unknown;
    readonly options: CanonicalOptions;
  },
): readonly Block[] | typeof NOT_SET {
  for (const [index, block,] of blocks.entries()) {
    /**
     Replacement block if this one owns the path, else the not-set sentinel.
     */
    const replaced = replaceInBlock({
      block,
      path,
      value,
      options,
    },);
    if (replaced !== NOT_SET)
      return blocks.with(
        index,
        replaced,
      );
  }
  return NOT_SET;
}

/**
 Attempt to replace the value at `path` within a single block.
 
 @returns Replacement block, or {@link NOT_SET}.
 */
function replaceInBlock(
  {
    block,
    path,
    value,
    options,
  }: {
    readonly block: Block;
    readonly path: TomlPath;
    readonly value: unknown;
    readonly options: CanonicalOptions;
  },
): Block | typeof NOT_SET {
  if (block.kind
    === 'keyvalue')
    return replaceInKeyValue({
      kv: block,
      path,
      value,
      options,
    },);
  if (block.kind
    !== 'table')
    return NOT_SET;
  /**
   Header path length; a strict prefix means the target lives in the body.
   */
  const headerLen = block.headerSegments
    .length;
  /**
   For array instances the next segment must select this instance's index.
   */
  const bodyPathStart = block.tableKind
    === 'array' ? headerLen + 1 : headerLen;
  if ((block.tableKind
    === 'array') && (path[headerLen] !== block.aotIndex))
    return NOT_SET;
  if ((!isPrefix({
    candidate: block.headerSegments,
    path,
  },)) || (path.length
    < (bodyPathStart
      + 1)))
    return NOT_SET;
  /**
   Body result after descending with the remaining path segments.
   */
  const newBody = replaceExistingValue({
    blocks: block.body,
    path: path.slice(bodyPathStart,),
    value,
    options,
  },);
  if (newBody === NOT_SET)
    return NOT_SET;
  return {
    ...block,
    body: newBody,
  };
}

/**
 Attempt to replace the value at `path` within a key-value.
 
 @returns Replacement key-value, or {@link NOT_SET}.
 */
function replaceInKeyValue(
  {
    kv,
    path,
    value,
    options,
  }: {
    readonly kv: KeyValueNode;
    readonly path: TomlPath;
    readonly value: unknown;
    readonly options: CanonicalOptions;
  },
): KeyValueNode | typeof NOT_SET {
  if (!isPrefix({
    candidate: kv.keySegments,
    path,
  },))
    return NOT_SET;
  if (kv.keySegments
    .length
    === path.length) {
    return {
      ...kv,
      value: buildValueFromInput({
        input: value,
        options,
        ...existingArg(kv.value,),
      },),
    };
  }
  /**
   Value after descending the remaining segments into the current value.
   */
  const newValue = replaceInValue({
    value: kv.value,
    rest: path.slice(kv.keySegments
      .length,),
    input: value,
    options,
    path,
  },);
  if (newValue === NOT_SET)
    return NOT_SET;
  return {
    ...kv,
    value: newValue,
  };
}
