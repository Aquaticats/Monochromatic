/**
 {@link tomlGetValue}: read the effective JS value at a path.
 
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
 The JS value at `path`, or `undefined` when the path does not exist.
 
 Walks the current document tree and materializes the addressed node, so the
 result always agrees with {@link tomlStringify}; tables and inline tables
 return nested objects, arrays and array-of-tables return arrays, primitives
 return the JS primitive.
 
 @returns Computed result (`unknown`).
 
 @example
 ```ts
 tomlGetValue({ edit, path: ['fruits', 0, 'name',], },);  // 'apple'
 ```
 */
export function tomlGetValue(
  {
    edit,
    path,
  }: {
    readonly edit: TomlEditState;
    readonly path: TomlPath;
  },
): unknown {
  /**
   Materialized value at the path; the missing sentinel maps to `undefined`.
   */
  const result = navigate({
    root: materializeDocument({ edit, },),
    path,
  },);
  return result === MISSING ? undefined : result;
}
