import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Where the result of a builder's call goes
// What `claude-schemas.unit.test.ts` feeds the builder search and the check behind it (`blindSpotsOf`) to hold the
// third total: the value of a builder's call must reach a function the search knows as a wrapper, an argument of a
// named sender, or a constant whose every reference does; any other place that takes it is reported. One case per
// class of place: an argument of a call that is no sender, an argument of `new` (here), a store into a member or a
// field, a constant whose reference hands the value on (`claude-schema-destination-store-forms.test-fixture.ts`), and
// the sender itself, by module and by parameter (`claude-schema-destination-sender-forms.test-fixture.ts`).

/**
 The import of the contest builder that opens each stage file.
 */
export const CONTEST_IMPORT =
  'import { contestResponseFormat, } from \'../../lane-contest/src/contest-ballot-wire.ts\';';

/**
 The arguments of a call of the contest builder, as an imaginary file writes them.
 */
export const ARGUMENTS = '{ schemaName: \'call\', verdicts: [], asksArchive: false, }';

/**
 The position the check gives a value that is an argument of a call.
 */
export const CALL_ARGUMENT = 'CallExpression.arguments';

/**
 The place the check reports for a call of a builder whose value goes where nothing answers for it.

 @param file - path of the imaginary file

 @param line - line of the call, counted from one

 @param position - where the value goes

 @returns Report

 @example
 ```ts
 const spot = spotAt({ file: 'imaginary/src/a.ts', line: 4, position: CALL_ARGUMENT, },);
 ```
 */
export function spotAt(
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
    shape: 'builder-result-outside-a-destination',
    position,
  };
}

/**
 Cases of a value that is an argument of a call or of `new`.
 */
export const DESTINATION_CALL_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A BUILDER\'S RESULT HANDED TO A CALL THAT IS NO SENDER, whole, inside a call it is nested in, '
      + 'spread, pushed, handed to a method, or put in an object that a call takes, so the files that call the '
      + 'function around it are not listed and the check reports each call',
    files: [
      {
        file: 'imaginary/src/handed-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          'export function frozen() {',
          `  return Object.freeze(contestResponseFormat(${ARGUMENTS},),);`,
          '}',
          '',
          'export function later() {',
          `  return Promise.resolve(contestResponseFormat(${ARGUMENTS},),);`,
          '}',
          '',
          'export function extended() {',
          `  return withExtra(contestResponseFormat(${ARGUMENTS},), 1,);`,
          '}',
          '',
          'export function copied() {',
          `  return JSON.parse(JSON.stringify(contestResponseFormat(${ARGUMENTS},),),);`,
          '}',
          '',
          'export function widest() {',
          `  return Math.max(...[contestResponseFormat(${ARGUMENTS},),],);`,
          '}',
          '',
          'export function pushed() {',
          '  const out = [];',
          `  out.push(contestResponseFormat(${ARGUMENTS},),);`,
          '  return out;',
          '}',
          '',
          'export class Wrapping {',
          '  wrap(value) {',
          '    return { value, };',
          '  }',
          '  run() {',
          `    return this.wrap(contestResponseFormat(${ARGUMENTS},),);`,
          '  }',
          '}',
          '',
          `export const SENT = send({ responseFormat: contestResponseFormat(${ARGUMENTS},), },);`,
          '',
          'export function holds() {',
          `  const format = contestResponseFormat(${ARGUMENTS},);`,
          '  return send(format,);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/handed-caller.ts',
        text: textOf([
          'import { frozen, } from \'./handed-stage.ts\';',
          '',
          'export const CALLED = frozen();',
        ],),
      },
    ],
    found: ['imaginary/src/handed-stage.ts',],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/handed-stage.ts',
        line: 4,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/handed-stage.ts',
        line: 8,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/handed-stage.ts',
        line: 12,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/handed-stage.ts',
        line: 16,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/handed-stage.ts',
        line: 20,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/handed-stage.ts',
        line: 25,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/handed-stage.ts',
        line: 34,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/handed-stage.ts',
        line: 38,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/handed-stage.ts',
        line: 41,
        position: CALL_ARGUMENT,
      },),
    ],
  },
  {
    name: 'REPORTS A BUILDER\'S RESULT HANDED TO new, whole or inside a list that new takes, and the check reports '
      + 'each call',
    files: [
      {
        file: 'imaginary/src/constructed-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          'export function boxed() {',
          `  return new Box(contestResponseFormat(${ARGUMENTS},),);`,
          '}',
          '',
          'export function table() {',
          `  return new Map([['k', contestResponseFormat(${ARGUMENTS},),],],);`,
          '}',
        ],),
      },
    ],
    found: ['imaginary/src/constructed-stage.ts',],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/constructed-stage.ts',
        line: 4,
        position: 'NewExpression.arguments',
      },),
      spotAt({
        file: 'imaginary/src/constructed-stage.ts',
        line: 8,
        position: 'NewExpression.arguments',
      },),
    ],
  },
];

//endregion Where the result of a builder's call goes
