import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { stageFile, } from './claude-schema-callback-forms.test-fixture.ts';
import { ARGUMENTS, } from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A function written as the argument of a method that the search does not know
// A method of a list that takes a callback and that the search knows hands the value to the parameters of the callback,
// which the walk follows. Any other method called on the value that is given a function written as an argument may hand
// the value to that function in a way the search does not follow: `sort`, `findLast`, `findIndex`, `reduceRight`,
// `toSorted` and a method of an object that holds the value. The call is reported as a callback that cannot be read.
// A call of a method with no function written as an argument is no such place.

/**
 The path of the stage file of every case.
 */
const STAGE = 'imaginary/src/method-callback-stage.ts';

/**
 What to add to the index of a line of a case to get its line in the file: the import before the lines, and one
 because lines are counted from one.
 */
const LINES_BEFORE_CASE_LINES = 2;

/**
 A case whose one stage file hands a builder's value to a method.

 @param method - name of the method called on the list that holds the value

 @param call - rest of the call after the method name, with its parentheses

 @param isReported - whether the callback is reported as unreadable

 @returns Case that lists the stage file

 @example
 ```ts
 const shapeCase = methodCase({ method: 'sort', call: '((a, b) => 0)', isReported: true, },);
 ```
 */
function methodCase(
  {
    method,
    call,
    isReported,
  }: {
    readonly method: string;
    readonly call: string;
    readonly isReported: boolean;
  },
): ShapeCase {
  /**
   The line of the stage file that holds the builder's call.
   */
  const lines = [
    'export function mk() {',
    `  [contestResponseFormat(${ARGUMENTS},),].${method}${call};`,
    '}',
  ];
  return {
    name: isReported
      ? `REPORTS A BUILDER'S RESULT HANDED TO A FUNCTION WRITTEN AS THE ARGUMENT OF ${method}`
      : `ACCEPTS A BUILDER'S RESULT IN A LIST WHOSE ${method} TAKES NO FUNCTION`,
    files: [
      stageFile({
        file: STAGE,
        lines,
      },),
    ],
    found: [STAGE,],
    builders: [],
    spots: isReported
      ? [
        {
          file: STAGE,
          line: lines.findIndex(function holdsBuilderCall(line,): boolean {
            return line.includes('contestResponseFormat(',);
          },) + LINES_BEFORE_CASE_LINES,
          shape: 'code-the-search-cannot-read',
          position: `callback of ${method} cannot be read`,
        } satisfies BlindSpot,
      ]
      : [],
  };
}

/**
 Cases of a function written as the argument of a method.
 */
export const METHOD_CALLBACK_CASES: readonly ShapeCase[] = [
  methodCase({
    method: 'sort',
    call: '((a, b,) => 0,)',
    isReported: true,
  },),
  methodCase({
    method: 'findLast',
    call: '((a,) => true,)',
    isReported: true,
  },),
  methodCase({
    method: 'findIndex',
    call: '((a,) => true,)',
    isReported: true,
  },),
  methodCase({
    method: 'reduceRight',
    call: '((acc, a,) => acc, 0,)',
    isReported: true,
  },),
  methodCase({
    method: 'toSorted',
    call: '(function (a, b,) { return 0; },)',
    isReported: true,
  },),
  methodCase({
    method: 'includes',
    call: '(1,)',
    isReported: false,
  },),
];

//endregion A function written as the argument of a method that the search does not know
