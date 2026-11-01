import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { declaringFile, } from './claude-schema-constant-reader-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CONTEST_IMPORT,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region The places where the walk of an exported constant ends
// The files that read a constant a file exports are followed (`claude-schema-constant-reader-forms.test-fixture.ts`).
// The walk ends without a report where the value ends in a sender, in the one function that checks that each schema
// of a list of producers is sendable, in an export from, which only gives the name to other files, in a member of a
// namespace that is not the constant, and in a name another module exports under the same name. The case holds the
// real shape the family has: a constant read by a sender in its own file, exported again by an index, put in a list of
// producers, and the list handed to the check by two files.

/**
 The import of the one-argument sender the case uses, as a specifier written in a file under `imaginary/src`.
 */
const SENDER_IMPORT =
  'import { gatherThroughCache, } from \'../../corpus-pass/src/corpus-run/pass-consistency-gather.ts\';';

/**
 Cases of the places where the walk of an exported constant ends.
 */
export const CONSTANT_END_CASES: readonly ShapeCase[] = [
  {
    name: 'READS A CONSTANT THAT A FILE EXPORTS IN THE FILES THAT IMPORT IT AND NO OTHER, and ends the walk in a '
      + 'sender, in the one function that checks the producers\' schemas, in an export from, in a member of a '
      + 'namespace that is not the constant, and in a name another module exports',
    files: [
      {
        file: 'corpus-pass/src/corpus-run/pass-consistency-gather.ts',
        text: textOf([
          'export async function gatherThroughCache({ responseFormat, }: { readonly responseFormat: unknown; },) {',
          '  return responseFormat;',
          '}',
        ],),
      },
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

//endregion The places where the walk of an exported constant ends
