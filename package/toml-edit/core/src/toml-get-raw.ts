/**
 {@link tomlGetRaw}: read the original source slice at a path. Splice mode only.
 
 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  TomlPathNotFoundError,
  TomlSpliceUnavailableError,
} from './errors.ts';
import { formatPath, } from './path.ts';
import {
  locateValueNode,
  NOT_LOCATED,
} from './resolve-document.ts';
import type {
  TomlEditState,
  TomlPath,
} from './types.ts';

/**
 Return the original source substring that spelled the value at `path`.
 
 Returns the parse-time bytes of a clean (unmutated) node, so round-trip-
 sensitive callers can diff without canonical reformatting. A path created or
 edited by a mutation has no source bytes and throws.
 
 @returns Computed string.
 
 @throws {@link TomlSpliceUnavailableError} when the state is in canonical mode.
 
 @throws {@link TomlPathNotFoundError} when `path` has no clean source slice.
 
 @example
 ```ts
 const raw = tomlGetRaw({ edit, path: ['version',], },);  // e.g. `'"1.0.0"'`
 ```
 */
export function tomlGetRaw(
  {
    edit,
    path,
  }: {
    readonly edit: TomlEditState;
    readonly path: TomlPath;
  },
): string {
  if (edit.mode
    === 'canonical') {
    throw new TomlSpliceUnavailableError(
      `tomlGetRaw requires splice mode; current state is canonical`,
    );
  }
  /**
   Structural location so a clean node's source range can be sliced.
   */
  const located = locateValueNode({
    blocks: edit.blocks,
    path,
  },);
  if (located === NOT_LOCATED)
    throw notFound({ path, },);
  if (located.kind
    === 'value') {
    if (located.value
      .origin
      .kind
      !== 'clean')
      throw notFound({ path, },);
    return edit.source
      .slice(
      located.value
        .origin
        .range[0],
      located.value
        .origin
        .range[1],
      );
  }
  if (located.kind
    === 'table') {
    if (located.table
      .headerOrigin
      .kind
      !== 'clean')
      throw notFound({ path, },);
    return edit.source
      .slice(
      located.table
        .headerOrigin
        .range[0],
      located.table
        .headerOrigin
        .range[1],
      );
  }
  /**
   First AoT instance header so the slice starts at the collection's top.
   */
  const first = nonNullishOrThrow(located.tables[0],);
  /**
   Last AoT instance header so the slice spans every instance.
   */
  const last = nonNullishOrThrow(located.tables
    .at(-1,),);
  if ((first.headerOrigin
    .kind
    !== 'clean') || (last.headerOrigin
      .kind
      !== 'clean'))
    throw notFound({ path, },);
  return edit.source
    .slice(
    first.headerOrigin
      .range[0],
    last.headerOrigin
      .range[1],
    );
}

/**
 Build the not-found error for `path`.
 
 @returns Error to throw.
 */
function notFound({ path, }: { readonly path: TomlPath; },): TomlPathNotFoundError {
  return new TomlPathNotFoundError(
    `Path ${formatPath({ path, },)} has no clean source slice`,
  );
}
