/**
 Unit tests for the built Cargo semver helpers.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  compareVersions,
  formatVersion,
  parseVersion,
  satisfies,
  splitRequirement,
} from '../dist/final/node/index.mjs';

/**
 Verdicts of `satisfies` for one requirement across several versions.

 @param requirement - requirement under test

 @param versions - candidate versions

 @returns one verdict per version, in order

 @example
 ```ts
 matches({ requirement: '1', versions: ['1.0.0',], }); // [true]
 ```
 */
function matches(
  {
    requirement,
    versions,
  }: {
    readonly requirement: string;
    readonly versions: readonly string[];
  },
): readonly boolean[] {
  return versions.map(function verdict(version,): boolean {
    return satisfies({
      version,
      requirement,
    },);
  },);
}

await describe({
  name: 'semver',
  children: [
    it({
      name: parseVersion.name,
      fn: async () => {
        expect(parseVersion('1.0.0-rc.1+b7',),).toEqual({ major: 1, minor: 0, patch: 0, prerelease: ['rc', '1',], },);
        expect(parseVersion('0.4',),).toEqual({ major: 0, minor: 4, patch: -1, prerelease: [], },);
        expect(parseVersion('2',),).toEqual({ major: 2, minor: -1, patch: -1, prerelease: [], },);
        try {
          parseVersion('1.x',);
        } catch (error) {
          expect(error,).toBeInstanceOf(TypeError,);
          return;
        }
        throw new Error('expected a TypeError',);
      },
    },),

    it({
      name: compareVersions.name,
      fn: async () => {
        /**
         Versions in ascending semver precedence (the semver spec's own prerelease example, then releases).
         */
        const ascending = [
          '1.0.0-alpha',
          '1.0.0-alpha.1',
          '1.0.0-alpha.beta',
          '1.0.0-beta',
          '1.0.0-beta.2',
          '1.0.0-beta.11',
          '1.0.0-rc.1',
          '1.0.0',
          '1.0.99',
          '1.0.104',
          '1.10.0',
          '2.0.0',
        ];
        /**
         Same versions sorted by the helper from a reversed start.
         */
        const sorted = ascending.toReversed().toSorted(function byPrecedence(left, right,): number {
          return compareVersions({ left: parseVersion(left,), right: parseVersion(right,), },);
        },);
        expect(sorted,).toEqual(ascending,);
        expect(compareVersions({ left: parseVersion('1.0',), right: parseVersion('1.0.0',), },),).toBe(0,);
      },
    },),

    it({
      name: splitRequirement.name,
      fn: async () => {
        expect(splitRequirement('>=1.0.104',).operator,).toBe('>=',);
        expect(splitRequirement('=1.18.1',).operator,).toBe('=',);
        expect(splitRequirement('~0.58.1',).operator,).toBe('~',);
        expect(splitRequirement('^0.4',).operator,).toBe('^',);
        expect(splitRequirement('0.4',).operator,).toBe('^',);
      },
    },),

    it({
      name: 'caret requirements follow Cargo rules for major, zero-major, and zero-minor versions',
      fn: async () => {
        expect(matches({ requirement: '1', versions: ['1.0.0', '1.99.0', '2.0.0', '0.9.0',], },),).toEqual([true, true, false, false,],);
        expect(matches({ requirement: '1.2.3', versions: ['1.2.3', '1.9.0', '1.2.2', '2.0.0',], },),).toEqual([true, true, false, false,],);
        expect(matches({ requirement: '0.58', versions: ['0.58.0', '0.58.9', '0.62.0', '0.57.0',], },),).toEqual([true, true, false, false,],);
        expect(matches({ requirement: '^0.0.3', versions: ['0.0.3', '0.0.4',], },),).toEqual([true, false,],);
        expect(matches({ requirement: '^0.0', versions: ['0.0.9', '0.1.0',], },),).toEqual([true, false,],);
        expect(matches({ requirement: '^0', versions: ['0.9.0', '1.0.0',], },),).toEqual([true, false,],);
      },
    },),

    it({
      name: 'tilde, exact, and floor requirements',
      fn: async () => {
        expect(matches({ requirement: '~1.2.3', versions: ['1.2.9', '1.3.0',], },),).toEqual([true, false,],);
        expect(matches({ requirement: '~1', versions: ['1.9.0', '2.0.0',], },),).toEqual([true, false,],);
        expect(matches({ requirement: '=1.18.1', versions: ['1.18.1', '1.18.2',], },),).toEqual([true, false,],);
        expect(matches({ requirement: '=1.18', versions: ['1.18.7', '1.19.0',], },),).toEqual([true, false,],);
        expect(matches({ requirement: '=1', versions: ['1.5.0', '2.0.0',], },),).toEqual([true, false,],);
        expect(matches({ requirement: '>=1.0.104', versions: ['1.0.104', '2.0.0', '1.0.103',], },),).toEqual([true, true, false,],);
      },
    },),

    it({
      name: 'prereleases match only comparators naming a prerelease of the same release',
      fn: async () => {
        expect(matches({ requirement: '>=1.0.0', versions: ['1.1.0-alpha',], },),).toEqual([false,],);
        expect(matches({ requirement: '>=1.0.0-alpha.5', versions: ['1.0.0-alpha.6', '1.0.1-alpha.1',], },),).toEqual([true, false,],);
      },
    },),

    it({
      name: formatVersion.name,
      fn: async () => {
        expect(
          formatVersion(parseVersion('0.4',),),
        ).toBe('0.4.0',);
        expect(
          formatVersion(parseVersion('1.0.0-rc.1+b',),),
        ).toBe('1.0.0-rc.1',);
      },
    },),
  ],
},);
