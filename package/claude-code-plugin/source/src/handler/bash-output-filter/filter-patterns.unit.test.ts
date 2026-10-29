import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  MISE_NOISE_PREDICATES,
  SANDBOX_NOISE_PREDICATES,
} from './filter-patterns.ts';

/**
 Classifies a line as sandbox noise via the exported predicate array.
 `isSandboxMiseCacheNoise` is the sole entry; it is not exported directly,
 so the test exercises it through the array that production code consults.
 
 @param line - candidate output line
 
 @returns whether any sandbox-noise predicate matches
 */
function isNoise(line: string,): boolean {
  return SANDBOX_NOISE_PREDICATES.some(function test(predicate,) {
    return predicate(line,);
  },);
}

/**
 Classifies a line as mise bootstrap/upgrade noise via the exported predicate
 array. Its member predicates are not exported directly, so the test drives
 them through the array that {@link shouldStripLine} consults.
 
 @param line - candidate output line
 
 @returns whether any mise-noise predicate matches
 */
function isMiseNoise(line: string,): boolean {
  return MISE_NOISE_PREDICATES.some(function test(predicate,) {
    return predicate(line,);
  },);
}

/** Canonical mise read-only cache warning the predicate is built to match. */
const FULL = 'mise WARN failed to write cache file: /x/y Read-only file system';

/** Real minimum-release-age warnings mise emits for gated tool releases. */
const MIN_RELEASE_AGE_WARNS: readonly string[] = [
  'mise WARN  newer pnpm release 11.12.0 ignored by minimum_release_age (24h); latest eligible release is 10.34.5',
  'mise WARN  newer tree-sitter release 0.26.11 ignored by minimum_release_age (24h); latest eligible release is 0.26.10',
];

await describe({
  name: 'filter-patterns sandbox noise',
  children: [
    describe({
      name: 'isSandboxMiseCacheNoise (via SANDBOX_NOISE_PREDICATES)',
      children: [
        it({
          name: 'matches the canonical warning',
          fn: async () => {
            expect(isNoise(FULL,),).toBe(true,);
          },
        },),
        it({
          name: 'matches with leading whitespace',
          fn: async () => {
            expect(isNoise(`   ${FULL}`,),).toBe(true,);
          },
        },),
        it({
          name: 'matches with an optional bracketed prefix',
          fn: async () => {
            expect(isNoise(`[INFO] ${FULL}`,),).toBe(true,);
          },
        },),
        it({
          name: 'rejects the empty string',
          fn: async () => {
            expect(isNoise('',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects all-whitespace input',
          fn: async () => {
            expect(isNoise('        ',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects a warning without the failed-to-write segment',
          fn: async () => {
            expect(isNoise('mise WARN something unrelated happened',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects when no whitespace follows the WARN token',
          fn: async () => {
            expect(isNoise('mise WARNfailed to write cache file: x Read-only file system',),)
              .toBe(false,);
          },
        },),
        it({
          name: 'rejects when the read-only-fs marker is absent',
          fn: async () => {
            expect(isNoise('mise WARN failed to write cache file: /x permission denied',),)
              .toBe(false,);
          },
        },),
        it({
          name: 'rejects unrelated prose',
          fn: async () => {
            expect(isNoise('hello world',),).toBe(false,);
          },
        },),
        it({
          name: 'matches with a long leading-whitespace run (linear scan)',
          fn: async () => {
            expect(isNoise(`${' '.repeat(200_000,)}${FULL}`,),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);

await describe({
  name: 'filter-patterns mise bootstrap and upgrade noise',
  children: [
    describe({
      name: 'MISE_NOISE_PREDICATES',
      children: [
        it({
          name: 'matches the bootstrap install echo',
          fn: async () => {
            expect(isMiseNoise('[//:bootstrap] $ mise install',),).toBe(true,);
          },
        },),
        it({
          name: 'matches the bootstrap upgrade echo',
          fn: async () => {
            expect(isMiseNoise('[//:bootstrap] $ mise upgrade',),).toBe(true,);
          },
        },),
        it({
          name: 'matches the install no-op summary',
          fn: async () => {
            expect(isMiseNoise('mise all tools are installed',),).toBe(true,);
          },
        },),
        it({
          name: 'matches the upgrade no-op summary',
          fn: async () => {
            expect(isMiseNoise('mise All tools are up to date',),).toBe(true,);
          },
        },),
        it({
          name: 'matches every real minimum-release-age warning',
          fn: async () => {
            for (const warn of MIN_RELEASE_AGE_WARNS) {
              expect(isMiseNoise(warn,),).toBe(true,);
            }
          },
        },),
        it({
          name: 'rejects a real running-install line',
          fn: async () => {
            expect(isMiseNoise('mise installing node@22.0.0',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects a different task echo',
          fn: async () => {
            expect(isMiseNoise('[//:build] $ tsc',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects an unrelated mise warning',
          fn: async () => {
            expect(isMiseNoise('mise WARN  something else entirely happened',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects the empty string',
          fn: async () => {
            expect(isMiseNoise('',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects unrelated prose',
          fn: async () => {
            expect(isMiseNoise('hello world',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
