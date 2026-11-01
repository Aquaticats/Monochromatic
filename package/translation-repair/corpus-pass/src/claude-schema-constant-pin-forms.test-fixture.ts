import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { declaringFile, } from './claude-schema-constant-reader-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Branches of the walk across files that the other cases leave unpinned
// The mutants of round 17 that survived the cases of the round: two modules that export everything of each other lead
// the walk back to a name it has followed, and it must end there; an `export *` carries no default; a member of a
// namespace computed from a name may be the constant; a constant bound inside a function is not the export of the same
// name; and a value that leaves by a function of another file is reported with that file.

/**
 The shape a report gives for a value a function that is no wrapper returns.
 */
const RETURNED = 'builder-result-returned-outside-a-wrapper';

/**
 The place the check reports for a constant read by a file that hands it to a call.
 */
const HANDED_ON = `${CALL_ARGUMENT} in`;

/**
 The cases of the branches of the walk across files that no other case pins.
 */
export const CONSTANT_PIN_CASES: readonly ShapeCase[] = [
  {
    name: 'ENDS THE WALK OF A CONSTANT THAT TWO MODULES EXPORT AGAIN BY export * EACH OF THE OTHER, and reports the '
      + 'file that reads it through them, a member of a namespace computed from a name, and a function of another '
      + 'file that returns it, and reports nothing for a default an export * leaves out or a name a function binds '
      + 'again',
    files: [
      declaringFile({
        file: 'imaginary/src/cycle-format.ts',
        name: 'CYCLE_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/cycle-first.ts',
        text: textOf([
          'export * from \'./cycle-format.ts\';',
          'export * from \'./cycle-second.ts\';',
        ],),
      },
      {
        file: 'imaginary/src/cycle-second.ts',
        text: textOf([
          'export * from \'./cycle-first.ts\';',
        ],),
      },
      {
        file: 'imaginary/src/cycle-reader.ts',
        text: textOf([
          'import { CYCLE_FORMAT, } from \'./cycle-second.ts\';',
          'export const kept = Object.freeze(CYCLE_FORMAT,);',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/computed-format.ts',
        name: 'COMPUTED_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/computed-reader.ts',
        text: textOf([
          'import * as formats from \'./computed-format.ts\';',
          'export function pick(key: string,) {',
          '  return Object.freeze(formats[key],);',
          '}',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/leave-format.ts',
        name: 'LEAVE_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/leave-reader.ts',
        text: textOf([
          'import { LEAVE_FORMAT, } from \'./leave-format.ts\';',
          'export function leaked() {',
          '  return LEAVE_FORMAT;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/default-star-format.ts',
        text: textOf([
          CONTEST_IMPORT,
          `const DEFAULT_STAR_FORMAT = contestResponseFormat(${ARGUMENTS},);`,
          'export default DEFAULT_STAR_FORMAT;',
        ],),
      },
      {
        file: 'imaginary/src/default-star-index.ts',
        text: textOf([
          'export * from \'./default-star-format.ts\';',
        ],),
      },
      {
        file: 'imaginary/src/default-star-reader.ts',
        text: textOf([
          'import chosen from \'./default-star-index.ts\';',
          'export const kept = Object.freeze(chosen,);',
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
        file: 'imaginary/src/shadow-format.ts',
        text: textOf([
          CONTEST_IMPORT,
          'import { runWindowedRounds, } from \'../../stage/src/stage-windowed-rounds.ts\';',
          'export const SHADOW_FORMAT = 1;',
          'export async function ask() {',
          `  const SHADOW_FORMAT = contestResponseFormat(${ARGUMENTS},);`,
          '  return await runWindowedRounds({ responseFormat: SHADOW_FORMAT, },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/shadow-reader.ts',
        text: textOf([
          'import { SHADOW_FORMAT, } from \'./shadow-format.ts\';',
          'export const kept = Object.freeze(SHADOW_FORMAT,);',
        ],),
      },
    ],
    found: [
      'imaginary/src/computed-format.ts',
      'imaginary/src/cycle-format.ts',
      'imaginary/src/default-star-format.ts',
      'imaginary/src/leave-format.ts',
      'imaginary/src/shadow-format.ts',
    ],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/cycle-format.ts',
        line: 2,
        position: `${HANDED_ON} imaginary/src/cycle-reader.ts`,
      },),
      spotAt({
        file: 'imaginary/src/computed-format.ts',
        line: 2,
        position: `${HANDED_ON} imaginary/src/computed-reader.ts`,
      },),
      {
        file: 'imaginary/src/leave-format.ts',
        line: 2,
        shape: RETURNED,
        position: 'FunctionDeclaration returns it in imaginary/src/leave-reader.ts',
      },
    ],
  },
];

//endregion Branches of the walk across files that the other cases leave unpinned
