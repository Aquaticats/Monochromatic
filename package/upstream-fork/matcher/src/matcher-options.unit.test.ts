/**
 Tests for matcher option resolution, exercised through the public `matcher`
 and `isMatch` entry points.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type MatcherOptions,
  isMatch,
  matcher,
} from '../dist/final/neutral/index.mjs';

/**
 Builds an options object inheriting polluted flags through its prototype,
 mirroring upstream matcher's own prototype-pollution regression test.
 Inherited flags must never enable matching behavior; only own properties
 count.
 
 @param own - Own options properties.
 
 @returns Options object with a polluted prototype and the given own properties.
 
 @example
 ```ts
 pollutedOptions({}); // options inheriting caseSensitive and allPatterns
 ```
 */
function pollutedOptions(own: Record<string, unknown>,): MatcherOptions {
  /**
   Polluted prototype carrying enabled flags the matcher must ignore.
   */
  const prototype = {
    caseSensitive: true,
    allPatterns: true,
  };
  /**
   Options object inheriting polluted flags through its prototype.
   */
  const polluted: MatcherOptions = Object.assign(
    Object.create(prototype,),
    own,
  ) as MatcherOptions;
  return polluted;
}

await describe({
  name: 'matcher option resolution',
  children: [
    //region Defaults

    it({
      name: 'matches case-insensitively by default',
      fn: async () => {
        expect(isMatch({
          inputs: 'UNICORN',
          patterns: 'unicorn',
        },),).toBe(true,);
      },
    },),

    it({
      name: 'accepts null options as defaults',
      fn: async () => {
        expect(isMatch({
          inputs: 'FOO',
          patterns: 'foo',
          options: null as never,
        },),).toBe(true,);
        expect(matcher({
          inputs: ['FOO'],
          patterns: 'foo',
          options: null as never,
        },),).toEqual([
          'FOO',
        ],);
      },
    },),

    it({
      name: 'keeps OR logic under null options',
      fn: async () => {
        expect(isMatch({
          inputs: 'foo',
          patterns: [
            'f*',
            'b*',
          ],
          options: null as never,
        },),).toBe(true,);
      },
    },),

    it({
      name: 'treats truthy option values as enabled',
      fn: async () => {
        expect(isMatch({
          inputs: 'FOO',
          patterns: 'foo',
          options: { caseSensitive: 1, } as never,
        },),).toBe(false,);
        expect(isMatch({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: [
            'f*',
            'b*',
          ],
          options: { allPatterns: 'yes', } as never,
        },),).toBe(false,);
      },
    },),

    //endregion Defaults

    //region caseSensitive

    it({
      name: 'matches case-sensitively when enabled',
      fn: async () => {
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
      },
    },),

    it({
      name: 'ignores prototype-polluted option flags',
      fn: async () => {
        expect(isMatch({
          inputs: 'SECRET.txt',
          patterns: 'secret*',
          options: pollutedOptions({}),
        },),).toBe(true,);
        expect(matcher({
          inputs: ['SECRET'],
          patterns: [
            '*',
            '!secret',
          ],
          options: pollutedOptions({}),
        },),).toEqual([],);
        expect(isMatch({
          inputs: 'foo',
          patterns: [
            'f*',
            'b*',
          ],
          options: pollutedOptions({}),
        },),).toBe(true,);
      },
    },),

    it({
      name: 'honors own flags over a polluted prototype',
      fn: async () => {
        expect(isMatch({
          inputs: 'SECRET.txt',
          patterns: 'secret*',
          options: pollutedOptions({ caseSensitive: true, }),
        },),).toBe(false,);
        expect(isMatch({
          inputs: 'foo',
          patterns: [
            'f*',
            'b*',
          ],
          options: pollutedOptions({ allPatterns: true, }),
        },),).toBe(false,);
      },
    },),

    //endregion caseSensitive

    //region allPatterns

    it({
      name: 'requires every normal pattern under allPatterns',
      fn: async () => {
        expect(isMatch({
          inputs: 'foobar',
          patterns: [
            'foo*',
            '*bar',
          ],
          options: { allPatterns: true, },
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'foo',
          patterns: [
            'foo*',
            '*bar',
          ],
          options: { allPatterns: true, },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'matches only negations under allPatterns when nothing is excluded',
      fn: async () => {
        expect(isMatch({
          inputs: 'foo',
          patterns: [
            '!bar',
            '!baz',
          ],
          options: { allPatterns: true, },
        },),).toBe(true,);
        expect(isMatch({
          inputs: 'bar',
          patterns: [
            '!bar',
            '!baz',
          ],
          options: { allPatterns: true, },
        },),).toBe(false,);
      },
    },),

    //endregion allPatterns
  ],
},);
