/**
 Equivalence tests for the two flat string scanners in
 `probe-field-parsers.ts`: `scanRepoEnd` (repo-span walk inside
 `parseGithubUrl`, reached through `parseRepository`) and `scanDigits`
 (digit-run walk inside `looksLikePinnedSemver`, reached through
 `resolveVersion`).
 
 Both were recursive cursor scans (`return scan(idx + 1)`) that grow stack
 depth with input length and overflow under V8, which lacks tail-call
 elimination. These cases capture the pre-rewrite behavior so the linear
 single-pass replacements stay behavior-identical: each delimiter, end of
 input, `.git` stripping, an empty span from a trailing separator, partial
 and prefix-only semver shapes, and a long repeated run that exercises the
 scan at a depth that would overflow a recursive walker on V8.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseRepository,
  REPO_UNPARSEABLE,
  resolveVersion,
  type VERSION_UNRESOLVED,
} from './probe-field-parsers.ts';
import type {
  NpmPackage,
  NpmVersion,
  RepositoryInfo,
} from './probe-field-types.ts';

/** Repeated-run length for the stack-safety cases; far past V8's recursion ceiling, so a recursive scan would overflow while the linear pass does not. */
const LONG_RUN = 100_000;

/**
 Fallback `dist-tags.latest` returned by {@link pinnedOrLatest} when the range is not a pinned semver; distinct from any pinned range so the branch is observable.
 */
const LATEST_SENTINEL = 'LATEST-FALLBACK';

/**
 Resolves `range` against a registry stub whose only concrete version key
 is `range` itself. `resolveVersion` returns `range` when
 `looksLikePinnedSemver(range)` holds (the `scanDigits`-driven branch) and
 otherwise falls back to `dist-tags.latest`, so the return value reveals the
 scanner's pinned/not-pinned verdict without exporting the private helper.
 
 @param range - Candidate range fed to the pinned-semver scan.
 
 @returns `range` when scanned as a pinned `major.minor.patch`, else {@link LATEST_SENTINEL}.
 
 @example
 ```ts
 pinnedOrLatest('1.2.3'); // '1.2.3'
 pinnedOrLatest('^1.0.0'); // 'LATEST-FALLBACK'
 ```
 */
function pinnedOrLatest(range: string,): string | typeof VERSION_UNRESOLVED {
  /** Registry stub whose sole version key is `range`; `latest` differs so the non-pinned branch returns an observably different value. */
  const pkg: NpmPackage = {
    versions: { [range]: {}, },
    'dist-tags': { latest: LATEST_SENTINEL, },
  };
  return resolveVersion({
    range,
    pkg,
  },);
}

/**
 Parses `raw` and narrows away {@link REPO_UNPARSEABLE} so positive parser
 assertions can read `.owner`/`.repo`/`.host` directly. Throws when the field
 does not parse, surfacing a fixture mistake instead of a silent `undefined`.
 
 @param raw - Raw `repository` field forwarded to {@link parseRepository}.
 
 @returns Parsed repository info.
 
 @throws When `raw` does not parse to a repository.
 */
function parsedRepo(raw: NpmVersion['repository'],): RepositoryInfo {
  /**
   Parse result; {@link REPO_UNPARSEABLE} here means the test fed an unparseable fixture.
   */
  const info = parseRepository(raw,);
  if (info === REPO_UNPARSEABLE)
    throw new Error(`expected a parseable repository, got REPO_UNPARSEABLE for ${JSON.stringify(raw,)}`,);
  return info;
}

await describe({
  name: '',
  children: [
    describe({
      name: parseRepository.name,
      children: [
        it({
          name: 'repo span ends at end of input when no delimiter follows',
          fn: async () => {
            expect(parsedRepo('https://github.com/owner/repo',).repo,).toBe('repo',);
          },
        },),

        it({
          name: 'repo span stops at a trailing-path slash',
          fn: async () => {
            expect(parsedRepo('https://github.com/owner/repo/tree/main',).repo,)
              .toBe('repo',);
          },
        },),

        it({
          name: 'repo span stops at a query-string question mark',
          fn: async () => {
            expect(parsedRepo('https://github.com/owner/repo?tab=readme',).repo,)
              .toBe('repo',);
          },
        },),

        it({
          name: 'repo span stops at a fragment hash',
          fn: async () => {
            expect(parsedRepo('https://github.com/owner/repo#readme',).repo,)
              .toBe('repo',);
          },
        },),

        it({
          name: 'strips a trailing .git when the span ends at end of input',
          fn: async () => {
            expect(parsedRepo('git+https://github.com/owner/repo.git',).repo,)
              .toBe('repo',);
          },
        },),

        it({
          name: 'scans past .git to a slash, then strips the .git suffix',
          fn: async () => {
            expect(parsedRepo('https://github.com/owner/repo.git/issues',).repo,)
              .toBe('repo',);
          },
        },),

        it({
          name: 'captures the owner alongside the repo span',
          fn: async () => {
            expect(parsedRepo('https://github.com/owner/repo#x',).owner,)
              .toBe('owner',);
          },
        },),

        it({
          name: 'an empty repo span from a trailing separator falls back to host other',
          fn: async () => {
            expect(parsedRepo('https://github.com/owner/',).host,).toBe('other',);
          },
        },),

        it({
          name: 'scans a long repeated run to end of input in one linear pass',
          fn: async () => {
            /** Repo name long enough that a recursive scan would overflow the stack on V8. */
            const longRepo = 'a'.repeat(LONG_RUN,);
            expect(parsedRepo(`https://github.com/owner/${longRepo}`,).repo,)
              .toBe(longRepo,);
          },
        },),

        it({
          name: 'stops a long repeated run at the first delimiter',
          fn: async () => {
            /** Repo name long enough to exercise the linear scan past the recursive depth ceiling. */
            const longRepo = 'a'.repeat(LONG_RUN,);
            expect(parsedRepo(`https://github.com/owner/${longRepo}#frag`,).repo,)
              .toBe(longRepo,);
          },
        },),
      ],
    },),

    describe({
      name: resolveVersion.name,
      children: [
        it({
          name: 'treats an empty range as not pinned',
          fn: async () => {
            expect(pinnedOrLatest('',),).toBe(LATEST_SENTINEL,);
          },
        },),

        it({
          name: 'treats an all-whitespace range as not pinned',
          fn: async () => {
            expect(pinnedOrLatest('   ',),).toBe(LATEST_SENTINEL,);
          },
        },),

        it({
          name: 'treats a non-numeric range as not pinned',
          fn: async () => {
            expect(pinnedOrLatest('abc',),).toBe(LATEST_SENTINEL,);
          },
        },),

        it({
          name: 'treats a leading-operator range as not pinned',
          fn: async () => {
            expect(pinnedOrLatest('^1.0.0',),).toBe(LATEST_SENTINEL,);
          },
        },),

        it({
          name: 'accepts a canonical major.minor.patch as pinned',
          fn: async () => {
            expect(pinnedOrLatest('1.2.3',),).toBe('1.2.3',);
          },
        },),

        it({
          name: 'accepts an all-zero version as pinned',
          fn: async () => {
            expect(pinnedOrLatest('0.0.0',),).toBe('0.0.0',);
          },
        },),

        it({
          name: 'accepts multi-digit runs in each segment',
          fn: async () => {
            expect(pinnedOrLatest('10.20.30',),).toBe('10.20.30',);
          },
        },),

        it({
          name: 'rejects a single-segment range with no minor or patch',
          fn: async () => {
            expect(pinnedOrLatest('1',),).toBe(LATEST_SENTINEL,);
          },
        },),

        it({
          name: 'rejects a two-segment range with no patch',
          fn: async () => {
            expect(pinnedOrLatest('1.2',),).toBe(LATEST_SENTINEL,);
          },
        },),

        it({
          name: 'rejects a trailing dot with no patch digits',
          fn: async () => {
            expect(pinnedOrLatest('1.2.',),).toBe(LATEST_SENTINEL,);
          },
        },),

        it({
          name: 'accepts a prerelease suffix because only the prefix is scanned',
          fn: async () => {
            expect(pinnedOrLatest('1.2.3-beta.1',),).toBe('1.2.3-beta.1',);
          },
        },),

        it({
          name: 'accepts a fourth version segment because only the prefix is scanned',
          fn: async () => {
            expect(pinnedOrLatest('1.2.3.4',),).toBe('1.2.3.4',);
          },
        },),

        it({
          name: 'scans a long digit run then rejects when no dot follows it',
          fn: async () => {
            expect(
              pinnedOrLatest('1'.repeat(LONG_RUN,),),
            ).toBe(LATEST_SENTINEL,);
          },
        },),

        it({
          name: 'accepts a pinned version whose major run is long, in one linear pass',
          fn: async () => {
            /** Range whose major segment is long enough to overflow a recursive digit scan on V8. */
            const longRange = `${'1'.repeat(LONG_RUN,)}.0.0`;
            expect(pinnedOrLatest(longRange,),).toBe(longRange,);
          },
        },),
      ],
    },),
  ],
},);
