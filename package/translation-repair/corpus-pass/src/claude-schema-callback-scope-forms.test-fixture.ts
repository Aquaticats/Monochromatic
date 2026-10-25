import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { stageFile, } from './claude-schema-callback-forms.test-fixture.ts';
import { ARGUMENTS, } from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A callback named like a declared function that a nearer scope binds again
// A callback written as a name is read as the function the file declares under that name only when no scope around the
// callback binds the name again: a parameter, a destructured parameter, a constant of the function or a `catch`
// clause makes it another thing there, which the search cannot read, so the call is reported. With no such scope the
// declared function is read.

/**
 The path of the stage file of every case.
 */
const STAGE = 'imaginary/src/callback-scope-stage.ts';

/**
 What to add to the index of a line of a case to get its line in the file: the import before the lines, and one
 because lines are counted from one.
 */
const LINES_BEFORE_CASE_LINES = 2;

/**
 The report for a callback the search cannot read.

 @param line - line of the builder's call

 @returns Report in the stage file

 @example
 ```ts
 const spot = unreadSpotAt({ line: 4, },);
 ```
 */
function unreadSpotAt({ line, }: { readonly line: number; },): BlindSpot {
  return {
    file: STAGE,
    line,
    shape: 'code-the-search-cannot-read',
    position: 'callback of forEach cannot be read',
  };
}

/**
 A case whose one stage file declares a function, then hands a one-element array of a builder's value to `forEach`.

 @param name - name of the case

 @param lines - lines of the stage file after its import, one of which holds the builder's call

 @param isReported - whether the callback is reported as unreadable

 @returns Case that lists the stage file

 @example
 ```ts
 const shapeCase = scopeCase({ name: 'x', lines: [], isReported: true, },);
 ```
 */
function scopeCase(
  {
    name,
    lines,
    isReported,
  }: {
    readonly name: string;
    readonly lines: readonly string[];
    readonly isReported: boolean;
  },
): ShapeCase {
  return {
    name,
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
        unreadSpotAt({
          line: lines.findIndex(function holdsBuilderCall(line,): boolean {
            return line.includes('contestResponseFormat(',);
          },) + LINES_BEFORE_CASE_LINES,
        },),
      ]
      : [],
  };
}

/**
 The line that hands the builder's value to `forEach` with the callback a case names.

 @param callback - callback as written

 @param indent - spaces before the line

 @returns Line of a function body

 @example
 ```ts
 const line = handsTo({ callback: 'cb', indent: '  ', },);
 ```
 */
function handsTo(
  {
    callback,
    indent,
  }: {
    readonly callback: string;
    readonly indent: string;
  },
): string {
  return `${indent}[contestResponseFormat(${ARGUMENTS},),].forEach(${callback},);`;
}

/**
 Cases of a callback named like a function the file declares.
 */
export const CALLBACK_SCOPE_CASES: readonly ShapeCase[] = [
  scopeCase({
    name: 'ACCEPTS A BUILDER\'S RESULT HANDED TO A CALLBACK NAMED FOR A FUNCTION THE FILE DECLARES AT ITS TOP LEVEL, the '
      + 'control of the cases that bind the name again',
    lines: [
      'function cb(_x: unknown,) {}',
      'export function mk() {',
      handsTo({
        callback: 'cb',
        indent: '  ',
      },),
      '}',
    ],
    isReported: false,
  },),
  scopeCase({
    name: 'REPORTS A CALLBACK WHOSE NAME A PARAMETER OF THE FUNCTION BINDS AGAIN, so it is not the function the file '
      + 'declares under that name',
    lines: [
      'function harmless(_x: unknown,) {}',
      'export function mk(harmless: (x: unknown,) => void,) {',
      handsTo({
        callback: 'harmless',
        indent: '  ',
      },),
      '}',
    ],
    isReported: true,
  },),
  scopeCase({
    name: 'REPORTS A CALLBACK WHOSE NAME A DESTRUCTURED PARAMETER BINDS AGAIN',
    lines: [
      'function cb(_x: unknown,) {}',
      'export function mk({ cb, }: { cb: (x: unknown,) => void; },) {',
      handsTo({
        callback: 'cb',
        indent: '  ',
      },),
      '}',
    ],
    isReported: true,
  },),
  scopeCase({
    name: 'REPORTS A CALLBACK WHOSE NAME A CONSTANT OF THE FUNCTION BINDS AGAIN',
    lines: [
      'function cb(_x: unknown,) {}',
      'declare function pick(): (x: unknown,) => void;',
      'export function mk() {',
      '  const cb = pick();',
      handsTo({
        callback: 'cb',
        indent: '  ',
      },),
      '}',
    ],
    isReported: true,
  },),
  scopeCase({
    name: 'REPORTS A CALLBACK WHOSE NAME THE PARAMETER OF A CATCH CLAUSE BINDS AGAIN',
    lines: [
      'function cb(_x: unknown,) {}',
      'export function mk() {',
      '  try {',
      '  } catch (cb: any) {',
      handsTo({
        callback: 'cb',
        indent: '    ',
      },),
      '  }',
      '}',
    ],
    isReported: true,
  },),
];

//endregion A callback named like a declared function that a nearer scope binds again
