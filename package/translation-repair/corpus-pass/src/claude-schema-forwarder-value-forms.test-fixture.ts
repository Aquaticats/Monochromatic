import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { CONTEST_IMPORT, } from './claude-schema-destination-forms.test-fixture.ts';
import {
  SENDER_FILE,
  SENDER_IMPORT,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';
import { keyOf, } from './claude-schema-usage.test-fixture.ts';

//region A forwarder read anywhere but as the callee of a call
// The files that call a forwarder are listed, because they write the arguments of its schema. A file that gets the
// function from a place that is no call (a property of an object, an element of an array, an argument of a call, an
// operand of a comma) can call it where the search does not look, so the check reports the place, as it reports a
// wrapper that is held the same way. A call is read, and so are the members `call`, `apply` and `bind`, the whole value
// of an export assignment (the listing reads it as the module), the members `name` and `length` that hand out no
// function, and a default export.

/**
 The path of the file that declares the forwarder in every case.
 */
const STAGE = 'imaginary/src/value-stage.ts';

/**
 The path of the file that holds the forwarder in a place that is no call, where a case has one.
 */
const MIDDLE = 'imaginary/src/value-middle.ts';

/**
 The lines that declare a forwarder named `mk`, after the two imports of the stage file.
 */
const DECLARATION = [
  'async function mk(args: Parameters<typeof contestResponseFormat>[0],) {',
  '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
  '}',
] as const;

/**
 The arguments of a call of `mk` as a file that calls it writes them.
 */
const VALUE_ARGUMENTS = '{ schemaName: \'value\', verdicts: [], asksArchive: false, }';

/**
 The report for the forwarder `mk` read in a place that is no call.

 @param file - file the place sits in

 @param line - line of the place

 @param position - kind of the node that holds the forwarder and the field

 @returns Report

 @example
 ```ts
 const spot = valueSpotAt({ file: 'imaginary/src/a.ts', line: 6, position: 'Property.value', },);
 ```
 */
function valueSpotAt(
  {
    file,
    line,
    position,
  }: {
    readonly file: string;
    readonly line: number;
    readonly position: string;
  },
): BlindSpot {
  return {
    file,
    line,
    shape: 'forwarder-outside-a-read-position',
    position,
    held: keyOf({
      file: STAGE,
      name: 'mk',
    },),
  };
}

/**
 A case whose stage file declares the forwarder and holds it as the lines say, with a caller file that calls the holder.

 @param name - name of the case

 @param lines - lines of the stage file after the declaration; the first is line six of the file

 @param call - expression the caller file keeps

 @param imported - name the caller file imports from the stage file

 @param position - where the forwarder sits in the first of the lines, empty for a case that reports nothing

 @param line - line of the place that holds the forwarder, six for the first of the lines

 @returns Case that lists the stage file, and the caller file when the case reports nothing

 @example
 ```ts
 const shapeCase = heldCase({
   name: 'x',
   lines: [],
   call: 'api.mk(1,)',
   imported: 'api',
   position: 'Property.value',
   line: 6,
 },);
 ```
 */
function heldCase(
  {
    name,
    lines,
    call,
    imported,
    position,
    line,
  }: {
    readonly name: string;
    readonly lines: readonly string[];
    readonly call: string;
    readonly imported: string;
    readonly position: string;
    readonly line: number;
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
          ...DECLARATION,
          ...lines,
        ],),
      },
      {
        file: 'imaginary/src/value-caller.ts',
        text: textOf([
          `import ${imported} from './value-stage.ts';`,
          `export const SENT = ${call};`,
        ],),
      },
    ],
    found: (position === '')
      ? [
        'imaginary/src/value-caller.ts',
        STAGE,
      ]
      : [STAGE,],
    builders: [],
    spots: (position === '')
      ? []
      : [
        valueSpotAt({
          file: STAGE,
          line,
          position,
        },),
      ],
  };
}

/**
 A case whose middle file holds an exported forwarder in the form the case names, inside a function its caller calls.

 @param name - name of the case

 @param held - expression that holds `mk` inside the function

 @param line - line of the expression in the middle file

 @param position - kind of the node that holds the forwarder and the field

 @returns Case that lists the stage file and the middle file

 @example
 ```ts
 const shapeCase = passedCase({
   name: 'x',
   held: 'Reflect.apply(mk, null, [args,],)',
   line: 2,
   position: 'CallExpression.arguments',
 },);
 ```
 */
function passedCase(
  {
    name,
    held,
    line,
    position,
  }: {
    readonly name: string;
    readonly held: string;
    readonly line: number;
    readonly position: string;
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
        file: MIDDLE,
        text: textOf([
          'import { mk, } from \'./value-stage.ts\';',
          'declare const run: (f: (a: unknown,) => unknown, args: unknown,) => unknown;',
          `export async function mid(args: any,) { return await ${held}; }`,
        ],),
      },
      {
        file: 'imaginary/src/value-caller.ts',
        text: textOf([
          'import { mid, } from \'./value-middle.ts\';',
          `export const SENT = mid(${VALUE_ARGUMENTS},);`,
        ],),
      },
    ],
    found: [
      MIDDLE,
      STAGE,
    ],
    builders: [],
    spots: [
      valueSpotAt({
        file: MIDDLE,
        line,
        position,
      },),
    ],
  };
}

/**
 A case with one more report: the caller file reads a member of the default import of a stage file that exports an
 object, which the search records no default for, as it reports the same read of a module that carries a builder.

 @param shapeCase - case whose caller imports the stage file as a default and calls a member of it

 @returns Case with the report of the caller added

 @example
 ```ts
 const held = heldCase({ name: 'x', lines: [], call: 's.mk(1,)', imported: 's', position: '', line: 6, },);
 const reported = withDefaultMemberSpot({ shapeCase: held, },);
 ```
 */
function withDefaultMemberSpot({ shapeCase, }: { readonly shapeCase: ShapeCase; },): ShapeCase {
  return {
    ...shapeCase,
    spots: [
      ...shapeCase.spots,
      {
        file: 'imaginary/src/value-caller.ts',
        line: 2,
        shape: 'module-binding-outside-a-read-position',
        position: 'member of a default the module does not export',
      },
    ],
  };
}

/**
 Cases of a forwarder read in a place that is no call.
 */
export const FORWARDER_VALUE_CASES: readonly ShapeCase[] = [
  heldCase({
    name: 'REPORTS A FORWARDER HELD BY A PROPERTY OF AN EXPORTED OBJECT, shorthand, since the caller of api.mk is not '
      + 'listed',
    lines: ['export const api = { mk, };',],
    call: `api.mk(${VALUE_ARGUMENTS},)`,
    imported: '{ api, }',
    position: 'Property.value',
    line: 6,
  },),
  heldCase({
    name: 'REPORTS A FORWARDER HELD BY A PROPERTY OF AN EXPORTED OBJECT UNDER ANOTHER KEY',
    lines: ['export const api = { run: mk, };',],
    call: `api.run(${VALUE_ARGUMENTS},)`,
    imported: '{ api, }',
    position: 'Property.value',
    line: 6,
  },),
  heldCase({
    name: 'REPORTS A FORWARDER HELD BY AN ELEMENT OF AN EXPORTED ARRAY',
    lines: ['export const list = [mk,];',],
    call: `list[0](${VALUE_ARGUMENTS},)`,
    imported: '{ list, }',
    position: 'ArrayExpression.elements',
    line: 6,
  },),
  withDefaultMemberSpot({
    shapeCase: heldCase({
      name: 'REPORTS A FORWARDER HELD BY AN OBJECT THAT AN EXPORT ASSIGNMENT EXPORTS, and the caller that reads a '
        + 'member of the default import',
      lines: ['export = { mk, };',],
      call: `s.mk(${VALUE_ARGUMENTS},)`,
      imported: 's',
      position: 'Property.value',
      line: 6,
    },),
  },),
  withDefaultMemberSpot({
    shapeCase: heldCase({
      name: 'REPORTS A FORWARDER HELD BY AN OBJECT THAT THE DEFAULT EXPORT IS, and the caller that reads a member of '
        + 'the default import',
      lines: ['export default { mk, };',],
      call: `s.mk(${VALUE_ARGUMENTS},)`,
      imported: 's',
      position: 'Property.value',
      line: 6,
    },),
  },),
  heldCase({
    name: 'REPORTS A FORWARDER HELD BY A CONSTANT OBJECT THAT IS EXPORTED BY NAME LATER',
    lines: [
      'const api = { mk, };',
      'export default api;',
    ],
    call: `s.mk(${VALUE_ARGUMENTS},)`,
    imported: 's',
    position: 'Property.value',
    line: 6,
  },),
  heldCase({
    name: 'REPORTS A FORWARDER HANDED TO A CALL THAT AN EXPORT ASSIGNMENT EXPORTS',
    lines: [
      'declare function wrap<T,>(f: T): T;',
      'export = wrap(mk,);',
    ],
    call: `s(${VALUE_ARGUMENTS},)`,
    imported: 's',
    position: 'CallExpression.arguments',
    line: 7,
  },),
  passedCase({
    name: 'REPORTS A FORWARDER CALLED THROUGH REFLECT.APPLY, since it is an argument and no callee',
    held: 'Reflect.apply(mk, null, [args,],)',
    line: 3,
    position: 'CallExpression.arguments',
  },),
  passedCase({
    name: 'REPORTS A FORWARDER CALLED AS THE LAST OPERAND OF A COMMA',
    held: '(0, mk)(args,)',
    line: 3,
    position: 'SequenceExpression.expressions',
  },),
  passedCase({
    name: 'REPORTS A FORWARDER PASSED TO A FUNCTION THAT CALLS ITS ARGUMENT',
    held: 'run(mk, args,)',
    line: 3,
    position: 'CallExpression.arguments',
  },),
  heldCase({
    name: 'ACCEPTS A FORWARDER THAT IS THE WHOLE VALUE OF AN EXPORT ASSIGNMENT, a DEFAULT EXPORT, a NAME READ OFF IT '
      + 'AND A CALL WITH call',
    lines: [
      'export default mk;',
      'export const tag = mk.name;',
      'export const width = mk.length;',
      'export const again = mk.call(null, { schemaName: \'again\', verdicts: [], asksArchive: false, },);',
    ],
    call: `s(${VALUE_ARGUMENTS},)`,
    imported: 's',
    position: '',
    line: 0,
  },),
];

//endregion A forwarder read anywhere but as the callee of a call
