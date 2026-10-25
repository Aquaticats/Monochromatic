import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  positionSpotAt,
  POS_ARGUMENTS,
  USER,
  userCase,
} from './claude-schema-forwarder-position-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A forwarder reached through the namespace of a module that declares forwarders and no builder
// A file that imports such a module as a namespace reaches its forwarders with a member written as a name, as a string
// or as a template, under a type mark, or computed from a name, and it can hand the namespace on whole, so the
// listing of callers does not see the files that call the forwarder afterwards. The module is read like a module that
// carries a builder: a member that names a forwarder and is no callee is reported where it sits, a member computed from
// a name and the namespace written as a whole value are reported as a module that is read where the search does not
// look, and a call of a member written as a string is read.

/**
 The import of the namespace that the user file starts with.
 */
const IMPORT_NAMESPACE = 'import * as ns from \'./pos-stage.ts\';';

/**
 The report for the namespace of the stage file read in a place the search does not look, in the user file.

 @param line - line of the place

 @param position - kind of the node that holds the namespace and the field

 @returns Report

 @example
 ```ts
 const spot = moduleSpotAt({ line: 2, position: 'CallExpression.arguments', },);
 ```
 */
function moduleSpotAt(
  {
    line,
    position,
  }: {
    readonly line: number;
    readonly position: string;
  },
): BlindSpot {
  return {
    file: USER,
    line,
    shape: 'module-binding-outside-a-read-position',
    position,
  };
}

/**
 Cases of a forwarder reached through a namespace.
 */
export const FORWARDER_NAMESPACE_CASES: readonly ShapeCase[] = [
  userCase({
    name: 'REPORTS A FORWARDER READ OFF A NAMESPACE WITH A STRING KEY AND HELD, since the caller of the holder is not '
      + 'listed',
    lines: [
      IMPORT_NAMESPACE,
      'export const held = [ns[\'mk\'],];',
    ],
    spots: [positionSpotAt({
      line: 2,
      position: 'ArrayExpression.elements',
    },),],
    isListed: true,
  },),
  userCase({
    name: 'REPORTS A FORWARDER READ OFF A NAMESPACE WITH A TEMPLATE KEY AND HELD',
    lines: [
      IMPORT_NAMESPACE,
      'export const held = [ns[`mk`],];',
    ],
    spots: [positionSpotAt({
      line: 2,
      position: 'ArrayExpression.elements',
    },),],
    isListed: true,
  },),
  userCase({
    name: 'REPORTS A NAMESPACE UNDER A TYPE MARK AND THE FORWARDER READ OFF IT, the forwarder where it is held and the '
      + 'module where it is the object of the member',
    lines: [
      IMPORT_NAMESPACE,
      'export const held = [(ns as any)[\'mk\'],];',
    ],
    spots: [
      positionSpotAt({
        line: 2,
        position: 'ArrayExpression.elements',
      },),
      moduleSpotAt({
        line: 2,
        position: 'MemberExpression.object',
      },),
    ],
    isListed: true,
  },),
  userCase({
    name: 'REPORTS A MEMBER OF A NAMESPACE COMPUTED FROM A NAME, which can be any forwarder of the module',
    lines: [
      IMPORT_NAMESPACE,
      'export const read = (key: string,) => (ns as any)[key];',
    ],
    spots: [moduleSpotAt({
      line: 2,
      position: 'MemberExpression.object',
    },),],
    isListed: false,
  },),
  userCase({
    name: 'REPORTS A NAMESPACE HANDED TO A CALL THAT LISTS ITS MEMBERS',
    lines: [
      IMPORT_NAMESPACE,
      'export const all = Object.values(ns,);',
    ],
    spots: [moduleSpotAt({
      line: 2,
      position: 'CallExpression.arguments',
    },),],
    isListed: false,
  },),
  userCase({
    name: 'REPORTS A NAMESPACE HELD BY AN ELEMENT OF AN ARRAY',
    lines: [
      IMPORT_NAMESPACE,
      'export const held = [ns,];',
    ],
    spots: [moduleSpotAt({
      line: 2,
      position: 'ArrayExpression.elements',
    },),],
    isListed: false,
  },),
  userCase({
    name: 'REPORTS A NAMESPACE HELD BY A PROPERTY OF AN OBJECT',
    lines: [
      IMPORT_NAMESPACE,
      'export const held = { ns, };',
    ],
    spots: [moduleSpotAt({
      line: 2,
      position: 'Property.value',
    },),],
    isListed: false,
  },),
  userCase({
    name: 'ACCEPTS A FORWARDER CALLED THROUGH A NAMESPACE, with a member written as a name or as a string',
    lines: [
      IMPORT_NAMESPACE,
      `export const first = ns.mk(${POS_ARGUMENTS},);`,
      `export const second = ns['mk'](${POS_ARGUMENTS},);`,
    ],
    spots: [],
    isListed: true,
  },),
  {
    name: 'ACCEPTS THE NAMESPACE OF A MODULE THAT DECLARES NO FORWARDER AND NO BUILDER, handed on whole',
    files: [
      {
        file: 'imaginary/src/pos-plain.ts',
        text: textOf(['export const plain = 1;',],),
      },
      {
        file: USER,
        text: textOf([
          'import * as ns from \'./pos-plain.ts\';',
          'export const held = [ns,];',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [],
  },
];

//endregion A forwarder reached through the namespace of a module that declares forwarders and no builder
