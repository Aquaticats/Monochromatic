/**
 Tests for `matcher` and `isMatch` matching semantics: wildcards, negation,
 ordering, empty patterns, and option modes.
 
 Ported from upstream `matcher`'s `test.js` (Sindre Sorhus, MIT): every
 documented example and behavioral case below asserts the same verdict
 upstream asserts, through this fork's destructured-options call shape.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isMatch,
  matcher,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'matcher semantics',
  children: [
    //region matcher basics

    it({
      name: 'filters inputs by one pattern',
      fn: async () => {
        expect(matcher({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: ['foo'],
        },),).toEqual([
          'foo',
        ],);
        expect(matcher({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: ['bar'],
        },),).toEqual([
          'bar',
        ],);
      },
    },),

    it({
      name: 'combines wildcards with negation',
      fn: async () => {
        expect(matcher({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: [
            'fo*',
            'ba*',
            '!bar',
          ],
        },),).toEqual([
          'foo',
        ],);
        expect(matcher({
          inputs: [
            'foo',
            'bar',
            'moo',
          ],
          patterns: ['!*o'],
        },),).toEqual([
          'bar',
        ],);
      },
    },),

    it({
      name: 'matches case-insensitively unless enabled',
      fn: async () => {
        expect(matcher({
          inputs: [
            'moo',
            'MOO',
          ],
          patterns: ['*oo'],
          options: { caseSensitive: true, },
        },),).toEqual([
          'moo',
        ],);
        expect(matcher({
          inputs: [
            'moo',
            'MOO',
          ],
          patterns: ['*oo'],
          options: { caseSensitive: false, },
        },),).toEqual([
          'moo',
          'MOO',
        ],);
      },
    },),

    it({
      name: 'matches the documented readme examples',
      fn: async () => {
        expect(matcher({
          inputs: [
            'foo',
            'bar',
            'moo',
          ],
          patterns: [
            '*oo',
            '!foo',
          ],
        },),).toEqual([
          'moo',
        ],);
        expect(matcher({
          inputs: [
            'foo',
            'bar',
            'moo',
          ],
          patterns: ['!*oo'],
        },),).toEqual([
          'bar',
        ],);
        expect(matcher({
          inputs: 'moo',
          patterns: [''],
        },),).toEqual([],);
        expect(matcher({
          inputs: 'moo',
          patterns: [],
        },),).toEqual([],);
        expect(matcher({
          inputs: [''],
          patterns: [''],
        },),).toEqual([
          '',
        ],);
      },
    },),

    it({
      name: 'keeps input order and duplicates',
      fn: async () => {
        expect(matcher({
          inputs: [
            'b',
            'a',
            'c',
            'b',
          ],
          patterns: [
            'a',
            'b',
          ],
        },),).toEqual([
          'b',
          'a',
          'b',
        ],);
        expect(matcher({
          inputs: [
            'b',
            'a',
            'c',
            'b',
          ],
          patterns: ['!c'],
        },),).toEqual([
          'b',
          'a',
          'b',
        ],);
      },
    },),

    it({
      name: 'matches across newlines',
      fn: async () => {
        expect(matcher({
          inputs: ['foo\nbar'],
          patterns: ['foo*'],
        },),).toEqual([
          'foo\nbar',
        ],);
        expect(matcher({
          inputs: ['foo\nbar'],
          patterns: ['foo*r'],
        },),).toEqual([
          'foo\nbar',
        ],);
      },
    },),

    //endregion matcher basics

    //region isMatch basics

    it({
      name: 'matches wildcards at every position',
      fn: async () => {
        expect(isMatch({
          inputs: 'unicorn',
          patterns: 'unicorn',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'MOO',
          patterns: 'MOO',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'unicorn',
          patterns: 'uni*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'UNICORN',
          patterns: 'unicorn',
          options: { caseSensitive: false, },
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'unicorn',
          patterns: '*corn',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'unicorn',
          patterns: 'un*rn',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'foo unicorn bar',
          patterns: '*unicorn*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'unicorn',
          patterns: '*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'UNICORN',
          patterns: 'UNI*',
          options: { caseSensitive: true, },
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'UNICORN',
          patterns: 'unicorn',
          options: { caseSensitive: true, },
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'unicorn',
          patterns: '',
        },),).toBe(false,);
      },
    },),

    it({
      name: 'applies negation with and without wildcards',
      fn: async () => {
        expect(isMatch({
          inputs: 'unicorn',
          patterns: '!unicorn',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'unicorn',
          patterns: '!uni*',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'unicorn',
          patterns: '!tricorn',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'unicorn',
          patterns: '!tri*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'rainbow',
          patterns: '!unicorn',
        },),).toBe(true,);
      },
    },),

    it({
      name: 'matches the documented isMatch readme examples',
      fn: async () => {
        expect(isMatch({
          inputs: 'unicorn',
          patterns: 'uni*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'unicorn',
          patterns: '*corn',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'unicorn',
          patterns: 'un*rn',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'rainbow',
          patterns: '!unicorn',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'foo bar baz',
          patterns: 'foo b* b*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: 'f*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: [
            'a*',
            'b*',
          ],
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'unicorn',
          patterns: [''],
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'unicorn',
          patterns: [],
        },),).toBe(false,);
        expect(isMatch({
          inputs: [],
          patterns: 'bar',
        },),).toBe(false,);
        expect(isMatch({
          inputs: [],
          patterns: [],
        },),).toBe(false,);
        expect(isMatch({
          inputs: [''],
          patterns: [''],
        },),).toBe(true,);
      },
    },),

    it({
      name: 'matches spaces inside one pattern',
      fn: async () => {
        expect(isMatch({
          inputs: 'foo bar baz',
          patterns: 'foo b* b*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'foo bar',
          patterns: 'foo b* b*',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'foo bx bz',
          patterns: 'foo b* b*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'foo b b',
          patterns: 'foo b* b*',
        },),).toBe(true,);
      },
    },),

    it({
      name: 'treats special regex characters as literal',
      fn: async () => {
        expect(isMatch({
          inputs: 'a.b',
          patterns: 'a.b',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'axb',
          patterns: 'a.b',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'a+b',
          patterns: 'a+b',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'a?b',
          patterns: 'a?b',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'a(b)',
          patterns: 'a(b)',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'a[b]',
          patterns: 'a[b]',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'a{b}',
          patterns: 'a{b}',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'a^b$',
          patterns: 'a^b$',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'axb',
          patterns: 'a*b',
        },),).toBe(true,);
      },
    },),

    it({
      name: 'respects negated pattern placement',
      fn: async () => {
        expect(matcher({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: [
            'fo*',
            '!bar',
            'ba*',
          ],
        },),).toEqual([
          'foo',
        ],);
        expect(matcher({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: [
            '!bar',
            'fo*',
            'ba*',
          ],
        },),).toEqual([
          'foo',
        ],);
        expect(matcher({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: ['!bar'],
        },),).toEqual([
          'foo',
        ],);
        expect(matcher({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: [
            '!bar',
            'fu',
          ],
        },),).toEqual([],);
        expect(isMatch({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: [
            'fo*',
            '*oo',
            '!bar',
          ],
        },),).toBe(true,);
        expect(isMatch({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: [
            '!bar',
            'fo*',
            '*oo',
          ],
        },),).toBe(true,);
        expect(isMatch({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: ['!bar'],
        },),).toBe(true,);
      },
    },),

    it({
      name: 'uses OR logic by default across many patterns',
      fn: async () => {
        expect(isMatch({
          inputs: 'foo',
          patterns: [
            'f*',
            'b*',
          ],
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'bar',
          patterns: [
            'f*',
            'b*',
          ],
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'zoo',
          patterns: [
            'f*',
            'b*',
          ],
        },),).toBe(false,);
        expect(isMatch({
          inputs: [
            'foo',
            'zoo',
          ],
          patterns: [
            'f*',
            'b*',
          ],
        },),).toBe(true,);
        expect(isMatch({
          inputs: [
            'zoo',
            'moo',
          ],
          patterns: [
            'f*',
            'b*',
          ],
        },),).toBe(false,);
        /**
         Allowed origins letting either suffix match.
         */
        const allowedOrigins = [
          '*.example.com',
          '*.dev.example.com',
        ];
        expect(isMatch({
          inputs: 'https://my.example.com',
          patterns: allowedOrigins,
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'https://my.dev.example.com',
          patterns: allowedOrigins,
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'https://my.other.com',
          patterns: allowedOrigins,
        },),).toBe(false,);
      },
    },),

    //endregion isMatch basics

    //region allPatterns

    it({
      name: 'filters every input against every pattern under allPatterns',
      fn: async () => {
        /**
         All-patterns flags shared by every case below.
         */
        const options = { allPatterns: true, };
        expect(matcher({
          inputs: 'foo',
          patterns: '!x*',
          options,
        },),).toEqual([
          'foo',
        ],);
        expect(matcher({
          inputs: [
            'foo',
            'bar',
            'for',
          ],
          patterns: [
            'f*',
            'b*',
          ],
          options,
        },),).toEqual([],);
        expect(matcher({
          inputs: [
            'foo',
            'bar',
            'for',
          ],
          patterns: [
            'f*',
            'x*',
          ],
          options,
        },),).toEqual([],);
        expect(matcher({
          inputs: [
            'foo',
            'bar',
            'for',
          ],
          patterns: [
            'f*',
            '!b*',
          ],
          options,
        },),).toEqual([
          'foo',
          'for',
        ],);
        expect(matcher({
          inputs: [
            'foo',
            'bar',
            'for',
          ],
          patterns: [
            'f*',
            '!x*',
          ],
          options,
        },),).toEqual([
          'foo',
          'for',
        ],);
        expect(matcher({
          inputs: [
            'Hey, tiger!',
            'tiger has edge over hyenas',
            'pushing a tiger over the edge is a stunt',
          ],
          patterns: [
            '*edge*',
            '*tiger*',
            '!*stunt*',
          ],
          options,
        },),).toEqual([
          'tiger has edge over hyenas',
        ],);
      },
    },),

    it({
      name: 'answers under allPatterns AND logic',
      fn: async () => {
        /**
         All-patterns flags shared by every case below.
         */
        const options = { allPatterns: true, };
        expect(isMatch({
          inputs: 'foo',
          patterns: '!x*',
          options,
        },),).toBe(true,);
        expect(isMatch({
          inputs: [
            'foo',
            'bar',
            'for',
          ],
          patterns: [
            'f*',
            'b*',
          ],
          options,
        },),).toBe(false,);
        expect(isMatch({
          inputs: [
            'foo',
            'bar',
            'for',
          ],
          patterns: [
            'f*',
            'x*',
          ],
          options,
        },),).toBe(false,);
        expect(isMatch({
          inputs: [
            'foo',
            'bar',
            'for',
          ],
          patterns: [
            'f*',
            '!b*',
          ],
          options,
        },),).toBe(true,);
        expect(isMatch({
          inputs: [
            'foo',
            'bar',
            'for',
          ],
          patterns: [
            'f*',
            '!x*',
          ],
          options,
        },),).toBe(true,);
        expect(isMatch({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: ['!bar'],
          options,
        },),).toBe(true,);
        expect(isMatch({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: ['!bar'],
        },),).toBe(true,);
        expect(isMatch({
          inputs: [
            'Hey, tiger!',
            'tiger has edge over hyenas',
            'pushing a tiger over the edge is a stunt',
          ],
          patterns: [
            '*edge*',
            '*tiger*',
            '!*stunt*',
          ],
          options,
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'foobar',
          patterns: [
            'foo*',
            '*bar',
          ],
          options,
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'foo',
          patterns: [
            'foo*',
            '*bar',
          ],
          options,
        },),).toBe(false,);
      },
    },),

    it({
      name: 'requires every input under allPatterns with only negations',
      fn: async () => {
        /**
         Only-negation patterns shared by every case below.
         */
        const patterns = [
          '!bar',
          '!baz',
        ];
        /**
         All-patterns flags shared by every case below.
         */
        const options = { allPatterns: true, };
        expect(matcher({
          inputs: [
            'foo',
            'bar',
          ],
          patterns,
          options,
        },),).toEqual([
          'foo',
        ],);
        expect(isMatch({
          inputs: [
            'foo',
            'bar',
          ],
          patterns,
          options,
        },),).toBe(false,);
        expect(isMatch({
          inputs: [
            'foo',
            'qux',
          ],
          patterns,
          options,
        },),).toBe(true,);
        expect(isMatch({
          inputs: [],
          patterns,
          options,
        },),).toBe(false,);
      },
    },),

    it({
      name: 'keeps some-branch under allPatterns with a single negation',
      fn: async () => {
        /**
         One negation plus one positive: requiresAllInputs stays false,
         so isMatch needs only one matching input.
         */
        const patterns = [
          '!bar',
          'f*',
        ];
        /**
         All-patterns flags shared by every case below.
         */
        const options = { allPatterns: true, };
        expect(isMatch({
          inputs: [
            'foo',
            'qux',
          ],
          patterns,
          options,
        },),).toBe(true,);
        expect(isMatch({
          inputs: [
            'foo',
            'qux',
          ],
          patterns: [
            'f*',
            '!bar',
          ],
          options,
        },),).toBe(true,);
      },
    },),

    it({
      name: 'combines empty and wildcard patterns under allPatterns',
      fn: async () => {
        expect(isMatch({
          inputs: '',
          patterns: [
            '',
            '*',
          ],
          options: { allPatterns: true, },
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'a',
          patterns: [
            '',
            '*',
          ],
          options: { allPatterns: true, },
        },),).toBe(false,);
        expect(matcher({
          inputs: [
            '',
            'a',
          ],
          patterns: [
            '*',
            '!',
          ],
          options: { allPatterns: true, },
        },),).toEqual([
          'a',
        ],);
      },
    },),

    //endregion allPatterns

    //region Negation edges

    it({
      name: 'applies negation with no positive patterns',
      fn: async () => {
        expect(isMatch({
          inputs: 'foo',
          patterns: ['!bar'],
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'bar',
          patterns: ['!bar'],
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'foo',
          patterns: [
            '!bar',
            '!baz',
            '!qux',
          ],
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'bar',
          patterns: [
            '!bar',
            '!baz',
            '!qux',
          ],
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'anything',
          patterns: ['!*'],
        },),).toBe(false,);
        expect(isMatch({
          inputs: '',
          patterns: ['!*'],
        },),).toBe(false,);
      },
    },),

    it({
      name: 'excludes only the empty string for a lone bang pattern',
      fn: async () => {
        expect(isMatch({
          inputs: 'a',
          patterns: '!',
        },),).toBe(true,);
        expect(isMatch({
          inputs: '',
          patterns: '!',
        },),).toBe(false,);
        expect(matcher({
          inputs: [
            '',
            'a',
            '',
          ],
          patterns: '!',
        },),).toEqual([
          'a',
        ],);
      },
    },),

    it({
      name: 'negates only on a leading bang',
      fn: async () => {
        /**
         Backslash code unit shared by every escaped-bang pattern below.
         */
        const escape = String.fromCodePoint(92,);
        expect(isMatch({
          inputs: '!foo',
          patterns: `${escape}!foo`,
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'foo',
          patterns: `${escape}!foo`,
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'a!b',
          patterns: 'a!*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: '!foo',
          patterns: '!!foo',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'foo',
          patterns: '!!foo',
        },),).toBe(true,);
        expect(isMatch({
          inputs: '!',
          patterns: `${escape}!`,
        },),).toBe(true,);
      },
    },),

    it({
      name: 'settles the same pattern with and without negation',
      fn: async () => {
        expect(isMatch({
          inputs: 'foo',
          patterns: 'foo',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'foo',
          patterns: '!foo',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'foo',
          patterns: 'foo',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'foo',
          patterns: [
            'foo',
            '!foo',
          ],
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'foo',
          patterns: [
            '!foo',
            'foo',
          ],
        },),).toBe(false,);
      },
    },),

    //endregion Negation edges

    //region Wildcard edges

    it({
      name: 'collapses consecutive wildcards into one',
      fn: async () => {
        expect(isMatch({
          inputs: 'test',
          patterns: '**',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'test',
          patterns: '*****',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'a/b/c',
          patterns: '***',
        },),).toBe(true,);
        expect(matcher({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: '**',
        },),).toEqual([
          'foo',
          'bar',
        ],);
      },
    },),

    it({
      name: 'matches empty inputs against wildcard patterns',
      fn: async () => {
        expect(isMatch({
          inputs: '',
          patterns: '*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: '',
          patterns: '!*',
        },),).toBe(false,);
        expect(isMatch({
          inputs: '',
          patterns: '',
        },),).toBe(true,);
        expect(isMatch({
          inputs: '',
          patterns: 'a*',
        },),).toBe(false,);
        expect(isMatch({
          inputs: '',
          patterns: '*a*',
        },),).toBe(false,);
        expect(isMatch({
          inputs: '',
          patterns: '**',
        },),).toBe(true,);
      },
    },),

    it({
      name: 'keeps literal parts from overlapping each other',
      fn: async () => {
        expect(isMatch({
          inputs: 'a',
          patterns: 'a*a',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'aa',
          patterns: 'a*a',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'abc',
          patterns: 'ab*bc',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'abbc',
          patterns: 'ab*bc',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'ab',
          patterns: '*ab*b',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'abb',
          patterns: '*ab*b',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'aba',
          patterns: 'aba*aba',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'abaaba',
          patterns: 'aba*aba',
        },),).toBe(true,);
      },
    },),

    it({
      name: 'orders middle parts without overlap',
      fn: async () => {
        expect(isMatch({
          inputs: 'xaxbx',
          patterns: '*a*b*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'xbxax',
          patterns: '*a*b*',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'aab',
          patterns: '*a*ab',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'abcabc',
          patterns: '*bc*ab*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'abcab',
          patterns: '*bc*abc',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'foo.min.js',
          patterns: '*.min.*js',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'foo.js.min',
          patterns: '*.min.*js',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'a',
          patterns: '*a*a*',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'aa',
          patterns: '*a*a*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'aba',
          patterns: '*ab*ba*',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'abba',
          patterns: '*ab*ba*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'aaa',
          patterns: '*aa*aa*',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'aaaa',
          patterns: '*aa*aa*',
        },),).toBe(true,);
      },
    },),

    it({
      name: 'matches consecutive wildcards around literals',
      fn: async () => {
        expect(isMatch({
          inputs: 'abc',
          patterns: '**b**',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'b',
          patterns: '**b**',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'ac',
          patterns: '**b**',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'ab',
          patterns: 'a***b',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'a',
          patterns: 'a***b',
        },),).toBe(false,);
      },
    },),

    it({
      name: 'matches whitespace inside patterns',
      fn: async () => {
        expect(isMatch({
          inputs: 'hello world',
          patterns: 'hello world',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'hello world',
          patterns: 'hello*world',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'hello   world',
          patterns: 'hello*world',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'helloworld',
          patterns: 'hello world',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'hello\tworld',
          patterns: 'hello*world',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'hello\nworld',
          patterns: 'hello*world',
        },),).toBe(true,);
      },
    },),

    it({
      name: 'matches file patterns from everyday use',
      fn: async () => {
        expect(isMatch({
          inputs: 'node_modules/foo',
          patterns: 'node_modules/*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: '.env.local',
          patterns: '.env*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'src/.env',
          patterns: '.env*',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'test.js',
          patterns: '*.js',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'test.jsx',
          patterns: '*.js',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'test.test.js',
          patterns: '*.test.js',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'component.test.tsx',
          patterns: '*.test.*',
        },),).toBe(true,);
      },
    },),

    it({
      name: 'filters many inputs and patterns at scale',
      fn: async () => {
        /**
         One thousand generated item inputs.
         */
        const inputs = Array.from(
          { length: 1_000, },
          function itemAt(_value: unknown, index: number,): string {
            return `item${String(index,)}`;
          },
        );
        expect(matcher({
          inputs,
          patterns: 'item1*',
        },).length,).toBe(111,);
        expect(matcher({
          inputs,
          patterns: [
            'item1*',
            '!*0',
          ],
        },).length,).toBe(100,);
        expect(matcher({
          inputs,
          patterns: [
            'item*',
            '!item?*',
          ],
        },).length,).toBe(1_000,);
        expect(matcher({
          inputs,
          patterns: [
            '*1*',
            '*2*',
          ],
          options: { allPatterns: true, },
        },).length,).toBe(54,);
        expect(matcher({
          inputs,
          patterns: Array.from(
            { length: 100, },
            function exactAt(_value: unknown, index: number,): string {
              return `item${String(index * 10,)}`;
            },
          ),
        },).length,).toBe(100,);
      },
    },),

    //endregion Wildcard edges

    //region Escapes

    it({
      name: 'keeps escaped wildcards literal',
      fn: async () => {
        /**
         Backslash code unit shared by every escaped pattern below.
         */
        const escape = String.fromCodePoint(92,);
        expect(isMatch({
          inputs: 'unicorn',
          patterns: `uni${escape}*`,
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'uni*',
          patterns: `uni${escape}*`,
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'unixcorn',
          patterns: `uni${escape}*`,
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'a*b*c',
          patterns: `a${escape}*b${escape}*c`,
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'axbxc',
          patterns: `a${escape}*b${escape}*c`,
        },),).toBe(false,);
      },
    },),

    it({
      name: 'escapes spaces and backslashes literally',
      fn: async () => {
        /**
         Backslash code unit shared by every escaped pattern below.
         */
        const escape = String.fromCodePoint(92,);
        expect(isMatch({
          inputs: 'a b',
          patterns: `a${escape} b`,
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'ab',
          patterns: `a${escape} b`,
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'axb',
          patterns: `a${escape} b`,
        },),).toBe(false,);
        expect(isMatch({
          inputs: `test${escape}${escape}`,
          patterns: `test${escape}${escape}${escape}${escape}`,
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'test',
          patterns: `test${escape}${escape}${escape}${escape}`,
        },),).toBe(false,);
      },
    },),

    it({
      name: 'mixes escaped and unescaped wildcards',
      fn: async () => {
        /**
         Backslash code unit shared by every escaped pattern below.
         */
        const escape = String.fromCodePoint(92,);
        expect(isMatch({
          inputs: 'a*bcd',
          patterns: `a${escape}*b*`,
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'a*bxd',
          patterns: `a${escape}*b*`,
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'axbcd',
          patterns: `a${escape}*b*`,
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'test*end',
          patterns: `*${escape}*end`,
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'testend',
          patterns: `*${escape}*end`,
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'a*b',
          patterns: `*${escape}**`,
        },),).toBe(true,);
        expect(isMatch({
          inputs: '*',
          patterns: `*${escape}**`,
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'ab',
          patterns: `*${escape}**`,
        },),).toBe(false,);
      },
    },),

    it({
      name: 'escapes any character with a backslash',
      fn: async () => {
        /**
         Backslash code unit shared by every escaped pattern below.
         */
        const escape = String.fromCodePoint(92,);
        expect(isMatch({
          inputs: 'abc',
          patterns: `${escape}a${escape}b${escape}c`,
        },),).toBe(true,);
        expect(isMatch({
          inputs: '😀',
          patterns: `${escape}😀`,
        },),).toBe(true,);
        expect(isMatch({
          inputs: `${escape}a`,
          patterns: `${escape}${escape}a`,
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'a',
          patterns: `${escape}${escape}a`,
        },),).toBe(false,);
        expect(isMatch({
          inputs: `a${escape}bc`,
          patterns: `a${escape}${escape}*`,
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'a*',
          patterns: `a${escape}${escape}*`,
        },),).toBe(false,);
      },
    },),

    it({
      name: 'treats a trailing escape as literal',
      fn: async () => {
        /**
         Backslash code unit shared by every escaped pattern below.
         */
        const escape = String.fromCodePoint(92,);
        expect(isMatch({
          inputs: 'test',
          patterns: `test${escape}`,
        },),).toBe(false,);
        expect(isMatch({
          inputs: `test${escape}`,
          patterns: `test${escape}${escape}`,
        },),).toBe(true,);
      },
    },),

    //endregion Escapes

    //region Case folding

    it({
      name: 'folds non-ASCII letters without ASCII lookalikes',
      fn: async () => {
        expect(isMatch({
          inputs: 'ÉCOLE',
          patterns: 'école',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'ΟΔΟΣ',
          patterns: 'οδος',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'ς',
          patterns: 'σ',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'ДОМ',
          patterns: 'дом',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'ÉCOLE',
          patterns: 'école',
          options: { caseSensitive: true, },
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'ÆBLE',
          patterns: 'æble',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'ΣΊΣΥΦΟΣ',
          patterns: 'σίσυφος',
        },),).toBe(true,);
      },
    },),

    it({
      name: 'never folds lookalike characters into ASCII',
      fn: async () => {
        expect(isMatch({
          inputs: 'https://gıthub.com/login',
          patterns: 'https://github.com/*',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'https://claßic.com/x',
          patterns: 'https://classic.com/*',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'report.jſ',
          patterns: '*.js',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'ﬁle',
          patterns: 'file',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'ß',
          patterns: 'ẞ',
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'İ',
          patterns: 'i',
        },),).toBe(false,);
      },
    },),

    it({
      name: 'matches surrogate pairs and lone surrogates',
      fn: async () => {
        expect(isMatch({
          inputs: '😀',
          patterns: '*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'a😀b',
          patterns: 'a*b',
        },),).toBe(true,);
        expect(isMatch({
          inputs: '😀🦄',
          patterns: '😀*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: '\uD800',
          patterns: '*',
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'a\uDC00',
          patterns: 'A\uDC00',
        },),).toBe(true,);
        expect(isMatch({
          inputs: '😀',
          patterns: '\uD83D*',
        },),).toBe(true,);
      },
    },),

    //endregion Case folding

    //region Consistency

    it({
      name: 'agrees between isMatch and matcher for zero or one input',
      fn: async () => {
        /**
         Input and pattern pairs sharing one verdict.
         */
        const cases: readonly (readonly [string, string])[] = [
          [
            'foo',
            'f*',
          ],
          [
            'bar',
            'f*',
          ],
          [
            '',
            '*',
          ],
          [
            'a',
            '!',
          ],
        ];
        for (const [input, pattern] of cases)
          expect(isMatch({
            inputs: [input],
            patterns: [pattern],
          },),).toBe(matcher({
            inputs: [input],
            patterns: [pattern],
          },).length > 0,);
        expect(isMatch({
          inputs: [],
          patterns: ['*'],
        },),).toBe(matcher({
          inputs: [],
          patterns: ['*'],
        },).length > 0,);
      },
    },),

    it({
      name: 'never mutates its arguments',
      fn: async () => {
        /**
         Frozen inputs the matcher must leave untouched.
         */
        const inputs = Object.freeze([
          'foo',
          'bar',
        ],);
        /**
         Frozen patterns the matcher must leave untouched.
         */
        const patterns = Object.freeze([
          'f*',
          '!bar',
        ],);
        /**
         Frozen options the matcher must leave untouched.
         */
        const options = Object.freeze({
          caseSensitive: true,
          allPatterns: true,
        },);
        expect(matcher({
          inputs,
          patterns,
          options,
        },),).toEqual([
          'foo',
        ],);
        expect(isMatch({
          inputs,
          patterns,
          options,
        },),).toBe(true,);
        expect(inputs,).toEqual([
          'foo',
          'bar',
        ],);
        expect(patterns,).toEqual([
          'f*',
          '!bar',
        ],);
      },
    },),

    //endregion Consistency
  ],
},);
