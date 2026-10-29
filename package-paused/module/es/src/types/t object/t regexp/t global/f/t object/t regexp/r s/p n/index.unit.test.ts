/* oxlint-disable no-restricted-syntax/no-regex -- this file tests the function that adds the `g` flag to a regex; every test must construct a regex literal as input. The regex literals here ARE the test fixtures. */

import { types, } from '@monochromatic-dev/module-es';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

const { $, } = types.object.regexp.global.from.object.regexp.sync.named;

await describe({
  name: $.name,
  children: [
    it({
      name: 'adds global flag to regexp without flags',
      fn: async () => {
        const regexp = /test/;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('g',);
        expect(result.global,).toBe(true,);
        expect(result.source,).toBe('test',);
      },
    },),

    it({
      name: 'adds global flag to regexp with existing flags',
      fn: async () => {
        const regexp = /test/i;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('gi',);
        expect(result.global,).toBe(true,);
        expect(result.ignoreCase,).toBe(true,);
        expect(result.source,).toBe('test',);
      },
    },),

    it({
      name: 'preserves global flag if already present',
      fn: async () => {
        const regexp = /test/g;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('g',);
        expect(result.global,).toBe(true,);
        expect(result.source,).toBe('test',);
      },
    },),

    it({
      name: 'preserves global flag with multiple existing flags',
      fn: async () => {
        const regexp = /test/gim;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('gim',);
        expect(result.global,).toBe(true,);
        expect(result.ignoreCase,).toBe(true,);
        expect(result.multiline,).toBe(true,);
        expect(result.source,).toBe('test',);
      },
    },),

    it({
      name: 'works with complex regex patterns',
      fn: async () => {
        const regexp = /^[a-z]+/;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('g',);
        expect(result.source,).toBe('^[a-z]+',);
        expect(result.test('abc',),).toBe(true,);
        expect(result.test('123',),).toBe(false,);
      },
    },),

    it({
      name: 'works with special characters in pattern',
      fn: async () => {
        const regexp = /[$()*+.?[\\\]^{|}]/;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('g',);
        expect(result.source,).toBe(String.raw`[$()*+.?[\\\]^{|}]`,);
      },
    },),

    it({
      name: 'works with unicode characters',
      fn: async () => {
        const regexp = /Hello 世界/;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('g',);
        // Bun (JavaScriptCore) escapes non-ASCII in .source when constructing via new RegExp()
        expect(result.source,).toBe(String.raw`Hello \u4E16\u754C`,);
        expect(result.test('Hello 世界',),).toBe(true,);
      },
    },),

    it({
      name: 'works with escaped characters',
      fn: async () => {
        const regexp = /\d+\.\d+/;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('g',);
        expect(result.source,).toBe(String.raw`\d+\.\d+`,);
      },
    },),

    it({
      name: 'returns new RegExp instance',
      fn: async () => {
        const original = /test/;
        const result = $({ regexp: original, },);

        expect(result,).not.toBe(original,);
        expect(result,).toBeInstanceOf(RegExp,);
      },
    },),

    it({
      name: 'new regexp works with exec for multiple matches',
      fn: async () => {
        const regexp = /\w+/;
        const globalRegexp = $({ regexp, },);

        const text = 'hello world test';
        const matches: RegExpExecArray[] = [];

        let match: RegExpExecArray | null;
        while ((match = globalRegexp.exec(text,)) !== null)
          matches.push(match,);

        expect(matches.length,).toBe(3,);
        expect(matches[0]?.[0],).toBe('hello',);
        expect(matches[1]?.[0],).toBe('world',);
        expect(matches[2]?.[0],).toBe('test',);
      },
    },),

    it({
      name: 'new regexp works with matchAll',
      fn: async () => {
        const regexp = /\d+/;
        const globalRegexp = $({ regexp, },);

        const text = 'test123 foo456 bar789';
        const matches = [...text.matchAll(globalRegexp,),];

        expect(matches.length,).toBe(3,);
        expect(matches[0]?.[0],).toBe('123',);
        expect(matches[1]?.[0],).toBe('456',);
        expect(matches[2]?.[0],).toBe('789',);
      },
    },),

    it({
      name: 'with dotAll flag (s)',
      fn: async () => {
        const regexp = /test.test/s;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('gs',);
        expect(result.dotAll,).toBe(true,);
      },
    },),

    it({
      name: 'with sticky flag (y)',
      fn: async () => {
        const regexp = /test/y;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('gy',);
        expect(result.sticky,).toBe(true,);
      },
    },),

    it({
      name: 'with unicode flag (u)',
      fn: async () => {
        const regexp = /\u{1F600}/u;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('gu',);
        expect(result.unicode,).toBe(true,);
      },
    },),

    it({
      name: 'empty regex gets global flag',
      fn: async () => {
        const regexp = /(?:)/;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('g',);
        expect(result.source,).toBe('(?:)',);
      },
    },),

    it({
      name: 'works with alternation patterns',
      fn: async () => {
        const regexp = /cat|dog|bird/;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('g',);
        expect(result.test('cat',),).toBe(true,);
        result.lastIndex = 0;
        expect(result.test('dog',),).toBe(true,);
        result.lastIndex = 0;
        expect(result.test('bird',),).toBe(true,);
        result.lastIndex = 0;
        expect(result.test('fish',),).toBe(false,);
      },
    },),

    it({
      name: 'works with capturing groups',
      fn: async () => {
        const regexp = /(\d+)-(\d+)-(\d+)/;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('g',);
        const match = result.exec('2024-12-25',);
        expect(match,).not.toBeNull();
        expect(match?.[1],).toBe('2024',);
        expect(match?.[2],).toBe('12',);
        expect(match?.[3],).toBe('25',);
      },
    },),

    it({
      name: 'works with lookaheads',
      fn: async () => {
        const regexp = /\w+(?=\s)/;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('g',);
        expect(result.source,).toBe(String.raw`\w+(?=\s)`,);
      },
    },),

    it({
      name: 'works with lookbehinds',
      fn: async () => {
        const regexp = /(?<=\s)\w+/;
        const result = $({ regexp, },);

        expect(result.flags,).toBe('g',);
        expect(result.source,).toBe(String.raw`(?<=\s)\w+`,);
      },
    },),
  ],
},);

/* oxlint-enable no-restricted-syntax/no-regex */
