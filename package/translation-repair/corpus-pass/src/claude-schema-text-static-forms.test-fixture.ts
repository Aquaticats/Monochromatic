import { textCase, } from './claude-schema-text-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region The constructor of a function read by a key reader that is called another way
// `Reflect.get(f, 'constructor')` and `Object.getOwnPropertyDescriptor(f, 'constructor')` read the constructor of a
// function by a key, and the call is the same when the method is called through `call` or `apply`, through
// `Reflect.apply` with the arguments in an array, with the arguments spread from an array written in place, as the last
// operand of a comma, or through a chain of global objects. What stays unread, and is pinned silent here: the arguments
// spread from a name, the arguments of `Reflect.apply` held in a name, and a method bound and called after.

/**
 What a report says of a place that reads the constructor of a value.
 */
const READS_CONSTRUCTOR = 'constructor of a function builds code from text the search cannot read';

/**
 Cases of the constructor of a function read by a key reader called another way.
 */
export const TEXT_STATIC_CASES: readonly ShapeCase[] = [
  textCase({
    name: 'REPORTS Reflect.get CALLED THROUGH call AND apply WITH THE KEY constructor',
    lines: [
      'export const first = Reflect.get.call(null, () => 1, \'constructor\',);',
      'export const second = Reflect.get.apply(null, [() => 1, \'constructor\',],);',
    ],
    expected: [
      {
        line: 1,
        position: READS_CONSTRUCTOR,
      },
      {
        line: 2,
        position: READS_CONSTRUCTOR,
      },
    ],
  },),
  textCase({
    name: 'REPORTS Reflect.get HANDED TO Reflect.apply WITH THE KEY constructor IN THE ARRAY OF ARGUMENTS',
    lines: ['export const read = Reflect.apply(Reflect.get, null, [() => 1, \'constructor\',],);',],
    expected: [
      {
        line: 1,
        position: READS_CONSTRUCTOR,
      },
    ],
  },),
  textCase({
    name: 'REPORTS Reflect.get CALLED WITH ITS ARGUMENTS SPREAD FROM AN ARRAY WRITTEN IN PLACE, THE KEY constructor '
      + 'AMONG THEM',
    lines: ['export const read = Reflect.get(...([() => 1, \'constructor\',] as [any, string,]),);',],
    expected: [
      {
        line: 1,
        position: READS_CONSTRUCTOR,
      },
    ],
  },),
  textCase({
    name: 'REPORTS Reflect.get AS THE LAST OPERAND OF A COMMA, AND Object.getOwnPropertyDescriptor CALLED THROUGH call',
    lines: [
      'export const first = (0, Reflect.get)(() => 1, \'constructor\',);',
      'export const second = Object.getOwnPropertyDescriptor.call(null, () => 1, \'constructor\',);',
    ],
    expected: [
      {
        line: 1,
        position: READS_CONSTRUCTOR,
      },
      {
        line: 2,
        position: READS_CONSTRUCTOR,
      },
    ],
  },),
  textCase({
    name: 'REPORTS Reflect.get READ OFF A CHAIN OF GLOBAL OBJECTS AND CALLED THROUGH call WITH THE KEY constructor',
    lines: ['export const read = (globalThis as any).globalThis.Reflect.get.call(null, () => 1, \'constructor\',);',],
    expected: [
      {
        line: 1,
        position: READS_CONSTRUCTOR,
      },
    ],
  },),
  textCase({
    name: 'ACCEPTS Reflect.get CALLED THROUGH call WITH ANOTHER KEY, AND A METHOD NAMED get OF ANOTHER OBJECT CALLED '
      + 'THROUGH call WITH THE KEY constructor',
    lines: [
      'declare const other: { readonly get: (a: unknown, b: string,) => unknown; };',
      'export const first = Reflect.get.call(null, () => 1, \'name\',);',
      'export const second = other.get.call(null, () => 1, \'constructor\',);',
    ],
    expected: [],
  },),
  textCase({
    name: 'ACCEPTS Reflect.get WITH ITS ARGUMENTS SPREAD FROM A NAME, Reflect.apply WITH THE ARGUMENTS IN A NAME AND A '
      + 'BOUND Reflect.get, limits of the search that the ledger states',
    lines: [
      'declare const pair: [unknown, string,];',
      'export const first = Reflect.get(...pair,);',
      'export const second = Reflect.apply(Reflect.get, null, pair,);',
      'export const third = Reflect.get.bind(null,)(() => 1, \'constructor\',);',
    ],
    expected: [],
  },),
];

//endregion The constructor of a function read by a key reader that is called another way
