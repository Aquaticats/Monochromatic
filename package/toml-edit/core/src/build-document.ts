/**
 Build the editable {@link Block} tree from a parsed `toml-eslint-parser`
 program and its source.
 
 The physical layout is flat: `program.body[0].body` is an ordered list of
 top-level key-values and table headers. Each entry's physical span runs from
 its first key char through its trailing newline; the gaps between spans (and
 the prologue/epilogue) become verbatim {@link FillerBlock}s. A table's body
 is tiled the same way, so the whole tree partitions `source` exactly and a
 clean document emits byte-for-byte.
 
 @module
 */

import type { AST, } from 'toml-eslint-parser';

import {
  attachedCommentValues,
  lineEndAfter,
  trailingCommentValue,
} from './build-comments.ts';
import { buildValue, } from './build-value.ts';
import type {
  Block,
  KeyValueNode,
  TableNode,
} from './document.ts';
import { keysOf, } from './path.ts';
import type { TomlComment, } from './types.ts';

/**
 An entry paired with its physical span so the tiler can place fillers.
 */
type Built = {
  readonly node: KeyValueNode | TableNode;
  readonly start: number;
  readonly end: number;
};

/**
 Build the top-level {@link Block} list for a parsed program.
 
 @returns Ordered top-level blocks that partition `source`.
 
 @example
 ```ts
 buildBlocks({ source, program: parseTOML(source,), },);
 ```
 */
export function buildBlocks(
  {
    source,
    program,
  }: {
    readonly source: string;
    readonly program: AST.TOMLProgram;
  },
): readonly Block[] {
  /**
   Flat comment list so entry builders can attribute leading/trailing comments.
   */
  const { comments, } = program;
  /**
   Top-level entries (key-values and table headers) in document order.
   */
  const children = program.body[0]
    .body;
  /**
   Each child built with its physical span so the tiler can interleave fillers.
   */
  const built = children.map(function each(
    child: AST.TOMLKeyValue | AST.TOMLTable,
  ) {
    return child.type
      === 'TOMLTable'
      ? buildTable({
        source,
        comments,
        table: child,
      },)
      : buildKeyValue({
        source,
        comments,
        kv: child,
      },);
  },);
  return tile({
    source,
    start: 0,
    end: source.length,
    built,
  },);
}

/**
 Interleave built entries with verbatim fillers across `[start, end)`.
 
 @returns Ordered blocks tiling the span.
 */
function tile(
  {
    source,
    start,
    end,
    built,
  }: {
    readonly source: string;
    readonly start: number;
    readonly end: number;
    readonly built: readonly Built[];
  },
): readonly Block[] {
  /**
   Accumulated blocks; a running cursor tracks how much source is placed.
   */
  const blocks: Block[] = [];
  /**
   Cursor over the source so gaps become fillers between entries.
   */
  let cursor = start;
  for (const b of built) {
    if (b.start
      > cursor)
      blocks.push({
        kind: 'filler',
        text: source.slice(
          cursor,
          b.start,
        ),
      },);
    blocks.push(b.node,);
    cursor = b.end;
  }
  if (cursor
    < end)
    blocks.push({
      kind: 'filler',
      text: source.slice(
        cursor,
        end,
      ),
    },);
  return blocks;
}

/**
 Build a top-level or table-body key-value entry with its physical span.
 
 @returns Built key-value with span.
 
 @mutates kv - Value construction can invoke caller-owned AST hooks through `getStaticTOMLValue`.
 */
function buildKeyValue(
  {
    source,
    comments,
    kv,
  }: {
    readonly source: string;
    readonly comments: readonly TomlComment[];
    readonly kv: AST.TOMLKeyValue;
  },
): Built {
  /**
   Physical line end so the entry span absorbs a trailing comment and newline.
   */
  const end = lineEndAfter({
    source,
    from: kv.range[1],
  },);
  /**
   Trailing same-line comment value, carried for reads and synthetic rendering.
   */
  const { value: commentAfter, } = trailingCommentValue({
    comments,
    source,
    from: kv.value
      .range[1],
  },);
  /**
   Key-value node; `origin.range` is the whole physical line.
   */
  const node: KeyValueNode = {
    kind: 'keyvalue',
    keySegments: keysOf({ key: kv.key, },),
    value: buildValue({ node: kv.value, },),
    origin: {
      kind: 'clean',
      range: [
        kv.range[0],
        end,
      ],
      astNode: kv,
    },
    valueRange: kv.value
      .range,
    commentsBefore: attachedCommentValues({
      comments,
      source,
      at: kv.range[0],
    },),
    ...(commentAfter === undefined ? {} : { commentAfter, }),
  };
  return {
    node,
    start: kv.range[0],
    end,
  };
}

/**
 Build a `[foo]` / `[[foo]]` table section with its recursively-tiled body.
 
 @returns Built table with span covering the header through its last body entry.
 */
function buildTable(
  {
    source,
    comments,
    table,
  }: {
    readonly source: string;
    readonly comments: readonly TomlComment[];
    readonly table: AST.TOMLTable;
  },
): Built {
  /**
   Header line end so the header span covers `[foo]` plus its trailing newline.
   */
  const headerEnd = lineEndAfter({
    source,
    from: table.key
      .range[1],
  },);
  /**
   Body entries built with spans so the body tiler can interleave fillers.
   */
  const bodyBuilt = table.body
    .map(function each(kv: AST.TOMLKeyValue,) {
      return buildKeyValue({
        source,
        comments,
        kv,
      },);
    },);
  /**
   Table span end: after the last body entry, or the header line when empty.
   */
  const end = bodyBuilt.length
    === 0 ? headerEnd : nonNullEnd(bodyBuilt,);
  /**
   Trailing same-line comment after the header's closing bracket.
   */
  const { value: commentAfter, } = trailingCommentValue({
    comments,
    source,
    from: table.key
      .range[1],
  },);
  /**
   Header path minus a trailing array index; that index is `aotIndex`.
   */
  const isArray = table.kind
    === 'array';
  /**
   Resolved header segments, keeping any interior numeric slots.
   */
  const resolved = table.resolvedKey;
  /**
   Trailing resolved segment; for an `[[foo]]` header it is the numeric index.
   */
  const lastResolved = resolved.at(-1,);
  /**
   Array-of-tables instance index, when this is an `[[foo]]` header. A numeric
   check narrows without an unsafe assertion (an array header's final resolved
   segment is always the numeric instance index).
   */
  const aotIndex = (isArray && ((typeof lastResolved) === 'number')) ? lastResolved : undefined;
  /**
   Table node; `headerOrigin.range` is the whole header line and `body` is the
   recursively-tiled section body.
   */
  const node: TableNode = {
    kind: 'table',
    tableKind: table.kind,
    headerSegments: isArray ? resolved.slice(
      0,
      -1,
    ) : resolved,
    ...(aotIndex === undefined ? {} : { aotIndex, }),
    headerOrigin: {
      kind: 'clean',
      range: [
        table.range[0],
        headerEnd,
      ],
      astNode: table,
    },
    body: tile({
      source,
      start: headerEnd,
      end,
      built: bodyBuilt,
    },),
    commentsBefore: attachedCommentValues({
      comments,
      source,
      at: table.range[0],
    },),
    ...(commentAfter === undefined ? {} : { commentAfter, }),
  };
  return {
    node,
    start: table.range[0],
    end,
  };
}

/**
 End offset of the last built body entry (non-empty list precondition).
 
 @param built - Body entries whose final span end bounds the table section;
   called only when non-empty, so the last entry decides the span.
 
 @returns Last entry's end offset.
 */
function nonNullEnd(built: readonly Built[],): number {
  /**
   Last entry so the table span reaches its final body line.
   */
  const last = built.at(-1);
  return last === undefined ? 0 : last.end;
}
