import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  STAGE,
  TAKES_LOADER,
  textSpotAt,
} from './claude-schema-text-pin-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region The loader of modules and the binding of Node read where they are no direct call
// `process.getBuiltinModule` loads any module of Node by its name, and `process.binding` reaches the binding that makes
// scripts, so the check reads each of them only as the callee of a call. A call of either with a string that names
// `vm` or `contextify` is reported by its text, and a call with another string is accepted. Any other read of the
// function (a member of it like `call`, `apply` and `bind`, an operand of a comma, an arm of a conditional, a value
// kept by a constant) can call it with a name the search cannot read, so it is reported where it sits. What stays
// unread, and is pinned silent here: a pattern that takes `binding` out of `process`, and `process` under another name
// (a constant, a default or namespace import of `node:process`), a name imported from `node:process`, and
// `internalBinding` read by `call`.

/**
 What a report says of `process.binding` read where it is no direct call.
 */
const TAKES_BINDING = 'binding of Node taken out of a call runs text the search cannot read';

/**
 One report a case expects: the line and what the report says.
 */
type Expected = {
  /**
   Line of the place, counted from one.
   */
  readonly line: number;

  /**
   What the report says.
   */
  readonly position: string;
};

/**
 A case whose one file holds the lines given.

 @param name - name of the case

 @param lines - lines of the file

 @param expected - reports the check gives, in the order of the lines

 @returns Case

 @example
 ```ts
 const shapeCase = textCase({ name: 'x', lines: ['export const a = 1;',], expected: [], },);
 ```
 */
export function textCase(
  {
    name,
    lines,
    expected,
  }: {
    readonly name: string;
    readonly lines: readonly string[];
    readonly expected: readonly Expected[];
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
    spots: expected.map(function spotOf({
      line,
      position,
    },) {
      return textSpotAt({
        line,
        position,
      },);
    },),
  };
}

/**
 Cases of the loader of modules and the binding of Node read where they are no direct call.
 */
export const TEXT_CALL_CASES: readonly ShapeCase[] = [
  textCase({
    name: 'REPORTS process.getBuiltinModule READ BY call, apply AND bind, since the name it is called with is then an '
      + 'argument no check reads',
    lines: [
      'export const first = process.getBuiltinModule.call(process, \'vm\',);',
      'export const second = process.getBuiltinModule.apply(process, [\'vm\',],);',
      'const bound = process.getBuiltinModule.bind(process,);',
      'export const third = bound(\'vm\',);',
    ],
    expected: [
      {
        line: 1,
        position: TAKES_LOADER,
      },
      {
        line: 2,
        position: TAKES_LOADER,
      },
      {
        line: 3,
        position: TAKES_LOADER,
      },
    ],
  },),
  textCase({
    name: 'REPORTS process.getBuiltinModule AS THE LAST OPERAND OF A COMMA AND AS AN ARM OF A CONDITIONAL, and read '
      + 'off the global object',
    lines: [
      'declare const flag: boolean;',
      'export const first = (0, process.getBuiltinModule)(\'vm\',);',
      'export const second = (flag ? process.getBuiltinModule : undefined)!(\'vm\',);',
      'export const third = globalThis.process.getBuiltinModule.call(null, \'vm\',);',
    ],
    expected: [
      {
        line: 2,
        position: TAKES_LOADER,
      },
      {
        line: 3,
        position: TAKES_LOADER,
      },
      {
        line: 4,
        position: TAKES_LOADER,
      },
    ],
  },),
  textCase({
    name: 'REPORTS process.binding READ BY call, apply AND bind, which reach the binding that makes scripts',
    lines: [
      'export const first = process.binding.call(process, \'contextify\',);',
      'export const second = process.binding.apply(process, [\'contextify\',],);',
      'export const third = process.binding.bind(process, \'contextify\',);',
    ],
    expected: [
      {
        line: 1,
        position: TAKES_BINDING,
      },
      {
        line: 2,
        position: TAKES_BINDING,
      },
      {
        line: 3,
        position: TAKES_BINDING,
      },
    ],
  },),
  textCase({
    name: 'REPORTS EACH OF TWO READS OF THE LOADER ON ONE LINE, since a report is kept once for each node and a second '
      + 'read of the same shape on the line is a second place',
    lines: ['export const both = [process.getBuiltinModule, process.getBuiltinModule,];',],
    expected: [
      {
        line: 1,
        position: TAKES_LOADER,
      },
      {
        line: 1,
        position: TAKES_LOADER,
      },
    ],
  },),
  textCase({
    name: 'ACCEPTS process.getBuiltinModule AND process.binding AS THE CALLEE OF A CALL, under a type mark, '
      + 'parentheses or an optional call, when the string names another module or binding',
    lines: [
      'export const first = process.getBuiltinModule(\'node:fs\',);',
      'export const second = (process as any).getBuiltinModule(\'node:fs\',);',
      'export const third = (process.getBuiltinModule)(\'node:fs\',);',
      'export const fourth = process.getBuiltinModule?.(\'node:fs\',);',
      'export const fifth = process.binding(\'natives\',);',
      'export const sixth = (process.binding as any)(\'natives\',);',
    ],
    expected: [],
  },),
  textCase({
    name: 'ACCEPTS A MEMBER NAMED binding OR getBuiltinModule OF ANOTHER OBJECT READ WHERE IT IS NO CALL, since only '
      + 'process has the loader and the binding',
    lines: [
      'declare const options: { readonly binding: unknown; readonly getBuiltinModule: unknown; };',
      'export const kept = [options.binding, options.getBuiltinModule,];',
    ],
    expected: [],
  },),
  textCase({
    name: 'ACCEPTS THE LOADER REACHED THROUGH ANOTHER NAME FOR process OR A NAME IMPORTED FROM node:process, and '
      + 'internalBinding READ BY call, limits of the search that the ledger states',
    lines: [
      'const alias = process;',
      'export const first = alias.getBuiltinModule(\'vm\',);',
      'import renamed from \'node:process\';',
      'export const second = renamed.getBuiltinModule(\'vm\',);',
      'import * as everything from \'node:process\';',
      'export const third = everything.getBuiltinModule(\'vm\',);',
      'import { getBuiltinModule as named, } from \'node:process\';',
      'export const fourth = named(\'vm\',);',
      'declare const internalBinding: (name: string,) => unknown;',
      'export const fifth = internalBinding.call(null, \'contextify\',);',
    ],
    expected: [],
  },),
  textCase({
    name: 'ACCEPTS binding TAKEN OUT OF process BY A PATTERN AND CALLED BY call, a limit of the search that the ledger '
      + 'states',
    lines: [
      'const { binding, } = process;',
      'export const made = binding.call(process, \'contextify\',);',
    ],
    expected: [],
  },),
];

//endregion The loader of modules and the binding of Node read where they are no direct call
