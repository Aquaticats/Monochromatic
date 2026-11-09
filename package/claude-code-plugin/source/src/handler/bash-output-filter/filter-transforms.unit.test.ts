import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  ALT_CWD_PREFIX,
  CWD_PREFIX,
  HOME_DIR,
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
  ],
},);
