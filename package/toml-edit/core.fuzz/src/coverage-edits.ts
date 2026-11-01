/**
 Edit-machinery reachability sweeps for the toml-edit fuzz coverage gate.
 
 Imports the package implementation from source (`../index.ts`) so a run under
 `NODE_V8_COVERAGE` attributes coverage to the `src` files the gate measures.
 A deterministic literal edit sequence reaches the editor, path-create, merge,
 array-of-tables, collision, and delete machinery; a value-encoding sweep
 exercises the from-scratch encoders over every generated value shape.
 
 @module
 */

import {
  emptyTomlEdit,
  parseTomlEdit,
  type TomlEditState,
  type TomlPath,
  tomlDelete,
  tomlSet,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core/ts';

import {
  attempt,
  attemptValue,
} from './coverage-harness.ts';

/**
 One step in the deterministic edit sequence.
 */
type EditStep =
  | {
    readonly kind: 'set';
    readonly path: TomlPath;
    readonly value: unknown
  }
    | {
      readonly kind: 'delete';
      readonly path: TomlPath
    };

/**
 Deterministic edit sequence reaching path-create, dotted-parent merge,
 array-of-tables, numeric-index, scalar-over-table collision, and delete paths.
 Literal values keep the sequence seed-independent and inside the exempt
 numeric range. Some steps throw on the edges tracked in #252; those rejections
 are caught and counted.
 */
const EDIT_SEQUENCE: readonly EditStep[] = [
  {
    kind: 'set',
    path: ['s',],
    value: 1,
  },
  {
    kind: 'set',
    path: ['s',],
    value: {
      x: 1,
      y: 2,
    },
  },
  {
    kind: 'set',
    path: [
      'x',
      'y',
    ],
    value: 'leaf',
  },
  {
    kind: 'set',
    path: [
      'x',
      'z',
    ],
    value: true,
  },
  {
    kind: 'set',
    path: ['aot',],
    value: [
      { id: 1, },
      { id: 2, },
    ],
  },
  {
    kind: 'set',
    path: ['arr',],
    value: [
      1,
      2,
    ],
  },
  {
    kind: 'set',
    path: [
      'arr',
      0,
    ],
    value: 2,
  },
  {
    kind: 'delete',
    path: [
      'x',
      'y',
    ],
  },
  {
    kind: 'delete',
    path: ['aot',],
  },
  {
    kind: 'delete',
    path: ['s',],
  },
];

/**
 Apply one edit step, materializing by reparse so the next step resolves
 against a real document. A by-design rejection leaves the state unchanged.
 
 
 
 @returns Next state, or the input state when the step was rejected.
 */
function applyStep({
  edit,
  step,
}: {
  readonly edit: TomlEditState;
  readonly step: EditStep
},): TomlEditState {
  return attemptValue({
    fallback: edit,
    thunk: function step2(): TomlEditState {
      /**
       State after the set or delete, before reparse materialization.
       */
      const next = step.kind === 'set'
        ? tomlSet({
          edit,
          path: step.path,
          value: step.value,
        },)
        : tomlDelete({
          edit,
          path: step.path,
        },);
      return parseTomlEdit({ source: tomlStringify({ edit: next, },), },);
    },
  },);
}

/**
 Run the deterministic edit sequence from `base`, reaching the editor, AOT,
 path-create, collision, and delete machinery.
 
 @example
 ```ts
 exerciseEditSequence({ base: emptyTomlEdit(), },);
 ```
 */
export function exerciseEditSequence({ base, }: { readonly base: TomlEditState; },): void {
  EDIT_SEQUENCE.reduce(
    /**
     Applies one deterministic sequence step.
     
     @param current - Edit state from prior step.
     
     @param editStep - Current deterministic edit operation.
     
     @returns Edit state after current operation.
     
     
     */
    function step(
      current,
      editStep,
    ): TomlEditState {
      return applyStep({
        edit: current,
        step: editStep,
      },);
    },
    base,
  );
}

/**
 Set each supplied value at a fixed key on a fresh empty document and stringify,
 exercising the from-scratch value and key encoders over every value shape.
 
 @example
 ```ts
 exerciseValueEncoding({ values: [ 1, 'a', { x: 1, }, ], },);
 ```
 
 */
export function exerciseValueEncoding({ values, }: { readonly values: readonly unknown[]; },): void {
  for (const value of values) {
    attempt({
      thunk: function once() {
        tomlStringify({ edit: tomlSet({
          edit: emptyTomlEdit(),
          path: ['probe',],
          value,
        },), },);
      },
    },);
  }
}
