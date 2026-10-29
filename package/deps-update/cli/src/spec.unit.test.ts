/**
 Unit tests for `name@version` spec parsing and exclude-entry expansion.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  expandExcludeEntry,
  PackageSpecError,
  parsePackageSpec,
} from './spec.ts';

/**
 Captures a synchronous throw for assertion.

 @param fn - code expected to throw

 @returns thrown value, or a marker string when nothing was thrown

 @example
 ```ts
 expect(caught(() => parsePackageSpec('a'))).toBeInstanceOf(PackageSpecError);
 ```
 */
function caught(fn: () => unknown,): unknown {
  try {
    fn();
    return 'did not throw';
  }
  catch (error) {
    return error;
  }
}

await describe({
  name: '',
  children: [
    describe({
      name: parsePackageSpec.name,
      children: [
        it({
          name: 'splits a scoped spec at its last @',
          fn: async () => {
            expect(parsePackageSpec('@earendil-works/chord@0.87.1',),).toEqual({
              name: '@earendil-works/chord',
              version: '0.87.1',
            },);
          },
        },),
        it({
          name: 'splits an unscoped spec',
          fn: async () => {
            expect(parsePackageSpec('left-pad@1.3.0',),).toEqual({
              name: 'left-pad',
              version: '1.3.0',
            },);
          },
        },),
        ...['left-pad', '@scope/name', 'left-pad@', '@1.0.0',].map(function rejects(spec,) {
          return it({
            name: `rejects ${JSON.stringify(spec,)}`,
            fn: async () => {
              expect(caught(function parse(): unknown {
                return parsePackageSpec(spec,);
              },),).toBeInstanceOf(PackageSpecError,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: expandExcludeEntry.name,
      children: [
        it({
          name: 'expands a merged version union',
          fn: async () => {
            expect(expandExcludeEntry('a@1.0.0 || 1.1.0',),).toEqual([
              {
                name: 'a',
                version: '1.0.0',
              },
              {
                name: 'a',
                version: '1.1.0',
              },
            ],);
          },
        },),
        ...['left-pad', '@scope/*', '@earendil-works/pi-*',].map(function bare(entry,) {
          return it({
            name: `expands bare pattern ${entry} to nothing`,
            fn: async () => {
              expect(expandExcludeEntry(entry,),).toEqual([],);
            },
          },);
        },),
        it({
          name: 'rejects an empty version inside a union',
          fn: async () => {
            expect(caught(function expand(): unknown {
              return expandExcludeEntry('a@1.0.0 ||  ',);
            },),).toBeInstanceOf(PackageSpecError,);
          },
        },),
      ],
    },),
  ],
},);
