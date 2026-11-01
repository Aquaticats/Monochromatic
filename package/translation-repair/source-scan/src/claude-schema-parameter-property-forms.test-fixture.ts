import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { CONTEST_IMPORT, } from './claude-schema-destination-forms.test-fixture.ts';
import {
  SENDER_FILE,
  SENDER_IMPORT,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A constructor whose parameter is a parameter property
// `constructor(private args: T)` binds the name `args` as a parameter and as a field. A constructor that hands a sender
// a builder call made from that name is a forwarder like any other function that takes the arguments of a schema from
// its caller, so it is reported where it is written as a function no declaration names, and the same constructor with a
// plain parameter is the control.

/**
 The path of the stage file of every case.
 */
const STAGE = 'imaginary/src/property-stage.ts';

/**
 The call of the constructor, as the caller file writes it.
 */
const CALLER_LINE = 'export const SENT = new K({ schemaName: \'property\', verdicts: [], asksArchive: false, },);';

/**
 The report for a constructor.

 @param line - line the constructor starts on, counted from one

 @returns Report in the stage file

 @example
 ```ts
 const spot = constructorSpotAt({ line: 4, },);
 ```
 */
function constructorSpotAt({ line, }: { readonly line: number; },): BlindSpot {
  return {
    file: STAGE,
    line,
    shape: 'forwarder-in-undeclared-function',
    position: 'FunctionExpression',
  };
}

/**
 A case whose stage file declares a class with one constructor and whose caller file builds an instance.

 @param name - name of the case

 @param parameter - constructor's parameter as written

 @returns Case that lists the stage file and reports its constructor

 @example
 ```ts
 const shapeCase = constructorCase({ name: 'x', parameter: 'private args: any', },);
 ```
 */
function constructorCase(
  {
    name,
    parameter,
  }: {
    readonly name: string;
    readonly parameter: string;
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
          'export class K {',
          `  constructor(${parameter},) {`,
          '    void gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
          '  }',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/property-caller.ts',
        text: textOf([
          'import { K, } from \'./property-stage.ts\';',
          CALLER_LINE,
        ],),
      },
    ],
    found: [STAGE,],
    builders: [],
    spots: [constructorSpotAt({ line: 4, },),],
  };
}

/**
 Cases of a constructor that takes the arguments of a schema.
 */
export const PARAMETER_PROPERTY_CASES: readonly ShapeCase[] = [
  constructorCase({
    name: 'REPORTS A CONSTRUCTOR WHOSE PARAMETER IS A PLAIN NAME, the control of the parameter property cases',
    parameter: 'args: any',
  },),
  constructorCase({
    name: 'REPORTS A CONSTRUCTOR WHOSE PARAMETER IS A PRIVATE PARAMETER PROPERTY, since the name it binds is a '
      + 'parameter',
    parameter: 'private args: any',
  },),
  constructorCase({
    name: 'REPORTS A CONSTRUCTOR WHOSE PARAMETER IS A READONLY PARAMETER PROPERTY WITH A DEFAULT',
    parameter: 'readonly args: any = {}',
  },),
];

//endregion A constructor whose parameter is a parameter property
