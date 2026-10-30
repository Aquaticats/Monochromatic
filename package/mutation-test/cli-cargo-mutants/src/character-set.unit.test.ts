import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  hyphenateOutside,
  isPositiveWholeNumber,
  startsThenUses,
  usesOnly,
} from './character-set.ts';

await describe({
  name: '',
  children: [
    describe({
      name: usesOnly.name,
      children: [
        it({
          name: 'accepts a text made of allowed characters',
          fn: async () => {
            expect(usesOnly({ text: 'abba', allowed: 'ab' })).toBe(true);
          },
        }),
        it({
          name: 'rejects a text with one other character',
          fn: async () => {
            expect(usesOnly({ text: 'abca', allowed: 'ab' })).toBe(false);
          },
        }),
        it({
          name: 'rejects the empty text',
          fn: async () => {
            expect(usesOnly({ text: '', allowed: 'ab' })).toBe(false);
          },
        }),
      ],
    }),
    describe({
      name: startsThenUses.name,
      children: [
        it({
          name: 'accepts a one-character text from the first set',
          fn: async () => {
            expect(startsThenUses({ text: 'a', first: 'a', rest: 'b' })).toBe(true);
          },
        }),
        it({
          name: 'accepts a first character from one set and the rest from the other',
          fn: async () => {
            expect(startsThenUses({ text: 'abb', first: 'a', rest: 'b' })).toBe(true);
          },
        }),
        it({
          name: 'rejects a first character that is only in the rest set',
          fn: async () => {
            expect(startsThenUses({ text: 'ba', first: 'a', rest: 'ab' })).toBe(false);
          },
        }),
        it({
          name: 'rejects a later character outside the rest set',
          fn: async () => {
            expect(startsThenUses({ text: 'abc', first: 'a', rest: 'b' })).toBe(false);
          },
        }),
        it({
          name: 'rejects the empty text',
          fn: async () => {
            expect(startsThenUses({ text: '', first: 'a', rest: 'b' })).toBe(false);
          },
        }),
      ],
    }),
    describe({
      name: hyphenateOutside.name,
      children: [
        it({
          name: 'keeps allowed characters and replaces each other character with one hyphen',
          fn: async () => {
            expect(hyphenateOutside({ text: 'a b\nc', allowed: 'abc' })).toBe('a-b-c');
          },
        }),
        it({
          name: 'replaces a character outside the basic plane once, not once per code unit',
          fn: async () => {
            expect(hyphenateOutside({ text: 'a\u{1F600}b', allowed: 'ab' })).toBe('a-b');
          },
        }),
      ],
    }),
    describe({
      name: isPositiveWholeNumber.name,
      children: [
        ['1', true],
        ['900', true],
        ['0', false],
        ['007', false],
        ['1.5', false],
        ['-1', false],
        ['', false],
        ['12345678901', false],
      ].map(function numberCase([text, expected]) {
        return it({
          name: `${JSON.stringify(text)} is ${expected === true ? '' : 'not '}a positive whole number of at most ten digits`,
          fn: async () => {
            expect(isPositiveWholeNumber({ text: String(text), maximumDigits: 10 })).toBe(expected);
          },
        });
      }),
    }),
  ],
});
