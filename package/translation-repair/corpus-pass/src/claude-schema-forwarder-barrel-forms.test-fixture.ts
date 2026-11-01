import {
  ARGUMENTS,
  BARREL,
  OUTER,
  pairsOf,
  type Row,
  SECOND,
  THIRD,
  USER,
} from './claude-schema-forwarder-barrel-pairs.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A module that declares only forwarders, reached through a barrel
// The forms of a barrel: it exports everything of the module or names its members, it exports the module as a
// namespace, it stands behind another barrel, and the namespace it gives is kept by a constant, held by a member or
// handed to a call. Each is a pair of cases (`claude-schema-forwarder-barrel-pairs.test-fixture.ts`).

/**
 The import of the barrel's namespace that the user file starts with.
 */
const IMPORT_OUTER = 'import * as outer from \'./barrel-index.ts\';';

/**
 The import of the namespace through the barrel of a barrel.
 */
const IMPORT_TWO_HOPS = 'import * as outer from \'./barrel-outer.ts\';';

/**
 The line of a barrel that exports everything of the module.
 */
const EXPORT_ALL = 'export * from \'./barrel-stage.ts\';';

/**
 The line that holds the member of the namespace by a string key.
 */
const HOLD_BY_KEY = 'export const held = [outer[\'mk\'],];';

/**
 Forms of a barrel.
 */
const BARREL_ROWS: readonly Row[] = [
  {
    form: 'a barrel that exports everything of it, a member held by a string key',
    files: [
      {
        file: BARREL,
        lines: [EXPORT_ALL,],
      },
      {
        file: USER,
        lines: [
          IMPORT_OUTER,
          HOLD_BY_KEY,
        ],
      },
    ],
    listed: [USER,],
    places: [
      {
        file: USER,
        line: 2,
        kind: 'held',
        position: 'ArrayExpression.elements',
      },
    ],
  },
  {
    form: 'a barrel that exports everything of it, the namespace handed to a call',
    files: [
      {
        file: BARREL,
        lines: [EXPORT_ALL,],
      },
      {
        file: USER,
        lines: [
          IMPORT_OUTER,
          'export const all = Object.values(outer,);',
        ],
      },
    ],
    listed: [],
    places: [
      {
        file: USER,
        line: 2,
        kind: 'module',
        position: 'CallExpression.arguments',
      },
    ],
  },
  {
    form: 'a barrel that exports it as a namespace, a member held by a string key',
    files: [
      {
        file: BARREL,
        lines: ['export * as inner from \'./barrel-stage.ts\';',],
      },
      {
        file: USER,
        lines: [
          IMPORT_OUTER,
          'export const held = [outer.inner[\'mk\'],];',
        ],
      },
    ],
    listed: [USER,],
    places: [
      {
        file: USER,
        line: 2,
        kind: 'held',
        position: 'ArrayExpression.elements',
      },
    ],
  },
  {
    form: 'a barrel namespace kept by an exported constant and a member held by a string key in a second file',
    files: [
      {
        file: BARREL,
        lines: [EXPORT_ALL,],
      },
      {
        file: USER,
        lines: [
          IMPORT_OUTER,
          'export const kept = outer;',
        ],
      },
      {
        file: SECOND,
        lines: [
          'import { kept, } from \'./barrel-user.ts\';',
          'export const held = [kept[\'mk\'],];',
        ],
      },
    ],
    listed: [SECOND,],
    places: [
      {
        file: SECOND,
        line: 2,
        kind: 'held',
        position: 'ArrayExpression.elements',
      },
    ],
  },
  {
    form: 'a barrel that names the member, the namespace handed to a call',
    files: [
      {
        file: BARREL,
        lines: ['export { mk, } from \'./barrel-stage.ts\';',],
      },
      {
        file: USER,
        lines: [
          IMPORT_OUTER,
          'export const all = Object.values(outer,);',
        ],
      },
    ],
    listed: [],
    places: [
      {
        file: USER,
        line: 2,
        kind: 'module',
        position: 'CallExpression.arguments',
      },
    ],
  },
  {
    form: 'a barrel that names the member, a member held by a string key',
    files: [
      {
        file: BARREL,
        lines: ['export { mk, } from \'./barrel-stage.ts\';',],
      },
      {
        file: USER,
        lines: [
          IMPORT_OUTER,
          HOLD_BY_KEY,
        ],
      },
    ],
    listed: [USER,],
    places: [
      {
        file: USER,
        line: 2,
        kind: 'held',
        position: 'ArrayExpression.elements',
      },
    ],
  },
  {
    form: 'a barrel of a barrel, a member held by a string key',
    files: [
      {
        file: BARREL,
        lines: [EXPORT_ALL,],
      },
      {
        file: OUTER,
        lines: ['export * from \'./barrel-index.ts\';',],
      },
      {
        file: USER,
        lines: [
          IMPORT_TWO_HOPS,
          HOLD_BY_KEY,
        ],
      },
    ],
    listed: [USER,],
    places: [
      {
        file: USER,
        line: 2,
        kind: 'held',
        position: 'ArrayExpression.elements',
      },
    ],
  },
  {
    form: 'a barrel that also declares a constant, a member held by a string key',
    files: [
      {
        file: BARREL,
        lines: [
          EXPORT_ALL,
          'export const plain = 1;',
        ],
      },
      {
        file: USER,
        lines: [
          IMPORT_OUTER,
          HOLD_BY_KEY,
        ],
      },
    ],
    listed: [USER,],
    places: [
      {
        file: USER,
        line: 2,
        kind: 'held',
        position: 'ArrayExpression.elements',
      },
    ],
  },
  {
    form: 'the namespace of a barrel handed to a call in one file and its first value called in a third',
    files: [
      {
        file: BARREL,
        lines: [EXPORT_ALL,],
      },
      {
        file: USER,
        lines: [
          IMPORT_OUTER,
          'export const values = Object.values(outer,);',
        ],
      },
      {
        file: THIRD,
        lines: [
          'import { values, } from \'./barrel-user.ts\';',
          `export const sent = (values[0] as any)(${ARGUMENTS},);`,
        ],
      },
    ],
    listed: [],
    places: [
      {
        file: USER,
        line: 2,
        kind: 'module',
        position: 'CallExpression.arguments',
      },
    ],
  },
  {
    form: 'an import-equals of the module, the namespace handed to a call',
    files: [
      {
        file: USER,
        lines: [
          'import ns = require(\'./barrel-stage.ts\');',
          'export const all = Object.values(ns,);',
        ],
      },
    ],
    listed: [],
    places: [
      {
        file: USER,
        line: 1,
        kind: 'path',
        position: 'TSExternalModuleReference.expression',
      },
      {
        file: USER,
        line: 2,
        kind: 'module',
        position: 'CallExpression.arguments',
      },
    ],
  },
];

/**
 Cases of a module that declares only forwarders, reached through a barrel, each with its twin.
 */
export const FORWARDER_BARREL_CASES: readonly ShapeCase[] = pairsOf({ rows: BARREL_ROWS, },);

//endregion A module that declares only forwarders, reached through a barrel
