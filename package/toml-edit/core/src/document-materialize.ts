/**
 Materialize a whole {@link TomlEditState} into a nested JS value, and
 navigate it by {@link TomlPath}.
 
 Folds top-level key-values and table sections into one object the same way
 `getStaticTOMLValue` folds an AST: dotted keys and `[a.b]` headers nest into
 objects, `[[foo]]` headers append objects to an array. Because it walks the
 current tree, reads reflect every prior mutation.
 
 The fold is immutable and prototype-safe: it threads a fresh root through
 {@link setDeep} / {@link updateDeep}, which write keys as own properties, so a
 `__proto__` key round-trips as a normal own property.
 
 @module
 */

import type {
  Block,
  TableNode,
} from './document.ts';
import type {
  TomlEditState,
  TomlPath,
} from './types.ts';
import {
  isRecord,
  isUnknownArray,
  materializeValue,
  setDeep,
  updateDeep,
} from './value-materialize.ts';

/**
 Sentinel for "no value at this path". A unique symbol so it never collides
 with a real materialized value (which may legitimately be `undefined`-free
 but never this symbol).
 */
export const MISSING: unique symbol = Symbol('toml-edit/document-navigate-path-missing',);

/**
 Materialize the whole document into a nested JS object.
 
 @param edit - Current document state whose blocks are folded into one object.
 
 @returns Root object.
 
 @example
 ```ts
 materializeDocument({ edit, },); // { title: 'x', tools: { bun: 'latest' } }
 ```
 */
export function materializeDocument(
  { edit, }: { readonly edit: TomlEditState; },
): Record<string, unknown> {
  return edit.blocks
    .reduce<Record<string, unknown>>(
    function step(
      root: Readonly<Record<string, unknown>>,
      block,
    ) {
      if (block.kind
        === 'filler')
        return root;
      if (block.kind
        === 'keyvalue') {
        return setDeep({
          container: root,
          path: block.keySegments,
          value: materializeValue({ value: block.value, },),
        },);
      }
      return foldTable({
        root,
        table: block,
      },);
    },
    {},
    );
}

/**
 Append a fresh empty array-of-tables instance to `existing`, creating the
 array when the slot is absent or not an array. The `updateDeep` leaf op for
 an `[[foo]]` header.
 
 @param existing - Current value at the header slot; an array is extended,
   anything else is replaced with a one-instance array.
 
 @returns Array with a fresh empty instance appended.
 */
function appendInstance(existing: unknown,): readonly unknown[] {
  return isUnknownArray(existing,)
    ? [
      ...existing,
      {},
    ]
    : [{},];
}

/**
 Keep the existing record at the slot, or create a fresh one when absent. The
 `updateDeep` leaf op for a standard `[foo]` header.
 
 @param existing - Current value at the header slot; a record is kept, anything
   else becomes a fresh table.
 
 @returns Record the table body folds into.
 */
function ensureRecord(existing: unknown,): Record<string, unknown> {
  return isRecord(existing,) ? existing : {};
}

/**
 Fold one table section (standard or array-of-tables) into `root`.
 
 @param root - Accumulated document object the section folds into.
 
 @param table - Table section whose header and body entries are folded.
 
 @returns Fresh root with the section folded in.
 
 @example
 ```ts
 foldTable({ root: {}, table, },);
 ```
 */
function foldTable(
  {
    root,
    table,
  }: {
    readonly root: Readonly<Record<string, unknown>>;
    readonly table: TableNode;
  },
): Record<string, unknown> {
  /**
   Header path both kinds fold their body entries under.
   */
  const header = table.headerSegments;
  /**
   Root with the target container established: a fresh appended instance for an
   array-of-tables header, else an ensured (kept-or-created) standard record.
   */
  const base = table.tableKind
    === 'array'
    ? updateDeep({
      container: root,
      path: header,
      update: appendInstance,
    },)
    : updateDeep({
      container: root,
      path: header,
      update: ensureRecord,
    },);
  /**
   Body key-values so both table kinds share the fold into their container.
   */
  const bodyKvs = table.body
    .filter(function isKv(b,): b is Extract<Block, { kind: 'keyvalue'; }> {
      return b.kind
        === 'keyvalue';
    },);
  return bodyKvs.reduce<Record<string, unknown>>(
    function foldKv(
      acc: Readonly<Record<string, unknown>>,
      kv,
    ) {
      return setDeep({
        container: acc,
        path: [
          ...header,
          ...kv.keySegments,
        ],
        value: materializeValue({ value: kv.value, },),
      },);
    },
    base,
  );
}

/**
 Navigate `path` through a materialized root, returning {@link MISSING} when a
 segment does not resolve.
 
 @param root - Materialized document object to walk.
 
 @param path - Segment chain addressing the value to read.
 
 @returns Value at the path, or {@link MISSING}.
 
 @mutates root - `Object.hasOwn` can invoke caller-owned proxy descriptor hooks while navigating.
 
 @example
 ```ts
 navigate({ root, path: ['tools', 'bun'], },);
 ```
 */
export function navigate(
  {
    root,
    path,
  }: {
    readonly root: Readonly<Record<string, unknown>>;
    readonly path: TomlPath;
  },
): unknown {
  /**
   Cursor descending one segment per step; short-circuits to MISSING on a miss.
   */
  let cursor: unknown = root;
  for (const seg of path) {
    if (isUnknownArray(cursor,)) {
      if ((typeof seg) !== 'number')
        return MISSING;
      if ((seg < 0) || (seg >= cursor.length))
        return MISSING;
      cursor = cursor[seg];
      continue;
    }
    if (!isRecord(cursor,))
      return MISSING;
    /**
     Object property key for this segment, read own-only for prototype safety.
     */
    const key = String(seg,);
    if (!Object.hasOwn(
      cursor,
      key,
    ))
      return MISSING;
    cursor = cursor[key];
  }
  return cursor;
}
