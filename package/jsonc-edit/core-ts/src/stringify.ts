import {
  isSingleLineComment,
  leadingComment,
  trailingComment,
} from './emit-comment.ts';
import {
  emitPlainJson,
  emitScalar,
} from './emit-value.ts';
import type {
  JsoncArray,
  JsoncRecord,
  JsoncRecordEntry,
  JsoncValue,
} from './value.ts';

//region Constants

/**
 One level of canonical indentation.
 */
const INDENT_UNIT = '  ';

//endregion Constants

//region Bare value emit

/**
 Emits a value's own text without its attached comment, recursing into
 containers. Comment placement is the caller's concern.
 
 @param node - Node to emit.
 
 @param indent - Indentation depth of this node in levels.
 
 @returns Value text.
 
 @mutates node - `JSON.stringify` may invoke hooks on embedded plain JSON values.
 
 @example
 ```ts
 emitBare({ node: { kind: 'boolean', value: true }, indent: 0 }); // => 'true'
 ```
 */
function emitBare({
  node,
  indent,
}: {
  node: JsoncValue;
  readonly indent: number;
},): string {
  if (node.kind === 'record')
    return emitRecord({
      node,
      indent,
    },);
  if (node.kind === 'array')
    return emitArray({
      node,
      indent,
    },);
  if (node.kind === 'plainJson')
    return emitPlainJson({
      json: node.json,
      indent,
    },);
  return emitScalar({ node, },);
}

//endregion Bare value emit

//region Container emit

/**
 Emits one record entry: the key's comment as leading lines, the key, the
 value, and the value's comment trailing (single line) or leading (multi-line).
 
 @param entry - Entry to emit.
 
 @param indent - Indentation depth of the entry in levels.
 
 @returns Entry text, ending with a trailing comma.
 
 @mutates entry - `JSON.stringify` may invoke hooks on embedded plain JSON values.
 
 @example
 ```ts
 emitEntry({ entry: { key: { value: 'a' }, value: { kind: 'number', value: 1 } }, indent: 1 });
 // => '  "a": 1,'
 ```
 */
function emitEntry({
  entry,
  indent,
}: {
  entry: JsoncRecordEntry;
  readonly indent: number;
},): string {
  /**
   Indentation prefix for the entry line.
   */
  const pad = INDENT_UNIT.repeat(indent,);
  /**
   Leading lines for the key's own comment.
   */
  const keyLead = (entry.key
    .comment
    === undefined)
    ? ''
    : leadingComment({
      comment: entry.key
        .comment,
      indent,
    },);
  /**
   Key text, reusing the author's raw quoting when present.
   */
  const keyText = entry.key
    .raw
    ?? JSON.stringify(entry.key
      .value,);
  /**
   Value text without its own comment.
   */
  const valText = emitBare({
    node: entry.value,
    indent,
  },);
  /**
   Value's attached comment, if any.
   */
  const valComment = entry.value
    .comment;
  if ((valComment !== undefined) && isSingleLineComment(valComment,))
    return `${keyLead}${pad}${keyText}: ${valText}, ${trailingComment({ comment: valComment, },)}`;
  if (valComment === undefined)
    return `${keyLead}${pad}${keyText}: ${valText},`;
  // A multi-line value comment must follow the colon: emitted before the key it
  // would reparse as that key's comment and silently change owner.
  return `${keyLead}${pad}${keyText}:\n${leadingComment({
    comment: valComment,
    indent,
  },)}${pad}${valText},`;
}

/**
 Emits a record node as a brace-delimited block, one entry per line.
 
 @param node - Record node.
 
 @param indent - Indentation depth of the record in levels.
 
 @returns Record text.
 
 @mutates node - `JSON.stringify` may invoke hooks on embedded plain JSON values.
 
 @example
 ```ts
 emitRecord({ node: { kind: 'record', entries: [] }, indent: 0 }); // => '{}'
 ```
 */
function emitRecord({
  node,
  indent,
}: {
  node: JsoncRecord;
  readonly indent: number;
},): string {
  if (node.entries
    .length
    === 0)
    return '{}';
  /**
   Canonical entry lines accumulated without another effect boundary.
   */
  const entryLines: string[] = [];
  for (const entry of node.entries) {
    entryLines.push(emitEntry({
      entry,
      indent: indent + 1,
    },),);
  }
  /**
   Entry lines joined with newlines.
   */
  const inner = entryLines.join('\n',);
  return `{\n${inner}\n${INDENT_UNIT.repeat(indent,)}}`;
}

/**
 Emits one array element: the element's comment trailing (single line) or
 leading (multi-line), then the element text.
 
 @param element - Element node.
 
 @param indent - Indentation depth of the element in levels.
 
 @returns Element text, ending with a trailing comma.
 
 @mutates element - `JSON.stringify` may invoke hooks on embedded plain JSON values.
 
 @example
 ```ts
 emitElement({ element: { kind: 'number', value: 1 }, indent: 1 }); // => '  1,'
 ```
 */
function emitElement({
  element,
  indent,
}: {
  element: JsoncValue;
  readonly indent: number;
},): string {
  /**
   Indentation prefix for the element line.
   */
  const pad = INDENT_UNIT.repeat(indent,);
  /**
   Element text without its own comment.
   */
  const valText = emitBare({
    node: element,
    indent,
  },);
  /**
   Element's attached comment, if any.
   */
  const {comment} = element;
  if ((comment !== undefined) && isSingleLineComment(comment,))
    return `${pad}${valText}, ${trailingComment({ comment, },)}`;
  /**
   Leading lines for a multi-line element comment.
   */
  const lead = (comment === undefined)
    ? ''
    : leadingComment({
      comment,
      indent,
    },);
  return `${lead}${pad}${valText},`;
}

/**
 Emits an array node as a bracket-delimited block, one element per line.
 
 @param node - Array node.
 
 @param indent - Indentation depth of the array in levels.
 
 @returns Array text.
 
 @mutates node - `JSON.stringify` may invoke hooks on embedded plain JSON values.
 
 @example
 ```ts
 emitArray({ node: { kind: 'array', elements: [] }, indent: 0 }); // => '[]'
 ```
 */
function emitArray({
  node,
  indent,
}: {
  node: JsoncArray;
  readonly indent: number;
},): string {
  if (node.elements
    .length
    === 0)
    return '[]';
  /**
   Canonical element lines accumulated without another effect boundary.
   */
  const elementLines: string[] = [];
  for (const element of node.elements) {
    elementLines.push(emitElement({
      element,
      indent: indent + 1,
    },),);
  }
  /**
   Element lines joined with newlines.
   */
  const inner = elementLines.join('\n',);
  return `[\n${inner}\n${INDENT_UNIT.repeat(indent,)}]`;
}

//endregion Container emit

//region Entry point

/**
 Serializes a parsed JSONC value back to canonical text, preserving all
 comments. Formatting is normalized to 2-space indentation with trailing
 commas; unedited scalar tokens keep their original spelling. This is canonical
 mode, not byte-identical splice: untouched whitespace is reflowed.
 
 @param value - Parsed JSONC value to serialize.
 
 @returns Canonical JSONC text.
 
 @mutates value - `JSON.stringify` may invoke hooks on embedded plain JSON values.
 
 @example
 ```ts
 emitJsoncValue({ value: parseJsonc({ source: '{"a":1}' as StringJsonc }) });
 // => '{\n  "a": 1,\n}'
 ```
 */
export function emitJsoncValue({
  value,
}: {
  value: JsoncValue;
},): string {
  /**
   Leading lines for a top-level document comment.
   */
  const lead = (value.comment === undefined)
    ? ''
    : leadingComment({
      comment: value.comment,
      indent: 0,
    },);
  return `${lead}${emitBare({
    node: value,
    indent: 0,
  },)}`;
}

//endregion Entry point
