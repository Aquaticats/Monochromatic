/**
 {@link tomlHas}: presence check for a path.
 
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
 True when `path` resolves to an existing key, value, table, or
 array-of-tables in the current document.
 
 @returns Resulting boolean.
 
 @example
 ```ts
 tomlHas({ edit, path: ['tools', 'bun',], },);  // true / false
 ```
 */
export function tomlHas(
  {
    edit,
    path,
  }: {
    readonly edit: TomlEditState;
    readonly path: TomlPath;
  },
): boolean {
  return navigate({
    root: materializeDocument({ edit, },),
    path,
  },) !== MISSING;
}
