import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Code made from text by the vm module or by the constructor of a function
// Besides `eval` and `Function`, code can be made from text by the `vm` module (`node:vm` or `vm`, imported, exported
// from, required or loaded with `import()`) and by the constructor of a function, which is the `Function` constructor
// itself (`(() => 1).constructor('return 1')`). The check reports each import, export from, `require` and `import()`
// of `vm` or `node:vm`, and each read of a member named `constructor`, whether written with a dot, with a string
// literal in brackets, with `?.`, or as `Reflect.get(value, 'constructor')`. A member with a key that is no literal,
// a pattern that takes `constructor` out of a value, a key of an object, the `constructor` of a class and a type are
// not reported; the first two are limits of the search that the ledger states.

/**
 What a report says of a place that loads the `vm` module.
 */
const LOADS_VM = 'vm runs text the search cannot read';

/**
 What a report says of a place that reads the constructor of a value.
 */
const READS_CONSTRUCTOR = 'constructor of a function builds code from text the search cannot read';

/**
 The lines of the case that load `vm` or read a constructor, one place on each.
 */
const REPORTED_LINES: readonly string[] = [
  'import vm from \'node:vm\';',
  'import { runInNewContext, } from \'vm\';',
  'export * from \'node:vm\';',
  'export { Script, } from \'vm\';',
  'export const required = require(\'vm\',);',
  'export const requiredWithPrefix = require(\'node:vm\',);',
  'export const loaded = (await import(\'node:vm\',)).runInThisContext(\'1\',);',
  'export const made = (() => 1).constructor(\'return 1\',)();',
  'export const bracketed = (() => 1)[\'constructor\'](\'return 1\',);',
  'export const reflected = Reflect.get(Object, \'constructor\',);',
  'export const prototyped = Object.getPrototypeOf(async function () {}).constructor(\'return 1\',);',
  'export const optional = (() => 1)?.constructor;',
];

/**
 The position each reported line gives, in the order of the lines.
 */
const REPORTED_POSITIONS: readonly string[] = [
  LOADS_VM,
  LOADS_VM,
  LOADS_VM,
  LOADS_VM,
  LOADS_VM,
  LOADS_VM,
  LOADS_VM,
  READS_CONSTRUCTOR,
  READS_CONSTRUCTOR,
  READS_CONSTRUCTOR,
  READS_CONSTRUCTOR,
  READS_CONSTRUCTOR,
];

/**
 The lines of the case that neither load `vm` nor read a constructor.
 */
const ACCEPTED_LINES: readonly string[] = [
  'import path from \'node:path\';',
  'import { spawn, } from \'node:child_process\';',
  'export const other = target.constructors;',
  'export class Holder { constructor(public readonly value: number,) {} }',
  'export const keyed = { constructor: 1, };',
  'export const dynamic = target[key];',
  'export const reflectedDynamic = Reflect.get(target, key,);',
  'export const reflectedOther = Reflect.get(target, \'name\',);',
  'export const word = \'vm node:vm constructor\';',
  'export const { constructor: Taken, } = target;',
  'export type Name = Target[\'constructor\'];',
];

/**
 The place the check reports for code made from text.

 @param line - line of the place

 @param position - what the place is

 @returns Report in the stage file

 @example
 ```ts
 const spot = textSpotAt({ line: 1, position: 'vm runs text the search cannot read', },);
 ```
 */
function textSpotAt(
  {
    line,
    position,
  }: {
    readonly line: number;
    readonly position: string;
  },
): BlindSpot {
  return {
    file: 'imaginary/src/code-from-text-stage.ts',
    line,
    shape: 'code-the-search-cannot-read',
    position,
  };
}

/**
 Cases of code made from text by the `vm` module or by the constructor of a function.
 */
export const CODE_FROM_TEXT_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS node:vm AND vm IMPORTED, EXPORTED FROM, REQUIRED AND LOADED, AND A READ OF constructor by a dot, by '
      + 'a string in brackets, by ?. and by Reflect.get with a string, and nothing for node:path, node:child_process, '
      + 'a member of another name, the constructor of a class, a key, a key that is no literal, a pattern that takes '
      + 'constructor out, a string and a type',
    files: [
      {
        file: 'imaginary/src/code-from-text-stage.ts',
        text: textOf([
          ...REPORTED_LINES,
          ...ACCEPTED_LINES,
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: REPORTED_POSITIONS.map(function spotOfPosition(
      position,
      index,
    ): BlindSpot {
      return textSpotAt({
        line: index + 1,
        position,
      },);
    },),
  },
];

//endregion Code made from text by the vm module or by the constructor of a function
