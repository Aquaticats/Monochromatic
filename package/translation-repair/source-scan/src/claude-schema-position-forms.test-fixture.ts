import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Places a builder's name stands in, and places it does not
// What `claude-schemas.unit.test.ts` feeds the builder search to show where a builder's name counts as a use: wherever
// an expression sits that is not a call (an array element, a value of a property, the operand of `return`, a branch of
// a conditional, the right side of an assignment, a default of a parameter, the callee of `new`, the operand of
// `await`, the body of an arrow function), and not as the key of a class member or a label. Each case is a few
// imaginary files of one line or two, with the contest builder (`contestResponseFormat`) beside them, and the files the
// search must list.

/**
 Cases of a name in a place that is no call and still a use.
 */
export const REFERENCE_POSITION_CASES: readonly FormCase[] = [
  {
    name: 'READS A BUILDER\'S NAME AS A USE IN AN ARRAY, AS THE VALUE OF A PROPERTY WRITTEN IN FULL OR SHORT, AFTER '
      + '`return` AND IN A BRANCH OF A CONDITIONAL',
    files: [
      {
        file: 'imaginary/src/position-array.ts',
        text: textOf(['export const IN_ARRAY = [contestResponseFormat,];',],),
      },
      {
        file: 'imaginary/src/position-property.ts',
        text: textOf(['export const IN_OBJECT = { make: contestResponseFormat, };',],),
      },
      {
        file: 'imaginary/src/position-shorthand.ts',
        text: textOf(['export const SHORTHAND = { contestResponseFormat, };',],),
      },
      {
        file: 'imaginary/src/position-return.ts',
        text: textOf([
          'export function returnsBuilder() {',
          '  return contestResponseFormat;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/position-conditional.ts',
        text: textOf(['export const CHOSEN = process.argv.length ? contestResponseFormat : undefined;',],),
      },
    ],
    found: [
      'imaginary/src/position-array.ts',
      'imaginary/src/position-conditional.ts',
      'imaginary/src/position-property.ts',
      'imaginary/src/position-return.ts',
      'imaginary/src/position-shorthand.ts',
    ],
    builders: [],
  },
  {
    name: 'READS A BUILDER\'S NAME AS A USE ON THE RIGHT OF AN ASSIGNMENT, AS THE DEFAULT OF A PARAMETER, AS THE '
      + 'CALLEE OF `new`, AS THE OPERAND OF `await` AND AS THE BODY OF AN ARROW FUNCTION',
    files: [
      {
        file: 'imaginary/src/position-assigned.ts',
        text: textOf([
          'let assigned: unknown;',
          'assigned = contestResponseFormat;',
          'export const MARK = assigned;',
        ],),
      },
      {
        file: 'imaginary/src/position-default.ts',
        text: textOf([
          'export function withDefault(make = contestResponseFormat,) {',
          '  return make;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/position-new.ts',
        text: textOf(['export const MADE = new contestResponseFormat();',],),
      },
      {
        file: 'imaginary/src/position-await.ts',
        text: textOf(['export const AWAITED = await contestResponseFormat;',],),
      },
      {
        file: 'imaginary/src/position-arrow-body.ts',
        text: textOf(['export const LAZY = () => contestResponseFormat;',],),
      },
    ],
    found: [
      'imaginary/src/position-arrow-body.ts',
      'imaginary/src/position-assigned.ts',
      'imaginary/src/position-await.ts',
      'imaginary/src/position-default.ts',
      'imaginary/src/position-new.ts',
    ],
    builders: [],
  },
];

/**
 Cases of a name in a place that is no use: the key of a member of a class and a label.
 */
export const NOT_A_REFERENCE_CASES: readonly FormCase[] = [
  {
    name: 'NAMES NOTHING IN THE KEY OF A MEMBER OF A CLASS, a field or a method, nor in a label',
    files: [
      {
        file: 'imaginary/src/class-member-keys.ts',
        text: textOf([
          'export class FieldHolder {',
          '  contestResponseFormat = 1;',
          '}',
          '',
          'export class MethodHolder {',
          '  contestResponseFormat() {',
          '    return 2;',
          '  }',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/label-names.ts',
        text: textOf([
          'contestResponseFormat: for (let index = 0; index < 1; index++) {',
          '  continue contestResponseFormat;',
          '}',
        ],),
      },
    ],
    found: [],
    builders: [],
  },
];

//endregion Places a builder's name stands in, and places it does not
