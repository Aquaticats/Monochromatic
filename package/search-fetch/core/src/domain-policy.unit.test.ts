/**
 Unit tests for Pi Search Fetch domain policy helpers.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  findBlockedHostMatch,
  isBlockedHost,
  normalizeBlocklist,
  normalizeBlocklistEntry,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Canonical blocked host fixture.
 */
const BLOCKED_HOST = 'badwikipedia.invalid';

/**
 Uppercase host fixture with root dot.
 */
const UPPERCASE_ROOT_DOT_HOST = ' BadWikipedia.INVALID. ';

/**
 Invalid blocklist entry fixtures.
 */
const INVALID_BLOCKLIST_ENTRIES = [
  'https://badwikipedia.invalid',
  'badwikipedia.invalid:443',
  'badwikipedia.invalid/path',
  '*.badwikipedia.invalid',
  'badwikipedia..invalid',
  '',
] as const;

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: normalizeBlocklistEntry.name,
      children: [
        it({
          name: 'lowercases uppercase input and strips one trailing dot',
          fn: async () => {
            expect(normalizeBlocklistEntry(UPPERCASE_ROOT_DOT_HOST,),).toBe(BLOCKED_HOST,);
          },
        },),
        ...INVALID_BLOCKLIST_ENTRIES.map(function invalidEntryTest(entry,) {
          return it({
            name: `rejects invalid entry ${JSON.stringify(entry,)}`,
            fn: async () => {
              let caught: unknown;
              try {
                normalizeBlocklistEntry(entry,);
              }
              catch (error: unknown) {
                caught = error;
              }
              expect(caught,).toBeInstanceOf(Error,);
              expect((caught as Error).message,).toContain('blocklist entry',);
            },
          },);
        },),
      ],
    },),
    describe({
      name: normalizeBlocklist.name,
      children: [
        it({
          name: 'deduplicates normalized entries',
          fn: async () => {
            expect(normalizeBlocklist([
              BLOCKED_HOST,
              UPPERCASE_ROOT_DOT_HOST,
            ],),).toEqual([BLOCKED_HOST,],);
          },
        },),
      ],
    },),
    describe({
      name: isBlockedHost.name,
      children: [
        it({
          name: 'matches exact host',
          fn: async () => {
            expect(isBlockedHost({
              host: BLOCKED_HOST,
              blocklist: [BLOCKED_HOST,],
            },),).toBe(true,);
          },
        },),
        it({
          name: 'matches subdomain suffix at a label boundary',
          fn: async () => {
            expect(isBlockedHost({
              host: `www.${BLOCKED_HOST}`,
              blocklist: [BLOCKED_HOST,],
            },),).toBe(true,);
          },
        },),
        it({
          name: 'does not match non-boundary suffix',
          fn: async () => {
            expect(isBlockedHost({
              host: `not${BLOCKED_HOST}`,
              blocklist: [BLOCKED_HOST,],
            },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: findBlockedHostMatch.name,
      children: [
        it({
          name: 'returns matching blocklist entry',
          fn: async () => {
            expect(findBlockedHostMatch({
              host: `www.${BLOCKED_HOST}`,
              blocklist: [BLOCKED_HOST,],
            },),).toEqual({
              blocked: true,
              entry: BLOCKED_HOST,
            },);
          },
        },),
      ],
    },),
  ],
},);
