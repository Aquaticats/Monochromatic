/**
 Equivalence tests for `startsWithUriScheme`.
 
 Lock in the pre-refactor behavior of the URI-scheme detector so the
 recursion-to-linear-pass rewrite stays behavior-identical: the length
 guard, the leading-letter requirement, scheme-body characters, the colon
 terminator, case-insensitivity, the reject paths (digit/symbol start,
 missing colon, non-scheme char before the colon), and a long repeated run
 that a per-character recursion would overflow on a V8 target.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { startsWithUriScheme, } from './url-detect.ts';

/** Run length for the long-input case; large enough to overflow a per-character recursion on V8, trivial for a linear scan. */
const LONG_RUN = 200_000;

await describe({
  name: '',
  children: [
    describe({
      name: startsWithUriScheme.name,
      children: [
        it({
          name: 'returns false for the empty string',
          fn: async () => {
            expect(startsWithUriScheme('',),).toBe(false,);
          },
        },),

        it({
          name: 'returns false for a single character (too short for scheme + colon)',
          fn: async () => {
            expect(startsWithUriScheme('a',),).toBe(false,);
          },
        },),

        it({
          name: 'accepts a minimal one-letter scheme',
          fn: async () => {
            expect(startsWithUriScheme('a:',),).toBe(true,);
          },
        },),

        it({
          name: 'accepts an https URL',
          fn: async () => {
            expect(startsWithUriScheme('https://example.com',),).toBe(true,);
          },
        },),

        it({
          name: 'accepts a data URI',
          fn: async () => {
            expect(startsWithUriScheme('data:image/png;base64,AAAA',),).toBe(true,);
          },
        },),

        it({
          name: 'is case-insensitive on the scheme',
          fn: async () => {
            expect(startsWithUriScheme('HTTPS://EXAMPLE.COM',),).toBe(true,);
          },
        },),

        it({
          name: 'accepts schemes containing +, ., and - body characters',
          fn: async () => {
            expect(startsWithUriScheme('a+b.c-d:x',),).toBe(true,);
          },
        },),

        it({
          name: 'rejects a relative path',
          fn: async () => {
            expect(startsWithUriScheme('./local.png',),).toBe(false,);
          },
        },),

        it({
          name: 'rejects a protocol-relative URL (first char is a slash)',
          fn: async () => {
            expect(startsWithUriScheme('//cdn.example.com/a.js',),).toBe(false,);
          },
        },),

        it({
          name: 'rejects a scheme that starts with a digit',
          fn: async () => {
            expect(startsWithUriScheme('1http:x',),).toBe(false,);
          },
        },),

        it({
          name: 'rejects a scheme that starts with a body symbol',
          fn: async () => {
            expect(startsWithUriScheme('-foo:x',),).toBe(false,);
          },
        },),

        it({
          name: 'rejects when no colon terminates the run',
          fn: async () => {
            expect(startsWithUriScheme('abc',),).toBe(false,);
          },
        },),

        it({
          name: 'rejects when a non-scheme char precedes the colon',
          fn: async () => {
            expect(startsWithUriScheme('foo/bar:baz',),).toBe(false,);
          },
        },),

        it({
          name: 'rejects when whitespace breaks the run before the colon',
          fn: async () => {
            expect(startsWithUriScheme('a b:',),).toBe(false,);
          },
        },),

        it({
          name: 'scans a long scheme body terminated by a colon without overflowing the stack',
          fn: async () => {
            expect(startsWithUriScheme(`${'a'.repeat(LONG_RUN,)}:`,),).toBe(true,);
          },
        },),

        it({
          name: 'returns false for a long body with no terminating colon',
          fn: async () => {
            expect(
              startsWithUriScheme('a'.repeat(LONG_RUN,),),
            ).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
