import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Code made from text and the global object, as the pin cases of the check hold them
// A read of the key `constructor` is a read of the constructor of a function only through `Reflect.get` itself: a
// `get` of another object, another method of `Reflect` and a method of `Reflect` named by a name are not. The global
// object is a whole value wherever it is written as the key of a member or under an operator other than `typeof`, and
// it is no value as the operand of `typeof` under a mark, or as a constant the file declares.

/**
 The path of the stage file of every case.
 */
const STAGE = 'imaginary/src/text-pin-stage.ts';

/**
 The place the check reports for the global object written as a whole value.

 @param line - line of the place

 @returns Report in the stage file

 @example
 ```ts
 const spot = globalSpotAt({ line: 1, },);
 ```
 */
function globalSpotAt({ line, }: { readonly line: number; },): BlindSpot {
  return {
    file: STAGE,
    line,
    shape: 'code-the-search-cannot-read',
    position: 'global object used as a value the search cannot follow',
  };
}

/**
 Cases of code made from text and of the global object.
 */
export const TEXT_PIN_CASES: readonly ShapeCase[] = [
  {
    name: 'ACCEPTS A READ OF THE KEY constructor THAT IS NO REFLECT.GET, a get of another object, another method of '
      + 'Reflect and a method of Reflect named by a name',
    files: [
      {
        file: STAGE,
        text: textOf([
          'declare const Other: { get(a: unknown, b: string,): unknown; };',
          'declare const get: \'get\';',
          'export const fromOther = Other.get({}, \'constructor\',);',
          'export const checked = Reflect.has({}, \'constructor\',);',
          'export const named = Reflect[get]({}, \'constructor\',);',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [],
  },
  {
    name: 'REPORTS THE GLOBAL OBJECT WRITTEN AS THE KEY OF A MEMBER AND UNDER VOID, since neither reads it as the '
      + 'operand of typeof',
    files: [
      {
        file: STAGE,
        text: textOf([
          'export const keyed = ({} as any)[globalThis];',
          'void globalThis;',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      globalSpotAt({ line: 1, },),
      globalSpotAt({ line: 2, },),
    ],
  },
  {
    name: 'ACCEPTS THE GLOBAL OBJECT AS THE OPERAND OF TYPEOF UNDER A MARK, and a constant the file declares with the '
      + 'name of a global object',
    files: [
      {
        file: STAGE,
        text: textOf([
          'export const kind = typeof (globalThis as any);',
          'const window = { width: 1, };',
          'export const kept = [window,];',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [],
  },
];

//endregion Code made from text and the global object, as the pin cases of the check hold them
