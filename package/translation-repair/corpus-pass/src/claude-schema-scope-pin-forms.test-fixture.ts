import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { declaringFile, } from './claude-schema-constant-reader-forms.test-fixture.ts';
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
import type { SourceFile, } from './claude-schema-sources.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Names, modules and forwarders, as the pin cases of the check hold them
// A namespace or an enum declared in the body of a namespace binds its name there, so a call of that name is no call of
// the sender. A type-only import-equals declaration binds nothing. A module that exports a forwarder by name is called
// through `import stage = require(path)` as `stage.mk`. A constant that a module exports with `export =` is read
// through the member of the import, and the read is reported beside the export. A class method that passes its
// parameter to a forwarder is a forwarder no name stands for.

/**
 A file that declares a forwarder and exports it by name.
 */
const FORWARDER_STAGE: SourceFile = {
  file: 'imaginary/src/scope-stage.ts',
  text: textOf([
    CONTEST_IMPORT,
    SENDER_IMPORT,
    'export async function mk(args: Parameters<typeof contestResponseFormat>[0],) {',
    '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
    '}',
  ],),
};

/**
 A stage file whose namespace declares a name like the sender and calls it with a constant of the file.

 @param file - path of the file

 @param declaration - line that declares the name inside the namespace

 @returns File whose call of that name is no call of the sender

 @example
 ```ts
 const stage = shadowedStage({ file: 'imaginary/src/a.ts', declaration: 'export enum gatherThroughCache { A }', },);
 ```
 */
function shadowedStage(
  {
    file,
    declaration,
  }: {
    readonly file: string;
    readonly declaration: string;
  },
): SourceFile {
  return {
    file,
    text: textOf([
      CONTEST_IMPORT,
      SENDER_IMPORT,
      `const FORMAT = contestResponseFormat(${ARGUMENTS},);`,
      'export namespace Holder {',
      `  ${declaration}`,
      '  export const SENT = gatherThroughCache({ responseFormat: FORMAT, },);',
      '}',
    ],),
  };
}

/**
 Cases of names, modules and forwarders.
 */
export const SCOPE_PIN_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A SCHEMA HANDED TO A NAME THAT A NAMESPACE OR AN ENUM DECLARES IN THE BODY OF A NAMESPACE, since the '
      + 'name is then no sender',
    files: [
      SENDER_FILE,
      shadowedStage({
        file: 'imaginary/src/scope-namespace-stage.ts',
        declaration: 'export namespace gatherThroughCache { export const x = 1; }',
      },),
      shadowedStage({
        file: 'imaginary/src/scope-enum-stage.ts',
        declaration: 'export enum gatherThroughCache { A }',
      },),
    ],
    found: [
      'imaginary/src/scope-enum-stage.ts',
      'imaginary/src/scope-namespace-stage.ts',
    ],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/scope-namespace-stage.ts',
        line: 3,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/scope-enum-stage.ts',
        line: 3,
        position: CALL_ARGUMENT,
      },),
    ],
  },
  {
    name: 'ACCEPTS A CONSTANT READ THROUGH A TYPE-ONLY IMPORT-EQUALS DECLARATION, since it binds nothing',
    files: [
      declaringFile({
        file: 'imaginary/src/scope-type-format.ts',
        name: 'TYPE_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/scope-type-reader.ts',
        text: textOf([
          'import type formats = require(\'./scope-type-format.ts\');',
          'export const kept: unknown[] = [];',
          'kept.push(formats.TYPE_FORMAT,);',
        ],),
      },
    ],
    found: ['imaginary/src/scope-type-format.ts',],
    builders: [],
    spots: [],
  },
  {
    name: 'LISTS THE FILE THAT CALLS A FORWARDER THROUGH A MEMBER OF A MODULE BROUGHT IN WITH `import x = require(path)`',
    files: [
      SENDER_FILE,
      FORWARDER_STAGE,
      {
        file: 'imaginary/src/scope-caller.ts',
        text: textOf([
          'import stage = require(\'./scope-stage.ts\');',
          `export const SENT = stage.mk(${ARGUMENTS},);`,
        ],),
      },
    ],
    found: [
      'imaginary/src/scope-caller.ts',
      'imaginary/src/scope-stage.ts',
    ],
    builders: [],
    spots: [],
  },
  {
    name: 'REPORTS A CONSTANT A FILE EXPORTS WITH `export =` WHEN ANOTHER FILE HANDS A MEMBER OF IT TO A CALL THAT IS NO '
      + 'SENDER THROUGH `import x = require(path)`, beside the export',
    files: [
      {
        file: 'imaginary/src/scope-assigned-format.ts',
        text: textOf([
          CONTEST_IMPORT,
          `const FORMAT = contestResponseFormat(${ARGUMENTS},);`,
          'export = FORMAT;',
        ],),
      },
      {
        file: 'imaginary/src/scope-assigned-reader.ts',
        text: textOf([
          'import format = require(\'./scope-assigned-format.ts\');',
          'export const kept: unknown[] = [];',
          'kept.push(format.json_schema,);',
        ],),
      },
    ],
    found: ['imaginary/src/scope-assigned-format.ts',],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/scope-assigned-format.ts',
        line: 2,
        position: `${CALL_ARGUMENT} in imaginary/src/scope-assigned-reader.ts, TSExportAssignment.expression`,
      },),
    ],
  },
  {
    name: 'REPORTS A CLASS METHOD THAT PASSES ITS PARAMETER TO A FORWARDER, since no name stands for the method that '
      + 'another file could write',
    files: [
      SENDER_FILE,
      FORWARDER_STAGE,
      {
        file: 'imaginary/src/scope-holder.ts',
        text: textOf([
          'import { mk, } from \'./scope-stage.ts\';',
          'export class Holder {',
          '  static make(args: any,) { return mk(args,); }',
          '}',
        ],),
      },
    ],
    found: [
      'imaginary/src/scope-holder.ts',
      'imaginary/src/scope-stage.ts',
    ],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/scope-holder.ts',
        line: 3,
        shape: 'forwarder-in-undeclared-function',
        position: 'FunctionExpression',
      },
    ],
  },
];

//endregion Names, modules and forwarders, as the pin cases of the check hold them
