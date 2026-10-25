import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { stageFile, } from './claude-schema-callback-forms.test-fixture.ts';
import { ARGUMENTS, } from './claude-schema-destination-forms.test-fixture.ts';
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
];

//endregion Callbacks the pin cases of the check hold apart from a change to one line of it
