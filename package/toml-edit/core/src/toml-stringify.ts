/**
 {@link tomlStringify}: emit the TOML text for the current state.
 
 @module
 */

import { emitDocument, } from './emit-document.ts';
import type { TomlEditState, } from './types.ts';

/**
 Emit the current state as TOML text by walking its block tree.
 
 Clean (unmutated) nodes emit their original bytes verbatim, so a parsed
 document with no edits is byte-identical to its source. Mutated nodes render
 canonically per `edit.canonical`.
 
 @param edit - The state to emit.
 
 @returns The TOML output.
 
 @example
 ```ts
 const edit = parseTomlEdit({ source, },);
 tomlStringify({ edit, },); // === source
 ```
 */
export function tomlStringify({ edit, }: { readonly edit: TomlEditState; },): string {
  return emitDocument({ edit, },);
}
