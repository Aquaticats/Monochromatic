import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Closing forms of the places that make code from text and that hold the global object
// The global object is held by an export list (`export { globalThis as gt }`) and by a default export, which are no
// value positions of a reference. Code is made from text by `process.getBuiltinModule('node:vm')` as by an import of
// the module, and by `Object.getOwnPropertyDescriptor` and `Reflect.getOwnPropertyDescriptor` with the key
// `constructor`, as by `Reflect.get`. An import or an export that names types only loads nothing. The key read through
// an alias of `Reflect` is a stated limit of the search, which the case holds.

/**
 The path of the stage file of every case.
 */
const STAGE = 'imaginary/src/closing-stage.ts';

/**
 The report of the check for a place that makes code from text or holds the global object.

 @param line - line of the place

 @param position - report text of the place

 @returns Report in the stage file

 @example
 ```ts
 const spot = unreadSpotAt({ line: 1, position: 'vm runs text the search cannot read', },);
 ```
 */
function unreadSpotAt(
  {
    line,
    position,
  }: {
    readonly line: number;
    readonly position: string;
  },
): BlindSpot {
  return {
    file: STAGE,
    line,
    shape: 'code-the-search-cannot-read',
    position,
  };
}

/**
 A case with one stage file.

 @param name - name of the case

 @param lines - lines of the stage file

 @param spots - reports the check gives

 @returns Case that lists no file

 @example
 ```ts
 const shapeCase = stageCase({ name: 'x', lines: [], spots: [], },);
 ```
 */
function stageCase(
  {
    name,
    lines,
    spots,
  }: {
    readonly name: string;
    readonly lines: readonly string[];
    readonly spots: readonly BlindSpot[];
  },
): ShapeCase {
  return {
    name,
    files: [
      {
        file: STAGE,
        text: textOf(lines,),
      },
    ],
    found: [],
    builders: [],
    spots,
  };
}

/**
 What the check says of a global object held by an export.
 */
const GLOBAL_TEXT = 'global object used as a value the search cannot follow';

/**
 What the check says of the module `vm`.
 */
const VM_TEXT = 'vm runs text the search cannot read';

/**
 What the check says of the constructor of a function.
 */
const CONSTRUCTOR_TEXT = 'constructor of a function builds code from text the search cannot read';

/**
 Cases of the closing forms.
 */
export const CLOSING_CASES: readonly ShapeCase[] = [
  stageCase({
    name: 'REPORTS THE GLOBAL OBJECT HELD BY AN EXPORT LIST, since the list reads it as a whole value that a file can '
      + 'take the constructor of',
    lines: ['export { globalThis as gt };',],
    spots: [unreadSpotAt({
      line: 1,
      position: GLOBAL_TEXT,
    },),],
  },),
  stageCase({
    name: 'REPORTS THE GLOBAL OBJECT HELD BY A DEFAULT EXPORT',
    lines: ['export default globalThis;',],
    spots: [unreadSpotAt({
      line: 1,
      position: GLOBAL_TEXT,
    },),],
  },),
  stageCase({
    name: 'ACCEPTS AN EXPORT LIST AND A DEFAULT EXPORT OF A NAME THAT IS NO GLOBAL OBJECT, or of a constant that takes '
      + 'the name of one',
    lines: [
      'const window = { width: 1, };',
      'const other = 1;',
      'export { other as gt, window as win };',
      'export default other;',
    ],
    spots: [],
  },),
  stageCase({
    name: 'REPORTS THE LOAD OF vm THROUGH process.getBuiltinModule, as an import of the module is reported',
    lines: ['export const vm = process.getBuiltinModule(\'node:vm\');',],
    spots: [unreadSpotAt({
      line: 1,
      position: VM_TEXT,
    },),],
  },),
  stageCase({
    name: 'ACCEPTS AN EXPORT OF THE NAME OF THE GLOBAL OBJECT FROM ANOTHER MODULE, which names an export of that module',
    lines: ['export { globalThis as other } from \'node:process\';',],
    spots: [],
  },),
  stageCase({
    name: 'ACCEPTS getBuiltinModule OF AN OBJECT THAT IS NOT process, even for vm',
    lines: [
      'declare const loader: { getBuiltinModule(name: string,): unknown; };',
      'export const vm = loader.getBuiltinModule(\'node:vm\');',
    ],
    spots: [],
  },),
  stageCase({
    name: 'ACCEPTS process.getBuiltinModule OF A MODULE THAT RUNS NO TEXT',
    lines: ['export const fs = process.getBuiltinModule(\'node:fs\');',],
    spots: [],
  },),
  stageCase({
    name: 'REPORTS THE KEY constructor READ BY getOwnPropertyDescriptor OF Object AND OF Reflect',
    lines: [
      'export const fromObject = Object.getOwnPropertyDescriptor(() => 1, \'constructor\',);',
      'export const fromReflect = Reflect.getOwnPropertyDescriptor(() => 1, \'constructor\',);',
    ],
    spots: [
      unreadSpotAt({
        line: 1,
        position: CONSTRUCTOR_TEXT,
      },),
      unreadSpotAt({
        line: 2,
        position: CONSTRUCTOR_TEXT,
      },),
    ],
  },),
  stageCase({
    name: 'ACCEPTS getOwnPropertyDescriptor OF ANOTHER KEY',
    lines: ['export const named = Object.getOwnPropertyDescriptor(() => 1, \'name\',);',],
    spots: [],
  },),
  stageCase({
    name: 'HOLDS THE LIMIT THAT A METHOD OF Reflect READ THROUGH AN ALIAS IS NOT TOLD, since the search reads names '
      + 'and does not follow what a constant holds',
    lines: [
      'const R = Reflect;',
      'export const read = R.get(() => 1, \'constructor\',);',
    ],
    spots: [],
  },),
  stageCase({
    name: 'ACCEPTS AN IMPORT OF TYPES FROM vm AND AN EXPORT OF TYPES FROM vm, which load nothing',
    lines: [
      'import type { Script } from \'node:vm\';',
      'export type { Context } from \'node:vm\';',
      'export type Held = Script;',
    ],
    spots: [],
  },),
];

//endregion Closing forms of the places that make code from text and that hold the global object
