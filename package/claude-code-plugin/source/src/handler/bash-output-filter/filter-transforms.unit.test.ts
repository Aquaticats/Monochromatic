import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  ALT_CWD_PREFIX,
  CWD_PREFIX,
  DEDUP_THRESHOLD,
  HOME_DIR,
  MAX_LINE_LENGTH,
  MAX_REPEATED_CHARS,
  PATH_PREFIX_ABSENT,
  REAL_HOME_DIR,
} from './filter-patterns.ts';
import {
  COLLAPSE_THRESHOLD,
  collapseCwdPaths,
  collapseHomePaths,
  collapseLineInitialPrefix,
  collapseRepeatedChars,
  flushRepeated,
  shouldStripLine,
  truncateLine,
} from './filter-transforms.ts';

await describe({
  name: 'filter-transforms',
  children: [
    describe({
      name: collapseRepeatedChars.name,
      children: [
        it({
          name: 'returns the empty string unchanged',
          fn: async () => {
            expect(collapseRepeatedChars('',),).toBe('',);
          },
        },),
        it({
          name: 'leaves word characters untouched even when repeated',
          fn: async () => {
            expect(collapseRepeatedChars('aaaaaaaaaaaa',),).toBe('aaaaaaaaaaaa',);
          },
        },),
        it({
          name: 'leaves whitespace runs untouched (not collapse candidates)',
          fn: async () => {
            expect(collapseRepeatedChars('          ',),).toBe('          ',);
          },
        },),
        it({
          name: 'leaves a run shorter than the threshold verbatim',
          fn: async () => {
            const shortRun = '='.repeat(COLLAPSE_THRESHOLD - 1,);
            expect(collapseRepeatedChars(shortRun,),).toBe(shortRun,);
          },
        },),
        it({
          name: 'collapses a run at the threshold',
          fn: async () => {
            /**
             Run of exactly the collapse threshold, built once so the
             expectation stays readable.
             */
            const atThreshold = '='.repeat(COLLAPSE_THRESHOLD,);
            expect(collapseRepeatedChars(atThreshold,),).toBe(
              `${'='.repeat(MAX_REPEATED_CHARS,)} (x${String(COLLAPSE_THRESHOLD,)} repeated characters)`,
            );
          },
        },),
        it({
          name: 'collapses a long run and preserves surrounding text',
          fn: async () => {
            expect(
              collapseRepeatedChars(`ab${'='.repeat(COLLAPSE_THRESHOLD,)}cd`,),
            ).toBe(
              `ab${'='.repeat(MAX_REPEATED_CHARS,)} (x${String(COLLAPSE_THRESHOLD,)} repeated characters)cd`,
            );
          },
        },),
        it({
          name: 'collapses multiple distinct runs independently',
          fn: async () => {
            const count = `(x${String(COLLAPSE_THRESHOLD,)} repeated characters)`;
            expect(
              collapseRepeatedChars(
                `${'+'.repeat(COLLAPSE_THRESHOLD,)} and ${'*'.repeat(COLLAPSE_THRESHOLD,)}`,
              ),
            ).toBe(
              `${'+'.repeat(MAX_REPEATED_CHARS,)} ${count} and ${'*'.repeat(MAX_REPEATED_CHARS,)} ${count}`,
            );
          },
        },),
        it({
          name: 'treats adjacent runs of different characters separately',
          fn: async () => {
            // Both runs stay below the threshold, so both are emitted verbatim.
            const shortRun = COLLAPSE_THRESHOLD - 1;
            expect(
              collapseRepeatedChars(`${'='.repeat(shortRun,)}${'-'.repeat(shortRun,)}`,),
            ).toBe(`${'='.repeat(shortRun,)}${'-'.repeat(shortRun,)}`,);
          },
        },),
        it({
          name: 'collapses a very long run without overflowing (linear pass)',
          fn: async () => {
            expect(
              collapseRepeatedChars('='.repeat(1_000_000,),),
            ).toBe(
              `${'='.repeat(MAX_REPEATED_CHARS,)} (x1000000 repeated characters)`,
            );
          },
        },),
      ],
    },),

    describe({
      name: collapseLineInitialPrefix.name,
      children: [
        it({
          name: 'prefers the longer of two matching prefixes',
          fn: async () => {
            expect(
              collapseLineInitialPrefix({
                line: '/repo/worktree/src/index.ts',
                prefixes: ['/repo/', '/repo/worktree/',],
                replacement: '',
              },),
            ).toBe('src/index.ts',);
          },
        },),
        it({
          name: 'prefers the longer prefix regardless of the order given',
          fn: async () => {
            expect(
              collapseLineInitialPrefix({
                line: '/repo/worktree/src/index.ts',
                prefixes: ['/repo/worktree/', '/repo/',],
                replacement: '',
              },),
            ).toBe('src/index.ts',);
          },
        },),
        it({
          name: 'ignores an absent prefix sentinel instead of matching it',
          fn: async () => {
            expect(
              collapseLineInitialPrefix({
                line: '/repo/src/index.ts',
                prefixes: [PATH_PREFIX_ABSENT, '/repo/',],
                replacement: '',
              },),
            ).toBe('src/index.ts',);
          },
        },),
        it({
          name: 'returns the line unchanged when no prefix matches',
          fn: async () => {
            expect(
              collapseLineInitialPrefix({
                line: 'error at /repo/src/index.ts',
                prefixes: ['/repo/',],
                replacement: '',
              },),
            ).toBe('error at /repo/src/index.ts',);
          },
        },),
        it({
          name: 'returns the line unchanged when every prefix is absent',
          fn: async () => {
            expect(
              collapseLineInitialPrefix({
                line: '/repo/src/index.ts',
                prefixes: [PATH_PREFIX_ABSENT,],
                replacement: '',
              },),
            ).toBe('/repo/src/index.ts',);
          },
        },),
      ],
    },),

    describe({
      name: collapseCwdPaths.name,
      children: [
        it({
          name: 'collapses a line-initial working-directory path to a relative path',
          fn: async () => {
            if (CWD_PREFIX === PATH_PREFIX_ABSENT) {
              // No working directory was resolvable here; the contract is to leave the line alone.
              expect(collapseCwdPaths('/repo/src/index.ts',),)
                .toBe('/repo/src/index.ts',);
              return;
            }
            expect(collapseCwdPaths(`${CWD_PREFIX}src/index.ts`,),)
              .toBe('src/index.ts',);
          },
        },),
        it({
          name: 'collapses the alternate working-directory spelling',
          fn: async () => {
            if (ALT_CWD_PREFIX === PATH_PREFIX_ABSENT) {
              // The working directory has one spelling here, so only the primary prefix can match.
              expect(collapseCwdPaths('/repo/src/index.ts',),)
                .toBe('/repo/src/index.ts',);
              return;
            }
            expect(collapseCwdPaths(`${ALT_CWD_PREFIX}src/index.ts`,),)
              .toBe('src/index.ts',);
          },
        },),
        it({
          name: 'leaves a mid-line working-directory path alone',
          fn: async () => {
            if (CWD_PREFIX === PATH_PREFIX_ABSENT) {
              expect(collapseCwdPaths('/repo/src/index.ts',),)
                .toBe('/repo/src/index.ts',);
              return;
            }
            expect(collapseCwdPaths(`error at ${CWD_PREFIX}src/index.ts`,),)
              .toBe(`error at ${CWD_PREFIX}src/index.ts`,);
          },
        },),
        it({
          name: 'leaves a prefix-adjacent sibling directory alone',
          fn: async () => {
            if (CWD_PREFIX === PATH_PREFIX_ABSENT) {
              expect(collapseCwdPaths('/repo-other/file.ts',),)
                .toBe('/repo-other/file.ts',);
              return;
            }
            /**
             Working-directory prefix without its trailing separator, so a
             sibling directory can be built that only shares the leading path.
             */
            const withoutTrailingSep = CWD_PREFIX.endsWith('/',)
              ? CWD_PREFIX.slice(
                0,
                -1,
              )
              : CWD_PREFIX;
            expect(collapseCwdPaths(`${CWD_PREFIX}../sibling/file.ts`,),)
              .toBe('../sibling/file.ts',);
            expect(collapseCwdPaths(`${withoutTrailingSep}-other/file.ts`,),)
              .toBe(`${withoutTrailingSep}-other/file.ts`,);
          },
        },),
      ],
    },),

    describe({
      name: collapseHomePaths.name,
      children: [
        it({
          name: 'collapses a line-initial home path to the tilde shorthand',
          fn: async () => {
            if (HOME_DIR === PATH_PREFIX_ABSENT) {
              expect(collapseHomePaths('/home/alice/notes.md',),)
                .toBe('/home/alice/notes.md',);
              return;
            }
            expect(collapseHomePaths(`${HOME_DIR}/notes.md`,),)
              .toBe('~/notes.md',);
          },
        },),
        it({
          name: 'collapses the symlink-resolved home spelling',
          fn: async () => {
            if (REAL_HOME_DIR === PATH_PREFIX_ABSENT) {
              // `$HOME` is already canonical, so there is no second spelling to match.
              expect(collapseHomePaths('/home/alice/notes.md',),)
                .toBe('/home/alice/notes.md',);
              return;
            }
            expect(collapseHomePaths(`${REAL_HOME_DIR}/notes.md`,),)
              .toBe('~/notes.md',);
          },
        },),
        it({
          name: 'leaves a mid-line home path alone',
          fn: async () => {
            if (HOME_DIR === PATH_PREFIX_ABSENT) {
              expect(collapseHomePaths('/home/alice/notes.md',),)
                .toBe('/home/alice/notes.md',);
              return;
            }
            expect(collapseHomePaths(`read ${HOME_DIR}/notes.md`,),)
              .toBe(`read ${HOME_DIR}/notes.md`,);
          },
        },),
      ],
    },),

    describe({
      name: truncateLine.name,
      children: [
        it({
          name: 'returns an empty line unchanged',
          fn: async () => {
            expect(truncateLine('',),).toBe('',);
          },
        },),
        it({
          name: 'returns a line exactly at the limit unchanged',
          fn: async () => {
            /**
             Line whose length is exactly the truncation limit.
             */
            const atLimit = 'x'.repeat(MAX_LINE_LENGTH,);
            expect(truncateLine(atLimit,),).toBe(atLimit,);
          },
        },),
        it({
          name: 'truncates a line one character over the limit and reports its length',
          fn: async () => {
            /**
             Line one character longer than the truncation limit.
             */
            const overLimit = 'x'.repeat(MAX_LINE_LENGTH + 1,);
            expect(truncateLine(overLimit,),)
              .toBe(`${'x'.repeat(MAX_LINE_LENGTH,)}... [${String(MAX_LINE_LENGTH + 1,)} chars]`,);
          },
        },),
      ],
    },),

    describe({
      name: flushRepeated.name,
      children: [
        it({
          name: 'emits nothing for a zero-count run',
          fn: async () => {
            expect(flushRepeated({
              line: 'foo',
              count: 0,
            },),)
              .toEqual([],);
          },
        },),
        it({
          name: 'emits the line once for a single occurrence',
          fn: async () => {
            expect(flushRepeated({
              line: 'foo',
              count: 1,
            },),)
              .toEqual(['foo',],);
          },
        },),
        it({
          name: 'still emits every copy one below the dedup threshold',
          fn: async () => {
            expect(flushRepeated({
              line: 'bar',
              count: DEDUP_THRESHOLD - 1,
            },),)
              .toEqual(Array.from(
                { length: DEDUP_THRESHOLD - 1, },
                function repeatBar(): string {
                  return 'bar';
                },
              ),);
          },
        },),
        it({
          name: 'collapses a run that reaches the dedup threshold',
          fn: async () => {
            expect(flushRepeated({
              line: 'baz',
              count: DEDUP_THRESHOLD,
            },),)
              .toEqual([`baz (x${String(DEDUP_THRESHOLD,)} repeated lines)`,],);
          },
        },),
        it({
          name: 'collapses a long run and reports its real count',
          fn: async () => {
            /**
             Run longer than the threshold, so the marker carries a two-digit count.
             */
            const longRunCount = DEDUP_THRESHOLD * 4;
            expect(flushRepeated({
              line: '',
              count: longRunCount,
            },),)
              .toEqual([` (x${String(longRunCount,)} repeated lines)`,],);
          },
        },),
      ],
    },),

    describe({
      name: shouldStripLine.name,
      children: [
        it({
          name: 'strips a git per-file mode line',
          fn: async () => {
            expect(shouldStripLine(' create mode 100644 foo.ts',),).toBe(true,);
          },
        },),
        it({
          name: 'strips a git transport progress line',
          fn: async () => {
            expect(shouldStripLine('Counting objects: 100% (4/4), done.',),).toBe(true,);
          },
        },),
        it({
          name: 'strips a git total summary line',
          fn: async () => {
            expect(shouldStripLine('Total 1234 (delta 567)',),).toBe(true,);
          },
        },),
        it({
          name: 'strips a remote transport progress line',
          fn: async () => {
            expect(shouldStripLine('remote: Compressing objects: 100% (3/3)',),).toBe(true,);
          },
        },),
        it({
          name: 'strips the sandbox mise cache-write warning',
          fn: async () => {
            expect(
              shouldStripLine('mise WARN failed to write cache file: /cache/x: Read-only file system',),
            ).toBe(true,);
          },
        },),
        it({
          name: 'strips a mise bootstrap command echo',
          fn: async () => {
            expect(shouldStripLine('[//:bootstrap] $ mise install',),).toBe(true,);
          },
        },),
        it({
          name: 'strips a mise no-op tool summary',
          fn: async () => {
            expect(shouldStripLine('mise All tools are up to date',),).toBe(true,);
          },
        },),
        it({
          name: 'strips a mise minimum-release-age warning',
          fn: async () => {
            expect(
              shouldStripLine('mise WARN  newer pnpm release 11.12.0 ignored by minimum_release_age (24h)',),
            ).toBe(true,);
          },
        },),
        it({
          name: 'keeps a line no predicate claims',
          fn: async () => {
            expect(shouldStripLine('hello world',),).toBe(false,);
          },
        },),
        it({
          name: 'keeps an empty line',
          fn: async () => {
            expect(shouldStripLine('',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
