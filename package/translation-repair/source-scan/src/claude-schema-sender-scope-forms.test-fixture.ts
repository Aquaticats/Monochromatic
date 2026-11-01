import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import {
  SENDER_FILE,
  SENDER_IMPORT,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A name of the sender that a namespace declares again, and a name of a forwarder a file leaves free
// A call is a call of the sender only where the name it calls is the one the file imports. A namespace that declares
// the name again, as a function with `declare function`, or as a qualified namespace
// (`namespace gatherThroughCache.Inner`), makes the name another thing inside it, so the object that the call takes is
// reported like the argument of any call. A name that a file leaves free, with only a `declare const` for it, is read
// as the forwarder of that name, so the files that call a function around it are listed.

/**
 The path of the stage file of the cases of the namespace.
 */
const STAGE = 'imaginary/src/scope-stage.ts';

/**
 The line of the stage file that holds the builder's call: the first line after the two imports.
 */
const LINE_OF_BUILDER = 3;

/**
 The constant that holds the schema, which every case of the namespace declares at the top level of the stage file,
 since a builder written inside a namespace is reported for the namespace.
 */
const SCHEMA_CONSTANT = `const SCHEMA = contestResponseFormat(${ARGUMENTS},);`;

/**
 A case whose stage file imports the sender, holds a schema in a constant, and then holds a namespace that calls a name
 of the sender with it.

 @param name - name of the case

 @param lines - lines of the stage file after the imports, one of which holds the call

 @param isReported - whether the call is reported at the builder's call

 @returns Case that lists the stage file

 @example
 ```ts
 const shapeCase = namespaceCase({ name: 'x', lines: [], isReported: true, },);
 ```
 */
function namespaceCase(
  {
    name,
    lines,
    isReported,
  }: {
    readonly name: string;
    readonly lines: readonly string[];
    readonly isReported: boolean;
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
          SCHEMA_CONSTANT,
          ...lines,
        ],),
      },
    ],
    found: [STAGE,],
    builders: [],
    spots: isReported
      ? [
        spotAt({
          file: STAGE,
          line: LINE_OF_BUILDER,
          position: CALL_ARGUMENT,
        },),
      ]
      : [],
  };
}

/**
 The call of the sender with the schema of the constant.
 */
const CALL_OF_SENDER = 'gatherThroughCache({ responseFormat: SCHEMA, },)';

/**
 Cases of a namespace that declares the name of the sender again, and of a name of a forwarder a file leaves free.
 */
export const SENDER_SCOPE_CASES: readonly ShapeCase[] = [
  namespaceCase({
    name: 'ACCEPTS A CALL OF THE SENDER INSIDE A NAMESPACE THAT DECLARES NOTHING UNDER ITS NAME, the control of the '
      + 'cases that declare the name again',
    lines: [
      'export namespace Outer {',
      `  export const r = ${CALL_OF_SENDER};`,
      '}',
    ],
    isReported: false,
  },),
  namespaceCase({
    name: 'REPORTS A CALL OF A NAME THAT A NAMESPACE DECLARES AS A FUNCTION, so it is not the sender the file imports',
    lines: [
      'export namespace Outer {',
      '  declare function gatherThroughCache(o: object,): object;',
      `  export const r = ${CALL_OF_SENDER};`,
      '}',
    ],
    isReported: true,
  },),
  namespaceCase({
    name: 'REPORTS A CALL OF A NAME THAT A QUALIFIED NAMESPACE DECLARES INSIDE A NAMESPACE, so it is not the sender '
      + 'the file imports',
    lines: [
      'export namespace Outer {',
      '  namespace gatherThroughCache.Inner {',
      '    export const x = 1;',
      '  }',
      '  export const r = (gatherThroughCache as any)({ responseFormat: SCHEMA, },);',
      '}',
    ],
    isReported: true,
  },),
  {
    name: 'LISTS THE FILE THAT CALLS A FUNCTION WHOSE CALL HAS A FREE NAME THAT IS THE NAME OF A FORWARDER, which a '
      + '`declare const` leaves with no value of its own in the file',
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/free-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          'export async function mk(args: Parameters<typeof contestResponseFormat>[0],) {',
          '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/free-middle.ts',
        text: textOf([
          'declare const mk: any;',
          'export async function mid(args: unknown,) {',
          '  return await mk(args,);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/free-caller.ts',
        text: textOf([
          'import { mid, } from \'./free-middle.ts\';',
          `export const SENT = mid(${ARGUMENTS},);`,
        ],),
      },
    ],
    found: [
      'imaginary/src/free-caller.ts',
      'imaginary/src/free-middle.ts',
      'imaginary/src/free-stage.ts',
    ],
    builders: [],
    spots: [],
  },
];

//endregion A name of the sender that a namespace declares again, and a name of a forwarder a file leaves free
