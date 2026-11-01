import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { detectsRegexUsage, } from './css-mixin.ts';

await describe({
  name: detectsRegexUsage.name,
  children: [
    describe({
      name: 'new RegExp constructor detection',
      children: [
        it({
          name: 'detects new RegExp with an immediate parenthesis',
          fn: async () => {
            expect(detectsRegexUsage('const re = new RegExp("foo");',),).toBe(true,);
          },
        },),
        it({
          name: 'detects new RegExp with whitespace before the parenthesis',
          fn: async () => {
            expect(detectsRegexUsage('new RegExp\t («x»)',),).toBe(true,);
          },
        },),
        it({
          name: 'does not detect new RegExp when no parenthesis follows',
          fn: async () => {
            expect(detectsRegexUsage('type T = new RegExpLike;',),).toBe(false,);
          },
        },),
        it({
          name: 'does not detect a token at end of input with no parenthesis',
          fn: async () => {
            expect(detectsRegexUsage('x = new RegExp',),).toBe(false,);
          },
        },),
        it({
          name: 'walks past a non-constructor occurrence to a later valid one',
          fn: async () => {
            expect(detectsRegexUsage('new RegExpX new RegExp(y)',),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: 'one-char regex literal detection',
      children: [
        it({
          name: 'detects a single-char regex literal',
          fn: async () => {
            expect(detectsRegexUsage('const re = /a/;',),).toBe(true,);
          },
        },),
        it({
          name: 'detects an escaped one-char body',
          fn: async () => {
            expect(detectsRegexUsage(String.raw`const re = /\./;`,),).toBe(true,);
          },
        },),
        it({
          name: 'does not detect a multi-char body (deliberately narrow shape)',
          fn: async () => {
            expect(detectsRegexUsage('const re = /foo/g;',),).toBe(false,);
          },
        },),
        it({
          name: 'does not detect an empty regex (//)',
          fn: async () => {
            expect(detectsRegexUsage('a // comment',),).toBe(false,);
          },
        },),
        it({
          name: 'does not detect a newline as the one-char body',
          fn: async () => {
            expect(detectsRegexUsage('/\n/',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'negative and stack-safety cases',
      children: [
        it({
          name: 'returns false for the empty string',
          fn: async () => {
            expect(detectsRegexUsage('',),).toBe(false,);
          },
        },),
        it({
          name: 'returns false for index-based parsing without regex',
          fn: async () => {
            expect(detectsRegexUsage('source.indexOf("foo")',),).toBe(false,);
          },
        },),
        it({
          name: 'walks tens of thousands of new RegExp tokens without overflowing',
          fn: async () => {
            const tokenCount = 50_000;
            expect(
              detectsRegexUsage('new RegExpX'.repeat(tokenCount,),),
            ).toBe(false,);
          },
        },),
        it({
          name: 'walks tens of thousands of slashes without overflowing',
          fn: async () => {
            const slashCount = 50_000;
            expect(
              detectsRegexUsage('/'.repeat(slashCount,),),
            ).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
