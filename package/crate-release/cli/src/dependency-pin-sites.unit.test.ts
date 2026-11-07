import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  inlinePinSites,
  tablePinSites,
} from './dependency-pin-sites.ts';
import { CrateReleaseError } from './error.ts';

/**
 Manifest lines with an inline pin on one line and a pin table further down.
 */
const bodies = [
  '[dependencies]',
  'dep = { path = "../d", version = "=1.0.0" }',
  '',
  '# the table form',
  '[build-dependencies.dep]',
  'path = "../d"',
  'version = "=1.0.0" # moves with the crate',
  '',
  '[lib]',
  'version = "=9.9.9"',
];

await describe({
  name: '',
  children: [
    describe({
      name: inlinePinSites.name,
      children: [
        it({
          name: 'reports the line and the quoted requirement of each inline dependency on the crate, in line order',
          fn: async () => {
            const sites = inlinePinSites({ bodies, crateName: 'dep' });
            expect(sites.map(function indexOf(site) {
              return site.index;
            })).toEqual([1]);
            expect(sites.map(function valueOf(site) {
              return site.location.value;
            })).toEqual(['=1.0.0']);
          },
        }),
        it({
          name: 'reports nothing for a crate that only a table names, and refuses an inline table without a quoted version',
          fn: async () => {
            expect(inlinePinSites({ bodies: ['[dependencies.dep]', 'version = "=1.0.0"'], crateName: 'dep' })).toEqual([]);
            let caught: unknown;
            try {
              inlinePinSites({ bodies: ['dep = { path = "../d" }'], crateName: 'dep' });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(CrateReleaseError);
          },
        }),
      ],
    }),
    describe({
      name: tablePinSites.name,
      children: [
        it({
          name: 'reports the line of the requirement, not the header, and stops at the next header',
          fn: async () => {
            const sites = tablePinSites({ bodies, crateName: 'dep' });
            expect(sites.map(function indexOf(site) {
              return site.index;
            })).toEqual([6]);
            expect(sites.map(function valueOf(site) {
              return site.location.value;
            })).toEqual(['=1.0.0']);
          },
        }),
        it({
          name: 'reports nothing for a crate that only an inline table names',
          fn: async () => {
            expect(tablePinSites({ bodies: ['dep = { version = "=1.0.0" }'], crateName: 'dep' })).toEqual([]);
          },
        }),
        it({
          name: 'reports each table of the crate, one per kind',
          fn: async () => {
            const sites = tablePinSites({
              bodies: ['[dependencies.dep]', 'version = "=1.0.0"', '[dev-dependencies.dep]', 'version = "=1.0.0"'],
              crateName: 'dep',
            });
            expect(sites.map(function indexOf(site) {
              return site.index;
            })).toEqual([1, 3]);
          },
        }),
      ],
    }),
  ],
});
