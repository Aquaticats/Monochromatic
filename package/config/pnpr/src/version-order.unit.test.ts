import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  chooseDistTag,
  compareVersions,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: compareVersions.name,
      children: [
        //region Core components

        it({
          name: 'orders by major, then minor, then patch',
          fn: async () => {
            expect(compareVersions({ left: '1.0.0', right: '2.0.0', },),).toBeLessThan(0,);
            expect(compareVersions({ left: '1.2.0', right: '1.1.9', },),).toBeGreaterThan(0,);
            expect(compareVersions({ left: '1.1.2', right: '1.1.10', },),).toBeLessThan(0,);
          },
        },),
        it({
          name: 'treats equal versions and differing build metadata as equal precedence',
          fn: async () => {
            expect(compareVersions({ left: '0.4.0', right: '0.4.0', },),).toBe(0,);
            expect(compareVersions({ left: '0.4.0+a', right: '0.4.0+b', },),).toBe(0,);
          },
        },),

        //endregion Core components

        //region Prerelease identifiers

        it({
          name: 'ranks a release above its prereleases',
          fn: async () => {
            expect(compareVersions({ left: '1.0.0-alpha', right: '1.0.0', },),).toBeLessThan(0,);
            expect(compareVersions({ left: '1.0.0', right: '1.0.0-alpha', },),).toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'compares numeric identifiers numerically',
          fn: async () => {
            expect(compareVersions({ left: '1.0.0-alpha.2', right: '1.0.0-alpha.10', },),).toBeLessThan(0,);
          },
        },),
        it({
          name: 'ranks numeric identifiers below alphanumeric ones',
          fn: async () => {
            expect(compareVersions({ left: '1.0.0-1', right: '1.0.0-alpha', },),).toBeLessThan(0,);
            expect(compareVersions({ left: '1.0.0-alpha', right: '1.0.0-1', },),).toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'compares alphanumeric identifiers in ASCII order',
          fn: async () => {
            expect(compareVersions({ left: '1.0.0-alpha', right: '1.0.0-beta', },),).toBeLessThan(0,);
            expect(compareVersions({ left: '1.0.0-beta', right: '1.0.0-alpha', },),).toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'ranks a longer prerelease above its prefix',
          fn: async () => {
            expect(compareVersions({ left: '1.0.0-alpha', right: '1.0.0-alpha.1', },),).toBeLessThan(0,);
          },
        },),
        it({
          name: 'treats identical prereleases as equal',
          fn: async () => {
            expect(compareVersions({ left: '1.0.0-rc.1', right: '1.0.0-rc.1', },),).toBe(0,);
          },
        },),
        it({
          name: 'treats an empty prerelease identifier as alphanumeric',
          fn: async () => {
            expect(compareVersions({ left: '1.0.0-1', right: '1.0.0-', },),).toBeLessThan(0,);
          },
        },),

        //endregion Prerelease identifiers

        //region Invalid versions

        ...[
          '1.2',
          '1.2.3.4',
          '1..3',
          '1.2.x',
          'v1.2.3',
          '1.-2.3',
        ].map(function invalidVersionCase(version,) {
          return it({
            name: `throws for ${version}`,
            fn: async () => {
              expect(function compareInvalid() {
                return compareVersions({ left: version, right: '1.0.0', },);
              },).toThrow('not a semantic version',);
            },
          },);
        },),

        //endregion Invalid versions
      ],
    },),
    describe({
      name: chooseDistTag.name,
      children: [
        it({
          name: 'uses latest for a package with no current latest',
          fn: async () => {
            expect(chooseDistTag({ version: '0.0.1', },),).toBe('latest',);
          },
        },),
        it({
          name: 'uses latest for a newer version',
          fn: async () => {
            expect(chooseDistTag({ version: '0.2.0', currentLatest: '0.1.0', },),).toBe('latest',);
          },
        },),
        it({
          name: 'uses backfill for an older or equal version',
          fn: async () => {
            expect(chooseDistTag({ version: '0.1.0', currentLatest: '0.2.0', },),).toBe('backfill',);
            expect(chooseDistTag({ version: '0.2.0', currentLatest: '0.2.0', },),).toBe('backfill',);
          },
        },),
      ],
    },),
  ],
},);
