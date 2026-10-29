/**
 Path used by file-enforcer's property-extraction and key-edit helpers.
 
 Array of segments: string segments index into object keys; numeric segments
 index into array elements. Mixed forms compose for nested array-of-tables
 access such as `['fruits', 0, 'name',]`.
 
 Coincides structurally with {@link TomlPath} from `\@monochromatic-dev/toml-edit-core`
 so values flow without conversion. Defined locally so callers do not have to
 import a TOML-named type for JSON helpers.
 
 @example
 ```ts
 const path: Path = ['settings', 'theme',];
 const arrayPath: Path = ['fruits', 0, 'name',];
 ```
 */
export type Path = readonly (string | number)[];
