/**
 {@link tomlKeys}: list the keys at a path (or at the root).
 
 @module
 */

import {
  materializeDocument,
  MISSING,
  navigate,
} from './document-materialize.ts';
import type {
  TomlEditState,
  TomlPath,
} from './types.ts';

/**
 Return the immediate children of the value at `path`.
 
 For objects (tables and inline tables) returns key strings; for arrays and
 array-of-tables returns numeric indices; for a scalar or a missing path
 returns an empty array.
 
 @returns Computed result (`readonly (string | number)[]`).
 
 @example
 ```ts
 tomlKeys({ edit, },);                    // ['title', 'tools', 'fruits']
 tomlKeys({ edit, path: ['fruits',], },); // [0, 1] (array-of-tables)
 ```
 */
export function tomlKeys(
  {
    edit,
    path = [],
  }: {
    readonly edit: TomlEditState;
    readonly path?: TomlPath;
  },
): readonly (string | number)[] {
  /**
   Materialized value at the path; missing and scalars yield no keys.
   */
  const result = navigate({
    root: materializeDocument({ edit, },),
    path,
  },);
  if (result === MISSING)
    return [];
  if (Array.isArray(result,)) {
    return result.map(function eachIdx(
      _: unknown,
      i: number,
    ) {
      return i;
    },);
  }
  if ((result !== null) && ((typeof result) === 'object'))
    return Object.keys(result,);
  return [];
}
