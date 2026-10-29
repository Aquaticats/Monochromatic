/**
 Unit tests for workspace manifest reads and exclude-list diffing.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  addedVersions,
  NO_MINIMUM_RELEASE_AGE,
  readExcludeList,
  readMinimumReleaseAge,
  WorkspaceManifestShapeError,
} from './exclude-list.ts';

/**
 Captures a synchronous throw for assertion.

 @param fn - code expected to throw

 @returns thrown value, or a marker string when nothing was thrown

 @example
 ```ts
 expect(caught(() => readExcludeList('- a'))).toBeInstanceOf(WorkspaceManifestShapeError);
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
      name: readExcludeList.name,
      children: [
        it({
          name: 'reads entries in file order',
          fn: async () => {
            expect(readExcludeList("minimumReleaseAgeExclude:\n  - '@a/*'\n  - 'b@1.0.0'\n",),).toEqual([
              '@a/*',
              'b@1.0.0',
            ],);
          },
        },),
        it({
          name: 'returns empty when the key is absent',
          fn: async () => {
            expect(readExcludeList('minimumReleaseAge: 1440\n',),).toEqual([],);
          },
        },),
        it({
          name: 'returns empty for an empty document',
          fn: async () => {
            expect(readExcludeList('',),).toEqual([],);
          },
        },),
        it({
          name: 'rejects a non-list value',
          fn: async () => {
            expect(caught(function read(): unknown {
              return readExcludeList('minimumReleaseAgeExclude: a\n',);
            },),).toBeInstanceOf(WorkspaceManifestShapeError,);
          },
        },),
        it({
          name: 'rejects a non-string entry',
          fn: async () => {
            expect(caught(function read(): unknown {
              return readExcludeList('minimumReleaseAgeExclude:\n  - 1\n',);
            },),).toBeInstanceOf(WorkspaceManifestShapeError,);
          },
        },),
        it({
          name: 'rejects a non-mapping document',
          fn: async () => {
            expect(caught(function read(): unknown {
              return readExcludeList('- a\n',);
            },),).toBeInstanceOf(WorkspaceManifestShapeError,);
          },
        },),
      ],
    },),
    describe({
      name: readMinimumReleaseAge.name,
      children: [
        it({
          name: 'reads minutes',
          fn: async () => {
            expect(readMinimumReleaseAge('minimumReleaseAge: 1440\n',),).toBe(1_440,);
          },
        },),
        it({
          name: 'returns sentinel when unset',
          fn: async () => {
            expect(readMinimumReleaseAge('packages: []\n',),).toBe(NO_MINIMUM_RELEASE_AGE,);
          },
        },),
        ...['-1', "'1440'", '.nan',].map(function rejects(value,) {
          return it({
            name: `rejects minimumReleaseAge ${value}`,
            fn: async () => {
              expect(caught(function read(): unknown {
                return readMinimumReleaseAge(`minimumReleaseAge: ${value}\n`,);
              },),).toBeInstanceOf(WorkspaceManifestShapeError,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: addedVersions.name,
      children: [
        it({
          name: 'finds a newly appended entry',
          fn: async () => {
            expect(addedVersions({
              before: ['@earendil-works/pi-*',],
              after: ['@earendil-works/pi-*', '@earendil-works/chord@0.87.1',],
            },),).toEqual([{
              name: '@earendil-works/chord',
              version: '0.87.1',
            },],);
          },
        },),
        it({
          name: 'finds a version merged into an existing entry',
          fn: async () => {
            expect(addedVersions({
              before: ['cidr-tools@12.1.3',],
              after: ['cidr-tools@12.1.3 || 13.0.2',],
            },),).toEqual([{
              name: 'cidr-tools',
              version: '13.0.2',
            },],);
          },
        },),
        it({
          name: 'finds nothing when lists match',
          fn: async () => {
            expect(addedVersions({
              before: ['a@1.0.0',],
              after: ['a@1.0.0',],
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
