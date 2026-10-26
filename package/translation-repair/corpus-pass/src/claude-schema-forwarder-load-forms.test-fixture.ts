import {
  ARGUMENTS,
  pairsOf,
  type Row,
  SECOND,
  USER,
} from './claude-schema-forwarder-barrel-pairs.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A module that declares only forwarders, reached through a dynamic import
// A dynamic import of such a module is read as a dynamic import of a module that carries a builder: it is followed
// where a constant takes the module whole or by a pattern, and reported in every other form. Each form is a pair of
// cases (`claude-schema-forwarder-barrel-pairs.test-fixture.ts`).

/**
 The dynamic import of the module, which the cases write on the line they need.
 */
const LOAD = 'import(\'./barrel-stage.ts\')';

/**
 The declaration of a function that takes anything.
 */
const DECLARE_TAKE = 'declare function take(value: unknown,): void;';

/**
 Forms of a dynamic import.
 */
const LOAD_ROWS: readonly Row[] = [
  {
    form: 'a dynamic import whose callback calls a member',
    files: [
      {
        file: USER,
        lines: [`export const sent = ${LOAD}.then((m,) => (m as any).mk(${ARGUMENTS},),);`,],
      },
    ],
    listed: [],
    places: [
      {
        file: USER,
        line: 1,
        kind: 'load',
        position: 'MemberExpression.object',
      },
    ],
  },
  {
    form: 'a dynamic import whose callback holds a member read by a dot',
    files: [
      {
        file: USER,
        lines: [`export const held = ${LOAD}.then((m,) => [m.mk,],);`,],
      },
    ],
    listed: [],
    places: [
      {
        file: USER,
        line: 1,
        kind: 'load',
        position: 'MemberExpression.object',
      },
    ],
  },
  {
    form: 'a dynamic import whose callback holds the namespace',
    files: [
      {
        file: USER,
        lines: [`export const held = ${LOAD}.then((m,) => [m,],);`,],
      },
    ],
    listed: [],
    places: [
      {
        file: USER,
        line: 1,
        kind: 'load',
        position: 'MemberExpression.object',
      },
    ],
  },
  {
    form: 'a dynamic import whose callback hands the namespace to a call',
    files: [
      {
        file: USER,
        lines: [
          DECLARE_TAKE,
          `export const handed = ${LOAD}.then((m,) => take(m,),);`,
        ],
      },
    ],
    listed: [],
    places: [
      {
        file: USER,
        line: 2,
        kind: 'load',
        position: 'MemberExpression.object',
      },
    ],
  },
  {
    form: 'a dynamic import whose callback holds a member read by a string key',
    files: [
      {
        file: USER,
        lines: [`export const held = ${LOAD}.then((m,) => [(m as any)['mk'],],);`,],
      },
    ],
    listed: [],
    places: [
      {
        file: USER,
        line: 1,
        kind: 'load',
        position: 'MemberExpression.object',
      },
    ],
  },
  {
    form: 'a dynamic import returned from a function',
    files: [
      {
        file: USER,
        lines: [`export const load = () => ${LOAD};`,],
      },
    ],
    listed: [],
    places: [
      {
        file: USER,
        line: 1,
        kind: 'load',
        position: 'ArrowFunctionExpression.body',
      },
    ],
  },
  {
    form: 'a dynamic import in a list handed to Promise.all',
    files: [
      {
        file: USER,
        lines: [`export const loaded = Promise.all([${LOAD},],);`,],
      },
    ],
    listed: [],
    places: [
      {
        file: USER,
        line: 1,
        kind: 'load',
        position: 'ArrayExpression.elements',
      },
    ],
  },
  {
    form: 'a dynamic import handed to a call',
    files: [
      {
        file: USER,
        lines: [
          DECLARE_TAKE,
          `take(${LOAD},);`,
        ],
      },
    ],
    listed: [],
    places: [
      {
        file: USER,
        line: 2,
        kind: 'load',
        position: 'CallExpression.arguments',
      },
    ],
  },
  {
    form: 'a dynamic import held by a constant and awaited in a function',
    files: [
      {
        file: USER,
        lines: [
          `const pending = ${LOAD};`,
          'export const held = async () => [((await pending) as any)[\'mk\'],];',
        ],
      },
    ],
    listed: [],
    places: [
      {
        file: USER,
        line: 1,
        kind: 'load',
        position: 'VariableDeclarator.init',
      },
    ],
  },
  {
    form: 'a dynamic import kept by an exported constant and a member called in then by a second file',
    files: [
      {
        file: USER,
        lines: [`export const pending = ${LOAD};`,],
      },
      {
        file: SECOND,
        lines: [
          'import { pending, } from \'./barrel-user.ts\';',
          `export const sent = pending.then((m,) => m.mk(${ARGUMENTS},),);`,
        ],
      },
    ],
    listed: [],
    places: [
      {
        file: USER,
        line: 1,
        kind: 'load',
        position: 'VariableDeclarator.init',
      },
    ],
  },
];

/**
 Cases of a module that declares only forwarders, reached through a dynamic import, each with its twin.
 */
export const FORWARDER_LOAD_CASES: readonly ShapeCase[] = pairsOf({ rows: LOAD_ROWS, },);

//endregion A module that declares only forwarders, reached through a dynamic import
