/**
 Tests for grammatical primitives.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type Capitalization,
  type GrammaticalNumber,
  type Person,
  type PersonNumberKey,
  personNumberKey,
  type Tense,
  type VerbFragmentForm,
} from './grammar-primitives.ts';

await describe({
  name: 'grammar-primitives',
  children: [
    it({
      name: 'personNumberKey joins person and number into a 2-char code',
      fn: async () => {
        expect(personNumberKey({ person: 1, number: 'singular', },),).toBe('1s',);
        expect(personNumberKey({ person: 2, number: 'singular', },),).toBe('2s',);
        expect(personNumberKey({ person: 3, number: 'singular', },),).toBe('3s',);
        expect(personNumberKey({ person: 1, number: 'plural', },),).toBe('1p',);
        expect(personNumberKey({ person: 2, number: 'plural', },),).toBe('2p',);
        expect(personNumberKey({ person: 3, number: 'plural', },),).toBe('3p',);
      },
    },),

    it({
      name: 'personNumberKey return type is the literal union PersonNumberKey',
      fn: async () => {
        /** Output narrowed to the 6-element literal union, not a generic string. */
        const out = personNumberKey({ person: 1, number: 'singular', },);
        expectTypeOf(out,).toEqualTypeOf<PersonNumberKey>();
      },
    },),

    it({
      name: 'Person union covers exactly 1 | 2 | 3',
      fn: async () => {
        expectTypeOf<Person>().toEqualTypeOf<1 | 2 | 3>();
      },
    },),

    it({
      name: 'GrammaticalNumber union covers exactly singular | plural',
      fn: async () => {
        expectTypeOf<GrammaticalNumber>().toEqualTypeOf<'singular' | 'plural'>();
      },
    },),

    it({
      name: 'Tense union covers exactly past | present | future',
      fn: async () => {
        expectTypeOf<Tense>().toEqualTypeOf<'past' | 'present' | 'future'>();
      },
    },),

    it({
      name: 'Capitalization union covers exactly preserve | firstLetter',
      fn: async () => {
        expectTypeOf<Capitalization>().toEqualTypeOf<'preserve' | 'firstLetter'>();
      },
    },),

    it({
      name: 'VerbFragmentForm union covers exactly imperative | infinitive | gerund',
      fn: async () => {
        expectTypeOf<VerbFragmentForm>().toEqualTypeOf<
          'imperative' | 'infinitive' | 'gerund'
          >();
      },
    },),
  ],
},);
