/**
 Stateful edit-model property: random `tomlSet` / `tomlDelete` sequences over
 `emptyTomlEdit`, applied directly to the accumulating state (no reparse
 between operations), checked against an in-memory nested model.
 
 A rejected operation must throw a `TomlEditError` and leave both the state
 and the model unchanged. A successful scalar set must make the effective read
 at its path equal the written value; a delete must make it `undefined`. After
 the whole sequence, `tomlStringify` output must reparse to a document
 semantically equal to the model.
 
 Paths are one to three segments over a tiny alphabet, so implicit dotted-key
 parents, overwrites, and type changes arise often. Because the document tree
 is the single source of truth, deltas no longer need to be flushed by a
 reparse between operations (issue #252).
 
 @module
 */

import {
  type Arbitrary,
  array,
  assert,
  asyncProperty,
  boolean,
  constantFrom,
  double,
  integer,
  oneof,
  string,
  tuple,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  emptyTomlEdit,
  parseTomlEdit,
  type TomlEditState,
  TomlEditError,
  tomlDelete,
  tomlGetValue,
  tomlSet,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core';

import { fuzzRunPlan, } from './fuzz-budget.ts';
import {
  semanticEquals,
  semanticModel,
} from './equality.ts';

//region Arbitraries

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Maximum operations in one generated sequence.
 */
const MAX_OPS = 14;

/**
 Maximum path depth so implicit dotted-key parents arise without exploding.
 */
const MAX_DEPTH = 3;

/**
 Scalar value arbitrary over the TOML-representable JS scalars (finite floats
 only; the special float and datetime spellings are covered elsewhere).
 */
const scalarValueArbitrary = oneof(
  string(),
  integer(),
  boolean(),
  double({
    noNaN: true,
    noDefaultInfinity: true,
  },),
);

/**
 Value arbitrary: scalars and arrays of scalars.
 
 Object values are intentionally excluded here. They would create inline
 tables, whose empty-after-delete behavior (`c = {}` persists) differs from an
 implicit dotted-key parent (which vanishes when its last child is deleted) in
 a way the plain nested oracle cannot distinguish without tracking physical
 representation. Inline-table set/delete and whole-table replace are covered by
 the deterministic `issue-252` and `tomlSet`/`tomlDelete` unit tests instead.
 */
const valueArbitrary: Arbitrary<unknown> = oneof(
  scalarValueArbitrary,
  array(scalarValueArbitrary, { maxLength: 3, },),
);

/**
 Path arbitrary: one to three bare segments over a tiny alphabet, exercising
 implicit dotted-key parents and deep edits.
 */
const pathArbitrary: Arbitrary<readonly string[]> = array(
  constantFrom('a', 'b', 'c',),
  {
    minLength: 1,
    maxLength: MAX_DEPTH,
  },
);

/**
 One edit operation: a set with a value, or a delete.
 */
type EditOp =
  | { readonly kind: 'set'; readonly path: readonly string[]; readonly value: unknown; }
    | { readonly kind: 'delete'; readonly path: readonly string[]; };

/**
 Operation arbitrary.
 */
const opArbitrary: Arbitrary<EditOp> = oneof(
  tuple(pathArbitrary, valueArbitrary,).map(function set([path, value,],) {
    return {
      kind: 'set',
      path,
      value,
    } as EditOp;
  },),
  pathArbitrary.map(function del(path,) {
    return {
      kind: 'delete',
      path,
    } as EditOp;
  },),
);

/**
 Sequence arbitrary.
 */
const opsArbitrary: Arbitrary<readonly EditOp[]> = array(opArbitrary, { maxLength: MAX_OPS, },);

//endregion Arbitraries

//region Model helpers

/**
 Mutable nested model node.
 */
type ModelTree = Record<string, unknown>;

/**
 Descend (creating) the intermediate tables named by `segments`, returning the
 container the final segment should be written into.
 
 @returns Deepest intermediate table.
 */
function descendModel(
  {
    tree,
    segments,
  }: {
    readonly tree: ModelTree;
    readonly segments: readonly string[];
  },
): ModelTree {
  /**
   Cursor descending into (and creating) each intermediate table.
   */
  let cursor = tree;
  for (const seg of segments) {
    /**
     Existing child; replaced with a fresh object when not a plain object.
     */
    const existing = cursor[seg];
    if ((existing === null) || ((typeof existing) !== 'object') || Array.isArray(existing,)) {
      /**
       Fresh intermediate table so the descent can continue.
       */
      const fresh: ModelTree = {};
      cursor[seg] = fresh;
      cursor = fresh;
      continue;
    }
    cursor = existing as ModelTree;
  }
  return cursor;
}

/**
 Set `value` at the nested `path` within `tree`, creating intermediate
 objects. Mirrors a successful `tomlSet` (the API rejects the cases that would
 make this diverge, leaving the model untouched).
 
 @returns Nothing; mutates `tree` in place.
 */
function modelSet(
  {
    tree,
    path,
    value,
  }: {
    readonly tree: ModelTree;
    readonly path: readonly string[];
    readonly value: unknown;
  },
): void {
  /**
   Container the final segment is written into.
   */
  const parent = descendModel({
    tree,
    segments: path.slice(
      0,
      -1,
    ),
  },);
  parent[path.at(-1) ?? ''] = structuredClone(value,);
}

/**
 Delete the nested `path` from `tree`, then prune ancestors that became empty
 (an implicit dotted-key parent with no remaining children has no TOML
 representation and vanishes). A no-op when a segment is absent.
 
 @returns Nothing; mutates `tree` in place.
 */
function modelDelete(
  {
    tree,
    path,
  }: {
    readonly tree: ModelTree;
    readonly path: readonly string[];
  },
): void {
  /**
   Objects along the path, root first; `cursors[k]` is the object at
   `path[0..k-1]`, so `cursors.at(-1)` is the deleted leaf's parent.
   */
  const cursors: ModelTree[] = [tree,];
  for (const seg of path.slice(
    0,
    -1,
  )) {
    /**
     Next object down the path, or a bail-out when the segment is absent.
     */
    const next = cursors.at(-1)?.[seg];
    if ((next === null) || ((typeof next) !== 'object') || Array.isArray(next,))
      return;
    cursors.push(next as ModelTree,);
  }
  Reflect.deleteProperty(
    cursors.at(-1) ?? tree,
    path.at(-1) ?? '',
  );
  for (let depth = cursors.length - 1; depth >= 1; depth--) {
    if (Object.keys(cursors[depth] ?? {},).length
      > 0)
      break;
    Reflect.deleteProperty(
      cursors[depth - 1] ?? tree,
      path[depth - 1] ?? '',
    );
  }
}

//endregion Model helpers

//region Operation application

/**
 Apply one operation to the live state and the model, asserting invariants.
 
 @returns Next state (unchanged when the operation was rejected).
 */
function applyOp(
  {
    edit,
    op,
    tree,
  }: {
    readonly edit: TomlEditState;
    readonly op: EditOp;
    readonly tree: ModelTree;
  },
): TomlEditState {
  try {
    if (op.kind === 'set') {
      /**
       State after the set, computed before the model is updated to match.
       */
      const next = tomlSet({
        edit,
        path: op.path,
        value: op.value,
      },);
      modelSet({
        tree,
        path: op.path,
        value: op.value,
      },);
      // Whole-path read-back is well-defined for scalars; object and array
      // whole-reads are validated through the final whole-document check.
      if (((typeof op.value) !== 'object') || (op.value === null)) {
        expect(
          semanticEquals({
            left: tomlGetValue({
              edit: next,
              path: op.path,
            },) as never,
            right: op.value as never,
          },),
        ).toBe(true,);
      }
      return next;
    }
    /**
     State after the delete.
     */
    const next = tomlDelete({
      edit,
      path: op.path,
    },);
    modelDelete({
      tree,
      path: op.path,
    },);
    expect(tomlGetValue({
      edit: next,
      path: op.path,
    },),).toBe(undefined,);
    return next;
  }
  catch (caught: unknown) {
    expect(caught,).toBeInstanceOf(TomlEditError,);
    return edit;
  }
}

//endregion Operation application

await describe({
  name: 'stateful edit model',
  children: [
    it({
      name: 'set/delete sequences keep effective reads and stringify in sync with the model',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(opsArbitrary, async function sequence(ops,) {
            /**
             Predicted effective document.
             */
            const tree: ModelTree = {};
            /**
             Live state, mutated directly across operations with no reparse.
             */
            const edit = ops.reduce(
              function step(current, op,) {
                return applyOp({
                  edit: current,
                  op,
                  tree,
                },);
              },
              emptyTomlEdit(),
            );
            /**
             Final serialization, which must reparse to the model.
             */
            const text = tomlStringify({ edit, },);
            expect(
              semanticEquals({
                left: semanticModel({ source: text, },),
                right: tree as never,
              },),
            ).toBe(true,);
          },),
          RUN.params,
        );
      },
    },),

    it({
      name: 'repeating a successful set on the reparsed result is byte-identical',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(
            pathArbitrary,
            valueArbitrary,
            async function idempotent(path, value,) {
              /**
               Document after the first set of an empty base.
               */
              const once = tomlStringify({
                edit: tomlSet({
                  edit: emptyTomlEdit(),
                  path,
                  value,
                },),
              },);
              /**
               Document after reparsing and repeating the identical set.
               */
              const twice = tomlStringify({
                edit: tomlSet({
                  edit: parseTomlEdit({ source: once, },),
                  path,
                  value,
                },),
              },);
              expect(twice,).toBe(once,);
            },
          ),
          RUN.params,
        );
      },
    },),
  ],
},);
