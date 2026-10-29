/**
 {@link emptyTomlEdit}: construct a fresh edit state with no source.
 
 @module
 */

import { parseTomlEdit, } from './parse-toml-edit.ts';
import type {
  TomlEditState,
  TomlEmptyOptions,
} from './types.ts';

/**
 Build a fresh {@link TomlEditState} over an empty source, via {@link parseTomlEdit}.
 
 Mode is forced to `'canonical'` because there are no bytes to splice.
 Use this when generating a TOML file from scratch via setters and
 `tomlStringify`.
 
 @param canonical - Partial override of {@link CanonicalOptions}.
 
 @returns Fresh canonical-mode {@link TomlEditState}.
 
 @example
 ```ts
 import { emptyTomlEdit, tomlSet, tomlStringify } from '\@monochromatic-dev/toml-edit-core';
 
 const e0 = emptyTomlEdit();
 const e1 = tomlSet({ edit: e0, path: ['title'], value: 'Demo', },);
 console.log(tomlStringify({ edit: e1, },),);
 ```
 */
export function emptyTomlEdit({ canonical, }: TomlEmptyOptions = {},): TomlEditState {
  /**
   Bootstrapped state from a zero-byte source so callers can build documents incrementally.
   */
  const base = parseTomlEdit(
    canonical === undefined
      ? {
        source: '',
        mode: 'canonical',
      }
      : {
        source: '',
        mode: 'canonical',
        canonical,
      },
  );
  return {
    ...base,
    mode: 'canonical',
  };
}
