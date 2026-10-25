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
 The place the check reports for code made from text, in the stage file.

 @param line - line of the place

 @param position - what the place is

 @returns Report

 @example
 ```ts
 const spot = textSpotAt({ line: 1, position: LOADS_VM, },);
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
    file: STAGE,
    line,
    shape: 'code-the-search-cannot-read',
    position,
  };
}

/**
 What a report says of a place that loads the `vm` module.
 */
const LOADS_VM = 'vm runs text the search cannot read';

/**
 What a report says of a pattern that takes `getBuiltinModule` out of a value, since the function it names can load
 `vm` under any name.
 */
const TAKES_LOADER = 'getBuiltinModule taken out of a value loads modules the search cannot read';

/**
 What a report says of a call of the binding of Node that makes scripts, the route to `vm.Script` without the module.
 */
const BINDS_CONTEXTIFY = 'contextify binding runs text the search cannot read';

/**
 Cases of code made from text and of the global object.
 */
export const TEXT_PIN_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS getBuiltinModule TAKEN OUT OF process BY A PATTERN, since the name it is called by is then free',
    files: [
      {
        file: STAGE,
        text: textOf([
          'const { getBuiltinModule, } = process;',
          'export const vm = getBuiltinModule(\'vm\',);',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      textSpotAt({
        line: 1,
        position: TAKES_LOADER,
      },),
    ],
  },
  {
    name: 'REPORTS process.getBuiltinModule OF vm READ THROUGH THE GLOBAL OBJECT, globalThis.process and '
      + 'global.process',
    files: [
      {
        file: STAGE,
        text: textOf([
          'export const first = globalThis.process.getBuiltinModule(\'vm\',);',
          'export const second = global.process.getBuiltinModule(\'node:vm\',);',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      textSpotAt({
        line: 1,
        position: LOADS_VM,
      },),
      textSpotAt({
        line: 2,
        position: LOADS_VM,
      },),
    ],
  },
  {
    name: 'REPORTS A CALL OF THE contextify BINDING, process.binding and internalBinding, which make scripts '
      + 'without the vm module',
    files: [
      {
        file: STAGE,
        text: textOf([
          'export const first = process.binding(\'contextify\',);',
          'declare const internalBinding: (name: string,) => unknown;',
          'export const second = internalBinding(\'contextify\',);',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      textSpotAt({
        line: 1,
        position: BINDS_CONTEXTIFY,
      },),
      textSpotAt({
        line: 3,
        position: BINDS_CONTEXTIFY,
      },),
    ],
  },
  {
    name: 'REPORTS A CALL OF binding TAKEN OUT OF process, since the binding is called by a free name',
    files: [
      {
        file: STAGE,
        text: textOf([
          'declare const binding: (name: string,) => unknown;',
          'export const made = binding(\'contextify\',);',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      textSpotAt({
        line: 2,
        position: BINDS_CONTEXTIFY,
      },),
    ],
  },
  {
    name: 'ACCEPTS A BINDING OF ANOTHER NAME AND A BINDING NAMED BY A NAME, and getBuiltinModule of a module that runs '
      + 'no text through the global object',
    files: [
      {
        file: STAGE,
        text: textOf([
          'declare const name: string;',
          'export const natives = process.binding(\'natives\',);',
          'export const named = process.binding(name,);',
          'export const fs = globalThis.process.getBuiltinModule(\'node:fs\',);',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [],
  },
  {
    name: 'ACCEPTS A LOADER THAT process.getBuiltinModule WAS ASSIGNED TO, a limit of the search that the ledger '
      + 'states',
    files: [
      {
        file: STAGE,
        text: textOf([
          'const load = process.getBuiltinModule;',
          'export const vm = load(\'node:vm\',);',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [],
  },
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
