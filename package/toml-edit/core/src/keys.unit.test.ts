/**
 Tests for TOML key encoding (`encodeKey`).
 
 Captures the bare-vs-quoted decision and the basic-string escaping so the
 linear `isBareKey` scanner stays equivalent to the prior recursive walker.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { _encodeKey as encodeKey, } from '@monochromatic-dev/toml-edit-core';

/** Length of the repeated-character keys exercising the long-input path. */
const longRunLength = 100_000;

await describe({
  name: encodeKey.name,
  children: [
    it({
      name: 'bare key of letters returns unquoted',
      fn: async () => {
        expect(encodeKey({ key: 'tools', },),).toBe('tools',);
      },
    },),

    it({
      name: 'single bare char returns unquoted',
      fn: async () => {
        expect(encodeKey({ key: 'a', },),).toBe('a',);
      },
    },),

    it({
      name: 'ASCII letter and digit endpoints remain bare keys',
      fn: async () => {
        for (const key of ['A', 'Z', 'a', 'z', '0', '9', 'Az9',])
          expect(encodeKey({ key, },),).toBe(key,);
      },
    },),

    it({
      name: 'digits, underscore, and hyphen are bare',
      fn: async () => {
        expect(encodeKey({ key: 'a_b-2', },),).toBe('a_b-2',);
        expect(encodeKey({ key: '123', },),).toBe('123',);
      },
    },),

    it({
      name: 'empty key quotes to an empty basic string',
      fn: async () => {
        expect(encodeKey({ key: '', },),).toBe('""',);
      },
    },),

    it({
      name: 'interior space forces quoting',
      fn: async () => {
        expect(encodeKey({ key: 'my key', },),).toBe('"my key"',);
      },
    },),

    it({
      name: 'dot forces quoting',
      fn: async () => {
        expect(encodeKey({ key: 'a.b', },),).toBe('"a.b"',);
      },
    },),

    it({
      name: 'leading and trailing spaces force quoting',
      fn: async () => {
        expect(encodeKey({ key: ' a ', },),).toBe('" a "',);
      },
    },),

    it({
      name: 'backslash is escaped inside the quoted form',
      fn: async () => {
        // key is the three chars: a, backslash, b
        expect(encodeKey({ key: String.raw`a\b`, },),).toBe(String.raw`"a\\b"`,);
      },
    },),

    it({
      name: 'double quote is escaped inside the quoted form',
      fn: async () => {
        // key is the three chars: a, double-quote, b
        expect(encodeKey({ key: 'a"b', },),).toBe(String.raw`"a\"b"`,);
      },
    },),

    it({
      name: 'astral (non-ASCII) char forces quoting without escaping',
      fn: async () => {
        expect(encodeKey({ key: 'a😀b', },),).toBe('"a😀b"',);
      },
    },),

    it({
      name: 'a long all-bare run stays unquoted',
      fn: async () => {
        const longKey = 'a'.repeat(longRunLength,);
        expect(encodeKey({ key: longKey, },),).toBe(longKey,);
      },
    },),

    it({
      name: 'a long run ending in one non-bare char quotes',
      fn: async () => {
        const longKey = `${'a'.repeat(longRunLength,)} `;
        expect(encodeKey({ key: longKey, },),).toBe(`"${longKey}"`,);
      },
    },),
  ],
},);
