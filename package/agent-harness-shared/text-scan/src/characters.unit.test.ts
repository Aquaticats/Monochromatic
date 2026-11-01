import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  isAlphaNum,
  isDigit,
  isWhitespace,
  isWordChar,
} from './index.ts';

await describe({
  name: 'text-scan character predicates',
  children: [
    describe({
      name: isDigit.name,
      children: [
        it({
          name: 'returns true for ASCII digits',
          fn: async () => {
            expect(isDigit('0',),).toBe(true,);
            expect(isDigit('9',),).toBe(true,);
            expect(isDigit('5',),).toBe(true,);
          },
        },),
        it({
          name: 'returns false for non-digit chars',
          fn: async () => {
            expect(isDigit('a',),).toBe(false,);
            expect(isDigit(' ',),).toBe(false,);
            expect(isDigit('_',),).toBe(false,);
            expect(isDigit('/',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: isAlphaNum.name,
      children: [
        it({
          name: 'accepts letters and digits',
          fn: async () => {
            expect(isAlphaNum('a',),).toBe(true,);
            expect(isAlphaNum('Z',),).toBe(true,);
            expect(isAlphaNum('5',),).toBe(true,);
          },
        },),
        it({
          name: 'rejects underscore, hyphen, whitespace',
          fn: async () => {
            expect(isAlphaNum('_',),).toBe(false,);
            expect(isAlphaNum('-',),).toBe(false,);
            expect(isAlphaNum(' ',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: isWordChar.name,
      children: [
        it({
          name: 'accepts alphanumeric and underscore',
          fn: async () => {
            expect(isWordChar('a',),).toBe(true,);
            expect(isWordChar('Z',),).toBe(true,);
            expect(isWordChar('0',),).toBe(true,);
            expect(isWordChar('_',),).toBe(true,);
          },
        },),
        it({
          name: 'rejects punctuation and whitespace',
          fn: async () => {
            expect(isWordChar('-',),).toBe(false,);
            expect(isWordChar('.',),).toBe(false,);
            expect(isWordChar(' ',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: isWhitespace.name,
      children: [
        it({
          name: 'accepts space, tab, newline, carriage return, form feed, vertical tab',
          fn: async () => {
            expect(isWhitespace(' ',),).toBe(true,);
            expect(isWhitespace('\t',),).toBe(true,);
            expect(isWhitespace('\n',),).toBe(true,);
            expect(isWhitespace('\r',),).toBe(true,);
            expect(isWhitespace('\f',),).toBe(true,);
            expect(isWhitespace('\v',),).toBe(true,);
          },
        },),
        it({
          name: 'rejects non-whitespace',
          fn: async () => {
            expect(isWhitespace('a',),).toBe(false,);
            expect(isWhitespace('0',),).toBe(false,);
            expect(isWhitespace('_',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
