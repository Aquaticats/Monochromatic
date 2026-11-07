import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { maximumVersionLength } from './constants.ts';
import { labelCases } from './crates-io-version-label-cases.ts';
import {
  cratesIoSkipReason,
  goesToCratesIo,
  releaseLabel,
} from './crates-io-version-label.ts';

/**
 Labels whose versions go to crates.io, by the owner's rule: a plain number and `-rc.N`.
 */
const cratesIoLabels: readonly string[] = ['stable', 'rc'];

await describe({
  name: '',
  children: [
    describe({
      name: releaseLabel.name,
      children: [
        ...labelCases.map(function labelCase({
          version,
          label,
        }) {
          return it({
            name: `gives ${JSON.stringify(version)} the label ${label}`,
            fn: async () => {
              expect(releaseLabel(version)).toBe(label);
            },
          });
        }),
        it({
          name: 'gives a version longer than the release tool accepts no crates.io label',
          fn: async () => {
            expect(releaseLabel(`1.0.0-rc.${'9'.repeat(maximumVersionLength)}`)).toBe('unlabelled');
          },
        }),
      ],
    }),
    describe({
      name: goesToCratesIo.name,
      children: [
        ...labelCases.map(function goesCase({
          version,
          label,
        }) {
          return it({
            name: `${cratesIoLabels.includes(label) ? 'sends' : 'never sends'} ${JSON.stringify(version)} to crates.io`,
            fn: async () => {
              expect(goesToCratesIo(version)).toBe(cratesIoLabels.includes(label));
            },
          });
        }),
      ],
    }),
    describe({
      name: cratesIoSkipReason.name,
      children: [
        ...labelCases.map(function reasonCase({
          version,
          label,
        }) {
          return it({
            name: `${cratesIoLabels.includes(label) ? 'has no skip reason for' : 'names its label in the skip reason of'} ${JSON.stringify(version)}`,
            fn: async () => {
              const reason = cratesIoSkipReason(version);
              if (cratesIoLabels.includes(label))
                expect(reason).toBe(undefined);
              else {
                expect(reason).toContain(label);
                expect(reason).toContain('not published to crates.io');
              }
            },
          });
        }),
      ],
    }),
  ],
});
