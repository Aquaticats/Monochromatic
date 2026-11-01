/**
 `tomlGet`: alias for `tomlGetValue`.
 
 @module
 */

import { tomlGetValue, } from './toml-get-value.ts';
import type {
  TomlEditState,
  TomlPath,
} from './types.ts';

/**
 Alias for {@link tomlGetValue}. Provided for ergonomic call sites where
 the explicit `Value` suffix is verbose.
 
 @returns Computed result (`unknown`).
 
 @example
 ```ts
 tomlGet({ edit, path: ['title',], },);  // 'Demo'
 ```
 */
export function tomlGet(
  {
    edit,
    path,
  }: {
    readonly edit: TomlEditState;
    readonly path: TomlPath;
  },
): unknown {
  return tomlGetValue({
    edit,
    path,
  },);
}
