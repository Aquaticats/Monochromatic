import {
  parseTomlEdit,
  tomlGetValue,
  tomlSet,
  tomlStringify,
  type TomlValueInput,
} from '@monochromatic-dev/toml-edit-core/ts';
import type { Path, } from '../types.ts';

/**
 Extracts a value from TOML content at a structured path.
 
 Parses with {@link parseTomlEdit} in splice mode and resolves the value via
 {@link tomlGetValue}. Array segments index array-of-tables; string segments index
 keys including those containing literal dots.
 
 Return type is `string` for the common path; missing paths surface as
 `undefined` (the underlying `JSON.stringify(undefined)` returns `undefined`),
 so callers may compare with `=== undefined`.
 
 @param path - Sequence of key segments (and numeric array-of-tables indices)
 
 @param content - TOML string to parse
 
 @returns Extracted value as string (`JSON.stringify(value, null, 2)` when not already a string), or `undefined` when missing
 
 @throws Error from the underlying parser when content is not valid TOML
 
 @example
 ```ts
 getTomlProperty({ path: ['package', 'version',], content: '[package]\nversion = "1.0.0"', },);
 // '1.0.0'
 ```
 */
export function getTomlProperty(
  {
    path,
    content,
  }: {
    readonly path: Path;
    readonly content: string;
  },
): string {
  /**
   Parsed TOML state in splice mode; no pending edits
   */
  const edit = parseTomlEdit({ source: content, },);
  /**
   Effective JS value at the path, or undefined when missing
   */
  const value: unknown = tomlGetValue({
    edit,
    path,
  },);
  return ((typeof value) === 'string') ? value : JSON.stringify(
    value,
    null,
    2,
  );
}

/**
 Sets a single key in TOML source text and returns the updated text.
 
 Splice mode preserves comments and unmutated whitespace byte-identically;
 only the affected node is canonicalised. Object values become inline tables;
 arrays of objects become array-of-tables blocks.
 
 Each call parses and stringifies. When applying multiple edits to the same
 source, prefer calling {@link parseTomlEdit} once, chaining {@link tomlSet},
 and calling {@link tomlStringify} at the end (all three are exported by
 `\@monochromatic-dev/toml-edit-core`).
 
 @param content - Source TOML text to edit
 
 @param path - Sequence of key segments (and numeric array-of-tables indices) identifying the target
 
 @param value - JS value to write
 
 @returns Updated TOML text
 
 @throws Error from {@link parseTomlEdit} when content is not valid TOML
 
 @throws {@link TomlTypeError} when value type mismatches the target node kind
 
 @throws {@link TomlImmutableNodeError} when path-create would violate TOML structure (e.g. through a scalar)
 
 @example
 ```ts
 editTomlKey({
   content: '[pkg]\nversion = "1.0.0"',
   path: ['pkg', 'version',],
   value: '1.2.3',
 },);
 // '[pkg]\nversion = "1.2.3"'
 ```
 */
export function editTomlKey(
  {
    content,
    path,
    value,
  }: {
    readonly content: string;
    readonly path: Path;
    readonly value: TomlValueInput;
  },
): string {
  /**
   Parsed TOML state ready for one mutation
   */
  const edit = parseTomlEdit({ source: content, },);
  /**
   Fresh state with the pending edit recorded
   */
  const edited = tomlSet({
    edit,
    path,
    value,
  },);
  return tomlStringify({ edit: edited, },);
}
