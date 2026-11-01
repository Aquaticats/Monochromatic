import {
  CALL_ARGUMENTS,
  heldAt,
  IMPORT_OBJECT,
  objectAt,
  objectCase,
  THIRD,
  USER,
} from './claude-schema-forwarder-object-support.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A forwarder written as a member of an object literal that another file reads
// The member held as a value, the object handed on whole, a member taken out by a pattern or read by a string key,
// and the files that call the member by name (`claude-schema-forwarder-object-support.test-fixture.ts`).

/**
 Cases of a forwarder written as a member of an object literal.
 */
export const FORWARDER_OBJECT_CASES: readonly ShapeCase[] = [
  objectCase({
    name: 'REPORTS A METHOD THAT MAPS A LIST WHEN ANOTHER FILE HOLDS IT AS A VALUE, since the caller of the held value '
      + 'is not listed',
    member: 'method',
    users: [
      {
        file: USER,
        lines: [
          IMPORT_OBJECT,
          'export const held = [o.run,];',
        ],
      },
    ],
    listed: [USER,],
    spots: [heldAt({
      file: USER,
      line: 2,
    },),],
  },),
  objectCase({
    name: 'REPORTS A METHOD THAT SENDS DIRECTLY WHEN ANOTHER FILE HOLDS IT AS A VALUE',
    member: 'direct',
    users: [
      {
        file: USER,
        lines: [
          IMPORT_OBJECT,
          'export const held = [o.run,];',
        ],
      },
    ],
    listed: [USER,],
    spots: [heldAt({
      file: USER,
      line: 2,
    },),],
  },),
  objectCase({
    name: 'REPORTS A PROPERTY HOLDING AN ARROW THAT MAPS A LIST WHEN ANOTHER FILE HOLDS IT AS A VALUE',
    member: 'arrow',
    users: [
      {
        file: USER,
        lines: [
          IMPORT_OBJECT,
          'export const held = [o.run,];',
        ],
      },
    ],
    listed: [USER,],
    spots: [heldAt({
      file: USER,
      line: 2,
    },),],
  },),
  objectCase({
    name: 'REPORTS THE FILE THAT HOLDS A METHOD EVEN WHEN A THIRD FILE CALLS THE HELD VALUE, and lists the third file '
      + 'nowhere',
    member: 'method',
    users: [
      {
        file: USER,
        lines: [
          IMPORT_OBJECT,
          'export const held = [o.run,];',
        ],
      },
      {
        file: THIRD,
        lines: [
          'import { held, } from \'./object-user.ts\';',
          `export const sent = held[0](${CALL_ARGUMENTS},);`,
        ],
      },
    ],
    listed: [USER,],
    spots: [heldAt({
      file: USER,
      line: 2,
    },),],
  },),
  objectCase({
    name: 'REPORTS A METHOD THAT SENDS DIRECTLY WHEN A THIRD FILE CALLS THE VALUE ANOTHER FILE HOLDS',
    member: 'direct',
    users: [
      {
        file: USER,
        lines: [
          IMPORT_OBJECT,
          'export const held = [o.run,];',
        ],
      },
      {
        file: THIRD,
        lines: [
          'import { held, } from \'./object-user.ts\';',
          `export const sent = held[0](${CALL_ARGUMENTS}[0],);`,
        ],
      },
    ],
    listed: [USER,],
    spots: [heldAt({
      file: USER,
      line: 2,
    },),],
  },),
  objectCase({
    name: 'REPORTS A METHOD TAKEN OUT OF THE OBJECT BY A PATTERN AND HELD',
    member: 'method',
    users: [
      {
        file: USER,
        lines: [
          IMPORT_OBJECT,
          'const { run, } = o;',
          'export const held = [run,];',
        ],
      },
    ],
    listed: [USER,],
    spots: [heldAt({
      file: USER,
      line: 3,
    },),],
  },),
  objectCase({
    name: 'REPORTS A METHOD READ BY A STRING KEY AND HELD',
    member: 'method',
    users: [
      {
        file: USER,
        lines: [
          IMPORT_OBJECT,
          'export const held = [o[\'run\'],];',
        ],
      },
    ],
    listed: [USER,],
    spots: [heldAt({
      file: USER,
      line: 2,
    },),],
  },),
  objectCase({
    name: 'REPORTS THE OBJECT HANDED TO A CALL THAT LISTS ITS VALUES',
    member: 'method',
    users: [
      {
        file: USER,
        lines: [
          IMPORT_OBJECT,
          'export const all = Object.values(o,);',
        ],
      },
    ],
    listed: [],
    spots: [objectAt({
      file: USER,
      line: 2,
      position: 'CallExpression.arguments',
    },),],
  },),
  objectCase({
    name: 'REPORTS THE OBJECT HELD BY AN ELEMENT OF AN ARRAY',
    member: 'method',
    users: [
      {
        file: USER,
        lines: [
          IMPORT_OBJECT,
          'export const held = [o,];',
        ],
      },
    ],
    listed: [],
    spots: [objectAt({
      file: USER,
      line: 2,
      position: 'ArrayExpression.elements',
    },),],
  },),
  objectCase({
    name: 'LISTS THE FILE THAT CALLS A METHOD BY NAME, written as a dot or as a string key',
    member: 'method',
    users: [
      {
        file: USER,
        lines: [
          IMPORT_OBJECT,
          `export const first = o.run(${CALL_ARGUMENTS},);`,
          `export const second = o['run'](${CALL_ARGUMENTS},);`,
        ],
      },
    ],
    listed: [USER,],
    spots: [],
  },),
];

//endregion A forwarder written as a member of an object literal that another file reads
