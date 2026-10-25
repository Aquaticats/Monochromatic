import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { CONTEST_IMPORT, } from './claude-schema-destination-forms.test-fixture.ts';
import {
  SENDER_FILE,
  SENDER_IMPORT,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Forwarders written in a form that declares no function
// A function that hands a sender a builder call made from its own parameters takes the arguments of the schema from
// its callers. When the function is a class method, a static method, a constructor, a getter or a setter, a member of
// an object nested in an object, an anonymous default export or a function expression that nothing names, no name
// stands for it that another file could write, so the files that call it cannot be listed. The check reports the
// function where it is written, as it does for a function that holds a format and that no declared function holds.

/**
 The arguments of a call of the function a case declares, as the file that calls it writes them.
 */
const SHAPE_ARGUMENTS = '{ schemaName: \'shape\', verdicts: [], asksArchive: false, }';

/**
 The type of the parameter every case gives its function.
 */
const PARAMETER_TYPE = 'Parameters<typeof contestResponseFormat>[0]';

/**
 The report for a forwarder written in a form that declares no function.

 @param line - line the function starts on, counted from one

 @param position - kind of the function

 @returns Report in the file the case declares the function in

 @example
 ```ts
 const spot = forwarderSpotAt({ line: 4, position: 'FunctionExpression', },);
 ```
 */
function forwarderSpotAt(
  {
    line,
    position,
  }: {
    readonly line: number;
    readonly position: string;
  },
): BlindSpot {
  return {
    file: 'imaginary/src/shape-stage.ts',
    line,
    shape: 'forwarder-in-undeclared-function',
    position,
  };
}

/**
 A case whose stage file declares the function and whose caller file calls it.

 @param name - name of the case

 @param lines - lines of the stage file after its two imports

 @param call - expression the caller file keeps, over an import of `imported`

 @param imported - name the caller file imports from the stage file, `default` for the default export

 @param spots - reports in the stage file

 @returns Case that lists the stage file and not the caller file

 @example
 ```ts
 const shapeCase = undeclaredCase({ name: 'x', lines: [], call: 'k()', imported: 'k', spots: [], },);
 ```
 */
function undeclaredCase(
  {
    name,
    lines,
    call,
    imported,
    spots,
  }: {
    readonly name: string;
    readonly lines: readonly string[];
    readonly call: string;
    readonly imported: string;
    readonly spots: readonly BlindSpot[];
  },
): ShapeCase {
  return {
    name,
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/shape-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          ...lines,
        ],),
      },
      {
        file: 'imaginary/src/shape-caller.ts',
        text: textOf([
          (imported === 'default')
            ? 'import ask from \'./shape-stage.ts\';'
            : `import { ${imported}, } from './shape-stage.ts';`,
          `export const SENT = ${call};`,
        ],),
      },
    ],
    found: ['imaginary/src/shape-stage.ts',],
    builders: [],
    spots,
  };
}

/**
 The line that sends the schema, which every case writes the same way.
 */
const SENDS = '    return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);';

/**
 Cases of a function written in a form that declares no function.
 */
export const UNDECLARED_FORWARDER_CASES: readonly ShapeCase[] = [
  undeclaredCase({
    name: 'REPORTS A CLASS METHOD THAT HANDS A SENDER A BUILDER CALL MADE FROM ITS PARAMETER, so a file that calls it '
      + 'on an instance, which no name could list, is not left to write the arguments of a schema unseen',
    lines: [
      'export class Asker {',
      `  async ask(args: ${PARAMETER_TYPE},) {`,
      SENDS,
      '  }',
      '}',
    ],
    call: `new Asker().ask(${SHAPE_ARGUMENTS},)`,
    imported: 'Asker',
    spots: [
      forwarderSpotAt({
        line: 4,
        position: 'FunctionExpression',
      },),
    ],
  },),
  undeclaredCase({
    name: 'REPORTS A STATIC METHOD THAT HANDS A SENDER A BUILDER CALL MADE FROM ITS PARAMETER, as a method of the class',
    lines: [
      'export class Asker {',
      `  static async ask(args: ${PARAMETER_TYPE},) {`,
      SENDS,
      '  }',
      '}',
    ],
    call: `Asker.ask(${SHAPE_ARGUMENTS},)`,
    imported: 'Asker',
    spots: [
      forwarderSpotAt({
        line: 4,
        position: 'FunctionExpression',
      },),
    ],
  },),
  undeclaredCase({
    name: 'REPORTS A CONSTRUCTOR THAT HANDS A SENDER A BUILDER CALL MADE FROM ITS PARAMETER, since `new` calls it',
    lines: [
      'export class Asker {',
      `  constructor(args: ${PARAMETER_TYPE},) {`,
      '    void gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
      '  }',
      '}',
    ],
    call: `new Asker(${SHAPE_ARGUMENTS},)`,
    imported: 'Asker',
    spots: [
      forwarderSpotAt({
        line: 4,
        position: 'FunctionExpression',
      },),
    ],
  },),
  undeclaredCase({
    name: 'REPORTS A MEMBER OF AN OBJECT NESTED IN AN OBJECT THAT HANDS A SENDER A BUILDER CALL MADE FROM ITS PARAMETER, '
      + 'which the one level of members a constant holds does not declare',
    lines: [
      'export const api = {',
      '  sub: {',
      `    async ask(args: ${PARAMETER_TYPE},) {`,
      `  ${SENDS}`,
      '    },',
      '  },',
      '};',
    ],
    call: `api.sub.ask(${SHAPE_ARGUMENTS},)`,
    imported: 'api',
    spots: [
      forwarderSpotAt({
        line: 5,
        position: 'FunctionExpression',
      },),
    ],
  },),
  undeclaredCase({
    name: 'REPORTS AN ANONYMOUS DEFAULT EXPORT ARROW THAT HANDS A SENDER A BUILDER CALL MADE FROM ITS PARAMETER, which '
      + 'no name declares',
    lines: [
      `export default async (args: ${PARAMETER_TYPE},) =>`,
      '  await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
    ],
    call: `ask(${SHAPE_ARGUMENTS},)`,
    imported: 'default',
    spots: [
      forwarderSpotAt({
        line: 3,
        position: 'ArrowFunctionExpression',
      },),
    ],
  },),
  undeclaredCase({
    name: 'REPORTS AN ANONYMOUS DEFAULT EXPORT FUNCTION THAT HANDS A SENDER A BUILDER CALL MADE FROM ITS PARAMETER, '
      + 'a function expression that nothing names',
    lines: [
      `export default async function (args: ${PARAMETER_TYPE},) {`,
      SENDS.slice(2,),
      '}',
    ],
    call: `ask(${SHAPE_ARGUMENTS},)`,
    imported: 'default',
    spots: [
      forwarderSpotAt({
        line: 3,
        position: 'FunctionDeclaration',
      },),
    ],
  },),
  undeclaredCase({
    name: 'REPORTS A FUNCTION A GETTER GIVES BACK THAT HANDS A SENDER A BUILDER CALL MADE FROM ITS PARAMETER, since the '
      + 'getter has no parameter to build from and the function it returns is what a caller calls',
    lines: [
      'export const api = {',
      '  get ask() {',
      `    return async (args: ${PARAMETER_TYPE},) =>`,
      '      await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
      '  },',
      '};',
    ],
    call: `api.ask(${SHAPE_ARGUMENTS},)`,
    imported: 'api',
    spots: [
      forwarderSpotAt({
        line: 5,
        position: 'ArrowFunctionExpression',
      },),
    ],
  },),
  undeclaredCase({
    name: 'REPORTS A SETTER THAT HANDS A SENDER A BUILDER CALL MADE FROM ITS PARAMETER, whose one parameter is the value '
      + 'a caller assigns',
    lines: [
      'export const api = {',
      `  set ask(args: ${PARAMETER_TYPE},) {`,
      '    void gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
      '  },',
      '};',
    ],
    call: `Object.assign(api, { ask: ${SHAPE_ARGUMENTS}, },)`,
    imported: 'api',
    spots: [
      forwarderSpotAt({
        line: 4,
        position: 'FunctionExpression',
      },),
    ],
  },),
  undeclaredCase({
    name: 'REPORTS A CLASS METHOD A DECLARED FUNCTION REACHES, since the function that calls the method is no forwarder '
      + 'the search could list by a name the method has',
    lines: [
      'class Impl {',
      `  async run(args: ${PARAMETER_TYPE},) {`,
      SENDS,
      '  }',
      '}',
      `export const ask = (args: ${PARAMETER_TYPE},) => new Impl().run(args,);`,
    ],
    call: `ask(${SHAPE_ARGUMENTS},)`,
    imported: 'ask',
    spots: [
      forwarderSpotAt({
        line: 4,
        position: 'FunctionExpression',
      },),
    ],
  },),
  undeclaredCase({
    name: 'DOES NOT REPORT A METHOD THAT HANDS A SENDER A BUILDER CALL MADE FROM NOTHING OF ITS PARAMETERS, since the '
      + 'file that holds the method writes the arguments of the schema and is listed already',
    lines: [
      'export class Asker {',
      '  static async ask(question: string,) {',
      '    return await gatherThroughCache({',
      '      responseFormat: contestResponseFormat({ schemaName: \'fixed\', verdicts: [], asksArchive: false, },),',
      '      question,',
      '    },);',
      '  }',
      '}',
    ],
    call: 'Asker.ask(\'where\',)',
    imported: 'Asker',
    spots: [],
  },),
  undeclaredCase({
    name: 'REPORTS NO FORWARDER FOR A METHOD THAT BUILDS FROM ITS PARAMETER AND CALLS NO SENDER, only the place it '
      + 'stores the schema, since a schema that goes to no sender is reported where it goes',
    lines: [
      'export class Asker {',
      '  static kept: unknown;',
      `  static remember(args: ${PARAMETER_TYPE},) {`,
      '    Asker.kept = contestResponseFormat(args,);',
      '  }',
      '}',
    ],
    call: `Asker.remember(${SHAPE_ARGUMENTS},)`,
    imported: 'Asker',
    spots: [
      {
        file: 'imaginary/src/shape-stage.ts',
        line: 6,
        shape: 'builder-result-outside-a-destination',
        position: 'AssignmentExpression.right',
      },
    ],
  },),
];

//endregion Forwarders written in a form that declares no function
