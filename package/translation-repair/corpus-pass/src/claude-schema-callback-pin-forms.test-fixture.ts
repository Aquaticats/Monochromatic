import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import {
  reportingCase,
  stageFile,
} from './claude-schema-callback-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Callbacks the pin cases of the check hold apart from a change to one line of it
// A callback named for a conversion function that keeps nothing (`Boolean`, `Number`, `String`) is read as that
// function only when nothing binds the name again: no scope around the callback, and no declaration of the file's top
// level, whatever it declares.

/**
 The report for a callback the search cannot read.

 @param line - line of the builder's call

 @param method - method the callback is handed to

 @returns Report in the stage file

 @example
 ```ts
 const spot = unreadSpotAt({ line: 5, method: 'filter', },);
 ```
 */
function unreadSpotAt(
  {
    line,
    method,
  }: {
    readonly line: number;
    readonly method: string;
  },
): BlindSpot {
  return {
    file: 'imaginary/src/callback-pin-stage.ts',
    line,
    shape: 'code-the-search-cannot-read',
    position: `callback of ${method} cannot be read`,
  };
}

/**
 A case with one stage file that the check lists and reports nothing for.

 @param name - name of the case

 @param lines - lines of the stage file after its import

 @returns Case

 @example
 ```ts
 const shapeCase = acceptingCase({ name: 'x', lines: [], },);
 ```
 */
function acceptingCase(
  {
    name,
    lines,
  }: {
    readonly name: string;
    readonly lines: readonly string[];
  },
): ShapeCase {
  return {
    name,
    files: [
      stageFile({
        file: 'imaginary/src/callback-pin-stage.ts',
        lines,
      },),
    ],
    found: ['imaginary/src/callback-pin-stage.ts',],
    builders: [],
    spots: [],
  };
}

/**
 Cases of a callback that holds the schema.
 */
export const CALLBACK_PIN_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A BUILDER\'S RESULT HANDED TO A CONVERSION FUNCTION NAME THAT THE TOP LEVEL OF THE FILE BINDS AGAIN '
      + 'WITH A CONSTANT, since the name is then the constant and not Boolean',
    files: [
      stageFile({
        file: 'imaginary/src/callback-pin-stage.ts',
        lines: [
          'declare function makeFilter(): (s: unknown,) => boolean;',
          'const Boolean = makeFilter();',
          'export function mk() {',
          `  [contestResponseFormat(${ARGUMENTS},),].filter(Boolean,);`,
          '}',
        ],
      },),
    ],
    found: ['imaginary/src/callback-pin-stage.ts',],
    builders: [],
    spots: [
      unreadSpotAt({
        line: 5,
        method: 'filter',
      },),
    ],
  },
  acceptingCase({
    name: 'ACCEPTS A BUILDER\'S RESULT WHOSE CALLBACK KEEPS ONLY THE INDEX, since the index is a number and the value '
      + 'is not handed to it',
    lines: [
      'const kept: unknown[] = [];',
      'export function mk() {',
      `  [contestResponseFormat(${ARGUMENTS},),].forEach((s, index,) => { kept.push(index,); },);`,
      '}',
    ],
  },),
  reportingCase({
    name: 'REPORTS A BUILDER\'S RESULT IN AN ARRAY THAT REDUCE HANDS TO A CALLBACK THAT KEEPS THE ARRAY, the fourth '
      + 'parameter',
    lines: [
      'const kept: unknown[] = [];',
      'export function mk() {',
      `  [contestResponseFormat(${ARGUMENTS},),].reduce(`
        + '(total, s, index, all,) => { kept.push(all,); return total; }, 0,);',
      '}',
    ],
    position: CALL_ARGUMENT,
  },),
  reportingCase({
    name: 'REPORTS A BUILDER\'S RESULT HANDED TO A METHOD THAT TAKES A SECOND ARGUMENT, the value the callback is '
      + 'called with as this, since only reduce reads that argument as the start value',
    lines: [
      'const kept: unknown[] = [];',
      'export function mk() {',
      `  [contestResponseFormat(${ARGUMENTS},),].forEach((s,) => { kept.push(s,); }, {},);`,
      '}',
    ],
    position: CALL_ARGUMENT,
  },),
  reportingCase({
    name: 'REPORTS A BUILDER\'S RESULT WHOSE MEMBER IS HANDED TO A CALL AND NOT CALLED, with no report of a callback',
    lines: [
      'declare function use(s: unknown,): void;',
      'export function mk() {',
      `  use(contestResponseFormat(${ARGUMENTS},).map,);`,
      '}',
    ],
    position: CALL_ARGUMENT,
  },),
  reportingCase({
    name: 'REPORTS A BUILDER\'S RESULT IN AN ARRAY WHOSE METHOD IS NAMED BY A STRING IN BRACKETS AND HANDS IT TO A '
      + 'CALLBACK THAT KEEPS IT',
    lines: [
      'const kept: unknown[] = [];',
      'export function mk() {',
      `  [contestResponseFormat(${ARGUMENTS},),]['forEach']((s,) => { kept.push(s,); },);`,
      '}',
    ],
    position: CALL_ARGUMENT,
  },),
  reportingCase({
    name: 'REPORTS A BUILDER\'S RESULT IN AN ARRAY WHOSE CALLBACK KEEPS THE ARRAY IT IS CALLED WITH THROUGH A REST '
      + 'PARAMETER THAT FOLLOWS A PARAMETER, which holds the index and the array',
    lines: [
      'const kept: unknown[] = [];',
      'export function mk() {',
      `  [contestResponseFormat(${ARGUMENTS},),].forEach((s, ...more) => { kept.push(more,); },);`,
      '}',
    ],
    position: CALL_ARGUMENT,
  },),
  acceptingCase({
    name: 'ACCEPTS A BUILDER\'S RESULT HANDED TO NUMBER AND TO STRING AS THE CALLBACK, which keep nothing of it',
    lines: [
      'export function mk() {',
      `  [contestResponseFormat(${ARGUMENTS},),].map(Number,);`,
      `  [contestResponseFormat(${ARGUMENTS},),].map(String,);`,
      '}',
    ],
  },),
  {
    name: 'REPORTS A BUILDER\'S RESULT HANDED TO A CONVERSION FUNCTION NAME THAT A SCOPE AROUND THE CALLBACK BINDS '
      + 'AGAIN WITH A PARAMETER',
    files: [
      stageFile({
        file: 'imaginary/src/callback-pin-stage.ts',
        lines: [
          'export function mk(Boolean: (s: unknown,) => boolean,) {',
          `  [contestResponseFormat(${ARGUMENTS},),].filter(Boolean,);`,
          '}',
        ],
      },),
    ],
    found: ['imaginary/src/callback-pin-stage.ts',],
    builders: [],
    spots: [
      unreadSpotAt({
        line: 3,
        method: 'filter',
      },),
    ],
  },
  {
    name: 'REPORTS A BUILDER\'S RESULT HANDED TO A CONVERSION FUNCTION NAME THAT THE FILE IMPORTS, since the name is '
      + 'then the import and not Boolean',
    files: [
      stageFile({
        file: 'imaginary/src/callback-pin-stage.ts',
        lines: [
          'import { Boolean, } from \'./callback-pin-conversion.ts\';',
          'export function mk() {',
          `  [contestResponseFormat(${ARGUMENTS},),].filter(Boolean,);`,
          '}',
        ],
      },),
      {
        file: 'imaginary/src/callback-pin-conversion.ts',
        text: 'export const Boolean = (s: unknown,): boolean => s !== undefined;\n',
      },
    ],
    found: ['imaginary/src/callback-pin-stage.ts',],
    builders: [],
    spots: [
      unreadSpotAt({
        line: 4,
        method: 'filter',
      },),
    ],
  },
  reportingCase({
    name: 'REPORTS A BUILDER\'S RESULT IN AN ARRAY WHOSE CALLBACK IS A FUNCTION UNDER A TYPE ASSERTION AND KEEPS IT',
    lines: [
      'const kept: unknown[] = [];',
      'export function mk() {',
      `  [contestResponseFormat(${ARGUMENTS},),].forEach(((s: unknown,) => { kept.push(s,); }) as any,);`,
      '}',
    ],
    position: CALL_ARGUMENT,
  },),
];

//endregion Callbacks the pin cases of the check hold apart from a change to one line of it
