import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region The global object read as a value
// A member of the global object computed from anything but a string literal can name `eval` or `Function`, and the
// check reports it. The global object itself can be held in a constant, a default, an array, a conditional, an argument
// or a pattern, and read through that name with a key no one can read, so the check reports the global object
// (`globalThis`, `window`, `self` or `global`) wherever it is written as a whole value that no scope binds again. The
// object of a member (`globalThis.fetch`, `globalThis['fetch']`), the operand of `typeof`, a type, a key of an object, a
// member read off another object and a parameter or constant of the same name are not the global object as a value.

/**
 The position a report gives for the global object written as a whole value.
 */
const GLOBAL_AS_VALUE = 'global object used as a value the search cannot follow';

/**
 The lines of the case that write the global object as a whole value, one place on each.
 */
const REPORTED_LINES: readonly string[] = [
  'export const alias: any = globalThis;',
  'export const { [key]: picked, } = globalThis as any;',
  'export const read = Reflect.get(globalThis, key,);',
  'export const sequenced = (0, globalThis);',
  'export const chosen = flag ? globalThis : {};',
  'export const fallback = maybe ?? globalThis;',
  'export const held = [globalThis,];',
  'export const described = Object.getOwnPropertyDescriptor(globalThis, key,);',
  'export function withDefault(root: any = globalThis,) { return root; }',
  'export const passed = identity(globalThis,);',
  'export const called = (function (this: any,) { return this; }).call(globalThis,);',
  'export const copied = Object.assign({}, globalThis,);',
  'export const onWindow: any = window;',
  'export const onSelf: any = self;',
  'export const onGlobal: any = global;',
];

/**
 The lines of the case that do not write the global object as a whole value.
 */
const ACCEPTED_LINES: readonly string[] = [
  'export const kind = typeof globalThis;',
  'export const fetcher = globalThis.fetch;',
  'export const literal = globalThis[\'fetch\'];',
  'export const keyed = { window: 1, self: 2, global: 3, };',
  'export const members = keyed.window + keyed.self + keyed.global;',
  'export type Root = typeof globalThis;',
  'export function bound(window: any,) { return window; }',
  'export function selfish(this: any,) { const self = this; return self; }',
  'export const local = (() => { const global = 1; return global; })();',
];

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
    file: 'imaginary/src/global-alias-stage.ts',
    line,
    shape: 'code-the-search-cannot-read',
    position: GLOBAL_AS_VALUE,
  };
}

/**
 Cases of the global object written as a whole value.
 */
export const GLOBAL_ALIAS_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS THE GLOBAL OBJECT WRITTEN AS A WHOLE VALUE, in a constant, a pattern with a computed key, the argument '
      + 'of Reflect.get, a comma expression, a conditional, a logical fallback, an array, the argument of '
      + 'getOwnPropertyDescriptor, a parameter default, the argument of a function, of call and of Object.assign, and '
      + 'as window, self and global, and nothing for its members, typeof, a type, a key, a member of another object, and '
      + 'a parameter or constant of the same name',
    files: [
      {
        file: 'imaginary/src/global-alias-stage.ts',
        text: textOf([
          ...REPORTED_LINES,
          ...ACCEPTED_LINES,
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [...REPORTED_LINES.keys(),]
      .map(function spotOfIndex(index,): BlindSpot {
        return globalSpotAt({ line: index + 1, },);
      },),
  },
];

//endregion The global object read as a value
