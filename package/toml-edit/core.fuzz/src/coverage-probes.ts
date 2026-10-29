/**
 Targeted reachability probes for the toml-edit fuzz coverage gate.
 
 Imports the package implementation from source (`../index.ts`) so a run under
 `NODE_V8_COVERAGE` attributes coverage to the `src` files the gate measures.
 These probes reach branches random documents rarely produce: parser edge cases
 (CRLF, bare carriage return, overflow), pending-insertion and pending-edit
 projection arms read before reparse, the existing-node-aware encoders, the
 from-scratch edit machinery, and the root-delete arm.
 
 @module
 */

import {
  emptyTomlEdit,
  parseTomlEdit,
  type TomlPath,
  tomlDelete,
  tomlGet,
  tomlGetValue,
  tomlFloat,
  tomlHas,
  tomlInteger,
  tomlKeys,
  tomlLocalDate,
  tomlLocalDateTime,
  tomlLocalTime,
  tomlSet,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core/ts';

import {
  attempt,
  tryParse,
} from './coverage-harness.ts';
import { exerciseEditSequence, } from './coverage-edits.ts';
import { exerciseValidSource, } from './coverage-exercise.ts';

//region Capture-free operation bodies

/**
 Inline-table depth that overflows the parser recursion (matches the parser
 property's pinned regression so the RangeError-wrapping branch is reached).
 */
const DEEP_INLINE_DEPTH = 3_000;

/**
 Array depth that overflows the parser recursion.
 */
const DEEP_ARRAY_DEPTH = 20_000;

/**
 Parser edge-case sources random documents rarely produce: CRLF (the
 normalization branch), a bare carriage return (the rejection branch), and
 pathologically deep array and inline-table nesting (the RangeError-wrapping
 branch). Numbers larger than the exempt range live in source text, not code.
 */
const PARSER_EDGE_SOURCES: readonly string[] = [
  'title = "crlf"\r\na = 1\r\n',
  'a = 1\rb = 2\n',
  `deep = ${'['.repeat(DEEP_ARRAY_DEPTH,)}${']'.repeat(DEEP_ARRAY_DEPTH,)}\n`,
  `deep = ${'{ b = '.repeat(DEEP_INLINE_DEPTH,)}1${' }'.repeat(DEEP_INLINE_DEPTH,)}\n`,
];

/**
 Delete the root path, reaching the top-level early-return arm of delete.
 */
function deleteRootOp(): void {
  tomlDelete({
    edit: emptyTomlEdit(),
    path: [],
  },);
}

/**
 Reach canonical final-newline handling for synthetic and parsed documents.
 */
function canonicalNewlineOps(): void {
  tomlStringify({ edit: tomlSet({
    edit: emptyTomlEdit({ canonical: { trailingNewline: false, }, },),
    path: ['x',],
    value: 1,
  },), },);
  tomlStringify({ edit: parseTomlEdit({
    source: 'x = 1',
    mode: 'canonical',
  },), },);
  tomlStringify({ edit: parseTomlEdit({
    source: 'x = 1\n',
    mode: 'canonical',
    canonical: { trailingNewline: false, },
  },), },);
}

/**
 Set two sibling pending insertions under an implicit parent, then read
 sub-paths on the same non-materialized state, reaching the pending-insertion
 projection arms of the effective resolver.
 */
function pendingReadOps(): void {
  /**
   Two sibling pending insertions under `p`, left unreparsed so reads resolve
   against the pending insertion list.
   */
  const edit = tomlSet({
    edit: tomlSet({
      edit: emptyTomlEdit(),
      path: [
        'p',
        'q',
      ],
      value: 1,
    },),
    path: [
      'p',
      'r',
    ],
    value: 'two',
  },);
  tomlHas({
    edit,
    path: ['p',],
  },);
  tomlHas({
    edit,
    path: [
      'p',
      'q',
    ],
  },);
  tomlGet({
    edit,
    path: [
      'p',
      'q',
    ],
  },);
  tomlGetValue({
    edit,
    path: ['p',],
  },);
  tomlGetValue({
    edit,
    path: [
      'p',
      'q',
    ],
  },);
  tomlKeys({
    edit,
    path: ['p',],
  },);
  tomlGetValue({
    edit,
    path: [],
  },);
}

/**
 Read sub-paths under pending edits and deletions on existing parsed nodes (no
 reparse), reaching the prefix-projection arms that navigate a pending
 `jsValue` through objects, arrays, and missing or deleted shapes.
 */
function pendingProjectionOps(): void {
  /**
   Parsed base with an existing inline table and array to edit and delete.
   */
  const base = parseTomlEdit({ source: 't = { a = 1, b = 2 }\narr = [ 1, 2 ]\nx = 0\n', },);
  /**
   Pending edit replacing the existing table node with a nested object.
   */
  const editedTable = tomlSet({
    edit: base,
    path: ['t',],
    value: {
      a: 2,
      nested: { deep: 1, },
    },
  },);
  tomlGetValue({
    edit: editedTable,
    path: [
      't',
      'a',
    ],
  },);
  tomlGetValue({
    edit: editedTable,
    path: [
      't',
      'nested',
      'deep',
    ],
  },);
  tomlHas({
    edit: editedTable,
    path: [
      't',
      'nested',
    ],
  },);
  tomlGetValue({
    edit: editedTable,
    path: [
      't',
      'missing',
    ],
  },);
  tomlKeys({
    edit: editedTable,
    path: ['t',],
  },);
  /**
   Pending edit replacing the existing array node, read by numeric and
   non-numeric sub-paths to reach the array and missing navigation arms.
   */
  const editedArray = tomlSet({
    edit: base,
    path: ['arr',],
    value: [
      1,
      2,
    ],
  },);
  tomlGetValue({
    edit: editedArray,
    path: [
      'arr',
      0,
    ],
  },);
  tomlGetValue({
    edit: editedArray,
    path: [
      'arr',
      2,
    ],
  },);
  tomlGetValue({
    edit: editedArray,
    path: [
      'arr',
      'k',
    ],
  },);
  /**
   Pending deletion on the existing table node, read by sub-path to reach the
   deleted-prefix arm.
   */
  const deletedTable = tomlDelete({
    edit: base,
    path: ['t',],
  },);
  tomlGetValue({
    edit: deletedTable,
    path: [
      't',
      'a',
    ],
  },);
  tomlHas({
    edit: deletedTable,
    path: [
      't',
      'a',
    ],
  },);
}

/**
 Reach the public tagged-value factories independently of edit operations.
 */
function wrapperOps(): void {
  tomlInteger(1,);
  tomlFloat(1,);
  tomlLocalDate('2026-05-14',);
  tomlLocalDateTime('2026-05-14T10:00:00',);
  tomlLocalTime('10:00:00',);
}

//endregion Capture-free operation bodies

//region Probe sweeps

/**
 Run the empty-document edit sequence and root delete, covering `emptyTomlEdit`
 and the from-scratch edit machinery independent of any parsed corpus.
 
 @example
 ```ts
 exerciseEmptyBase();
 ```
 */
export function exerciseEmptyBase(): void {
  exerciseEditSequence({ base: emptyTomlEdit(), },);
  attempt({ thunk: deleteRootOp, },);
  canonicalNewlineOps();
  wrapperOps();
}

/**
 Drive the parser edge-case sources through the valid-source spread, reaching
 the CRLF normalization, bare-CR rejection, and overflow branches.
 
 @example
 ```ts
 exerciseParserEdges();
 ```
 */
export function exerciseParserEdges(): void {
  for (const source of PARSER_EDGE_SOURCES) {
    exerciseValidSource({ source, },);
  }
}

/**
 Read sub-paths on a non-materialized state with pending insertions, reaching
 the pending-insertion projection arms of the effective resolver.
 
 @example
 ```ts
 exercisePendingReads();
 ```
 */
export function exercisePendingReads(): void {
  attempt({ thunk: pendingReadOps, },);
}

/**
 Read sub-paths under pending edits and deletions on existing parsed nodes,
 reaching the prefix-projection navigation arms.
 
 @example
 ```ts
 exercisePendingProjections();
 ```
 */
export function exercisePendingProjections(): void {
  attempt({ thunk: pendingProjectionOps, },);
}

/**
 Re-set each existing top-level value back to itself on a parsed document,
 reaching the existing-node-aware string and number encoders that preserve the
 parse-time spelling.
 
 @example
 ```ts
 exerciseExistingResets({ source: 'a = "x"\n', },);
 ```
 */
export function exerciseExistingResets({ source, }: { readonly source: string; },): void {
  /**
   Splice-mode parse whose existing nodes drive the encoders.
   */
  const parsed = tryParse({
    source,
    mode: 'splice',
  },);
  if (!parsed.ok) return;
  attempt({
    thunk: function resets() {
      for (const key of tomlKeys({ edit: parsed.edit, },)) {
        /**
         Single-segment path for the current top-level key.
         */
        const path: TomlPath = [key,];
        /**
         Current value, re-set unchanged so the encoder takes the existing arm.
         */
        const value = tomlGetValue({
          edit: parsed.edit,
          path,
        },);
        if (value === undefined) continue;
        tomlStringify({ edit: tomlSet({
          edit: parsed.edit,
          path,
          value,
        },), },);
      }
    },
  },);
}

//endregion Probe sweeps
