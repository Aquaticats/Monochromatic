import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { SourceFile, } from './claude-schema-sources.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region The files that read a constant a builder's value is held by
// The third total follows a constant, whose every reference must end in a sender, a wrapper or a constant that does.
// A constant that a file exports is read in the files that import it, by its name, by an alias, through a module that
// exports it again, through a namespace import or an `import()` of the file. The cases give one file for each way, each
// with one reader that hands the value to a call that is no sender (reported at the builder's call, naming the file of
// the reader) or one that reads it where it ends (silent). The real shape the family has is held too: a constant read
// in a sender in its own file, exported again by an index, put in a list of producers, and the list handed to the one
// function that checks that each schema is sendable.

/**
 The import of the one-argument sender the cases use, as a specifier written in a file under `imaginary/src`.
 */
const SENDER_IMPORT =
  'import { gatherThroughCache, } from \'../../corpus-pass/src/corpus-run/pass-consistency-gather.ts\';';

/**
 The module of the sender the cases use.
 */
const GATHER_FILE = {
  file: 'corpus-pass/src/corpus-run/pass-consistency-gather.ts',
  text: textOf([
    'export async function gatherThroughCache({ responseFormat, }: { readonly responseFormat: unknown; },) {',
    '  return responseFormat;',
    '}',
  ],),
};

/**
 The file that declares a constant a builder's value is held by, the builder's call on the second line.

 @param file - path of the imaginary file

 @param name - name of the constant

 @param more - lines after the constant

 @returns File

 @example
 ```ts
 const declaring = declaringFile({ file: 'imaginary/src/a.ts', name: 'A_FORMAT', more: [], },);
 ```
 */
function declaringFile(
  {
    file,
    name,
    more,
  }: {
    readonly file: string;
    readonly name: string;
    readonly more: readonly string[];
  },
): SourceFile {
  return {
    file,
    text: textOf([
      CONTEST_IMPORT,
      `export const ${name} = contestResponseFormat(${ARGUMENTS},);`,
      ...more,
    ],),
  };
}

/**
 The place the check reports for a constant read by a file.

 @param reader - path of the file that reads it

 @returns Position that names the kind of place and the file

 @example
 ```ts
 const position = handedOnIn({ reader: 'imaginary/src/a.ts', },);
 ```
 */
function handedOnIn({ reader, }: { readonly reader: string; },): string {
  return `${CALL_ARGUMENT} in ${reader}`;
}

/**
 Cases of the files that read a constant a builder's value is held by.
 */
export const CONSTANT_READER_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A CONSTANT THAT A FILE EXPORTS WHEN ANOTHER FILE HANDS IT TO A CALL THAT IS NO SENDER, by its name, '
      + 'by an alias through a module that exports it again and through a star, through a member of a namespace '
      + 'import, through a namespace import as a whole and through an import() of the file, and names the file',
    files: [
      declaringFile({
        file: 'imaginary/src/direct-format.ts',
        name: 'DIRECT_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/direct-reader.ts',
        text: textOf([
          'import { DIRECT_FORMAT, } from \'./direct-format.ts\';',
          'export const kept = Object.freeze(DIRECT_FORMAT,);',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/chain-format.ts',
        name: 'CHAIN_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/chain-index.ts',
        text: textOf([
          'export { CHAIN_FORMAT as RENAMED_FORMAT, } from \'./chain-format.ts\';',
        ],),
      },
      {
        file: 'imaginary/src/chain-star.ts',
        text: textOf([
          'export * from \'./chain-index.ts\';',
        ],),
      },
      {
        file: 'imaginary/src/chain-reader.ts',
        text: textOf([
          'import { RENAMED_FORMAT as renamed, } from \'./chain-star.ts\';',
          'export const kept = Object.freeze(renamed,);',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/member-format.ts',
        name: 'MEMBER_FORMAT',
        more: ['export const MEMBER_COUNT = 1;',],
      },),
      {
        file: 'imaginary/src/member-reader.ts',
        text: textOf([
          'import * as formats from \'./member-format.ts\';',
          'export const kept = Object.freeze(formats.MEMBER_FORMAT,);',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/whole-format.ts',
        name: 'WHOLE_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/whole-reader.ts',
        text: textOf([
          'import * as formats from \'./whole-format.ts\';',
          'export const kept = Object.freeze(formats,);',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/load-format.ts',
        name: 'LOAD_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/load-reader.ts',
        text: textOf([
          'export const kept = Object.freeze((await import(\'./load-format.ts\')).LOAD_FORMAT,);',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/list-format.ts',
        name: 'LIST_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/list-producers.ts',
        text: textOf([
          'import { LIST_FORMAT, } from \'./list-format.ts\';',
          'export const LIST_PRODUCERS = [{ name: \'list\', formats: [LIST_FORMAT,], },];',
        ],),
      },
      {
        file: 'imaginary/src/list-reader.ts',
        text: textOf([
          'import { LIST_PRODUCERS, } from \'./list-producers.ts\';',
          'export const kept = Object.freeze(LIST_PRODUCERS,);',
        ],),
      },
    ],
    found: [
      'imaginary/src/chain-format.ts',
      'imaginary/src/direct-format.ts',
      'imaginary/src/list-format.ts',
      'imaginary/src/load-format.ts',
      'imaginary/src/member-format.ts',
      'imaginary/src/whole-format.ts',
    ],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/direct-format.ts',
        line: 2,
        position: handedOnIn({ reader: 'imaginary/src/direct-reader.ts', },),
      },),
      spotAt({
        file: 'imaginary/src/chain-format.ts',
        line: 2,
        position: handedOnIn({ reader: 'imaginary/src/chain-reader.ts', },),
      },),
      spotAt({
        file: 'imaginary/src/member-format.ts',
        line: 2,
        position: handedOnIn({ reader: 'imaginary/src/member-reader.ts', },),
      },),
      spotAt({
        file: 'imaginary/src/whole-format.ts',
        line: 2,
        position: handedOnIn({ reader: 'imaginary/src/whole-reader.ts', },),
      },),
      spotAt({
        file: 'imaginary/src/load-format.ts',
        line: 2,
        position: 'import() or require of imaginary/src/load-format.ts in imaginary/src/load-reader.ts',
      },),
      spotAt({
        file: 'imaginary/src/list-format.ts',
        line: 2,
        position: handedOnIn({ reader: 'imaginary/src/list-reader.ts', },),
      },),
    ],
  },
  {
    name: 'READS A CONSTANT THAT A FILE EXPORTS IN THE FILES THAT IMPORT IT AND NO OTHER, and ends the walk in a '
      + 'sender, in the one function that checks the producers\' schemas, in an export from, in a member of a '
      + 'namespace that is not the constant, and in a name another module exports',
    files: [
      GATHER_FILE,
      {
        file: 'stage/src/stage-windowed-rounds.ts',
        text: textOf([
          'export async function runWindowedRounds({ responseFormat, }: { readonly responseFormat: unknown; },) {',
          '  return responseFormat;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/gate-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          'import { runWindowedRounds, } from \'../../stage/src/stage-windowed-rounds.ts\';',
          `export const GATE_RESPONSE_FORMAT = contestResponseFormat(${ARGUMENTS},);`,
          'export async function gate() {',
          '  return await runWindowedRounds({ responseFormat: GATE_RESPONSE_FORMAT, },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/gate-index.ts',
        text: textOf([
          'export { GATE_RESPONSE_FORMAT, } from \'./gate-stage.ts\';',
        ],),
      },
      {
        file: 'imaginary/src/gate-producers.ts',
        text: textOf([
          'import { GATE_RESPONSE_FORMAT, } from \'./gate-index.ts\';',
          'export const GATE_PRODUCERS = [',
          '  { name: \'gate\', formats: [GATE_RESPONSE_FORMAT,], },',
          '];',
        ],),
      },
      {
        file: 'corpus-pass/src/corpus-run/pass-schema-check.ts',
        text: textOf([
          'export function assertSchemasSendable({ producers, }: { readonly producers: readonly unknown[]; },): void {',
          '  void producers;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/gate-check.ts',
        text: textOf([
          'import { assertSchemasSendable, } from \'../../corpus-pass/src/corpus-run/pass-schema-check.ts\';',
          'import { GATE_PRODUCERS, } from \'./gate-producers.ts\';',
          'export function checkProducers() {',
          '  assertSchemasSendable({ producers: GATE_PRODUCERS, },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/gate-run.ts',
        text: textOf([
          'import { assertSchemasSendable, } from \'../../corpus-pass/src/corpus-run/pass-schema-check.ts\';',
          'import { GATE_PRODUCERS, } from \'./gate-producers.ts\';',
          'export function run(input: { readonly schemaProducers?: readonly unknown[]; },) {',
          '  assertSchemasSendable({ producers: input.schemaProducers ?? GATE_PRODUCERS, },);',
          '}',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/send-format.ts',
        name: 'SEND_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/send-reader.ts',
        text: textOf([
          SENDER_IMPORT,
          'import { SEND_FORMAT, } from \'./send-format.ts\';',
          'export async function send() {',
          '  return await gatherThroughCache({ responseFormat: SEND_FORMAT, },);',
          '}',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/other-format.ts',
        name: 'OTHER_FORMAT',
        more: ['export const OTHER_COUNT = 1;',],
      },),
      {
        file: 'imaginary/src/other-reader.ts',
        text: textOf([
          'import * as formats from \'./other-format.ts\';',
          'export const kept = Object.freeze(formats.OTHER_COUNT,);',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/colliding-format.ts',
        name: 'COLLIDING_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/colliding-other.ts',
        text: textOf([
          'export const COLLIDING_FORMAT = 1;',
        ],),
      },
      {
        file: 'imaginary/src/colliding-reader.ts',
        text: textOf([
          'import { COLLIDING_FORMAT, } from \'./colliding-other.ts\';',
          'export const kept = Object.freeze(COLLIDING_FORMAT,);',
        ],),
      },
    ],
    found: [
      'imaginary/src/colliding-format.ts',
      'imaginary/src/gate-stage.ts',
      'imaginary/src/other-format.ts',
      'imaginary/src/send-format.ts',
    ],
    builders: [],
    spots: [],
  },
];

//endregion The files that read a constant a builder's value is held by
