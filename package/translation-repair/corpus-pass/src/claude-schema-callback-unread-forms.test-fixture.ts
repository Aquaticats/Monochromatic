import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { stageFile, } from './claude-schema-callback-forms.test-fixture.ts';
import { ARGUMENTS, } from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A value handed to a callback the search cannot read, and to one that keeps nothing
// The cases beside `claude-schema-callback-forms.test-fixture.ts` that end the walk of a callback: a callback that is any
// expression but a function written there or the name of a function the file declares is code the search cannot read,
// and the check reports it where the builder's call is. A callback that keeps nothing of its parameter ends the walk
// with no report.

/**
 The report for a builder's value handed to a callback the search cannot read.

 @param file - path of the file that holds the builder's call

 @param line - line of the builder's call

 @returns Report at the line of the builder's call

 @example
 ```ts
 const spot = unreadAt({ file: 'imaginary/src/a.ts', line: 3, },);
 ```
 */
function unreadAt(
  {
    file,
    line,
  }: {
    readonly file: string;
    readonly line: number;
  },
): BlindSpot {
  return {
    file,
    line,
    shape: 'code-the-search-cannot-read',
    position: 'callback of forEach cannot be read',
  };
}

/**
 Cases of a builder's value handed to a callback the search cannot read, and to one that keeps nothing.
 */
export const UNREAD_CALLBACK_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A BUILDER\'S RESULT HANDED TO A CALLBACK THE SEARCH CANNOT READ, a parameter, a declared function with '
      + 'no body, a computed member, a member, a name another file exports, and a call that makes the callback',
    files: [
      stageFile({
        file: 'imaginary/src/callback-parameter.ts',
        lines: [
          'export function mk(callback: (s: unknown,) => void,) {',
          `  [contestResponseFormat(${ARGUMENTS},),].forEach(callback,);`,
          '}',
        ],
      },),
      stageFile({
        file: 'imaginary/src/callback-declared.ts',
        lines: [
          'declare function leak(s: unknown,): void;',
          'export function mk() {',
          `  [contestResponseFormat(${ARGUMENTS},),].forEach(leak,);`,
          '}',
        ],
      },),
      stageFile({
        file: 'imaginary/src/callback-computed.ts',
        lines: [
          'const handlers: Record<string, (s: unknown,) => void> = {};',
          'export function mk(key: string,) {',
          `  [contestResponseFormat(${ARGUMENTS},),].forEach(handlers[key],);`,
          '}',
        ],
      },),
      stageFile({
        file: 'imaginary/src/callback-member.ts',
        lines: [
          'const handlers = { keep(s: unknown,) {}, };',
          'export function mk() {',
          `  [contestResponseFormat(${ARGUMENTS},),].forEach(handlers.keep,);`,
          '}',
        ],
      },),
      stageFile({
        file: 'imaginary/src/callback-imported.ts',
        lines: [
          'import { keep, } from \'./callback-keeper.ts\';',
          'export function mk() {',
          `  [contestResponseFormat(${ARGUMENTS},),].forEach(keep,);`,
          '}',
        ],
      },),
      {
        file: 'imaginary/src/callback-keeper.ts',
        text: textOf([
          'export function keep(s: unknown,) {',
          '  console.log(s,);',
          '}',
        ],),
      },
      stageFile({
        file: 'imaginary/src/callback-made.ts',
        lines: [
          'declare function makeKeeper(): (s: unknown,) => void;',
          'export function mk() {',
          `  [contestResponseFormat(${ARGUMENTS},),].forEach(makeKeeper(),);`,
          '}',
        ],
      },),
    ],
    found: [
      'imaginary/src/callback-computed.ts',
      'imaginary/src/callback-declared.ts',
      'imaginary/src/callback-imported.ts',
      'imaginary/src/callback-made.ts',
      'imaginary/src/callback-member.ts',
      'imaginary/src/callback-parameter.ts',
    ],
    builders: [],
    spots: [
      unreadAt({
        file: 'imaginary/src/callback-parameter.ts',
        line: 3,
      },),
      unreadAt({
        file: 'imaginary/src/callback-declared.ts',
        line: 4,
      },),
      unreadAt({
        file: 'imaginary/src/callback-computed.ts',
        line: 4,
      },),
      unreadAt({
        file: 'imaginary/src/callback-member.ts',
        line: 4,
      },),
      unreadAt({
        file: 'imaginary/src/callback-imported.ts',
        line: 4,
      },),
      unreadAt({
        file: 'imaginary/src/callback-made.ts',
        line: 4,
      },),
    ],
  },
  {
    name: 'ACCEPTS A BUILDER\'S RESULT IN AN ARRAY WHOSE CALLBACK KEEPS NOTHING OF IT, one that only drops its parameter, '
      + 'the filter of the array by `Boolean` whose length is kept, and the accumulator of a reduce that has a start value',
    files: [
      stageFile({
        file: 'imaginary/src/callback-quiet.ts',
        lines: [
          'export function mk() {',
          `  [contestResponseFormat(${ARGUMENTS},),].forEach((s,) => { void s; },);`,
          '}',
          'export const COUNT = [',
          `  contestResponseFormat(${ARGUMENTS},),`,
          '].filter(Boolean,).length;',
          'const kept: unknown[] = [];',
          'export function started() {',
          `  [contestResponseFormat(${ARGUMENTS},),].reduce((total, s,) => { kept.push(total,); return 0; }, 0,);`,
          '}',
        ],
      },),
    ],
    found: ['imaginary/src/callback-quiet.ts',],
    builders: [],
    spots: [],
  },
];

//endregion A value handed to a callback the search cannot read, and to one that keeps nothing
