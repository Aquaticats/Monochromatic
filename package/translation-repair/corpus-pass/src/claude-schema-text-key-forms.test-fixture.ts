import { textCase, } from './claude-schema-text-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region The constructor of a function read by a method written with a string, a mark or the global object
// `Reflect.get(f, 'constructor')` and `Object.getOwnPropertyDescriptor(f, 'constructor')` read the constructor of a
// function by a key, and the method is the same function when its name is written as a string
// (`Reflect['get']`), when the object is under a type mark or parentheses, and when it is read off the global object.
// What stays unread, and is pinned silent here: a method that returns every member of a value
// (`Object.getOwnPropertyDescriptors`), and a pattern in an assignment that takes `constructor` out of a value.

/**
 What a report says of a place that reads the constructor of a value.
 */
const READS_CONSTRUCTOR = 'constructor of a function builds code from text the search cannot read';

/**
 Cases of the constructor of a function read by a method written another way.
 */
export const TEXT_KEY_CASES: readonly ShapeCase[] = [
  textCase({
    name: 'REPORTS Reflect.get AND Object.getOwnPropertyDescriptor WITH THE KEY constructor WHEN THE METHOD IS WRITTEN '
      + 'AS A STRING OR A TEMPLATE',
    lines: [
      'export const first = Reflect[\'get\'](() => 1, \'constructor\',);',
      'export const second = Object[\'getOwnPropertyDescriptor\'](() => 1, \'constructor\',);',
      'export const third = Reflect[`get`](() => 1, \'constructor\',);',
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
      {
        line: 3,
        position: READS_CONSTRUCTOR,
      },
    ],
  },),
  textCase({
    name: 'REPORTS Reflect.get AND Object.getOwnPropertyDescriptor WITH THE KEY constructor WHEN THE OBJECT IS UNDER A '
      + 'TYPE MARK OR READ OFF THE GLOBAL OBJECT',
    lines: [
      'export const first = (Reflect as any).get(() => 1, \'constructor\',);',
      'export const second = globalThis.Object.getOwnPropertyDescriptor(() => 1, \'constructor\',);',
      'export const third = (globalThis as any)[\'Reflect\'][\'get\'](() => 1, \'constructor\',);',
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
      {
        line: 3,
        position: READS_CONSTRUCTOR,
      },
    ],
  },),
  textCase({
    name: 'ACCEPTS A METHOD OF Reflect OR OF ANOTHER OBJECT WRITTEN AS A STRING THAT READS NO MEMBER BY A KEY, and a '
      + 'member named Reflect of another object',
    lines: [
      'declare const other: { readonly Reflect: { get(a: unknown, b: string,): unknown; }; };',
      'export const first = Reflect[\'has\'](() => 1, \'constructor\',);',
      'export const second = other.Reflect.get(() => 1, \'constructor\',);',
      'export const third = Reflect[\'get\'](() => 1, \'name\',);',
    ],
    expected: [],
  },),
  textCase({
    name: 'ACCEPTS A METHOD THAT RETURNS EVERY MEMBER OF A VALUE AND A PATTERN IN AN ASSIGNMENT THAT TAKES constructor, '
      + 'limits of the search that the ledger states',
    lines: [
      'export const all = Object.getOwnPropertyDescriptors(() => 1,);',
      'let taken: unknown;',
      '({ constructor: taken, } = () => 1);',
    ],
    expected: [],
  },),
];

//endregion The constructor of a function read by a method written with a string, a mark or the global object
