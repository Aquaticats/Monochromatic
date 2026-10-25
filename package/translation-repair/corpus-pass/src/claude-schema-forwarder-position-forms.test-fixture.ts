import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { CONTEST_IMPORT, } from './claude-schema-destination-forms.test-fixture.ts';
import {
  SENDER_FILE,
  SENDER_IMPORT,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';
import { keyOf, } from './claude-schema-usage.test-fixture.ts';

//region The places a forwarder is read that the first cases of its position did not separate
// A forwarder is read as the callee of a call, of `new` or of a tagged template, and as a function bound by `bind` that
// is called at once. A bound function that is kept, a member of it that is no call method, and a free name that is the
// name of a forwarder held in an object are reported where they sit, as the other places are.

/**
 The path of the file that declares the forwarder in every case.
 */
const STAGE = 'imaginary/src/pos-stage.ts';

/**
 The path of the file that reads the forwarder in the form a case names.
 */
const USER = 'imaginary/src/pos-user.ts';

/**
 The arguments of a call of the forwarder as the user file writes them.
 */
const POS_ARGUMENTS = '{ schemaName: \'pos\', verdicts: [], asksArchive: false, }';

/**
 The report for the forwarder `mk` read in a place that is no call, in the user file.

 @param line - line of the place

 @param position - kind of the node that holds the forwarder and the field, or the member text

 @param held - name the report holds for the forwarder, the key of the function in its declaring file by default

 @returns Report

 @example
 ```ts
 const spot = positionSpotAt({ line: 2, position: 'Property.value', },);
 ```
 */
function positionSpotAt(
  {
    line,
    position,
    held = keyOf({
      file: STAGE,
      name: 'mk',
    },),
  }: {
    readonly line: number;
    readonly position: string;
    readonly held?: string;
  },
): BlindSpot {
  return {
    file: USER,
    line,
    shape: 'forwarder-outside-a-read-position',
    position,
    held,
  };
}

/**
 A case whose stage file exports a forwarder named `mk` and whose user file holds it as the lines say.

 @param name - name of the case

 @param lines - lines of the user file

 @param spots - reports the check gives, in the user file

 @param isListed - whether the user file is listed as a file that calls the forwarder

 @returns Case that lists the stage file, and the user file when it calls the forwarder

 @example
 ```ts
 const shapeCase = userCase({ name: 'x', lines: [], spots: [], isListed: true, },);
 ```
 */
function userCase(
  {
    name,
    lines,
    spots,
    isListed,
  }: {
    readonly name: string;
    readonly lines: readonly string[];
    readonly spots: readonly BlindSpot[];
    readonly isListed: boolean;
  },
): ShapeCase {
  return {
    name,
    files: [
      SENDER_FILE,
      {
        file: STAGE,
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          'export async function mk(args: Parameters<typeof contestResponseFormat>[0],) {',
          '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
          '}',
        ],),
      },
      {
        file: USER,
        text: textOf(lines,),
      },
    ],
    found: isListed ? [
      STAGE,
      USER,
    ] : [STAGE,],
    builders: [],
    spots,
  };
}

/**
 The import of the forwarder that the user file starts with.
 */
const IMPORT_MK = 'import { mk, } from \'./pos-stage.ts\';';

/**
 Cases of the places a forwarder is read.
 */
export const FORWARDER_POSITION_CASES: readonly ShapeCase[] = [
  userCase({
    name: 'ACCEPTS A FORWARDER THAT IS THE TAG OF A TAGGED TEMPLATE, a call of it',
    lines: [
      IMPORT_MK,
      'export const tagged = mk`pos`;',
    ],
    spots: [],
    isListed: true,
  },),
  userCase({
    name: 'ACCEPTS A FORWARDER THAT NEW CALLS',
    lines: [
      IMPORT_MK,
      `export const made = new mk(${POS_ARGUMENTS},);`,
    ],
    spots: [],
    isListed: true,
  },),
  userCase({
    name: 'ACCEPTS A FORWARDER BOUND AND CALLED AT ONCE',
    lines: [
      IMPORT_MK,
      `export const sent = mk.bind(null,)(${POS_ARGUMENTS},);`,
    ],
    spots: [],
    isListed: true,
  },),
  userCase({
    name: 'REPORTS A FORWARDER BOUND AND KEPT, since the bound function is the forwarder again and a file can call it '
      + 'where the search does not look',
    lines: [
      IMPORT_MK,
      'export const kept = mk.bind(null,);',
    ],
    spots: [positionSpotAt({
      line: 2,
      position: 'VariableDeclarator.init',
    },),],
    isListed: true,
  },),
  userCase({
    name: 'REPORTS A MEMBER OF A FORWARDER THAT IS NO CALL METHOD',
    lines: [
      IMPORT_MK,
      'export const text = mk.toString();',
    ],
    spots: [positionSpotAt({
      line: 2,
      position: 'member toString of a forwarder',
    },),],
    isListed: false,
  },),
  userCase({
    name: 'REPORTS A FREE NAME THAT IS THE NAME OF A FORWARDER AND IS HELD BY AN OBJECT, which a declare const leaves '
      + 'with no value of its own in the file',
    lines: [
      'declare const mk: any;',
      'export const api = { mk, };',
    ],
    spots: [positionSpotAt({
      line: 2,
      position: 'Property.value',
      held: 'mk',
    },),],
    isListed: true,
  },),
];

//endregion The places a forwarder is read that the first cases of its position did not separate
