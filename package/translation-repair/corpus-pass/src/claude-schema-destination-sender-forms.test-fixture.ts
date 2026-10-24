import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region The one sender the family names
// The case of the third total that holds the list of senders: a sender is a function of a module and one parameter of
// it, where a stage hands the schema it makes to the bench. The case gives the sender's module a stand-in at its real
// path, so a call of it is told from a call of another function of the same name, from a free name, and from another
// parameter of the sender.

/**
 The import of the sender, as a specifier written in a file under `imaginary/src`.
 */
const SENDER_IMPORT =
  'import { gatherThroughCache, } from \'../../corpus-pass/src/corpus-run/pass-consistency-gather.ts\';';

/**
 The case of the sender and of the calls that are not the sender.
 */
export const DESTINATION_SENDER_CASES: readonly ShapeCase[] = [
  {
    name: 'READS THE SENDER THE FAMILY NAMES BY MODULE AND BY PARAMETER AND REPORTS A CALL OF ANOTHER FUNCTION OF THE '
      + 'SAME NAME, A FREE NAME OF IT, ANOTHER PARAMETER OR A NESTED ONE: the one sender is `gatherThroughCache` of '
      + '`corpus-pass/src/corpus-run/pass-consistency-gather.ts`, and its `responseFormat` is the schema the bench '
      + 'is asked with',
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
        file: 'imaginary/src/sent-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          '',
          'export async function ask() {',
          '  return await gatherThroughCache({',
          '    messages: [],',
          `    responseFormat: contestResponseFormat(${ARGUMENTS},),`,
          '  },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/free-sent-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          'export async function ask() {',
          `  return await gatherThroughCache({ responseFormat: contestResponseFormat(${ARGUMENTS},), },);`,
          '}',
        ],),
      },
      {
        file: 'imaginary/src/other-gather.ts',
        text: textOf([
          'export function gatherThroughCache({ responseFormat, }: { readonly responseFormat: unknown; },) {',
          '  return responseFormat;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/other-sent-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          'import { gatherThroughCache, } from \'./other-gather.ts\';',
          '',
          'export async function ask() {',
          `  return await gatherThroughCache({ responseFormat: contestResponseFormat(${ARGUMENTS},), },);`,
          '}',
        ],),
      },
      {
        file: 'imaginary/src/wrong-parameter-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          '',
          'export async function ask() {',
          `  return await gatherThroughCache({ messages: contestResponseFormat(${ARGUMENTS},), },);`,
          '}',
        ],),
      },
      {
        file: 'imaginary/src/nested-parameter-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          '',
          'export async function ask() {',
          '  return await gatherThroughCache({',
          `    options: { responseFormat: contestResponseFormat(${ARGUMENTS},), },`,
          '  },);',
          '}',
        ],),
      },
    ],
    found: [
      'imaginary/src/free-sent-stage.ts',
      'imaginary/src/nested-parameter-stage.ts',
      'imaginary/src/other-sent-stage.ts',
      'imaginary/src/sent-stage.ts',
      'imaginary/src/wrong-parameter-stage.ts',
    ],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/free-sent-stage.ts',
        line: 4,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/other-sent-stage.ts',
        line: 5,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/wrong-parameter-stage.ts',
        line: 5,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/nested-parameter-stage.ts',
        line: 6,
        position: CALL_ARGUMENT,
      },),
    ],
  },
  {
    name: 'REPORTS A CALL OF ANOTHER FUNCTION OF THE SENDER\'S MODULE, A COMPUTED KEY OF THE SENDER AND A SENDER\'S NAME '
      + 'THAT IS AN OBJECT: only a function of the module, by its name, with the parameter as a plain key, is a sender',
    files: [
      {
        file: 'corpus-pass/src/corpus-run/pass-consistency-gather.ts',
        text: textOf([
          'export async function gatherThroughCache({ responseFormat, }: { readonly responseFormat: unknown; },) {',
          '  return responseFormat;',
          '}',
          '',
          'export async function gatherElsewhere({ responseFormat, }: { readonly responseFormat: unknown; },) {',
          '  return responseFormat;',
          '}',
        ],),
      },
      {
        file: 'stage/src/stage-windowed-rounds.ts',
        text: textOf([
          'export const runWindowedRounds = { go() { return 1; }, };',
        ],),
      },
      {
        file: 'imaginary/src/other-function-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          'import { gatherElsewhere, } from \'../../corpus-pass/src/corpus-run/pass-consistency-gather.ts\';',
          '',
          'export async function ask() {',
          `  return await gatherElsewhere({ responseFormat: contestResponseFormat(${ARGUMENTS},), },);`,
          '}',
        ],),
      },
      {
        file: 'imaginary/src/computed-key-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          '',
          'export async function ask() {',
          `  return await gatherThroughCache({ ['responseFormat']: contestResponseFormat(${ARGUMENTS},), },);`,
          '}',
        ],),
      },
      {
        file: 'imaginary/src/object-sender-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          'import { runWindowedRounds, } from \'../../stage/src/stage-windowed-rounds.ts\';',
          '',
          'export async function ask() {',
          `  return await runWindowedRounds({ responseFormat: contestResponseFormat(${ARGUMENTS},), },);`,
          '}',
        ],),
      },
    ],
    found: [
      'imaginary/src/computed-key-stage.ts',
      'imaginary/src/object-sender-stage.ts',
      'imaginary/src/other-function-stage.ts',
    ],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/other-function-stage.ts',
        line: 5,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/computed-key-stage.ts',
        line: 5,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/object-sender-stage.ts',
        line: 5,
        position: CALL_ARGUMENT,
      },),
    ],
  },
];

//endregion The one sender the family names
