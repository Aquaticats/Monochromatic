/**
 Shared types and kind maps for the `toml-test` tagged-JSON interchange.
 
 The upstream `toml-test` runner (`toml-lang/toml-test`) speaks a tagged JSON
 dialect where every scalar is an object `{ type, value }` with `value` always
 a JSON string, tables are JSON objects, and arrays are JSON arrays. These
 types model that dialect, and the maps translate between the toml-test tag
 names and `toml-eslint-parser`'s value `kind` names so the decode and encode
 adapters never hand-spell the correspondence twice.
 
 @module
 */

/**
 Tagged-scalar type names defined by the toml-test JSON encoding.
 
 Mirrors the `TOML_TYPE` enumeration in the runner's README so a decoder emits
 and an encoder accepts exactly these eight spellings.
 */
export type TaggedType =
  | 'string'
    | 'integer'
    | 'float'
    | 'bool'
    | 'datetime'
    | 'datetime-local'
    | 'date-local'
    | 'time-local';

/**
 Tagged scalar: a leaf value carrying its TOML type and a string payload.
 
 The runner compares payloads per type (floats numerically, datetimes as
 instants, everything else as exact strings), so `value` stays a string even
 for numbers and booleans.
 */
export type TaggedValue = {
  readonly type: TaggedType;
  readonly value: string;
};

/**
 Recursive tagged tree: a scalar, an array of trees, or a table of trees.
 
 A table is distinguished from a scalar at runtime by the absence of a string
 `type` discriminant, matching how the runner reads the dialect.
 */
export type TaggedTree =
  | TaggedValue
    | readonly TaggedTree[]
    | { readonly [key: string]: TaggedTree; };

/**
 `toml-eslint-parser` scalar `kind` names, narrowed to the value kinds.
 
 Excludes the structural kinds (`standard` / `array` table headers) because
 only leaf values cross the tagged-JSON boundary as scalars.
 */
export type TomlValueKind =
  | 'string'
    | 'integer'
    | 'float'
    | 'boolean'
    | 'offset-date-time'
    | 'local-date-time'
    | 'local-date'
    | 'local-time';

/**
 Parser `kind` to toml-test `type`.
 
 Used by the decode adapter when tagging a parsed leaf. The boolean and
 datetime spellings differ between the two vocabularies, so the map is not an
 identity.
 
 @example
 ```ts
 KIND_TO_TAG['offset-date-time']; // 'datetime'
 ```
 */
export const KIND_TO_TAG: Readonly<Record<TomlValueKind, TaggedType>> = Object.freeze({
  'string': 'string',
  'integer': 'integer',
  'float': 'float',
  'boolean': 'bool',
  'offset-date-time': 'datetime',
  'local-date-time': 'datetime-local',
  'local-date': 'date-local',
  'local-time': 'time-local',
},);

/**
 TOML versions the conformance adapters accept on the command line.
 
 Matches the runner's `-toml` selector so a single argument threads the right
 grammar into {@link parseTomlEdit}.
 */
export type ConformanceVersion = '1.0' | '1.1';
