import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { hasUnescapedCommentClose, } from './tag-escaping.ts';

/** Count of repeated closers used to exercise the walk across many matches. */
const LONG_RUN = 1_000;

await describe({
  name: hasUnescapedCommentClose.name,
  children: [
    it({
      name: 'returns false for empty string',
      fn: async () => {
        expect(hasUnescapedCommentClose('',),).toBe(false,);
      },
    },),
    it({
      name: 'returns false when no comment closer is present',
      fn: async () => {
        expect(hasUnescapedCommentClose('plain content',),).toBe(false,);
      },
    },),
    it({
      name: 'returns true when the line begins with an unescaped closer',
      fn: async () => {
        expect(hasUnescapedCommentClose('*/',),).toBe(true,);
      },
    },),
    it({
      name: 'returns true for an unescaped closer mid-line',
      fn: async () => {
        expect(hasUnescapedCommentClose('a */ b',),).toBe(true,);
      },
    },),
    it({
      name: 'returns false for a backslash-escaped closer',
      fn: async () => {
        // value: \*/  (backslash precedes the closer)
        expect(hasUnescapedCommentClose(String.raw`\*/`,),).toBe(false,);
      },
    },),
    it({
      name: 'returns true when an unescaped closer follows an escaped one',
      fn: async () => {
        // value: \*/ */
        expect(hasUnescapedCommentClose(String.raw`\*/ */`,),).toBe(true,);
      },
    },),
    it({
      name: 'returns false when every closer is escaped',
      fn: async () => {
        // value: \*/\*/
        expect(hasUnescapedCommentClose(String.raw`\*/\*/`,),).toBe(false,);
      },
    },),
    it({
      name: 'returns false across a long run of escaped closers',
      fn: async () => {
        /** Long escaped comment closers used to prove linear scanning. */
        const longEscaped = String.raw`\*/ `.repeat(LONG_RUN,);
        expect(hasUnescapedCommentClose(longEscaped,),).toBe(false,);
      },
    },),
    it({
      name: 'returns true when a long escaped run ends with an unescaped closer',
      fn: async () => {
        /** Long run of escaped closers; the trailing unescaped closer is appended below. */
        const longEscaped = String.raw`\*/ `.repeat(LONG_RUN,);
        expect(hasUnescapedCommentClose(`${longEscaped}*/`,),).toBe(true,);
      },
    },),
  ],
},);
