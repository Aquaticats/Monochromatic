/**
 The editable document tree: one always-current model that reads, writes, and
 emit all operate on.
 
 Physical structure mirrors `toml-eslint-parser`'s flat layout (a top-level
 ordered list of key-values and table headers); logical nesting is derived on
 demand from `keySegments` / `headerSegments`. Every node carries an
 {@link Origin}: a clean node retains its original source range and emits
 verbatim (so an unmutated document round-trips byte-for-byte); a dirty or
 synthetic node renders canonically.
 
 The cornerstone invariant: for a fully-clean document the blocks partition
 `source` exactly (every byte belongs to one entry range or one filler span,
 in order), so emitting each clean entry verbatim plus each filler verbatim
 reproduces `source`.
 
 @module
 */

import type { AST, } from 'toml-eslint-parser';

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
/**
 Half-open `[start, end)` char-offset span into the immutable source.
 */
export type Span = readonly [
  number,
  number,
];

/**
 Emission provenance of a node.
 
 `clean`: the node came from parse and is unmutated; emit `source.slice(range)`
 verbatim. `astNode` is retained so {@link tomlGetNode} / {@link tomlGetRaw}
 can return parse-time views.
 
 `synthetic`: the node was created or replaced by a mutation and has no
 faithful source span; render canonically.
 */
export type Origin =
  | {
    readonly kind: 'clean';
    readonly range: Span;
    readonly astNode: ForeignBorrowed<AST.TOMLNode>;
  }
    | { readonly kind: 'synthetic'; };

/**
 A TOML value: scalar leaf, array, or inline table.
 */
export type ValueNode =
  | ScalarNode
    | ArrayNode
    | InlineTableNode;

/**
 TOML scalar kinds, mirroring `AST.TOMLValue['kind']`.
 */
export type ScalarKind =
  | 'string'
    | 'integer'
    | 'float'
    | 'boolean'
    | 'offset-date-time'
    | 'local-date-time'
    | 'local-date'
    | 'local-time';

/**
 A primitive value leaf.
 
 `jsValue` is the materialized plain JS value (per `getStaticTOMLValue`
 semantics) used by reads. A clean scalar renders through its AST node
 (preserving raw spelling); a synthetic scalar renders from `renderText`,
 precomputed at creation via `jsValueToTomlText` so wrapped inputs (forced
 integer/float, datetimes) keep their intended spelling.
 */
export type ScalarNode = {
  readonly kind: 'scalar';
  readonly tomlKind: ScalarKind;
  readonly jsValue: unknown;
  readonly origin: Origin;
  readonly renderText?: string;
};

/**
 An array value. `elements` are child value nodes in order.
 */
export type ArrayNode = {
  readonly kind: 'array';
  readonly elements: readonly ValueNode[];
  readonly origin: Origin;
};

/**
 An inline-table value. `entries` are its key-values in order.
 */
export type InlineTableNode = {
  readonly kind: 'inline-table';
  readonly entries: readonly KeyValueNode[];
  readonly origin: Origin;
};

/**
 A key-value entry (top-level, inside a table body, or inside an inline table).
 
 `keySegments` is the flattened dotted-key chain (`['a','b']` for `a.b = 1`).
 `origin.range` (when clean) is the whole physical line span (key through the
 trailing newline, absorbing a same-line comment). `valueRange` is the
 original value's span, so a value-only edit re-renders as
 `source.slice(lineStart, valueStart)` + new value +
 `source.slice(valueEnd, lineEnd)`, keeping key spelling and the trailing
 comment byte-exact. `commentsBefore` / `commentAfter` carry attached comment
 text (without the leading `#`); for clean nodes the actual comment bytes live
 in surrounding fillers, so these feed reads and synthetic rendering only.
 */
export type KeyValueNode = {
  readonly kind: 'keyvalue';
  readonly keySegments: readonly string[];
  readonly value: ValueNode;
  readonly origin: Origin;
  readonly valueRange?: Span;
  readonly commentsBefore: readonly string[];
  readonly commentAfter?: string;
  readonly trailingCommentAppend?: string;
};

/**
 A standard `[foo]` or array `[[foo]]` table section with its own ordered
 body of blocks (key-values and fillers).
 
 `headerSegments` is the resolved header path minus any trailing array index;
 `aotIndex` is that index for an array-of-tables instance.
 */
export type TableNode = {
  readonly kind: 'table';
  readonly tableKind: 'standard' | 'array';
  readonly headerSegments: readonly (string | number)[];
  readonly aotIndex?: number;
  readonly headerOrigin: Origin;
  readonly body: readonly Block[];
  readonly commentsBefore: readonly string[];
  readonly commentAfter?: string;
};

/**
 Verbatim source span preserved between entries: blank lines, standalone
 comment lines, indentation, and the document prologue/epilogue.
 */
export type FillerBlock = {
  readonly kind: 'filler';
  readonly text: string;
};

/**
 One block in an ordered block list (top-level or a table body).
 */
export type Block =
  | FillerBlock
    | KeyValueNode
    | TableNode;
