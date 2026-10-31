import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CrateReleaseError } from './error.ts';
import {
  assertReleaseVersion,
  isReleaseVersion,
} from './version.ts';

/**
 Versions Cargo accepts: plain, multi-digit, prerelease and build forms.
 */
const acceptedVersions: readonly string[] = [
  '0.0.0',
  '0.4.2',
  '0.4.10',
  '10.20.30',
  '1.0.0-rc.1',
  '1.0.0-alpha',
  '1.0.0-0.3.7',
  '1.0.0-x-y-z.--',
  '1.0.0+build.5',
  '1.0.0-rc.1+build-5.sha',
];

/**
 Texts that are not versions, or that would change meaning when placed in a URL, a tag or a manifest line.
 */
const rejectedVersions: readonly string[] = [
  '',
  '1',
  '1.2',
  '1.2.3.4',
  '01.2.3',
  '1.02.3',
  '1.2.03',
  '1.2.3-',
  '1.2.3-01',
  '1.2.3-a..b',
  '1.2.3-.a',
  '1.2.3+',
  '1.2.3+a..b',
  '1.2.3-a+b+c',
  'v1.2.3',
  ' 1.2.3',
  '1.2.3 ',
  '1.2.3\n',
  '1.2.3\0',
  '-1.2.3',
  '1.2.-3',
  '../1.2.3',
  '1.2.3/..',
  '1.2.3?x=1',
  '1.2.3#frag',
  `1.2.3'`,
  '1.2.3"',
  '1.2.3;rm',
  '1.2.3$HOME',
  '1.2.é',
  '=1.2.3',
  '1.2.3 - 4.5.6',
];

await describe({
  name: '',
  children: [
    describe({
      name: isReleaseVersion.name,
      children: [
        ...acceptedVersions.map(function acceptCase(version) {
          return it({
            name: `accepts ${JSON.stringify(version)}`,
            fn: async () => {
              expect(isReleaseVersion(version)).toBe(true);
            },
          });
        }),
        ...rejectedVersions.map(function rejectCase(version) {
          return it({
            name: `rejects ${JSON.stringify(version)}`,
            fn: async () => {
              expect(isReleaseVersion(version)).toBe(false);
            },
          });
        }),
      ],
    }),
    describe({
      name: assertReleaseVersion.name,
      children: [
        it({
          name: 'returns an accepted version unchanged',
          fn: async () => {
            expect(assertReleaseVersion({ option: '<to>', value: '0.4.10' })).toBe('0.4.10');
          },
        }),
        it({
          name: 'names the option and the offending value when it rejects',
          fn: async () => {
            let caught: unknown;
            try {
              assertReleaseVersion({ option: '<to>', value: '../1.0.0' });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('<to>');
            expect((caught as Error).message).toContain('"../1.0.0"');
          },
        }),
      ],
    }),
  ],
});
