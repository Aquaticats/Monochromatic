import type {
  JsonValue,
  ReadonlyDeep,
} from 'type-fest';
import type {
  JsoncBoolean,
  JsoncNull,
  JsoncNumber,
  JsoncString,
} from './value.ts';

//region Constants

/**
 One level of canonical indentation.
 */
const INDENT_UNIT = '  ';

//endregion Constants

//region Scalar emit

/**
 Emits a scalar node's text. The author's raw token is reused when present, so
 an unedited `1.0` or escaped string round-trips exactly; an edited value
 (which carries no `raw`) is re-encoded through the JSON grammar.
 
 @param node - Scalar node to emit.
 
 @returns Scalar text.
 
 @example
 ```ts
 emitScalar({ node: { kind: 'number', value: 1, raw: '1.0' } }); // => '1.0'
 ```
 */
export function emitScalar({
  node,
}: {
  readonly node: JsoncString | JsoncNumber | JsoncBoolean | JsoncNull;
},): string {
  if (node.kind === 'string')
    return node.raw ?? JSON.stringify(node.value,);
  if (node.kind === 'number')
    return node.raw ?? JSON.stringify(node.value,);
  if (node.kind === 'boolean')
    return node.value
      ? 'true'
      : 'false';
  return 'null';
}

//endregion Scalar emit

//region Plain JSON emit

/**
 Emits a fast-path `plainJson` leaf as canonical 2-space JSON, re-indented so
 nested lines align with the surrounding structure. The leaf has no comments by
 construction, so native `JSON.stringify` is faithful.
 
 @param json - Plain JSON value.
 
 @param indent - Indentation depth in levels for continuation lines.
 
 @returns Canonical JSON text.
 
 @mutates json - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
 
 @example
 ```ts
 emitPlainJson({ json: { a: 1 }, indent: 0 });
 // => '{\n  "a": 1\n}'
 ```
 */
export function emitPlainJson({
  json,
  indent,
}: {
  json: ReadonlyDeep<JsonValue>;
  readonly indent: number;
},): string {
  /**
   Indentation prefix added to every continuation line.
   */
  const pad = INDENT_UNIT.repeat(indent,);
  return JSON.stringify(
    json,
    null,
    2,
  )
    .split('\n',)
    .join(`\n${pad}`,);
}

//endregion Plain JSON emit
