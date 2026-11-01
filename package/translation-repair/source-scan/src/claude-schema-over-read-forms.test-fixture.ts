import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Files the search lists though they build no schema
// What `claude-schemas.unit.test.ts` feeds the builder search to pin its over-reads, the places where it lists a file
// that builds no answer schema: a class or a constant that is no function and has a builder's name, a constant inside
// a function or the parameter of a `catch` clause that has it, a function with a second `return` that holds the
// builder's call, and a constant that holds the result of mapping a builder over items. Each fails loudly, with a
// demand in the producer list that a named exception answers, and none loses a producer. Each case is a few imaginary
// files, with the contest builder (`contestResponseFormat`) beside them, and the files and builders the search must
// list.

/**
 Cases of a file that is listed though it builds no schema.
 */
export const OVER_READ_CASES: readonly FormCase[] = [
  {
    name: 'READS A CLASS OR A CONSTANT THAT IS NO FUNCTION AND HAS THE BUILDER\'S NAME AS THE BUILDER, which the '
      + 'search does not tell apart and so over-reads',
    files: [
      {
        file: 'imaginary/src/class-named-like-builder.ts',
        text: textOf([
          'export class contestResponseFormat {}',
          '',
          'export const HELD = new contestResponseFormat();',
        ],),
      },
      {
        file: 'imaginary/src/constant-named-like-builder.ts',
        text: textOf([
          'const contestResponseFormat = 1;',
          '',
          'export const COPY = contestResponseFormat;',
        ],),
      },
    ],
    found: [
      'imaginary/src/class-named-like-builder.ts',
      'imaginary/src/constant-named-like-builder.ts',
    ],
    builders: [],
  },
  {
    name: 'READS A CONSTANT OR A FUNCTION INSIDE A FUNCTION AND THE PARAMETER OF A catch CLAUSE THAT HAVE THE '
      + 'BUILDER\'S NAME AS THE BUILDER, which the search does not tell apart and so over-reads (a function that '
      + 'returns the local function\'s call is read as a wrapper)',
    files: [
      {
        file: 'imaginary/src/local-constant-named-like-builder.ts',
        text: textOf([
          'export function localName() {',
          '  const contestResponseFormat = 1;',
          '  return contestResponseFormat;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/local-function-named-like-builder.ts',
        text: textOf([
          'export function localFunction() {',
          '  const contestResponseFormat = () => 1;',
          '  return contestResponseFormat();',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/catch-parameter-named-like-builder.ts',
        text: textOf([
          'try {',
          '  run();',
          '} catch (contestResponseFormat) {',
          '  throw contestResponseFormat;',
          '}',
        ],),
      },
    ],
    found: [
      'imaginary/src/catch-parameter-named-like-builder.ts',
      'imaginary/src/local-constant-named-like-builder.ts',
      'imaginary/src/local-function-named-like-builder.ts',
    ],
    builders: ['localFunction',],
  },
  {
    name: 'READS A FUNCTION WITH A SECOND return THAT HOLDS THE BUILDER\'S CALL AS A WRAPPER, which the search does '
      + 'not tell apart from one that returns the call alone and so over-reads',
    files: [
      {
        file: 'imaginary/src/second-return-wire.ts',
        text: textOf([
          'export function secondReturnResponseFormat(on: boolean, schemaName: string,) {',
          '  if (on) {',
          '    return 1;',
          '  }',
          '  return contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/second-return-caller.ts',
        text: textOf(['export const SECOND = secondReturnResponseFormat(true, \'second\',);',],),
      },
    ],
    found: [
      'imaginary/src/second-return-caller.ts',
      'imaginary/src/second-return-wire.ts',
    ],
    builders: ['secondReturnResponseFormat',],
  },
  {
    name: 'READS A CONSTANT THAT HOLDS THE RESULT OF MAPPING A BUILDER OVER ITEMS AS A WRAPPER, which the search reads '
      + 'as a constant that holds a call taking a function first and so over-reads',
    files: [
      {
        file: 'imaginary/src/mapped-wire.ts',
        text: textOf([
          'export const MAPPED_FORMATS = [\'a\', \'b\',].map((label,) =>',
          '  contestResponseFormat({ schemaName: label, verdicts: [], asksArchive: false, },),);',
        ],),
      },
      {
        file: 'imaginary/src/mapped-caller.ts',
        text: textOf([
          'import { MAPPED_FORMATS, } from \'./mapped-wire.ts\';',
          '',
          'export const COPY = MAPPED_FORMATS;',
        ],),
      },
    ],
    found: [
      'imaginary/src/mapped-caller.ts',
      'imaginary/src/mapped-wire.ts',
    ],
    builders: ['MAPPED_FORMATS',],
  },
];

//endregion Files the search lists though they build no schema
