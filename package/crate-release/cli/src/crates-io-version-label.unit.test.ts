import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { maximumVersionLength } from './constants.ts';
import { labelCases } from './crates-io-version-label-cases.ts';
import {
  cratesIoRoute,
  goesToCratesIo,
  isAlphaVersion,
  releaseLabel,
} from './crates-io-version-label.ts';

/**
 Labels whose versions go to crates.io, by the owner's rule: a plain number and `-rc.N`.
 */
const cratesIoLabels: ReadonlySet<string> = new Set(['stable', 'rc']);

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
      children: labelCases.map(function goesCase({
        version,
        label,
      }) {
        return it({
          name: `${cratesIoLabels.has(label) ? 'sends' : 'never sends'} ${JSON.stringify(version)} to crates.io`,
          fn: async () => {
            expect(goesToCratesIo(version)).toBe(cratesIoLabels.has(label));
          },
        });
      }),
    }),
    describe({
      name: cratesIoRoute.name,
      children: labelCases.map(function reasonCase({
        version,
        label,
      }) {
        return it({
          name: `${cratesIoLabels.has(label) ? 'goes to crates.io for' : 'names its label in the reason it stays on GitHub for'} ${JSON.stringify(version)}`,
          fn: async () => {
            const route = cratesIoRoute(version);
            expect(route.goes).toBe(cratesIoLabels.has(label));
            if (!route.goes) {
              expect(route.reason).toContain(label);
              expect(route.reason).toContain('not published to crates.io');
            }
          },
        });
      }),
    }),
    describe({
      name: isAlphaVersion.name,
      children: [
        ...labelCases.map(function alphaCase({
          version,
          label,
        }) {
          return it({
            name: `${label === 'alpha' ? 'calls' : 'does not call'} ${JSON.stringify(version)} an alpha`,
            fn: async () => {
              expect(isAlphaVersion(version)).toBe(label === 'alpha');
            },
          });
        }),
        it({
          name: 'reads the label rule instead of parsing the text itself, so a form the rule does not name is no alpha',
          fn: async () => {
            // Each of these spells alpha in a way the label rule refuses, so none of them may buy an alpha's shortcuts:
            // an alpha skips work that a stable release does, and the rule is the only thing that decides who gets it.
            for (const version of ['1.0.0-alpha', '1.0.0-alpha.1.2', '1.0.0-Alpha.1', '1.0.0-alpha-1', '1.0.0-rc.1-alpha.2']) {
              expect(releaseLabel(version)).not.toBe('alpha');
              expect(isAlphaVersion(version)).toBe(false);
            }
          },
        }),
      ],
    }),
  ],
});
