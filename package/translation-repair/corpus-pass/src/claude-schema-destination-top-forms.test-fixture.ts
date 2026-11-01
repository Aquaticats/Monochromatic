import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A builder's result held by a constant at the top level
// The case of the third total for a constant at the top level of a file, which is followed like a constant in a
// function: every reference to it must end in a sender, a function the search lists as a builder, or a constant in
// turn. The case holds the second sender of the family, `runWindowedRounds` of the windowed rounds of `stage`, where
// the stages that share the contest builder hand their schema to the bench, with a stand-in at its real path.

/**
 The import of the second sender, as a specifier written in a file under `imaginary/src`.
 */
const ROUNDS_IMPORT =
  'import { runWindowedRounds, } from \'../../stage/src/stage-windowed-rounds.ts\';';

/**
 The case of constants at the top level and of the second sender.
 */
export const DESTINATION_TOP_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A BUILDER\'S RESULT HELD BY A CONSTANT AT THE TOP LEVEL WHOSE REFERENCE HANDS IT TO A CALL THAT '
      + 'IS NO SENDER, and reads the windowed rounds as a sender, a constant that is only exported, and the parameter '
      + 'of the rounds that is no schema',
    files: [
      {
        file: 'stage/src/stage-windowed-rounds.ts',
        text: textOf([
          'export async function runWindowedRounds({ responseFormat, }: { readonly responseFormat: unknown; },) {',
          '  return responseFormat;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/declared-sent-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          ROUNDS_IMPORT,
          '',
          `export const SENT_FORMAT = contestResponseFormat(${ARGUMENTS},);`,
          '',
          'export async function ask() {',
          '  return await runWindowedRounds({ responseFormat: SENT_FORMAT, },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/declared-handed-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          `export const HANDED_FORMAT = contestResponseFormat(${ARGUMENTS},);`,
          '',
          'export function frozen() {',
          '  return Object.freeze(HANDED_FORMAT,);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/declared-listed-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          `export const LISTED = [contestResponseFormat(${ARGUMENTS},),];`,
          '',
          'export function register() {',
          '  keep(LISTED,);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/declared-wrong-parameter-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          ROUNDS_IMPORT,
          '',
          `export const WRONG_FORMAT = contestResponseFormat(${ARGUMENTS},);`,
          '',
          'export async function ask() {',
          '  return await runWindowedRounds({ messages: WRONG_FORMAT, },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/declared-exported-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          `const EXPORTED = contestResponseFormat(${ARGUMENTS},);`,
          'export { EXPORTED, };',
          'export default EXPORTED;',
        ],),
      },
    ],
    found: [
      'imaginary/src/declared-exported-stage.ts',
      'imaginary/src/declared-handed-stage.ts',
      'imaginary/src/declared-listed-stage.ts',
      'imaginary/src/declared-sent-stage.ts',
      'imaginary/src/declared-wrong-parameter-stage.ts',
    ],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/declared-handed-stage.ts',
        line: 3,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/declared-listed-stage.ts',
        line: 3,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/declared-wrong-parameter-stage.ts',
        line: 4,
        position: CALL_ARGUMENT,
      },),
    ],
  },
];

//endregion A builder's result held by a constant at the top level
