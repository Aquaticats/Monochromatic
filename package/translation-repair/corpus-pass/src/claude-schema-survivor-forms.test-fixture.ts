import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Cases that pin the reading of a mark, a load and a cycle
// Four inputs for which the check gives an answer that a small change of its code would change, each one the case of a
// mutant of the check that survived the whole suite: the marks around a value the check reads through (an optional
// chain, an angle-bracket assertion), a load that is the computed property of a member, and two modules that export
// everything of each other beside the module that carries a builder. Each case passes with the check as it is.

/**
 The module of the contest builder, as a specifier written in a file under `imaginary/src`.
 */
const CONTEST_SPECIFIER = '../../lane-contest/src/contest-ballot-wire.ts';

/**
 The arguments of a call of the contest builder, as an imaginary file writes them.
 */
const ARGUMENTS = '{ schemaName: \'shape\', verdicts: [], asksArchive: false, }';

/**
 The import of the contest builder, as a file under `imaginary/src` writes it.
 */
const CONTEST_IMPORT = `import { contestResponseFormat, } from '${CONTEST_SPECIFIER}';`;

/**
 Cases that pin the reading of a mark, a load and a cycle.
 */
export const SURVIVOR_CASES: readonly ShapeCase[] = [
  {
    name: 'READS THROUGH AN OPTIONAL CALL WHOSE RESULT IS SPREAD INTO A RETURNED OBJECT, and reports the function that '
      + 'returns it',
    files: [
      {
        file: 'imaginary/src/optional-spread-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          'export function wireOf() {',
          `  return { ...contestResponseFormat?.(${ARGUMENTS},), extra: 1, };`,
          '}',
        ],),
      },
    ],
    found: ['imaginary/src/optional-spread-stage.ts',],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/optional-spread-stage.ts',
        line: 3,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
    ],
  },
  {
    name: 'READS THROUGH AN ANGLE-BRACKET ASSERTION OF A BUILDER HELD BY A CONSTANT, and lists the file that calls the '
      + 'constant',
    files: [
      {
        file: 'imaginary/src/asserted-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          'export const make = <Maker>contestResponseFormat;',
        ],),
      },
      {
        file: 'imaginary/src/asserted-caller.ts',
        text: textOf([
          'import { make, } from \'./asserted-stage.ts\';',
          `export const FORMAT = make(${ARGUMENTS},);`,
        ],),
      },
    ],
    found: [
      'imaginary/src/asserted-caller.ts',
      'imaginary/src/asserted-stage.ts',
    ],
    builders: [],
    spots: [],
  },
  {
    name: 'DOES NOT FOLLOW A LOAD THAT IS THE COMPUTED PROPERTY OF A MEMBER, and reports it',
    files: [
      {
        file: 'imaginary/src/computed-load-stage.ts',
        text: textOf([
          `export const FORMAT = registry[await import('${CONTEST_SPECIFIER}')];`,
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/computed-load-stage.ts',
        line: 1,
        shape: 'load-in-unfollowed-form',
        position: 'MemberExpression.property',
      },
    ],
  },
  {
    name: 'ENDS THE WALK OF TWO MODULES THAT EXPORT EVERYTHING OF EACH OTHER BESIDE A MODULE THAT CARRIES A BUILDER, '
      + 'and reads the call through the namespace',
    files: [
      {
        file: 'imaginary/src/cycle-first.ts',
        text: textOf([
          'export * from \'./cycle-second.ts\';',
          `export * from '${CONTEST_SPECIFIER}';`,
        ],),
      },
      {
        file: 'imaginary/src/cycle-second.ts',
        text: textOf([
          'export * from \'./cycle-first.ts\';',
        ],),
      },
      {
        file: 'imaginary/src/cycle-stage.ts',
        text: textOf([
          'import * as first from \'./cycle-first.ts\';',
          `export const FORMAT = first.contestResponseFormat(${ARGUMENTS},);`,
        ],),
      },
    ],
    found: ['imaginary/src/cycle-stage.ts',],
    builders: [],
    spots: [],
  },
];

//endregion Cases that pin the reading of a mark, a load and a cycle
