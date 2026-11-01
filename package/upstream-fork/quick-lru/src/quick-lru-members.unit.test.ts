/**
 Tests for member attachment: upstream `quick-lru` 7.3.0's
 class-prototype descriptor flags pinned onto the factory shell.
 
 The fuzz sidecar's differential oracle compares these descriptors against
 upstream directly, so any drift here is a parity break.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createQuickLru,
} from '../dist/final/neutral/index.mjs';

/**
 Members upstream attaches as writable class-prototype methods.
 */
const METHOD_MEMBERS: readonly PropertyKey[] = [
  'set',
  'get',
  'has',
  'peek',
  'delete',
  'clear',
  'expiresIn',
  'resize',
  'evict',
  'keys',
  'values',
  'entries',
  'entriesAscending',
  'entriesDescending',
  'forEach',
  'toString',
  Symbol.iterator,
  Symbol.for('nodejs.util.inspect.custom',),
];

/**
 Members upstream attaches as getter-only class-prototype accessors.
 */
const GETTER_MEMBERS: readonly PropertyKey[] = [
  'size',
  'maxSize',
  'maxAge',
  '__oldCache',
  Symbol.toStringTag,
];

await describe({
  name: 'cache member attachment',
  children: [
    it({
      name: 'keeps every member non-enumerable so Object.keys stays empty',
      fn: async () => {
        const lru = createQuickLru<string, number>({
          maxSize: 1,
        },);
        expect(Object.keys(lru,),).toEqual([],);
        /**
         Expected own string member names, in attachment order.
         */
        const expectedStringMembers = [
          ...METHOD_MEMBERS,
          ...GETTER_MEMBERS,
        ].filter(function isStringMember(member: PropertyKey,): boolean {
          return (typeof member) === 'string';
        },);
        /**
         Expected own symbol member keys, in attachment order.
         */
        const expectedSymbolMembers = [
          ...METHOD_MEMBERS,
          ...GETTER_MEMBERS,
        ].filter(function isSymbolMember(member: PropertyKey,): boolean {
          return (typeof member) === 'symbol';
        },);
        expect(Object.getOwnPropertyNames(lru,),).toEqual(expectedStringMembers,);
        expect(Object.getOwnPropertySymbols(lru,),).toEqual(expectedSymbolMembers,);
      },
    },),

    ...METHOD_MEMBERS.map(function mapMethodMember(member: PropertyKey,) {
      return it({
        name: `pins method member ${String(member,)} to upstream class-prototype flags`,
        fn: async () => {
          const lru = createQuickLru<string, number>({
            maxSize: 1,
          },);
          /**
           Own descriptor of the member under comparison.
           */
          const descriptor = Object.getOwnPropertyDescriptor(lru, member,);
          expect(descriptor,).toBeDefined();
          expect(descriptor?.writable,).toBe(true,);
          expect(descriptor?.enumerable,).toBe(false,);
          expect(descriptor?.configurable,).toBe(true,);
          expect(descriptor?.value,).toBeDefined();
        },
      },);
    },),

    ...GETTER_MEMBERS.map(function mapGetterMember(member: PropertyKey,) {
      return it({
        name: `pins getter member ${String(member,)} to upstream class-prototype flags`,
        fn: async () => {
          const lru = createQuickLru<string, number>({
            maxSize: 1,
          },);
          /**
           Own descriptor of the member under comparison.
           */
          const descriptor = Object.getOwnPropertyDescriptor(lru, member,);
          /**
           Descriptor slots keyed by name, so the setter slot is read
           without referencing an unbound method.
           */
          const slots = descriptor as unknown as Record<string, unknown>;
          expect(descriptor,).toBeDefined();
          expect(typeof descriptor?.get,).toBe('function',);
          expect(slots.set,).toBe(undefined,);
          expect(descriptor?.enumerable,).toBe(false,);
          expect(descriptor?.configurable,).toBe(true,);
        },
      },);
    },),
  ],
},);
