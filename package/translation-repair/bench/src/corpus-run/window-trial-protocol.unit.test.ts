/**
 Tests for the window trial's two protocol rules.

 The digest decides which ledger rows a resumed run may pool with its own;
 the streak rule decides when a run of refusals is the run's fault. The
 second held a defect: a slice the ledger already held reset the streak, so a
 resumed run could refuse every new slice without ever reaching the stop.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  protocolDigest,
  streakAfter,
} from '../../dist/final/node/index.mjs';

/**
 Hex characters in a SHA-256 digest.
 */
const SHA256_HEX_LENGTH = 64;

/**
 Clone and commit a trial reads at, invented.
 */
const TRIAL_PIN = {
  cloneDir: '/cats/corpus/clone',
  commitSha: 'feedface'.repeat(5,),
} as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: streakAfter.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES to reset the streak on a slice the ledger already held, which bought nothing and so says '
            + 'nothing about whether the run can still buy',
          fn: async () => {
            expect(streakAfter({
              refusedInARow: 2,
              yielded: 'already-held',
            },),).toBe(2,);
          },
        },),

        it({
          name: 'climbs by one on a refusal',
          fn: async () => {
            expect(streakAfter({
              refusedInARow: 2,
              yielded: 'refused',
            },),).toBe(3,);
          },
        },),

        it({
          name: 'resets on a slice that bought arms',
          fn: async () => {
            expect(streakAfter({
              refusedInARow: 4,
              yielded: 'bought',
            },),).toBe(0,);
          },
        },),

        it({
          name: 'reaches the stop across a resumed run whose held slices interleave with refusals of every new one',
          fn: async () => {
            /**
             What a resumed run sees: every slice the ledger held, then every new
             slice refused, alternating.
             */
            const walk = [
              'already-held',
              'refused',
              'already-held',
              'refused',
              'already-held',
              'refused',
            ] as const;

            expect(walk.reduce(function step(streak, yielded,): number {
              return streakAfter({
                refusedInARow: streak,
                yielded,
              },);
            }, 0,),).toBe(3,);
          },
        },),
      ],
    },),

    describe({
      name: protocolDigest.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'changes with the commit, so rows bought under other code are never pooled',
          fn: async () => {
            expect(protocolDigest({
              headSha: 'a'.repeat(SHA256_HEX_LENGTH,),
              pin: TRIAL_PIN,
            },),).not.toBe(protocolDigest({
              headSha: 'b'.repeat(SHA256_HEX_LENGTH,),
              pin: TRIAL_PIN,
            },),);
          },
        },),

        it({
          name: 'CHANGES WITH THE CORPUS COMMIT IT IS HANDED, so rows bought at another pin are never pooled',
          fn: async () => {
            expect(protocolDigest({
              headSha: 'a'.repeat(SHA256_HEX_LENGTH,),
              pin: TRIAL_PIN,
            },),).not.toBe(protocolDigest({
              headSha: 'a'.repeat(SHA256_HEX_LENGTH,),
              pin: {
                ...TRIAL_PIN,
                commitSha: 'deadbeef'.repeat(5,),
              },
            },),);
          },
        },),

        it({
          name: 'is the same digest twice for the same commit, which is what resumption keys on',
          fn: async () => {
            expect(protocolDigest({
              headSha: 'a'.repeat(SHA256_HEX_LENGTH,),
              pin: TRIAL_PIN,
            },),).toBe(protocolDigest({
              headSha: 'a'.repeat(SHA256_HEX_LENGTH,),
              pin: TRIAL_PIN,
            },),);
          },
        },),

        it({
          name: 'is a whole SHA-256 in hex, the shape the ledger stores',
          fn: async () => {
            expect(protocolDigest({
              headSha: 'a'.repeat(SHA256_HEX_LENGTH,),
              pin: TRIAL_PIN,
            },).length,).toBe(SHA256_HEX_LENGTH,);
          },
        },),
      ],
    },),
  ],
},);
