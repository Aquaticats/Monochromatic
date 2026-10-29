import type {
  JsonValue,
  ReadonlyDeep,
} from 'type-fest';
import type { JsoncComment, } from './comment.ts';

/**
 Parsed JSONC string node.
 
 `value` is the decoded string; `raw` is the original quoted source slice when
 available, kept so canonical emit can reproduce the author's exact escapes for
 an unedited value. `raw` is absent on values produced by an edit.
 
 @example
 ```ts
 const node: JsoncString = { kind: 'string', value: 'a', raw: '"a"' };
 ```
 */
export type JsoncString = {
  readonly kind: 'string';
  readonly value: string;
  readonly raw?: string;
  readonly comment?: JsoncComment;
};

/**
 Parsed JSONC number node.
 
 `value` is the decoded number; `raw` preserves the author's literal form
 (`1.0`, `1e3`, `-0`) for an unedited value, since those collapse under JS
 number formatting.
 
 @example
 ```ts
 const node: JsoncNumber = { kind: 'number', value: 1, raw: '1.0' };
 ```
 */
export type JsoncNumber = {
  readonly kind: 'number';
  readonly value: number;
  readonly raw?: string;
  readonly comment?: JsoncComment;
};

/**
 Parsed JSONC boolean node. No `raw` is needed because `true` and `false` have
 a single canonical spelling.
 
 @example
 ```ts
 const node: JsoncBoolean = { kind: 'boolean', value: true };
 ```
 */
export type JsoncBoolean = {
  readonly kind: 'boolean';
  readonly value: boolean;
  readonly comment?: JsoncComment;
};

/**
 Parsed JSONC null node.
 
 @example
 ```ts
 const node: JsoncNull = { kind: 'null' };
 ```
 */
export type JsoncNull = {
  readonly kind: 'null';
  readonly comment?: JsoncComment;
};

/**
 Parsed JSONC array node. `elements` preserves order; each element is itself a
 comment-bearing node.
 
 @example
 ```ts
 const node: JsoncArray = { kind: 'array', elements: [] };
 ```
 */
export type JsoncArray = {
  readonly kind: 'array';
  readonly elements: readonly JsoncValue[];
  readonly comment?: JsoncComment;
};

/**
 Key of one record entry. A key is comment-bearing in its own right, so a
 comment sitting on the key (`{ /* k *\/ "a": 1 }`) is addressable separately
 from a comment on the value.
 
 @example
 ```ts
 const key: JsoncKey = { value: 'a', raw: '"a"' };
 ```
 */
export type JsoncKey = {
  readonly value: string;
  readonly raw?: string;
  readonly comment?: JsoncComment;
};

/**
 One key-to-value entry of a record. Entries are stored as an ordered list, not
 a map, so insertion order and duplicate keys are preserved losslessly.
 
 @example
 ```ts
 const entry: JsoncRecordEntry = {
   key: { value: 'a' },
   value: { kind: 'number', value: 1 },
 };
 ```
 */
export type JsoncRecordEntry = {
  readonly key: JsoncKey;
  readonly value: JsoncValue;
};

/**
 Parsed JSONC object node. Modeled as an ordered list of entries rather than a
 `Record` so duplicate keys survive and serialization order is stable.
 
 @example
 ```ts
 const node: JsoncRecord = { kind: 'record', entries: [] };
 ```
 */
export type JsoncRecord = {
  readonly kind: 'record';
  readonly entries: readonly JsoncRecordEntry[];
  readonly comment?: JsoncComment;
};

/**
 Leaf produced by the comment-free fast-path: a clean region parsed with native
 `JSON.parse` and held as a plain JS value. A clean region carries no inner
 comments by definition, so it needs no per-node structure; a `comment` may
 still sit on the leaf as a whole.
 
 @example
 ```ts
 const node: JsoncPlainJson = { kind: 'plainJson', json: { a: 1 } };
 ```
 */
export type JsoncPlainJson = {
  readonly kind: 'plainJson';
  readonly json: ReadonlyDeep<JsonValue>;
  readonly comment?: JsoncComment;
};

/**
 Discriminated union of every parsed JSONC node, tagged by `kind`. The
 `plainJson` variant is the fast-path leaf; the rest are structured nodes that
 preserve inner comments.
 
 @example
 ```ts
 function isRecord(node: JsoncValue): node is JsoncRecord {
   return node.kind === 'record';
 }
 ```
 */
export type JsoncValue =
  | JsoncString
    | JsoncNumber
    | JsoncBoolean
    | JsoncNull
    | JsoncArray
    | JsoncRecord
    | JsoncPlainJson;
