/**
 Tests for the two failures an operator fixes by installing something: the
 linter program is missing, or it is another version than the one pinned.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  LINT_BINARY_NAME,
  LintBinaryMissingError,
  LintBinaryVersionError,
  PINNED_LINT_BINARY_VERSION,
  UNREADABLE_ANSWER,
  versionClause,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: versionClause.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES A VERSION TOKEN, and says an answer that held none could not be read',
          fn: async () => {
            expect({
              token: versionClause({ found: '0.1.1', },),
              unreadable: versionClause({ found: UNREADABLE_ANSWER, },),
            },).toEqual({
              token: 'version 0.1.1',
              unreadable: 'not a version this package can read',
            },);
          },
        },),
      ],
    },),

    describe({
      name: LintBinaryMissingError.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE PROGRAM, THE PINNED VERSION AND EVERY WAY TO INSTALL IT, and declares itself safe to forward',
          fn: async () => {
            const error = new LintBinaryMissingError({ binary: 'purr-lint', },);

            expect({
              name: error.name,
              marked: error.messageNamesOnly,
              isError: Error.isError(error,),
            },).toEqual({
              name: 'LintBinaryMissingError',
              marked: true,
              isError: true,
            },);
            expect(error.message,).toContain('purr-lint was not found on the PATH',);
            expect(error.message,).toContain('"mise run toolchain:sync"',);
            expect(error.message,).toContain(`"cargo:${LINT_BINARY_NAME}" in toolchain.lock.json`,);
            expect(error.message,).toContain(`"cargo binstall ${LINT_BINARY_NAME}@${PINNED_LINT_BINARY_VERSION}"`,);
            expect(error.message,).toContain('under "mise run" or with the tree\'s ".cache/toolchain-bin" directory',);
            // THE WORDS OF BEFORE THE TOOL MIGRATION stay out: mise installs no tool now.
            expect(error.message,).not.toContain('mise install',);
            expect(error.message,).not.toContain('mise.toml',);
            expect(error.message,).not.toContain('mise exec',);
          },
        },),
      ],
    },),

    describe({
      name: LintBinaryVersionError.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE VERSION FOUND AND THE ONE PINNED with the same advice, and SAYS SO where the answer '
            + 'held no version to repeat',
          fn: async () => {
            const found = new LintBinaryVersionError({ binary: 'purr-lint', found: '0.1.1', },);
            const unreadable = new LintBinaryVersionError({ binary: 'purr-lint', found: UNREADABLE_ANSWER, },);

            expect({
              name: found.name,
              marked: found.messageNamesOnly,
            },).toEqual({
              name: 'LintBinaryVersionError',
              marked: true,
            },);
            expect(found.message,).toContain('purr-lint is version 0.1.1, but',);
            expect(found.message,).toContain(`wrapped by version ${PINNED_LINT_BINARY_VERSION} only`,);
            expect(found.message,).toContain('"mise run toolchain:sync"',);
            expect(found.message,).not.toContain('mise install',);
            expect(unreadable.message,).toContain('purr-lint is not a version this package can read, but',);
          },
        },),
      ],
    },),
  ],
},);
